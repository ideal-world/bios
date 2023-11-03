use tardis::chrono::{DateTime, Utc};
use tardis::serde_json::Value;
use tardis::web::context_extractor::TardisContextExtractor;

use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::{Path, Query};
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use crate::dto::stats_record_dto::{StatsDimRecordAddReq, StatsDimRecordDeleteReq, StatsFactRecordLoadReq, StatsFactRecordsLoadReq};
use crate::serv::stats_record_serv;

#[derive(Clone)]
pub struct StatsCiRecordApi;

/// Interface Console Statistics Record API
#[poem_openapi::OpenApi(prefix_path = "/ci/record", tag = "bios_basic::ApiTag::Interface")]
impl StatsCiRecordApi {
    /// Load Fact Record
    #[oai(path = "/fact/:fact_key/latest/:record_key", method = "get")]
    async fn get_fact_record_latest(&self, fact_key: Path<String>, record_key: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<serde_json::Value> {
        let funs = crate::get_tardis_inst();
        TardisResp::ok(stats_record_serv::get_fact_record_latest(&fact_key.0, [record_key.0.as_str()], &funs, &ctx.0).await?.pop().unwrap_or_default())
    }

    /// Load Fact Record
    #[oai(path = "/fact/:fact_key/latest", method = "get")]
    async fn get_fact_record_latest_many(&self, fact_key: Path<String>, record_keys: Query<String>, ctx: TardisContextExtractor) -> TardisApiResult<Vec<serde_json::Value>> {
        let funs = crate::get_tardis_inst();
        TardisResp::ok(stats_record_serv::get_fact_record_latest(&fact_key.0, record_keys.0.split(','), &funs, &ctx.0).await?)
    }

    /// Get Fact Record Pagenated
    #[oai(path = "/fact/:fact_key/:record_key", method = "get")]
    async fn get_fact_record_pagenated(
        &self,
        fact_key: Path<String>,
        record_key: Path<String>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        desc_by_create: Query<Option<bool>>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<serde_json::Value>> {
        let funs = crate::get_tardis_inst();
        TardisResp::ok(stats_record_serv::get_fact_record_pagenated(&fact_key.0, &record_key.0, page_number.0, page_size.0, desc_by_create.0, &funs, &ctx.0).await?)
    }

    /// Load Fact Record
    #[oai(path = "/fact/:fact_key/:record_key", method = "put")]
    async fn fact_record_load(
        &self,
        fact_key: Path<String>,
        record_key: Path<String>,
        add_req: Json<StatsFactRecordLoadReq>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_record_serv::fact_record_load(&fact_key.0, &record_key.0, add_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// 删除事实记录
    /// Delete Fact Record
    #[oai(path = "/fact/:fact_key/:record_key", method = "delete")]
    async fn fact_record_delete(&self, fact_key: Path<String>, record_key: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_record_serv::fact_record_delete(&fact_key.0, &record_key.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// 删除事实记录
    /// Load Fact Records
    #[oai(path = "/fact/:fact_key/batch/load", method = "put")]
    async fn fact_records_load(&self, fact_key: Path<String>, add_req: Json<Vec<StatsFactRecordsLoadReq>>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_record_serv::fact_records_load(&fact_key.0, add_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// 删除事实记录
    /// Delete Fact Records
    #[oai(path = "/fact/:fact_key/batch/remove", method = "put")]
    async fn fact_records_delete(&self, fact_key: Path<String>, delete_req: Json<Vec<String>>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_record_serv::fact_records_delete(&fact_key.0, &delete_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// 删除事实记录
    /// 注意:此操作将物理删除所有事实记录，且无法恢复，请谨慎使用!
    /// Delete Fact Records
    /// Note:This operation will physically delete all fact records and cannot be recovered, please use caution!
    #[oai(path = "/fact/:fact_key/ownership/remove", method = "delete")]
    async fn fact_records_delete_by_ownership(&self, fact_key: Path<String>, own_paths: Query<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_record_serv::fact_records_delete_by_ownership(&fact_key.0, &own_paths.0, &funs, &ctx.0).await?;
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
    ) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_record_serv::fact_records_delete_by_dim_key(&fact_key.0, &dim_conf_key.0, Some(delete_req.0.key), &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// 清空事实记录
    /// 注意:此操作将物理删除所有事实记录，且无法恢复，请谨慎使用!
    /// Clean Fact Records
    /// Note:This operation will physically delete all fact records and cannot be recovered, please use caution!
    #[oai(path = "/fact/:fact_key/batch/clean", method = "delete")]
    async fn fact_records_clean(&self, fact_key: Path<String>, before_ct: Query<Option<DateTime<Utc>>>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_record_serv::fact_records_clean(&fact_key.0, before_ct.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Add Dimension Record
    #[oai(path = "/dim/:dim_key", method = "put")]
    async fn dim_record_add(&self, dim_key: Path<String>, add_req: Json<StatsDimRecordAddReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
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
    ) -> TardisApiResult<TardisPage<Value>> {
        let funs = crate::get_tardis_inst();
        let resp = stats_record_serv::dim_record_paginate(dim_key.0, None, show_name.0, page_number.0, page_size.0, desc_by_create.0, desc_by_update.0, &funs, &ctx.0).await?;
        TardisResp::ok(resp)
    }

    /// Delete Dimension Record
    #[oai(path = "/dim/:dim_key/remove", method = "put")]
    async fn dim_record_delete(&self, dim_key: Path<String>, delete_req: Json<StatsDimRecordDeleteReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_record_serv::dim_record_delete(dim_key.0, delete_req.0.key, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Delete Dimension Record
    #[oai(path = "/dim/:dim_key/remove", method = "delete")]
    async fn dim_record_real_delete(&self, dim_key: Path<String>, delete_req: Json<StatsDimRecordDeleteReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_record_serv::dim_record_real_delete(dim_key.0, delete_req.0.key, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }
}
