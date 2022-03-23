use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::chrono::{DateTime, Utc};
use tardis::web::poem_openapi::Object;

use crate::rbum::enumeration::{RbumDataTypeKind, RbumWidgetKind};

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumKindAttrAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub label: String,
    pub data_type_kind: RbumDataTypeKind,
    pub widget_type: RbumWidgetKind,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub note: Option<String>,
    pub sort: Option<i32>,
    pub main_column: Option<bool>,
    pub position: Option<bool>,
    pub capacity: Option<bool>,
    pub overload: Option<bool>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub default_value: Option<String>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub options: Option<String>,
    pub required: Option<bool>,
    pub min_length: Option<i8>,
    pub max_length: Option<i8>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub action: Option<String>,

    pub scope_level: i32,

    #[oai(validator(min_length = "2", max_length = "255"))]
    pub rel_rbum_kind_id: String,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumKindAttrModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub label: Option<String>,
    pub data_type_kind: Option<RbumDataTypeKind>,
    pub widget_type: Option<RbumWidgetKind>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub note: Option<String>,
    pub sort: Option<i32>,
    pub main_column: Option<bool>,
    pub position: Option<bool>,
    pub capacity: Option<bool>,
    pub overload: Option<bool>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub default_value: Option<String>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub options: Option<String>,
    pub required: Option<bool>,
    pub min_length: Option<i8>,
    pub max_length: Option<i8>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub action: Option<String>,

    pub scope_level: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object, tardis::db::sea_orm::FromQueryResult))]
pub struct RbumKindAttrSummaryResp {
    pub id: String,
    pub name: String,
    pub label: String,
    pub sort: i32,

    pub main_column: bool,
    pub position: bool,
    pub capacity: bool,
    pub overload: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object, tardis::db::sea_orm::FromQueryResult))]
pub struct RbumKindAttrDetailResp {
    pub id: String,
    pub name: String,
    pub label: String,
    pub note: String,
    pub sort: i32,
    pub main_column: bool,
    pub position: bool,
    pub capacity: bool,
    pub overload: bool,
    pub data_type_kind: String,
    pub widget_type: String,
    pub default_value: String,
    pub options: String,
    pub required: bool,
    pub min_length: i8,
    pub max_length: i8,
    pub action: String,
    pub rel_rbum_kind_id: String,
    pub rel_rbum_kind_name: String,

    pub scope_ids: String,
    pub updater_id: String,
    pub updater_name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: i32,
}
