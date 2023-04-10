use std::collections::HashMap;

use bios_auth::{
    auth_config::AuthConfig,
    auth_constants::DOMAIN_CODE,
    dto::auth_kernel_dto::{AuthReq, AuthResp},
};
use tardis::chrono::{Duration, Utc};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    log::info,
    serde_json::Value,
    web::{web_client::TardisWebClient, web_resp::TardisResp},
    TardisFuns,
};

async fn mock_req(method: &str, path: &str, query: &str, body: &str, mut headers: Vec<(&str, &str)>, pub_key: &str, need_crypto_req: bool, need_crypto_resp: bool) -> AuthResp {
    let web_client = TardisWebClient::init(1).unwrap();
    info!(">>>>[Request]| path:{}, query:{}, headers:{:#?}", path, query, headers);
    let config = TardisFuns::cs_config::<AuthConfig>(DOMAIN_CODE);
    let data = if need_crypto_req {
        let pub_key = TardisFuns::crypto.sm2.new_public_key_from_public_key(&pub_key).unwrap();

        let sm4_key = TardisFuns::crypto.key.rand_16_hex().unwrap();
        let sm4_iv = TardisFuns::crypto.key.rand_16_hex().unwrap();

        let data = TardisFuns::crypto.sm4.encrypt_cbc(&body, &sm4_key, &sm4_iv).unwrap();

        // let pub_key = pub_key.encrypt(&format!("{sm4_key} {sm4_iv}")).unwrap();
        headers.push((&config.head_key_crypto, &TardisFuns::crypto.base64.encode(&format!("{sm4_key} {sm4_iv} {pub_key}"))));
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
    //todo need_crypto_resp
    result.data.unwrap()
}
async fn init_get_pub_key() -> TardisResult<String> {
    //frontend init sm2
    let pri_key = TardisFuns::crypto.sm2.new_private_key()?;
    let pub_key = TardisFuns::crypto.sm2.new_public_key(&pri_key)?;

    let web_client = TardisWebClient::init(1).unwrap();
    let result: TardisResp<String> = web_client.get(&format!("https://localhost:8080/{DOMAIN_CODE}/auth/crypto/key"), None).await.unwrap().body.unwrap();
    result.data.unwrap()
}

pub async fn test_encrypt() -> TardisResult<()> {}
