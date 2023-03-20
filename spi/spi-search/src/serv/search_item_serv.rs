use bios_basic::spi::spi_constants;
use bios_basic::spi::spi_funs::SpiBsInstExtractor;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::web::web_resp::TardisPage;
use tardis::TardisFunsInst;

use crate::dto::search_item_dto::{SearchItemAddReq, SearchItemModifyReq, SearchItemSearchReq, SearchItemSearchResp};
use crate::search_initializer;

use super::pg;

pub async fn add(add_req: &mut SearchItemAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    match funs.init(ctx, true, search_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::search_pg_item_serv::add(add_req, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn modify(tag: &str, key: &str, modify_req: &mut SearchItemModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    match funs.init(ctx, true, search_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::search_pg_item_serv::modify(tag, key, modify_req, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn delete(tag: &str, key: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    match funs.init(ctx, true, search_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::search_pg_item_serv::delete(tag, key, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn search(search_req: &mut SearchItemSearchReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<TardisPage<SearchItemSearchResp>> {
    match funs.init(ctx, true, search_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::search_pg_item_serv::search(search_req, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}
