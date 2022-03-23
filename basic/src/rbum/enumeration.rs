use std::str::FromStr;

use derive_more::Display;
use serde::{Deserialize, Serialize};
#[cfg(feature = "default")]
use tardis::db::sea_orm::{DbErr, QueryResult, TryGetError, TryGetable};

#[derive(Display, Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Enum, sea_orm::strum::EnumString))]
pub enum RbumCertStatusKind {
    Pending,
    Enabled,
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
    DatetimeRange,
    TimeRange,
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
    String,
    Number,
    Boolean,
    Date,
    DateTime,
    Json,
    Strings,
    Numbers,
    Booleans,
    Dates,
    DateTimes,
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
    Input,
    InputTxt,
    InputNum,
    Textarea,
    Number,
    Date,
    DateTime,
    Upload,
    Radio,
    Checkbox,
    Switch,
    Select,
}

#[cfg(feature = "default")]
impl TryGetable for RbumWidgetKind {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let s = String::try_get(res, pre, col)?;
        RbumWidgetKind::from_str(&s).map_err(|_| TryGetError::DbErr(DbErr::RecordNotFound(format!("{}:{}", pre, col))))
    }
}
