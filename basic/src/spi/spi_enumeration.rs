use serde::{Deserialize, Serialize};
#[cfg(feature = "default")]
use tardis::derive_more::Display;
#[cfg(feature = "default")]
use tardis::web::poem_openapi;

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "default", derive(poem_openapi::Enum))]
pub enum SpiQueryOpKind {
    Eq,
    Ne,
    Gt,
    Ge,
    Lt,
    Le,
    Like,
}
