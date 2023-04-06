use js_sys::Date;
use wasm_bindgen::JsValue;

use crate::constants::DOUBLE_AUTH_CACHE_EXP_SEC;

use super::resource_process;

pub(crate) fn set_latest_authed() -> Result<(), JsValue> {
    let mut double_auth_exp_sec = DOUBLE_AUTH_CACHE_EXP_SEC.write().unwrap();
    *double_auth_exp_sec = (Date::now() + (double_auth_exp_sec.1 * 1000) as f64, double_auth_exp_sec.1);
    Ok(())
}

pub(crate) fn need_auth(method: &str, uri: &str) -> Result<bool, JsValue> {
    if DOUBLE_AUTH_CACHE_EXP_SEC.read().unwrap().0 > Date::now() {
        Ok(false)
    } else {
        if let Some(info) = resource_process::match_res(method, uri)?.first() {
            Ok(info.need_double_auth)
        } else {
            Ok(false)
        }
    }
}
