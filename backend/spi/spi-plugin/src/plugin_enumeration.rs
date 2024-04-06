use std::str::FromStr;

use serde::{Deserialize, Serialize};
use tardis::db::sea_orm;
use tardis::db::sea_orm::{DbErr, QueryResult, TryGetError, TryGetable};
use tardis::derive_more::Display;
use tardis::web::poem_openapi;

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, strum::EnumString)]
pub enum PluginAppBindRelKind {
    PluginAppBindKind,
}

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "default", derive(poem_openapi::Enum, strum::EnumString))]
pub enum PluginApiMethodKind {
    GET,
    PUT,
    POST,
    DELETE,
    PATCH,
}

#[cfg(feature = "default")]
impl TryGetable for PluginApiMethodKind {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let s = String::try_get(res, pre, col)?;
        PluginApiMethodKind::from_str(&s).map_err(|_| TryGetError::DbErr(DbErr::RecordNotFound(format!("{pre}:{col}"))))
    }

    fn try_get_by<I: sea_orm::ColIdx>(_res: &QueryResult, _index: I) -> Result<Self, TryGetError> {
        panic!("not implemented")
    }
}
