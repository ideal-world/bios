use constants::STRICT_SECURITY_MODE;
use modules::global_api_process::MixRequest;
use wasm_bindgen::prelude::*;
mod constants;
mod initializer;
mod mini_tardis;
mod modules;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub async fn init(service_url: &str, config: JsValue) -> Result<(), JsValue> {
   let strict_security_mode = if config == JsValue::NULL {
        initializer::init(service_url, None).await?
    } else {
        initializer::init(service_url, Some(mini_tardis::serde::jsvalue_to_obj(config)?)).await?
    };
    if !strict_security_mode{
        console_error_panic_hook::set_once();
    }
    Ok(())
}

#[wasm_bindgen]
pub fn set_latest_double_authed() -> Result<(), JsValue> {
    Ok(modules::double_auth_process::set_latest_authed()?)
}

#[wasm_bindgen]
pub fn set_token(token: &str) -> Result<(), JsValue> {
    Ok(modules::logout_process::set_token(token)?)
}

#[wasm_bindgen]
pub fn remove_token() -> Result<(), JsValue> {
    Ok(modules::logout_process::remove_token()?)
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
pub fn encrypt(text: &str) -> Result<String, JsValue> {
    Ok(modules::crypto_process::simple_encrypt(text)?)
}

#[wasm_bindgen]
pub fn decrypt(encrypt_text: &str) -> Result<String, JsValue> {
    Ok(modules::crypto_process::simple_decrypt(encrypt_text)?)
}
