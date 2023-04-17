use std::collections::HashMap;

use bios_auth::{
    auth_config::AuthConfig,
    auth_constants::DOMAIN_CODE,
    dto::{
        auth_crypto_dto::{AuthEncryptReq, AuthEncryptResp},
        auth_kernel_dto::{AuthReq, AuthResp, MixRequest},
    },
};
use tardis::{
    basic::result::TardisResult,
    chrono,
    crypto::crypto_sm2_4::TardisCryptoSm2,
    log::info,
    web::{web_client::TardisWebClient, web_resp::TardisResp},
    TardisFuns,
};
use tardis::{
    crypto::crypto_sm2_4::{TardisCryptoSm2PrivateKey, TardisCryptoSm2PublicKey},
    serde_json::json,
};

fn crypto_req(body: &str, serv_pub_key: &str, front_pub_key: &str) -> (String, String) {
    let pub_key = TardisFuns::crypto.sm2.new_public_key_from_public_key(&serv_pub_key).unwrap();

    let sm4_key = TardisFuns::crypto.key.rand_16_hex().unwrap();
    let sm4_iv = TardisFuns::crypto.key.rand_16_hex().unwrap();

    let data = TardisFuns::crypto.sm4.encrypt_cbc(body, &sm4_key, &sm4_iv).unwrap();
    let sign_data = TardisFuns::crypto.digest.sm3(&data).unwrap();

    let sm4_encrypt = pub_key.encrypt(&format!("{sign_data} {sm4_key} {sm4_iv} {front_pub_key}",)).unwrap();
    let base64_encrypt = TardisFuns::crypto.base64.encode(&sm4_encrypt);
    (data, base64_encrypt)
}

async fn mock_req(method: &str, path: &str, query: &str, body: &str, mut headers: Vec<(&str, &str)>, serv_pub_key: &str, front_pub_key: &str, need_crypto_req: bool) -> AuthResp {
    let web_client = TardisWebClient::init(1).unwrap();
    info!(">>>>[Request]| path:{}, query:{}, headers:{:#?}", path, query, headers);
    let config = TardisFuns::cs_config::<AuthConfig>(DOMAIN_CODE);
    let encrypt;
    let data = if need_crypto_req {
        let (data, base64_encrypt) = crypto_req(body, serv_pub_key, front_pub_key);
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
            &format!("https://localhost:8080/{DOMAIN_CODE}/auth"),
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

pub async fn mock_req_mix_apis(method: &str, uri: &str, body: &str, mut headers: Vec<(&str, &str)>, serv_pub_key: &str, front_pub_key: &str) -> AuthResp {
    let web_client = TardisWebClient::init(1).unwrap();
    let config = TardisFuns::cs_config::<AuthConfig>(DOMAIN_CODE);
    let mix_body = json!({
        "method": method,
        "uri": uri,
        "body": body,
        "headers": headers.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect::<HashMap<String, String>>(),
        "ts": chrono::Utc::now(),
    });
    info!(">>>>[Mix_Request]| method:{}, uri:{},body:{},headers:{:#?}", method, uri, mix_body, headers);
    let mix_body = TardisFuns::json.obj_to_string(&mix_body).unwrap();
    let (data, base64_encrypt) = crypto_req(&mix_body.to_string(), serv_pub_key, front_pub_key);
    headers.push((&config.head_key_crypto, &base64_encrypt));

    let result: TardisResp<AuthResp> = web_client
        .post(
            &format!("https://localhost:8080/{DOMAIN_CODE}/auth/apis"),
            &MixRequest {
                headers: headers.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect::<HashMap<String, String>>(),
                body: data,
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
    let web_client = TardisWebClient::init(1).unwrap();
    info!(">>>>[Response]| headers:{:#?}", headers);
    let config = TardisFuns::cs_config::<AuthConfig>(DOMAIN_CODE);
    let result: TardisResp<AuthEncryptResp> = web_client
        .put(
            &format!("https://localhost:8080/{DOMAIN_CODE}/auth/crypto"),
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
    let decode_base64 = TardisFuns::crypto.base64.decode(result.headers.get(&config.head_key_crypto).unwrap()).unwrap();
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

    let web_client = TardisWebClient::init(1).unwrap();
    let result: TardisResp<String> = web_client.get(&format!("https://localhost:8080/{DOMAIN_CODE}/auth/crypto/key"), None).await.unwrap().body.unwrap();
    Ok((sm2.new_public_key_from_public_key(&result.data.unwrap()).unwrap(), pri_key, pub_key))
}

pub async fn test_encrypt() -> TardisResult<()> {
    let config = TardisFuns::cs_config::<AuthConfig>(DOMAIN_CODE);
    let sm2 = TardisCryptoSm2 {};
    let (serve_pub_key, front_pri_key, front_pub_key) = init_get_pub_key(&sm2).await.unwrap();

    let resp = mock_req(
        "POST",
        "/",
        "",
        "AAAA",
        vec![],
        serve_pub_key.serialize().unwrap().as_ref(),
        front_pub_key.serialize().unwrap().as_str(),
        true,
    )
    .await;
    assert!(!resp.allow);
    assert_eq!(resp.status_code, 400);
    assert_eq!(resp.reason.unwrap(), "[Auth] Request is not legal, missing [path]");
    let resp = mock_req(
        "POST",
        "/iam",
        "",
        "AAAA",
        vec![],
        serve_pub_key.serialize().unwrap().as_ref(),
        front_pub_key.serialize().unwrap().as_str(),
        true,
    )
    .await;
    assert!(resp.allow);
    assert_eq!(resp.body, Some("AAAA".to_string()));
    print!("{:?}", resp.headers);
    assert!(resp.headers.get(&config.head_key_crypto).is_some());

    let mock_body = r###"!@#$%^&*"()AZXdfds测试内容_~/n'//n/r/n'<>|\"###;
    let return_resp_body = mock_encrypt_resp(mock_body, resp.headers, &front_pri_key).await;
    assert_eq!(return_resp_body, mock_body);

    //todo
    let mix_req = mock_req_mix_apis(
        "PUT",
        "iam/cs/add/account",
        mock_body,
        vec![("test", "head1")],
        serve_pub_key.serialize().unwrap().as_ref(),
        front_pub_key.serialize().unwrap().as_ref(),
    )
    .await;
    print!("mix_req===={:?}", mix_req);
    assert_eq!(mix_req.body.unwrap(), mock_body);

    Ok(())
}
