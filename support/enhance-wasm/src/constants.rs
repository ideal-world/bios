use std::sync::RwLock;

use lazy_static::lazy_static;
use wasm_bindgen::{JsError, JsValue};

use crate::{
    mini_tardis::{
        crypto::sm::{TardisCryptoSm2PrivateKey, TardisCryptoSm2PublicKey},
        error::TardisError,
    },
    modules::resource_process::ResContainerNode,
};

pub const TARDIS_CRYPTO: &str = "Tardis-Crypto";
pub const TARDIS_TOKEN: &str = "tardis_token";

lazy_static! {
    pub(crate) static ref SERV_URL: RwLock<String> = RwLock::new(String::new());
    pub(crate) static ref STRICT_SECURITY_MODE: RwLock<bool> = RwLock::new(false);
    // token
    pub(crate) static ref TOKEN_INFO: RwLock<Option<String>> = RwLock::new(None);
    // last auth time, expire sec
    pub(crate) static ref DOUBLE_AUTH_CACHE_EXP_SEC: RwLock<(f64, u32)> = RwLock::new((0.0, 0));
    pub(crate) static ref RES_CONTAINER: RwLock<Option<ResContainerNode>> = RwLock::new(None);
    pub(crate) static ref ENCRYPT_SERV_PUB_KEY: RwLock<Option<TardisCryptoSm2PublicKey>> = RwLock::new(None);
    pub(crate) static ref ENCRYPT_FD_SM2_KEYS: RwLock<Option<(String, TardisCryptoSm2PrivateKey)>> = RwLock::new(None);
    // Only use for simple crypto
    pub(crate) static ref ENCRYPT_FD_SM4_KEY: RwLock<(String, String)> = RwLock::new((String::new(), String::new()));
}

impl From<TardisError> for JsValue {
    fn from(error: TardisError) -> Self {
        if *STRICT_SECURITY_MODE.read().unwrap() {
            JsValue::try_from(JsError::new(&format!("Abnormal operation"))).unwrap()
        } else {
            JsValue::try_from(JsError::new(&format!("[{}]{}", error.code, error.message))).unwrap()
        }
    }
}
