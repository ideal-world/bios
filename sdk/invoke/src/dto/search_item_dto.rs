use serde::{Deserialize, Serialize};
use tardis::{
    basic::field::TrimString,
    chrono::{DateTime, Utc},
    serde_json::Value,
    web::poem_openapi,
};

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct SearchItemAddReq {
    #[oai(validator(pattern = r"^[a-z0-9-_]+$"))]
    pub tag: String,
    #[oai(validator(min_length = "2"))]
    pub kind: String,
    #[oai(validator(min_length = "2"))]
    pub key: TrimString,
    #[oai(validator(min_length = "2"))]
    pub title: String,
    #[oai(validator(min_length = "2"))]
    pub name: Option<String>,
    // #[oai(validator(min_length = "2"))]
    pub content: String,
    #[oai(validator(min_length = "2"))]
    pub owner: Option<String>,
    #[oai(validator(min_length = "2"))]
    pub own_paths: Option<String>,
    pub create_time: Option<DateTime<Utc>>,
    pub update_time: Option<DateTime<Utc>>,
    pub ext: Option<Value>,
    pub visit_keys: Option<SearchItemVisitKeysReq>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct SearchItemModifyReq {
    #[oai(validator(min_length = "2"))]
    pub kind: Option<String>,
    #[oai(validator(min_length = "2"))]
    pub title: Option<String>,
    #[oai(validator(min_length = "2"))]
    pub name: Option<String>,
    // #[oai(validator(min_length = "2"))]
    pub content: Option<String>,
    #[oai(validator(min_length = "2"))]
    pub owner: Option<String>,
    // #[oai(validator(min_length = "2"))]
    pub own_paths: Option<String>,
    pub create_time: Option<DateTime<Utc>>,
    pub update_time: Option<DateTime<Utc>>,
    pub ext: Option<Value>,
    // Overwrites the original content when it is true
    pub ext_override: Option<bool>,
    pub visit_keys: Option<SearchItemVisitKeysReq>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct SearchItemVisitKeysReq {
    pub accounts: Option<Vec<String>>,
    pub apps: Option<Vec<String>>,
    pub tenants: Option<Vec<String>>,
    pub roles: Option<Vec<String>>,
    pub groups: Option<Vec<String>>,
}
