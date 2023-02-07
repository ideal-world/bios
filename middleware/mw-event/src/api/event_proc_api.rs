use bios_basic::spi::spi_funs::SpiTardisFunInstExtractor;
use tardis::serde_json::Value;
use tardis::web::poem::web::websocket::{BoxWebSocketUpgraded, WebSocket};
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::{Path, Query};
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use crate::serv::event_proc_serv;

pub struct EventProcApi;

/// Event Process API
#[poem_openapi::OpenApi(prefix_path = "/proc")]
impl EventProcApi {
    #[oai(path = "/ws/:event_code/:listener_code", method = "get")]
    async fn ws_process(&self, event_code: Path<String>, listener_code: Path<String>, token: Query<String>, websocket: WebSocket, request: &Request) -> BoxWebSocketUpgraded {
        let funs = request.tardis_fun_inst();
        event_proc_serv::ws_process(event_code.0, listener_code.0, token.0, websocket, &funs).await
    }

    #[oai(path = "/http/:event_code/:listener_code", method = "post")]
    async fn http_process(&self, event_code: Path<String>, listener_code: Path<String>, token: Query<String>, msg: Json<Value>, request: &Request) -> TardisApiResult<Void> {
        let funs = request.tardis_fun_inst();
        event_proc_serv::http_process(event_code.0, listener_code.0, token.0, msg.0, &funs).await?;
        TardisResp::ok(Void {})
    }
}
