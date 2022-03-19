use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::chrono::{DateTime, Utc};
use tardis::web::poem_openapi::Object;

use crate::rbum::enumeration::RbumScopeKind;

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumSetCateAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub bus_code: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: TrimString,
    pub sort: Option<i32>,
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub rbum_sibling_cate_id: Option<String>,
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub rbum_parent_cate_id: Option<String>,

    pub scope_kind: Option<RbumScopeKind>,

    #[oai(validator(min_length = "2", max_length = "255"))]
    pub rel_rbum_set_id: String,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumSetCateModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub bus_code: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: Option<TrimString>,
    pub sort: Option<i32>,

    pub scope_kind: Option<RbumScopeKind>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object, tardis::db::sea_orm::FromQueryResult))]
pub struct RbumSetCateSummaryResp {
    pub id: String,
    pub bus_code: String,
    pub name: String,
    pub sort: i32,

    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_kind: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object))]
pub struct RbumSetCateSummaryWithPidResp {
    pub id: String,
    pub bus_code: String,
    pub name: String,
    pub sort: i32,

    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_kind: String,
    
    pub pid: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object, tardis::db::sea_orm::FromQueryResult))]
pub struct RbumSetCateDetailResp {
    pub id: String,
    pub bus_code: String,
    pub name: String,
    pub sort: i32,

    pub rel_app_id: String,
    pub rel_app_name: String,
    pub updater_id: String,
    pub updater_name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_kind: String,
}
