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
  }
}
