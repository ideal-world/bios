use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::web::poem_openapi::Object;

use bios_basic::rbum::rbum_enumeration::{RbumDataTypeKind, RbumWidgetTypeKind};

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamKindAttrAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub label: String,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub note: Option<String>,
    pub sort: Option<u32>,
    pub main_column: Option<bool>,
    pub position: Option<bool>,
    pub capacity: Option<bool>,
    pub overload: Option<bool>,
    pub idx: Option<bool>,
    pub data_type: RbumDataTypeKind,
    pub widget_type: RbumWidgetTypeKind,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub default_value: Option<String>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub options: Option<String>,
    pub required: Option<bool>,
    pub min_length: Option<u32>,
    pub max_length: Option<u32>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub action: Option<String>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub ext: Option<String>,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamItemAttrAddReq {
    #[oai(validator(min_length = "1", max_length = "2000"))]
    pub value: String,
}
