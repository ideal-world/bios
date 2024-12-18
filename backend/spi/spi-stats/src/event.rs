use crate::{dto::stats_record_dto::StatsFactRecordLoadReq, get_tardis_inst, serv::stats_record_serv};
use bios_sdk_invoke::clients::{
    event_client::{get_topic, mq_error, ContextHandler, SPI_RPC_TOPIC},
    spi_log_client::{StatsItemAddReq, StatsItemDeleteReq},
};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    {log as tracing, log::instrument},
};

#[instrument]
async fn handle_add_event(req: StatsItemAddReq, ctx: TardisContext) -> TardisResult<()> {
    let funs = get_tardis_inst();
    if let Some(ref key) = req.key {
        let record_load_req = StatsFactRecordLoadReq {
            own_paths: req.own_paths.unwrap_or_default(),
            ct: req.ts.unwrap_or_default(),
            idempotent_id: req.idempotent_id,
            ignore_updates: Some(true),
            data: req.content,
            ext: req.ext,
        };
        stats_record_serv::fact_record_load(&req.tag, key.as_ref(), record_load_req, &funs, &ctx).await?;
    }
    Ok(())
}
#[instrument]
async fn handle_delete_event(req: StatsItemDeleteReq, ctx: TardisContext) -> TardisResult<()> {
    let funs = get_tardis_inst();
    if let Some(ref key) = req.key {
        stats_record_serv::fact_record_delete(&req.tag, key.as_ref(), &funs, &ctx).await?;
    }
    Ok(())
}

pub async fn handle_events() -> TardisResult<()> {
    use bios_sdk_invoke::clients::event_client::asteroid_mq::prelude::*;
    if let Some(topic) = get_topic(&SPI_RPC_TOPIC) {
        topic.create_endpoint([Interest::new("stats/add")]).await.map_err(mq_error)?.create_event_loop().with_handler(ContextHandler(handle_add_event)).spawn();
        topic.create_endpoint([Interest::new("stats/delete")]).await.map_err(mq_error)?.create_event_loop().with_handler(ContextHandler(handle_delete_event)).spawn();
    }

    Ok(())
}
