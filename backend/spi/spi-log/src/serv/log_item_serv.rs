use tardis::basic::result::TardisResult;

use bios_basic::spi::spi_constants;
use bios_basic::spi::spi_funs::SpiBsInstExtractor;
use bios_basic::spi_dispatch_service;

use crate::dto::log_item_dto::{LogConfigReq, LogItemAddReq, LogItemFindReq, LogItemFindResp};
use crate::log_initializer;
use tardis::web::web_resp::TardisPage;

use super::super::log_constants;
use super::pg;
use super::pgv2;

spi_dispatch_service! {
    @mgr: true,
    @init: log_initializer::init_fun,
    @dispatch: {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::log_pg_item_serv,
        log_constants::SPI_PG_V2_KIND_CODE => pgv2::log_pg_item_serv,
    },
    @method: {
        add(add_req: &mut LogItemAddReq) -> TardisResult<String>;
        find(find_req: &mut LogItemFindReq) -> TardisResult<TardisPage<LogItemFindResp>>;
        add_config(config: &mut LogConfigReq) -> TardisResult<()>;
        delete_config(config: &mut LogConfigReq) -> TardisResult<()>;
    }
}
