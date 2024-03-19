use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use bios_auth::dto::auth_kernel_dto::MixAuthResp;
use bios_auth::{
    auth_config::AuthConfig,
    auth_constants::DOMAIN_CODE,
    dto::{
        auth_crypto_dto::{AuthEncryptReq, AuthEncryptResp},
        auth_kernel_dto::{AuthReq, AuthResp},
    },
};
use tardis::config::config_dto::WebClientModuleConfig;
use tardis::{
    basic::result::TardisResult,
    crypto::crypto_sm2_4::TardisCryptoSm2,
    log::info,
    web::{web_client::TardisWebClient, web_resp::TardisResp},
    TardisFuns,
};
use tardis::{
    crypto::crypto_sm2_4::{TardisCryptoSm2PrivateKey, TardisCryptoSm2PublicKey},
    serde_json::json,
};

fn crypto_req(body: &str, serv_pub_key: &str, front_pub_key: &str, need_crypto_resp: bool) -> (String, String) {
    let pub_key = TardisFuns::crypto.sm2.new_public_key_from_public_key(serv_pub_key).unwrap();

    let sm4_key = TardisFuns::crypto.key.rand_16_hex();
    let sm4_iv = TardisFuns::crypto.key.rand_16_hex();

    let data = TardisFuns::crypto.sm4.encrypt_cbc(body, &sm4_key, &sm4_iv).unwrap();
    let sign_data = TardisFuns::crypto.digest.sm3(&data).unwrap();

    let sm4_encrypt = if need_crypto_resp {
        pub_key.encrypt(&format!("{sign_data} {sm4_key} {sm4_iv} {front_pub_key}",)).unwrap()
    } else {
        pub_key.encrypt(&format!("{sign_data} {sm4_key} {sm4_iv}",)).unwrap()
    };
    let base64_encrypt = TardisFuns::crypto.base64.encode(sm4_encrypt);
    (data, base64_encrypt)
}

const WEB_CLIENT_CFG: WebClientModuleConfig = WebClientModuleConfig {
    connect_timeout_sec: 1,
    request_timeout_sec: 60,
};

async fn mock_req(
    method: &str,
    path: &str,
    query: &str,
    body: &str,
    mut headers: Vec<(&str, &str)>,
    serv_pub_key: &str,
    front_pub_key: &str,
    need_crypto_req: bool,
    need_crypto_resp: bool,
) -> AuthResp {
    let web_client = TardisWebClient::init(&WEB_CLIENT_CFG).unwrap();
    info!(">>>>[Request]| path:{}, query:{}, headers:{:#?}", path, query, headers);
    let config = TardisFuns::cs_config::<AuthConfig>(DOMAIN_CODE);
    let encrypt;
    let data = if need_crypto_req {
        let (data, base64_encrypt) = crypto_req(body, serv_pub_key, front_pub_key, need_crypto_resp);
        encrypt = base64_encrypt;
        headers.push((&config.head_key_crypto, &encrypt));
        data
    } else {
        body.to_string()
    };
    let hashmap_query = if query.is_empty() {
        HashMap::new()
    } else {
        query
            .split('&')
            .map(|a| {
                let split: Vec<_> = a.split('=').collect();
                (split[0].to_string(), split[1].to_string())
            })
            .collect::<HashMap<_, _>>()
    };
    let result: TardisResp<AuthResp> = web_client
        .put(
            &format!("https://127.0.0.1:8080/{DOMAIN_CODE}/auth"),
            &AuthReq {
                scheme: "http".to_string(),
                path: path.to_string(),
                query: hashmap_query,
                method: method.to_string(),
                host: "".to_string(),
                port: 80,
                headers: headers.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect::<HashMap<String, String>>(),
                body: Some(data),
            },
            None,
        )
        .await
        .unwrap()
        .body
        .unwrap();
    info!("<<<<[Request]|path:{}, query:{}, headers:{:#?}, result:{:#?}", path, query, headers, result);
    result.data.unwrap()
}

pub async fn mock_req_mix_apis(method: &str, uri: &str, body: &str, mut headers: Vec<(&str, &str)>, serv_pub_key: &str, front_pub_key: &str) -> MixAuthResp {
    let web_client = TardisWebClient::init(&WEB_CLIENT_CFG).unwrap();
    let config = TardisFuns::cs_config::<AuthConfig>(DOMAIN_CODE);
    let mix_body = json!({
        "method": method,
        "uri": uri,
        "body": body,
        "headers": headers.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect::<HashMap<String, String>>(),
        "ts":SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
    });
    info!(">>>>[Mix_Request]| method:{}, uri:{},body:{},headers:{:#?}", method, uri, mix_body, headers);
    let mix_body = TardisFuns::json.obj_to_string(&mix_body).unwrap();
    let (data, base64_encrypt) = crypto_req(&mix_body.to_string(), serv_pub_key, front_pub_key, true);
    headers.push((&config.head_key_crypto, &base64_encrypt));
    let url: Vec<&str> = uri.split('?').collect();
    let hashmap_query = if url.len() != 2 {
        HashMap::new()
    } else {
        url[1]
            .split('&')
            .map(|a| {
                let split: Vec<_> = a.split('=').collect();
                (split[0].to_string(), split[1].to_string())
            })
            .collect::<HashMap<_, _>>()
    };
    let result: TardisResp<MixAuthResp> = web_client
        .put(
            &format!("https://127.0.0.1:8080/{DOMAIN_CODE}/auth/apis"),
            &AuthReq {
                scheme: "http".to_string(),
                path: url[0].to_string(),
                query: hashmap_query,
                method: method.to_string(),
                host: "".to_string(),
                port: 80,
                headers: headers.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect::<HashMap<String, String>>(),
                body: Some(data),
            },
            None,
        )
        .await
        .unwrap()
        .body
        .unwrap();
    info!("<<<<[Mix_Request]| headers:{:#?}, result:{:#?}", headers, result);
    result.data.unwrap()
}

async fn mock_encrypt_resp(body: &str, headers: HashMap<String, String>, front_pri_key: &TardisCryptoSm2PrivateKey) -> String {
    let web_client = TardisWebClient::init(&WEB_CLIENT_CFG).unwrap();
    info!(">>>>[Response]| headers:{:#?}", headers);
    let config = TardisFuns::cs_config::<AuthConfig>(DOMAIN_CODE);
    let result: TardisResp<AuthEncryptResp> = web_client
        .put(
            &format!("https://127.0.0.1:8080/{DOMAIN_CODE}/auth/crypto"),
            &AuthEncryptReq {
                headers: headers.clone(),
                body: body.to_string(),
            },
            None,
        )
        .await
        .unwrap()
        .body
        .unwrap();
    info!("<<<<[Response]| headers:{:#?}, result:{:#?}", headers, result);
    let result = result.data.unwrap();
    let decode_base64 = TardisFuns::crypto.base64.decode_to_string(result.headers.get(&config.head_key_crypto).unwrap()).unwrap();
    let decrypt_key = front_pri_key.decrypt(&decode_base64).unwrap();
    let splits: Vec<_> = decrypt_key.split(' ').collect();
    if splits.len() != 3 {
        panic!("splits:{:?}", splits);
    }

    let sign_data = splits[0];
    let sm4_key = splits[1];
    let sm4_iv = splits[2];
    let gen_sign_data = TardisFuns::crypto.digest.sm3(&result.body).unwrap();
    assert_eq!(sign_data, gen_sign_data);
    TardisFuns::crypto.sm4.decrypt_cbc(&result.body, sm4_key, sm4_iv).unwrap()
}

async fn init_get_pub_key(sm2: &TardisCryptoSm2) -> TardisResult<(TardisCryptoSm2PublicKey, TardisCryptoSm2PrivateKey, TardisCryptoSm2PublicKey)> {
    //frontend init sm2
    let pri_key = TardisFuns::crypto.sm2.new_private_key().unwrap();
    let pub_key = TardisFuns::crypto.sm2.new_public_key(&pri_key).unwrap();

    let web_client = TardisWebClient::init(&WEB_CLIENT_CFG).unwrap();
    let result: TardisResp<String> = web_client.get(&format!("https://127.0.0.1:8080/{DOMAIN_CODE}/auth/crypto/key"), None).await.unwrap().body.unwrap();
    Ok((sm2.new_public_key_from_public_key(&result.data.unwrap()).unwrap(), pri_key, pub_key))
}

pub async fn test_encrypt() -> TardisResult<()> {
    let config = TardisFuns::cs_config::<AuthConfig>(DOMAIN_CODE);
    let sm2 = TardisCryptoSm2 {};
    let (serve_pub_key, front_pri_key, front_pub_key) = init_get_pub_key(&sm2).await.unwrap();

    info!("【test crypto request】");
    let resp = mock_req(
        "POST",
        "/",
        "",
        "AAAA",
        vec![],
        serve_pub_key.serialize().unwrap().as_ref(),
        front_pub_key.serialize().unwrap().as_str(),
        true,
        config.default_resp_crypto,
    )
    .await;
    assert!(!resp.allow);
    assert_eq!(resp.status_code, 400);
    assert_eq!(resp.reason.unwrap(), "[Auth] Request is not legal, missing [path]");

    let mock_body = r#"!@#$%^&*"()AZXdfds测试内容_~/n'//n/r/n'<>|\"#;
    let mock_resp_body = r#"!@#$%^&*"()AZXdfds测试内容_~/n'//n/r/n'<>|\内容内容"#;

    let resp = mock_req(
        "POST",
        "/iam",
        "",
        mock_body,
        vec![],
        serve_pub_key.serialize().unwrap().as_ref(),
        front_pub_key.serialize().unwrap().as_str(),
        true,
        config.default_resp_crypto,
    )
    .await;
    assert!(resp.allow);
    assert_eq!(resp.body, Some(mock_body.to_string()));
    assert!(if config.default_resp_crypto {
        resp.headers.get(&config.head_key_crypto).is_some()
    } else {
        resp.headers.get(&config.head_key_crypto).is_none()
    });

    if config.default_resp_crypto {
        let return_resp_body = mock_encrypt_resp(mock_body, resp.headers, &front_pri_key).await;
        assert_eq!(return_resp_body, mock_body);
    }

    let resp = mock_req(
        "GET",
        "/iam/apis",
        "",
        "",
        vec![],
        serve_pub_key.serialize().unwrap().as_ref(),
        front_pub_key.serialize().unwrap().as_str(),
        true,
        config.default_resp_crypto,
    )
    .await;
    assert!(resp.allow);
    assert_eq!(resp.body, Some("".to_string()));
    assert!(if config.default_resp_crypto {
        resp.headers.get(&config.head_key_crypto).is_some()
    } else {
        resp.headers.get(&config.head_key_crypto).is_none()
    });

    if config.default_resp_crypto {
        let return_resp_body = mock_encrypt_resp(mock_body, resp.headers, &front_pri_key).await;
        assert_eq!(return_resp_body, mock_body);
    }

    let resp = mock_req(
        "POST",
        "/iam",
        "",
        mock_body,
        vec![],
        serve_pub_key.serialize().unwrap().as_ref(),
        front_pub_key.serialize().unwrap().as_str(),
        true,
        config.default_resp_crypto,
    )
    .await;
    assert!(resp.allow);
    assert_eq!(resp.body, Some(mock_body.to_string()));
    assert!(if config.default_resp_crypto {
        resp.headers.get(&config.head_key_crypto).is_some()
    } else {
        resp.headers.get(&config.head_key_crypto).is_none()
    });

    if config.default_resp_crypto {
        let return_resp_body = mock_encrypt_resp(mock_body, resp.headers, &front_pri_key).await;
        assert_eq!(return_resp_body, mock_body);
    }

    info!("【test mix apis】");

    let mix_req = mock_req_mix_apis(
        "PUT",
        "/",
        mock_body,
        vec![("test", "head1")],
        serve_pub_key.serialize().unwrap().as_ref(),
        front_pub_key.serialize().unwrap().as_ref(),
    )
    .await;
    assert_eq!(mix_req.body, None);
    assert!(!mix_req.allow);

    let true_url1 = "/iam/cs/add/account";
    let mix_req = mock_req_mix_apis(
        "PUT",
        true_url1,
        mock_body,
        vec![("test", "head1")],
        serve_pub_key.serialize().unwrap().as_ref(),
        front_pub_key.serialize().unwrap().as_ref(),
    )
    .await;
    assert_eq!(mix_req.body.unwrap(), mock_body);
    assert_eq!(mix_req.url, true_url1);

    let return_resp_body = mock_encrypt_resp(mock_resp_body, mix_req.headers, &front_pri_key).await;
    assert_eq!(return_resp_body, mock_resp_body);

    let true_url2 = "iam/cs/add/account?p1=a1&p1=a2";
    let mix_req = mock_req_mix_apis(
        "PUT",
        true_url2,
        mock_body,
        vec![("test", "head1")],
        serve_pub_key.serialize().unwrap().as_ref(),
        front_pub_key.serialize().unwrap().as_ref(),
    )
    .await;
    assert_eq!(mix_req.body.unwrap(), mock_body);
    assert_eq!(mix_req.url, format!("/{}", true_url2));

    let return_resp_body = mock_encrypt_resp(mock_resp_body, mix_req.headers, &front_pri_key).await;
    assert_eq!(return_resp_body, mock_resp_body);

    Ok(())
}
