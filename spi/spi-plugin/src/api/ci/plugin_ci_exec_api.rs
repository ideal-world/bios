use tardis::log::info;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::web::Json;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Path;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use crate::dto::plugin_exec_dto::{PluginExecReq, PluginExecResp};
use crate::serv::plugin_exec_serv::PluginExecServ;
#[derive(Clone)]

pub struct PluginExecApi;

/// Plugin exec API
#[poem_openapi::OpenApi(prefix_path = "/ci/spi/plugin", tag = "bios_basic::ApiTag::Interface")]
impl PluginExecApi {
    /// Put Plugin exec
    #[oai(path = "/:kind_code/api/:api_code/exec", method = "put")]
    async fn plugin_exec(&self, kind_code: Path<String>, api_code: Path<String>, exec_req: Json<PluginExecReq>, ctx: TardisContextExtractor) -> TardisApiResult<PluginExecResp> {
        let funs = crate::get_tardis_inst();
        let result = PluginExecServ::exec(&kind_code.0, &api_code.0, exec_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(PluginExecResp {
            code: result.code,
            headers: result.headers.clone(),
            body: result.body,
        })
    }

    /// Put Plugin exec
    #[oai(path = "/test/exec/:msg", method = "delete")]
    async fn test_exec(&self, msg: Path<String>, _ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<PluginExecReq> {
        for (k, v) in request.headers() {
            info!("{}: {}", k, v.to_str().unwrap_or("invalid utf8 header value"));
        }
        TardisResp::ok(PluginExecReq {
            body: Some(tardis::serde_json::Value::String(msg.0)),
            header: None,
        })
    }
}
