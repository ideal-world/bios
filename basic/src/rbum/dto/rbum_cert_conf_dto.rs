use serde::{Deserialize, Serialize};
use tardis::chrono::{DateTime, Utc};
use tardis::db::sea_orm::*;
use tardis::web::poem_openapi::Object;

use crate::rbum::enumeration::RbumScopeKind;

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumCertConfAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: String,
    #[oai(validator(max_length = "2000"))]
    pub note: String,
    #[oai(validator(max_length = "2000"))]
    pub ak_note: String,
    #[oai(validator(max_length = "2000"))]
    pub ak_rule: String,
    #[oai(validator(max_length = "2000"))]
    pub sk_note: String,
    #[oai(validator(max_length = "2000"))]
    pub sk_rule: String,
    pub repeatable: bool,
    pub is_basic: bool,
    #[oai(validator(max_length = "2000"))]
    pub rest_by_kinds: String,
    pub expire_sec: i32,
    pub coexist_num: i32,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumCertConfModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: Option<String>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub note: Option<String>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub ak_note: Option<String>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub ak_rule: Option<String>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub sk_note: Option<String>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub sk_rule: Option<String>,
    pub repeatable: Option<bool>,
    pub is_basic: Option<bool>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub rest_by_kinds: Option<String>,
    pub expire_sec: Option<i32>,
    pub coexist_num: Option<i32>,
}

#[derive(Object, FromQueryResult, Serialize, Deserialize, Debug)]
pub struct RbumCertConfSummaryResp {
    pub id: String,
    pub name: String,
    pub sort: i32,

    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

#[derive(Object, FromQueryResult, Serialize, Deserialize, Debug)]
pub struct RbumCertConfDetailResp {
    pub id: String,
    pub name: String,
    pub note: String,
    pub ak_note: String,
    pub ak_rule: String,
    pub sk_note: String,
    pub sk_rule: String,
    pub repeatable: bool,
    pub is_basic: bool,
    pub rest_by_kinds: String,
    pub expire_sec: i32,
    pub coexist_num: i32,

    pub rel_app_id: String,
    pub rel_app_name: String,
    pub rel_tenant_id: String,
    pub rel_tenant_name: String,
    pub updater_id: String,
    pub updater_name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}
