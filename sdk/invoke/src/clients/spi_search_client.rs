use tardis::async_trait::async_trait;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::web::ws_client::TardisWSClient;
use tardis::web::ws_processor::TardisWebsocketReq;
use tardis::{async_trait, TardisFuns, TardisFunsInst};

use crate::dto::search_item_dto::{SearchEventItemDeleteReq, SearchEventItemModifyReq, SearchItemAddReq, SearchItemModifyReq};
use crate::invoke_config::InvokeConfigApi;
use crate::invoke_enumeration::InvokeModuleKind;

use super::base_spi_client::BaseSpiClient;
use super::spi_kv_client::SpiKvClient;

pub struct SpiSearchClient;

impl SpiSearchClient {
    pub async fn add_item(add_req: &SearchItemAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let search_url: String = BaseSpiClient::module_url(InvokeModuleKind::Search, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;
        funs.web_client().put_obj_to_str(&format!("{search_url}/ci/item"), add_req, headers.clone()).await?;
        let name = if let Some(name) = add_req.name.clone() { name } else { add_req.title.clone() };
        SpiKvClient::add_or_modify_key_name(&format!("{}:{}", add_req.tag, add_req.key), &name, funs, ctx).await?;
        Ok(())
    }

    pub async fn modify_item(tag: &str, key: &str, modify_req: &SearchItemModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let search_url: String = BaseSpiClient::module_url(InvokeModuleKind::Search, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;
        funs.web_client().put_obj_to_str(&format!("{search_url}/ci/item/{tag}/{key}"), modify_req, headers.clone()).await?;
        if modify_req.title.is_some() || modify_req.name.is_some() {
            let name = modify_req.name.clone().unwrap_or(modify_req.title.clone().unwrap_or("".to_string()));
            SpiKvClient::add_or_modify_key_name(&format!("{tag}:{key}"), &name, funs, ctx).await?;
        }
        Ok(())
    }

    pub async fn delete_item(tag: &str, key: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let search_url = BaseSpiClient::module_url(InvokeModuleKind::Search, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;
        funs.web_client().delete_to_void(&format!("{search_url}/ci/item/{tag}/{key}"), headers.clone()).await?;
        Ok(())
    }
}

#[async_trait]
pub trait SpiSearchEventExt {
    async fn publish_add_item(&self, req: &SearchItemAddReq, from: String, spi_app_id: String, ctx: &TardisContext) -> TardisResult<()>;
    async fn publish_modify_item(&self, tag: String, key: String, req: &SearchItemModifyReq, from: String, spi_app_id: String, ctx: &TardisContext) -> TardisResult<()>;
    async fn publish_delete_item(&self, tag: String, key: String, from: String, spi_app_id: String, ctx: &TardisContext) -> TardisResult<()>;
}

#[async_trait]
impl SpiSearchEventExt for TardisWSClient {
    async fn publish_add_item(&self, req: &SearchItemAddReq, from: String, spi_app_id: String, ctx: &TardisContext) -> TardisResult<()> {
        let spi_ctx = TardisContext { owner: spi_app_id, ..ctx.clone() };
        let req = TardisWebsocketReq {
            msg: TardisFuns::json.obj_to_json(&(req, spi_ctx)).expect("invalid json"),
            to_avatars: Some(vec!["spi-search/service".into()]),
            from_avatar: from,
            event: Some("spi-search/add".into()),
            ..Default::default()
        };
        self.send_obj(&req).await?;
        return Ok(());
    }

    async fn publish_modify_item(&self, tag: String, key: String, req: &SearchItemModifyReq, from: String, spi_app_id: String, ctx: &TardisContext) -> TardisResult<()> {
        let spi_ctx = TardisContext { owner: spi_app_id, ..ctx.clone() };
        let event_req = SearchEventItemModifyReq {
            tag,
            key,
            item: TardisFuns::json.obj_to_json(req).expect("invalid json"),
        };
        let req = TardisWebsocketReq {
            msg: TardisFuns::json.obj_to_json(&(event_req, spi_ctx)).expect("invalid json"),
            to_avatars: Some(vec!["spi-search/service".into()]),
            from_avatar: from,
            event: Some("spi-search/modify".into()),
            ..Default::default()
        };
        self.send_obj(&req).await?;
        return Ok(());
    }

    async fn publish_delete_item(&self, tag: String, key: String, from: String, spi_app_id: String, ctx: &TardisContext) -> TardisResult<()> {
        let spi_ctx = TardisContext { owner: spi_app_id, ..ctx.clone() };
        let event_req = SearchEventItemDeleteReq { tag, key };
        let req = TardisWebsocketReq {
            msg: TardisFuns::json.obj_to_json(&(event_req, spi_ctx)).expect("invalid json"),
            to_avatars: Some(vec!["spi-search/service".into()]),
            from_avatar: from,
            event: Some("spi-search/delete".into()),
            ..Default::default()
        };
        self.send_obj(&req).await?;
        return Ok(());
    }
}
