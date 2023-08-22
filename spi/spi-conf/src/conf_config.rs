use bios_basic::rbum::rbum_config::RbumConfig;
use serde::{Deserialize, Serialize};

use crate::dto::conf_auth_dto::RegisterRequest;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct ConfConfig {
    pub rbum: RbumConfig,
    /// token ttl in second, default as 18000
    pub token_ttl: u32,
    pub auth_key: String,
    pub auth_username: String,
    pub auth_password: String,
    pub grpc_port: u16,
    pub grpc_host: std::net::IpAddr,
}

impl ConfConfig {
    pub(crate) fn get_admin_account(&self) -> RegisterRequest {
        RegisterRequest {
            username: Some(self.auth_username.clone().into()),
            password: Some(self.auth_password.clone().into()),
        }
    }
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
            grpc_port: 9080,
            grpc_host: std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
        }
    }
}
