use bios_basic::TardisFunInstExtractor;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use crate::dto::log_item_dto::{LogItemAddReq, LogItemFindReq, LogItemFindResp};
use crate::serv::log_item_serv;

pub struct LogCiItemApi;

/// Interface Console Log API
#[poem_openapi::OpenApi(prefix_path = "/ci/item", tag = "bios_basic::ApiTag::Interface")]
impl LogCiItemApi {
    /// Add Item
    #[oai(path = "/", method = "post")]
    async fn add(&self, mut add_req: Json<LogItemAddReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let funs = request.tardis_fun_inst();
        log_item_serv::add(&mut add_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Find Items
    #[oai(path = "/find", method = "put")]
    async fn find(&self, mut find_req: Json<LogItemFindReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<TardisPage<LogItemFindResp>> {
        let funs = request.tardis_fun_inst();
        let resp = log_item_serv::find(&mut find_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(resp)
    }
}
