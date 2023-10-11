use bios_basic::process::task_processor::TaskProcessor;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use crate::basic::dto::iam_filer_dto::IamTenantFilterReq;
use crate::basic::dto::iam_tenant_dto::{IamTenantAggDetailResp, IamTenantAggModifyReq};
use crate::basic::serv::iam_tenant_serv::IamTenantServ;
use crate::iam_constants;
use bios_basic::helper::request_helper::add_remote_ip;
use tardis::web::poem::Request;
#[derive(Clone, Default)]
pub struct IamCtTenantApi;

/// Tenant Console Tenant API
#[poem_openapi::OpenApi(prefix_path = "/ct/tenant", tag = "bios_basic::ApiTag::Tenant")]
impl IamCtTenantApi {
    /// Get Current Tenant
    #[oai(path = "/", method = "get")]
    async fn get(&self, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<IamTenantAggDetailResp> {
        add_remote_ip(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamTenantServ::get_tenant_agg(&IamTenantServ::get_id_by_ctx(&ctx.0, &funs)?, &IamTenantFilterReq::default(), &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Modify Current Tenant
    ///
    /// When code = 202, the return value is the asynchronous task id
    #[oai(path = "/", method = "put")]
    async fn modify(&self, modify_req: Json<IamTenantAggModifyReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Option<String>> {
        add_remote_ip(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamTenantServ::modify_tenant_agg(&IamTenantServ::get_id_by_ctx(&ctx.0, &funs)?, &modify_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        if let Some(task_id) = TaskProcessor::get_task_id_with_ctx(&ctx.0).await? {
            TardisResp::accepted(Some(task_id))
        } else {
            TardisResp::ok(None)
        }
    }

    /// modify Current Tenant config
    #[oai(path = "/config", method = "put")]
    async fn modify_config(&self, config_req: Json<IamTenantConfigReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        add_remote_ip(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamTenantServ::modify_tenant_config_agg(&IamTenantServ::get_id_by_ctx(&ctx.0, &funs)?, &config_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Get Current Tenant config
    #[oai(path = "/config", method = "get")]
    async fn get_config(&self, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<IamTenantConfigResp> {
        add_remote_ip(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamTenantServ::get_tenant_config_agg(&IamTenantServ::get_id_by_ctx(&ctx.0, &funs)?, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }
}
