use std::collections::HashMap;

use serde::{de::DeserializeOwned, Serialize};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

use super::{
    basic::{TardisResp, TardisResult},
    error::TardisError,
};

pub async fn request<T: Serialize + DeserializeOwned>(method: &str, url: &str, body: Option<&JsValue>, headers: HashMap<String, String>) -> TardisResult<Option<T>> {
    let mut opts = RequestInit::new();
    opts.method(method);
    opts.mode(RequestMode::Cors);
    opts.body(body);
    let request = Request::new_with_str_and_init(url, &opts)?;
    for (k, v) in &headers {
        request.headers().set(k, v)?;
    }
    request.headers().set("Content-Type", "application/json")?;
    let window = web_sys::window().unwrap();
    let resp = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp.dyn_into()?;
    if resp.status() > 300 {
        return Err(TardisError::wrap(&format!("[Tardis.Http] [{}]", resp.status()), ""));
    }
    let resp = JsFuture::from(resp.json()?).await?;
    let resp = crate::mini_tardis::serde::jsvalue_to_obj::<TardisResp<T>>(resp)?;
    if resp.is_ok() {
        Ok(resp.data)
    } else {
        Err(TardisError::wrap(&format!("[Tardis.Http] [{}]{}", resp.code, resp.msg), ""))
    }
}
