use bios_basic::spi::spi_funs::SpiTardisFunInstExtractor;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use crate::dto::stats_query_dto::{StatsQueryMetricsReq, StatsQueryMetricsResp};
use crate::serv::stats_query_serv;

pub struct StatsCiQueryApi;

/// Interface Console Statistics Query API
#[poem_openapi::OpenApi(prefix_path = "/ci/query", tag = "bios_basic::ApiTag::Interface")]
impl StatsCiQueryApi {
    /// Query Metrics
    #[oai(path = "/metrics", method = "put")]
    async fn query_metrics(&self, add_req: Json<StatsQueryMetricsReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Vec<StatsQueryMetricsResp>> {
        let funs = request.tardis_fun_inst();
        let resp = stats_query_serv::query_metrics(&add_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(resp)
    }
}
