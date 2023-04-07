use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::Serializer;
use wasm_bindgen::{JsError, JsValue};

use crate::mini_tardis::time;

use super::crypto_process;

pub fn mix(method: &str, uri: &str, body: &str, headers: HashMap<String, String>) -> Result<JsValue, JsValue> {
    let mix_body = MixRequestBody {
        method: method.to_string(),
        uri: uri.to_string(),
        body: body.to_string(),
        headers,
        ts: time::now(),
    };
    let mix_body = mix_body
        .serialize(&Serializer::json_compatible())
        .map_err(|e| JsValue::try_from(JsError::new(&format!("[BIOS.GlobalApi] Serialize mixed body error:{e}"))).unwrap())?
        .as_string()
        .unwrap();
    let resp = crypto_process::do_encrypt(&mix_body, true, true)?;
    let resp = MixRequest {
        method: "POST".to_string(),
        uri: "apis".to_string(),
        body: resp.body,
        headers: resp.additional_headers,
    };
    Ok(resp.serialize(&Serializer::json_compatible())?)
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct MixRequestBody {
    pub method: String,
    pub uri: String,
    pub body: String,
    pub headers: HashMap<String, String>,
    pub ts: f64,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct MixRequest {
    pub method: String,
    pub uri: String,
    pub body: String,
    pub headers: HashMap<String, String>,
}
