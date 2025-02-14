use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::web::web_resp::{TardisPage, TardisResp};
use tardis::TardisFunsInst;

#[cfg(feature = "event")]
use crate::clients::event_client::mq_client_node;
use crate::dto::search_item_dto::{SearchEventItemDeleteReq, SearchEventItemModifyReq, SearchItemAddReq, SearchItemModifyReq, SearchItemSearchReq, SearchItemSearchResp};
#[cfg(feature = "event")]
use crate::invoke_config::InvokeConfigApi as _;
use crate::invoke_enumeration::InvokeModuleKind;

use super::base_spi_client::BaseSpiClient;
#[cfg(feature = "event")]
use super::event_client::{mq_error, mq_client_node_opt, EventAttributeExt, EventCenterClient, SPI_RPC_TOPIC};
use super::spi_kv_client::SpiKvClient;

pub struct SpiSearchClient;
#[cfg(feature = "event")]
pub mod event {
    use crate::dto::search_item_dto::{SearchEventItemDeleteReq, SearchEventItemModifyReq, SearchItemAddReq, SearchItemModifyReq};
    use asteroid_mq_sdk::model::{event::EventAttribute, *};
    use serde::{Deserialize, Serialize};
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SearchItemModifyEventPayload {
        pub tag: String,
        pub key: String,
        pub req: SearchItemModifyReq,
    }

    impl EventAttribute for SearchItemAddReq {
        const SUBJECT: Subject = Subject::const_new("search/add");
    }
    impl EventAttribute for SearchEventItemModifyReq {
        const SUBJECT: Subject = Subject::const_new("search/modify");
    }
    impl EventAttribute for SearchEventItemDeleteReq {
        const SUBJECT: Subject = Subject::const_new("search/delete");
    }
}

impl SpiSearchClient {
    pub async fn add_item_and_name(add_req: &SearchItemAddReq, name: Option<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        #[cfg(feature = "event")]
        if funs.invoke_conf_in_event(InvokeModuleKind::Search) && mq_client_node_opt().is_some() {
            return EventCenterClient { topic_code: SPI_RPC_TOPIC }.add_item_and_name(add_req, name, funs, ctx).await;
        }
        let search_url: String = BaseSpiClient::module_url(InvokeModuleKind::Search, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;
        funs.web_client().put_obj_to_str(&format!("{search_url}/ci/item"), add_req, headers.clone()).await?;
        let name = name.unwrap_or_else(|| add_req.title.clone());
        SpiKvClient::add_or_modify_key_name(&format!("{}:{}", add_req.tag, add_req.key), &name, add_req.kv_disable, funs, ctx).await?;
        Ok(())
    }

    pub async fn modify_item_and_name(tag: &str, key: &str, modify_req: &SearchItemModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        #[cfg(feature = "event")]
        if funs.invoke_conf_in_event(InvokeModuleKind::Search) && mq_client_node_opt().is_some() {
            return EventCenterClient { topic_code: SPI_RPC_TOPIC }.modify_item_and_name(tag, key, modify_req, funs, ctx).await;
        }
        let search_url: String = BaseSpiClient::module_url(InvokeModuleKind::Search, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;
        funs.web_client().put_obj_to_str(&format!("{search_url}/ci/item/{tag}/{key}"), modify_req, headers.clone()).await?;
        if modify_req.title.is_some() || modify_req.name.is_some() {
            let name = modify_req.name.clone().unwrap_or(modify_req.title.clone().unwrap_or("".to_string()));
            SpiKvClient::add_or_modify_key_name(&format!("{tag}:{key}"), &name, modify_req.kv_disable, funs, ctx).await?;
        }
        Ok(())
    }

    pub async fn delete_item_and_name(tag: &str, key: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        #[cfg(feature = "event")]
        if funs.invoke_conf_in_event(InvokeModuleKind::Search) && mq_client_node_opt().is_some() {
            return EventCenterClient { topic_code: SPI_RPC_TOPIC }.delete_item_and_name(tag, key, funs, ctx).await;
        }
        let search_url = BaseSpiClient::module_url(InvokeModuleKind::Search, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;
        funs.web_client().delete_to_void(&format!("{search_url}/ci/item/{tag}/{key}"), headers.clone()).await?;
        SpiKvClient::delete_item(&format!("__k_n__:{tag}:{key}"), funs, ctx).await?;
        Ok(())
    }

    pub async fn search(search_req: &SearchItemSearchReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<TardisPage<SearchItemSearchResp>>> {
        let search_url = BaseSpiClient::module_url(InvokeModuleKind::Search, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;
        let resp =
            funs.web_client().put::<SearchItemSearchReq, TardisResp<TardisPage<SearchItemSearchResp>>>(format!("{search_url}/ci/item/search"), search_req, headers.clone()).await?;
        BaseSpiClient::package_resp(resp)
    }
}
#[cfg(feature = "event")]
impl EventCenterClient {
    pub async fn add_item_and_name(&self, add_req: &SearchItemAddReq, name: Option<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let node = mq_client_node();
        use super::event_client::EventAttributeExt;
        node.send_event(self.topic_code.clone(), add_req.clone().inject_context(funs, ctx).json()).await.map_err(mq_error)?;
        let name = name.unwrap_or_else(|| add_req.title.clone());
        SpiKvClient::add_or_modify_key_name(&format!("{}:{}", add_req.tag, add_req.key), &name, add_req.kv_disable, funs, ctx).await?;
        Ok(())
    }

    pub async fn modify_item_and_name(&self, tag: &str, key: &str, modify_req: &SearchItemModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let node = mq_client_node();
        node.send_event(
            self.topic_code.clone(),
            SearchEventItemModifyReq {
                tag: tag.to_string(),
                key: key.to_string(),
                item: modify_req.clone(),
            }
            .inject_context(funs, ctx)
            .json(),
        )
        .await
        .map_err(mq_error)?;
        if modify_req.title.is_some() || modify_req.name.is_some() {
            let name = modify_req.name.clone().unwrap_or(modify_req.title.clone().unwrap_or("".to_string()));
            SpiKvClient::add_or_modify_key_name(&format!("{}:{}", tag, key), &name, modify_req.kv_disable, funs, ctx).await?;
        }
        Ok(())
    }

    pub async fn delete_item_and_name(&self, tag: &str, key: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let topic = mq_client_node();
        topic
            .send_event(
                self.topic_code.clone(),
                SearchEventItemDeleteReq {
                    tag: tag.to_string(),
                    key: key.to_string(),
                }
                .inject_context(funs, ctx)
                .json(),
            )
            .await
            .map_err(mq_error)?;
        SpiKvClient::delete_item(&format!("__k_n__:{tag}:{key}"), funs, ctx).await?;
        Ok(())
    }
}
