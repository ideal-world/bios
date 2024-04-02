use std::collections::HashMap;

use bios_basic::rbum::dto::rbum_set_item_dto::RbumSetItemDetailResp;
use bios_basic::rbum::helper::rbum_event_helper;
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_set_serv::RbumSetItemServ;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::{param::Path, param::Query, payload::Json};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumItemRelFilterReq, RbumSetCateFilterReq, RbumSetItemFilterReq, RbumSetItemRelFilterReq};
use bios_basic::rbum::rbum_enumeration::{RbumRelFromKind, RbumSetCateLevelQueryKind};
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_account_dto::{IamAccountAggAddReq, IamAccountAggModifyReq, IamAccountDetailAggResp, IamAccountModifyReq, IamAccountSummaryAggResp};
use crate::basic::dto::iam_filer_dto::IamAccountFilterReq;
use crate::basic::serv::clients::iam_search_client::IamSearchClient;
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::iam_config::IamBasicConfigApi;
use crate::iam_constants;
use crate::iam_enumeration::{IamAccountLockStateKind, IamAccountStatusKind, IamRelKind, IamSetKind};
use bios_basic::helper::request_helper::add_remote_ip;
use tardis::web::poem::Request;
#[derive(Clone, Default)]
pub struct IamCtAccountApi;

/// Tenant Console Account API
#[poem_openapi::OpenApi(prefix_path = "/ct/account", tag = "bios_basic::ApiTag::Tenant")]
impl IamCtAccountApi {
    /// Add Account
    #[oai(path = "/", method = "post")]
    async fn add(&self, app_id: Query<Option<String>>, add_req: Json<IamAccountAggAddReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<String> {
        let ctx = IamCertServ::try_use_app_ctx(ctx.0, app_id.0)?;
        add_remote_ip(request, &ctx).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let result = IamAccountServ::add_account_agg(&add_req.0, false, &funs, &ctx).await?;
        IamSearchClient::async_add_or_modify_account_search(result.clone(), Box::new(false), "".to_string(), &funs, &ctx).await?;
        // rbum_event_helper::get_notify_event_with_ctx(&funs, &ctx).await?;
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Modify Account
    #[oai(path = "/:id", method = "put")]
    async fn modify(
        &self,
        id: Path<String>,
        app_id: Query<Option<String>>,
        modify_req: Json<IamAccountAggModifyReq>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Void> {
        let ctx = IamCertServ::try_use_app_ctx(ctx.0, app_id.0)?;
        add_remote_ip(request, &ctx).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamAccountServ::modify_account_agg(&id.0, &modify_req.0, &funs, &ctx).await?;
        IamSearchClient::async_add_or_modify_account_search(id.0, Box::new(true), "".to_string(), &funs, &ctx).await?;
        funs.commit().await?;
        ctx.execute_task().await?;
        if let Some(notify_events) = rbum_event_helper::get_notify_event_with_ctx(&ctx).await? {
            rbum_event_helper::try_notifies(notify_events, &iam_constants::get_tardis_inst(), &ctx).await?;
        }
        TardisResp::ok(Void {})
    }

    /// Get Account
    #[oai(path = "/:id", method = "get")]
    async fn get(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<IamAccountDetailAggResp> {
        add_remote_ip(request, &ctx.0).await?;
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
            false,
            true,
            &funs,
            &ctx.0,
        )
        .await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find Accounts
    #[oai(path = "/", method = "get")]
    #[allow(clippy::too_many_arguments)]
    async fn paginate(
        &self,
        ids: Query<Option<String>>,
        name: Query<Option<String>>,
        role_ids: Query<Option<String>>,
        app_ids: Query<Option<String>>,
        cate_ids: Query<Option<String>>,
        status: Query<Option<bool>>,
        app_id: Query<Option<String>>,
        with_sub: Query<Option<bool>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<TardisPage<IamAccountSummaryAggResp>> {
        let ctx = IamCertServ::try_use_app_ctx(ctx.0, app_id.0)?;
        add_remote_ip(request, &ctx).await?;
        let funs = iam_constants::get_tardis_inst();
        let rel = role_ids.0.map(|role_ids| {
            let role_ids = role_ids.split(',').map(|r| r.to_string()).collect::<Vec<_>>();
            RbumItemRelFilterReq {
                rel_by_from: true,
                tag: Some(IamRelKind::IamAccountRole.to_string()),
                from_rbum_kind: Some(RbumRelFromKind::Item),
                rel_item_ids: Some(role_ids),
                own_paths: Some(ctx.own_paths.clone()),
                ..Default::default()
            }
        });
        let rel2 = app_ids.0.map(|app_ids| {
            let app_ids = app_ids.split(',').map(|r| r.to_string()).collect::<Vec<_>>();
            RbumItemRelFilterReq {
                rel_by_from: true,
                tag: Some(IamRelKind::IamAccountApp.to_string()),
                from_rbum_kind: Some(RbumRelFromKind::Item),
                rel_item_ids: Some(app_ids),
                own_paths: Some(ctx.own_paths.clone()),
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
                rel2,
                set_rel,
                ..Default::default()
            },
            false,
            true,
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

    /// Delete Account
    #[oai(path = "/:id", method = "delete")]
    async fn delete(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        add_remote_ip(request, &ctx.0).await?;
        funs.begin().await?;
        IamAccountServ::delete_item_with_all_rels(&id.0, &funs, &ctx.0).await?;
        IamSearchClient::async_delete_account_search(id.0, &funs, ctx.0.clone()).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Delete Token By Account Id
    #[oai(path = "/:id/token", method = "delete")]
    async fn offline(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        add_remote_ip(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamAccountServ::delete_tokens(&id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Count Accounts
    #[oai(path = "/total", method = "get")]
    async fn count(&self, app_id: Query<Option<String>>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<u64> {
        let ctx = IamCertServ::try_use_app_ctx(ctx.0, app_id.0)?;
        add_remote_ip(request, &ctx).await?;
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

    /// Active account
    #[oai(path = "/:id/active", method = "put")]
    async fn active_account(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        add_remote_ip(request, &ctx.0).await?;
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
            },
            &funs,
            &ctx.0,
        )
        .await?;
        IamSearchClient::async_add_or_modify_account_search(id.0, Box::new(true), "".to_string(), &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Logout account
    #[oai(path = "/:id/logout", method = "put")]
    async fn logout_account(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        add_remote_ip(request, &ctx.0).await?;
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
            },
            &funs,
            &ctx.0,
        )
        .await?;
        IamSearchClient::async_add_or_modify_account_search(id.0, Box::new(true), "Manual cancellation.".to_string(), &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    ///lock account
    #[oai(path = "/:id/lock", method = "put")]
    async fn lock_account(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        add_remote_ip(request, &ctx.0).await?;
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
            },
            &funs,
            &ctx.0,
        )
        .await?;
        IamSearchClient::async_add_or_modify_account_search(id.0, Box::new(true), "".to_string(), &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    ///unlock account
    #[oai(path = "/:id/unlock", method = "post")]
    async fn unlock_account(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        add_remote_ip(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamAccountServ::unlock_account(&id.0, &funs, &ctx.0).await?;
        IamSearchClient::async_add_or_modify_account_search(id.0, Box::new(true), "".to_string(), &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// paginate App Set Items (account)
    #[oai(path = "/apps/item", method = "get")]
    async fn paginate_items(
        &self,
        cate_sys_codes: Query<Option<String>>,
        sys_code_query_kind: Query<Option<RbumSetCateLevelQueryKind>>,
        sys_code_query_depth: Query<Option<i16>>,
        cate_ids: Query<Option<String>>,
        item_ids: Query<Option<String>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<TardisPage<RbumSetItemDetailResp>> {
        let funs = iam_constants::get_tardis_inst();
        let ctx = IamCertServ::use_sys_or_tenant_ctx_unsafe(ctx.0)?;
        add_remote_ip(request, &ctx).await?;
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Apps, &funs, &ctx).await?;
        let result = RbumSetItemServ::paginate_detail_rbums(
            &RbumSetItemFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: false,
                    ..Default::default()
                },
                rel_rbum_item_disabled: Some(false),
                table_rbum_set_cate_is_left: Some(true),
                rel_rbum_set_id: Some(set_id.clone()),
                rel_rbum_item_kind_ids: Some(vec![funs.iam_basic_kind_account_id()]),
                rel_rbum_set_cate_sys_codes: cate_sys_codes.0.map(|codes| codes.split(',').map(|code| code.to_string()).collect::<Vec<String>>()),
                sys_code_query_kind: sys_code_query_kind.0,
                sys_code_query_depth: sys_code_query_depth.0,
                rel_rbum_set_cate_ids: cate_ids.0.map(|ids| ids.split(',').map(|id| id.to_string()).collect::<Vec<String>>()),
                rel_rbum_item_ids: item_ids.0.map(|ids| ids.split(',').map(|id| id.to_string()).collect::<Vec<String>>()),
                ..Default::default()
            },
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
}
