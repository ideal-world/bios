use bios_basic::spi::spi_funs::SpiTardisFunInstExtractor;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Query;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use crate::dto::search_item_dto::{SearchItemAddOrModifyReq, SearchItemSearchReq, SearchItemSearchResp};
use crate::serv::search_item_serv::SearchItemServ;

pub struct SearchCiItemApi;

/// Interface Console Search API
#[poem_openapi::OpenApi(prefix_path = "/ci/item", tag = "bios_basic::ApiTag::Interface")]
impl SearchCiItemApi {
    /// Add Or Modify Item
    #[oai(path = "/", method = "put")]
    async fn add_or_modify(&self, mut add_or_modify_req: Json<SearchItemAddOrModifyReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let mut funs = request.tardis_fun_inst();
        SearchItemServ::add_or_modify(&mut add_or_modify_req.0, &mut funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Delete Item
    #[oai(path = "/", method = "delete")]
    async fn delete(&self, tag: Query<String>, key: Query<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let mut funs = request.tardis_fun_inst();
        SearchItemServ::delete(&tag.0, &key.0, &mut funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Search Items
    #[oai(path = "/search", method = "put")]
    async fn search(&self, mut search_req: Json<SearchItemSearchReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<TardisPage<SearchItemSearchResp>> {
        let mut funs = request.tardis_fun_inst();
        let resp = SearchItemServ::search(&mut search_req.0, &mut funs, &ctx.0).await?;
        TardisResp::ok(resp)
    }
}
