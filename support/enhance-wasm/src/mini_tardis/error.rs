use core::fmt::Display;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::num::{ParseIntError, TryFromIntError};
use std::str::Utf8Error;
use std::string::FromUtf8Error;
use std::sync::{PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};
use wasm_bindgen::{JsError, JsValue};

static HIDE_ERROR_DETAIL: RwLock<bool> = RwLock::new(false);

pub fn set_hide_error_detail(is_hide: bool) {
    let mut hide_error_detail = HIDE_ERROR_DETAIL.write().unwrap();
    *hide_error_detail = is_hide;
}

pub static ERROR_DEFAULT_CODE: &str = "-1";

/// Tardis unified error wrapper / Tardis统一错误封装
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TardisError {
    pub code: String,
    pub message: String,
}

impl Display for TardisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.code, self.message)
    }
}

impl TardisError {
    fn error(code: &str, msg: &str, _locale_code: &str) -> TardisError {
        TardisError {
            code: code.to_string(),
            message: msg.to_string(),
        }
    }

    pub fn internal_error(msg: &str, locale_code: &str) -> TardisError {
        Self::error("500", msg, locale_code)
    }

    pub fn not_implemented(msg: &str, locale_code: &str) -> TardisError {
        Self::error("501", msg, locale_code)
    }

    pub fn io_error(msg: &str, locale_code: &str) -> TardisError {
        Self::error("503", msg, locale_code)
    }

    pub fn bad_request(msg: &str, locale_code: &str) -> TardisError {
        Self::error("400", msg, locale_code)
    }

    pub fn unauthorized(msg: &str, locale_code: &str) -> TardisError {
        Self::error("401", msg, locale_code)
    }

    pub fn forbidden(msg: &str, locale_code: &str) -> TardisError {
        Self::error("403", msg, locale_code)
    }

    pub fn not_found(msg: &str, locale_code: &str) -> TardisError {
        Self::error("404", msg, locale_code)
    }

    pub fn format_error(msg: &str, locale_code: &str) -> TardisError {
        Self::error("406", msg, locale_code)
    }

    pub fn timeout(msg: &str, locale_code: &str) -> TardisError {
        Self::error("408", msg, locale_code)
    }

    pub fn conflict(msg: &str, locale_code: &str) -> TardisError {
        Self::error("409", msg, locale_code)
    }

    pub fn custom(code: &str, msg: &str, locale_code: &str) -> TardisError {
        Self::error(code, msg, locale_code)
    }

    pub fn wrap(msg: &str, locale_code: &str) -> TardisError {
        Self::error(ERROR_DEFAULT_CODE, msg, locale_code)
    }
}

impl From<std::io::Error> for TardisError {
    fn from(error: std::io::Error) -> Self {
        TardisError::io_error(&format!("[Tardis.Basic] {error}"), "")
    }
}

impl From<Utf8Error> for TardisError {
    fn from(error: Utf8Error) -> Self {
        TardisError::format_error(&format!("[Tardis.Basic] {error}"), "")
    }
}

impl From<FromUtf8Error> for TardisError {
    fn from(error: FromUtf8Error) -> Self {
        TardisError::format_error(&format!("[Tardis.Basic] {error}"), "")
    }
}

impl From<ParseIntError> for TardisError {
    fn from(error: ParseIntError) -> Self {
        TardisError::format_error(&format!("[Tardis.Basic] {error}"), "")
    }
}

impl From<Infallible> for TardisError {
    fn from(error: Infallible) -> Self {
        TardisError::format_error(&format!("[Tardis.Basic] {error}"), "")
    }
}

impl From<base64::DecodeError> for TardisError {
    fn from(error: base64::DecodeError) -> Self {
        TardisError::format_error(&format!("[Tardis.Basic] {error}"), "")
    }
}

impl From<hex::FromHexError> for TardisError {
    fn from(error: hex::FromHexError) -> Self {
        TardisError::format_error(&format!("[Tardis.Basic] {error}"), "")
    }
}

impl From<TryFromIntError> for TardisError {
    fn from(error: TryFromIntError) -> Self {
        TardisError::format_error(&format!("[Tardis.Basic] {error}"), "")
    }
}

impl<P> From<PoisonError<RwLockReadGuard<'_, P>>> for TardisError {
    fn from(error: PoisonError<RwLockReadGuard<'_, P>>) -> Self {
        TardisError::conflict(&format!("[Tardis.Basic] {error}"), "")
    }
}

impl<P> From<PoisonError<RwLockWriteGuard<'_, P>>> for TardisError {
    fn from(error: PoisonError<RwLockWriteGuard<'_, P>>) -> Self {
        TardisError::conflict(&format!("[Tardis.Basic] {error}"), "")
    }
}

impl From<TardisError> for JsValue {
    fn from(error: TardisError) -> Self {
        if *HIDE_ERROR_DETAIL.read().unwrap() {
            JsValue::try_from(JsError::new(&format!("Abnormal operation"))).unwrap()
        } else {
            JsValue::try_from(JsError::new(&format!("[{}]{}", error.code, error.message))).unwrap()
        }
    }
}

impl From<JsValue> for TardisError {
    fn from(error: JsValue) -> Self {
        if *HIDE_ERROR_DETAIL.read().unwrap() {
            TardisError::wrap("Abnormal operation", "")
        } else {
            TardisError::wrap(&format!("{error:?}"), "")
        }
    }
}
