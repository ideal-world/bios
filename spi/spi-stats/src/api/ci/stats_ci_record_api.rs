use bios_basic::TardisFunInstExtractor;
use tardis::chrono::{DateTime, Utc};
use tardis::serde_json::Value;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::{Path, Query};
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use crate::dto::stats_record_dto::{StatsDimRecordAddReq, StatsDimRecordDeleteReq, StatsFactRecordLoadReq, StatsFactRecordsLoadReq};
use crate::serv::stats_record_serv;

pub struct StatsCiRecordApi;

/// Interface Console Statistics Record API
#[poem_openapi::OpenApi(prefix_path = "/ci/record", tag = "bios_basic::ApiTag::Interface")]
impl StatsCiRecordApi {
    /// Load Fact Record
    #[oai(path = "/fact/:fact_key/:record_key", method = "put")]
    async fn fact_record_load(
        &self,
        fact_key: Path<String>,
        record_key: Path<String>,
        add_req: Json<StatsFactRecordLoadReq>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Void> {
        let funs = request.tardis_fun_inst();
        stats_record_serv::fact_record_load(&fact_key.0, &record_key.0, add_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Delete Fact Record
    #[oai(path = "/fact/:fact_key/:record_key", method = "delete")]
    async fn fact_record_delete(&self, fact_key: Path<String>, record_key: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let funs = request.tardis_fun_inst();
        stats_record_serv::fact_record_delete(&fact_key.0, &record_key.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Load Fact Records
    #[oai(path = "/fact/:fact_key/batch/load", method = "put")]
    async fn fact_records_load(
        &self,
        fact_key: Path<String>,
        add_req: Json<Vec<StatsFactRecordsLoadReq>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Void> {
        let funs = request.tardis_fun_inst();
        stats_record_serv::fact_records_load(&fact_key.0, add_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Delete Fact Records
    #[oai(path = "/fact/:fact_key/batch/remove", method = "put")]
    async fn fact_records_delete(&self, fact_key: Path<String>, delete_req: Json<Vec<String>>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let funs = request.tardis_fun_inst();
        stats_record_serv::fact_records_delete(&fact_key.0, &delete_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Delete Fact Records
    #[oai(path = "/fact/:fact_key/dim/:dim_conf_key/batch/remove", method = "put")]
    async fn fact_records_delete_by_dim_key(
        &self,
        fact_key: Path<String>,
        dim_conf_key: Path<String>,
        delete_req: Json<StatsDimRecordDeleteReq>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Void> {
        let funs = request.tardis_fun_inst();
        stats_record_serv::fact_records_delete_by_dim_key(&fact_key.0, &dim_conf_key.0, Some(delete_req.0.key), &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Clean Fact Records
    ///
    /// Note:This operation will physically delete all fact records and cannot be recovered, please use caution!
    #[oai(path = "/fact/:fact_key/batch/clean", method = "delete")]
    async fn fact_records_clean(&self, fact_key: Path<String>, before_ct: Query<Option<DateTime<Utc>>>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let funs = request.tardis_fun_inst();
        stats_record_serv::fact_records_clean(&fact_key.0, before_ct.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Add Dimension Record
    #[oai(path = "/dim/:dim_key", method = "put")]
    async fn dim_record_add(&self, dim_key: Path<String>, add_req: Json<StatsDimRecordAddReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let funs = request.tardis_fun_inst();
        stats_record_serv::dim_record_add(dim_key.0, add_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Find Dimension Records
    #[oai(path = "/dim/:dim_key", method = "get")]
    async fn dim_record_paginate(
        &self,
        dim_key: Path<String>,
        show_name: Query<Option<String>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<TardisPage<Value>> {
        let funs = request.tardis_fun_inst();
        let resp = stats_record_serv::dim_record_paginate(dim_key.0, None, show_name.0, page_number.0, page_size.0, desc_by_create.0, desc_by_update.0, &funs, &ctx.0).await?;
        TardisResp::ok(resp)
    }

    /// Delete Dimension Record
    #[oai(path = "/dim/:dim_key/remove", method = "put")]
    async fn dim_record_delete(&self, dim_key: Path<String>, delete_req: Json<StatsDimRecordDeleteReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let funs = request.tardis_fun_inst();
        stats_record_serv::dim_record_delete(dim_key.0, delete_req.0.key, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }
}
