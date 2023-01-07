use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tardis::{
    basic::field::TrimString,
    chrono::{DateTime, Utc},
    web::poem_openapi,
};

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct GraphRelAddReq {
    #[oai(validator(pattern = r"^[a-z0-9-_.]+$"))]
    pub tag: String,
    pub from_key: TrimString,
    #[oai(validator(pattern = r"^[a-z0-9-_.]+$"))]
    pub from_version: String,
    pub to_key: TrimString,
    #[oai(validator(pattern = r"^[a-z0-9-_.]+$"))]
    pub to_version: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct GraphRelUpgardeVersionReq {
    pub key: TrimString,
    #[oai(validator(pattern = r"^[a-z0-9-_.]+$"))]
    pub old_version: String,
    #[oai(validator(pattern = r"^[a-z0-9-_.]+$"))]
    pub new_version: String,
    pub del_rels: Vec<GraphRelUpgardeDelRelReq>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct GraphNodeVersionResp {
    pub version: String,
    pub ts: DateTime<Utc>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct GraphRelUpgardeDelRelReq {
    #[oai(validator(pattern = r"^[a-z0-9-_.]+$"))]
    pub tag: Option<String>,
    pub rel_key: Option<TrimString>,
    #[oai(validator(pattern = r"^[a-z0-9-_.]+$"))]
    pub rel_version: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct GraphRelDetailResp {
    pub key: String,
    pub version: String,
    pub form_rels: HashMap<String, Vec<GraphRelDetailResp>>,
    pub to_rels: HashMap<String, Vec<GraphRelDetailResp>>,
}
