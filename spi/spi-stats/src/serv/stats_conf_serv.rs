use std::collections::HashMap;
use std::sync::Arc;

use bios_basic::spi::spi_constants;
use bios_basic::spi::spi_funs::SpiBsInstExtractor;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::tokio::sync::RwLock;
use tardis::web::web_resp::TardisPage;
use tardis::TardisFunsInst;
use lazy_static::lazy_static;

use crate::dto::stats_conf_dto::{
    StatsConfDimAddReq, StatsConfDimInfoResp, StatsConfDimModifyReq, StatsConfFactAddReq, StatsConfFactColAddReq, StatsConfFactColInfoResp, StatsConfFactColModifyReq,
    StatsConfFactInfoResp, StatsConfFactModifyReq,
};
use crate::stats_initializer;

use super::pg;

lazy_static! {
    pub static ref CONF_FACTS: Arc<RwLock<HashMap<String, (StatsConfFactInfoResp, Vec<StatsConfFactColInfoResp>)>>> = Arc::new(RwLock::new(HashMap::new()));
    pub static ref CONF_DIMS: Arc<RwLock<HashMap<String, StatsConfDimInfoResp>>> = Arc::new(RwLock::new(HashMap::new()));
}

pub async fn dim_add(add_req: &StatsConfDimAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    match funs.init(ctx, true, stats_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_conf_dim_serv::add(add_req, &funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn dim_modify(key: &str, modify_req: &StatsConfDimModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    match funs.init(ctx, true, stats_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_conf_dim_serv::modify(key, modify_req, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn dim_delete(key: &String, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    match funs.init(ctx, true, stats_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_conf_dim_serv::delete(key, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn dim_paginate(
    key: Option<String>,
    show_name: Option<String>,
    page_number: u64,
    page_size: u64,
    desc_by_create: Option<bool>,
    desc_by_update: Option<bool>,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
) -> TardisResult<TardisPage<StatsConfDimInfoResp>> {
    match funs.init(ctx, true, stats_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_conf_dim_serv::paginate(key, show_name, page_number, page_size, desc_by_create, desc_by_update, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn fact_add(add_req: &StatsConfFactAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    match funs.init(ctx, true, stats_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_conf_fact_serv::add(add_req, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn fact_modify(key: &str, modify_req: &StatsConfFactModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    match funs.init(ctx, true, stats_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_conf_fact_serv::modify(key, modify_req, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn fact_delete(key: &String, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    match funs.init(ctx, true, stats_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_conf_fact_serv::delete(key, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn fact_paginate(
    key: Option<String>,
    show_name: Option<String>,
    page_number: u64,
    page_size: u64,
    desc_by_create: Option<bool>,
    desc_by_update: Option<bool>,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
) -> TardisResult<TardisPage<StatsConfFactInfoResp>> {
    match funs.init(ctx, true, stats_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_conf_fact_serv::paginate(key, show_name, page_number, page_size, desc_by_create, desc_by_update, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn fact_col_add(fact_key: &str, add_req: &StatsConfFactColAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    match funs.init(ctx, true, stats_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_conf_fact_col_serv::add(fact_key, add_req, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn fact_col_modify(key: &str, modify_req: &StatsConfFactColModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    match funs.init(ctx, true, stats_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_conf_fact_col_serv::modify(key, modify_req, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn fact_col_delete(key: &String, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    match funs.init(ctx, true, stats_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_conf_fact_col_serv::delete(key, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn fact_col_paginate(
    key: Option<String>,
    show_name: Option<String>,
    rel_conf_fact_key: Option<String>,
    page_number: u64,
    page_size: u64,
    desc_by_create: Option<bool>,
    desc_by_update: Option<bool>,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
) -> TardisResult<TardisPage<StatsConfFactColInfoResp>> {
    match funs.init(ctx, true, stats_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => {
            pg::stats_pg_conf_fact_col_serv::paginate(key, show_name, rel_conf_fact_key, page_number, page_size, desc_by_create, desc_by_update, funs, ctx).await
        }
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn dim_confirm(key: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    match funs.init(ctx, true, stats_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_conf_dim_serv::create_inst(key, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn fact_confirm(key: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    match funs.init(ctx, true, stats_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_conf_fact_serv::create_inst(key, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}
