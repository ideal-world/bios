use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Query;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use crate::basic::serv::iam_tenant_serv::IamTenantServ;
use crate::iam_constants;
use bios_basic::helper::request_helper::try_set_real_ip_from_req_to_ctx;
use tardis::web::poem::Request;

#[derive(Clone, Default)]
pub struct IamCcTenantApi;

/// Common Console Tenant API
#[poem_openapi::OpenApi(prefix_path = "/cc/tenant", tag = "bios_basic::ApiTag::Common")]
impl IamCcTenantApi {
    /// Find Tenant Name By Ids
    ///
    /// Return format: ["<id>,<name>"]
    #[oai(path = "/name", method = "get")]
    async fn find_name_by_ids(
        &self,
        // Tenant Ids, multiple ids separated by ,
        ids: Query<String>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Vec<String>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let ids = ids.0.split(',').map(|s| s.to_string()).collect();
        let result = IamTenantServ::find_name_by_ids(ids, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }
}
