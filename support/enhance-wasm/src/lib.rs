use std::collections::HashMap;

use constants::{BIOS_TOKEN, STABLE_CONFIG};
use modules::global_api_process::MixRequest;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
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
    if !strict_security_mode {
        console_error_panic_hook::set_once();
    }
    Ok(())
}

#[wasm_bindgen]
/// uri: path?query eg. /iam/ct/xxx?q=1
pub fn on_before_request(method: &str, uri: &str, body: JsValue, headers: JsValue) -> Result<JsValue, JsValue> {
    if modules::double_auth_process::need_auth(method, uri)? {
        return Err(JsValue::try_from(JsError::new(&format!("Need double auth."))).unwrap());
    }
    let body = mini_tardis::serde::jsvalue_to_str(&body)?;
    let mut headers = mini_tardis::serde::jsvalue_to_obj::<HashMap<String, String>>(headers)?;
    if let Some(token) = modules::token_process::get_token()? {
        headers.insert(BIOS_TOKEN.to_string(), token);
    }
    let mix_req = if constants::get_strict_security_mode()? {
        modules::global_api_process::mix(method, uri, &body, headers)?
    } else {
        let resp = modules::crypto_process::encrypt(method, uri, &body)?;
        headers.extend(resp.additional_headers);
        MixRequest {
            method: method.to_string(),
            uri: uri.to_string(),
            body: resp.body,
            headers,
        }
    };
    Ok(mini_tardis::serde::obj_to_jsvalue(&mix_req)?)
}

#[wasm_bindgen]
pub fn on_before_response(body: JsValue, headers: JsValue) -> Result<String, JsValue> {
    let body = mini_tardis::serde::jsvalue_to_str(&body)?;
    let headers = mini_tardis::serde::jsvalue_to_obj(headers)?;
    Ok(modules::crypto_process::decrypt(&body, headers)?)
}

#[wasm_bindgen]
pub fn on_response_success(method: &str, uri: &str, body: JsValue) -> Result<(), JsValue> {
    let uri = if uri.starts_with("/") { uri.to_string() } else { format!("/{uri}") };
    let spec_opt = {
        let config = STABLE_CONFIG.read().unwrap();
        let config = config.as_ref().unwrap();
        if config.login_req_method.to_lowercase() == method.to_lowercase() && config.login_req_paths.iter().any(|u| uri.starts_with(u)) {
            1
        } else if config.logout_req_method.to_lowercase() == method.to_lowercase() && uri.starts_with(&config.logout_req_path) {
            2
        } else if config.double_auth_req_method.to_lowercase() == method.to_lowercase() && uri.starts_with(&config.double_auth_req_path) {
            3
        } else {
            0
        }
    };
    match spec_opt {
        1 => {
            if let Ok(body) = js_sys::Reflect::get(&body, &"token".into()) {
                let token = body.as_string().unwrap();
                modules::token_process::set_token(&token)?;
                modules::double_auth_process::remove_latest_authed()?;
            } else {
                return Err(JsValue::try_from(JsError::new(&format!("Body format error."))).unwrap());
            }
        }
        2 => {
            modules::token_process::remove_token()?;
            modules::double_auth_process::remove_latest_authed()?;
        }
        3 => {
            modules::double_auth_process::set_latest_authed()?;
        }
        _ => {}
    }
    Ok(())
}

#[wasm_bindgen]
pub fn encrypt(text: &str) -> Result<String, JsValue> {
    Ok(modules::crypto_process::simple_encrypt(text)?)
}

#[wasm_bindgen]
pub fn decrypt(encrypt_text: &str) -> Result<String, JsValue> {
    Ok(modules::crypto_process::simple_decrypt(encrypt_text)?)
}
