use std::time::Duration;

use serde::{Deserialize, Serialize};
use spacegate_shell::{hyper::HeaderMap, kernel::extension::ExtensionPack};
use tardis::serde_json::{json, Value};

use super::cert_info::RoleInfo;

#[derive(Clone)]
pub struct AuditLogParam {
    pub request_path: String,
    pub request_method: String,
    pub request_headers: HeaderMap,
    pub request_scheme: String,
    pub request_ip: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LogParamContent {
    pub op: String,
    pub name: String,
    pub user_id: Option<String>,
    pub own_paths: Option<String>,
    pub role: Vec<RoleInfo>,
    pub ip: String,
    pub path: String,
    pub scheme: String,
    pub token: Option<String>,
    pub server_timing: Option<Duration>,
    pub resp_status: String,
    //Indicates whether the business operation was successful.
    pub success: bool,
}

impl ExtensionPack for LogParamContent {}
