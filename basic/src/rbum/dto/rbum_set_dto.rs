use crate::rbum::dto::rbum_domain_dto::RbumDomainSummaryResp;
use crate::rbum::dto::rbum_kind_dto::RbumKindSummaryResp;
use crate::rbum::dto::rbum_set_item_dto::RbumSetItemInfoResp;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tardis::basic::field::TrimString;
use tardis::chrono::{DateTime, Utc};
#[cfg(feature = "default")]
use tardis::db::sea_orm;
#[cfg(feature = "default")]
use tardis::web::poem_openapi;

use crate::rbum::rbum_enumeration::RbumScopeLevelKind;

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct RbumSetAddReq {
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub code: TrimString,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub kind: TrimString,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub name: TrimString,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "2000")))]
    pub note: Option<String>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "1000")))]
    pub icon: Option<String>,
    pub sort: Option<i64>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "1000")))]
    pub ext: Option<String>,

    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct RbumSetModifyReq {
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub name: Option<TrimString>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "2000")))]
    pub note: Option<String>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "1000")))]
    pub icon: Option<String>,
    pub sort: Option<i64>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "1000")))]
    pub ext: Option<String>,

    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object, sea_orm::FromQueryResult))]
pub struct RbumSetSummaryResp {
    pub id: String,
    pub code: String,
    pub kind: String,
    pub name: String,
    pub icon: String,
    pub sort: i64,
    pub ext: String,

    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
    pub disabled: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object, sea_orm::FromQueryResult))]
pub struct RbumSetDetailResp {
    pub id: String,
    pub code: String,
    pub kind: String,
    pub name: String,
    pub note: String,
    pub icon: String,
    pub sort: i64,
    pub ext: String,

    pub own_paths: String,
    pub owner: String,
    pub owner_name: Option<String>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
    pub disabled: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object, sea_orm::FromQueryResult))]
pub struct RbumSetPathResp {
    pub id: String,
    pub name: String,

    pub own_paths: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct RbumSetTreeResp {
    pub main: Vec<RbumSetTreeMainResp>,
    pub ext: Option<RbumSetTreeExtResp>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct RbumSetTreeMainResp {
    pub id: String,
    pub sys_code: String,
    pub bus_code: String,
    pub name: String,
    pub icon: String,
    pub sort: i64,
    pub ext: String,

    pub own_paths: String,
    pub owner: String,

    pub scope_level: RbumScopeLevelKind,
    pub pid: Option<String>,
    //关联的set_id
    pub rel: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct RbumSetTreeExtResp {
    // cate.id -> items
    pub items: HashMap<String, Vec<RbumSetItemInfoResp>>,
    // cate.id -> kind.id -> item number
    pub item_number_agg: HashMap<String, HashMap<String, u64>>,
    // kind.id -> kind info
    pub item_kinds: HashMap<String, RbumKindSummaryResp>,
    // domain.id -> domain info
    pub item_domains: HashMap<String, RbumDomainSummaryResp>,
}
