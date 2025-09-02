use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tardis::{serde_json::Value, web::poem_openapi};

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct PluginExecReq {
    // 具体绑定的 relId
    pub rel_id: Option<String>,
    pub header: Option<HashMap<String, String>>,
    pub query: Option<HashMap<String, String>>,
    pub body: Option<Value>,
    #[serde(default)]
    #[oai(default)]
    pub percent_encode: Option<bool>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct PluginExecResp {
    pub code: u16,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}
