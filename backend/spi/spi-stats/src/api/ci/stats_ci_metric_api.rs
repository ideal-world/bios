use tardis::web::context_extractor::TardisContextExtractor;

use tardis::web::poem_openapi;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp};

use crate::dto::stats_query_dto::{StatsQueryMetricsRecordReq, StatsQueryMetricsReq, StatsQueryMetricsResp};
use crate::serv::stats_metric_serv;

#[derive(Clone)]
pub struct StatsCiMetricApi;

/// Interface Console Statistics Metric API
///
/// 统计指标接口
#[poem_openapi::OpenApi(prefix_path = "/ci/metric", tag = "bios_basic::ApiTag::Interface")]
impl StatsCiMetricApi {
    /// Query Metrics
    ///
    /// 查询指标
    #[oai(path = "/", method = "put")]
    async fn query_metrics(&self, query_req: Json<StatsQueryMetricsReq>, ctx: TardisContextExtractor) -> TardisApiResult<StatsQueryMetricsResp> {
        let funs = crate::get_tardis_inst();
        let resp = stats_metric_serv::query_metrics(&query_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(resp)
    }

    /// Query Metrics record
    ///
    /// 查询指标 记录
    #[oai(path = "/record", method = "put")]
    async fn query_metrics_record_paginated(&self, query_req: Json<StatsQueryMetricsRecordReq>, ctx: TardisContextExtractor) -> TardisApiResult<TardisPage<serde_json::Value>> {
        let funs = crate::get_tardis_inst();
        let resp = stats_metric_serv::query_metrics_record_paginated(&query_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(resp)
    }
}
