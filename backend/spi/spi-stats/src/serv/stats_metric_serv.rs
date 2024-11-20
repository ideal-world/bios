use bios_basic::spi::spi_constants;
use bios_basic::spi::spi_funs::SpiBsInstExtractor;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::web::web_resp::TardisPage;
use tardis::TardisFunsInst;

use crate::dto::stats_query_dto::{StatsQueryMetricsRecordReq, StatsQueryMetricsReq, StatsQueryMetricsResp};
use crate::stats_initializer;

use super::pg;

pub async fn query_metrics(query_req: &StatsQueryMetricsReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<StatsQueryMetricsResp> {
    let inst = funs.init(None, ctx, true, stats_initializer::init_fun).await?;
    match inst.kind_code() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_metric_serv::query_metrics(query_req, funs, ctx, &inst).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn query_metrics_record_paginated(query_req: &StatsQueryMetricsRecordReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<TardisPage<serde_json::Value>> {
    let inst = funs.init(None, ctx, true, stats_initializer::init_fun).await?;
    match inst.kind_code() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_metric_serv::query_metrics_record_paginated(query_req, funs, ctx, &inst).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}
