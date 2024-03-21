use bios_basic::helper::request_helper::add_remote_ip;
use tardis::web::poem_openapi;
use tardis::web::{
    context_extractor::TardisContextExtractor,
    poem::Request,
    web_resp::{TardisApiResult, TardisResp},
};

use crate::{
    basic::{
        dto::{iam_filer_dto::IamTenantFilterReq, iam_tenant_dto::IamTenantAggDetailResp},
        serv::iam_tenant_serv::IamTenantServ,
    },
    iam_constants,
};

#[derive(Clone, Default)]
pub struct IamCiTenantApi;

#[poem_openapi::OpenApi(prefix_path = "/ci/tenant", tag = "bios_basic::ApiTag::Tenant")]
impl IamCiTenantApi {
    /// Get Current Tenant
    #[oai(path = "/", method = "get")]
    async fn get(&self, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<IamTenantAggDetailResp> {
        add_remote_ip(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamTenantServ::get_tenant_agg(&IamTenantServ::get_id_by_ctx(&ctx.0, &funs)?, &IamTenantFilterReq::default(), &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }
}
