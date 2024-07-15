use serde::{Deserialize, Serialize};

use tardis::{
    async_trait::async_trait,
    basic::{dto::TardisContext, field::TrimString, result::TardisResult},
    chrono::{DateTime, Utc},
    log::info,
    serde_json::Value,
    web::{
        poem_openapi,
        web_resp::{TardisPage, TardisResp},
        ws_client::TardisWSClient,
        ws_processor::TardisWebsocketReq,
    },
    TardisFuns, TardisFunsInst,
};

use crate::{clients::base_spi_client::BaseSpiClient, invoke_constants::DYNAMIC_LOG, invoke_enumeration::InvokeModuleKind};

pub mod event {
    use crate::clients::event_client::{ContextEvent, Event};

    const EVENT_ADD_LOG: &str = "spi-log/add";
    pub type AddLogEvent = ContextEvent<super::LogItemAddReq>;

    impl Event for AddLogEvent {
        const CODE: &'static str = EVENT_ADD_LOG;
    }
}
#[derive(Debug, Default, Clone)]
pub struct SpiLogClient;

#[derive(poem_openapi::Object, Serialize, Deserialize, Default, Debug)]
pub struct LogItemFindReq {
    pub tag: String,
    pub kinds: Option<Vec<TrimString>>,
    pub keys: Option<Vec<TrimString>>,
    pub ops: Option<Vec<String>>,
    pub owners: Option<Vec<String>>,
    pub own_paths: Option<String>,
    pub rel_keys: Option<Vec<TrimString>>,
    pub ts_start: Option<DateTime<Utc>>,
    pub ts_end: Option<DateTime<Utc>>,
    pub page_number: u32,
    pub page_size: u16,
}
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct LogItemFindResp {
    #[oai(validator(min_length = "2"))]
    pub content: String,
    pub kind: String,
    pub ext: Value,
    pub owner: String,
    pub own_paths: String,
    pub key: String,
    pub id: String,
    pub op: String,
    pub rel_key: String,
    pub ts: DateTime<Utc>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Default, Debug)]
pub struct LogDynamicContentReq {
    pub details: Option<String>,
    pub sub_kind: Option<String>,
    pub content: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LogItemAddReq {
    pub tag: String,
    pub content: String,
    pub kind: Option<String>,
    pub ext: Option<Value>,
    pub key: Option<String>,
    pub op: Option<String>,
    pub rel_key: Option<String>,
    pub id: Option<String>,
    pub ts: Option<DateTime<Utc>>,
    pub owner: Option<String>,
    pub own_paths: Option<String>,
}

impl SpiLogClient {
    pub async fn add_dynamic_log(
        content: &LogDynamicContentReq,
        ext: Option<Value>,
        kind: Option<String>,
        key: Option<String>,
        op: Option<String>,
        rel_key: Option<String>,
        ts: Option<String>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        Self::add_with_many_params(
            DYNAMIC_LOG,
            &TardisFuns::json.obj_to_string(content)?,
            ext,
            kind,
            key,
            op,
            rel_key,
            ts,
            Some(ctx.owner.clone()),
            Some(ctx.own_paths.clone()),
            funs,
            ctx,
        )
        .await?;
        Ok(())
    }

    pub async fn add(req: &LogItemAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let log_url: String = BaseSpiClient::module_url(InvokeModuleKind::Log, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;
        funs.web_client().post_obj_to_str(&format!("{log_url}/ci/item"), &req, headers.clone()).await?;
        Ok(())
    }

    #[deprecated = "this function has too many parameters, use `SpiLogClient::add` instead"]
    pub async fn add_with_many_params(
        tag: &str,
        content: &str,
        ext: Option<Value>,
        kind: Option<String>,
        key: Option<String>,
        op: Option<String>,
        rel_key: Option<String>,
        ts: Option<String>,
        owner: Option<String>,
        own_paths: Option<String>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let req = LogItemAddReq {
            tag: tag.to_string(),
            content: content.to_string(),
            kind,
            ext,
            key,
            op,
            rel_key,
            id: None,
            ts: ts.map(|ts| DateTime::parse_from_rfc3339(&ts).unwrap_or_default().with_timezone(&Utc)),
            owner,
            own_paths,
        };
        Self::add(&req, funs, ctx).await
    }

    pub async fn find(find_req: LogItemFindReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<TardisPage<LogItemFindResp>>> {
        let log_url: String = BaseSpiClient::module_url(InvokeModuleKind::Log, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;
        let resp = funs.web_client().put::<LogItemFindReq, TardisResp<TardisPage<LogItemFindResp>>>(&format!("{log_url}/ci/item/find"), &find_req, headers.clone()).await?;
        BaseSpiClient::package_resp(resp)
    }
}

pub struct LogEventClient {}

#[async_trait]
pub trait SpiLogEventExt {
    async fn publish_add_log(&self, req: &LogItemAddReq, from: String, spi_app_id: String, ctx: &TardisContext) -> TardisResult<()>;
}

#[async_trait]
impl SpiLogEventExt for TardisWSClient {
    async fn publish_add_log(&self, req: &LogItemAddReq, from: String, spi_app_id: String, ctx: &TardisContext) -> TardisResult<()> {
        let spi_ctx = TardisContext { owner: spi_app_id, ..ctx.clone() };
        let req = TardisWebsocketReq {
            msg: TardisFuns::json.obj_to_json(&(req, spi_ctx)).expect("invalid json"),
            to_avatars: Some(vec!["spi-log/service".into()]),
            from_avatar: from,
            event: Some("spi-log/add".into()),
            ..Default::default()
        };
        info!("event add log {}", TardisFuns::json.obj_to_string(&req).expect("invalid json"));
        self.send_obj(&req).await?;
        return Ok(());
    }
}
