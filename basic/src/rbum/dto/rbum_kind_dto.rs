use crate::rbum::enumeration::RbumScopeKind;
use serde::{Deserialize, Serialize};
use tardis::chrono::{DateTime, Utc};
use tardis::db::sea_orm::*;
use tardis::web::poem_openapi::Object;

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumKindAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub id: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: String,
    #[oai(validator(max_length = "2000"))]
    pub note: String,
    #[oai(validator(max_length = "1000"))]
    pub icon: String,
    pub sort: i32,
    #[oai(validator(max_length = "255"))]
    pub ext_table_name: String,

    #[oai(validator(max_length = "255"))]
    pub rel_app_id: Option<String>,
    pub scope_kind: RbumScopeKind,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumKindModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: Option<String>,
    #[oai(validator(max_length = "2000"))]
    pub note: Option<String>,
    #[oai(validator(max_length = "1000"))]
    pub icon: Option<String>,
    pub sort: Option<i32>,
    #[oai(validator(max_length = "255"))]
    pub ext_table_name: Option<String>,

    #[oai(validator(max_length = "255"))]
    pub rel_app_id: Option<String>,
    pub scope_kind: Option<RbumScopeKind>,
}

#[derive(Object, FromQueryResult, Serialize, Deserialize, Debug)]
pub struct RbumKindSummaryResp {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub sort: i32,
    pub ext_table_name: String,
}

#[derive(Object, FromQueryResult, Serialize, Deserialize, Debug)]
pub struct RbumKindDetailResp {
    pub id: String,
    pub name: String,
    pub note: String,
    pub icon: String,
    pub sort: i32,
    pub ext_table_name: String,

    pub rel_app_id: String,
    pub rel_app_name: String,
    pub rel_tenant_id: String,
    pub rel_tenant_name: String,
    pub updater_id: String,
    pub updater_name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub scope_kind: RbumScopeKind,
}
