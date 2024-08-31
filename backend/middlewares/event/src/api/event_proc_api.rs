use tardis::web::poem::web::websocket::{BoxWebSocketUpgraded, WebSocket};
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::{Path, Query};

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
    async fn ws_process(&self, listener_code: Path<String>, token: Query<String>, websocket: WebSocket) -> Result<BoxWebSocketUpgraded, tardis::web::poem::Error> {
        Err(tardis::web::poem::Error::from_status(tardis::web::poem::http::StatusCode::NOT_IMPLEMENTED))
    }
}
