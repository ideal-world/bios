use std::collections::HashMap;

use bios_auth::{
    auth_config::AuthConfig,
    auth_constants::DOMAIN_CODE,
    dto::auth_dto::{AuthReq, AuthResp},
};
use serde::{Deserialize, Serialize};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    log::info,
    web::{web_client::TardisWebClient, web_resp::TardisResp},
    TardisFuns,
};

async fn mock_req(method: &str, path: &str, query: &str, headers: Vec<(&str, &str)>) -> AuthResp {
    #[derive(Serialize, Deserialize)]
    struct ApisixAuthReq {
        pub request: AuthReq,
    }
    let web_client = TardisWebClient::init(1).unwrap();
    info!(">>>>[Request]| path:{}, query:{}, headers:{:#?}", path, query, headers);
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
        .post(
            &format!("https://localhost:8080/{DOMAIN_CODE}/auth/apisix"),
            &AuthReq {
                scheme: "http".to_string(),
                path: path.to_string(),
                query: hashmap_query,
                method: method.to_string(),
                host: "".to_string(),
                port: 80,
                headers: headers.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect::<HashMap<String, String>>(),
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

fn decode_context(headers: &HashMap<String, String>) -> TardisContext {
    let config = TardisFuns::cs_config::<AuthConfig>(DOMAIN_CODE);
    let ctx = headers.get(&config.head_key_context).unwrap();
    let ctx = TardisFuns::crypto.base64.decode(ctx).unwrap();
    TardisFuns::json.str_to_obj(&ctx).unwrap()
}

pub async fn test_req() -> TardisResult<()> {
    let config = TardisFuns::cs_config::<AuthConfig>(DOMAIN_CODE);
    let cache_client = TardisFuns::cache_by_module_or_default(DOMAIN_CODE);

    // request is not legal, missing [domain] in path
    let resp = mock_req("GET", "/", "", vec![]).await;
    assert!(!resp.allow);
    assert_eq!(resp.status_code, 400);
    assert_eq!(resp.reason.unwrap(), "[Auth] Request is not legal, missing [path]");
    let resp = mock_req("GET", "/iam", "", vec![]).await;
    assert!(resp.allow);

    // token is not legal
    let resp = mock_req("GET", "/iam/cp/account", "", vec![("Bios-Token", "aaaa")]).await;
    assert!(!resp.allow);
    assert_eq!(resp.status_code, 401);
    assert_eq!(resp.reason.unwrap(), "[Auth] Token [aaaa] is not legal");

    // request public
    let resp = mock_req("POST", "/iam/cp/login", "p=xx", vec![]).await;
    assert!(resp.allow);
    assert_eq!(resp.status_code, 200);
    let ctx = decode_context(&resp.headers);
    assert_eq!(ctx.own_paths, "");
    assert_eq!(ctx.owner, "");
    assert!(ctx.roles.is_empty());
    assert!(ctx.groups.is_empty());

    // request token by system account
    cache_client.set(&format!("{}tokenxxx", config.cache_key_token_info), "default,accountxxx").await?;
    cache_client
        .hset(
            &format!("{}accountxxx", config.cache_key_account_info),
            "",
            "{\"own_paths\":\"\",\"owner\":\"account1\",\"roles\":[\"r001\"],\"groups\":[\"g001\"]}",
        )
        .await?;
    let resp = mock_req("GET", "/iam/api/p1", "bb=y&aa=x", vec![("Bios-Token", "tokenxxx")]).await;
    assert!(resp.allow);
    assert_eq!(resp.status_code, 200);
    let ctx = decode_context(&resp.headers);
    assert_eq!(ctx.own_paths, "");
    assert_eq!(ctx.owner, "account1");
    assert_eq!(ctx.roles, vec!["r001"]);
    assert_eq!(ctx.groups, vec!["g001"]);

    // request token by tenant account
    cache_client.set(&format!("{}tokenxxx", config.cache_key_token_info), "default,accountxxx").await?;
    cache_client
        .hset(
            &format!("{}accountxxx", config.cache_key_account_info),
            "",
            "{\"own_paths\":\"tenant1\",\"owner\":\"account1\",\"roles\":[\"r001\"],\"groups\":[\"g001\"]}",
        )
        .await?;
    let resp = mock_req("GET", "/iam/api/p1", "bb=y&aa=x", vec![("Bios-Token", "tokenxxx")]).await;
    assert!(resp.allow);
    assert_eq!(resp.status_code, 200);
    let ctx = decode_context(&resp.headers);
    assert_eq!(ctx.own_paths, "tenant1");
    assert_eq!(ctx.owner, "account1");
    assert_eq!(ctx.roles, vec!["r001"]);
    assert_eq!(ctx.groups, vec!["g001"]);

    // request token by app account
    cache_client.set(&format!("{}tokenxxx", config.cache_key_token_info), "default,accountxxx").await?;
    cache_client
        .hset(
            &format!("{}accountxxx", config.cache_key_account_info),
            "",
            "{\"own_paths\":\"tenant1\",\"owner\":\"account1\",\"roles\":[\"r001\"],\"groups\":[\"g001\"]}",
        )
        .await?;
    cache_client
        .hset(
            &format!("{}accountxxx", config.cache_key_account_info),
            "app1",
            "{\"own_paths\":\"tenant1/app1\",\"owner\":\"account1\",\"roles\":[\"r002\"],\"groups\":[\"g002\"]}",
        )
        .await?;
    let resp = mock_req("GET", "/iam/api/p1", "bb=y&aa=x", vec![("Bios-Token", "tokenxxx"), ("Bios-App", "app2")]).await;
    assert!(!resp.allow);
    assert_eq!(resp.status_code, 401);
    assert_eq!(resp.reason.unwrap(), "[Auth] Token [tokenxxx] with App [app2] is not legal");
    cache_client.set(&format!("{}tokenxxx", config.cache_key_token_info), "default,accountxxx").await?;
    cache_client
        .hset(
            &format!("{}accountxxx", config.cache_key_account_info),
            "",
            "{\"own_paths\":\"tenant1\",\"owner\":\"account1\",\"roles\":[\"r001\"],\"groups\":[\"g001\"]}",
        )
        .await?;
    cache_client
        .hset(
            &format!("{}accountxxx", config.cache_key_account_info),
            "app1",
            "{\"own_paths\":\"tenant1/app1\",\"owner\":\"account1\",\"roles\":[\"r002\"],\"groups\":[\"g002\"]}",
        )
        .await?;
    let resp = mock_req("GET", "/iam/api/p1", "bb=y&aa=x", vec![("Bios-Token", "tokenxxx"), ("Bios-App", "app1")]).await;
    assert!(resp.allow);
    assert_eq!(resp.status_code, 200);
    let ctx = decode_context(&resp.headers);
    assert_eq!(ctx.own_paths, "tenant1/app1");
    assert_eq!(ctx.owner, "account1");
    assert_eq!(ctx.roles, vec!["r002", "r001"]);
    assert_eq!(ctx.groups, vec!["g002", "g001"]);

    Ok(())
}
