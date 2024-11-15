use crate::dto::stats_conf_dto::{StatsSyncDbConfigAddReq, StatsSyncDbConfigInfoResp, StatsSyncDbConfigModifyReq};
use crate::stats_initializer;
use bios_basic::spi::spi_constants;
use bios_basic::spi::spi_funs::SpiBsInstExtractor;
use bios_basic::spi_dispatch_service;
use tardis::basic::result::TardisResult;

use super::pg;

spi_dispatch_service! {
  @mgr: true,
  @init: stats_initializer::init_fun,
  @dispatch: {
      #[cfg(feature = "spi-pg")]
      spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_sync_serv,
  },
  @method: {
    fact_record_sync(fact_key: &str) -> TardisResult<()>;
    fact_col_record_sync(fact_key: &str, col_key: &str) -> TardisResult<()>;
    db_config_add(add_req: StatsSyncDbConfigAddReq) -> TardisResult<String>;
    db_config_modify(modify_req: StatsSyncDbConfigModifyReq) -> TardisResult<()>;
    db_config_list() -> TardisResult<Vec<StatsSyncDbConfigInfoResp>>;
  }
}
