use serde::{Deserialize, Serialize};
use tardis::{
    basic::field::TrimString,
    chrono::{DateTime, Utc},
    db::sea_orm,
    serde_json::Value,
    web::poem_openapi,
};

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct KvItemAddOrModifyReq {
    #[oai(validator(min_length = "2"))]
    pub key: TrimString,
    pub value: Value,
    pub disable: Option<bool>,
    pub info: Option<String>,
    pub scope_level: Option<i16>,
}
impl From<bios_sdk_invoke::clients::spi_kv_client::KvItemAddOrModifyReq> for KvItemAddOrModifyReq {
    fn from(req: bios_sdk_invoke::clients::spi_kv_client::KvItemAddOrModifyReq) -> Self {
        Self {
            key: req.key.into(),
            value: req.value,
            info: req.info,
            scope_level: req.scope_level,
            disable: None,
        }
    }
}
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, sea_orm::FromQueryResult)]
pub struct KvItemDetailResp {
    #[oai(validator(min_length = "2"))]
    pub key: String,
    pub value: Value,
    pub info: String,
    pub owner: String,
    pub own_paths: String,
    pub disable: bool,
    pub scope_level: i16,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, sea_orm::FromQueryResult)]
pub struct KvItemSummaryResp {
    #[oai(validator(min_length = "2"))]
    pub key: String,
    pub value: Value,
    pub info: String,
    pub owner: String,
    pub own_paths: String,
    pub disable: bool,
    pub scope_level: i16,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Default)]
pub struct KvItemMatchReq {
    pub key_prefix: String,
    pub query_path: Option<String>,
    pub query_values: Option<Value>,
    pub extract: Option<String>,
    pub create_time_start: Option<DateTime<Utc>>,
    pub create_time_end: Option<DateTime<Utc>>,
    pub update_time_start: Option<DateTime<Utc>>,
    pub update_time_end: Option<DateTime<Utc>>,
    pub page_number: u32,
    pub page_size: u16,
    pub disable: Option<bool>,
    pub key_like: Option<bool>,
    pub desc_sort_by_create: Option<bool>,
    pub desc_sort_by_update: Option<bool>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct KvItemKeyReq {
    #[oai(validator(min_length = "2"))]
    pub key: TrimString,
}

impl From<bios_sdk_invoke::clients::spi_kv_client::KvItemDeleteReq> for KvItemKeyReq {
    fn from(value: bios_sdk_invoke::clients::spi_kv_client::KvItemDeleteReq) -> Self {
        KvItemKeyReq { key: value.key.into() }
    }
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct KvNameAddOrModifyReq {
    #[oai(validator(min_length = "2"))]
    pub key: TrimString,
    pub name: String,
    pub disable: Option<bool>,
    pub scope_level: Option<i16>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct KvNameFindResp {
    pub key: String,
    pub name: String,
    pub disable: bool,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct KvTagAddOrModifyReq {
    #[oai(validator(min_length = "2"))]
    pub key: TrimString,
    pub items: Vec<KvTagItemAddReq>,
    pub disable: Option<bool>,
    pub scope_level: Option<i16>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct KvTagItemAddReq {
    pub code: TrimString,
    pub label: String,
    pub color: String,
    pub icon: String,
    pub url: Option<String>,
    pub service: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct KvTagFindResp {
    pub key: String,
    pub items: Vec<KvTagItemFindResp>,
    pub disable: bool,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct KvTagItemFindResp {
    pub code: String,
    pub label: String,
    pub color: String,
    pub icon: String,
    pub url: Option<String>,
    pub service: Option<String>,
}
