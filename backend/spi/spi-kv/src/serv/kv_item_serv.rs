use tardis::basic::result::TardisResult;
use tardis::web::web_resp::TardisPage;

use bios_basic::spi::spi_constants;
use bios_basic::spi::spi_funs::SpiBsInstExtractor;
use bios_basic::spi_dispatch_service;

use crate::dto::kv_item_dto::{
    KvItemAddOrModifyReq, KvItemDetailResp, KvItemMatchReq, KvItemSummaryResp, KvNameAddOrModifyReq, KvNameFindResp, KvTagAddOrModifyReq, KvTagFindResp,
};
use crate::kv_initializer;

use super::pg;

spi_dispatch_service! {
    @mgr: true,
    @init: kv_initializer::init_fun,
    @dispatch: {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::kv_pg_item_serv,
    },
    @method: {
        add_or_modify_item(add_or_modify_req: &mut KvItemAddOrModifyReq) -> TardisResult<()>;
        add_or_modify_key_name(add_or_modify_req: &mut KvNameAddOrModifyReq) -> TardisResult<()>;
        add_or_modify_tag(add_or_modify_req: &mut KvTagAddOrModifyReq) -> TardisResult<()>;
        get_item(key: String, extract: Option<String>) -> TardisResult<Option<KvItemDetailResp>>;
        find_items(keys: Vec<String>, extract: Option<String>) -> TardisResult<Vec<KvItemSummaryResp>>;
        find_key_names(keys: Vec<String>) -> TardisResult<Vec<KvNameFindResp>>;
        find_tags(keys: Vec<String>) -> TardisResult<Vec<KvTagFindResp>>;
        page_tags(
            key_prefix: String,
            page_number: u32,
            page_size: u16,
            desc_sort_by_create: Option<bool>,
            desc_sort_by_update: Option<bool>
        ) -> TardisResult<TardisPage<KvTagFindResp>>;
        match_items(match_req: KvItemMatchReq) -> TardisResult<TardisPage<KvItemSummaryResp>>;
        delete_item(key: String) -> TardisResult<()>;
    }
}