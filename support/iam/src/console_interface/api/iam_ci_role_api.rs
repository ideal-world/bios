use crate::basic::serv::iam_app_serv::IamAppServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_role_serv::IamRoleServ;
use crate::iam_constants::{self, RBUM_SCOPE_LEVEL_APP};
use bios_basic::helper::request_helper::add_remote_ip;
use bios_basic::process::task_processor::TaskProcessor;
use tardis::tokio;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::{Request, RequestBody};
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::{Path, Query};
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

#[derive(Clone, Default)]
pub struct IamCiRoleApi;

/// # Interface Console Manage Cert API
///
/// Allow Management Of aksk (an authentication method between applications)
#[poem_openapi::OpenApi(prefix_path = "/ci/role", tag = "bios_basic::ApiTag::Interface")]
impl IamCiRoleApi {
    /// Batch add Role Rel Account
    #[oai(path = "/:id/account/batch/:account_ids", method = "put")]
    async fn batch_add_rel_account(&self, id: Path<String>, account_ids: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        add_remote_ip(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
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
    async fn batch_delete_rel_account(&self, id: Path<String>, account_ids: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        add_remote_ip(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
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
    async fn delete_rel_account(&self, id: Path<String>, account_id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        add_remote_ip(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
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
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Option<String>> {
        add_remote_ip(request, &ctx.0).await?;
        let ctx = ctx.0;
        let ctx_clone = ctx.clone();
        ctx.add_async_task(Box::new(|| {
            Box::pin(async move {
                let task_handle = tokio::spawn(async move {
                    let funs = iam_constants::get_tardis_inst();
                    let apps_split: Vec<&str> = app_ids.0.split(',').collect::<Vec<_>>();
                    let account_split: Vec<&str> = account_ids.0.split(',').collect::<Vec<_>>();
                    for app_id in apps_split {
                        let mock_app_ctx = IamCertServ::try_use_app_ctx(ctx_clone.clone(), Some(app_id.to_string())).unwrap_or(ctx_clone.clone());
                        for account_id in account_split.clone() {
                            let _ = IamAppServ::add_rel_account(app_id, account_id, true, &funs, &mock_app_ctx).await;
                            let _ = IamRoleServ::add_rel_account(&id.0, account_id, Some(RBUM_SCOPE_LEVEL_APP), &funs, &mock_app_ctx).await;
                        }
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
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Void> {
        add_remote_ip(request, &ctx.0).await?;
        let ctx = ctx.0;
        let mut funs = iam_constants::get_tardis_inst();
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
}
