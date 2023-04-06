use js_sys::Date;
use wasm_bindgen::JsValue;

use crate::constants::DOUBLE_AUTH_CACHE_EXP_SEC;

pub(crate) fn set_latest_authed() -> Result<(), JsValue> {
    let mut double_auth_exp_sec = DOUBLE_AUTH_CACHE_EXP_SEC.write().unwrap();
    *double_auth_exp_sec = (Date::now() + (double_auth_exp_sec.1 * 1000) as f64, double_auth_exp_sec.1);
    Ok(())
}

pub(crate) fn need_auth() -> Result<bool, JsValue> {
    Ok(DOUBLE_AUTH_CACHE_EXP_SEC.read().unwrap().0 < Date::now())
}
