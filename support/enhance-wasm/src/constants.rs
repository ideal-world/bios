use std::sync::RwLock;

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::{
    mini_tardis::{
        self,
        basic::TardisResult,
        channel,
        crypto::{
            self,
            sm::{TardisCryptoSm2PrivateKey, TardisCryptoSm2PublicKey},
        },
        error,
    },
    modules::resource_process::ResContainerNode,
};

pub const BIOS_CONFIG: &str = "tardis_config";
pub const BIOS_CRYPTO: &str = "Bios-Crypto";
pub const BIOS_TOKEN: &str = "Bios-Token";

lazy_static! {
    pub(crate) static ref INST: RwLock<f64> = RwLock::new(mini_tardis::time::now());
    pub(crate) static ref CONFIG: RwLock<Option<Config>> = RwLock::new(None);
}

pub(crate) fn is_conf_unset() -> TardisResult<bool> {
    Ok(CONFIG.read().unwrap().is_none())
}

pub(crate) fn set_config(config: Config) -> TardisResult<()> {
    error::set_hide_error_detail(config.strict_security_mode);
    let mut config_container = CONFIG.write().unwrap();
    if config_container.is_none() {
        *config_container = Some(config);
    }
    Ok(())
}

pub(crate) fn remove_config() -> TardisResult<()> {
    let mut config_container = CONFIG.write().unwrap();
    *config_container = None;
    Ok(())
}

pub(crate) fn conf_by_double_auth_last_time(double_auth_last_time: f64) -> TardisResult<()> {
    let mut config_container = CONFIG.write().unwrap();
    let config = config_container.as_mut().unwrap();
    config.double_auth_last_time = double_auth_last_time;
    #[cfg(target_arch = "wasm32")]
    {
        channel::send(BIOS_CONFIG, &SerConfig::from(&config))?;
    }
    Ok(())
}

pub(crate) fn conf_by_token(token: Option<String>) -> TardisResult<()> {
    let mut config_container = CONFIG.write().unwrap();
    let config = config_container.as_mut().unwrap();
    config.token = token;
    #[cfg(target_arch = "wasm32")]
    {
        channel::send(BIOS_CONFIG, &SerConfig::from(&config))?;
    }
    Ok(())
}

pub(crate) fn conf_by_strict_security_mode() -> TardisResult<bool> {
    let config = CONFIG.read().unwrap();
    Ok(config.as_ref().unwrap().strict_security_mode)
}

pub(crate) fn init() -> TardisResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        channel::init(
            BIOS_CONFIG,
            |_| {
                let config_container = CONFIG.read().unwrap();
                if let Some(config) = config_container.as_ref() {
                    channel::send(BIOS_CONFIG, &SerConfig::from(config)).unwrap();
                }
            },
            |ser_config| {
                let ser_config = mini_tardis::serde::jsvalue_to_obj::<SerConfig>(ser_config).unwrap();
                remove_config().unwrap();
                set_config(Config::from(ser_config)).unwrap();
            },
        )?;
    }
    Ok(())
}

pub(crate) struct Config {
    pub strict_security_mode: bool,
    pub serv_url: String,
    pub token: Option<String>,
    pub double_auth_last_time: f64,
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

impl Config {
    fn from(ser_config: SerConfig) -> Self {
        Config {
            strict_security_mode: ser_config.strict_security_mode,
            serv_url: ser_config.serv_url,
            token: ser_config.token,
            double_auth_last_time: ser_config.double_auth_last_time,
            double_auth_exp_sec: ser_config.double_auth_exp_sec,
            res_container: ser_config.res_container,
            serv_pub_key: crypto::sm::TardisCryptoSm2PublicKey::from_public_key_str(&ser_config.serv_pub_key).unwrap(),
            fd_sm2_pub_key: ser_config.fd_sm2_pub_key,
            fd_sm2_pri_key: crypto::sm::TardisCryptoSm2PrivateKey::from(&ser_config.fd_sm2_pri_key).unwrap(),
            fd_sm4_key: ser_config.fd_sm4_key,
            login_req_method: ser_config.login_req_method,
            login_req_paths: ser_config.login_req_paths,
            logout_req_method: ser_config.logout_req_method,
            logout_req_path: ser_config.logout_req_path,
            double_auth_req_method: ser_config.double_auth_req_method,
            double_auth_req_path: ser_config.double_auth_req_path,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct SerConfig {
    pub strict_security_mode: bool,
    pub serv_url: String,
    pub token: Option<String>,
    pub double_auth_last_time: f64,
    pub double_auth_exp_sec: u32,
    pub res_container: ResContainerNode,
    pub serv_pub_key: String,
    pub fd_sm2_pub_key: String,
    pub fd_sm2_pri_key: String,
    pub fd_sm4_key: (String, String),
    pub login_req_method: String,
    pub login_req_paths: Vec<String>,
    pub logout_req_method: String,
    pub logout_req_path: String,
    pub double_auth_req_method: String,
    pub double_auth_req_path: String,
}

impl SerConfig {
    fn from(config: &Config) -> Self {
        SerConfig {
            strict_security_mode: config.strict_security_mode,
            serv_url: config.serv_url.clone(),
            token: config.token.clone(),
            double_auth_last_time: config.double_auth_last_time,
            double_auth_exp_sec: config.double_auth_exp_sec,
            res_container: mini_tardis::serde::copy(&config.res_container).unwrap(),
            serv_pub_key: config.serv_pub_key.serialize().unwrap(),
            fd_sm2_pub_key: config.fd_sm2_pub_key.clone(),
            fd_sm2_pri_key: config.fd_sm2_pri_key.serialize().unwrap(),
            fd_sm4_key: config.fd_sm4_key.clone(),
            login_req_method: config.login_req_method.clone(),
            login_req_paths: config.login_req_paths.clone(),
            logout_req_method: config.logout_req_method.clone(),
            logout_req_path: config.logout_req_path.clone(),
            double_auth_req_method: config.double_auth_req_method.clone(),
            double_auth_req_path: config.double_auth_req_path.clone(),
        }
    }
}
