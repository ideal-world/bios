use tardis::web::context_extractor::TardisContextExtractor;

use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Query;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use crate::dto::kv_item_dto::{
    KvItemAddOrModifyReq, KvItemDetailResp, KvItemMatchReq, KvItemSummaryResp, KvNameAddOrModifyReq, KvNameFindResp, KvTagAddOrModifyReq, KvTagFindResp,
};
use crate::serv::kv_item_serv;

#[derive(Clone)]
pub struct KvCiItemApi;

/// Interface Console KV API
#[poem_openapi::OpenApi(prefix_path = "/ci", tag = "bios_basic::ApiTag::Interface")]
impl KvCiItemApi {
    /// Add Or Modify Item
    #[oai(path = "/item", method = "put")]
    async fn add_or_modify_item(&self, mut add_or_modify_req: Json<KvItemAddOrModifyReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        kv_item_serv::add_or_modify_item(&mut add_or_modify_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Get Item
    #[oai(path = "/item", method = "get")]
    async fn get_item(&self, key: Query<String>, extract: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<Option<KvItemDetailResp>> {
        let funs = crate::get_tardis_inst();
        let resp = kv_item_serv::get_item(key.0, extract.0, &funs, &ctx.0).await?;
        dbg!(&resp);
        TardisResp::ok(resp)
    }

    /// Find Items By keys
    #[oai(path = "/items", method = "get")]
    async fn find_items(&self, keys: Query<Vec<String>>, extract: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<Vec<KvItemSummaryResp>> {
        let funs = crate::get_tardis_inst();
        let resp = kv_item_serv::find_items(keys.0, extract.0, &funs, &ctx.0).await?;
        TardisResp::ok(resp)
    }

    /// Match Items By key prefix
    #[oai(path = "/item/match", method = "get")]
    async fn match_items_by_key_prefix(
        &self,
        key_prefix: Query<String>,
        extract: Query<Option<String>>,
        page_number: Query<u32>,
        page_size: Query<u16>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<KvItemSummaryResp>> {
        let funs = crate::get_tardis_inst();
        let resp = kv_item_serv::match_items(
            KvItemMatchReq {
                key_prefix: key_prefix.0,
                extract: extract.0,
                page_number: page_number.0,
                page_size: page_size.0,
                ..Default::default()
            },
            &funs,
            &ctx.0,
        )
        .await?;
        TardisResp::ok(resp)
    }

    /// Match Items
    #[oai(path = "/item/match", method = "put")]
    async fn match_items(&self, match_req: Json<KvItemMatchReq>, ctx: TardisContextExtractor) -> TardisApiResult<TardisPage<KvItemSummaryResp>> {
        let funs = crate::get_tardis_inst();
        let resp = kv_item_serv::match_items(match_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(resp)
    }

    /// Delete Item
    #[oai(path = "/item", method = "delete")]
    async fn delete_item(&self, key: Query<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        kv_item_serv::delete_item(key.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Add Or Modify Key-Name
    #[oai(path = "/scene/key-name", method = "put")]
    async fn add_or_modify_key_name(&self, mut add_or_modify_req: Json<KvNameAddOrModifyReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        kv_item_serv::add_or_modify_key_name(&mut add_or_modify_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Find Names By keys
    #[oai(path = "/scene/key-names", method = "get")]
    async fn find_key_names(&self, keys: Query<Vec<String>>, ctx: TardisContextExtractor) -> TardisApiResult<Vec<KvNameFindResp>> {
        let funs = crate::get_tardis_inst();
        let resp = kv_item_serv::find_key_names(keys.0, &funs, &ctx.0).await?;
        TardisResp::ok(resp)
    }

    /// Add Or Modify Tag
    #[oai(path = "/scene/tag", method = "put")]
    async fn add_or_modify_tag(&self, mut add_or_modify_req: Json<KvTagAddOrModifyReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        kv_item_serv::add_or_modify_tag(&mut add_or_modify_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Find Tags By key prefix
    #[oai(path = "/scene/tags", method = "get")]
    async fn find_tags(
        &self,
        key_prefix: Query<String>,
        page_number: Query<u32>,
        page_size: Query<u16>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<KvTagFindResp>> {
        let funs = crate::get_tardis_inst();
        let resp = kv_item_serv::find_tags(key_prefix.0, page_number.0, page_size.0, &funs, &ctx.0).await?;
        TardisResp::ok(resp)
    }
}
