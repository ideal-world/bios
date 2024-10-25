use crate::dto::stats_conf_dto::{StatsSyncDbConfigAddReq, StatsSyncDbConfigModifyReq};
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
      db_config_add(add_req: StatsSyncDbConfigAddReq) -> TardisResult<()>;
      db_config_modify(modify_req: StatsSyncDbConfigModifyReq) -> TardisResult<()>;
  }
}