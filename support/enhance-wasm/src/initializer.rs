use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;

use crate::{
    constants::{DOUBLE_AUTH_CACHE_EXP_SEC, SERV_URL, STRICT_SECURITY_MODE, TOKEN_INFO},
    mini_tardis::{basic::TardisResult, http, log},
    modules::{crypto_process, resource_process},
};

pub(crate) async fn init(service_url: &str, config: Option<Config>) -> Result<bool, JsValue> {
    let service_url = if service_url.ends_with("/") {
        service_url.to_string()
    } else {
        format!("{service_url}/")
    };
    log::log(&format!("[BIOS] Init."));
    let config = if let Some(config) = config {
        config
    } else {
        log::log(&format!("[BIOS] Init by url: {service_url}."));
        http::request::<Config>("GET", &format!("{service_url}auth/apis"), "", HashMap::new()).await?.unwrap()
    };
    do_init(&service_url, &config)?;
    Ok(config.strict_security_mode)
}

pub(crate) fn do_init(service_url: &str, config: &Config) -> TardisResult<()> {
    let mut serv_url = SERV_URL.write().unwrap();
    *serv_url = service_url.to_string();
    if config.strict_security_mode {
        let mut strict_security_mode = STRICT_SECURITY_MODE.write().unwrap();
        *strict_security_mode = true;
    }
    if config.double_auth_exp_sec != 0 {
        let mut double_auth_exp_sec = DOUBLE_AUTH_CACHE_EXP_SEC.write().unwrap();
        *double_auth_exp_sec = (0.0, config.double_auth_exp_sec);
    }
    let mut token_info = TOKEN_INFO.write().unwrap();
    *token_info = None;
    for api in &config.apis {
        resource_process::add_res(&api.action, &api.uri, api.need_crypto_req, api.need_crypto_resp, api.need_double_auth)?;
    }
    crypto_process::init(&config.pub_key)?;
    Ok(())
}

#[derive(Serialize, Deserialize, Default)]
pub(crate) struct Config {
    pub strict_security_mode: bool,
    pub pub_key: String,
    pub double_auth_exp_sec: u32,
    pub apis: Vec<Api>,
}

#[derive(Serialize, Deserialize, Default)]
pub(crate) struct Api {
    pub action: String,
    pub uri: String,
    pub need_crypto_req: bool,
    pub need_crypto_resp: bool,
    pub need_double_auth: bool,
}
