use std::time::Duration;

use crate::{log_initializer::get_tardis_inst, serv};
use bios_sdk_invoke::clients::{
    event_client::{BiosEventCenter, EventCenter},
    spi_log_client::LogItemAddReq,
};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    {log as tracing, log::instrument},
};

#[instrument]
async fn handle_add_event(req: LogItemAddReq, ctx: TardisContext) -> TardisResult<()> {
    let funs = get_tardis_inst();
    serv::log_item_serv::add(&mut req.into(), &funs, &ctx).await?;
    Ok(())
}

pub fn register_log_event() {
    if let Some(bios_event_center) = BiosEventCenter::worker_queue() {
        bios_event_center.subscribe(handle_add_event);
    }
}
