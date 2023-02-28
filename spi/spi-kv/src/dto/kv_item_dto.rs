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
    pub info: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, sea_orm::FromQueryResult)]
pub struct KvItemDetailResp {
    #[oai(validator(min_length = "2"))]
    pub key: String,
    pub value: Value,
    pub info: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, sea_orm::FromQueryResult)]
pub struct KvItemSummaryResp {
    #[oai(validator(min_length = "2"))]
    pub key: String,
    pub value: Value,
    pub info: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct KvNameAddOrModifyReq {
    #[oai(validator(min_length = "2"))]
    pub key: TrimString,
    pub name: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct KvNameFindResp {
    pub key: String,
    pub name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct KvTagAddOrModifyReq {
    #[oai(validator(min_length = "2"))]
    pub key: TrimString,
    pub items: Vec<KvTagItemAddReq>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct KvTagItemAddReq {
    pub code: TrimString,
    pub label: String,
    pub color: String,
    pub icon: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct KvTagFindResp {
    pub key: String,
    pub items: Vec<KvTagItemFindResp>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct KvTagItemFindResp {
    pub code: String,
    pub label: String,
    pub color: String,
    pub icon: String,
}
