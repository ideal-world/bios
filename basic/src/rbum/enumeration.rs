use std::str::FromStr;

use derive_more::Display;
use sea_orm::strum::EnumString;
use serde::{Deserialize, Serialize};
use tardis::db::sea_orm::{DbErr, QueryResult, TryGetError, TryGetable};
use tardis::web::poem_openapi::Enum;

#[derive(Enum, Display, EnumString, Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum RbumScopeKind {
    /// 标签级
    /// 表明只这个标签可用
    #[display(fmt = "TAG")]
    TAG,
    /// 应用级
    /// 表明在应用内共享
    #[display(fmt = "APP")]
    APP,
    /// 租户级
    /// 表明在租户内共享
    #[display(fmt = "TENANT")]
    TENANT,
    /// 系统级
    /// 表明整个系统共享
    #[display(fmt = "GLOBAL")]
    GLOBAL,
}

impl TryGetable for RbumScopeKind {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let s = String::try_get(res, pre, col)?;
        RbumScopeKind::from_str(&s).map_err(|_| TryGetError::DbErr(DbErr::RecordNotFound(format!("{}:{}", pre, col))))
    }
}
