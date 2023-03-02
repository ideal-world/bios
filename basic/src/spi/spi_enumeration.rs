use serde::{Deserialize, Serialize};
#[cfg(feature = "default")]
use tardis::derive_more::Display;
#[cfg(feature = "default")]
use tardis::web::poem_openapi;

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "default", derive(poem_openapi::Enum))]
pub enum SpiQueryOpKind {
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
    #[oai(rename = "in")]
    In,
}

impl SpiQueryOpKind {
    pub fn to_sql(&self) -> String {
        match self {
            SpiQueryOpKind::Eq => "=".to_string(),
            SpiQueryOpKind::Ne => "!=".to_string(),
            SpiQueryOpKind::Gt => ">".to_string(),
            SpiQueryOpKind::Ge => ">=".to_string(),
            SpiQueryOpKind::Lt => "<".to_string(),
            SpiQueryOpKind::Le => "<=".to_string(),
            SpiQueryOpKind::Like => "LIKE".to_string(),
            SpiQueryOpKind::In => "IN".to_string(),
        }
    }
}
