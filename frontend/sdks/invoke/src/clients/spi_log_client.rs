use serde::{Deserialize, Serialize};

use tardis::{
    basic::{dto::TardisContext, field::TrimString, result::TardisResult},
    chrono::{DateTime, Utc},
    futures::TryFutureExt as _,
    serde_json::Value,
    web::{
        poem_openapi,
        web_resp::{TardisPage, TardisResp},
    },
    TardisFuns, TardisFunsInst,
};

use crate::{clients::base_spi_client::BaseSpiClient, invoke_config::InvokeConfig, invoke_constants::DYNAMIC_LOG, invoke_enumeration::InvokeModuleKind};

use super::{
    event_client::{get_topic, mq_error, EventAttributeExt, SPI_RPC_TOPIC},
    iam_client::IamClient,
};

pub mod event {
    use asteroid_mq::prelude::*;

    pub const LOG_AVATAR: &str = "spi-log";

    impl EventAttribute for super::LogItemAddReq {
        const SUBJECT: Subject = Subject::const_new("log/add");
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
    pub idempotent_id: Option<String>,
    pub ts: Option<DateTime<Utc>>,
    pub owner: Option<String>,
    pub owner_name: Option<String>,
    pub own_paths: Option<String>,
    pub push: bool,
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
        let cfg = funs.conf::<InvokeConfig>();
        let owner_name = IamClient::new("", funs, &ctx, cfg.module_urls.get("iam").expect("missing iam base url")).get_account(&ctx.owner, &ctx.own_paths).await?.owner_name;
        let req = LogItemAddReq {
            tag: DYNAMIC_LOG.to_string(),
            content: TardisFuns::json.obj_to_json(content)?,
            kind,
            ext,
            key,
            op,
            rel_key,
            idempotent_id: None,
            ts: ts.map(|ts| DateTime::parse_from_rfc3339(&ts).unwrap_or_default().with_timezone(&Utc)),
            owner: Some(ctx.owner.clone()),
            own_paths: Some(ctx.own_paths.clone()),
            msg: None,
            owner_name: owner_name,
            push: false,
        };
        Self::add(req, funs, ctx).await?;
        Ok(())
    }

    pub async fn add(req: LogItemAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if let Some(topic) = get_topic(&SPI_RPC_TOPIC) {
            topic.send_event(req.inject_context(funs, ctx).json()).map_err(mq_error).await?;
        } else {
            let log_url: String = BaseSpiClient::module_url(InvokeModuleKind::Log, funs).await?;
            let headers = BaseSpiClient::headers(None, funs, ctx).await?;
            funs.web_client().post_obj_to_str(&format!("{log_url}/ci/item"), &req, headers.clone()).await?;
        }
        Ok(())
    }

    pub async fn find(find_req: LogItemFindReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<TardisPage<LogItemFindResp>>> {
        let log_url: String = BaseSpiClient::module_url(InvokeModuleKind::Log, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;
        let resp = funs.web_client().put::<LogItemFindReq, TardisResp<TardisPage<LogItemFindResp>>>(&format!("{log_url}/ci/item/find"), &find_req, headers.clone()).await?;
        BaseSpiClient::package_resp(resp)
    }
}
