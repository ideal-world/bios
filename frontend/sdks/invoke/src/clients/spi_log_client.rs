use serde::{Deserialize, Serialize};

#[cfg(feature = "event")]
use crate::invoke_config::InvokeConfigApi as _;
use crate::{clients::base_spi_client::BaseSpiClient, invoke_config::InvokeConfig, invoke_constants::DYNAMIC_LOG, invoke_enumeration::InvokeModuleKind};
#[cfg(feature = "event")]
use tardis::futures::TryFutureExt as _;
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

#[cfg(feature = "event")]
use super::event_client::{mq_client_node_opt, mq_error, EventAttributeExt, SPI_RPC_TOPIC};
use super::iam_client::IamClient;

#[cfg(feature = "event")]
pub mod event {
    use asteroid_mq_sdk::model::{event::EventAttribute, MessageDurableConfig, Subject};
    use tardis::chrono::{Duration, Utc};

    impl EventAttribute for super::LogItemAddV2Req {
        const SUBJECT: Subject = Subject::const_new("log/add");
        fn durable_config() -> Option<MessageDurableConfig> {
            Some(MessageDurableConfig {
                // 两个月后过期
                expire: Utc::now() + Duration::days(60),
                max_receiver: Some(1),
            })
        }
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
    pub content: String,
    pub kind: Option<String>,
    pub ext: Option<Value>,
    pub key: Option<String>,
    pub op: Option<String>,
    pub rel_key: Option<String>,
    pub id: Option<String>,
    pub ts: Option<DateTime<Utc>>,
    pub data_source: Option<String>,
    pub owner: Option<String>,
    pub own_paths: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct LogItemAddV2Req {
    pub tag: String,
    pub content: Value,
    pub kind: Option<String>,
    pub ext: Option<Value>,
    pub key: Option<String>,
    pub op: Option<String>,
    pub rel_key: Option<String>,
    pub idempotent_id: Option<String>,
    pub ts: Option<DateTime<Utc>>,
    pub data_source: Option<String>,
    pub owner: Option<String>,
    pub owner_name: Option<String>,
    pub own_paths: Option<String>,
    pub disable: Option<bool>,
    pub push: bool,
    pub ignore_push: Option<bool>,
    pub msg: Option<String>,
}

impl LogItemAddV2Req {
    pub fn make_valid(&mut self) {
        if self.kind.as_ref().is_some_and(String::is_empty) {
            self.kind = None
        }
        macro_rules! not_empty_or_none {
            ($obj: ident: $($field: ident,)*) => {
                $(
                    if $obj.$field.as_ref().is_some_and(String::is_empty) {
                        $obj.$field = None
                    }
                )*
            };
        }
        not_empty_or_none! {
            self:
                kind,
                key,
                op,
                rel_key,
                idempotent_id,
                owner,
                owner_name,
        }
    }
}

impl SpiLogClient {
    #[deprecated]
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
        let req = LogItemAddReq {
            tag: DYNAMIC_LOG.to_string(),
            content: TardisFuns::json.obj_to_string(content)?,
            kind,
            ext,
            key,
            op,
            rel_key,
            id: None,
            ts: ts.map(|ts| DateTime::parse_from_rfc3339(&ts).unwrap_or_default().with_timezone(&Utc)),
            data_source: None,
            owner: Some(ctx.owner.clone()),
            own_paths: Some(ctx.own_paths.clone()),
        };
        Self::add(req, funs, ctx).await?;
        Ok(())
    }

    pub async fn add_dynamic_log_v2(
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
        let owner_name = IamClient::new("", funs, ctx, cfg.module_urls.get("iam").expect("missing iam base url")).get_account(&ctx.owner, &ctx.own_paths).await?.owner_name;
        let req = LogItemAddV2Req {
            tag: DYNAMIC_LOG.to_string(),
            content: TardisFuns::json.obj_to_json(content)?,
            kind,
            ext,
            key,
            op,
            rel_key,
            idempotent_id: None,
            ts: ts.map(|ts| DateTime::parse_from_rfc3339(&ts).unwrap_or_default().with_timezone(&Utc)),
            data_source: None,
            owner: Some(ctx.owner.clone()),
            own_paths: Some(ctx.own_paths.clone()),
            msg: None,
            owner_name,
            push: false,
            ignore_push: None,
            disable: None,
        };
        Self::addv2(req, funs, ctx).await?;
        Ok(())
    }

    #[deprecated]
    pub async fn add(req: LogItemAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let log_url: String = BaseSpiClient::module_url(InvokeModuleKind::Log, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;
        funs.web_client().post_obj_to_str(&format!("{log_url}/ci/item"), &req, headers.clone()).await?;
        Ok(())
    }

    pub async fn addv2(mut req: LogItemAddV2Req, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        req.make_valid();
        #[cfg(feature = "event")]
        if funs.invoke_conf_in_event(InvokeModuleKind::Log) {
            if let Some(node) = mq_client_node_opt() {
                node.send_event(SPI_RPC_TOPIC, req.inject_context(funs, ctx).json()).map_err(mq_error).await?;
                return Ok(());
            }
        }
        let log_url: String = BaseSpiClient::module_url(InvokeModuleKind::Log, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;
        funs.web_client().post_obj_to_str(&format!("{log_url}/ci/v2/item"), &req, headers.clone()).await?;
        Ok(())
    }

    #[deprecated]
    pub async fn find(find_req: LogItemFindReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<TardisPage<LogItemFindResp>>> {
        Self::do_find(find_req, "/ci/item/find", funs, ctx).await
    }

    pub async fn findv2(find_req: LogItemFindReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<TardisPage<LogItemFindResp>>> {
        Self::do_find(find_req, "/ci/v2/item/find", funs, ctx).await
    }

    async fn do_find(find_req: LogItemFindReq, path: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<TardisPage<LogItemFindResp>>> {
        let log_url: String = BaseSpiClient::module_url(InvokeModuleKind::Log, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;
        let resp = funs.web_client().put::<LogItemFindReq, TardisResp<TardisPage<LogItemFindResp>>>(&format!("{log_url}{path}"), &find_req, headers.clone()).await?;
        BaseSpiClient::package_resp(resp)
    }
}
