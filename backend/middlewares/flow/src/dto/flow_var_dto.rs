use serde::{Deserialize, Serialize};
use std::{collections::HashMap, ops::Not, str::FromStr};
use strum::Display;
use tardis::{
    db::sea_orm::{self, DbErr, QueryResult, TryGetError, TryGetable},
    serde_json::Value,
    web::poem_openapi,
};

use super::flow_cond_dto::BasicQueryCondInfo;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, poem_openapi::Object, sea_orm::FromJsonQueryResult)]
pub struct FlowVarSimpleInfo {
    #[oai(validator(min_length = "2", max_length = "200"))]
    pub name: String,
    pub data_type: RbumDataTypeKind,
    pub default_value: Value,
    pub required: bool,
}

/// 变量展示/隐藏状态
#[derive(Display, Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, strum::EnumString)]
pub enum FlowVarVisibilityKind {
    /// 展示
    #[default]
    Show,
    /// 隐藏
    Hide,
}

impl Not for FlowVarVisibilityKind {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            FlowVarVisibilityKind::Show => FlowVarVisibilityKind::Hide,
            FlowVarVisibilityKind::Hide => FlowVarVisibilityKind::Show,
        }
    }
}

/// 变量展示/隐藏条件规则
///
/// 满足 ``conds`` 条件时，应用 ``visibility`` 指定的展示/隐藏状态
#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, poem_openapi::Object, sea_orm::FromJsonQueryResult)]
pub struct FlowVarVisibilityCondRule {
    /// 条件列表，外层OR、内层AND
    pub conds: Vec<Vec<BasicQueryCondInfo>>,
    /// 满足条件时的展示/隐藏状态
    pub visibility: FlowVarVisibilityKind,
}

/// 变量展示/隐藏配置
///
/// 描述变量的默认展示或隐藏状态，以及在特定条件下的展示或隐藏状态
#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, poem_openapi::Object, sea_orm::FromJsonQueryResult)]
pub struct FlowVarVisibilityInfo {
    /// 默认展示/隐藏状态
    pub default_visibility: Option<FlowVarVisibilityKind>,
    /// 条件规则，满足条件时应用对应的展示/隐藏状态
    pub cond_rule: Option<FlowVarVisibilityCondRule>,
}

impl FlowVarVisibilityInfo {
    /// 根据字段值解析变量的展示/隐藏状态
    /// - 存在条件规则时：命中条件则使用规则状态，未命中则使用规则状态的反向状态
    /// - 不存在条件规则时：使用默认状态
    pub fn resolve(&self, check_vars: &HashMap<String, Value>) -> FlowVarVisibilityKind {
        if let Some(cond_rule) = &self.cond_rule {
            if BasicQueryCondInfo::check_or_and_conds(&cond_rule.conds, check_vars).unwrap_or(false) {
                return cond_rule.visibility.clone();
            } else {
                return !cond_rule.visibility.clone();
            }
        }
        self.default_visibility.clone().unwrap_or(FlowVarVisibilityKind::Show)
    }
}

impl FlowVarInfo {
    /// 根据 visibility 配置过滤不应展示的变量
    pub fn filter_by_visibility(vars: Vec<Self>, check_vars: &HashMap<String, Value>) -> Vec<Self> {
        vars.into_iter()
            .filter(|var| match &var.visibility {
                Some(visibility) => visibility.resolve(check_vars) == FlowVarVisibilityKind::Show,
                None => true,
            })
            .collect()
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, poem_openapi::Object, sea_orm::FromJsonQueryResult)]
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
    pub widget_type: FlowidgetTypeKind,
    pub widget_columns: Option<i16>,
    pub default_value: Option<DefaultValue>,
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
    pub is_edit: Option<bool>,
    pub visibility: Option<FlowVarVisibilityInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, poem_openapi::Object)]
pub struct DefaultValue {
    pub value_type: DefaultValueType,
    pub value: Value,
    pub ext: Value,
    pub value_name: Option<String>,
}

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, strum::EnumString)]
pub enum DefaultValueType {
    Custom,
    // Associated attribute
    AssociatedAttr,
    // Auto fill attribute
    AutoFill,
}

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, strum::EnumString)]
pub enum FillType {
    Person,
    Time,
}

// In order to adapt to the JAVA program, the corresponding kind in rbum is changed to uppercase format (only here for the time being, the subsequent can be placed in the public module)
// 为了和JAVA程序适配，此处把rbum中对应的kind改为大写格式（暂时只有此处需要，后续可以放置到公共模块）
#[derive(Display, Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, strum::EnumString)]
pub enum RbumDataTypeKind {
    #[default]
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

#[derive(Display, Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, strum::EnumString)]
pub enum FlowidgetTypeKind {
    #[default]
    INPUT,
    INPUTTXT,
    INPUTNUM,
    TEXTAREA,
    NUMBER,
    DATE,
    DATETIME,
    TIME,
    UPLOAD,
    RADIO,
    BUTTON,
    CHECKBOX,
    SWITCH,
    SELECT,
    MULTISELECT,
    LINK,
    CODEEDITOR,
    DOC,
    CONTAINER, // Display group subtitles, datatype = String, value is empty
    CONTROL,   // Json fields : all parent_attr_name = current attribute, datatype = Json
    GROUP,     // Sub fields : all parent_attr_name = current attribute, datatype = Array, The value of the json array is stored to the current field.
}

impl TryGetable for FlowidgetTypeKind {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let s = String::try_get(res, pre, col)?;
        FlowidgetTypeKind::from_str(&s).map_err(|_| TryGetError::DbErr(DbErr::RecordNotFound(format!("{pre}:{col}"))))
    }

    fn try_get_by<I: sea_orm::ColIdx>(_res: &QueryResult, _index: I) -> Result<Self, TryGetError> {
        panic!("not implemented")
    }
}
