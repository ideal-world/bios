use bios_basic::{enumeration::BasicQueryOpKind, dto::BasicQueryCondInfo};
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
    #[oai(validator(min_length = "2"))]
    pub owner: Option<String>,
    pub own_paths: Option<String>,
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
    #[oai(validator(min_length = "2"))]
    pub content: String,
    pub kind: String,
    pub ext: Value,
    pub owner: String,
    pub own_paths: String,
    pub id: String,
    pub key: String,
    pub op: String,
    pub rel_key: String,
    pub ts: DateTime<Utc>,
}
