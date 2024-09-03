use serde::{Deserialize, Serialize};

use tardis::{
    basic::{dto::TardisContext, field::TrimString, result::TardisResult},
    chrono::{DateTime, Utc},
    serde_json::Value,
    web::{
        poem_openapi,
        web_resp::{TardisPage, TardisResp},
    },
    TardisFuns, TardisFunsInst,
};

use crate::{clients::base_spi_client::BaseSpiClient, invoke_constants::DYNAMIC_LOG, invoke_enumeration::InvokeModuleKind};

pub mod event {
    use asteroid_mq::prelude::*;

    pub const LOG_AVATAR: &str = "spi-log";

    impl EventAttribute for super::LogItemAddReq {
        const SUBJECT: Subject = Subject::const_new(b"log/add");
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

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct LogItemAddReq {
    pub tag: String,
    pub content: Value,
    pub kind: Option<String>,
    pub ext: Option<Value>,
    pub key: Option<String>,
    pub op: Option<String>,
    pub rel_key: Option<String>,
    pub id: Option<String>,
    pub ts: Option<DateTime<Utc>>,
    pub owner: Option<String>,
    pub own_paths: Option<String>,
    pub msg: Option<String>,
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
            TardisFuns::json.obj_to_json(content)?,
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
        content: Value,
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
            content: content,
            kind,
            ext,
            key,
            op,
            rel_key,
            id: None,
            ts: ts.map(|ts| DateTime::parse_from_rfc3339(&ts).unwrap_or_default().with_timezone(&Utc)),
            owner,
            own_paths,
            msg: None,
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
