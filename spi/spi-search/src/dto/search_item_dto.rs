use bios_basic::spi::dto::spi_basic_dto::SpiQueryCondReq;
use serde::{Deserialize, Serialize};
use tardis::{
    chrono::{DateTime, Utc},
    serde_json::Value,
    web::poem_openapi,
};

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct SearchItemAddOrModifyReq {
    #[oai(validator(min_length = "2"))]
    pub tag: String,
    #[oai(validator(min_length = "2"))]
    pub key: String,
    #[oai(validator(min_length = "2"))]
    pub title: String,
    #[oai(validator(min_length = "2"))]
    pub content: String,
    pub ext: Option<Value>,
    #[oai(validator(min_length = "2"))]
    pub owner: Option<String>,
    #[oai(validator(min_length = "2"))]
    pub own_paths: Option<String>,
    pub create_time: Option<DateTime<Utc>>,
    pub update_time: Option<DateTime<Utc>>,
    pub visit_keys: Option<Vec<String>>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct SearchItemSearchReq {
    pub ctx: SearchItemSearchCtxReq,
    pub query: SearchItemQueryReq,
    pub sort: Option<Vec<SearchItemQuerySortReq>>,
    pub page: Option<SearchItemQueryPageReq>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct SearchItemSearchCtxReq {
    #[oai(validator(min_length = "2"))]
    pub account: Option<String>,
    #[oai(validator(min_length = "2"))]
    pub app: Option<String>,
    #[oai(validator(min_length = "2"))]
    pub tenant: Option<String>,
    pub roles: Option<Vec<String>>,
    pub groups: Option<Vec<String>>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct SearchItemQueryReq {
    #[oai(validator(min_length = "2"))]
    pub tag: String,
    #[oai(validator(min_length = "2"))]
    pub key: Option<String>,
    #[oai(validator(min_length = "2"))]
    pub title: Option<String>,
    #[oai(validator(min_length = "2"))]
    pub content: Option<String>,
    #[oai(validator(min_length = "2"))]
    pub owner: Option<String>,
    #[oai(validator(min_length = "2"))]
    pub own_paths: Option<String>,
    pub create_time_start: Option<DateTime<Utc>>,
    pub create_time_end: Option<DateTime<Utc>>,
    pub update_time_start: Option<DateTime<Utc>>,
    pub update_time_end: Option<DateTime<Utc>>,
    pub ext: Option<Vec<SpiQueryCondReq>>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct SearchItemQuerySortReq {
    #[oai(validator(min_length = "2"))]
    pub field: String,
    pub order: SearchItemQuerySortKind,
}

#[derive(poem_openapi::Enum, Serialize, Deserialize, Debug)]
pub enum SearchItemQuerySortKind {
    Asc,
    Desc,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct SearchItemQueryPageReq {
    pub page_number: u32,
    pub page_size: u16,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct SearchItemQueryResp {
    #[oai(validator(min_length = "2"))]
    pub tag: String,
    #[oai(validator(min_length = "2"))]
    pub key: String,
    #[oai(validator(min_length = "2"))]
    pub title: String,
    #[oai(validator(min_length = "2"))]
    pub content: String,
    pub ext: Option<Value>,
    #[oai(validator(min_length = "2"))]
    pub owner: Option<String>,
    #[oai(validator(min_length = "2"))]
    pub own_paths: Option<String>,
    pub create_time: Option<DateTime<Utc>>,
    pub update_time: Option<DateTime<Utc>>,
}
