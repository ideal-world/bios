use crate::{log_initializer::get_tardis_inst, serv};
use bios_sdk_invoke::clients::{
    event_client::{asteroid_mq_sdk::model::Interest, mq_client_node_opt, mq_error, ContextHandler, SPI_RPC_TOPIC},
    spi_log_client::LogItemAddV2Req,
};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    {log as tracing, log::instrument},
};

#[instrument(name = "[SPI.Log.AddEvent]", level = "trace", skip_all)]
async fn handle_add_event(req: LogItemAddV2Req, ctx: TardisContext) -> TardisResult<()> {
    tracing::trace!("Received LogItemAddV2Req: {:?}", req);
    tracing::trace!("Attempting to add log item with context: {:?}", ctx);
    let funs = get_tardis_inst();
    serv::log_item_serv::addv2(&mut req.into(), &funs, &ctx).await?;
    Ok(())
}

pub async fn handle_events() -> TardisResult<()> {
    if let Some(topic) = mq_client_node_opt() {
        topic.create_endpoint(SPI_RPC_TOPIC, [Interest::new("log/*")]).await.map_err(mq_error)?.into_event_loop().with_handler(ContextHandler(handle_add_event)).spawn();
    }
    Ok(())
}
