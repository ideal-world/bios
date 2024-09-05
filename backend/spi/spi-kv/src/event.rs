use crate::{get_tardis_inst, serv};
use bios_sdk_invoke::clients::{
    event_client::{get_topic, mq_error, ContextHandler, SPI_RPC_TOPIC},
    spi_kv_client::{KvItemAddOrModifyReq, KvItemDeleteReq},
};
use tardis::basic::result::TardisResult;
use tardis::{
    basic::dto::TardisContext,
    log::{self as tracing, instrument},
};
#[instrument]
async fn handle_kv_add_event(req: KvItemAddOrModifyReq, ctx: TardisContext) -> TardisResult<()> {
    let funs = get_tardis_inst();
    serv::kv_item_serv::add_or_modify_item(&mut req.into(), &funs, &ctx).await?;
    Ok(())
}

#[instrument]
async fn handle_kv_delete_event(req: KvItemDeleteReq, ctx: TardisContext) -> TardisResult<()> {
    let funs = get_tardis_inst();
    serv::kv_item_serv::delete_item(req.key.trim().to_string(), &funs, &ctx).await?;
    Ok(())
}

pub async fn handle_events() -> TardisResult<()> {
    use bios_sdk_invoke::clients::event_client::asteroid_mq::prelude::*;
    if let Some(topic) = get_topic(&SPI_RPC_TOPIC) {
        topic
            .create_endpoint([Interest::new("kv/*")])
            .await
            .map_err(mq_error)?
            .create_event_loop()
            .with_handler(ContextHandler(handle_kv_add_event))
            .with_handler(ContextHandler(handle_kv_delete_event))
            .spawn();
    }
    // let topic = get_topic(&SPI_RPC_TOPIC).expect("topic not initialized");

    Ok(())
}
