use std::str::FromStr;

use derive_more::Display;
use serde::{Deserialize, Serialize};
use tardis::db::sea_orm::{DbErr, QueryResult, TryGetable, TryGetError};
use tardis::db::sea_orm::strum::EnumString;
use tardis::web::poem_openapi::Enum;
use tardis::web::poem_openapi::Tags;

#[derive(Tags, Display, EnumString, Debug)]
pub enum Components {
    /// IAM Component
    #[oai(rename = "IAM")]
    #[display(fmt = "iam")]
    Iam,
}

#[derive(Enum, Display, EnumString, Debug, Deserialize, Serialize)]
pub enum DataTypeKind {
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

impl TryGetable for DataTypeKind {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let s = String::try_get(res, pre, col)?;
        DataTypeKind::from_str(&s).map_err(|_| TryGetError::DbErr(DbErr::RecordNotFound(format!("{}:{}", pre, col))))
    }
}
