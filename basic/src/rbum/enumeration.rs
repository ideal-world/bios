use std::str::FromStr;

use derive_more::Display;
use sea_orm::strum::EnumString;
use serde::{Deserialize, Serialize};
use tardis::db::sea_orm::{DbErr, QueryResult, TryGetError, TryGetable};
use tardis::web::poem_openapi::Enum;

#[derive(Enum, Display, EnumString, Clone, Debug, PartialEq, Deserialize, Serialize)]
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

impl TryGetable for RbumScopeKind {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let s = String::try_get(res, pre, col)?;
        RbumScopeKind::from_str(&s).map_err(|_| TryGetError::DbErr(DbErr::RecordNotFound(format!("{}:{}", pre, col))))
    }
}

#[derive(Enum, Display, EnumString, Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum RbumCertStatusKind {
    #[display(fmt = "PENDING")]
    Pending,
    #[display(fmt = "ENABLED")]
    Enabled,
    #[display(fmt = "DISABLED")]
    Disabled,
}

impl TryGetable for RbumCertStatusKind {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let s = String::try_get(res, pre, col)?;
        RbumCertStatusKind::from_str(&s).map_err(|_| TryGetError::DbErr(DbErr::RecordNotFound(format!("{}:{}", pre, col))))
    }
}

#[derive(Enum, Display, EnumString, Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum RbumRelEnvKind {
    #[display(fmt = "DT_DATETIME_RANGE")]
    DatetimeRange,
    #[display(fmt = "DT_TIME_RANGE")]
    TimeRange,
    #[display(fmt = "SPACE_IPS")]
    Ips,
}

impl TryGetable for RbumRelEnvKind {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let s = String::try_get(res, pre, col)?;
        RbumRelEnvKind::from_str(&s).map_err(|_| TryGetError::DbErr(DbErr::RecordNotFound(format!("{}:{}", pre, col))))
    }
}
