use crate::{get_tardis_inst, serv};
use bios_sdk_invoke::clients::{
    event_client::{get_topic, mq_error, ContextHandler, SPI_RPC_TOPIC},
    spi_log_client::LogItemAddReq,
};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    {log as tracing, log::instrument},
};

#[instrument]
async fn handle_add_event(req: LogItemAddReq, ctx: TardisContext) -> TardisResult<()> {
    let funs = get_tardis_inst();
    //TODO
    Ok(())
}

pub async fn handle_events() -> TardisResult<()> {
    use bios_sdk_invoke::clients::event_client::asteroid_mq::prelude::*;
    if let Some(topic) = get_topic(&SPI_RPC_TOPIC) {
        topic.create_endpoint([Interest::new("stats/*")]).await.map_err(mq_error)?.create_event_loop().with_handler(ContextHandler(handle_add_event)).spawn();
    }

    Ok(())
}
