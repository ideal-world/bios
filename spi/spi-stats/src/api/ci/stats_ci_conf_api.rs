use bios_basic::spi::spi_funs::SpiTardisFunInstExtractor;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::{Path, Query};
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use crate::dto::stats_conf_dto::{
    StatsConfDimAddReq, StatsConfDimInfoResp, StatsConfDimModifyReq, StatsConfFactAddReq, StatsConfFactColAddReq, StatsConfFactColInfoResp, StatsConfFactColModifyReq,
    StatsConfFactInfoResp, StatsConfFactModifyReq,
};
use crate::serv::stats_conf_serv;

pub struct StatsCiConfApi;

/// Interface Console Statistics Configuration API
#[poem_openapi::OpenApi(prefix_path = "/ci/conf", tag = "bios_basic::ApiTag::Interface")]
impl StatsCiConfApi {
    /// Add Dimension Configuration
    #[oai(path = "/dim", method = "post")]
    async fn dim_add(&self, add_req: Json<StatsConfDimAddReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let funs = request.tardis_fun_inst();
        stats_conf_serv::dim_add(&add_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Modify Dimension Configuration
    #[oai(path = "/dim/:dim_key", method = "patch")]
    async fn dim_modify(&self, dim_key: Path<String>, modify_req: Json<StatsConfDimModifyReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let funs = request.tardis_fun_inst();
        stats_conf_serv::dim_modify(&dim_key.0, &modify_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Delete Dimension Configuration
    #[oai(path = "/dim/:dim_key", method = "delete")]
    async fn dim_delete(&self, dim_key: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let funs = request.tardis_fun_inst();
        stats_conf_serv::dim_delete(&dim_key.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Find Dimension Configurations
    #[oai(path = "/dim", method = "get")]
    async fn dim_paginate(
        &self,
        key: Query<Option<String>>,
        show_name: Query<Option<String>>,
        page_number: Query<u64>,
        page_size: Query<u64>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<TardisPage<StatsConfDimInfoResp>> {
        let funs = request.tardis_fun_inst();
        let resp = stats_conf_serv::dim_paginate(key.0, show_name.0, page_number.0, page_size.0, desc_by_create.0, desc_by_update.0, &funs, &ctx.0).await?;
        TardisResp::ok(resp)
    }

    /// Add Fact Configuration
    #[oai(path = "/fact", method = "post")]
    async fn fact_add(&self, add_req: Json<StatsConfFactAddReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let funs = request.tardis_fun_inst();
        stats_conf_serv::fact_add(&add_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Modify Fact Configuration
    #[oai(path = "/fact/:fact_key", method = "patch")]
    async fn fact_modify(&self, fact_key: Path<String>, modify_req: Json<StatsConfFactModifyReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let funs = request.tardis_fun_inst();
        stats_conf_serv::fact_modify(&fact_key.0, &modify_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Delete Fact Configuration
    #[oai(path = "/fact/:fact_key", method = "delete")]
    async fn fact_delete(&self, fact_key: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let funs = request.tardis_fun_inst();
        stats_conf_serv::fact_delete(&fact_key.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Find Fact Configurations
    #[oai(path = "/fact", method = "get")]
    async fn fact_paginate(
        &self,
        key: Query<Option<String>>,
        show_name: Query<Option<String>>,
        page_number: Query<u64>,
        page_size: Query<u64>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<TardisPage<StatsConfFactInfoResp>> {
        let funs = request.tardis_fun_inst();
        let resp = stats_conf_serv::fact_paginate(key.0, show_name.0, page_number.0, page_size.0, desc_by_create.0, desc_by_update.0, &funs, &ctx.0).await?;
        TardisResp::ok(resp)
    }

    /// Add Fact Column Configuration
    #[oai(path = "/fact/:fact_key/col", method = "post")]
    async fn fact_col_add(&self, fact_key: Path<String>, add_req: Json<StatsConfFactColAddReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let funs = request.tardis_fun_inst();
        stats_conf_serv::fact_col_add(&fact_key.0, &add_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Modify Fact Column Configuration
    #[oai(path = "/fact/:fact_key/col/:fact_col_key", method = "patch")]
    async fn fact_col_modify(
        &self,
        fact_col_key: Path<String>,
        modify_req: Json<StatsConfFactColModifyReq>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Void> {
        let funs = request.tardis_fun_inst();
        stats_conf_serv::fact_col_modify(&fact_col_key.0, &modify_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Delete Fact Column Configuration
    #[oai(path = "/fact/:fact_key/col/:fact_col_key", method = "delete")]
    async fn fact_col_delete(&self, fact_col_key: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let funs = request.tardis_fun_inst();
        stats_conf_serv::fact_col_delete(&fact_col_key.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Find Fact Column Configurations
    #[oai(path = "/fact/:fact_key/col", method = "get")]
    async fn fact_col_paginate(
        &self,
        fact_key: Query<Option<String>>,
        show_name: Query<Option<String>>,
        rel_conf_fact_key: Query<Option<String>>,
        page_number: Query<u64>,
        page_size: Query<u64>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<TardisPage<StatsConfFactColInfoResp>> {
        let funs = request.tardis_fun_inst();
        let resp = stats_conf_serv::fact_col_paginate(
            fact_key.0,
            show_name.0,
            rel_conf_fact_key.0,
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

    /// Confirm Dim Configuration
    #[oai(path = "/dim/:dim_key/confirm", method = "put")]
    async fn dim_confirm(&self, dim_key: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let funs = request.tardis_fun_inst();
        stats_conf_serv::dim_confirm(&dim_key.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Confirm Fact Configuration
    #[oai(path = "/fact/:fact_key/confirm", method = "put")]
    async fn fact_confirm(&self, fact_key: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let funs = request.tardis_fun_inst();
        stats_conf_serv::fact_confirm(&fact_key.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }
}
