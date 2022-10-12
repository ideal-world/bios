use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tardis::web::poem_openapi;

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct RelDbQueryReq {
    #[oai(validator(min_length = "2"))]
    pub sql: String,
    pub params: HashMap<String, String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct RelDbExecuteReq {
    #[oai(validator(min_length = "2"))]
    pub sql: String,
    pub params: HashMap<String, String>,
}
