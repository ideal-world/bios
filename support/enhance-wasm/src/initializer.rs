use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use wasm_bindgen::{JsError, JsValue};

use crate::{
    constants::{DOUBLE_AUTH_CACHE_EXP_SEC, SERV_URL, STRICT_SECURITY_MODE},
    mini_tardis::http,
    modules::{crypto_process, resource_process},
};

pub(crate) async fn init_by_url(service_url: &str) -> Result<(), JsValue> {
    let service_url = if service_url.ends_with("/") {
        service_url.to_string()
    } else {
        format!("{service_url}/")
    };
    let config = http::request::<Config>("GET", &format!("{service_url}apis"), "", HashMap::new()).await?.unwrap();
    do_init(config)?;
    let mut serv_url = SERV_URL.write().unwrap();
    *serv_url = service_url;
    Ok(())
}

pub(crate) fn init_by_conf(config: JsValue) -> Result<(), JsValue> {
    let config = serde_wasm_bindgen::from_value::<Config>(config).map_err(|e| JsValue::try_from(JsError::new(&format!("[Init] Deserialize error:{e}"))).unwrap())?;
    do_init(config)?;
    Ok(())
}

fn do_init(config: Config) -> Result<(), JsValue> {
    if config.strict_security_mode {
        let mut strict_security_mode = STRICT_SECURITY_MODE.write().unwrap();
        *strict_security_mode = true;
    }
    if config.double_auth_exp_sec != 0 {
        let mut double_auth_exp_sec = DOUBLE_AUTH_CACHE_EXP_SEC.write().unwrap();
        *double_auth_exp_sec = (0.0, config.double_auth_exp_sec);
    }
    for api in config.apis {
        resource_process::add_res(&api.action, &api.uri, api.need_crypto_req, api.need_crypto_resp, api.need_double_auth)?;
    }
    crypto_process::init(&config.pub_key)?;
    Ok(())
}

#[derive(Serialize, Deserialize, Default)]
struct Config {
    pub strict_security_mode: bool,
    pub pub_key: String,
    pub double_auth_exp_sec: u32,
    pub apis: Vec<Api>,
}

#[derive(Serialize, Deserialize, Default)]
struct Api {
    pub action: String,
    pub uri: String,
    pub need_crypto_req: bool,
    pub need_crypto_resp: bool,
    pub need_double_auth: bool,
}
