use tardis::tokio::{self, task};
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use crate::basic::dto::iam_platform_dto::{IamPlatformConfigReq, IamPlatformConfigResp};
use crate::basic::serv::iam_platform_serv::IamPlatformServ;
use crate::iam_constants;

pub struct IamCsPlatformApi;

/// System Console Platform API
#[poem_openapi::OpenApi(prefix_path = "/cs/platform", tag = "bios_basic::ApiTag::System")]
impl IamCsPlatformApi {
    /// modify Platform config
    #[oai(path = "/", method = "put")]
    async fn modify(&self, modify_req: Json<IamPlatformConfigReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamPlatformServ::modify_platform_config_agg(&modify_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        let task_handle = task::spawn_blocking(move || tokio::runtime::Runtime::new().unwrap().block_on(ctx.0.execute_task()));
        let _ = task_handle.await;
        TardisResp::ok(Void {})
    }

    /// Get Platform config
    #[oai(path = "/", method = "get")]
    async fn get(&self, ctx: TardisContextExtractor) -> TardisApiResult<IamPlatformConfigResp> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamPlatformServ::get_platform_config_agg(&funs, &ctx.0).await?;
        let task_handle = task::spawn_blocking(move || tokio::runtime::Runtime::new().unwrap().block_on(ctx.0.execute_task()));
        let _ = task_handle.await;
        TardisResp::ok(result)
    }
}
