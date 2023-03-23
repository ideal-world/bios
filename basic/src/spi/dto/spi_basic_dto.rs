use serde::{Deserialize, Serialize};
use tardis::{serde_json::Value, web::poem_openapi};

use crate::spi::spi_enumeration::SpiQueryOpKind;

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct SpiQueryCondReq {
    #[oai(validator(min_length = "2"))]
    pub field: String,
    pub op: SpiQueryOpKind,
    pub value: Value,
}
