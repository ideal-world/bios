use tardis::web::context_extractor::TardisContextExtractor;

use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::{Path, Query};
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use crate::dto::stats_conf_dto::{
    StatsConfDimAddReq, StatsConfDimGroupAddReq, StatsConfDimGroupInfoResp, StatsConfDimGroupModifyReq, StatsConfDimInfoResp, StatsConfDimModifyReq, StatsConfFactAddReq,
    StatsConfFactColAddReq, StatsConfFactColInfoResp, StatsConfFactColModifyReq, StatsConfFactInfoResp, StatsConfFactModifyReq, StatsSyncDbConfigAddReq, StatsSyncDbConfigInfoResp,
    StatsSyncDbConfigModifyReq,
};
use crate::serv::{stats_conf_dim_group_serv, stats_conf_dim_serv, stats_conf_fact_col_serv, stats_conf_fact_serv, stats_sync_serv};
use crate::stats_enumeration::StatsFactColKind;

#[derive(Clone)]
pub struct StatsCiConfApi;

/// Interface Console Statistics Configuration API
///
/// 接口控制台统计配置 API
#[poem_openapi::OpenApi(prefix_path = "/ci/conf", tag = "bios_basic::ApiTag::Interface")]
impl StatsCiConfApi {
    /// Add Dimension Group Configuration
    ///
    /// 添加维度组配置
    #[oai(path = "/dim/group", method = "put")]
    async fn dim_group_add(&self, add_req: Json<StatsConfDimGroupAddReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_conf_dim_group_serv::add(&add_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Modify Dimension Group Configuration
    ///
    /// 修改维度组配置
    #[oai(path = "/dim/group/:dim_group_key", method = "patch")]
    async fn dim_group_modify(&self, dim_group_key: Path<String>, modify_req: Json<StatsConfDimGroupModifyReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_conf_dim_group_serv::modify(&dim_group_key.0, &modify_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Find Dimension Group Configurations
    ///
    /// 查询维度组配置
    #[oai(path = "/dim/group", method = "get")]
    async fn dim_group_paginate(
        &self,
        page_number: Query<u32>,
        page_size: Query<u32>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<StatsConfDimGroupInfoResp>> {
        let funs = crate::get_tardis_inst();
        TardisResp::ok(stats_conf_dim_group_serv::paginate(page_number.0, page_size.0, desc_by_create.0, desc_by_update.0, &funs, &ctx.0).await?)
    }

    /// Add Dimension Configuration
    ///
    /// 添加维度配置
    #[oai(path = "/dim", method = "put")]
    async fn dim_add(&self, add_req: Json<StatsConfDimAddReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_conf_dim_serv::add(&add_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Modify Dimension Configuration
    ///
    /// 修改维度配置
    #[oai(path = "/dim/:dim_key", method = "patch")]
    async fn dim_modify(&self, dim_key: Path<String>, modify_req: Json<StatsConfDimModifyReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_conf_dim_serv::modify(&dim_key.0, &modify_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Delete Dimension Configuration
    ///
    /// 删除维度配置
    #[oai(path = "/dim/:dim_key", method = "delete")]
    async fn dim_delete(&self, dim_key: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_conf_dim_serv::delete(&dim_key.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Find Dimension Configurations
    ///
    /// 查询维度配置
    #[oai(path = "/dim", method = "get")]
    async fn dim_paginate(
        &self,
        key: Query<Option<String>>,
        group_key: Query<Option<String>>,
        group_is_empty: Query<Option<bool>>,
        show_name: Query<Option<String>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<StatsConfDimInfoResp>> {
        let funs = crate::get_tardis_inst();
        let resp = stats_conf_dim_serv::paginate(
            key.0,
            group_key.0,
            group_is_empty.0,
            show_name.0,
            page_number.0,
            page_size.0,
            desc_by_create.0,
            desc_by_update.0,
            &funs,
            &ctx.0,
        )
        .await?;
        TardisResp::ok(resp)
    }

    /// Add Fact Configuration
    ///
    /// 添加事实配置
    #[oai(path = "/fact", method = "put")]
    async fn fact_add(&self, add_req: Json<StatsConfFactAddReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_conf_fact_serv::add(&add_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Modify Fact Configuration
    ///
    /// 修改事实配置
    #[oai(path = "/fact/:fact_key", method = "patch")]
    async fn fact_modify(&self, fact_key: Path<String>, modify_req: Json<StatsConfFactModifyReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_conf_fact_serv::modify(&fact_key.0, &modify_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Delete Fact Configuration
    ///
    /// 删除事实配置
    #[oai(path = "/fact/:fact_key", method = "delete")]
    async fn fact_delete(&self, fact_key: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_conf_fact_serv::delete(&fact_key.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Find Fact Configurations
    ///
    /// 查询事实配置
    #[oai(path = "/fact", method = "get")]
    async fn fact_paginate(
        &self,
        keys: Query<Option<String>>,
        show_name: Query<Option<String>>,
        dim_rel_conf_dim_keys: Query<Option<String>>,
        is_online: Query<Option<bool>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<StatsConfFactInfoResp>> {
        let funs = crate::get_tardis_inst();
        let keys = keys.0.map(|key| key.split(',').map(|r| r.to_string()).collect());
        let dim_rel_conf_dim_keys = dim_rel_conf_dim_keys.0.map(|keys| keys.split(',').map(|r| r.to_string()).collect());
        let resp = stats_conf_fact_serv::paginate(
            keys,
            show_name.0,
            dim_rel_conf_dim_keys,
            is_online.0,
            page_number.0,
            page_size.0,
            desc_by_create.0,
            desc_by_update.0,
            &funs,
            &ctx.0,
        )
        .await?;
        TardisResp::ok(resp)
    }

    /// Add Fact Column Configuration
    ///
    /// 添加事实列配置
    #[oai(path = "/fact/:fact_key/col", method = "put")]
    async fn fact_col_add(&self, fact_key: Path<String>, add_req: Json<StatsConfFactColAddReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_conf_fact_col_serv::add(&fact_key.0, &add_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Modify Fact Column Configuration
    ///
    /// 修改事实列配置
    #[oai(path = "/fact/:fact_key/col/:fact_col_key", method = "patch")]
    async fn fact_col_modify(
        &self,
        fact_key: Path<String>,
        fact_col_key: Path<String>,
        modify_req: Json<StatsConfFactColModifyReq>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_conf_fact_col_serv::modify(&fact_key.0, &fact_col_key.0, &modify_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Delete Fact Column Configuration
    ///
    /// 删除事实列配置
    #[oai(path = "/fact/:fact_key/col/:fact_col_key", method = "delete")]
    async fn fact_col_delete(
        &self,
        fact_key: Path<String>,
        fact_col_key: Path<String>,
        rel_external_id: Query<Option<String>>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_conf_fact_col_serv::delete(&fact_key.0, Some(fact_col_key.0.as_str()), rel_external_id.0, None, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Delete Fact Column Configuration
    ///
    /// 删除事实列配置
    #[oai(path = "/fact/:fact_key/col/:fact_col_key/kind/:kind", method = "delete")]
    async fn fact_col_kind_delete(
        &self,
        fact_key: Path<String>,
        fact_col_key: Path<String>,
        kind: Path<StatsFactColKind>,
        rel_external_id: Query<Option<String>>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_conf_fact_col_serv::delete(&fact_key.0, Some(fact_col_key.0.as_str()), rel_external_id.0, Some(kind.0), &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Delete All Column Configuration
    ///
    /// 删除所有列配置
    #[oai(path = "/fact/:fact_key/kind/:kind", method = "delete")]
    async fn fact_col_delete_by_kind(
        &self,
        fact_key: Path<String>,
        kind: Path<StatsFactColKind>,
        rel_external_id: Query<Option<String>>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_conf_fact_col_serv::delete(&fact_key.0, None, rel_external_id.0, Some(kind.0), &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Find Fact Column Configurations
    ///
    /// 查询事实列配置
    #[oai(path = "/fact/:fact_key/col", method = "get")]
    async fn fact_col_paginate(
        &self,
        fact_key: Path<String>,
        key: Query<Option<String>>,
        group_key: Query<Option<String>>,
        show_name: Query<Option<String>>,
        rel_external_id: Query<Option<String>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<StatsConfFactColInfoResp>> {
        let funs = crate::get_tardis_inst();
        let resp = stats_conf_fact_col_serv::paginate(
            Some(fact_key.0),
            key.0,
            None,
            group_key.0,
            show_name.0,
            rel_external_id.0,
            page_number.0,
            page_size.0,
            desc_by_create.0,
            desc_by_update.0,
            &funs,
            &ctx.0,
        )
        .await?;
        TardisResp::ok(resp)
    }

    /// Find Fact Column Configurations
    ///
    /// 查询事实列配置
    #[oai(path = "/dim/:dim_key/col", method = "get")]
    async fn fact_col_paginate_by_dim(
        &self,
        dim_key: Path<String>,
        key: Query<Option<String>>,
        fact_key: Query<Option<String>>,
        group_key: Query<Option<String>>,
        show_name: Query<Option<String>>,
        rel_external_id: Query<Option<String>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<StatsConfFactColInfoResp>> {
        let funs = crate::get_tardis_inst();
        let resp = stats_conf_fact_col_serv::paginate(
            fact_key.0,
            key.0,
            Some(dim_key.0),
            group_key.0,
            show_name.0,
            rel_external_id.0,
            page_number.0,
            page_size.0,
            desc_by_create.0,
            desc_by_update.0,
            &funs,
            &ctx.0,
        )
        .await?;
        TardisResp::ok(resp)
    }

    /// Online dimension configuration
    ///
    /// 上线维度配置
    #[oai(path = "/dim/:dim_key/online", method = "put")]
    async fn dim_online(&self, dim_key: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_conf_dim_serv::create_inst(&dim_key.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Online Fact Configuration
    ///
    /// 上线事实配置
    #[oai(path = "/fact/:fact_key/online", method = "put")]
    async fn fact_online(&self, fact_key: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_conf_fact_serv::create_inst(&fact_key.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Add Sync DateBase Config
    ///
    /// 添加同步数据库配置
    #[oai(path = "/sync/db", method = "post")]
    async fn db_config_add(&self, add_req: Json<StatsSyncDbConfigAddReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_sync_serv::db_config_add(add_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Modify Sync DateBase Config
    ///
    /// 修改同步数据库配置
    #[oai(path = "/sync/db", method = "put")]
    async fn db_config_modify(&self, modify_req: Json<StatsSyncDbConfigModifyReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_sync_serv::db_config_modify(modify_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// List Sync DateBase Config
    ///
    /// 查询同步数据库配置
    #[oai(path = "/sync/db", method = "get")]
    async fn db_config_list(&self, ctx: TardisContextExtractor) -> TardisApiResult<Vec<StatsSyncDbConfigInfoResp>> {
        let funs = crate::get_tardis_inst();
        TardisResp::ok(stats_sync_serv::db_config_list(&funs, &ctx.0).await?)
    }
}
