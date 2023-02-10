use bios_basic::spi::spi_funs::SpiTardisFunInstExtractor;
use tardis::web::poem::web::websocket::{BoxWebSocketUpgraded, WebSocket};
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::{Path, Query};

use crate::serv::event_proc_serv;

pub struct EventProcApi;

/// Event Process API
#[poem_openapi::OpenApi(prefix_path = "/proc")]
impl EventProcApi {
    #[oai(path = "/:listener_code", method = "get")]
    async fn ws_process(&self, listener_code: Path<String>, token: Query<String>, websocket: WebSocket, request: &Request) -> BoxWebSocketUpgraded {
        let funs = request.tardis_fun_inst();
        event_proc_serv::ws_process(listener_code.0, token.0, websocket, &funs).await
    }
}
