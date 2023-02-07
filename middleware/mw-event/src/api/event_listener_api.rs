use bios_basic::spi::spi_funs::SpiTardisFunInstExtractor;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::{Path, Query};
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use crate::dto::event_dto::{EventListenerRegisterReq, EventListenerRegisterResp};
use crate::serv::event_listener_serv;

pub struct EventListenerApi;

/// Event Listener API
#[poem_openapi::OpenApi(prefix_path = "/listener")]
impl EventListenerApi {
    #[oai(path = "/", method = "post")]
    async fn register(&self, listener: Json<EventListenerRegisterReq>, request: &Request) -> TardisApiResult<EventListenerRegisterResp> {
        let funs = request.tardis_fun_inst();
        let resp = event_listener_serv::register(listener.0, &funs).await?;
        TardisResp::ok(resp)
    }

    #[oai(path = "/:event_code/:listener_code", method = "delete")]
    async fn remove(&self, event_code: Path<String>, listener_code: Path<String>, token: Query<String>, request: &Request) -> TardisApiResult<Void> {
        let funs = request.tardis_fun_inst();
        event_listener_serv::remove(&event_code.0, &listener_code.0, &token.0, &funs).await?;
        TardisResp::ok(Void {})
    }
}
