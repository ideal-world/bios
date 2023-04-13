use std::collections::HashMap;

use constants::{BIOS_TOKEN, STABLE_CONFIG};
use modules::global_api_process::MixRequest;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
mod constants;
mod initializer;
mod mini_tardis;
mod modules;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub async fn init(service_url: &str, config: JsValue) -> Result<(), JsValue> {
    let strict_security_mode = if config == JsValue::UNDEFINED {
        initializer::init(service_url, None).await?
    } else {
        initializer::init(service_url, Some(mini_tardis::serde::jsvalue_to_obj(config)?)).await?
    };
    if !strict_security_mode {
        console_error_panic_hook::set_once();
    }
    Ok(())
}

#[wasm_bindgen]
pub fn on_before_request(method: &str, uri: &str, body: JsValue, headers: JsValue) -> Result<JsValue, JsValue> {
    if modules::double_auth_process::need_auth(method, uri)? {
        return Err(JsValue::try_from(JsError::new(&format!("Need double auth."))).unwrap());
    }
    let body = mini_tardis::serde::jsvalue_to_str(&body)?;
    let mut headers = mini_tardis::serde::jsvalue_to_obj::<HashMap<String, String>>(headers)?;
    if let Some(token) = modules::token_process::get_token()? {
        headers.insert(BIOS_TOKEN.to_string(), token);
    }
    let mix_req = if constants::get_strict_security_mode()? {
        modules::global_api_process::mix(method, uri, &body, headers)?
    } else {
        let resp = modules::crypto_process::encrypt(method, uri, &body)?;
        headers.extend(resp.additional_headers);
        MixRequest {
            method: method.to_string(),
            uri: uri.to_string(),
            body: resp.body,
            headers,
        }
    };
    Ok(mini_tardis::serde::obj_to_jsvalue(&mix_req)?)
}

#[wasm_bindgen]
pub fn on_before_response(body: JsValue, headers: JsValue) -> Result<String, JsValue> {
    let body = mini_tardis::serde::jsvalue_to_str(&body)?;
    let headers = mini_tardis::serde::jsvalue_to_obj(headers)?;
    Ok(modules::crypto_process::decrypt(&body, headers)?)
}

#[wasm_bindgen]
pub fn on_response_success(method: &str, uri: &str, body: JsValue) -> Result<(), JsValue> {
    let uri = if uri.starts_with("/") { uri.to_string() } else { format!("/{uri}") };
    let spec_opt = {
        let config = STABLE_CONFIG.read().unwrap();
        let config = config.as_ref().unwrap();
        if config.login_req_method.to_lowercase() == method.to_lowercase() && config.login_req_paths.iter().any(|u| uri.starts_with(u)) {
            1
        } else if config.logout_req_method.to_lowercase() == method.to_lowercase() && uri.starts_with(&config.logout_req_path) {
            2
        } else if config.double_auth_req_method.to_lowercase() == method.to_lowercase() && uri.starts_with(&config.double_auth_req_path) {
            3
        } else {
            0
        }
    };
    match spec_opt {
        1 => {
            if let Ok(body) = js_sys::Reflect::get(&body, &"token".into()) {
                let token = body.as_string().unwrap();
                modules::token_process::set_token(&token)?;
            } else {
                return Err(JsValue::try_from(JsError::new(&format!("Body format error."))).unwrap());
            }
        }
        2 => {
            modules::token_process::remove_token()?;
        }
        3 => {
            modules::double_auth_process::set_latest_authed()?;
        }
        _ => {}
    }
    Ok(())
}

#[wasm_bindgen]
pub fn encrypt(text: &str) -> Result<String, JsValue> {
    Ok(modules::crypto_process::simple_encrypt(text)?)
}

#[wasm_bindgen]
pub fn decrypt(encrypt_text: &str) -> Result<String, JsValue> {
    Ok(modules::crypto_process::simple_decrypt(encrypt_text)?)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::{
        constants::{BIOS_CRYPTO, BIOS_TOKEN},
        decrypt, encrypt,
        initializer::{self, Api, ServConfig},
        mini_tardis::{
            self,
            crypto::{
                self,
                sm::{TardisCryptoSm2, TardisCryptoSm4},
            },
        },
        modules::global_api_process::MixRequest,
        on_before_request, on_before_response, on_response_success,
    };

    #[wasm_bindgen_test]
    fn test_non_strict_security_mode() {
        // Prepare
        let sm2 = TardisCryptoSm2 {};
        let mock_serv_pri_key = sm2.new_private_key().unwrap();
        let mock_serv_pub_key = sm2.new_public_key(&mock_serv_pri_key).unwrap();
        initializer::do_init(
            "",
            &ServConfig {
                strict_security_mode: false,
                pub_key: mock_serv_pub_key.serialize().unwrap(),
                double_auth_exp_sec: 1,
                apis: vec![
                    Api {
                        action: "get".to_string(),
                        uri: "iam/ct/**".to_string(),
                        need_crypto_req: true,
                        need_crypto_resp: true,
                        need_double_auth: false,
                    },
                    Api {
                        action: "get".to_string(),
                        uri: "iam/cs/**".to_string(),
                        need_crypto_req: true,
                        need_crypto_resp: true,
                        need_double_auth: true,
                    },
                ],
                login_req_method: "put".to_string(),
                login_req_paths: vec!["/iam/cp/login/userpwd".to_string()],
                logout_req_method: "delete".to_string(),
                logout_req_path: "/iam/cp/logout/".to_string(),
                double_auth_req_method: "put".to_string(),
                double_auth_req_path: "/iam/cp/login/check".to_string(),
            },
        )
        .unwrap();

        // test url crypto
        let encrypt_uri = encrypt("iam/ct/all/1/2/3?q=测试").unwrap();
        assert_eq!(decrypt(&encrypt_uri).unwrap(), "iam/ct/all/1/2/3?q=测试");

        // =========================================
        // test simple request
        let mock_req_body = "中台经过几年“滚雪球”的发展或是资本地运作，已是个“庞然大物”，是到了“减肥”，“减负”的时候。一言以避之：解构中台，让他融合到更大的IT能力共享架构中，把共享交给开放平台，把技术还给技术平台，让中台专注于领域服务及事件 。";
        let mix_req = on_before_request(
            "get",
            "/iam/ct/xxx",
            mini_tardis::serde::str_to_jsvalue(mock_req_body).unwrap(),
            mini_tardis::serde::obj_to_jsvalue(&HashMap::from([("X-Test".to_string(), "测试头信息".to_string())])).unwrap(),
        )
        .unwrap();
        let mix_req = mini_tardis::serde::jsvalue_to_obj::<MixRequest>(mix_req).unwrap();
        assert_eq!(mix_req.method, "get");
        assert_eq!(mix_req.uri, "/iam/ct/xxx");
        assert_eq!(mix_req.headers.len(), 2);
        assert!(mix_req.body != mock_req_body);

        // Mock serv process
        // ------------------------------------------------
        // 1. Decrypt request key by service private key
        let key = &mix_req.headers[BIOS_CRYPTO];
        let key = mock_serv_pri_key.decrypt(&crypto::base64::decode(key).unwrap()).unwrap();
        let key = key.split(" ").collect::<Vec<&str>>();
        assert_eq!(key.len(), 4);
        let sm3_digest = key[0];
        let sm4_key = key[1];
        let sm4_iv = key[2];
        // 2. Check request body digest by frontend sm3
        assert_eq!(crypto::sm::digest(&mix_req.body).unwrap(), sm3_digest);
        let fd_pub_key = sm2.new_public_key_from_public_key(key[3]).unwrap();
        // 3. Decrypt request body by frontend sm4 key & iv
        assert_eq!(TardisCryptoSm4 {}.decrypt_cbc(&mix_req.body, sm4_key, sm4_iv).unwrap(), mock_req_body);
        // 4. Encrypt response body by service sm4 key & iv
        let sm4_key = crypto::key::rand_16_hex().unwrap();
        let sm4_iv = crypto::key::rand_16_hex().unwrap();
        let mock_resp_body = "开放平台 天然具备了开放、生态化的能力，可解决跨业务群/跨企业共享需求，可用于接入各领域中台的能力，由其提供统一规范的共享能力支撑。并且开放平台仅为一层比较“薄”的共享能力封装，它也可以接入非中台化的后台系统或是二方/三方的服务，也就是说 如果只为实现企业数字化能力共享的话可以没有中台，企业现有的IT系统都可以直接注册到开放平台，这在一定程度规避了中台建设非渐进式，投入高、风险大的问题。";
        let encrypt_body = TardisCryptoSm4 {}.encrypt_cbc(&mock_resp_body, &sm4_key, &sm4_iv).unwrap();
        // 4. Digest response body by service sm3
        let sign_body = crypto::sm::digest(&encrypt_body).unwrap();
        // 5. Encrypt response key by frontend public key
        let key = crypto::base64::encode(&fd_pub_key.encrypt(&format!("{sign_body} {sm4_key} {sm4_iv}")).unwrap());
        let headers = HashMap::from([{ (BIOS_CRYPTO.to_string(), key) }]);
        // ------------------------------------------------

        let resp_body = on_before_response(
            mini_tardis::serde::str_to_jsvalue(&encrypt_body).unwrap(),
            mini_tardis::serde::obj_to_jsvalue(&headers).unwrap(),
        )
        .unwrap();
        assert_eq!(resp_body, mock_resp_body);

        // =========================================
        // test token
        on_response_success("put", "/iam/cp/login/userpwd", mini_tardis::serde::str_to_jsvalue(r#"{"token":"t001"}"#).unwrap()).unwrap();
        let mix_req = on_before_request(
            "get",
            "/iam/ct/xxx",
            mini_tardis::serde::str_to_jsvalue(mock_req_body).unwrap(),
            mini_tardis::serde::obj_to_jsvalue(&HashMap::from([("X-Test".to_string(), "测试头信息".to_string())])).unwrap(),
        )
        .unwrap();
        let mix_req = mini_tardis::serde::jsvalue_to_obj::<MixRequest>(mix_req).unwrap();
        assert_eq!(mix_req.method, "get");
        assert_eq!(mix_req.uri, "/iam/ct/xxx");
        assert_eq!(mix_req.headers.len(), 3);
        assert_eq!(mix_req.headers[BIOS_TOKEN], "t001");
        assert!(mix_req.body != mock_req_body);

        // =========================================
        // test double auth
        let mix_req = on_before_request(
            "get",
            "/iam/cs/xxx",
            mini_tardis::serde::str_to_jsvalue(mock_req_body).unwrap(),
            mini_tardis::serde::obj_to_jsvalue(&HashMap::from([("X-Test".to_string(), "测试头信息".to_string())])).unwrap(),
        );
        assert!(mix_req.is_err());
        on_response_success("put", "/iam/cp/login/check", mini_tardis::serde::str_to_jsvalue(r#""#).unwrap()).unwrap();
        let mix_req = on_before_request(
            "get",
            "/iam/cs/xxx",
            mini_tardis::serde::str_to_jsvalue(mock_req_body).unwrap(),
            mini_tardis::serde::obj_to_jsvalue(&HashMap::from([("X-Test".to_string(), "测试头信息".to_string())])).unwrap(),
        )
        .unwrap();
        let mix_req = mini_tardis::serde::jsvalue_to_obj::<MixRequest>(mix_req).unwrap();
        assert_eq!(mix_req.method, "get");
        assert_eq!(mix_req.uri, "/iam/cs/xxx");
        assert_eq!(mix_req.headers.len(), 3);
        assert_eq!(mix_req.headers[BIOS_TOKEN], "t001");
        assert!(mix_req.body != mock_req_body);
    }

    #[wasm_bindgen_test]
    fn test_strict_security_mode() {
        // Prepare
        let sm2 = TardisCryptoSm2 {};
        let mock_serv_pri_key = sm2.new_private_key().unwrap();
        let mock_serv_pub_key = sm2.new_public_key(&mock_serv_pri_key).unwrap();
        initializer::do_init(
            "",
            &ServConfig {
                strict_security_mode: true,
                pub_key: mock_serv_pub_key.serialize().unwrap(),
                double_auth_exp_sec: 1,
                apis: vec![Api {
                    action: "get".to_string(),
                    uri: "iam/ct/**".to_string(),
                    need_crypto_req: true,
                    need_crypto_resp: true,
                    need_double_auth: false,
                }],
                login_req_method: "put".to_string(),
                login_req_paths: vec!["/iam/cp/login/userpwd".to_string()],
                logout_req_method: "delete".to_string(),
                logout_req_path: "/iam/cp/logout/".to_string(),
                double_auth_req_method: "put".to_string(),
                double_auth_req_path: "/iam/cp/login/check".to_string(),
            },
        )
        .unwrap();

        let mock_req_body = "中台经过几年“滚雪球”的发展或是资本地运作，已是个“庞然大物”，是到了“减肥”，“减负”的时候。一言以避之：解构中台，让他融合到更大的IT能力共享架构中，把共享交给开放平台，把技术还给技术平台，让中台专注于领域服务及事件 。";
        let mix_req = on_before_request(
            "get",
            "/iam/ct/xxx",
            mini_tardis::serde::str_to_jsvalue(mock_req_body).unwrap(),
            mini_tardis::serde::obj_to_jsvalue(&HashMap::from([("X-Test".to_string(), "测试头信息".to_string())])).unwrap(),
        )
        .unwrap();
        let mix_req = mini_tardis::serde::jsvalue_to_obj::<MixRequest>(mix_req).unwrap();
        assert_eq!(mix_req.method, "POST");
        assert_eq!(mix_req.uri, "apis");
        assert_eq!(mix_req.headers.len(), 2);

        // Mock serv process
        // ------------------------------------------------
        // 1. Decrypt request key by service private key
        let key = mix_req.headers.get(BIOS_CRYPTO).unwrap();
        let encrypt_body = mix_req.body;
        let key = mock_serv_pri_key.decrypt(&crypto::base64::decode(key).unwrap()).unwrap();
        let key = key.split(" ").collect::<Vec<&str>>();
        assert_eq!(key.len(), 4);
        let sm4_key = key[1];
        let sm4_iv = key[2];
        sm2.new_public_key_from_public_key(key[3]).unwrap();
        // 2. Decrypt request body by frontend sm4 key & iv
        let mix_req = serde_json::from_str::<MixRequest>(&TardisCryptoSm4 {}.decrypt_cbc(&encrypt_body, sm4_key, sm4_iv).unwrap()).unwrap();
        assert_eq!(mix_req.method, "PUT");
        assert_eq!(mix_req.uri, "iam/ct/xxx");
        assert_eq!(mix_req.headers.len(), 1);
        assert_eq!(mix_req.headers["Test-Key"], "xxx");
        assert_eq!(mix_req.body, mock_req_body);
    }
}
