use bios_basic::spi::spi_constants;
use bios_basic::spi::spi_funs::SpiBsInstExtractor;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::web::web_resp::TardisPage;
use tardis::TardisFunsInst;

use crate::dto::stats_conf_dto::{
    StatsConfDimAddReq, StatsConfDimInfoResp, StatsConfDimModifyReq, StatsConfFactAddReq, StatsConfFactColAddReq, StatsConfFactColInfoResp, StatsConfFactColModifyReq,
    StatsConfFactInfoResp, StatsConfFactModifyReq,
};
use crate::stats_enumeration::StatsFactColKind;
use crate::stats_initializer;

use super::pg;


// TODO FIXME ------------ 使用 spi_dispatch_service , 前后逻辑写到 api 中 ------------

pub async fn dim_add(add_req: &StatsConfDimAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let inst = funs.init(ctx, true, stats_initializer::init_fun).await?;
    match inst.kind_code() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_conf_dim_serv::add(add_req, funs, ctx, &inst).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn dim_modify(dim_conf_key: &str, modify_req: &StatsConfDimModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let inst = funs.init(ctx, true, stats_initializer::init_fun).await?;
    match inst.kind_code() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_conf_dim_serv::modify(dim_conf_key, modify_req, funs, ctx, &inst).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn dim_delete(dim_conf_key: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let inst = funs.init(ctx, true, stats_initializer::init_fun).await?;
    match inst.kind_code() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_conf_dim_serv::delete(dim_conf_key, funs, ctx, &inst).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn fact_add(add_req: &StatsConfFactAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let inst = funs.init(ctx, true, stats_initializer::init_fun).await?;
    match inst.kind_code() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_conf_fact_serv::add(add_req, funs, ctx, &inst).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn fact_modify(fact_conf_key: &str, modify_req: &StatsConfFactModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let inst = funs.init(ctx, true, stats_initializer::init_fun).await?;
    match inst.kind_code() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_conf_fact_serv::modify(fact_conf_key, modify_req, funs, ctx, &inst).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn fact_delete(fact_conf_key: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let inst = funs.init(ctx, true, stats_initializer::init_fun).await?;
    match inst.kind_code() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_conf_fact_serv::delete(fact_conf_key, funs, ctx, &inst).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}
pub async fn fact_col_modify(fact_conf_key: &str, fact_col_conf_key: &str, modify_req: &StatsConfFactColModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let inst = funs.init(ctx, true, stats_initializer::init_fun).await?;
    match inst.kind_code() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_conf_fact_col_serv::modify(fact_conf_key, fact_col_conf_key, modify_req, funs, ctx, &inst).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn fact_col_delete(fact_conf_key: &str, fact_col_conf_key: Option<&str>, kind: Option<StatsFactColKind>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let inst = funs.init(ctx, true, stats_initializer::init_fun).await?;
    match inst.kind_code() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_conf_fact_col_serv::delete(fact_conf_key, fact_col_conf_key, kind, funs, ctx, &inst).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn dim_online(dim_conf_key: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let inst = funs.init(ctx, true, stats_initializer::init_fun).await?;
    match inst.kind_code() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_conf_dim_serv::create_inst(dim_conf_key, funs, ctx, &inst).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn fact_online(fact_conf_key: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let inst = funs.init(ctx, true, stats_initializer::init_fun).await?;
    match inst.kind_code() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_conf_fact_serv::create_inst(fact_conf_key, funs, ctx, &inst).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn dim_paginate(
    dim_conf_key: Option<String>,
    show_name: Option<String>,
    page_number: u32,
    page_size: u32,
    desc_by_create: Option<bool>,
    desc_by_update: Option<bool>,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
) -> TardisResult<TardisPage<StatsConfDimInfoResp>> {
    let inst = funs.init(ctx, true, stats_initializer::init_fun).await?;
    match inst.kind_code() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => {
            pg::stats_pg_conf_dim_serv::paginate(dim_conf_key, show_name, page_number, page_size, desc_by_create, desc_by_update, funs, ctx, &inst).await
        }
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn fact_paginate(
    fact_conf_key: Option<String>,
    show_name: Option<String>,
    dim_rel_conf_dim_keys: Option<Vec<String>>,
    is_online: Option<bool>,
    page_number: u32,
    page_size: u32,
    desc_by_create: Option<bool>,
    desc_by_update: Option<bool>,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
) -> TardisResult<TardisPage<StatsConfFactInfoResp>> {
    let inst = funs.init(ctx, true, stats_initializer::init_fun).await?;
    match inst.kind_code() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => {
            pg::stats_pg_conf_fact_serv::paginate(
                fact_conf_key,
                show_name,
                dim_rel_conf_dim_keys,
                is_online,
                page_number,
                page_size,
                desc_by_create,
                desc_by_update,
                funs,
                ctx,
                &inst,
            )
            .await
        }
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}
pub async fn fact_col_paginate(
    fact_conf_key: String,
    fact_col_conf_key: Option<String>,
    show_name: Option<String>,
    page_number: u32,
    page_size: u32,
    desc_by_create: Option<bool>,
    desc_by_update: Option<bool>,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
) -> TardisResult<TardisPage<StatsConfFactColInfoResp>> {
    let inst = funs.init(ctx, true, stats_initializer::init_fun).await?;
    match inst.kind_code() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => {
            pg::stats_pg_conf_fact_col_serv::paginate(
                Some(fact_conf_key),
                fact_col_conf_key,
                None,
                show_name,
                page_number,
                page_size,
                desc_by_create,
                desc_by_update,
                funs,
                ctx,
                &inst,
            )
            .await
        }
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn fact_col_paginate_by_dim(
    fact_conf_key: Option<String>,
    dim_key: String,
    fact_col_conf_key: Option<String>,
    show_name: Option<String>,
    page_number: u32,
    page_size: u32,
    desc_by_create: Option<bool>,
    desc_by_update: Option<bool>,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
) -> TardisResult<TardisPage<StatsConfFactColInfoResp>> {
    let inst = funs.init(ctx, true, stats_initializer::init_fun).await?;
    match inst.kind_code() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => {
            pg::stats_pg_conf_fact_col_serv::paginate(
                fact_conf_key,
                fact_col_conf_key,
                Some(dim_key),
                show_name,
                page_number,
                page_size,
                desc_by_create,
                desc_by_update,
                funs,
                ctx,
                &inst,
            )
            .await
        }
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn fact_col_add(fact_conf_key: &str, add_req: &StatsConfFactColAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let inst = funs.init(ctx, true, stats_initializer::init_fun).await?;
    match inst.kind_code() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_conf_fact_col_serv::add(fact_conf_key, add_req, funs, ctx, &inst).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

// TODO FIXME ------------ 使用 spi_dispatch_service , 前后逻辑写到 api 中 ------------