use std::time::Duration;

use crate::{log_constants::EVENT_ADD_LOG, log_initializer::get_tardis_inst, serv};
use bios_sdk_invoke::clients::{
    event_client::{self, BiosEventCenter, EventCenter, EventTopicConfig, FnEventHandler},
    spi_log_client::event::AddLogEvent,
};
use tardis::{
    basic::result::TardisResult,
    log::{error, info, warn},
    tokio,
    web::ws_processor::TardisWebsocketMessage,
    TardisFuns,
};
pub const RECONNECT_INTERVAL: Duration = Duration::from_secs(10);

pub fn register_log_event() {
    let bios_event_center = TardisFuns::store().get_singleton::<BiosEventCenter>().expect("bios event center should be initialized");
    bios_event_center.subscribe::<AddLogEvent, _>(FnEventHandler(|req: AddLogEvent| async {
        let funs = get_tardis_inst();
        let (ctx, req) = req.unpack();
        let mut req = req.into();
        serv::log_item_serv::add(&mut req, &funs, &ctx).await?;
        Ok(())
    }));
}
