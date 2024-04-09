use crate::basic::dto::iam_filer_dto::IamRoleFilterReq;
use crate::basic::dto::iam_role_dto::{IamRoleRelAccountCertResp, IamRoleSummaryResp};
use bios_basic::rbum::helper::rbum_scope_helper::check_without_owner_and_unsafe_fill_ctx;
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_item_serv::{RbumItemCrudOperation, RbumItemServ};
use itertools::Itertools;
use tardis::log::error;

use crate::basic::serv::iam_app_serv::IamAppServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_role_serv::IamRoleServ;
use crate::iam_config::IamBasicConfigApi;
use crate::iam_constants::{self, RBUM_SCOPE_LEVEL_APP};
use bios_basic::helper::request_helper::add_remote_ip;
use bios_basic::process::task_processor::TaskProcessor;
use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumCertFilterReq};
use tardis::tokio;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::{Path, Query};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

#[derive(Clone, Default)]
pub struct IamCiRoleApi;

/// # Interface Console Manage Cert API
///
/// Allow Management Of aksk (an authentication method between applications)
#[poem_openapi::OpenApi(prefix_path = "/ci/role", tag = "bios_basic::ApiTag::Interface")]
impl IamCiRoleApi {
    #[oai(path = "/verify/tenant/admin", method = "get")]
    async fn get_verify_role_tenant_admin(&self, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<bool> {
        let funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        add_remote_ip(request, &ctx.0).await?;
        let mut verify_tenant_admin = false;
        for role in &ctx.0.roles {
            if role.contains(&funs.iam_basic_role_tenant_admin_id()) {
                verify_tenant_admin = true;
            }
        }
        TardisResp::ok(verify_tenant_admin)
    }

    /// Batch add Role Rel Account
    #[oai(path = "/:id/account/batch/:account_ids", method = "put")]
    async fn batch_add_rel_account(&self, id: Path<String>, account_ids: Path<String>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        add_remote_ip(request, &ctx.0).await?;
        funs.begin().await?;
        let app_id = IamAppServ::get_id_by_ctx(&ctx.0, &funs)?;
        let split = account_ids.0.split(',').collect::<Vec<_>>();
        for s in split {
            IamAppServ::add_rel_account(&app_id, s, true, &funs, &ctx.0).await?;
            IamRoleServ::add_rel_account(&id.0, s, Some(RBUM_SCOPE_LEVEL_APP), &funs, &ctx.0).await?;
        }
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Batch delete Role Rel Account
    #[oai(path = "/:id/account/batch/:account_ids", method = "delete")]
    async fn batch_delete_rel_account(&self, id: Path<String>, account_ids: Path<String>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        add_remote_ip(request, &ctx.0).await?;
        funs.begin().await?;
        let split = account_ids.0.split(',').collect::<Vec<_>>();
        for s in split {
            IamRoleServ::delete_rel_account(&id.0, s, Some(RBUM_SCOPE_LEVEL_APP), &funs, &ctx.0).await?;
        }
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Delete Role Rel Account
    #[oai(path = "/:id/account/:account_id", method = "delete")]
    async fn delete_rel_account(&self, id: Path<String>, account_id: Path<String>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        add_remote_ip(request, &ctx.0).await?;
        funs.begin().await?;
        IamRoleServ::delete_rel_account(&id.0, &account_id.0, Some(RBUM_SCOPE_LEVEL_APP), &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Add Role Rel Account
    #[oai(path = "/:id/apps/account/batch", method = "put")]
    async fn batch_add_apps_rel_account(
        &self,
        id: Path<String>,
        app_ids: Query<String>,
        account_ids: Query<String>,
        mut ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Option<String>> {
        let funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        add_remote_ip(request, &ctx.0).await?;
        let ctx = ctx.0;
        let ctx_clone = ctx.clone();
        ctx.add_async_task(Box::new(|| {
            Box::pin(async move {
                let task_handle = tokio::spawn(async move {
                    let mut funs = iam_constants::get_tardis_inst();
                    if let Err(err) = funs.begin().await {
                        error!("[IAM] batch_add_apps_rel_account begin error: {:?}", err);
                        return;
                    }
                    let apps_split: Vec<&str> = app_ids.0.split(',').collect::<Vec<_>>();
                    let account_split: Vec<&str> = account_ids.0.split(',').collect::<Vec<_>>();
                    for app_id in apps_split {
                        let mock_app_ctx = IamCertServ::try_use_app_ctx(ctx_clone.clone(), Some(app_id.to_string())).unwrap_or(ctx_clone.clone());
                        for account_id in account_split.clone() {
                            let _ = IamAppServ::add_rel_account(app_id, account_id, true, &funs, &mock_app_ctx).await;
                            let _ = IamRoleServ::add_rel_account(&id.0, account_id, Some(RBUM_SCOPE_LEVEL_APP), &funs, &mock_app_ctx).await;
                        }
                    }
                    if let Err(err) = funs.commit().await {
                        error!("[IAM] batch_add_apps_rel_account commit error: {:?}", err);
                    }
                });
                task_handle.await.unwrap();
                Ok(())
            })
        }))
        .await?;
        ctx.execute_task().await?;
        if let Some(task_id) = TaskProcessor::get_task_id_with_ctx(&ctx).await? {
            TardisResp::accepted(Some(task_id))
        } else {
            TardisResp::ok(None)
        }
    }

    /// Add Role Rel Account
    #[oai(path = "/:id/apps/account/batch", method = "delete")]
    async fn batch_delete_apps_rel_account(
        &self,
        id: Path<String>,
        app_ids: Query<String>,
        account_ids: Query<String>,
        mut ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        add_remote_ip(request, &ctx.0).await?;
        let ctx = ctx.0;
        funs.begin().await?;
        let apps_split: Vec<&str> = app_ids.0.split(',').collect::<Vec<_>>();
        let account_split: Vec<&str> = account_ids.0.split(',').collect::<Vec<_>>();
        for app_id in apps_split {
            let mock_app_ctx = IamCertServ::try_use_app_ctx(ctx.clone(), Some(app_id.to_string()))?;
            for account_id in account_split.clone() {
                IamRoleServ::delete_rel_account(&id.0, account_id, Some(RBUM_SCOPE_LEVEL_APP), &funs, &mock_app_ctx).await?;
            }
        }
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// get Rel Account by role_code
    #[oai(path = "/:role_code/accounts", method = "get")]
    async fn get_rel_accounts(&self, role_code: Path<String>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Vec<IamRoleRelAccountCertResp>> {
        let funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        add_remote_ip(request, &ctx.0).await?;
        let role_id = RbumItemServ::find_one_rbum(
            &RbumBasicFilterReq {
                code: Some(role_code.0),
                ..Default::default()
            },
            &funs,
            &ctx.0,
        )
        .await?
        .ok_or_else(|| funs.err().internal_error("iam_ci_role", "get_rel_accounts", "role is not found", "404-iam-res-not-exist"))?
        .id;
        let account_ids = IamRoleServ::find_id_rel_accounts(&role_id, None, None, &funs, &ctx.0).await?;
        let certs = IamCertServ::find_certs(
            &RbumCertFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                rel_rbum_ids: Some(account_ids.clone()),
                ..Default::default()
            },
            None,
            None,
            &funs,
            &ctx.0,
        )
        .await?
        .into_iter()
        .map(|r| (r.rel_rbum_id, r.rel_rbum_cert_conf_code.unwrap_or_default(), r.ak))
        .collect_vec();
        let result = account_ids
            .iter()
            .map(|account_id| IamRoleRelAccountCertResp {
                account_id: account_id.clone(),
                certs: certs.iter().filter(|cert| &cert.0 == account_id).map(|r| (r.1.clone(), r.2.clone())).collect(),
            })
            .collect_vec();
        TardisResp::ok(result)
    }

    /// Find Roles
    #[oai(path = "/", method = "get")]
    async fn paginate(
        &self,
        id: Query<Option<String>>,
        name: Query<Option<String>>,
        app_id: Query<Option<String>>,
        in_base: Query<Option<bool>>,
        in_embed: Query<Option<bool>>,
        extend_role_id: Query<Option<String>>,
        with_sub: Query<Option<bool>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        mut ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<TardisPage<IamRoleSummaryResp>> {
        let funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        let ctx = IamCertServ::try_use_app_ctx(ctx.0, app_id.0)?;
        add_remote_ip(request, &ctx).await?;
        let result = IamRoleServ::paginate_items(
            &IamRoleFilterReq {
                basic: RbumBasicFilterReq {
                    ids: id.0.map(|id| vec![id]),
                    name: name.0,
                    with_sub_own_paths: with_sub.0.unwrap_or(false),
                    ..Default::default()
                },
                in_base: in_base.0,
                in_embed: in_embed.0,
                extend_role_id: extend_role_id.0,
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
