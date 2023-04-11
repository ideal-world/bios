use std::collections::HashMap;

use bios_auth::{
    auth_config::AuthConfig,
    auth_constants::DOMAIN_CODE,
    dto::auth_kernel_dto::{AuthReq, AuthResp},
};
use tardis::crypto::crypto_sm2_4::{TardisCryptoSm2PrivateKey, TardisCryptoSm2PublicKey};
use tardis::{
    basic::result::TardisResult,
    crypto::crypto_sm2_4::TardisCryptoSm2,
    log::info,
    web::{web_client::TardisWebClient, web_resp::TardisResp},
    TardisFuns,
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
    let web_client = TardisWebClient::init(1).unwrap();
    info!(">>>>[Request]| path:{}, query:{}, headers:{:#?}", path, query, headers);
    let base64_encrypt;
    let config = TardisFuns::cs_config::<AuthConfig>(DOMAIN_CODE);
    let data = if need_crypto_req {
        let pub_key = TardisFuns::crypto.sm2.new_public_key_from_public_key(&serv_pub_key).unwrap();

        let sm4_key = TardisFuns::crypto.key.rand_16_hex().unwrap();
        let sm4_iv = TardisFuns::crypto.key.rand_16_hex().unwrap();

        let data = TardisFuns::crypto.sm4.encrypt_cbc(body, &sm4_key, &sm4_iv).unwrap();
        let sign_data = TardisFuns::crypto.digest.sm3(&data).unwrap();

        let sm4_encrypt = pub_key.encrypt(&format!("{sign_data} {sm4_key} {sm4_iv} {front_pub_key}",)).unwrap();
        base64_encrypt = TardisFuns::crypto.base64.encode(&sm4_encrypt);
        headers.push((&config.head_key_crypto, base64_encrypt.as_str()));
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
async fn init_get_pub_key(sm2: &TardisCryptoSm2) -> TardisResult<(TardisCryptoSm2PublicKey, TardisCryptoSm2PrivateKey, TardisCryptoSm2PublicKey)> {
    //frontend init sm2
    let pri_key = TardisFuns::crypto.sm2.new_private_key().unwrap();
    let pub_key = TardisFuns::crypto.sm2.new_public_key(&pri_key).unwrap();

    let web_client = TardisWebClient::init(1).unwrap();
    let result: TardisResp<String> = web_client.get(&format!("https://localhost:8080/{DOMAIN_CODE}/auth/crypto/key"), None).await.unwrap().body.unwrap();
    Ok((sm2.new_public_key_from_public_key(&result.data.unwrap()).unwrap(), pri_key, pub_key))
}

pub async fn test_encrypt() -> TardisResult<()> {
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
        false,
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
        false,
    )
    .await;
    assert!(resp.allow);
    assert_eq!(resp.body, Some("AAAA".to_string()));

    //todo

    Ok(())
}
