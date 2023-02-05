use std::collections::HashMap;

use bios_auth::{
    auth_config::AuthConfig,
    auth_constants::DOMAIN_CODE,
    dto::auth_dto::{AuthReq, AuthResp},
};
use serde::{Deserialize, Serialize};
use tardis::chrono::Local;
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

    // missing header [Bios-Date]
    let resp = mock_req("GET", "/iam/ci/account", "", vec![(&config.head_key_ak_authorization, "aaaa")]).await;
    assert!(!resp.allow);
    assert_eq!(resp.status_code, 401);
    assert_eq!(resp.reason.unwrap(), "[Auth] Request is not legal, missing header [Bios-Date]");

    // [Auth] Ak-Authorization [aaaa] is not legal
    let now = Local::now().format("%Y-%m-%d %H:%M:%S");
    let now = now.to_string();
    let resp = mock_req(
        "GET",
        "/iam/ci/account",
        "",
        vec![(&config.head_key_ak_authorization, "aaaa"), (&config.head_key_date_flag, &now)],
    )
    .await;
    assert!(!resp.allow);
    assert_eq!(resp.status_code, 401);
    assert_eq!(resp.reason.unwrap(), "[Auth] Ak-Authorization [aaaa] is not legal");

    let sk = "bbbbbb";

    // is not legal
    let now = Local::now().format("%Y-%m-%d %H:%M:%S");
    let now = now.to_string();
    let calc_signature = TardisFuns::crypto.base64.encode(&TardisFuns::crypto.digest.hmac_sha256(&format!("GET\n{}\niam/ci/account\n", now,).to_lowercase(), sk)?);
    let resp = mock_req(
        "GET",
        "/iam/ci/account",
        "",
        vec![(&config.head_key_ak_authorization, &format!("aaaa:{}", calc_signature)), (&config.head_key_date_flag, &now)],
    )
    .await;
    assert!(!resp.allow);
    assert_eq!(resp.status_code, 401);
    assert_eq!(resp.reason.unwrap(), "[Auth] Ak [aaaa] is not legal");

    // 200
    let now = Local::now().format("%Y-%m-%d %H:%M:%S");
    let now = now.to_string();
    cache_client.set(&format!("{}aaaa", config.cache_key_aksk_info), &format!("{sk},tenant_id123,")).await?;
    let calc_signature = TardisFuns::crypto.base64.encode(&TardisFuns::crypto.digest.hmac_sha256(&format!("GET\n{}\niam/ci/account\n", now,).to_lowercase(), sk)?);
    let resp = mock_req(
        "GET",
        "/iam/ci/account",
        "",
        vec![(&config.head_key_ak_authorization, &format!("aaaa:{}", calc_signature)), (&config.head_key_date_flag, &now)],
    )
    .await;
    assert!(resp.allow);
    assert_eq!(resp.status_code, 200);

    // app_id not legal
    let now = Local::now().format("%Y-%m-%d %H:%M:%S");
    let now = now.to_string();
    let app_id = "app_idcc";
    cache_client.set(&format!("{}aaaa", config.cache_key_aksk_info), &format!("{sk},tenant_id123,")).await?;
    let calc_signature = TardisFuns::crypto.base64.encode(&TardisFuns::crypto.digest.hmac_sha256(&format!("GET\n{}\niam/ci/account\n", now,).to_lowercase(), sk)?);
    let resp = mock_req(
        "GET",
        "/iam/ci/account",
        "",
        vec![
            (&config.head_key_ak_authorization, &format!("aaaa:{}", calc_signature)),
            (&config.head_key_date_flag, &now),
            (&config.head_key_app, app_id),
        ],
    )
    .await;
    assert!(!resp.allow);
    assert_eq!(resp.status_code, 401);
    assert_eq!(resp.reason.unwrap(), "Ak [aaaa]  with App [app_idcc] is not legal");

    // app_id legal
    let now = Local::now().format("%Y-%m-%d %H:%M:%S");
    let now = now.to_string();
    let app_id = "app_idcc";
    cache_client.set(&format!("{}aaaa", config.cache_key_aksk_info), &format!("{sk},tenant_id123,{app_id}")).await?;
    let calc_signature = TardisFuns::crypto.base64.encode(&TardisFuns::crypto.digest.hmac_sha256(&format!("GET\n{}\niam/ci/account\n", now,).to_lowercase(), sk)?);
    let resp = mock_req(
        "GET",
        "/iam/ci/account",
        "",
        vec![
            (&config.head_key_ak_authorization, &format!("aaaa:{}", calc_signature)),
            (&config.head_key_date_flag, &now),
            (&config.head_key_app, app_id),
        ],
    )
    .await;
    assert!(resp.allow);
    assert_eq!(resp.status_code, 200);

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
