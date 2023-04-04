use std::collections::HashMap;

use serde::{de::DeserializeOwned, Serialize};
use wasm_bindgen::{JsCast, JsError, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

use crate::basic::basic::TardisResp;

pub async fn request<T: Serialize + DeserializeOwned>(method: &str, url: &str, body: &str, headers: HashMap<String, String>) -> Result<Option<T>, JsValue> {
    let mut opts = RequestInit::new();
    opts.method(method);
    opts.mode(RequestMode::Cors);
    // TODO POST/PUT
    let request = Request::new_with_str_and_init(&url, &opts)?;
    for (k, v) in &headers {
        request.headers().set(k, v)?;
    }
    let window = web_sys::window().unwrap();
    let resp = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp.dyn_into().unwrap();
    let resp = JsFuture::from(resp.json()?).await?;
    // TODO 404 process
    let resp = serde_wasm_bindgen::from_value::<TardisResp<T>>(resp)?;
    if resp.is_ok() {
        Ok(resp.data)
    } else {
        Err(JsValue::try_from(JsError::new(&format!("[HTTP]:[{}]{}", resp.code, resp.msg))).unwrap())
    }
}
