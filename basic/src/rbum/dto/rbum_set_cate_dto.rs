use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::chrono::{DateTime, Utc};

use crate::rbum::rbum_enumeration::RbumScopeLevelKind;

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object))]
pub struct RbumSetCateAddReq {
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub bus_code: TrimString,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub name: TrimString,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "1000")))]
    pub icon: Option<String>,
    pub sort: Option<u32>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "1000")))]
    pub ext: Option<String>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "1000")))]
    pub rbum_parent_cate_id: Option<String>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub rel_rbum_set_id: String,

    pub scope_level: RbumScopeLevelKind,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object))]
pub struct RbumSetCateModifyReq {
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub bus_code: Option<TrimString>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub name: Option<TrimString>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "1000")))]
    pub icon: Option<String>,
    pub sort: Option<u32>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "1000")))]
    pub ext: Option<String>,

    pub scope_level: Option<RbumScopeLevelKind>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object, tardis::db::sea_orm::FromQueryResult))]
pub struct RbumSetCateSummaryResp {
    pub id: String,
    pub sys_code: String,
    pub bus_code: String,
    pub name: String,
    pub icon: String,
    pub sort: u32,
    pub ext: String,

    pub rel_rbum_set_id: String,

    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object))]
pub struct RbumSetTreeResp {
    pub id: String,
    pub bus_code: String,
    pub name: String,
    pub icon: String,
    pub sort: u32,
    pub ext: String,

    pub own_paths: String,
    pub owner: String,

    pub scope_level: RbumScopeLevelKind,
    pub pid: Option<String>,

    pub rbum_set_items: Vec<RbumSetItemInfoResp>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object, tardis::db::sea_orm::FromQueryResult))]
pub struct RbumSetItemInfoResp {
    pub id: String,
    pub sort: u32,
    pub rel_rbum_item_id: String,
    pub rel_rbum_item_name: String,

    pub own_paths: String,
    pub owner: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object, tardis::db::sea_orm::FromQueryResult))]
pub struct RbumSetCateDetailResp {
    pub id: String,
    pub sys_code: String,
    pub bus_code: String,
    pub name: String,
    pub icon: String,
    pub sort: u32,
    pub ext: String,

    pub rel_rbum_set_id: String,

    pub own_paths: String,
    pub owner: String,
    pub owner_name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
}
