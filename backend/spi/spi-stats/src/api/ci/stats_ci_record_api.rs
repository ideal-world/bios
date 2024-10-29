use bios_basic::rbum::dto::rbum_cert_conf_dto::RbumCertConfAddReq;
use bios_basic::rbum::dto::rbum_cert_dto::RbumCertAddReq;
use tardis::chrono::{DateTime, Utc};
use tardis::serde_json::Value;
use tardis::web::context_extractor::TardisContextExtractor;

use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::{Path, Query};
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use crate::dto::stats_conf_dto::{StatsSyncDbConfigAddReq, StatsSyncDbConfigInfoResp, StatsSyncDbConfigModifyReq};
use crate::dto::stats_record_dto::{StatsDimRecordAddReq, StatsDimRecordDeleteReq, StatsFactRecordLoadReq, StatsFactRecordsLoadReq};
use crate::serv::{stats_record_serv, stats_sync_serv};

#[derive(Clone)]
pub struct StatsCiRecordApi;
#[derive(Clone)]
pub struct StatsCiSyncApi;

/// Interface Console Statistics Record API
///
/// 统计记录接口
#[poem_openapi::OpenApi(prefix_path = "/ci/record", tag = "bios_basic::ApiTag::Interface")]
impl StatsCiRecordApi {
    /// Load Fact Record
    ///
    /// 加载事实记录
    #[oai(path = "/fact/:fact_key/latest/:record_key", method = "get")]
    async fn get_fact_record_latest(&self, fact_key: Path<String>, record_key: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<serde_json::Value> {
        let funs = crate::get_tardis_inst();
        TardisResp::ok(stats_record_serv::get_fact_record_latest(&fact_key.0, [record_key.0.as_str()], &funs, &ctx.0).await?.pop().unwrap_or_default())
    }

    /// Load Fact Record
    ///
    /// 加载事实记录
    #[oai(path = "/fact/:fact_key/latest", method = "get")]
    async fn get_fact_record_latest_many(&self, fact_key: Path<String>, record_keys: Query<String>, ctx: TardisContextExtractor) -> TardisApiResult<Vec<serde_json::Value>> {
        let funs = crate::get_tardis_inst();
        TardisResp::ok(stats_record_serv::get_fact_record_latest(&fact_key.0, record_keys.0.split(','), &funs, &ctx.0).await?)
    }

    /// Get Fact Record Paginated
    ///
    /// 获取事实记录分页
    #[oai(path = "/fact/:fact_key/:record_key", method = "get")]
    async fn get_fact_record_paginated(
        &self,
        fact_key: Path<String>,
        record_key: Path<String>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        desc_by_create: Query<Option<bool>>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<serde_json::Value>> {
        let funs = crate::get_tardis_inst();
        TardisResp::ok(stats_record_serv::get_fact_record_paginated(&fact_key.0, &record_key.0, page_number.0, page_size.0, desc_by_create.0, &funs, &ctx.0).await?)
    }

    /// Load Fact Record
    ///
    /// 加载事实记录
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

    /// Delete Fact Record
    ///
    /// 删除事实记录
    #[oai(path = "/fact/:fact_key/:record_key", method = "delete")]
    async fn fact_record_delete(&self, fact_key: Path<String>, record_key: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_record_serv::fact_record_delete(&fact_key.0, &record_key.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Load Fact Records
    ///
    /// 批量加载事实记录
    #[oai(path = "/fact/:fact_key/batch/load", method = "put")]
    async fn fact_records_load(&self, fact_key: Path<String>, add_req: Json<Vec<StatsFactRecordsLoadReq>>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_record_serv::fact_records_load(&fact_key.0, add_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Delete Fact Records
    ///
    /// 批量删除事实记录
    #[oai(path = "/fact/:fact_key/batch/remove", method = "put")]
    async fn fact_records_delete(&self, fact_key: Path<String>, delete_req: Json<Vec<String>>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_record_serv::fact_records_delete(&fact_key.0, &delete_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Delete Fact Records
    /// Note:This operation will physically delete all fact records and cannot be recovered, please use caution!
    ///
    /// 删除事实记录
    /// 注意:此操作将物理删除所有事实记录，且无法恢复，请谨慎使用!
    #[oai(path = "/fact/:fact_key/ownership/remove", method = "delete")]
    async fn fact_records_delete_by_ownership(&self, fact_key: Path<String>, own_paths: Query<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_record_serv::fact_records_delete_by_ownership(&fact_key.0, &own_paths.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Delete Fact Records
    ///
    /// 删除事实记录
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

    /// Clean Fact Records
    /// Note:This operation will physically delete all fact records and cannot be recovered, please use caution!
    ///
    /// 清空事实记录
    /// 注意:此操作将物理删除所有事实记录，且无法恢复，请谨慎使用!
    #[oai(path = "/fact/:fact_key/batch/clean", method = "delete")]
    async fn fact_records_clean(&self, fact_key: Path<String>, before_ct: Query<Option<DateTime<Utc>>>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_record_serv::fact_records_clean(&fact_key.0, before_ct.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Add Dimension Record
    ///
    /// 添加维度记录
    #[oai(path = "/dim/:dim_key", method = "put")]
    async fn dim_record_add(&self, dim_key: Path<String>, add_req: Json<StatsDimRecordAddReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_record_serv::dim_record_add(dim_key.0, add_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Find Dimension Records
    ///
    /// 查找维度记录
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
    ///
    /// 删除维度记录
    #[oai(path = "/dim/:dim_key/remove", method = "put")]
    async fn dim_record_delete(&self, dim_key: Path<String>, delete_req: Json<StatsDimRecordDeleteReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_record_serv::dim_record_delete(dim_key.0, delete_req.0.key, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Delete Dimension Record
    /// Note:This operation is a real delete and cannot be recovered, please use caution!
    ///
    /// 删除维度记录
    /// 注意:该操作是进行真实删除，不可恢复，请谨慎使用!
    #[oai(path = "/dim/:dim_key/remove", method = "delete")]
    async fn dim_record_real_delete(&self, dim_key: Path<String>, delete_req: Json<StatsDimRecordDeleteReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_record_serv::dim_record_real_delete(dim_key.0, delete_req.0.key, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }
}

/// Interface Console Statistics Sync API
///
/// 统计同步接口
#[poem_openapi::OpenApi(prefix_path = "/ci", tag = "bios_basic::ApiTag::Interface")]
impl StatsCiSyncApi {
    /// Sync Fact Record
    ///
    /// 同步事实记录
    #[oai(path = "/fact/:fact_key/sync", method = "put")]
    async fn fact_record_sync(&self, fact_key: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_sync_serv::fact_record_sync(&fact_key.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Sync Fact Column Record
    ///
    /// 同步事实列记录
    #[oai(path = "/fact/:fact_key/col/:col_key/sync", method = "put")]
    async fn fact_col_record_sync(&self, fact_key: Path<String>, col_key: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_sync_serv::fact_col_record_sync(&fact_key.0, &col_key.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Add Sync DateBase Config
    ///
    /// 添加同步数据库配置
    #[oai(path = "/sync/db/config", method = "post")]
    async fn db_config_add(&self, add_req: Json<StatsSyncDbConfigAddReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_sync_serv::db_config_add(add_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Modify Sync DateBase Config
    ///
    /// 修改同步数据库配置
    #[oai(path = "/sync/db/config", method = "put")]
    async fn db_config_modify(&self, modify_req: Json<StatsSyncDbConfigModifyReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_sync_serv::db_config_modify(modify_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Paginate Sync DateBase Config
    ///
    /// 分页查询同步数据库配置
    #[oai(path = "/sync/db/config", method = "get")]
    async fn db_config_paginate(&self, page_number: Query<u32>, page_size: Query<u32>, ctx: TardisContextExtractor) -> TardisApiResult<TardisPage<StatsSyncDbConfigInfoResp>> {
        let funs = crate::get_tardis_inst();
        TardisResp::ok(stats_sync_serv::db_config_paginate(page_number.0, page_size.0, &funs, &ctx.0).await?)
    }
}
