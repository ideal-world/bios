use bios_basic::spi::spi_constants;
use bios_basic::spi::spi_funs::SpiBsInstExtractor;
use bios_basic::spi_dispatch_service;
use tardis::basic::result::TardisResult;
use tardis::web::web_resp::TardisPage;

use crate::dto::search_item_dto::{
    GroupSearchItemSearchReq, GroupSearchItemSearchResp, SearchExportDataReq, SearchExportDataResp, SearchImportDataReq, SearchItemAddReq, SearchItemModifyReq,
    SearchItemSearchReq, SearchItemSearchResp, SearchQueryMetricsReq, SearchQueryMetricsResp,
};
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
        group_search(search_req: &mut GroupSearchItemSearchReq) -> TardisResult<Vec<GroupSearchItemSearchResp>>;
        query_metrics(query_req: &SearchQueryMetricsReq) -> TardisResult<SearchQueryMetricsResp>;
        refresh_tsv(tag: &str) -> TardisResult<()>;
        export_data(export_req: &SearchExportDataReq) -> TardisResult<SearchExportDataResp>;
        import_data(import_req: &SearchImportDataReq) -> TardisResult<bool>;
    }
}
