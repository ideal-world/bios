use crate::{get_tardis_inst, serv::stats_record_serv};
use bios_sdk_invoke::clients::{
    event_client::{asteroid_mq_sdk::model::Interest, mq_client_node_opt, mq_error, ContextHandler, SPI_RPC_TOPIC},
    spi_stats_client::{StatsItemAddReq, StatsItemAddsReq, StatsItemDeleteReq},
};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    {log as tracing, log::instrument},
};

#[instrument]
async fn handle_add_event(req: StatsItemAddReq, ctx: TardisContext) -> TardisResult<()> {
    let funs = get_tardis_inst();
    stats_record_serv::fact_record_load(&req.fact_key, &req.record_key, req.req.into(), &funs, &ctx).await?;
    Ok(())
}

#[instrument]
async fn handle_adds_event(req: StatsItemAddsReq, ctx: TardisContext) -> TardisResult<()> {
    let funs = get_tardis_inst();
    stats_record_serv::fact_records_load(&req.fact_key, req.reqs.into_iter().map(|r| r.into()).collect(), &funs, &ctx).await?;
    Ok(())
}

#[instrument]
async fn handle_delete_event(req: StatsItemDeleteReq, ctx: TardisContext) -> TardisResult<()> {
    let funs = get_tardis_inst();
    stats_record_serv::fact_record_delete(&req.fact_key, &req.record_key, &funs, &ctx).await?;
    Ok(())
}

pub async fn handle_events() -> TardisResult<()> {
    if let Some(node) = mq_client_node_opt() {
        node.create_endpoint(SPI_RPC_TOPIC, [Interest::new("stats/*")])
            .await
            .map_err(mq_error)?
            .into_event_loop()
            .with_handler(ContextHandler(handle_add_event))
            .with_handler(ContextHandler(handle_adds_event))
            .with_handler(ContextHandler(handle_delete_event))
            .spawn();
    }

    Ok(())
}
