use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use tardis::chrono::{DateTime, Utc};
use tardis::db::sea_orm::*;

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumItemAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub code: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: String,
    #[oai(validator(max_length = "2000"))]
    pub uri_part: String,
    #[oai(validator(max_length = "1000"))]
    pub icon: String,
    pub sort: i32,

    #[oai(validator(max_length = "255"))]
    pub rel_rbum_kind_id: String,
    #[oai(validator(max_length = "255"))]
    pub rel_rbum_domain_id: String,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumItemModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: Option<String>,
    #[oai(validator(max_length = "2000"))]
    pub uri_part: Option<String>,
    #[oai(validator(max_length = "1000"))]
    pub icon: Option<String>,
    pub sort: Option<i32>,
    #[oai(validator(max_length = "255"))]
    pub scope_kind: Option<String>,

    pub disabled: Option<bool>,
}

#[derive(Object, FromQueryResult, Serialize, Deserialize, Debug)]
pub struct RbumItemSummaryResp {
    pub id: String,
    pub code: String,
    pub name: String,
    pub icon: String,
    pub sort: i32,
}

#[derive(Object, FromQueryResult, Serialize, Deserialize, Debug)]
pub struct RbumItemDetailResp {
    pub id: String,
    pub rel_app_name: String,
    pub rel_tenant_name: String,
    pub creator_name: String,
    pub updater_name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_kind: String,

    pub disabled: bool,

    pub code: String,
    pub name: String,
    pub uri_part: String,
    pub icon: String,
    pub sort: i32,

    pub rel_rbum_kind_name: String,
    pub rel_rbum_domain_name: String,
}
