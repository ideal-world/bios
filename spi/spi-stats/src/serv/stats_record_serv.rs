use crate::dto::stats_record_dto::{StatsDimRecordAddReq, StatsFactRecordLoadReq, StatsFactRecordsLoadReq};
use crate::stats_initializer;
use bios_basic::spi::spi_constants;
use bios_basic::spi::spi_funs::SpiBsInstExtractor;
use bios_basic::spi_dispatch_service;
use tardis::basic::result::TardisResult;
use tardis::chrono::{DateTime, Utc};
use tardis::serde_json::{self, Value};
use tardis::web::web_resp::TardisPage;

use super::pg;
spi_dispatch_service! {
    @mgr: true,
    @init: stats_initializer::init_fun,
    @dispatch: {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_record_serv,
    },
    @method: {
        get_fact_record_latest(fact_conf_key: &str,fact_record_key: &str) -> TardisResult<serde_json::Value>;
        fact_record_load(fact_conf_key: &str,fact_record_key: &str, add_req: StatsFactRecordLoadReq) -> TardisResult<()>;
        fact_record_delete(fact_conf_key: &str, fact_record_key: &str) -> TardisResult<()>;
        fact_records_load(fact_conf_key: &str, add_req_set: Vec<StatsFactRecordsLoadReq>) -> TardisResult<()>;
        fact_records_delete(fact_conf_key: &str, fact_record_delete_keys: &[String]) -> TardisResult<()>;
        fact_records_delete_by_dim_key(fact_conf_key: &str, dim_conf_key: &str,dim_record_key: Option<serde_json::Value>) -> TardisResult<()>;
        fact_records_clean(fact_conf_key: &str, before_ct: Option<DateTime<Utc>>) -> TardisResult<()>;
        dim_record_add(dim_conf_key: String, add_req: StatsDimRecordAddReq) -> TardisResult<()>;
        dim_record_paginate(
            dim_conf_key: String,
            dim_record_key: Option<Value>,
            show_name: Option<String>,
            page_number: u32,
            page_size: u32,
            desc_by_create: Option<bool>,
            desc_by_update: Option<bool>
        ) -> TardisResult<TardisPage<Value>>;
        dim_record_delete(dim_conf_key: String, dim_record_key: Value) -> TardisResult<()>;
    }
}
