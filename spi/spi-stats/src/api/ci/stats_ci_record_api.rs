use bios_basic::spi::spi_funs::SpiTardisFunInstExtractor;
use tardis::chrono::{DateTime, Utc};
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::{Path, Query};
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use crate::dto::stats_record_dto::{StatsDimRecordAddReq, StatsFactRecordLoadReq, StatsFactRecordsLoadReq};
use crate::serv::stats_record_serv;

pub struct StatsCiRecordApi;

/// Interface Console Statistics Record API
#[poem_openapi::OpenApi(prefix_path = "/ci/record", tag = "bios_basic::ApiTag::Interface")]
impl StatsCiRecordApi {
    /// Load Fact Record
    #[oai(path = "/fact/:fact_key/:record_key", method = "put")]
    async fn fact_load_record(
        &self,
        fact_key: Path<String>,
        record_key: Path<String>,
        add_req: Json<StatsFactRecordLoadReq>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Void> {
        let funs = request.tardis_fun_inst();
        stats_record_serv::fact_load_record(fact_key.0, record_key.0, add_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Delete Fact Record
    #[oai(path = "/fact/:fact_key/:record_key", method = "delete")]
    async fn fact_delete_record(&self, fact_key: Path<String>, record_key: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let funs = request.tardis_fun_inst();
        stats_record_serv::fact_delete_record(fact_key.0, record_key.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Load Fact Records
    #[oai(path = "/fact/:fact_key/batch/load", method = "put")]
    async fn fact_load_records(
        &self,
        fact_key: Path<String>,
        add_req: Json<Vec<StatsFactRecordsLoadReq>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Void> {
        let funs = request.tardis_fun_inst();
        stats_record_serv::fact_load_records(fact_key.0, add_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Delete Fact Records
    #[oai(path = "/fact/:fact_key/batch/remove", method = "put")]
    async fn fact_delete_records(&self, fact_key: Path<String>, delete_req: Json<Vec<String>>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let funs = request.tardis_fun_inst();
        stats_record_serv::fact_delete_records(fact_key.0, delete_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Clean Fact Records
    #[oai(path = "/fact/:fact_key/batch/remove", method = "put")]
    async fn fact_clean_records(&self, fact_key: Path<String>, before_ct: Query<Option<DateTime<Utc>>>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let funs = request.tardis_fun_inst();
        stats_record_serv::fact_clean_records(fact_key.0, before_ct.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Add Dimension Record
    #[oai(path = "/dim/:dim_key/:record_key", method = "put")]
    async fn dim_add_record(
        &self,
        dim_key: Path<String>,
        record_key: Path<String>,
        add_req: Json<StatsDimRecordAddReq>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Void> {
        let funs = request.tardis_fun_inst();
        stats_record_serv::dim_add_record(dim_key.0, record_key.0, add_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Delete Dimension Record
    #[oai(path = "/dim/:dim_key/:record_key", method = "delete")]
    async fn dim_delete_record(&self, dim_key: Path<String>, record_key: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let funs = request.tardis_fun_inst();
        stats_record_serv::dim_delete_record(dim_key.0, record_key.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }
}
