use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct AuthConfig {
    pub head_key_token: String,
    pub head_key_ak_authorization: String,
    pub head_key_date_flag: String,
    pub head_key_app: String,
    pub head_key_protocol: String,
    pub head_key_context: String,
    pub head_key_crypto: String,
    pub head_date_format: String,
    pub head_date_interval_millsec: u32,

    pub cache_key_token_info: String,
    pub cache_key_account_info: String,
    pub cache_key_aksk_info: String,
    pub cache_key_crypto_key: String,
    pub cache_key_double_auth_info: String,

    pub cache_key_res_info: String,
    pub cache_key_res_changed_info: String,
    pub cache_key_res_changed_timer_sec: u32,

    pub cors_allow_origin: String,
    pub cors_allow_methods: String,
    pub cors_allow_headers: String,

    pub strict_security_mode: bool,
    pub double_auth_exp_sec: u32,
    pub extra_api: ApiConfig,
}

impl Default for AuthConfig {
    fn default() -> Self {
        AuthConfig {
            head_key_token: "Bios-Token".to_string(),
            head_key_ak_authorization: "Bios-Authorization".to_string(),
            /// Special: need use UTC Time
            head_key_date_flag: "Bios-Date".to_string(),
            head_key_app: "Bios-App".to_string(),
            head_key_protocol: "Bios-Proto".to_string(),
            head_key_context: "Tardis-Context".to_string(),
            head_key_crypto: "Bios-Crypto".to_string(),
            head_date_format: "%a, %d %b %Y %T GMT".to_string(),
            head_date_interval_millsec: 10000,
            cache_key_token_info: "iam:cache:token:info:".to_string(),
            cache_key_account_info: "iam:cache:account:info:".to_string(),
            cache_key_aksk_info: "iam:cache:aksk:info:".to_string(),
            cache_key_crypto_key: "auth:crypto:key:".to_string(),
            // ..:<account_id>
            cache_key_double_auth_info: "iam:cache:double_auth:info:".to_string(),
            cache_key_res_info: "iam:res:info".to_string(),
            cache_key_res_changed_info: "iam:res:changed:info:".to_string(),
            cache_key_res_changed_timer_sec: 30,
            cors_allow_origin: "*".to_string(),
            cors_allow_methods: "*".to_string(),
            cors_allow_headers: "*".to_string(),
            strict_security_mode: false,
            double_auth_exp_sec: 300,
            extra_api: ApiConfig::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct ApiConfig {
    pub login_req_method: String,
    pub login_req_paths: Vec<String>,
    pub logout_req_method: String,
    pub logout_req_path: String,
    pub double_auth_req_method: String,
    pub double_auth_req_path: String,
}
impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            login_req_method: "put".to_string(),
            login_req_paths: vec![
                "/iam/cp/login/userpwd".to_string(),
                "/iam/cp/login/oauth2".to_string(),
                "/iam/cp/login/mailvcode/vcode".to_string(),
                "/iam/cp/login/mailvcode".to_string(),
                "/iam/cp/login/phonecode/vcode".to_string(),
                "/iam/cp/login/phonevcode".to_string(),
                "/iam/cp/ldap/login".to_string(),
            ],
            logout_req_method: "delete".to_string(),
            logout_req_path: "/iam/cp/logout".to_string(),
            double_auth_req_method: "put".to_string(),
            double_auth_req_path: "/iam/cp/validate/userpwd".to_string(),
        }
    }
}
