use event::{SearchItemAddEvent, SearchItemDeleteEvent, SearchItemModifyEvent};
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::TardisFunsInst;

use crate::dto::search_item_dto::{SearchEventItemDeleteReq, SearchEventItemModifyReq, SearchItemAddReq, SearchItemModifyReq};
use crate::invoke_config::InvokeConfigApi;
use crate::invoke_enumeration::InvokeModuleKind;

use super::base_spi_client::BaseSpiClient;
use super::event_client::{BiosEventCenter, EventCenter, EventExt};
use super::spi_kv_client::event::{KvItemAddOrModifyEvent, KvItemDeleteEvent};
use super::spi_kv_client::{KvItemAddOrModifyReq, KvItemDeleteReq, SpiKvClient};

pub struct SpiSearchClient;
pub mod event {
    use serde::{Deserialize, Serialize};

    use crate::{
        clients::event_client::{ContextEvent, Event},
        dto::search_item_dto::{SearchEventItemDeleteReq, SearchEventItemModifyReq, SearchItemAddReq, SearchItemModifyReq},
    };
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SearchItemModifyEventPayload {
        pub tag: String,
        pub key: String,
        pub req: SearchItemModifyReq,
    }
    const EVENT_ADD_SEARCH: &str = "spi-search/add";
    const EVENT_MODIFY_SEARCH: &str = "spi-search/modify";
    const EVENT_DELETE_SEARCH: &str = "spi-search/delete";
    pub type SearchItemAddEvent = ContextEvent<SearchItemAddReq>;
    pub type SearchItemModifyEvent = ContextEvent<SearchEventItemModifyReq>;
    pub type SearchItemDeleteEvent = ContextEvent<SearchEventItemDeleteReq>;
    impl Event for SearchItemAddEvent {
        const CODE: &'static str = EVENT_ADD_SEARCH;
    }
    impl Event for SearchItemModifyEvent {
        const CODE: &'static str = EVENT_MODIFY_SEARCH;
    }
    impl Event for SearchItemDeleteEvent {
        const CODE: &'static str = EVENT_DELETE_SEARCH;
    }
}

impl SpiSearchClient {
    pub async fn add_item_and_name(add_req: &SearchItemAddReq, name: Option<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let search_url: String = BaseSpiClient::module_url(InvokeModuleKind::Search, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;
        funs.web_client().put_obj_to_str(&format!("{search_url}/ci/item"), add_req, headers.clone()).await?;
        let name = if let Some(name) = name.clone() { name } else { add_req.title.clone() };
        SpiKvClient::add_or_modify_key_name(&format!("{}:{}", add_req.tag, add_req.key), &name, funs, ctx).await?;
        Ok(())
    }

    pub async fn modify_item_and_name(tag: &str, key: &str, modify_req: &SearchItemModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let search_url: String = BaseSpiClient::module_url(InvokeModuleKind::Search, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;
        funs.web_client().put_obj_to_str(&format!("{search_url}/ci/item/{tag}/{key}"), modify_req, headers.clone()).await?;
        if modify_req.title.is_some() || modify_req.name.is_some() {
            let name = modify_req.name.clone().unwrap_or(modify_req.title.clone().unwrap_or("".to_string()));
            SpiKvClient::add_or_modify_key_name(&format!("{tag}:{key}"), &name, funs, ctx).await?;
        }
        Ok(())
    }

    pub async fn delete_item_and_name(tag: &str, key: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let search_url = BaseSpiClient::module_url(InvokeModuleKind::Search, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;
        funs.web_client().delete_to_void(&format!("{search_url}/ci/item/{tag}/{key}"), headers.clone()).await?;
        SpiKvClient::delete_item(&format!("__k_n__:{tag}:{key}"), funs, ctx).await?;
        Ok(())
    }
}

impl BiosEventCenter {
    pub async fn add_item_and_name(&self, source: &str, add_req: &SearchItemAddReq, name: Option<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        self.publish(
            SearchItemAddEvent {
                ctx: funs.invoke_conf_inject_context(ctx),
                event: add_req.clone(),
            }.with_source(source),
        )
        .await?;
        let name = if let Some(name) = name.clone() { name } else { add_req.title.clone() };
        self.publish(
            KvItemAddOrModifyEvent {
                ctx: funs.invoke_conf_inject_context(ctx),
                event: KvItemAddOrModifyReq {
                    key: format!("{}:{}", add_req.tag, add_req.key),
                    value: tardis::serde_json::Value::String(name),
                    ..Default::default()
                },
            }.with_source(source),
        )
        .await?;
        Ok(())
    }

    pub async fn modify_item_and_name(&self, source: &str, tag: &str, key: &str, modify_req: &SearchItemModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        self.publish(
            SearchItemModifyEvent {
                ctx: funs.invoke_conf_inject_context(ctx),
                event: SearchEventItemModifyReq {
                    tag: tag.to_string(),
                    key: key.to_string(),
                    item: modify_req.clone(),
                },
            },
        )
        .await?;
        if modify_req.title.is_some() || modify_req.name.is_some() {
            let name = modify_req.name.clone().unwrap_or(modify_req.title.clone().unwrap_or("".to_string()));
            self.publish(
                KvItemAddOrModifyEvent {
                    ctx: funs.invoke_conf_inject_context(ctx),
                    event: KvItemAddOrModifyReq {
                        key: format!("{}:{}", tag, key),
                        value: tardis::serde_json::Value::String(name),
                        ..Default::default()
                    },
                }.with_source(source),
            )
            .await?;
        }
        Ok(())
    }

    pub async fn delete_item_and_name(&self, source: &str, tag: &str, key: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        self.publish(
            SearchItemDeleteEvent {
                ctx: funs.invoke_conf_inject_context(ctx),
                event: SearchEventItemDeleteReq {
                    tag: tag.to_string(),
                    key: key.to_string(),
                },
            }.with_source(source),
        )
        .await?;
        self.publish(
            KvItemDeleteEvent {
                ctx: funs.invoke_conf_inject_context(ctx),
                event: KvItemDeleteReq {
                    key: format!("__k_n__:{}:{}", tag, key),
                },
            }.with_source(source),
        )
        .await?;
        Ok(())
    }
}
