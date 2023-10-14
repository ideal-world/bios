use serde::{Deserialize, Serialize};

use tardis::{
    basic::{dto::TardisContext, field::TrimString, result::TardisResult},
    chrono::{DateTime, Utc},
    serde_json::{json, Value},
    web::{
        poem_openapi,
        web_resp::{TardisPage, TardisResp},
    },
    TardisFuns, TardisFunsInst,
};

use crate::{clients::base_spi_client::BaseSpiClient, invoke_constants::DYNAMIC_LOG, invoke_enumeration::InvokeModuleKind};

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
        Self::add(
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

    pub async fn add(
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
        let log_url: String = BaseSpiClient::module_url(InvokeModuleKind::Log, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;
        let body = json!({
            "tag": tag,
            "content": content,
            "owner": owner,
            "own_paths":own_paths,
            "kind": kind,
            "ext": ext,
            "key": key,
            "op": op,
            "rel_key": rel_key,
            "ts": ts,
        });
        funs.web_client().post_obj_to_str(&format!("{log_url}/ci/item"), &body, headers.clone()).await?;
        Ok(())
    }

    pub async fn find(find_req: LogItemFindReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<TardisPage<LogItemFindResp>>> {
        let log_url: String = BaseSpiClient::module_url(InvokeModuleKind::Log, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;
        let resp = funs.web_client().put::<LogItemFindReq, TardisResp<TardisPage<LogItemFindResp>>>(&format!("{log_url}/ci/item/find"), &find_req, headers.clone()).await?;
        BaseSpiClient::package_resp(resp)
    }
}
