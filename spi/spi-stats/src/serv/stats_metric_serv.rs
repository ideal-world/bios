use bios_basic::spi::spi_constants;
use bios_basic::spi::spi_funs::SpiBsInstExtractor;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::TardisFunsInst;

use crate::dto::stats_query_dto::{StatsQueryMetricsReq, StatsQueryMetricsResp};
use crate::stats_initializer;

use super::pg;

pub async fn query_metrics(query_req: &StatsQueryMetricsReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<StatsQueryMetricsResp> {
    match funs.init(ctx, true, stats_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_metric_serv::query_metrics(query_req, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}
