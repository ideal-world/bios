use std::collections::HashMap;

use tardis::{
    basic::{error::TardisError, result::TardisResult},
    crypto::crypto_sm2_4::{TardisCryptoSm2PrivateKey, TardisCryptoSm2PublicKey},
    log::trace,
    tokio::sync::RwLock,
    TardisFuns,
};

use lazy_static::lazy_static;

use crate::{
    auth_config::AuthConfig,
    auth_constants::DOMAIN_CODE,
    dto::auth_crypto_dto::{AuthEncryptReq, AuthEncryptResp},
};

lazy_static! {
    static ref SM2_KEYS: RwLock<Option<(TardisCryptoSm2PublicKey, TardisCryptoSm2PrivateKey)>> = RwLock::new(None);
}

pub(crate) async fn init() -> TardisResult<()> {
    let cache_client = TardisFuns::cache_by_module_or_default(DOMAIN_CODE);
    let config = TardisFuns::cs_config::<AuthConfig>(DOMAIN_CODE);
    let (pri_key, pub_key) = if let Some(pri_key) = cache_client.get(&config.cache_key_crypto_key).await? {
        let pri_key = TardisFuns::crypto.sm2.new_private_key_from_str(&pri_key)?;
        let pub_key = TardisFuns::crypto.sm2.new_public_key(&pri_key)?;
        (pri_key, pub_key)
    } else {
        let pri_key = TardisFuns::crypto.sm2.new_private_key()?;
        let pub_key = TardisFuns::crypto.sm2.new_public_key(&pri_key)?;
        cache_client.set(&config.cache_key_crypto_key, &pri_key.serialize()?).await?;
        (pri_key, pub_key)
    };
    let mut sm_keys = SM2_KEYS.write().await;
    *sm_keys = Some((pub_key, pri_key));
    Ok(())
}

pub(crate) async fn fetch_public_key() -> TardisResult<String> {
    let sm_keys = SM2_KEYS.read().await;
    sm_keys.as_ref().ok_or_else(|| TardisError::internal_error("[Auth.crypto] get sm keys none", ""))?.0.serialize()
}

pub(crate) async fn decrypt_req(
    headers: &HashMap<String, String>,
    body: &Option<String>,
    need_crypto_req: bool,
    need_crypto_resp: bool,
    config: &AuthConfig,
) -> TardisResult<(Option<String>, Option<HashMap<String, String>>)> {
    let input_keys = if let Some(r) = headers.get(&config.head_key_crypto) {
        r
    } else if let Some(r) = headers.get(&config.head_key_crypto.to_lowercase()) {
        r
    } else {
        return Err(TardisError::bad_request(
            &format!("[Auth] Encrypted request: {} field is not in header.", config.head_key_crypto),
            "401-auth-req-crypto-error",
        ));
    };

    let input_keys = TardisFuns::crypto.base64.decode(input_keys).map_err(|_| {
        TardisError::bad_request(
            &format!("[Auth] Encrypted request: {} field in header is not base64 format.", config.head_key_crypto),
            "401-auth-req-crypto-error",
        )
    })?;
    let sm_keys = SM2_KEYS.read().await;
    let input_keys = sm_keys
        .as_ref()
        .expect("[Auth] Encrypted request: sm keys none")
        .1
        .decrypt(&input_keys)
        .map_err(|e| TardisError::bad_request(&format!("[Auth] Encrypted request: decrypt error:{e}"), "401-auth-req-crypto-error"))?;
    let input_keys = input_keys.split(' ').collect::<Vec<&str>>();

    // allow scope of encryption is enlarged .see[enhance-wasm::modules::crypto_process::encrypt]
    if (need_crypto_req && need_crypto_resp && input_keys.len() != 4) || (need_crypto_req && input_keys.len() < 3) || (input_keys.is_empty()) {
        return Err(TardisError::bad_request(
            &format!("[Auth] Encrypted request: {} field in header is illegal.", config.head_key_crypto),
            "401-auth-req-crypto-error",
        ));
    };

    if input_keys.len() == 4 {
        let input_sm3_digest = input_keys[0];
        let input_sm4_key = input_keys[1];
        let input_sm4_iv = input_keys[2];
        let input_pub_key = input_keys[3];

        if let Some(body) = body.as_ref() {
            if input_sm3_digest != TardisFuns::crypto.digest.sm3(body)? {
                return Err(TardisError::bad_request("[Auth] Encrypted request: body digest error.", "401-auth-req-crypto-error"));
            }

            let data = TardisFuns::crypto
                .sm4
                .decrypt_cbc(body, input_sm4_key, input_sm4_iv)
                .map_err(|e| TardisError::bad_request(&format!("[Auth] Encrypted request: key decrypt error:{e}"), "401-auth-req-crypto-error"))?;
            Ok((
                Some(data),
                Some(HashMap::from([(config.head_key_crypto.to_string(), TardisFuns::crypto.base64.encode(input_pub_key))])),
            ))
        } else {
            Ok((
                None,
                Some(HashMap::from([(config.head_key_crypto.to_string(), TardisFuns::crypto.base64.encode(input_pub_key))])),
            ))
        }
    } else if input_keys.len() == 3 {
        let input_sm3_digest = input_keys[0];
        let input_sm4_key = input_keys[1];
        let input_sm4_iv = input_keys[2];
        if let Some(body) = body.as_ref() {
            if input_sm3_digest != TardisFuns::crypto.digest.sm3(body)? {
                return Err(TardisError::bad_request("[Auth] Encrypted request: body digest error.", "401-auth-req-crypto-error"));
            }

            let data = TardisFuns::crypto
                .sm4
                .decrypt_cbc(body, input_sm4_key, input_sm4_iv)
                .map_err(|e| TardisError::bad_request(&format!("[Auth] Encrypted request: body decrypt error:{e}"), "401-auth-req-crypto-error"))?;
            Ok((Some(data), None))
        } else {
            Ok((None, None))
        }
    } else {
        let input_pub_key = input_keys[0];
        Ok((
            None,
            Some(HashMap::from([(config.head_key_crypto.to_string(), TardisFuns::crypto.base64.encode(input_pub_key))])),
        ))
    }
}

pub(crate) async fn encrypt_body(req: &AuthEncryptReq) -> TardisResult<AuthEncryptResp> {
    let config = TardisFuns::cs_config::<AuthConfig>(DOMAIN_CODE);
    let pub_key = if let Some(r) = req.headers.get(&config.head_key_crypto) {
        r
    } else if let Some(r) = req.headers.get(&config.head_key_crypto.to_lowercase()) {
        r
    } else {
        return Err(TardisError::bad_request(
            &format!("[Auth] Encrypt response: {} field is not in header.", config.head_key_crypto),
            "401-auth-req-crypto-error",
        ));
    };

    let pub_key = TardisFuns::crypto.base64.decode(pub_key).map_err(|_| {
        TardisError::bad_request(
            &format!("[Auth] Encrypt response: {} field in header is not base64 format.", config.head_key_crypto),
            "401-auth-req-crypto-error",
        )
    })?;
    trace!("[Auth] Encrypt response: pub_key: {}", pub_key);

    let pub_key = TardisFuns::crypto
        .sm2
        .new_public_key_from_public_key(&pub_key)
        .map_err(|e| TardisError::bad_request(&format!("[Auth] Encrypt response: generate public key error:{e}"), "401-auth-req-crypto-error"))?;

    let sm4_key = TardisFuns::crypto.key.rand_16_hex()?;
    let sm4_iv = TardisFuns::crypto.key.rand_16_hex()?;

    let data = TardisFuns::crypto
        .sm4
        .encrypt_cbc(&req.body, &sm4_key, &sm4_iv)
        .map_err(|e| TardisError::bad_request(&format!("[Auth] Encrypt response: body encrypt error:{e}"), "401-auth-req-crypto-error"))?;

    let sign_data = TardisFuns::crypto.digest.sm3(&data)?;

    let pub_key = pub_key
        .encrypt(&format!("{sign_data} {sm4_key} {sm4_iv}"))
        .map_err(|e| TardisError::bad_request(&format!("[Auth] Encrypt response: key encrypt error:{e}"), "401-auth-req-crypto-error"))?;
    Ok(AuthEncryptResp {
        headers: HashMap::from([(config.head_key_crypto.to_string(), TardisFuns::crypto.base64.encode(&pub_key))]),
        body: data,
    })
}
