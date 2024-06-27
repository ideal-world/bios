use std::collections::HashMap;

use bios_basic::process::task_processor::TaskProcessor;
use bios_basic::rbum::helper::rbum_event_helper;

use tardis::basic::dto::TardisContext;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::{param::Path, param::Query, payload::Json};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumItemRelFilterReq, RbumSetCateFilterReq, RbumSetItemRelFilterReq};
use bios_basic::rbum::rbum_enumeration::RbumRelFromKind;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_account_dto::{
    AccountTenantInfoResp, IamAccountAggAddReq, IamAccountAggModifyReq, IamAccountDetailAggResp, IamAccountModifyReq, IamAccountSummaryAggResp,
};
use crate::basic::dto::iam_filer_dto::IamAccountFilterReq;
use crate::basic::serv::clients::iam_log_client::{IamLogClient, LogParamTag};
use crate::basic::serv::clients::iam_search_client::IamSearchClient;
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::iam_constants;
use crate::iam_enumeration::{IamAccountLockStateKind, IamAccountStatusKind, IamRelKind};
use bios_basic::helper::request_helper::try_set_real_ip_from_req_to_ctx;
use tardis::web::poem::Request;
#[derive(Clone, Default)]
pub struct IamCsAccountApi;

/// System Console Account API
/// 系统控制台账号API
#[poem_openapi::OpenApi(prefix_path = "/cs/account", tag = "bios_basic::ApiTag::System")]
impl IamCsAccountApi {
    /// Add Account By Tenant Id
    /// 添加账号
    #[oai(path = "/", method = "post")]
    async fn add(&self, tenant_id: Query<Option<String>>, add_req: Json<IamAccountAggAddReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<String> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let result = IamAccountServ::add_account_agg(&add_req.0, false, &funs, &ctx).await?;
        IamSearchClient::async_add_or_modify_account_search(&result, Box::new(false), "", &funs, &ctx).await?;
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Modify Account By Account Id
    /// 修改账号
    #[oai(path = "/:id", method = "put")]
    async fn modify(
        &self,
        id: Path<String>,
        tenant_id: Query<Option<String>>,
        modify_req: Json<IamAccountAggModifyReq>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Void> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamAccountServ::modify_account_agg(&id.0, &modify_req.0, &funs, &ctx).await?;
        IamSearchClient::async_add_or_modify_account_search(&id.0, Box::new(true), "", &funs, &ctx).await?;
        funs.commit().await?;
        ctx.execute_task().await?;
        if let Some(notify_events) = rbum_event_helper::get_notify_event_with_ctx(&ctx).await? {
            rbum_event_helper::try_notifies(notify_events, &iam_constants::get_tardis_inst(), &ctx).await?;
        }
        TardisResp::ok(Void {})
    }

    /// Get Account By Account Id
    /// 获取账号
    #[oai(path = "/:id", method = "get")]
    async fn get(&self, id: Path<String>, tenant_id: Query<Option<String>>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<IamAccountDetailAggResp> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0.clone())?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamAccountServ::get_account_detail_aggs(
            &id.0,
            &IamAccountFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            tenant_id.0.is_none(),
            tenant_id.0.is_none(),
            &funs,
            &ctx,
        )
        .await?;
        ctx.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find Accounts
    /// 查找账号
    #[oai(path = "/", method = "get")]
    #[allow(clippy::too_many_arguments)]
    async fn paginate(
        &self,
        ids: Query<Option<String>>,
        name: Query<Option<String>>,
        role_ids: Query<Option<String>>,
        cate_ids: Query<Option<String>>,
        status: Query<Option<bool>>,
        tenant_id: Query<Option<String>>,
        with_sub: Query<Option<bool>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<TardisPage<IamAccountSummaryAggResp>> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0.clone())?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let funs = iam_constants::get_tardis_inst();
        let rel = role_ids.0.map(|role_ids| {
            let role_ids = role_ids.split(',').map(|r| r.to_string()).collect::<Vec<_>>();
            RbumItemRelFilterReq {
                rel_by_from: true,
                tag: Some(IamRelKind::IamAccountRole.to_string()),
                from_rbum_kind: Some(RbumRelFromKind::Item),
                rel_item_ids: Some(role_ids),
                ..Default::default()
            }
        });
        let set_rel = if let Some(cate_ids) = cate_ids.0 {
            let cate_ids = cate_ids.split(',').map(|r| r.to_string()).collect::<Vec<_>>();
            let set_cate_vec = IamSetServ::find_set_cate(
                &RbumSetCateFilterReq {
                    basic: RbumBasicFilterReq {
                        own_paths: Some("".to_string()),
                        with_sub_own_paths: true,
                        ids: Some(cate_ids),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                None,
                None,
                &funs,
                &ctx,
            )
            .await?;
            Some(RbumSetItemRelFilterReq {
                set_ids_and_cate_codes: Some(
                    set_cate_vec.into_iter().map(|sc| (sc.rel_rbum_set_id, sc.sys_code)).fold(HashMap::new(), |mut acc, (key, value)| {
                        acc.entry(key).or_default().push(value);
                        acc
                    }),
                ),
                with_sub_set_cate_codes: false,
                ..Default::default()
            })
        } else {
            None
        };
        let result = IamAccountServ::paginate_account_summary_aggs(
            &IamAccountFilterReq {
                basic: RbumBasicFilterReq {
                    ids: ids.0.map(|ids| ids.split(',').map(|id| id.to_string()).collect::<Vec<String>>()),
                    name: name.0,
                    with_sub_own_paths: with_sub.0.unwrap_or(false),
                    enabled: status.0,
                    ..Default::default()
                },
                rel,
                set_rel,
                ..Default::default()
            },
            tenant_id.0.is_none(),
            tenant_id.0.is_none(),
            page_number.0,
            page_size.0,
            desc_by_create.0,
            desc_by_update.0,
            &funs,
            &ctx,
        )
        .await?;
        ctx.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Delete Account By Account Id
    /// 删除账号
    #[oai(path = "/:id", method = "delete")]
    async fn delete(&self, id: Path<String>, tenant_id: Query<Option<String>>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Option<String>> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamAccountServ::delete_item_with_all_rels(&id.0, &funs, &ctx).await?;
        IamSearchClient::async_delete_account_search(id.0, &funs, ctx.clone()).await?;
        funs.commit().await?;
        ctx.execute_task().await?;
        if let Some(task_id) = TaskProcessor::get_task_id_with_ctx(&ctx).await? {
            TardisResp::accepted(Some(task_id))
        } else {
            TardisResp::ok(None)
        }
    }

    /// Delete Token By Account Id
    /// 删除账号Token
    #[oai(path = "/:id/token", method = "delete")]
    async fn offline(&self, id: Path<String>, tenant_id: Query<Option<String>>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamAccountServ::delete_tokens(&id.0, &funs, &ctx).await?;
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Count Accounts By Tenant Id
    /// 统计账号
    #[oai(path = "/total", method = "get")]
    async fn count(&self, tenant_id: Query<Option<String>>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<u64> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamAccountServ::count_items(
            &IamAccountFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            &funs,
            &ctx,
        )
        .await?;
        ctx.execute_task().await?;
        TardisResp::ok(result)
    }

    ///Get account's tenant_info by account id
    /// 获取账号租户信息
    #[oai(path = "/:id/tenant", method = "get")]
    async fn get_account_tenant_info(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<AccountTenantInfoResp> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamAccountServ::get_account_tenant_info(&id.0, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Active account
    /// 激活账号
    #[oai(path = "/:id/active", method = "put")]
    async fn active_account(&self, id: Path<String>, tenant_id: Query<Option<String>>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamAccountServ::modify_item(
            &id.0,
            &mut IamAccountModifyReq {
                status: Some(IamAccountStatusKind::Active),
                is_auto: Some(false),
                name: None,
                icon: None,
                disabled: None,
                scope_level: None,
                lock_status: None,
                temporary: None,
                logout_type: None,
                labor_type: None,
            },
            &funs,
            &ctx,
        )
        .await?;
        IamSearchClient::add_or_modify_account_search(
            IamAccountServ::get_account_detail_aggs(
                &id.0,
                &IamAccountFilterReq {
                    basic: RbumBasicFilterReq {
                        ignore_scope: true,
                        own_paths: Some("".to_string()),
                        with_sub_own_paths: true,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                true,
                true,
                &funs,
                &TardisContext {
                    own_paths: "".to_string(),
                    ..ctx.clone()
                },
            )
            .await?,
            Box::new(true),
            "",
            &funs,
            &ctx,
        )
        .await?;
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Logout account
    /// 注销账号
    #[oai(path = "/:id/logout", method = "put")]
    async fn logout_account(&self, id: Path<String>, tenant_id: Query<Option<String>>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamAccountServ::modify_item(
            &id.0,
            &mut IamAccountModifyReq {
                status: Some(IamAccountStatusKind::Logout),
                is_auto: Some(false),
                name: None,
                icon: None,
                disabled: None,
                scope_level: None,
                lock_status: None,
                temporary: None,
                logout_type: Some(crate::iam_enumeration::IamAccountLogoutTypeKind::ArtificialLogout),
                labor_type: None,
            },
            &funs,
            &ctx,
        )
        .await?;
        IamSearchClient::async_add_or_modify_account_search(&id.0, Box::new(true), "Manual cancellation.", &funs, &ctx).await?;
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// lock account
    /// 锁定账号
    #[oai(path = "/:id/lock", method = "put")]
    async fn lock_account(&self, id: Path<String>, tenant_id: Query<Option<String>>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamAccountServ::modify_item(
            &id.0,
            &mut IamAccountModifyReq {
                lock_status: Some(IamAccountLockStateKind::ManualLocked),
                is_auto: None,
                name: None,
                icon: None,
                disabled: None,
                scope_level: None,
                status: None,
                temporary: None,
                logout_type: None,
                labor_type: None,
            },
            &funs,
            &ctx,
        )
        .await?;
        IamSearchClient::async_add_or_modify_account_search(&id.0, Box::new(true), "", &funs, &ctx).await?;
        let _ = IamLogClient::add_ctx_task(
            LogParamTag::IamAccount,
            Some(id.0.clone()),
            "人工锁定账号".to_string(),
            Some("ManuallyLockAccount".to_string()),
            &ctx,
        )
        .await;
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Unlock account
    /// 解锁账号
    #[oai(path = "/:id/unlock", method = "post")]
    async fn unlock_account(&self, id: Path<String>, tenant_id: Query<Option<String>>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamAccountServ::unlock_account(&id.0, &funs, &ctx).await?;
        IamSearchClient::async_add_or_modify_account_search(&id.0, Box::new(true), "", &funs, &ctx).await?;
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(Void {})
    }
}
