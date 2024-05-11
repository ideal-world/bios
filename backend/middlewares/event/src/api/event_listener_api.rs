use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::{Path, Query};
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use crate::dto::event_dto::{EventListenerRegisterReq, EventListenerRegisterResp};
use crate::event_constants::get_tardis_inst;
use crate::serv::event_listener_serv;
#[derive(Clone)]
pub struct EventListenerApi;

/// Event Listener API
/// 事件监听器API
#[poem_openapi::OpenApi(prefix_path = "/listener")]
impl EventListenerApi {
    /// Register event listener
    /// 注册事件监听器
    #[oai(path = "/", method = "post")]
    async fn register(&self, listener: Json<EventListenerRegisterReq>) -> TardisApiResult<EventListenerRegisterResp> {
        let funs = get_tardis_inst();
        let resp = event_listener_serv::register(listener.0, &funs).await?;
        TardisResp::ok(resp)
    }

    /// Remove event listener
    /// 移除事件监听器
    #[oai(path = "/:listener_code", method = "delete")]
    async fn remove(&self, listener_code: Path<String>, token: Query<String>) -> TardisApiResult<Void> {
        let funs = get_tardis_inst();
        event_listener_serv::remove(&listener_code.0, &token.0, &funs).await?;
        TardisResp::ok(Void {})
    }
}
