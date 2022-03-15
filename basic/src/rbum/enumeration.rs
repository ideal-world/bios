use std::str::FromStr;

use derive_more::Display;
use serde::{Deserialize, Serialize};
#[cfg(feature = "default")]
use tardis::db::sea_orm::{DbErr, QueryResult, TryGetable, TryGetError};

#[derive(Display, Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Enum, sea_orm::strum::EnumString))]
pub enum RbumScopeKind {
    #[display(fmt = "TAG")]
    Tag,
    #[display(fmt = "APP")]
    App,
    #[display(fmt = "TENANT")]
    Tenant,
    #[display(fmt = "GLOBAL")]
    Global,
}

#[cfg(feature = "default")]
impl TryGetable for RbumScopeKind {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let s = String::try_get(res, pre, col)?;
        RbumScopeKind::from_str(&s).map_err(|_| TryGetError::DbErr(DbErr::RecordNotFound(format!("{}:{}", pre, col))))
    }
}

#[derive(Display, Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Enum, sea_orm::strum::EnumString))]
pub enum RbumCertStatusKind {
    #[display(fmt = "PENDING")]
    Pending,
    #[display(fmt = "ENABLED")]
    Enabled,
    #[display(fmt = "DISABLED")]
    Disabled,
}

#[cfg(feature = "default")]
impl TryGetable for RbumCertStatusKind {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let s = String::try_get(res, pre, col)?;
        RbumCertStatusKind::from_str(&s).map_err(|_| TryGetError::DbErr(DbErr::RecordNotFound(format!("{}:{}", pre, col))))
    }
}

#[derive(Display, Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Enum, sea_orm::strum::EnumString))]
pub enum RbumRelEnvKind {
    #[display(fmt = "DT_DATETIME_RANGE")]
    DatetimeRange,
    #[display(fmt = "DT_TIME_RANGE")]
    TimeRange,
    #[display(fmt = "SPACE_IPS")]
    Ips,
}

#[cfg(feature = "default")]
impl TryGetable for RbumRelEnvKind {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let s = String::try_get(res, pre, col)?;
        RbumRelEnvKind::from_str(&s).map_err(|_| TryGetError::DbErr(DbErr::RecordNotFound(format!("{}:{}", pre, col))))
    }
}

#[derive(Display, Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Enum, sea_orm::strum::EnumString))]
pub enum RbumDataTypeKind {
    #[display(fmt = "STRING")]
    String,
    #[display(fmt = "NUMBER")]
    Number,
    #[display(fmt = "BOOLEAN")]
    Boolean,
    #[display(fmt = "DATE")]
    Date,
    #[display(fmt = "DATETIME")]
    DateTime,
    #[display(fmt = "JSON")]
    Json,
    #[display(fmt = "STRINGS")]
    Strings,
    #[display(fmt = "NUMBERS")]
    Numbers,
    #[display(fmt = "BOOLEANS")]
    Booleans,
    #[display(fmt = "DATES")]
    Dates,
    #[display(fmt = "DATETIMES")]
    DateTimes,
    #[display(fmt = "ARRAY")]
    Array,
}

#[cfg(feature = "default")]
impl TryGetable for RbumDataTypeKind {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let s = String::try_get(res, pre, col)?;
        RbumDataTypeKind::from_str(&s).map_err(|_| TryGetError::DbErr(DbErr::RecordNotFound(format!("{}:{}", pre, col))))
    }
}

#[derive(Display, Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Enum, sea_orm::strum::EnumString))]
pub enum RbumWidgetKind {
    #[display(fmt = "INPUT")]
    Input,
    #[display(fmt = "INPUT_TXT")]
    InputTxt,
    #[display(fmt = "INPUT_NUM")]
    InputNum,
    #[display(fmt = "TEXTAREA")]
    Textarea,
    #[display(fmt = "NUMBER")]
    Number,
    #[display(fmt = "DATE")]
    Date,
    #[display(fmt = "DATETIME")]
    DateTime,
    #[display(fmt = "UPLOAD")]
    Upload,
    #[display(fmt = "RADIO")]
    Radio,
    #[display(fmt = "CHECKBOX")]
    Checkbox,
    #[display(fmt = "SWITCH")]
    Switch,
    #[display(fmt = "SELECT")]
    Select,
}

#[cfg(feature = "default")]
impl TryGetable for RbumWidgetKind {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let s = String::try_get(res, pre, col)?;
        RbumWidgetKind::from_str(&s).map_err(|_| TryGetError::DbErr(DbErr::RecordNotFound(format!("{}:{}", pre, col))))
    }
}
