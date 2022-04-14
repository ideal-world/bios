use std::str::FromStr;

use derive_more::Display;
use serde::{Deserialize, Serialize};
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
#[cfg(feature = "default")]
use tardis::db::sea_orm::{DbErr, QueryResult, TryGetError, TryGetable};

#[derive(Display, Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Enum))]
pub enum RbumScopeLevelKind {
    Root,
    L1,
    L2,
    L3,
}

impl RbumScopeLevelKind {
    pub fn from_int(s: u8) -> TardisResult<RbumScopeLevelKind> {
        match s {
            0 => Ok(RbumScopeLevelKind::Root),
            1 => Ok(RbumScopeLevelKind::L1),
            2 => Ok(RbumScopeLevelKind::L2),
            3 => Ok(RbumScopeLevelKind::L3),
            _ => Err(TardisError::FormatError(format!("Invalid RbumScopeLevelKind: {}", s))),
        }
    }

    pub fn to_int(&self) -> u8 {
        match self {
            RbumScopeLevelKind::Root => 0,
            RbumScopeLevelKind::L1 => 1,
            RbumScopeLevelKind::L2 => 2,
            RbumScopeLevelKind::L3 => 3,
        }
    }
}

#[cfg(feature = "default")]
impl TryGetable for RbumScopeLevelKind {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let s = u8::try_get(res, pre, col)?;
        RbumScopeLevelKind::from_int(s).map_err(|_| TryGetError::DbErr(DbErr::RecordNotFound(format!("{}:{}", pre, col))))
    }
}

#[derive(Display, Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Enum))]
pub enum RbumCertRelKind {
    Item,
    Set,
    Rel,
}

impl RbumCertRelKind {
    pub fn from_int(s: u8) -> TardisResult<RbumCertRelKind> {
        match s {
            0 => Ok(RbumCertRelKind::Item),
            1 => Ok(RbumCertRelKind::Set),
            2 => Ok(RbumCertRelKind::Rel),
            _ => Err(TardisError::FormatError(format!("Invalid RbumCertRelKind: {}", s))),
        }
    }

    pub fn to_int(&self) -> u8 {
        match self {
            RbumCertRelKind::Item => 0,
            RbumCertRelKind::Set => 1,
            RbumCertRelKind::Rel => 2,
        }
    }
}

#[cfg(feature = "default")]
impl TryGetable for RbumCertRelKind {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let s = u8::try_get(res, pre, col)?;
        RbumCertRelKind::from_int(s).map_err(|_| TryGetError::DbErr(DbErr::RecordNotFound(format!("{}:{}", pre, col))))
    }
}

#[derive(Display, Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Enum))]
pub enum RbumRelFromKind {
    Item,
    Set,
    SetCate,
}

impl RbumRelFromKind {
    pub fn from_int(s: u8) -> TardisResult<RbumRelFromKind> {
        match s {
            0 => Ok(RbumRelFromKind::Item),
            1 => Ok(RbumRelFromKind::Set),
            2 => Ok(RbumRelFromKind::SetCate),
            _ => Err(TardisError::FormatError(format!("Invalid RbumRelFromKind: {}", s))),
        }
    }

    pub fn to_int(&self) -> u8 {
        match self {
            RbumRelFromKind::Item => 0,
            RbumRelFromKind::Set => 1,
            RbumRelFromKind::SetCate => 2,
        }
    }
}

#[cfg(feature = "default")]
impl TryGetable for RbumRelFromKind {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let s = u8::try_get(res, pre, col)?;
        RbumRelFromKind::from_int(s).map_err(|_| TryGetError::DbErr(DbErr::RecordNotFound(format!("{}:{}", pre, col))))
    }
}

#[derive(Display, Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Enum))]
pub enum RbumCertStatusKind {
    Disabled,
    Enabled,
    Pending,
}

impl RbumCertStatusKind {
    pub fn from_int(s: u8) -> TardisResult<RbumCertStatusKind> {
        match s {
            0 => Ok(RbumCertStatusKind::Disabled),
            1 => Ok(RbumCertStatusKind::Enabled),
            2 => Ok(RbumCertStatusKind::Pending),
            _ => Err(TardisError::FormatError(format!("Invalid RbumCertStatusKind: {}", s))),
        }
    }

    pub fn to_int(&self) -> u8 {
        match self {
            RbumCertStatusKind::Disabled => 0,
            RbumCertStatusKind::Enabled => 1,
            RbumCertStatusKind::Pending => 2,
        }
    }
}

#[cfg(feature = "default")]
impl TryGetable for RbumCertStatusKind {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let s = u8::try_get(res, pre, col)?;
        RbumCertStatusKind::from_int(s).map_err(|_| TryGetError::DbErr(DbErr::RecordNotFound(format!("{}:{}", pre, col))))
    }
}

#[derive(Display, Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Enum))]
pub enum RbumRelEnvKind {
    DatetimeRange,
    TimeRange,
    Ips,
}

impl RbumRelEnvKind {
    pub fn from_int(s: u8) -> TardisResult<RbumRelEnvKind> {
        match s {
            0 => Ok(RbumRelEnvKind::DatetimeRange),
            1 => Ok(RbumRelEnvKind::TimeRange),
            2 => Ok(RbumRelEnvKind::Ips),
            _ => Err(TardisError::FormatError(format!("Invalid RbumRelEnvKind: {}", s))),
        }
    }

    pub fn to_int(&self) -> u8 {
        match self {
            RbumRelEnvKind::DatetimeRange => 0,
            RbumRelEnvKind::TimeRange => 1,
            RbumRelEnvKind::Ips => 2,
        }
    }
}

#[cfg(feature = "default")]
impl TryGetable for RbumRelEnvKind {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let s = u8::try_get(res, pre, col)?;
        RbumRelEnvKind::from_int(s).map_err(|_| TryGetError::DbErr(DbErr::RecordNotFound(format!("{}:{}", pre, col))))
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
pub enum RbumWidgetTypeKind {
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
impl TryGetable for RbumWidgetTypeKind {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let s = String::try_get(res, pre, col)?;
        RbumWidgetTypeKind::from_str(&s).map_err(|_| TryGetError::DbErr(DbErr::RecordNotFound(format!("{}:{}", pre, col))))
    }
}
