use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct AuthConfig {
    pub head_key_token: String,
    pub head_key_app: String,
    pub head_key_protocol: String,
    pub head_key_context: String,

    pub cache_key_token_info: String,
    pub cache_key_account_info: String,

    pub cache_key_res_info: String,
    pub cache_key_res_changed_info: String,
    pub cache_key_res_changed_timer_sec: u32,

    pub cors_allow_origin: String,
    pub cors_allow_methods: String,
    pub cors_allow_headers: String,
}

impl Default for AuthConfig {
    fn default() -> Self {
        AuthConfig {
            head_key_token: "Bios-Token".to_string(),
            head_key_app: "Bios-app".to_string(),
            head_key_protocol: "Bios-Proto".to_string(),
            head_key_context: "Tardis-Context".to_string(),
            cache_key_token_info: "iam:cache:token:info:".to_string(),
            cache_key_account_info: "iam:cache:account:info:".to_string(),
            cache_key_res_info: "iam:res:info".to_string(),
            cache_key_res_changed_info: "iam:res:changed:info:".to_string(),
            cache_key_res_changed_timer_sec: 30,
            cors_allow_origin: "*".to_string(),
            cors_allow_methods: "*".to_string(),
            cors_allow_headers: "*".to_string(),
        }
    }
}
