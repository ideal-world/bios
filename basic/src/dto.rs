use serde::{Deserialize, Serialize};
use tardis::serde_json::Value;

use crate::basic_enumeration::BasicQueryOpKind;
#[cfg(feature = "default")]
use tardis::web::poem_openapi;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct BasicQueryCondInfo {
    #[oai(validator(min_length = "2"))]
    pub field: String,
    pub op: BasicQueryOpKind,
    pub value: Value,
}
