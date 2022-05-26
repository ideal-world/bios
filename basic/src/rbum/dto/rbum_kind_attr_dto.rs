use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::chrono::{DateTime, Utc};

use crate::rbum::rbum_enumeration::{RbumDataTypeKind, RbumScopeLevelKind, RbumWidgetTypeKind};

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object))]
pub struct RbumKindAttrAddReq {
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub name: TrimString,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub module: Option<TrimString>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub label: String,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "2000")))]
    pub note: Option<String>,
    pub sort: Option<u32>,
    pub main_column: Option<bool>,
    pub position: Option<bool>,
    pub capacity: Option<bool>,
    pub overload: Option<bool>,
    pub hide: Option<bool>,
    pub idx: Option<bool>,
    pub data_type: RbumDataTypeKind,
    pub widget_type: RbumWidgetTypeKind,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "2000")))]
    pub default_value: Option<String>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub options: Option<String>,
    pub required: Option<bool>,
    pub min_length: Option<u32>,
    pub max_length: Option<u32>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "2000")))]
    pub action: Option<String>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "2000")))]
    pub ext: Option<String>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub rel_rbum_kind_id: String,

    pub scope_level: Option<RbumScopeLevelKind>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object))]
pub struct RbumKindAttrModifyReq {
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub label: Option<String>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "2000")))]
    pub note: Option<String>,
    pub sort: Option<u32>,
    pub main_column: Option<bool>,
    pub position: Option<bool>,
    pub capacity: Option<bool>,
    pub overload: Option<bool>,
    pub hide: Option<bool>,
    pub idx: Option<bool>,
    pub data_type: Option<RbumDataTypeKind>,
    pub widget_type: Option<RbumWidgetTypeKind>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "2000")))]
    pub default_value: Option<String>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub options: Option<String>,
    pub required: Option<bool>,
    pub min_length: Option<u32>,
    pub max_length: Option<u32>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "2000")))]
    pub action: Option<String>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "2000")))]
    pub ext: Option<String>,

    pub scope_level: Option<RbumScopeLevelKind>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object, tardis::db::sea_orm::FromQueryResult))]
pub struct RbumKindAttrSummaryResp {
    pub id: String,
    pub name: String,
    pub module: String,
    pub label: String,
    pub note: String,
    pub sort: u32,
    pub main_column: bool,
    pub position: bool,
    pub capacity: bool,
    pub overload: bool,
    pub hide: bool,
    pub idx: bool,
    pub data_type: RbumDataTypeKind,
    pub widget_type: RbumWidgetTypeKind,
    pub default_value: String,
    pub options: String,
    pub required: bool,
    pub min_length: u32,
    pub max_length: u32,
    pub action: String,
    pub ext: String,
    pub rel_rbum_kind_id: String,

    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object, tardis::db::sea_orm::FromQueryResult))]
pub struct RbumKindAttrDetailResp {
    pub id: String,
    pub name: String,
    pub module: String,
    pub label: String,
    pub note: String,
    pub sort: u32,
    pub main_column: bool,
    pub position: bool,
    pub capacity: bool,
    pub overload: bool,
    pub hide: bool,
    pub idx: bool,
    pub data_type: RbumDataTypeKind,
    pub widget_type: RbumWidgetTypeKind,
    pub default_value: String,
    pub options: String,
    pub required: bool,
    pub min_length: u32,
    pub max_length: u32,
    pub action: String,
    pub ext: String,
    pub rel_rbum_kind_id: String,
    pub rel_rbum_kind_name: String,

    pub own_paths: String,
    pub owner: String,
    pub owner_name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
}
