use tardis::web::poem::web::websocket::{BoxWebSocketUpgraded, WebSocket};
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::{Path, Query};

use crate::event_constants::get_tardis_inst;
use crate::serv::event_proc_serv;
#[derive(Clone)]
pub struct EventProcApi;

/// Event Process API
///
/// 事件处理API
#[poem_openapi::OpenApi(prefix_path = "/proc")]
impl EventProcApi {
    /// Process event
    ///
    /// 处理事件
    #[oai(path = "/:listener_code", method = "get")]
    async fn ws_process(&self, listener_code: Path<String>, token: Query<String>, websocket: WebSocket) -> BoxWebSocketUpgraded {
        let funs = get_tardis_inst();
        event_proc_serv::ws_process(listener_code.0, token.0, websocket, funs).await
    }
}
