use bios_sdk_invoke::{
    clients::spi_stats_client::SpiStatsClient,
    dto::stats_record_dto::{StatsFactRecordLoadReq, StatsFactRecordsLoadReq},
};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    TardisFunsInst,
};

pub struct FlowStatsClient;

impl FlowStatsClient {
    pub async fn fact_record_load(
        fact_key: &str,
        record_key: &str,
        add_req: StatsFactRecordLoadReq,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        SpiStatsClient::fact_record_load(fact_key, record_key, add_req, funs, ctx).await
    }

    pub async fn fact_records_load(
        fact_key: &str,
        add_req: Vec<StatsFactRecordsLoadReq>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        SpiStatsClient::fact_records_load(fact_key, add_req, funs, ctx).await
    }

    pub async fn fact_record_delete(fact_key: &str, record_key: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        SpiStatsClient::fact_record_delete(fact_key, record_key, funs, ctx).await
    }
}
