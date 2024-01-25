use bios_basic::spi::spi_constants;
use bios_basic::spi::spi_funs::SpiBsInstExtractor;
use bios_basic::spi_dispatch_service;

use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::web::web_resp::TardisPage;
use tardis::TardisFunsInst;

use crate::dto::search_item_dto::{SearchItemAddReq, SearchItemModifyReq, SearchItemSearchReq, SearchItemSearchResp, SearchQueryMetricsReq, SearchQueryMetricsResp};
use crate::search_initializer;

#[cfg(feature = "spi-es")]
use super::es;
#[cfg(feature = "spi-pg")]
use super::pg;

spi_dispatch_service! {
    @mgr: true,
    @init: search_initializer::init_fun,
    @dispatch: {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::search_pg_item_serv,
        #[cfg(feature = "spi-es")]
        spi_constants::SPI_ES_KIND_CODE => es::search_es_item_serv,
    },
    @method: {
        add(add_req: &mut SearchItemAddReq) -> TardisResult<()>;
        modify(tag: &str, key: &str, modify_req: &mut SearchItemModifyReq) -> TardisResult<()>;
        delete(tag: &str, key: &str) -> TardisResult<()>;
        delete_by_ownership(tag: &str, own_paths: &str) -> TardisResult<()>;
        search(search_req: &mut SearchItemSearchReq) -> TardisResult<TardisPage<SearchItemSearchResp>>;
        query_metrics(query_req: &SearchQueryMetricsReq) -> TardisResult<SearchQueryMetricsResp>;
    }
}

pub async fn refresh_data(tag: String, funs: &TardisFunsInst) -> TardisResult<()> {
    let global_ctx = TardisContext::default();
    let inst = funs.init(&global_ctx, true, search_initializer::init_fun).await?;
    match inst.kind_code() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::search_pg_item_serv::refresh_data(tag, &global_ctx, &inst).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}