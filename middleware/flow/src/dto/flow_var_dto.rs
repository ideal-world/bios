use bios_basic::rbum::rbum_enumeration::{RbumDataTypeKind, RbumWidgetTypeKind};
use serde::{Deserialize, Serialize};
use tardis::{serde_json::Value, web::poem_openapi};

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowVarSimpleInfo {
    #[oai(validator(min_length = "2", max_length = "200"))]
    pub name: String,
    pub data_type: RbumDataTypeKind,
    pub default_value: Value,
    pub required: bool,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowVarInfo {
    #[oai(validator(min_length = "2", max_length = "200"))]
    pub name: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub label: String,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub note: Option<String>,
    pub sort: Option<i64>,
    pub hide: Option<bool>,
    pub secret: Option<bool>,
    pub show_by_conds: Option<String>,
    pub data_type: RbumDataTypeKind,
    pub widget_type: RbumWidgetTypeKind,
    pub widget_columns: Option<i16>,
    pub default_value: Option<Value>,
    pub dyn_default_value: Option<Value>,
    pub options: Option<String>,
    pub dyn_options: Option<String>,
    pub required: Option<bool>,
    pub min_length: Option<i32>,
    pub max_length: Option<i32>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub action: Option<String>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub ext: Option<String>,
    pub parent_attr_name: Option<String>,
}
