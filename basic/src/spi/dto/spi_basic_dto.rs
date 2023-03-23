use serde::{Deserialize, Serialize};
use tardis::{web::poem_openapi, serde_json::Value};

use crate::spi::spi_enumeration::SpiQueryOpKind;

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct SpiQueryCondReq {
    #[oai(validator(min_length = "2"))]
    pub field: String,
    pub op: SpiQueryOpKind,
    pub value: Value,
}
