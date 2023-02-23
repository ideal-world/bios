use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    TardisFunsInst,
};

use crate::dto::stats_query_dto::{StatsQueryMetricsReq, StatsQueryMetricsResp};

pub async fn query_metrics(add_req: &StatsQueryMetricsReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<StatsQueryMetricsResp>> {
    Ok(vec![])
}
