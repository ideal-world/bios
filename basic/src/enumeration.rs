use std::str::FromStr;

use derive_more::Display;
use sea_orm::strum::EnumString;
use serde::{Deserialize, Serialize};
use tardis::db::sea_orm::{DbErr, QueryResult, TryGetError, TryGetable};
use tardis::web::poem_openapi::Enum;

#[derive(Enum, Display, EnumString, Debug, Deserialize, Serialize)]
pub enum DataTypeKind {
    #[display(fmt = "STRING")]
    STRING,
    #[display(fmt = "NUMBER")]
    NUMBER,
    #[display(fmt = "BOOLEAN")]
    BOOLEAN,
    #[display(fmt = "DATE")]
    DATE,
    #[display(fmt = "DATETIME")]
    DATETIME,
    #[display(fmt = "JSON")]
    JSON,
    #[display(fmt = "STRINGS")]
    STRINGS,
    #[display(fmt = "NUMBERS")]
    NUMBERS,
    #[display(fmt = "BOOLEANS")]
    BOOLEANS,
    #[display(fmt = "DATES")]
    DATES,
    #[display(fmt = "DATETIMES")]
    DATETIMES,
    #[display(fmt = "ARRAY")]
    ARRAY,
}

impl TryGetable for DataTypeKind {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let s = String::try_get(res, pre, col)?;
        DataTypeKind::from_str(&s).map_err(|_| TryGetError::DbErr(DbErr::RecordNotFound(format!("{}:{}", pre, col))))
    }
}
