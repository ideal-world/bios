use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Query;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use crate::dto::kv_item_dto::{
    KvItemAddOrModifyReq, KvItemDetailResp, KvItemKeyReq, KvItemMatchReq, KvItemSummaryResp, KvNameAddOrModifyReq, KvNameFindResp, KvTagAddOrModifyReq, KvTagFindResp,
};
use crate::serv::kv_item_serv;

#[derive(Clone)]
pub struct KvCiItemApi;

/// Interface Console KV API
///
/// 接口控制台KV API
#[poem_openapi::OpenApi(prefix_path = "/ci", tag = "bios_basic::ApiTag::Interface")]
impl KvCiItemApi {
    /// Add Or Modify Item
    ///
    /// 添加或修改Item
    #[oai(path = "/item", method = "put")]
    async fn add_or_modify_item(&self, mut add_or_modify_req: Json<KvItemAddOrModifyReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        kv_item_serv::add_or_modify_item(&mut add_or_modify_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Get Item
    ///
    /// 获取Item
    #[oai(path = "/item", method = "get")]
    async fn get_item(&self, key: Query<String>, extract: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<Option<KvItemDetailResp>> {
        let funs = crate::get_tardis_inst();
        let resp = kv_item_serv::get_item(key.0, extract.0, &funs, &ctx.0).await?;
        TardisResp::ok(resp)
    }

    /// Find Items By keys
    ///
    /// 通过keys查找Items
    #[oai(path = "/items", method = "get")]
    async fn find_items(&self, keys: Query<Vec<String>>, extract: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<Vec<KvItemSummaryResp>> {
        let funs = crate::get_tardis_inst();
        let resp = kv_item_serv::find_items(keys.0, extract.0, &funs, &ctx.0).await?;
        TardisResp::ok(resp)
    }

    /// Match Items By key prefix
    ///
    /// 通过key前缀匹配Items
    #[oai(path = "/item/match", method = "get")]
    async fn match_items_by_key_prefix(
        &self,
        key_prefix: Query<String>,
        extract: Query<Option<String>>,
        key_like: Query<Option<bool>>,
        page_number: Query<u32>,
        page_size: Query<u16>,
        disable: Query<Option<bool>>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<KvItemSummaryResp>> {
        let funs = crate::get_tardis_inst();
        let resp = kv_item_serv::match_items(
            KvItemMatchReq {
                key_prefix: key_prefix.0,
                extract: extract.0,
                page_number: page_number.0,
                page_size: page_size.0,
                key_like: key_like.0,
                desc_sort_by_create: desc_by_create.0,
                desc_sort_by_update: desc_by_update.0,
                disable: disable.0,
                ..Default::default()
            },
            &funs,
            &ctx.0,
        )
        .await?;
        TardisResp::ok(resp)
    }

    /// Match Items
    ///
    /// 匹配Items
    #[oai(path = "/item/match", method = "put")]
    async fn match_items(&self, match_req: Json<KvItemMatchReq>, ctx: TardisContextExtractor) -> TardisApiResult<TardisPage<KvItemSummaryResp>> {
        let funs = crate::get_tardis_inst();
        let resp = kv_item_serv::match_items(match_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(resp)
    }

    /// Delete Item
    ///
    /// 删除Item
    #[oai(path = "/item", method = "delete")]
    async fn delete_item(&self, key: Query<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        kv_item_serv::delete_item(key.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Delete Item Body
    /// ps: key may contain special characters such as spaces, so you need to use body to receive
    ///
    /// 删除Item
    /// ps: key可能会存在空格等特殊字符,所以需要使用 body 进行接收
    #[oai(path = "/item/delete", method = "put")]
    async fn delete_item_body(&self, delete_req: Json<KvItemKeyReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        kv_item_serv::delete_item(delete_req.0.key.to_string(), &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Disable Item Body
    /// ps: key may contain special characters such as spaces, so you need to use body to receive
    ///
    /// 禁用Item body
    /// ps: key可能会存在空格等特殊字符,所以需要使用 body 进行接收
    #[oai(path = "/item/disable", method = "put")]
    async fn disable_item_body(&self, delete_req: Json<KvItemKeyReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        kv_item_serv::disable_item(delete_req.0.key.to_string(), &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Enable Item Body
    /// ps: key may contain special characters such as spaces, so you need to use body to receive
    ///
    /// 启用Item body
    /// ps: key可能会存在空格等特殊字符,所以需要使用 body 进行接收
    #[oai(path = "/item/disable", method = "put")]
    async fn enable_item_body(&self, delete_req: Json<KvItemKeyReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        kv_item_serv::enabled_item(delete_req.0.key.to_string(), &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Add Or Modify Key-Name
    ///
    /// 添加或修改Key-Name
    #[oai(path = "/scene/key-name", method = "put")]
    async fn add_or_modify_key_name(&self, mut add_or_modify_req: Json<KvNameAddOrModifyReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        kv_item_serv::add_or_modify_key_name(&mut add_or_modify_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Find Names By keys
    ///
    /// 通过keys查找Names
    #[oai(path = "/scene/key-names", method = "get")]
    async fn find_key_names(&self, keys: Query<Vec<String>>, ctx: TardisContextExtractor) -> TardisApiResult<Vec<KvNameFindResp>> {
        let funs = crate::get_tardis_inst();
        let resp = kv_item_serv::find_key_names(keys.0, &funs, &ctx.0).await?;
        TardisResp::ok(resp)
    }

    /// Add Or Modify Tag
    ///
    /// 添加或修改Tag
    #[oai(path = "/scene/tag", method = "put")]
    async fn add_or_modify_tag(&self, mut add_or_modify_req: Json<KvTagAddOrModifyReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        kv_item_serv::add_or_modify_tag(&mut add_or_modify_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Page Tags By key prefix
    ///
    /// 通过key前缀分页Tags
    #[oai(path = "/scene/tags", method = "get")]
    async fn page_tags(
        &self,
        key_prefix: Query<String>,
        key_like: Query<Option<bool>>,
        page_number: Query<u32>,
        page_size: Query<u16>,
        disable: Query<Option<bool>>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<KvTagFindResp>> {
        let funs = crate::get_tardis_inst();
        let resp = kv_item_serv::page_tags(
            key_prefix.0,
            key_like.0,
            page_number.0,
            page_size.0,
            disable.0,
            desc_by_create.0,
            desc_by_update.0,
            &funs,
            &ctx.0,
        )
        .await?;
        TardisResp::ok(resp)
    }

    /// Find Tags By
    ///
    /// 通过keys查找Tags
    #[oai(path = "/tags", method = "get")]
    async fn find_tags(&self, keys: Query<Vec<String>>, ctx: TardisContextExtractor) -> TardisApiResult<Vec<KvTagFindResp>> {
        let funs = crate::get_tardis_inst();
        let resp = kv_item_serv::find_tags(keys.0, &funs, &ctx.0).await?;
        TardisResp::ok(resp)
    }
}
