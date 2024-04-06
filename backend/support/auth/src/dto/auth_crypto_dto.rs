use std::collections::HashMap;

use serde::{Deserialize, Serialize};
#[cfg(feature = "web-server")]
use tardis::web::poem_openapi;

#[cfg_attr(feature = "web-server", derive(poem_openapi::Object))]
#[derive(Serialize, Deserialize, Debug)]
pub struct AuthEncryptReq {
    pub headers: HashMap<String, String>,
    pub body: String,
}

#[cfg_attr(feature = "web-server", derive(poem_openapi::Object))]
#[derive(Serialize, Deserialize, Debug)]
pub struct AuthEncryptResp {
    pub headers: HashMap<String, String>,
    pub body: String,
}
