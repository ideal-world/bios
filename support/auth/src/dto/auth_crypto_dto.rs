use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tardis::web::poem_openapi;

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct AuthEncryptReq {
    pub headers: HashMap<String, String>,
    pub body: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct AuthEncryptResp {
    pub headers: HashMap<String, String>,
    pub body: String,
}
