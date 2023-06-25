use bios_basic::rbum::rbum_config::RbumConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct ConfConfig {
    pub rbum: RbumConfig,
    /// token ttl in second, default as 18000
    pub token_ttl: u32,
    pub auth_key: String,
    pub auth_username: String,
    pub auth_password: String,

}

impl Default for ConfConfig {
    fn default() -> Self {
        use tardis::crypto::*;
        use tardis::rand::*;
        let auth_key = crypto_base64::TardisCryptoBase64.encode(&format!("{:016x}", random::<u128>()));
        let password = format!("{:016x}", random::<u128>());

        Self {
            /// 18000 secs (5 hours)
            token_ttl: 18000,
            auth_key,
            auth_username: String::from("nacos"),
            auth_password: password,
            rbum: Default::default(),
        }
    }
}
