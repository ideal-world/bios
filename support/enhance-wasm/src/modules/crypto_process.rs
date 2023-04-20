use std::collections::HashMap;

use crate::{
    constants::{BIOS_CRYPTO, SIMPLE_SM4_SEED_CONFIG, STABLE_CONFIG},
    initializer::init_simple_sm_config_by_window,
    mini_tardis::{
        basic::TardisResult,
        crypto::{
            self,
            sm::{TardisCryptoSm2, TardisCryptoSm2PrivateKey, TardisCryptoSm4},
        },
        error::TardisError,
    },
};
use serde::{Deserialize, Serialize};

use super::resource_process;

pub fn init_fd_sm2_keys() -> TardisResult<(String, TardisCryptoSm2PrivateKey)> {
    let sm_obj = TardisCryptoSm2 {};
    let pri_key = sm_obj.new_private_key()?;
    let pub_key = sm_obj.new_public_key(&pri_key)?;
    Ok((pub_key.serialize()?, pri_key))
}

pub fn init_fd_sm4_key(seed: &str) -> TardisResult<(String, String)> {
    let key = crypto::key::rand_16_hex_by_str(&crypto::sm::digest(seed)?)?;
    Ok((key.clone(), key))
}

pub fn encrypt(method: &str, uri: &str, body: &str) -> TardisResult<EncryptResp> {
    let method = method.to_lowercase();
    let matched_res = {
        let config = STABLE_CONFIG.read().unwrap();
        let res_container = &config.as_ref().unwrap().res_container;
        resource_process::match_res(res_container, &method, uri)?
    };
    let resp = if matched_res.is_empty() {
        EncryptResp {
            body: body.to_string(),
            additional_headers: HashMap::new(),
        }
    } else if matched_res.len() == 1 {
        let matched_res = matched_res.first().unwrap();
        do_encrypt(body, matched_res.need_crypto_req, matched_res.need_crypto_resp)?
    } else {
        // After matched multiple resources, the final selection of which resource is decided by the logic of the back-end service, the front-end does not know.
        // Therefore, the scope of encryption is enlarged, and encryption is performed whenever one of the matched multiple resources needs to be encrypted.
        if matched_res.iter().any(|f| f.need_crypto_req && f.need_crypto_resp) {
            do_encrypt(body, true, true)?
        } else if matched_res.iter().any(|f| f.need_crypto_req) {
            do_encrypt(body, true, false)?
        } else if matched_res.iter().any(|f| f.need_crypto_resp) {
            do_encrypt(body, false, true)?
        } else {
            EncryptResp {
                body: body.to_string(),
                additional_headers: HashMap::new(),
            }
        }
    };
    Ok(resp)
}

pub fn do_encrypt(body: &str, need_crypto_req: bool, need_crypto_resp: bool) -> TardisResult<EncryptResp> {
    let config = STABLE_CONFIG.read().unwrap();
    let config = config.as_ref().unwrap();
    let serv_pub_key = &config.serv_pub_key;

    let (body, encrypt_key) = if need_crypto_req && need_crypto_resp {
        let sm4_key = crypto::key::rand_16_hex()?;
        let sm4_iv = crypto::key::rand_16_hex()?;
        let encrypt_body = TardisCryptoSm4 {}
            .encrypt_cbc(body, &sm4_key, &sm4_iv)
            .map_err(|e| TardisError::bad_request(&format!("[BIOS.Crypto] Encrypted request: body encrypt error:{e}"), ""))?;

        let fd_pub_key = &config.fd_sm2_pub_key;

        let sign_body = crypto::sm::digest(&encrypt_body)?;
        let encrypt_key = serv_pub_key
            .encrypt(&format!("{sign_body} {sm4_key} {sm4_iv} {fd_pub_key}"))
            .map_err(|e| TardisError::bad_request(&format!("[BIOS.Crypto] Encrypted request: key encrypt error:{e}"), ""))?;
        (encrypt_body, encrypt_key)
    } else if need_crypto_req {
        let sm4_key = crypto::key::rand_16_hex()?;
        let sm4_iv = crypto::key::rand_16_hex()?;
        let encrypt_body = TardisCryptoSm4 {}
            .encrypt_cbc(body, &sm4_key, &sm4_iv)
            .map_err(|e| TardisError::bad_request(&format!("[BIOS.Crypto] Encrypted request: body encrypt error:{e}"), ""))?;

        let sign_body = crypto::sm::digest(&encrypt_body)?;
        let encrypt_key = serv_pub_key
            .encrypt(&format!("{sign_body} {sm4_key} {sm4_iv}"))
            .map_err(|e| TardisError::bad_request(&format!("[BIOS.Crypto] Encrypted request: key encrypt error:{e}"), ""))?;
        (encrypt_body, encrypt_key)
    } else {
        let fd_pub_key = &config.fd_sm2_pub_key;

        let encrypt_key =
            serv_pub_key.encrypt(&format!("{fd_pub_key}")).map_err(|e| TardisError::bad_request(&format!("[BIOS.Crypto] Encrypted request: key encrypt error:{e}"), ""))?;
        (body.to_string(), encrypt_key)
    };
    let encrypt_key = crypto::base64::encode(&encrypt_key);
    Ok(EncryptResp {
        body,
        additional_headers: HashMap::from([(BIOS_CRYPTO.to_string(), encrypt_key)]),
    })
}

pub fn decrypt(body: &str, headers: HashMap<String, String>) -> TardisResult<String> {
    if let Some(encrypt_key) = headers.get(BIOS_CRYPTO) {
        let resp = do_decrypt(body, encrypt_key)?;
        return Ok(resp);
    } else {
        Ok(body.to_string())
    }
}

pub fn do_decrypt(body: &str, encrypt_key: &str) -> TardisResult<String> {
    let config = STABLE_CONFIG.read().unwrap();
    let fd_pri_key = &config.as_ref().unwrap().fd_sm2_pri_key;

    let encrypt_key = crypto::base64::decode(encrypt_key)?;
    let key = fd_pri_key.decrypt(&encrypt_key)?;
    let key = key.split(" ").collect::<Vec<&str>>();
    let sm3_digest = key[0];
    let sm4_key = key[1];
    let sm4_iv = key[2];

    if sm3_digest != crypto::sm::digest(body)? {
        return Err(TardisError::bad_request(&format!("[BIOS.Crypto] Encrypted response: body digest error."), ""));
    }

    let body =
        TardisCryptoSm4 {}.decrypt_cbc(body, sm4_key, sm4_iv).map_err(|e| TardisError::bad_request(&format!("[BIOS.Crypto] Encrypted response: body decrypt error:{e}"), ""))?;
    Ok(body)
}

pub fn simple_encrypt(text: &str) -> TardisResult<String> {
    let seed = {
        let config = SIMPLE_SM4_SEED_CONFIG.read().unwrap();
        (config.0.clone(), config.1.clone())
    };
    let seed = if seed.0.is_empty() { init_simple_sm_config_by_window()?.unwrap() } else { seed };
    crypto::sm::TardisCryptoSm4.encrypt_cbc(text, &seed.0, &seed.1)
}

pub fn simple_decrypt(encrypted_text: &str) -> TardisResult<String> {
    let seed = {
        let config = SIMPLE_SM4_SEED_CONFIG.read().unwrap();
        (config.0.clone(), config.1.clone())
    };
    let seed = if seed.0.is_empty() { init_simple_sm_config_by_window()?.unwrap() } else { seed };
    crypto::sm::TardisCryptoSm4.decrypt_cbc(encrypted_text, &seed.0, &seed.1)
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct EncryptResp {
    pub body: String,
    pub additional_headers: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        constants::BIOS_CRYPTO,
        initializer::{self, Api, ServConfig},
        mini_tardis::crypto::{
            self,
            sm::{TardisCryptoSm2, TardisCryptoSm2PrivateKey, TardisCryptoSm4},
        },
        modules::crypto_process::{decrypt, encrypt},
    };

    use super::do_encrypt;

    #[test]
    fn test_crypto() {
        // Prepare
        let sm2 = TardisCryptoSm2 {};
        let mock_serv_pri_key = sm2.new_private_key().unwrap();
        let mock_serv_pub_key = sm2.new_public_key(&mock_serv_pri_key).unwrap();
        initializer::do_init(
            "",
            &ServConfig {
                strict_security_mode: false,
                pub_key: mock_serv_pub_key.serialize().unwrap(),
                double_auth_exp_sec: 0,
                apis: vec![
                    Api {
                        action: "get".to_string(),
                        uri: "iam/ct/all/**".to_string(),
                        need_crypto_req: true,
                        need_crypto_resp: true,
                        need_double_auth: false,
                    },
                    Api {
                        action: "GET".to_string(),
                        uri: "iam/ct/req/**".to_string(),
                        need_crypto_req: true,
                        need_crypto_resp: false,
                        need_double_auth: false,
                    },
                    Api {
                        action: "POST".to_string(),
                        uri: "iam/ct/resp/**".to_string(),
                        need_crypto_req: false,
                        need_crypto_resp: true,
                        need_double_auth: false,
                    },
                    Api {
                        action: "get".to_string(),
                        uri: "iam/ct/all/spec/**".to_string(),
                        need_crypto_req: false,
                        need_crypto_resp: false,
                        need_double_auth: false,
                    },
                ],
                login_req_method: "".to_string(),
                login_req_paths: vec![],
                logout_req_method: "".to_string(),
                logout_req_path: "".to_string(),
                double_auth_req_method: "".to_string(),
                double_auth_req_path: "".to_string(),
            },
        )
        .unwrap();

        // empty_body also can be encrypt
        let empty_body = do_encrypt("", true, true).unwrap();
        assert!(!empty_body.body.is_empty());

        test_crypto_req_and_resp(&mock_serv_pri_key, &sm2);
        test_crypto_req(&mock_serv_pri_key);
        test_crypto_resp(&mock_serv_pri_key, &sm2);
        test_crypto_none();
    }

    fn test_crypto_req_and_resp(mock_serv_pri_key: &TardisCryptoSm2PrivateKey, sm2: &TardisCryptoSm2) {
        // Encrypt
        let mock_req_body = "中台经过几年“滚雪球”的发展或是资本地运作，已是个“庞然大物”，是到了“减肥”，“减负”的时候。一言以避之：解构中台，让他融合到更大的IT能力共享架构中，把共享交给开放平台，把技术还给技术平台，让中台专注于领域服务及事件 。";
        let encrypt_req = encrypt("Get", "iam/ct/all/spec/xxx", mock_req_body).unwrap();
        let encrypt_body = encrypt_req.body;
        let key = &encrypt_req.additional_headers[BIOS_CRYPTO];

        // Mock serv process
        // ------------------------------------------------
        // 1. Decrypt request key by service private key
        let key = mock_serv_pri_key.decrypt(&crypto::base64::decode(key).unwrap()).unwrap();
        let key = key.split(" ").collect::<Vec<&str>>();
        assert_eq!(key.len(), 4);
        let sm3_digest = key[0];
        let sm4_key = key[1];
        let sm4_iv = key[2];
        // 2. Check request body digest by frontend sm3
        assert_eq!(crypto::sm::digest(&encrypt_body).unwrap(), sm3_digest);
        let fd_pub_key = sm2.new_public_key_from_public_key(key[3]).unwrap();
        // 3. Decrypt request body by frontend sm4 key & iv
        assert_eq!(TardisCryptoSm4 {}.decrypt_cbc(&encrypt_body, sm4_key, sm4_iv).unwrap(), mock_req_body);
        // 4. Encrypt response body by service sm4 key & iv
        let sm4_key = crypto::key::rand_16_hex().unwrap();
        let sm4_iv = crypto::key::rand_16_hex().unwrap();
        let mock_resp_body = "开放平台 天然具备了开放、生态化的能力，可解决跨业务群/跨企业共享需求，可用于接入各领域中台的能力，由其提供统一规范的共享能力支撑。并且开放平台仅为一层比较“薄”的共享能力封装，它也可以接入非中台化的后台系统或是二方/三方的服务，也就是说 如果只为实现企业数字化能力共享的话可以没有中台，企业现有的IT系统都可以直接注册到开放平台，这在一定程度规避了中台建设非渐进式，投入高、风险大的问题。";
        let encrypt_body = TardisCryptoSm4 {}.encrypt_cbc(&mock_resp_body, &sm4_key, &sm4_iv).unwrap();
        // 4. Digest response body by service sm3
        let sign_body = crypto::sm::digest(&encrypt_body).unwrap();
        // 5. Encrypt response key by frontend public key
        let key = crypto::base64::encode(&fd_pub_key.encrypt(&format!("{sign_body} {sm4_key} {sm4_iv}")).unwrap());
        // ------------------------------------------------

        assert_eq!(decrypt(&encrypt_body, HashMap::from([(BIOS_CRYPTO.to_string(), key)])).unwrap(), mock_resp_body);
    }

    fn test_crypto_req(mock_serv_pri_key: &TardisCryptoSm2PrivateKey) {
        // Encrypt
        let mock_req_body = "中台经过几年“滚雪球”的发展或是资本地运作，已是个“庞然大物”，是到了“减肥”，“减负”的时候。一言以避之：解构中台，让他融合到更大的IT能力共享架构中，把共享交给开放平台，把技术还给技术平台，让中台专注于领域服务及事件 。";
        let encrypt_req = encrypt("Get", "iam/ct/req/xxx", mock_req_body).unwrap();
        let encrypt_body = encrypt_req.body;
        let key = &encrypt_req.additional_headers[BIOS_CRYPTO];

        // Mock serv process
        // ------------------------------------------------
        // 1. Decrypt request key by service private key
        let key = mock_serv_pri_key.decrypt(&crypto::base64::decode(key).unwrap()).unwrap();
        let key = key.split(" ").collect::<Vec<&str>>();
        assert_eq!(key.len(), 3);
        let sm3_digest = key[0];
        let sm4_key = key[1];
        let sm4_iv = key[2];
        // 2. Check request body digest by frontend sm3
        assert_eq!(crypto::sm::digest(&encrypt_body).unwrap(), sm3_digest);
        // 3. Decrypt request body by frontend sm4 key & iv
        assert_eq!(TardisCryptoSm4 {}.decrypt_cbc(&encrypt_body, sm4_key, sm4_iv).unwrap(), mock_req_body);
        // ------------------------------------------------
    }

    fn test_crypto_resp(mock_serv_pri_key: &TardisCryptoSm2PrivateKey, sm2: &TardisCryptoSm2) {
        // Encrypt
        let mock_req_body = "中台经过几年“滚雪球”的发展或是资本地运作，已是个“庞然大物”，是到了“减肥”，“减负”的时候。一言以避之：解构中台，让他融合到更大的IT能力共享架构中，把共享交给开放平台，把技术还给技术平台，让中台专注于领域服务及事件 。";
        let encrypt_req = encrypt("post", "iam/ct/resp/xxx", mock_req_body).unwrap();
        assert_eq!(encrypt_req.body, mock_req_body);
        let key = &encrypt_req.additional_headers[BIOS_CRYPTO];

        // Mock serv process
        // ------------------------------------------------
        // 1. Decrypt request key by service private key
        let fd_pub_key = mock_serv_pri_key.decrypt(&crypto::base64::decode(key).unwrap()).unwrap();
        let fd_pub_key = sm2.new_public_key_from_public_key(&fd_pub_key).unwrap();
        // 2. Encrypt response body by service sm4 key & iv
        let sm4_key = crypto::key::rand_16_hex().unwrap();
        let sm4_iv = crypto::key::rand_16_hex().unwrap();
        let mock_resp_body = "开放平台 天然具备了开放、生态化的能力，可解决跨业务群/跨企业共享需求，可用于接入各领域中台的能力，由其提供统一规范的共享能力支撑。并且开放平台仅为一层比较“薄”的共享能力封装，它也可以接入非中台化的后台系统或是二方/三方的服务，也就是说 如果只为实现企业数字化能力共享的话可以没有中台，企业现有的IT系统都可以直接注册到开放平台，这在一定程度规避了中台建设非渐进式，投入高、风险大的问题。";
        let encrypt_body = TardisCryptoSm4 {}.encrypt_cbc(&mock_resp_body, &sm4_key, &sm4_iv).unwrap();
        // 3. Digest response body by service sm3
        let sign_body = crypto::sm::digest(&encrypt_body).unwrap();
        // 4. Encrypt response key by frontend public key
        let key = crypto::base64::encode(&fd_pub_key.encrypt(&format!("{sign_body} {sm4_key} {sm4_iv}")).unwrap());
        // ------------------------------------------------

        assert_eq!(decrypt(&encrypt_body, HashMap::from([(BIOS_CRYPTO.to_string(), key)])).unwrap(), mock_resp_body);
    }

    fn test_crypto_none() {
        // Encrypt
        let mock_req_body = "中台经过几年“滚雪球”的发展或是资本地运作，已是个“庞然大物”，是到了“减肥”，“减负”的时候。一言以避之：解构中台，让他融合到更大的IT能力共享架构中，把共享交给开放平台，把技术还给技术平台，让中台专注于领域服务及事件 。";
        let encrypt_req = encrypt("delete", "iam/ct/resp/xxx", mock_req_body).unwrap();
        assert_eq!(encrypt_req.body, mock_req_body);
        assert!(encrypt_req.additional_headers.is_empty());
    }
}
