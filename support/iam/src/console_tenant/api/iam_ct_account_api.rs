use bios_basic::process::task_processor::TaskProcessor;
use bios_basic::rbum::helper::rbum_event_helper;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::{param::Path, param::Query, payload::Json};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumItemRelFilterReq, RbumSetCateFilterReq, RbumSetItemRelFilterReq};
use bios_basic::rbum::rbum_enumeration::RbumRelFromKind;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_account_dto::{IamAccountAggAddReq, IamAccountAggModifyReq, IamAccountDetailAggResp, IamAccountModifyReq, IamAccountSummaryAggResp};
use crate::basic::dto::iam_filer_dto::IamAccountFilterReq;
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::iam_constants;
use crate::iam_enumeration::{IamAccountLockStateKind, IamAccountStatusKind, IamRelKind};

pub struct IamCtAccountApi;

/// Tenant Console Account API
#[poem_openapi::OpenApi(prefix_path = "/ct/account", tag = "bios_basic::ApiTag::Tenant")]
impl IamCtAccountApi {
    /// Add Account  安全审计日志--添加长期账号、添加临时账号
    #[oai(path = "/", method = "post")]
    async fn add(&self, app_id: Query<Option<String>>, add_req: Json<IamAccountAggAddReq>, ctx: TardisContextExtractor) -> TardisApiResult<String> {
        let ctx = IamCertServ::try_use_app_ctx(ctx.0, app_id.0)?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let result = IamAccountServ::add_account_agg(&add_req.0, false, &funs, &ctx).await?;
        IamAccountServ::async_add_or_modify_account_search(result.clone(), false, "".to_string(), &funs, ctx).await?;
        // TaskProcessor::get_notify_event_with_ctx(&funs, &ctx).await?;
        funs.commit().await?;
        TardisResp::ok(result)
    }

    /// Modify Account  安全审计日志--修改账号头像、休眠账号、修改用户类型为临时账号、修改姓名
    #[oai(path = "/:id", method = "put")]
    async fn modify(&self, id: Path<String>, app_id: Query<Option<String>>, modify_req: Json<IamAccountAggModifyReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let ctx = IamCertServ::try_use_app_ctx(ctx.0, app_id.0)?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamAccountServ::modify_account_agg(&id.0, &modify_req.0, &funs, &ctx).await?;
        IamAccountServ::async_add_or_modify_account_search(id.0, true, "".to_string(), &funs, ctx.clone()).await?;
        funs.commit().await?;
        if let Some(notify_events) = TaskProcessor::get_notify_event_with_ctx(&ctx).await? {
            rbum_event_helper::try_notifies(notify_events, &iam_constants::get_tardis_inst(), &ctx).await?;
        }
        TardisResp::ok(Void {})
    }

    /// Get Account
    #[oai(path = "/:id", method = "get")]
    async fn get(&self, id: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<IamAccountDetailAggResp> {
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
    ) -> TardisApiResult<TardisPage<IamAccountSummaryAggResp>> {
        let ctx = IamCertServ::try_use_app_ctx(ctx.0, app_id.0)?;
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
                set_ids_and_cate_codes: Some(set_cate_vec.into_iter().map(|sc| (sc.rel_rbum_set_id, sc.sys_code)).collect()),
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
        TardisResp::ok(result)
    }

    /// Delete Account
    #[oai(path = "/:id", method = "delete")]
    async fn delete(&self, id: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamAccountServ::delete_item_with_all_rels(&id.0, &funs, &ctx.0).await?;
        IamAccountServ::async_delete_account_search(id.0, &funs, ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Delete Token By Account Id
    #[oai(path = "/:id/token", method = "delete")]
    async fn offline(&self, id: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamAccountServ::delete_tokens(&id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Count Accounts
    #[oai(path = "/total", method = "get")]
    async fn count(&self, app_id: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<u64> {
        let ctx = IamCertServ::try_use_app_ctx(ctx.0, app_id.0)?;
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
        TardisResp::ok(result)
    }

    /// Active account  安全审计日志--激活账号
    #[oai(path = "/:id/active", method = "put")]
    async fn active_account(&self, id: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
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
            },
            &funs,
            &ctx.0,
        )
        .await?;
        IamAccountServ::async_add_or_modify_account_search(id.0, true, "".to_string(), &funs, ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Logout account  安全审计日志--注销账号、下线账号
    #[oai(path = "/:id/logout", method = "put")]
    async fn logout_account(&self, id: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
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
            },
            &funs,
            &ctx.0,
        )
        .await?;
        IamAccountServ::async_add_or_modify_account_search(id.0, true, "Manual cancellation.".to_string(), &funs, ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    ///lock account  安全审计日志--人工锁定账号
    #[oai(path = "/:id/lock", method = "put")]
    async fn lock_account(&self, id: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
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
            },
            &funs,
            &ctx.0,
        )
        .await?;
        IamAccountServ::async_add_or_modify_account_search(id.0, true, "".to_string(), &funs, ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    ///unlock account  安全审计日志--解锁账号
    #[oai(path = "/:id/unlock", method = "post")]
    async fn unlock_account(&self, id: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamAccountServ::unlock_account(&id.0, &funs, &ctx.0).await?;
        IamAccountServ::async_add_or_modify_account_search(id.0, true, "".to_string(), &funs, ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }
}
