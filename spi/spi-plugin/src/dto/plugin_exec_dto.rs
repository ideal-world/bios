use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tardis::web::poem_openapi;

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct PluginExecReq {
    pub header: Option<HashMap<String, String>>,
    pub body: Option<HashMap<String, String>>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct PluginExecResp {
    pub code: u16,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}
