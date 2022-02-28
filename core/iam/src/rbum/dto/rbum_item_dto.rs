use serde::{Deserialize, Serialize};
use tardis::chrono::{DateTime, Utc};
use tardis::db::sea_orm::*;
use tardis::web::poem_openapi::Object;

use crate::rbum::enumeration::RbumScopeKind;

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumItemAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: String,
    #[oai(validator(max_length = "2000"))]
    pub uri_part: String,
    #[oai(validator(max_length = "1000"))]
    pub icon: String,
    pub sort: i32,
    #[oai(validator(max_length = "255"))]
    pub rel_rbum_domain_id: String,

    pub scope_kind: RbumScopeKind,
    pub disabled: bool,
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

    pub scope_kind: Option<RbumScopeKind>,
    pub disabled: Option<bool>,
}

#[derive(Object, FromQueryResult, Serialize, Deserialize, Debug)]
pub struct RbumItemSummaryResp {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub sort: i32,
}

#[derive(Object, FromQueryResult, Serialize, Deserialize, Debug)]
pub struct RbumItemDetailResp {
    pub id: String,
    pub name: String,
    pub uri_part: String,
    pub icon: String,
    pub sort: i32,
    pub rel_rbum_kind_id: String,
    pub rel_rbum_kind_name: String,
    pub rel_rbum_domain_id: String,
    // TODO
    // pub rel_rbum_domain_name: String,
    pub rel_app_id: String,
    pub rel_app_name: String,
    pub rel_tenant_id: String,
    pub rel_tenant_name: String,
    pub creator_id: String,
    pub creator_name: String,
    pub updater_id: String,
    pub updater_name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub scope_kind: RbumScopeKind,
    pub disabled: bool,
}
