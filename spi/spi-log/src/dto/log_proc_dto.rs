use serde::{Deserialize, Serialize};
use tardis::{
    chrono::{DateTime, Utc},
    web::poem_openapi,
};

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct LogItemAddReq {
    #[oai(validator(pattern = r"^[a-z0-9]+$"))]
    pub tag: String,
    #[oai(validator(min_length = "2"))]
    pub content: String,
    #[oai(validator(min_length = "2"))]
    pub key: Option<String>,
    #[oai(validator(min_length = "2"))]
    pub op: Option<String>,
    #[oai(validator(min_length = "2"))]
    pub rel_key: Option<String>,
    pub ts: Option<DateTime<Utc>>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct LogItemFindReq {
    #[oai(validator(pattern = r"^[a-z0-9]+$"))]
    pub tag: String,
    pub keys: Option<Vec<String>>,
    pub ops: Option<Vec<String>>,
    pub rel_keys: Option<Vec<String>>,
    pub ts_start: Option<DateTime<Utc>>,
    pub ts_end: Option<DateTime<Utc>>,
    pub page_number: u32,
    pub page_size: u16,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct LogItemFindResp {
    #[oai(validator(min_length = "2"))]
    pub content: String,
    pub key: String,
    pub op: String,
    pub rel_key: String,
    pub ts: DateTime<Utc>,
}
