use std::sync::RwLock;

use crate::{
    initializer,
    mini_tardis::{
        basic::TardisResult,
        crypto::sm::{TardisCryptoSm2PrivateKey, TardisCryptoSm2PublicKey},
    },
    modules::resource_process::ResContainerNode,
};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

pub const BIOS_CRYPTO: &str = "Bios-Crypto";
pub const BIOS_TOKEN: &str = "Bios-Token";
pub const BIOS_SESSION_CONFIG: &str = "tardis_config";

static STRICT_SECURITY_MODE: RwLock<bool> = RwLock::new(false);

lazy_static! {
    pub(crate) static ref STABLE_CONFIG: RwLock<Option<StableConfig>> = RwLock::new(None);
    pub(crate) static ref SESSION_CONFIG: RwLock<Option<SessionConfig>> = RwLock::new(None);
}

pub(crate) fn init_stable_config(strict_security_mode: bool, config: StableConfig) -> TardisResult<()> {
    let mut config_container = STABLE_CONFIG.write().unwrap();
    *config_container = Some(config);
    let mut config_container = STRICT_SECURITY_MODE.write().unwrap();
    *config_container = strict_security_mode;
    Ok(())
}

pub(crate) fn init_session_config(config: SessionConfig) -> TardisResult<()> {
    initializer::change_behavior(&config, true)?;
    let mut session_config = SESSION_CONFIG.write().unwrap();
    *session_config = Some(config);
    Ok(())
} 

pub(crate) fn get_strict_security_mode() -> TardisResult<bool> {
    let strict_security_mode = STRICT_SECURITY_MODE.read().unwrap();
    Ok(*strict_security_mode)
}

pub(crate) struct StableConfig {
    pub serv_url: String,
    pub double_auth_exp_sec: u32,
    pub res_container: ResContainerNode,
    pub serv_pub_key: TardisCryptoSm2PublicKey,
    pub fd_sm2_pub_key: String,
    pub fd_sm2_pri_key: TardisCryptoSm2PrivateKey,
    pub fd_sm4_key: (String, String),
    pub login_req_method: String,
    pub login_req_paths: Vec<String>,
    pub logout_req_method: String,
    pub logout_req_path: String,
    pub double_auth_req_method: String,
    pub double_auth_req_path: String,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct SessionConfig {
    pub token: Option<String>,
    pub double_auth_last_time: f64,
}
