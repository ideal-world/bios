use chrono::{DateTime, Utc};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use tardis::db::FromQueryResult;

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumKindAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub code: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: String,
    #[oai(validator(max_length = "2000"))]
    pub note: String,
    #[oai(validator(max_length = "1000"))]
    pub icon: String,
    pub sort: i32,

    #[oai(validator(max_length = "255"))]
    pub ext_table_name: String,
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
    pub scope_kind: Option<String>,

    #[oai(validator(max_length = "255"))]
    pub ext_table_name: Option<String>,
}

#[derive(Object, FromQueryResult, Serialize, Deserialize, Debug)]
pub struct RbumKindSummaryResp {
    pub id: String,
    pub code: String,
    pub name: String,
    pub icon: String,
    pub sort: i32,

    pub ext_table_name: String,
}

#[derive(Object, FromQueryResult, Serialize, Deserialize, Debug)]
pub struct RbumKindDetailResp {
    pub id: String,
    pub rel_app_name: String,
    pub rel_tenant_name: String,
    pub creator_name: String,
    pub updater_name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub code: String,
    pub name: String,
    pub note: String,
    pub icon: String,
    pub sort: i32,
    pub scope_kind: String,

    pub ext_table_name: String,
}
