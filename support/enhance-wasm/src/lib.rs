use constants::STRICT_SECURITY_MODE;
use wasm_bindgen::prelude::*;
mod constants;
mod initializer;
mod mini_tardis;
mod modules;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub async fn init_by_url(service_url: &str) -> Result<(), JsValue> {
    initializer::init_by_url(service_url).await
}

#[wasm_bindgen]
pub fn init_by_conf(config: JsValue) -> Result<(), JsValue> {
    initializer::init_by_conf(config)
}

#[wasm_bindgen]
pub fn strict_security_mode() -> Result<bool, JsValue> {
    Ok(*STRICT_SECURITY_MODE.read().unwrap())
}

#[wasm_bindgen]
pub fn request(method: &str, uri: &str, body: &str, headers: JsValue) -> Result<JsValue, JsValue> {
    if modules::double_auth_process::need_auth(method, uri)? {
        return Ok(JsValue::NULL);
    }
    if *STRICT_SECURITY_MODE.read().unwrap() {
        modules::global_api_process::mix(method, uri, body, headers)
    } else {
        modules::crypto_process::encrypt(method, body, uri)
    }
}

#[wasm_bindgen]
pub fn response(body: &str, headers: JsValue, set_latest_authed: bool) -> Result<String, JsValue> {
    if set_latest_authed {
        modules::double_auth_process::set_latest_authed()?;
    }
    modules::crypto_process::decrypt(body, headers)
}

#[wasm_bindgen]
pub fn crypto_encrypt(body: &str, method: &str, uri: &str) -> Result<JsValue, JsValue> {
    modules::crypto_process::encrypt(body, method, uri)
}

#[wasm_bindgen]
pub fn crypto_decrypt(encrypt_body: &str, headers: JsValue) -> Result<String, JsValue> {
    modules::crypto_process::decrypt(encrypt_body, headers)
}

#[wasm_bindgen]
pub fn double_auth_set_latest_authed() -> Result<(), JsValue> {
    modules::double_auth_process::set_latest_authed()
}

#[wasm_bindgen]
pub fn double_auth_need_auth(method: &str, uri: &str) -> Result<bool, JsValue> {
    modules::double_auth_process::need_auth(method, uri)
}
