use bios_basic::{dto::BasicQueryCondInfo, enumeration::BasicQueryOpKind};
use serde::{Deserialize, Serialize};
use tardis::{
    basic::field::TrimString,
    chrono::{DateTime, Utc},
    serde_json::Value,
    web::poem_openapi,
};

#[derive(poem_openapi::Object, Serialize, Deserialize, Clone, Debug)]
pub struct LogItemAddReq {
    #[oai(validator(pattern = r"^[a-z0-9_]+$"))]
    pub tag: String,
    pub content: Value,
    #[oai(validator(min_length = "2"))]
    pub kind: Option<TrimString>,
    pub ext: Option<Value>,
    #[oai(validator(min_length = "2"))]
    pub key: Option<TrimString>,
    #[oai(validator(min_length = "2"))]
    pub op: Option<String>,
    #[oai(validator(min_length = "2"))]
    pub rel_key: Option<TrimString>,
    #[oai(validator(min_length = "2"))]
    pub idempotent_id: Option<String>,
    pub ts: Option<DateTime<Utc>>,
    #[oai(validator(min_length = "2"))]
    pub owner: Option<String>,
    #[oai(validator(min_length = "1"))]
    pub owner_name: Option<String>,
    pub own_paths: Option<String>,
    pub push: bool,
    pub msg: Option<String>,
}
impl From<bios_sdk_invoke::clients::spi_log_client::LogItemAddReq> for LogItemAddReq {
    fn from(value: bios_sdk_invoke::clients::spi_log_client::LogItemAddReq) -> Self {
        Self {
            tag: value.tag,
            content: value.content,
            kind: value.kind.map(Into::into),
            ext: value.ext,
            key: value.key.map(Into::into),
            op: value.op,
            rel_key: value.rel_key.map(Into::into),
            idempotent_id: value.idempotent_id,
            ts: value.ts,
            owner: value.owner,
            own_paths: value.own_paths,
            msg: value.msg,
            owner_name: value.owner_name,
            push: value.push,
        }
    }
}
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct LogItemFindReq {
    #[oai(validator(pattern = r"^[a-z0-9_]+$"))]
    pub tag: String,
    pub kinds: Option<Vec<TrimString>>,
    pub keys: Option<Vec<TrimString>>,
    pub ops: Option<Vec<String>>,
    pub owners: Option<Vec<String>>,
    pub own_paths: Option<String>,
    pub ext_or: Option<Vec<BasicQueryCondInfo>>,
    // Extended filtering conditions
    pub ext: Option<Vec<BasicQueryCondInfo>>,
    // Advanced search
    pub adv_query: Option<Vec<AdvLogItemQueryReq>>,
    pub rel_keys: Option<Vec<TrimString>>,
    pub ts_start: Option<DateTime<Utc>>,
    pub ts_end: Option<DateTime<Utc>>,
    pub page_number: u32,
    pub page_size: u16,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Default)]
pub struct AdvLogItemQueryReq {
    pub group_by_or: Option<bool>,
    // Extended filtering conditions
    pub ext: Option<Vec<AdvBasicQueryCondInfo>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct AdvBasicQueryCondInfo {
    pub in_ext: Option<bool>,
    #[oai(validator(min_length = "1"))]
    pub field: String,
    pub op: BasicQueryOpKind,
    pub value: Value,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct LogItemFindResp {
    pub content: Value,
    pub kind: String,
    pub ext: Value,
    pub owner: String,
    pub owner_name: String,
    pub own_paths: String,
    pub id: String,
    pub key: String,
    pub op: String,
    pub rel_key: String,
    pub ts: DateTime<Utc>,
    pub msg: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct LogConfigReq {
    #[oai(validator(pattern = r"^[a-z0-9_]+$"))]
    pub tag: String,
    pub ref_field: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsItemAddReq {
    #[oai(validator(min_length = "2"))]
    pub idempotent_id: Option<String>,
    #[oai(validator(pattern = r"^[a-z0-9_]+$"))]
    pub tag: String,
    pub content: Value,
    pub ext: Option<Value>,
    #[oai(validator(min_length = "2"))]
    pub key: Option<TrimString>,
    pub ts: Option<DateTime<Utc>>,
    #[oai(validator(min_length = "2"))]
    pub owner: Option<String>,
    pub own_paths: Option<String>,
}

impl From<LogItemAddReq> for StatsItemAddReq {
    fn from(value: LogItemAddReq) -> Self {
        StatsItemAddReq {
            idempotent_id: value.idempotent_id,
            tag: value.tag,
            content: value.content,
            ext: value.ext,
            key: value.key,
            ts: value.ts,
            owner: value.owner,
            own_paths: value.own_paths,
        }
    }
}
