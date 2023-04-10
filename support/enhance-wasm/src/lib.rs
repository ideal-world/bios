use constants::STRICT_SECURITY_MODE;
use modules::global_api_process::MixRequest;
use wasm_bindgen::prelude::*;
mod constants;
mod initializer;
mod mini_tardis;
mod modules;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

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
    let mut headers = mini_tardis::serde::jsvalue_to_obj(headers)?;
    if *STRICT_SECURITY_MODE.read().unwrap() {
        let resp = modules::global_api_process::mix(method, uri, body, headers)?;
        Ok(mini_tardis::serde::obj_to_jsvalue(&resp)?)
    } else {
        let resp = modules::crypto_process::encrypt(method, uri, body)?;
        headers.extend(resp.additional_headers);
        let resp = MixRequest {
            method: method.to_string(),
            uri: uri.to_string(),
            body: resp.body,
            headers,
        };
        Ok(mini_tardis::serde::obj_to_jsvalue(&resp)?)
    }
}

#[wasm_bindgen]
pub fn response(body: &str, headers: JsValue, set_latest_authed: bool) -> Result<String, JsValue> {
    if set_latest_authed {
        modules::double_auth_process::set_latest_authed()?;
    }
    let headers = mini_tardis::serde::jsvalue_to_obj(headers)?;
    Ok(modules::crypto_process::decrypt(body, headers)?)
}

#[wasm_bindgen]
pub fn crypto_encrypt(method: &str, uri: &str, body: &str) -> Result<JsValue, JsValue> {
    let resp = modules::crypto_process::encrypt(method, uri, body);
    Ok(mini_tardis::serde::obj_to_jsvalue(&resp)?)
}

#[wasm_bindgen]
pub fn crypto_decrypt(encrypt_body: &str, headers: JsValue) -> Result<String, JsValue> {
    let headers = mini_tardis::serde::jsvalue_to_obj(headers)?;
    Ok(modules::crypto_process::decrypt(encrypt_body, headers)?)
}

#[wasm_bindgen]
pub fn double_auth_set_latest_authed() -> Result<(), JsValue> {
    Ok(modules::double_auth_process::set_latest_authed()?)
}

#[wasm_bindgen]
pub fn double_auth_need_auth(method: &str, uri: &str) -> Result<bool, JsValue> {
    Ok(modules::double_auth_process::need_auth(method, uri)?)
}
