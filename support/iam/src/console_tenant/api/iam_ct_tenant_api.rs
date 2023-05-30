use bios_basic::process::task_processor::TaskProcessor;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use crate::basic::dto::iam_filer_dto::IamTenantFilterReq;
use crate::basic::dto::iam_tenant_dto::{IamTenantAggDetailResp, IamTenantAggModifyReq, IamTenantConfigReq, IamTenantConfigResp};
use crate::basic::serv::iam_tenant_serv::IamTenantServ;
use crate::iam_constants;
use tardis::tokio::{self, task};
pub struct IamCtTenantApi;

/// Tenant Console Tenant API
#[poem_openapi::OpenApi(prefix_path = "/ct/tenant", tag = "bios_basic::ApiTag::Tenant")]
impl IamCtTenantApi {
    /// Get Current Tenant
    #[oai(path = "/", method = "get")]
    async fn get(&self, ctx: TardisContextExtractor) -> TardisApiResult<IamTenantAggDetailResp> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamTenantServ::get_tenant_agg(&IamTenantServ::get_id_by_ctx(&ctx.0, &funs)?, &IamTenantFilterReq::default(), &funs, &ctx.0).await?;
        let task_handle = task::spawn_blocking(move || tokio::runtime::Runtime::new().unwrap().block_on(ctx.0.execute_task()));
        let _ = task_handle.await;
        TardisResp::ok(result)
    }

    /// Modify Current Tenant
    ///
    /// When code = 202, the return value is the asynchronous task id
    #[oai(path = "/", method = "put")]
    async fn modify(&self, modify_req: Json<IamTenantAggModifyReq>, ctx: TardisContextExtractor) -> TardisApiResult<Option<String>> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamTenantServ::modify_tenant_agg(&IamTenantServ::get_id_by_ctx(&ctx.0, &funs)?, &modify_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        let ctx_task = ctx.0.clone();
        let task_handle = task::spawn_blocking(move || tokio::runtime::Runtime::new().unwrap().block_on(ctx_task.execute_task()));
        let _ = task_handle.await;
        if let Some(task_id) = TaskProcessor::get_task_id_with_ctx(&ctx.0).await? {
            TardisResp::accepted(Some(task_id))
        } else {
            TardisResp::ok(None)
        }
    }

    /// modify Current Tenant config
    #[oai(path = "/config", method = "put")]
    async fn modify_config(&self, config_req: Json<IamTenantConfigReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamTenantServ::modify_tenant_config_agg(&IamTenantServ::get_id_by_ctx(&ctx.0, &funs)?, &config_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        let task_handle = task::spawn_blocking(move || tokio::runtime::Runtime::new().unwrap().block_on(ctx.0.execute_task()));
        let _ = task_handle.await;
        TardisResp::ok(Void {})
    }

    /// Get Current Tenant config
    #[oai(path = "/config", method = "get")]
    async fn get_config(&self, ctx: TardisContextExtractor) -> TardisApiResult<IamTenantConfigResp> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamTenantServ::get_tenant_config_agg(&IamTenantServ::get_id_by_ctx(&ctx.0, &funs)?, &funs, &ctx.0).await?;
        let task_handle = task::spawn_blocking(move || tokio::runtime::Runtime::new().unwrap().block_on(ctx.0.execute_task()));
        let _ = task_handle.await;
        TardisResp::ok(result)
    }
}
