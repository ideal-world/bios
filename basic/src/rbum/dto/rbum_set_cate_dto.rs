use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::chrono::{DateTime, Utc};
use tardis::web::poem_openapi::Object;

use crate::rbum::rbum_enumeration::RbumScopeLevelKind;

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumSetCateAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub bus_code: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: TrimString,
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub icon: Option<String>,
    pub sort: Option<i32>,
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub ext: Option<String>,
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub rbum_sibling_cate_id: Option<String>,
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub rbum_parent_cate_id: Option<String>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub rel_rbum_set_id: String,

    pub scope_level: RbumScopeLevelKind,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumSetCateModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub bus_code: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub icon: Option<String>,
    pub sort: Option<i32>,
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub ext: Option<String>,

    pub scope_level: Option<RbumScopeLevelKind>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object, tardis::db::sea_orm::FromQueryResult))]
pub struct RbumSetCateSummaryResp {
    pub id: String,
    pub bus_code: String,
    pub name: String,
    pub icon: String,
    pub sort: i32,
    pub ext: String,

    pub own_paths: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object))]
pub struct RbumSetCateSummaryWithPidResp {
    pub id: String,
    pub bus_code: String,
    pub name: String,
    pub icon: String,
    pub sort: i32,
    pub ext: String,

    pub own_paths: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
    pub pid: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object, tardis::db::sea_orm::FromQueryResult))]
pub struct RbumSetCateDetailResp {
    pub id: String,
    pub bus_code: String,
    pub name: String,
    pub icon: String,
    pub sort: i32,
    pub ext: String,

    pub own_paths: String,
    pub owner: String,
    pub owner_name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
}
