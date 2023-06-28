use serde::{Deserialize, Serialize};
use tardis::{serde_json::Value, web::poem_openapi, db::sea_orm::{self, strum::Display, TryGetable, QueryResult, TryGetError, DbErr}};
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowVarSimpleInfo {
    #[oai(validator(min_length = "2", max_length = "200"))]
    pub name: String,
    pub data_type: RbumDataTypeKind,
    pub default_value: Value,
    pub required: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, poem_openapi::Object)]
pub struct FlowVarInfo {
    #[oai(validator(min_length = "2", max_length = "200"))]
    pub name: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub label: String,
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

// In order to adapt to the JAVA program, the corresponding kind in rbum is changed to uppercase format (only here for the time being, the subsequent can be placed in the public module)
// 为了和JAVA程序适配，此处把rbum中对应的kind改为大写格式（暂时只有此处需要，后续可以放置到公共模块）
#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[derive(poem_openapi::Enum, sea_orm::strum::EnumString)]
pub enum RbumDataTypeKind {
    STRING,
    NUMBER,
    BOOLEAN,
    DATE,
    DATETIME,
    JSON,
    STRINGS,
    NUMBERS,
    BOOLEANS,
    DATES,
    DATETIMES,
    ARRAY,
    LABEL,
}

impl TryGetable for RbumDataTypeKind {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let s = String::try_get(res, pre, col)?;
        RbumDataTypeKind::from_str(&s).map_err(|_| TryGetError::DbErr(DbErr::RecordNotFound(format!("{pre}:{col}"))))
    }

    fn try_get_by<I: sea_orm::ColIdx>(_res: &QueryResult, _index: I) -> Result<Self, TryGetError> {
        panic!("not implemented")
    }
}

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[derive(poem_openapi::Enum, sea_orm::strum::EnumString)]
pub enum RbumWidgetTypeKind {
    INPUT,
    INPUTTXT,
    INPUTNUM,
    TEXTAREA,
    NUMBER,
    DATE,
    DATETIME,
    UPLOAD,
    RADIO,
    BUTTON,
    CHECKBOX,
    SWITCH,
    SELECT,
    MULTISELECT,
    LINK,
    CODEEDITOR,
    CONTAINER, // Display group subtitles, datatype = String, value is empty
    CONTROL,   // Json fields : all parent_attr_name = current attribute, datatype = Json
    GROUP,     // Sub fields : all parent_attr_name = current attribute, datatype = Array, The value of the json array is stored to the current field.
}

impl TryGetable for RbumWidgetTypeKind {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let s = String::try_get(res, pre, col)?;
        RbumWidgetTypeKind::from_str(&s).map_err(|_| TryGetError::DbErr(DbErr::RecordNotFound(format!("{pre}:{col}"))))
    }

    fn try_get_by<I: sea_orm::ColIdx>(_res: &QueryResult, _index: I) -> Result<Self, TryGetError> {
        panic!("not implemented")
    }
}
