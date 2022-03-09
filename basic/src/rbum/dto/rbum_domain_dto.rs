use serde::{Deserialize, Serialize};
use tardis::chrono::{DateTime, Utc};
use tardis::db::sea_orm::*;
use tardis::web::poem_openapi::Object;

use crate::rbum::enumeration::RbumScopeKind;

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumDomainAddReq {
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub uri_authority: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: String,
    #[oai(validator(max_length = "2000"))]
    pub note: String,
    #[oai(validator(max_length = "1000"))]
    pub icon: String,
    pub sort: i32,

    pub scope_kind: RbumScopeKind,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumDomainModifyReq {
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub uri_authority: Option<String>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: Option<String>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub note: Option<String>,
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub icon: Option<String>,
    pub sort: Option<i32>,

    pub scope_kind: Option<RbumScopeKind>,
}

#[derive(Object, FromQueryResult, Serialize, Deserialize, Debug)]
pub struct RbumDomainSummaryResp {
    pub id: String,
    pub uri_authority: String,
    pub name: String,
    pub icon: String,
    pub sort: i32,

    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_kind: RbumScopeKind,
}

#[derive(Object, FromQueryResult, Serialize, Deserialize, Debug)]
pub struct RbumDomainDetailResp {
    pub id: String,
    pub uri_authority: String,
    pub name: String,
    pub note: String,
    pub icon: String,
    pub sort: i32,

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
