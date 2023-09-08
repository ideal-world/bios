use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use crate::basic::dto::iam_platform_dto::{IamPlatformConfigReq, IamPlatformConfigResp};
use crate::basic::serv::iam_platform_serv::IamPlatformServ;
use crate::iam_constants;
use bios_basic::helper::request_helper::add_remote_ip;
use tardis::web::poem::Request;
#[derive(Clone, Default)]
pub struct IamCsPlatformApi;

/// System Console Platform API
#[poem_openapi::OpenApi(prefix_path = "/cs/platform", tag = "bios_basic::ApiTag::System")]
impl IamCsPlatformApi {
    /// modify Platform config
    #[oai(path = "/", method = "put")]
    async fn modify(&self, modify_req: Json<IamPlatformConfigReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        add_remote_ip(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamPlatformServ::modify_platform_config_agg(&modify_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Get Platform config
    #[oai(path = "/", method = "get")]
    async fn get(&self, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<IamPlatformConfigResp> {
        add_remote_ip(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamPlatformServ::get_platform_config_agg(&funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }
}
