use std::collections::HashMap;

use js_sys::Date;
use serde::{Deserialize, Serialize};
use wasm_bindgen::{JsError, JsValue};

use super::crypto_process;

pub(crate) fn mix(method: &str, uri: &str, body: &str, headers: JsValue) -> Result<JsValue, JsValue> {
    let headers =
        serde_wasm_bindgen::from_value::<HashMap<String, String>>(headers).map_err(|e| JsValue::try_from(JsError::new(&format!("[Request] Deserialize error:{e}"))).unwrap())?;
    let mix_body = MixRequestBody {
        method: method.to_string(),
        uri: uri.to_string(),
        body: body.to_string(),
        headers,
        ts: Date::now(),
    };
    let mix_body = serde_wasm_bindgen::to_value(&mix_body).map_err(|e| JsValue::try_from(JsError::new(&format!("[Request] Serialize error:{e}"))).unwrap())?.as_string().unwrap();
    let resp = crypto_process::do_encrypt(&mix_body, true, true)?;
    let resp = MixRequest {
        method: "POST".to_string(),
        uri: "apis".to_string(),
        body: resp.body,
        headers: resp.additional_headers,
    };
    Ok(serde_wasm_bindgen::to_value(&resp)?)
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub(crate) struct MixRequestBody {
    pub method: String,
    pub uri: String,
    pub body: String,
    pub headers: HashMap<String, String>,
    pub ts: f64,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub(crate) struct MixRequest {
    pub method: String,
    pub uri: String,
    pub body: String,
    pub headers: HashMap<String, String>,
}
