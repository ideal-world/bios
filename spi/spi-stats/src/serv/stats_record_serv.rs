use crate::dto::stats_record_dto::{StatsDimRecordAddReq, StatsFactRecordLoadReq, StatsFactRecordsLoadReq};
use crate::stats_initializer;
use bios_basic::spi::spi_constants;
use bios_basic::spi::spi_funs::SpiBsInstExtractor;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::chrono::{DateTime, Utc};
use tardis::serde_json::Value;
use tardis::web::web_resp::TardisPage;
use tardis::TardisFunsInst;

use super::pg;

pub(crate) async fn fact_record_load(
    fact_conf_key: String,
    fact_record_key: String,
    add_req: StatsFactRecordLoadReq,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
) -> TardisResult<()> {
    match funs.init(ctx, true, stats_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_record_serv::fact_record_load(&fact_conf_key, &fact_record_key, add_req, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub(crate) async fn fact_record_delete(fact_conf_key: String, fact_record_key: String, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    match funs.init(ctx, true, stats_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_record_serv::fact_record_delete(&fact_conf_key, &fact_record_key, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub(crate) async fn fact_records_load(fact_conf_key: String, add_req_set: Vec<StatsFactRecordsLoadReq>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    match funs.init(ctx, true, stats_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_record_serv::fact_records_load(&fact_conf_key, add_req_set, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub(crate) async fn fact_records_delete(fact_conf_key: String, fact_record_delete_keys: Vec<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    match funs.init(ctx, true, stats_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_record_serv::fact_records_delete(&fact_conf_key, &fact_record_delete_keys, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub(crate) async fn fact_records_clean(fact_conf_key: String, before_ct: Option<DateTime<Utc>>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    match funs.init(ctx, true, stats_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_record_serv::fact_records_clean(&fact_conf_key, before_ct, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub(crate) async fn dim_record_add(dim_conf_key: String, dim_record_key: String, add_req: StatsDimRecordAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    match funs.init(ctx, true, stats_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_record_serv::dim_record_add(dim_conf_key, dim_record_key, add_req, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub(crate) async fn dim_record_paginate(
    dim_conf_key: String,
    dim_record_key: Option<String>,
    show_name: Option<String>,
    page_number: u32,
    page_size: u32,
    desc_by_create: Option<bool>,
    desc_by_update: Option<bool>,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
) -> TardisResult<TardisPage<Value>> {
    match funs.init(ctx, true, stats_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => {
            pg::stats_pg_record_serv::dim_record_paginate(dim_conf_key, dim_record_key, show_name, page_number, page_size, desc_by_create, desc_by_update, funs, ctx).await
        }
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub(crate) async fn dim_record_delete(dim_conf_key: String, dim_record_key: String, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    match funs.init(ctx, true, stats_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_record_serv::dim_record_delete(dim_conf_key, dim_record_key, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}
