use tardis::web::context_extractor::TardisContextExtractor;

use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::{Path, Query};
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use crate::dto::stats_conf_dto::{
    StatsConfDimAddReq, StatsConfDimInfoResp, StatsConfDimModifyReq, StatsConfFactAddReq, StatsConfFactColAddReq, StatsConfFactColAggWithDimInfoResp, StatsConfFactColInfoResp,
    StatsConfFactColModifyReq, StatsConfFactInfoResp, StatsConfFactModifyReq,
};
use crate::serv::stats_conf_serv;
use crate::stats_enumeration::StatsFactColKind;

#[derive(Clone)]
pub struct StatsCiConfApi;

/// Interface Console Statistics Configuration API
#[poem_openapi::OpenApi(prefix_path = "/ci/conf", tag = "bios_basic::ApiTag::Interface")]
impl StatsCiConfApi {
    /// Add Dimension Configuration
    #[oai(path = "/dim", method = "put")]
    async fn dim_add(&self, add_req: Json<StatsConfDimAddReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_conf_serv::dim_add(&add_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Modify Dimension Configuration
    #[oai(path = "/dim/:dim_key", method = "patch")]
    async fn dim_modify(&self, dim_key: Path<String>, modify_req: Json<StatsConfDimModifyReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_conf_serv::dim_modify(&dim_key.0, &modify_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Delete Dimension Configuration
    #[oai(path = "/dim/:dim_key", method = "delete")]
    async fn dim_delete(&self, dim_key: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_conf_serv::dim_delete(&dim_key.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Find Dimension Configurations
    #[oai(path = "/dim", method = "get")]
    async fn dim_paginate(
        &self,
        key: Query<Option<String>>,
        show_name: Query<Option<String>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<StatsConfDimInfoResp>> {
        let funs = crate::get_tardis_inst();
        let resp = stats_conf_serv::dim_paginate(key.0, show_name.0, page_number.0, page_size.0, desc_by_create.0, desc_by_update.0, &funs, &ctx.0).await?;
        TardisResp::ok(resp)
    }

    /// Add Fact Configuration
    #[oai(path = "/fact", method = "put")]
    async fn fact_add(&self, add_req: Json<StatsConfFactAddReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_conf_serv::fact_add(&add_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Modify Fact Configuration
    #[oai(path = "/fact/:fact_key", method = "patch")]
    async fn fact_modify(&self, fact_key: Path<String>, modify_req: Json<StatsConfFactModifyReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_conf_serv::fact_modify(&fact_key.0, &modify_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Delete Fact Configuration
    #[oai(path = "/fact/:fact_key", method = "delete")]
    async fn fact_delete(&self, fact_key: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_conf_serv::fact_delete(&fact_key.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Find Fact Configurations
    #[oai(path = "/fact", method = "get")]
    async fn fact_paginate(
        &self,
        key: Query<Option<String>>,
        show_name: Query<Option<String>>,
        is_online: Query<Option<bool>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<StatsConfFactInfoResp>> {
        let funs = crate::get_tardis_inst();
        let resp = stats_conf_serv::fact_paginate(
            key.0,
            show_name.0,
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
    #[oai(path = "/fact/:fact_key/col", method = "put")]
    async fn fact_col_add(&self, fact_key: Path<String>, add_req: Json<StatsConfFactColAddReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_conf_serv::fact_col_add(&fact_key.0, &add_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Modify Fact Column Configuration
    #[oai(path = "/fact/:fact_key/col/:fact_col_key", method = "patch")]
    async fn fact_col_modify(
        &self,
        fact_key: Path<String>,
        fact_col_key: Path<String>,
        modify_req: Json<StatsConfFactColModifyReq>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_conf_serv::fact_col_modify(&fact_key.0, &fact_col_key.0, &modify_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Delete Fact Column Configuration
    #[oai(path = "/fact/:fact_key/col/:fact_col_key", method = "delete")]
    async fn fact_col_delete(&self, fact_key: Path<String>, fact_col_key: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_conf_serv::fact_col_delete(&fact_key.0, Some(fact_col_key.0.as_str()), None, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Delete All Column Configuration
    #[oai(path = "/fact/:fact_key/kind/:kind", method = "delete")]
    async fn fact_col_delete_by_kind(&self, fact_key: Path<String>, kind: Path<StatsFactColKind>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_conf_serv::fact_col_delete(&fact_key.0, None, Some(kind.0), &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Find Fact Column Configurations
    #[oai(path = "/fact/:fact_key/col", method = "get")]
    async fn fact_col_paginate(
        &self,
        fact_key: Path<String>,
        key: Query<Option<String>>,
        show_name: Query<Option<String>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<StatsConfFactColInfoResp>> {
        let funs = crate::get_tardis_inst();
        let resp = stats_conf_serv::fact_col_paginate(
            fact_key.0,
            key.0,
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

    /// Find Fact Column Configurations
    #[oai(path = "/fact/:fact_key/dim/:dim_key/col", method = "get")]
    async fn fact_col_paginate_agg_with_dim(
        &self,
        fact_key: Path<String>,
        dim_key: Path<String>,
        key: Query<Option<String>>,
        show_name: Query<Option<String>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<StatsConfFactColAggWithDimInfoResp>> {
        let funs = crate::get_tardis_inst();
        let resp = stats_conf_serv::fact_col_paginate_agg_with_dim(
            fact_key.0,
            dim_key.0,
            key.0,
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

    /// Online dimension configuration
    #[oai(path = "/dim/:dim_key/online", method = "put")]
    async fn dim_online(&self, dim_key: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_conf_serv::dim_online(&dim_key.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Online Fact Configuration
    #[oai(path = "/fact/:fact_key/online", method = "put")]
    async fn fact_online(&self, fact_key: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_conf_serv::fact_online(&fact_key.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }
}
