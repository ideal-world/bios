use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use wasm_bindgen::{JsError, JsValue};

use crate::{
    constants::{GLOBAL_API_MODE, SERV_URL},
    mini_tardis::http,
    modules::{crypto_process, res_process},
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
    if config.global_api_mode {
        let mut global_api_mode = GLOBAL_API_MODE.write().unwrap();
        *global_api_mode = true;
    }
    for api in config.apis {
        res_process::add_res(&api.action, &api.uri, api.need_crypto_req, api.need_crypto_resp, api.need_double_auth)?;
    }
    crypto_process::init(&config.pub_key)?;
    Ok(())
}

#[derive(Serialize, Deserialize, Default)]
struct Config {
    pub global_api_mode: bool,
    pub pub_key: String,
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
