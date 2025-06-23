use std::collections::HashMap;

use bios_basic::{dto::BasicQueryCondInfo, enumeration::BasicQueryOpKind};
use bios_sdk_invoke::dto::stats_record_dto::StatsFactRecordLoadReq;
use serde::{Deserialize, Serialize};
use tardis::{
    basic::field::TrimString,
    chrono::{DateTime, Utc},
    serde_json::Value,
    web::poem_openapi,
};

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct LogItemAddReq {
    #[oai(validator(pattern = r"^[a-z0-9_]+$"))]
    pub tag: String,
    // #[oai(validator(min_length = "2"))]
    pub content: String,
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
    pub id: Option<String>,
    pub ts: Option<DateTime<Utc>>,
    pub data_source: Option<String>,
    #[oai(validator(min_length = "2"))]
    pub owner: Option<String>,
    pub own_paths: Option<String>,
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
            id: value.id,
            ts: value.ts,
            data_source: value.data_source,
            owner: value.owner,
            own_paths: value.own_paths,
        }
    }
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Clone, Debug)]
pub struct LogItemAddV2Req {
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
    pub data_source: Option<String>,
    #[oai(validator(min_length = "2"))]
    pub owner: Option<String>,
    #[oai(validator(min_length = "1"))]
    pub owner_name: Option<String>,
    pub own_paths: Option<String>,
    pub disable: Option<bool>,
    pub push: Option<bool>,
    pub ignore_push: Option<bool>,
    pub msg: Option<String>,
}
impl From<bios_sdk_invoke::clients::spi_log_client::LogItemAddV2Req> for LogItemAddV2Req {
    fn from(value: bios_sdk_invoke::clients::spi_log_client::LogItemAddV2Req) -> Self {
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
            data_source: value.data_source,
            owner: value.owner,
            own_paths: value.own_paths,
            msg: value.msg,
            owner_name: value.owner_name,
            push: value.push,
            disable: value.disable,
            ignore_push: value.ignore_push,
        }
    }
}
impl From<LogItemAddV2Req> for StatsFactRecordLoadReq {
    fn from(value: LogItemAddV2Req) -> Self {
        StatsFactRecordLoadReq {
            own_paths: value.own_paths.unwrap_or_default(),
            ct: value.ts.unwrap_or_default(),
            idempotent_id: value.idempotent_id,
            ignore_updates: None,
            data: value.content,
            ext: value.ext,
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
    pub kind: Vec<String>,
    pub ext: Value,
    pub data_source: String,
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
pub struct LogExportDataReq {
    pub tags: Vec<String>,
    pub tags_v2: Vec<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct LogExportDataResp {
    pub tag_data: HashMap<String, Vec<LogExportAggResp>>,
    pub tag_v2_data: HashMap<String, Vec<LogExportV2AggResp>>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct LogImportDataReq {
    pub data_source: String,
    pub tag_data: HashMap<String, Vec<LogImportAggReq>>,
    pub tag_v2_data: HashMap<String, Vec<LogImportV2AggReq>>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct LogExportAggResp {
    pub tag: String,
    pub content: String,
    pub kind: String,
    pub ext: Value,
    pub key: String,
    pub op: String,
    pub rel_key: String,
    pub id: String,
    pub ts: DateTime<Utc>,
    pub data_source: String,
    pub owner: String,
    pub own_paths: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct LogExportV2AggResp {
    pub tag: String,
    pub content: Value,
    pub kind: String,
    pub ext: Value,
    pub key: String,
    pub op: String,
    pub rel_key: String,
    pub idempotent_id: String,
    pub ts: DateTime<Utc>,
    pub data_source: String,
    pub owner: String,
    pub owner_name: String,
    pub own_paths: String,
    pub push: bool,
    pub disable: bool,
    pub msg: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct LogImportAggReq {
    pub tag: String,
    pub content: String,
    pub kind: String,
    pub ext: Value,
    pub key: String,
    pub op: String,
    pub rel_key: String,
    pub id: String,
    pub ts: DateTime<Utc>,
    pub data_source: String,
    pub owner: String,
    pub own_paths: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct LogImportV2AggReq {
    pub tag: String,
    pub content: Value,
    pub kind: String,
    pub ext: Value,
    pub key: String,
    pub op: String,
    pub rel_key: String,
    pub idempotent_id: String,
    pub ts: DateTime<Utc>,
    pub data_source: String,
    pub owner: String,
    pub owner_name: String,
    pub own_paths: String,
    pub push: bool,
    pub disable: bool,
    pub msg: String,
}
