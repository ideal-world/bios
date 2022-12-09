use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::chrono::{DateTime, Utc};
#[cfg(feature = "default")]
use tardis::db::sea_orm;
#[cfg(feature = "default")]
use tardis::web::poem_openapi;

use crate::rbum::rbum_enumeration::{RbumDataTypeKind, RbumScopeLevelKind, RbumWidgetTypeKind};

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct RbumKindAttrAddReq {
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub name: TrimString,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub module: Option<TrimString>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub label: String,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "2000")))]
    pub note: Option<String>,
    pub sort: Option<i64>,
    pub main_column: Option<bool>,
    pub position: Option<bool>,
    pub capacity: Option<bool>,
    pub overload: Option<bool>,
    pub hide: Option<bool>,
    pub secret: Option<bool>,
    pub show_by_conds: Option<String>,
    pub idx: Option<bool>,
    pub data_type: RbumDataTypeKind,
    pub widget_type: RbumWidgetTypeKind,
    pub widget_columns: Option<i16>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "2000")))]
    pub default_value: Option<String>,
    pub dyn_default_value: Option<String>,
    pub options: Option<String>,
    pub dyn_options: Option<String>,
    pub required: Option<bool>,
    pub min_length: Option<i32>,
    pub max_length: Option<i32>,
    pub parent_attr_name: Option<TrimString>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "2000")))]
    pub action: Option<String>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "2000")))]
    pub ext: Option<String>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub rel_rbum_kind_id: String,

    pub scope_level: Option<RbumScopeLevelKind>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct RbumKindAttrModifyReq {
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub label: Option<String>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "2000")))]
    pub note: Option<String>,
    pub sort: Option<i64>,
    pub main_column: Option<bool>,
    pub position: Option<bool>,
    pub capacity: Option<bool>,
    pub overload: Option<bool>,
    pub hide: Option<bool>,
    pub secret: Option<bool>,
    pub show_by_conds: Option<String>,
    pub idx: Option<bool>,
    pub data_type: Option<RbumDataTypeKind>,
    pub widget_type: Option<RbumWidgetTypeKind>,
    pub widget_columns: Option<i16>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "2000")))]
    pub default_value: Option<String>,
    pub dyn_default_value: Option<String>,
    pub options: Option<String>,
    pub dyn_options: Option<String>,
    pub required: Option<bool>,
    pub min_length: Option<i32>,
    pub max_length: Option<i32>,
    pub parent_attr_name: Option<TrimString>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "2000")))]
    pub action: Option<String>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "2000")))]
    pub ext: Option<String>,

    pub scope_level: Option<RbumScopeLevelKind>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object, sea_orm::FromQueryResult))]
pub struct RbumKindAttrSummaryResp {
    pub id: String,
    pub name: String,
    pub module: String,
    pub label: String,
    pub note: String,
    pub sort: i64,
    pub main_column: bool,
    pub position: bool,
    pub capacity: bool,
    pub overload: bool,
    pub hide: bool,
    pub secret: bool,
    pub show_by_conds: String,
    pub idx: bool,
    pub data_type: RbumDataTypeKind,
    pub widget_type: RbumWidgetTypeKind,
    pub widget_columns: i16,
    pub default_value: String,
    pub dyn_default_value: String,
    pub options: String,
    pub dyn_options: String,
    pub required: bool,
    pub min_length: i32,
    pub max_length: i32,
    pub action: String,
    pub ext: String,
    pub parent_attr_name: String,
    pub rel_rbum_kind_id: String,

    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object, sea_orm::FromQueryResult))]
pub struct RbumKindAttrDetailResp {
    pub id: String,
    pub name: String,
    pub module: String,
    pub label: String,
    pub note: String,
    pub sort: i64,
    pub main_column: bool,
    pub position: bool,
    pub capacity: bool,
    pub overload: bool,
    pub hide: bool,
    pub secret: bool,
    pub show_by_conds: String,
    pub idx: bool,
    pub data_type: RbumDataTypeKind,
    pub widget_type: RbumWidgetTypeKind,
    pub widget_columns: i16,
    pub default_value: String,
    pub dyn_default_value: String,
    pub options: String,
    pub dyn_options: String,
    pub required: bool,
    pub min_length: i32,
    pub max_length: i32,
    pub action: String,
    pub ext: String,
    pub parent_attr_name: String,
    pub rel_rbum_kind_id: String,
    pub rel_rbum_kind_name: String,

    pub own_paths: String,
    pub owner: String,
    pub owner_name: Option<String>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
}
