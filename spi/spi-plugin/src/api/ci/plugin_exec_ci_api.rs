use bios_basic::spi::spi_funs::SpiTardisFunInstExtractor;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::web::Json;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Path;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use crate::dto::plugin_exec_dto::{PluginExecReq, PluginExecResp};
use crate::serv::plugin_exec_serv::PluginExecServ;

pub struct PluginExecApi;

/// Plugin exec API
#[poem_openapi::OpenApi(prefix_path = "/ci/spi/plugin", tag = "bios_basic::ApiTag::Interface")]
impl PluginExecApi {
    /// Put Plugin exec
    #[oai(path = "/:kind_code/api/:api_code/exec", method = "post")]
    async fn plugin_exec(
        &self,
        kind_code: Path<String>,
        api_code: Path<String>,
        exec_req: Json<PluginExecReq>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<PluginExecResp> {
        let funs = request.tardis_fun_inst();
        let result = PluginExecServ::exec(&kind_code.0, &api_code.0, exec_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(PluginExecResp {
            code: result.code,
            headers: result.headers.clone(),
            body: result.body,
        })
    }

    /// Put Plugin exec
    #[oai(path = "/test/exec/:msg", method = "get")]
    async fn test_exec(&self, msg: Path<String>, _ctx: TardisContextExtractor, _request: &Request) -> TardisApiResult<String> {
        TardisResp::ok(msg.0)
    }
}
