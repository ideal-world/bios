use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::RwLock};
use wasm_bindgen::prelude::*;

use lazy_static::lazy_static;

use crate::{
    basic::error::TardisError,
    helper::{
        crypto_base64, crypto_key,
        crypto_sm2_4::{TardisCryptoSm2, TardisCryptoSm2PrivateKey, TardisCryptoSm2PublicKey, TardisCryptoSm4},
        http,
    },
};

lazy_static! {
    static ref SERV_PUB_KEY: RwLock<Option<TardisCryptoSm2PublicKey>> = RwLock::new(None);
    static ref FD_SM2_KEYS: RwLock<Option<(String, TardisCryptoSm2PrivateKey)>> = RwLock::new(None);
}

pub(crate) async fn init(serv_url: &str) -> Result<(), JsValue> {
    let pub_key = http::request::<String>(
        "GET",
        &format!("{serv_url}/auth/crypto/key"),
        "",
        HashMap::from([("Content-Type".to_string(), "application/json".to_string())]),
    )
    .await?
    .unwrap();
    let mut serv_pub_key = SERV_PUB_KEY.write().unwrap();
    *serv_pub_key = Some(TardisCryptoSm2 {}.new_public_key_from_public_key(&pub_key)?);

    init_fd_key()
}

pub(crate) fn init_fd_key() -> Result<(), JsValue> {
    let sm_obj = TardisCryptoSm2 {};
    let pri_key = sm_obj.new_private_key()?;
    let pub_key = sm_obj.new_public_key(&pri_key)?;
    let mut sm_keys = FD_SM2_KEYS.write().unwrap();
    *sm_keys = Some((pub_key.serialize()?, pri_key));
    Ok(())
}

pub fn encrypt(body: &str, need_crypto_req: bool, need_crypto_resp: bool) -> Result<JsValue, JsValue> {
    let serv_pub_key = SERV_PUB_KEY.read().unwrap();
    let serv_pub_key = serv_pub_key.as_ref().unwrap();

    let (body, encrypt_key) = if need_crypto_req && need_crypto_resp {
        let sm4_key = crypto_key::rand_16_hex()?;
        let sm4_iv = crypto_key::rand_16_hex()?;
        let encrypt_body =
            TardisCryptoSm4 {}.encrypt_cbc(body, &sm4_key, &sm4_iv).map_err(|e| TardisError::bad_request(&format!("[Auth] Encrypted request: body encrypt error:{e}"), ""))?;

        let fd_pub_key = FD_SM2_KEYS.read().unwrap();
        let fd_pub_key = &fd_pub_key.as_ref().unwrap().0;

        let encrypt_key = serv_pub_key
            .encrypt(&format!("{sm4_key} {sm4_iv} {fd_pub_key}"))
            .map_err(|e| TardisError::bad_request(&format!("[Auth] Encrypted request: key encrypt error:{e}"), ""))?;
        (encrypt_body, encrypt_key)
    } else if need_crypto_req {
        let sm4_key = crypto_key::rand_16_hex()?;
        let sm4_iv = crypto_key::rand_16_hex()?;
        let encrypt_body =
            TardisCryptoSm4 {}.encrypt_cbc(body, &sm4_key, &sm4_iv).map_err(|e| TardisError::bad_request(&format!("[Auth] Encrypted request: body encrypt error:{e}"), ""))?;

        let encrypt_key =
            serv_pub_key.encrypt(&format!("{sm4_key} {sm4_iv}")).map_err(|e| TardisError::bad_request(&format!("[Auth] Encrypted request: key encrypt error:{e}"), ""))?;
        (encrypt_body, encrypt_key)
    } else {
        let fd_pub_key = FD_SM2_KEYS.read().unwrap();
        let fd_pub_key = &fd_pub_key.as_ref().unwrap().0;

        let encrypt_key = serv_pub_key.encrypt(&format!("{fd_pub_key}")).map_err(|e| TardisError::bad_request(&format!("[Auth] Encrypted request: key encrypt error:{e}"), ""))?;
        (body.to_string(), encrypt_key)
    };
    let encrypt_key = crypto_base64::encode(&encrypt_key);
    let resp = EncryptResp { body, key: encrypt_key };
    Ok(serde_wasm_bindgen::to_value(&resp)?)
}

pub fn decrypt(encrypt_body: &str, encrypt_key: &str) -> Result<String, JsValue> {
    let fd_pri_key = FD_SM2_KEYS.read().unwrap();
    let fd_pri_key = &fd_pri_key.as_ref().unwrap().1;

    let encrypt_key = crypto_base64::decode(encrypt_key)?;
    let key = fd_pri_key.decrypt(&encrypt_key)?;
    let key = key.split(" ").collect::<Vec<&str>>();
    let sm4_key = key[0];
    let sm4_iv = key[1];

    let body =
        TardisCryptoSm4 {}.decrypt_cbc(encrypt_body, sm4_key, sm4_iv).map_err(|e| TardisError::bad_request(&format!("[Auth] Encrypted response: body decrypt error:{e}"), ""))?;
    Ok(body)
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct EncryptResp {
    pub body: String,
    pub key: String,
}
