use bios_basic::spi::spi_constants;
use bios_basic::spi::spi_funs::SpiBsInstExtractor;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::web::web_resp::TardisPage;
use tardis::TardisFunsInst;

use crate::dto::log_item_dto::{LogItemAddReq, LogItemFindReq, LogItemFindResp};
use crate::log_initializer;

use super::pg;

pub async fn add(add_req: &mut LogItemAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    match funs.init(ctx, true, log_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::log_pg_item_serv::add(add_req, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn find(find_req: &mut LogItemFindReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<TardisPage<LogItemFindResp>> {
    match funs.init(ctx, true, log_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::log_pg_item_serv::find(find_req, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}
