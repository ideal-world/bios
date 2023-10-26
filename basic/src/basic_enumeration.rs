use serde::{Deserialize, Serialize};
use tardis::derive_more::Display;
#[cfg(feature = "default")]
use tardis::web::poem_openapi;

#[derive(Display, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Tags))]
pub enum ApiTag {
    #[oai(rename = "Common Console")]
    Common,
    #[oai(rename = "Tenant Console")]
    Tenant,
    #[oai(rename = "App Console")]
    App,
    #[oai(rename = "System Console")]
    System,
    #[oai(rename = "Passport Console")]
    Passport,
    #[oai(rename = "Interface Console")]
    Interface,
}

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "default", derive(poem_openapi::Enum))]
pub enum BasicQueryOpKind {
    #[oai(rename = "=")]
    Eq,
    #[oai(rename = "!=")]
    Ne,
    #[oai(rename = ">")]
    Gt,
    #[oai(rename = ">=")]
    Ge,
    #[oai(rename = "<")]
    Lt,
    #[oai(rename = "<=")]
    Le,
    #[oai(rename = "like")]
    Like,
    #[oai(rename = "not_like")]
    NotLike,
    #[oai(rename = "in")]
    In,
    #[oai(rename = "not_in")]
    NotIn,
    #[oai(rename = "is_null")]
    IsNull,
    #[oai(rename = "is_not_null")]
    IsNotNull,
    #[oai(rename = "is_not_null_or_empty")]
    IsNullOrEmpty,
}

impl BasicQueryOpKind {
    pub fn to_sql(&self) -> String {
        match self {
            BasicQueryOpKind::Eq => "=".to_string(),
            BasicQueryOpKind::Ne => "!=".to_string(),
            BasicQueryOpKind::Gt => ">".to_string(),
            BasicQueryOpKind::Ge => ">=".to_string(),
            BasicQueryOpKind::Lt => "<".to_string(),
            BasicQueryOpKind::Le => "<=".to_string(),
            BasicQueryOpKind::Like => "LIKE".to_string(),
            BasicQueryOpKind::NotLike => "NOT LIKE".to_string(),
            BasicQueryOpKind::In => "IN".to_string(),
            BasicQueryOpKind::NotIn => "NOT IN".to_string(),
            BasicQueryOpKind::IsNull => "IS NULL".to_string(),
            BasicQueryOpKind::IsNotNull => "IS NOT NULL".to_string(),
            BasicQueryOpKind::IsNullOrEmpty => "IS NULL".to_string(),
        }
    }
}
