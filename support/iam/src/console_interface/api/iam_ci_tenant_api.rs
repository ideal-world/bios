use crate::basic::dto::iam_app_dto::IamAppAggAddReq;
use crate::basic::serv::iam_app_serv::IamAppServ;
use crate::iam_constants;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

pub struct IamCiTenantApi;

/// # Interface Console Manage Cert API
///
/// Allow Management Of aksk (an authentication method between applications)
#[poem_openapi::OpenApi(prefix_path = "/ci/tenant", tag = "bios_basic::ApiTag::Interface")]
impl IamCiTenantApi {
    /// Add App
    #[oai(path = "/app", method = "post")]
    async fn add(&self, add_req: Json<IamAppAggAddReq>, ctx: TardisContextExtractor) -> TardisApiResult<String> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let result = IamAppServ::add_app_agg(&add_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(result)
    }
}
