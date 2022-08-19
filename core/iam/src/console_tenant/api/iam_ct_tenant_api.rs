use bios_basic::process::task_processor::TaskProcessor;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use crate::basic::dto::iam_filer_dto::IamTenantFilterReq;
use crate::basic::dto::iam_tenant_dto::{IamTenantAggDetailResp, IamTenantAggModifyReq};
use crate::basic::serv::iam_tenant_serv::IamTenantServ;
use crate::iam_constants;

pub struct IamCtTenantApi;

/// Tenant Console Tenant API
#[poem_openapi::OpenApi(prefix_path = "/ct/tenant", tag = "crate::iam_enumeration::Tag::Tenant")]
impl IamCtTenantApi {
    /// Get Current Tenant
    #[oai(path = "/", method = "get")]
    async fn get(&self, ctx: TardisContextExtractor) -> TardisApiResult<IamTenantAggDetailResp> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamTenantServ::get_tenant_agg(&IamTenantServ::get_id_by_ctx(&ctx.0, &funs)?, &IamTenantFilterReq::default(), &funs, &ctx.0).await?;
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
        if let Some(task_id) = TaskProcessor::get_task_id_with_ctx(&ctx.0)? {
            TardisResp::accepted(Some(task_id))
        } else {
            TardisResp::ok(None)
        }
    }
}
