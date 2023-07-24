use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;

use tardis::web::web_resp::{TardisApiResult, TardisResp};

use crate::basic::dto::iam_platform_dto::IamPlatformConfigResp;
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_platform_serv::IamPlatformServ;
use crate::iam_constants;

#[derive(Clone, Default)]
pub struct IamCcPlatformApi;

/// System Console Platform API
#[poem_openapi::OpenApi(prefix_path = "/cc/platform", tag = "bios_basic::ApiTag::Common")]
impl IamCcPlatformApi {
    /// Get Platform config
    #[oai(path = "/", method = "get")]
    async fn get(&self, ctx: TardisContextExtractor) -> TardisApiResult<IamPlatformConfigResp> {
        let funs = iam_constants::get_tardis_inst();
        let new_ctx = IamAccountServ::new_context_if_account_is_global(&ctx.0, &funs).await?;
        let result = IamPlatformServ::get_platform_config_agg(&funs, &new_ctx).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }
}
