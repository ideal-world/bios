use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::{
    constants::{ENCRYPT_FD_SM2_KEYS, ENCRYPT_SERV_PUB_KEY},
    mini_tardis::{
        crypto::{
            self,
            sm::{TardisCryptoSm2, TardisCryptoSm4},
        },
        error::TardisError,
    },
};

use super::resource_process;

pub(crate) fn init(pub_key: &str) -> Result<(), JsValue> {
    let mut serv_pub_key = ENCRYPT_SERV_PUB_KEY.write().unwrap();
    *serv_pub_key = Some(TardisCryptoSm2 {}.new_public_key_from_public_key(&pub_key)?);

    init_fd_key()
}

fn init_fd_key() -> Result<(), JsValue> {
    let sm_obj = TardisCryptoSm2 {};
    let pri_key = sm_obj.new_private_key()?;
    let pub_key = sm_obj.new_public_key(&pri_key)?;
    let mut sm_keys = ENCRYPT_FD_SM2_KEYS.write().unwrap();
    *sm_keys = Some((pub_key.serialize()?, pri_key));
    Ok(())
}

pub(crate) fn encrypt(method: &str, body: &str, uri: &str) -> Result<JsValue, JsValue> {
    let matched_res = resource_process::match_res(method, uri)?;
    // After matched multiple resources, the final selection of which resource is decided by the logic of the back-end service, the front-end does not know.
    // Therefore, the scope of encryption is enlarged, and encryption is performed whenever one of the matched multiple resources needs to be encrypted.
    if matched_res.iter().any(|f| f.need_crypto_req && f.need_crypto_resp) {
        let resp = do_encrypt(body, true, true)?;
        return Ok(serde_wasm_bindgen::to_value(&resp)?);
    } else if matched_res.iter().any(|f| f.need_crypto_req) {
        let resp = do_encrypt(body, true, false)?;
        return Ok(serde_wasm_bindgen::to_value(&resp)?);
    } else if matched_res.iter().any(|f| f.need_crypto_resp) {
        let resp = do_encrypt(body, false, true)?;
        return Ok(serde_wasm_bindgen::to_value(&resp)?);
    }
    Ok(JsValue::null())
}

pub(crate) fn do_encrypt(body: &str, need_crypto_req: bool, need_crypto_resp: bool) -> Result<EncryptResp, JsValue> {
    let serv_pub_key = ENCRYPT_SERV_PUB_KEY.read().unwrap();
    let serv_pub_key = serv_pub_key.as_ref().unwrap();

    let (body, encrypt_key) = if need_crypto_req && need_crypto_resp {
        let sm4_key = crypto::key::rand_16_hex()?;
        let sm4_iv = crypto::key::rand_16_hex()?;
        let encrypt_body =
            TardisCryptoSm4 {}.encrypt_cbc(body, &sm4_key, &sm4_iv).map_err(|e| TardisError::bad_request(&format!("[Auth] Encrypted request: body encrypt error:{e}"), ""))?;

        let fd_pub_key = ENCRYPT_FD_SM2_KEYS.read().unwrap();
        let fd_pub_key = &fd_pub_key.as_ref().unwrap().0;

        let encrypt_key = serv_pub_key
            .encrypt(&format!("{sm4_key} {sm4_iv} {fd_pub_key}"))
            .map_err(|e| TardisError::bad_request(&format!("[Auth] Encrypted request: key encrypt error:{e}"), ""))?;
        (encrypt_body, encrypt_key)
    } else if need_crypto_req {
        let sm4_key = crypto::key::rand_16_hex()?;
        let sm4_iv = crypto::key::rand_16_hex()?;
        let encrypt_body =
            TardisCryptoSm4 {}.encrypt_cbc(body, &sm4_key, &sm4_iv).map_err(|e| TardisError::bad_request(&format!("[Auth] Encrypted request: body encrypt error:{e}"), ""))?;

        let encrypt_key =
            serv_pub_key.encrypt(&format!("{sm4_key} {sm4_iv}")).map_err(|e| TardisError::bad_request(&format!("[Auth] Encrypted request: key encrypt error:{e}"), ""))?;
        (encrypt_body, encrypt_key)
    } else {
        let fd_pub_key = ENCRYPT_FD_SM2_KEYS.read().unwrap();
        let fd_pub_key = &fd_pub_key.as_ref().unwrap().0;

        let encrypt_key = serv_pub_key.encrypt(&format!("{fd_pub_key}")).map_err(|e| TardisError::bad_request(&format!("[Auth] Encrypted request: key encrypt error:{e}"), ""))?;
        (body.to_string(), encrypt_key)
    };
    let encrypt_key = crypto::base64::encode(&encrypt_key);
    Ok(EncryptResp {
        body,
        additional_headers: HashMap::from([("Tardis-Crypto".to_string(), encrypt_key)]),
    })
}

pub(crate) fn decrypt(body: &str, headers: JsValue) -> Result<String, JsValue> {
    let headers =
        serde_wasm_bindgen::from_value::<HashMap<String, String>>(headers).map_err(|e| JsValue::try_from(JsError::new(&format!("[Request] Deserialize error:{e}"))).unwrap())?;
    if let Some(encrypt_key) = headers.get("Tardis-Crypto") {
        let resp = do_decrypt(body, encrypt_key)?;
        return Ok(resp);
    } else {
        Ok(body.to_string())
    }
}

pub(crate) fn do_decrypt(body: &str, encrypt_key: &str) -> Result<String, JsValue> {
    let fd_pri_key = ENCRYPT_FD_SM2_KEYS.read().unwrap();
    let fd_pri_key = &fd_pri_key.as_ref().unwrap().1;

    let encrypt_key = crypto::base64::decode(encrypt_key)?;
    let key = fd_pri_key.decrypt(&encrypt_key)?;
    let key = key.split(" ").collect::<Vec<&str>>();
    let sm4_key = key[0];
    let sm4_iv = key[1];

    let body = TardisCryptoSm4 {}.decrypt_cbc(body, sm4_key, sm4_iv).map_err(|e| TardisError::bad_request(&format!("[Auth] Encrypted response: body decrypt error:{e}"), ""))?;
    Ok(body)
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub(crate) struct EncryptResp {
    pub body: String,
    pub additional_headers: HashMap<String, String>,
}
