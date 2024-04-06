use tardis::basic::result::TardisResult;
use tardis::web::web_resp::TardisPage;

use bios_basic::spi::spi_constants;
use bios_basic::spi::spi_funs::SpiBsInstExtractor;
use bios_basic::spi_dispatch_service;

use crate::dto::log_item_dto::{LogItemAddReq, LogItemFindReq, LogItemFindResp};
use crate::log_initializer;

use super::pg;

spi_dispatch_service! {
    @mgr: true,
    @init: log_initializer::init_fun,
    @dispatch: {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::log_pg_item_serv,
    },
    @method: {
        add(add_req: &mut LogItemAddReq) -> TardisResult<String>;
        find(find_req: &mut LogItemFindReq) -> TardisResult<TardisPage<LogItemFindResp>>;
    }
}
