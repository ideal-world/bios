use std::collections::HashMap;

use bios_auth::{
    auth_config::AuthConfig,
    auth_constants::DOMAIN_CODE,
    auth_initializer,
    dto::auth_kernel_dto::{AuthReq, AuthResult},
    serv::{auth_res_serv, auth_temp_serv},
};
use tardis::{basic::result::TardisResult, cache::AsyncCommands, tokio, TardisFuns};

mod init_cache_container;

const ROLE_ACCOUNT: &str = "account-role";
const TEMP_ACCOUNT: &str = "account-temp";
const PRODUCT: &str = "product-1";

#[tokio::test]
async fn temp_product_access_keeps_formal_auth_and_adds_a_fallback() -> TardisResult<()> {
    let _redis = init_cache_container::init().await?;
    let _auth_task = auth_initializer::init_without_webserver().await?;
    let config = TardisFuns::cs_config::<AuthConfig>(DOMAIN_CODE);

    auth_res_serv::add_res("get", "iam-res://cp/by-role", Some(role_auth("role-1")?), false, false, false, false, true)?;
    auth_res_serv::add_res("get", "iam-res://cp/account/*", Some(role_auth("nope")?), false, false, false, false, true)?;
    auth_res_serv::add_res("post", "iam-res://cp/**", Some(role_auth("nope")?), false, false, false, false, true)?;
    auth_res_serv::add_res("get", "iam-res://cp/secure", Some(role_auth("nope")?), false, false, true, false, true)?;
    auth_res_serv::add_res("get", "iam-res://cp/aksk-only", Some(role_auth("nope")?), false, false, false, true, true)?;

    let cache = TardisFuns::cache_by_module_or_default(DOMAIN_CODE);
    cache.set(&format!("{}token-role", config.cache_key_token_info), &format!("TokenDefault,{ROLE_ACCOUNT}")).await?;
    cache
        .hset(
            &format!("{}{ROLE_ACCOUNT}", config.cache_key_account_info),
            PRODUCT,
            &context_json(ROLE_ACCOUNT, "tenant-1/product-1", r#"["role-1"]"#),
        )
        .await?;

    let allowed = auth_request("GET", "/cp/by-role", "token-role", Some(PRODUCT), &config).await?;
    assert!(allowed.e.is_none(), "an existing role still allows the request");
    assert_eq!(allowed.ctx.as_ref().and_then(|ctx| ctx.roles.clone()).unwrap_or_default(), vec!["role-1".to_string()]);

    cache.set(&format!("{}token-temp", config.cache_key_token_info), &format!("TokenDefault,{TEMP_ACCOUNT}")).await?;
    cache
        .hset(
            &format!("{}{TEMP_ACCOUNT}", config.cache_key_temp_product),
            PRODUCT,
            r#"{"own_paths":"tenant-1/product-1","expire_at":0}"#,
        )
        .await?;
    replace_temp_apis(&config, &["iam-res://cp/account/*##get", "iam-res://cp/other/**##*"]).await?;

    let temp_ctx = auth_request("GET", "/cp/account/item", "token-temp", Some(PRODUCT), &config).await?;
    assert!(temp_ctx.e.is_none(), "a temp product and a matching api should allow access");
    let ctx = temp_ctx.ctx.as_ref().expect("allowed request carries context");
    assert_eq!(ctx.account_id.as_deref(), Some(TEMP_ACCOUNT));
    assert_eq!(ctx.own_paths.as_deref(), Some("tenant-1/product-1"));
    assert!(ctx.roles.as_ref().unwrap().is_empty());

    let wildcard = auth_request("POST", "/cp/other/deep", "token-temp", Some(PRODUCT), &config).await?;
    assert!(wildcard.e.is_none(), "`**` and method `*` follow the existing resource matcher");

    let missing_api = auth_request("GET", "/cp/by-role", "token-temp", Some(PRODUCT), &config).await?;
    assert_eq!(missing_api.e.as_ref().map(|error| error.message.as_str()), Some("[Auth] Permission denied"));

    cache
        .hset(
            &format!("{}{TEMP_ACCOUNT}", config.cache_key_temp_product),
            PRODUCT,
            r#"{"own_paths":"tenant-1/product-1","expire_at":1}"#,
        )
        .await?;
    let expired = auth_request("GET", "/cp/account/item", "token-temp", Some(PRODUCT), &config).await?;
    assert_eq!(expired.e.as_ref().map(|error| error.code.as_str()), Some("401"));

    cache
        .hset(
            &format!("{}{TEMP_ACCOUNT}", config.cache_key_temp_product),
            PRODUCT,
            r#"{"own_paths":"tenant-b/product-1","expire_at":0}"#,
        )
        .await?;
    cache
        .hset(
            &format!("{}{TEMP_ACCOUNT}", config.cache_key_account_info),
            PRODUCT,
            &context_json(TEMP_ACCOUNT, "tenant-a/product-1", "[]"),
        )
        .await?;
    let mismatched = auth_request("GET", "/cp/account/item", "token-temp", Some(PRODUCT), &config).await?;
    assert_eq!(mismatched.e.as_ref().map(|error| error.message.as_str()), Some("[Auth] Permission denied"));

    cache.del(&format!("{}{TEMP_ACCOUNT}", config.cache_key_account_info)).await?;
    cache
        .hset(
            &format!("{}{TEMP_ACCOUNT}", config.cache_key_temp_product),
            PRODUCT,
            r#"{"own_paths":"tenant-1/product-1","expire_at":0}"#,
        )
        .await?;
    let double_auth = auth_request("GET", "/cp/secure", "token-temp", Some(PRODUCT), &config).await?;
    assert_eq!(
        double_auth.e.as_ref().map(|error| error.message.as_str()),
        Some("[Auth] Secondary confirmation is required")
    );
    let aksk_only = auth_request("GET", "/cp/aksk-only", "token-temp", Some(PRODUCT), &config).await?;
    assert_eq!(
        aksk_only.e.as_ref().map(|error| error.message.as_str()),
        Some("[Auth] Only AK/SK authentication is allowed")
    );

    Ok(())
}

fn role_auth(role: &str) -> TardisResult<bios_auth::dto::auth_kernel_dto::ResAuthInfo> {
    let marker = format!("#{role}#");
    Ok(TardisFuns::json.str_to_obj(&format!(r#"{{"roles":"{marker}"}}"#))?)
}

fn context_json(owner: &str, own_paths: &str, roles: &str) -> String {
    format!(r#"{{"own_paths":"{own_paths}","owner":"{owner}","roles":{roles},"groups":[]}}"#)
}

async fn replace_temp_apis(config: &AuthConfig, apis: &[&str]) -> TardisResult<()> {
    let cache = TardisFuns::cache_by_module_or_default(DOMAIN_CODE);
    let mut conn = cache.cmd().await?;
    let _: i32 = conn.del(&config.cache_key_temp_api).await?;
    let _: i32 = conn.sadd(&config.cache_key_temp_api, apis).await?;
    cache.incr(&config.cache_key_temp_api_version, 1).await?;
    auth_temp_serv::sync_temp_api_cache().await
}

async fn auth_request(method: &str, path: &str, token: &str, app_id: Option<&str>, config: &AuthConfig) -> TardisResult<AuthResult> {
    let mut headers = HashMap::from([(config.head_key_token.clone(), token.to_string())]);
    if let Some(app_id) = app_id {
        headers.insert(config.head_key_app.clone(), app_id.to_string());
    }
    bios_auth::serv::auth_kernel_serv::auth(
        &mut AuthReq {
            scheme: "http".to_string(),
            path: path.to_string(),
            query: HashMap::new(),
            method: method.to_string(),
            host: "localhost".to_string(),
            port: 80,
            headers,
            body: None,
        },
        false,
    )
    .await
}
