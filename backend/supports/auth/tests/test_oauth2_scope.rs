use std::collections::HashMap;

use bios_auth::{
    auth_config::AuthConfig,
    auth_constants::DOMAIN_CODE,
    auth_initializer,
    dto::auth_kernel_dto::{AuthReq, AuthResult},
    serv::{auth_kernel_serv, auth_res_serv},
};
use tardis::{basic::result::TardisResult, tokio, TardisFuns};

mod init_cache_container;

const OAUTH2_META_KEY_PREFIX: &str = "iam:cache:token:oauth2:meta:";

async fn set_token(token: &str, kind: &str, metadata: Option<&str>, config: &AuthConfig) -> TardisResult<()> {
    let cache = TardisFuns::cache_by_module_or_default(DOMAIN_CODE);
    cache.set(&format!("{}{}", config.cache_key_token_info, token), &format!("{kind},accountxxx")).await?;
    cache
        .hset(
            &format!("{}accountxxx", config.cache_key_account_info),
            "",
            r#"{"own_paths":"","owner":"account1","roles":["r001"],"groups":[]}"#,
        )
        .await?;
    if let Some(metadata) = metadata {
        cache.set(&format!("{OAUTH2_META_KEY_PREFIX}{token}"), metadata).await?;
    }
    Ok(())
}

fn metadata(scopes: &[&str]) -> String {
    let scopes = scopes.iter().map(|scope| format!("\"{scope}\"")).collect::<Vec<_>>().join(",");
    format!(r#"{{"version":1,"client_id":"hub-client","scopes":[{scopes}]}}"#)
}

async fn auth_request(method: &str, path: &str, token: &str, config: &AuthConfig) -> TardisResult<AuthResult> {
    auth_request_with_protocol(method, path, token, None, config).await
}

async fn auth_request_with_protocol(method: &str, path: &str, token: &str, protocol: Option<&str>, config: &AuthConfig) -> TardisResult<AuthResult> {
    let mut headers = HashMap::from([(config.head_key_token.clone(), token.to_string())]);
    if let Some(protocol) = protocol {
        headers.insert(config.head_key_protocol.clone(), protocol.to_string());
    }
    auth_kernel_serv::auth(
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

fn assert_forbidden(result: &AuthResult) {
    let error = result.e.as_ref().expect("request must be denied");
    assert!(error.code.starts_with("403"), "expected a 403 error, got {}", error.code);
}

fn add_account_rule(uri: &str, account: &str) -> TardisResult<()> {
    add_account_rule_for_method("GET", uri, account)
}

fn add_account_rule_for_method(method: &str, uri: &str, account: &str) -> TardisResult<()> {
    auth_res_serv::add_res(
        method,
        uri,
        Some(TardisFuns::json.str_to_obj(&format!(r##"{{"accounts":"#{account}#"}}"##))?),
        false,
        false,
        false,
        false,
        true,
    )
}

#[tokio::test]
async fn oauth2_scopes_are_enforced_and_still_intersect_with_rbac() -> TardisResult<()> {
    let _redis = init_cache_container::init().await?;
    let _auth_task = auth_initializer::init_without_webserver().await?;
    let config = TardisFuns::cs_config::<AuthConfig>(DOMAIN_CODE);
    let cache = TardisFuns::cache_by_module_or_default(DOMAIN_CODE);

    add_account_rule("iam-res://cp/oauth2/apps", "account1")?;
    add_account_rule("iam-res://cp/oauth2/userinfo", "account1")?;
    add_account_rule("iam-res://cp/oauth2/apps/*/role-members", "account1")?;

    let app_scope = metadata(&["iam.app.read"]);
    set_token("app-read", "TokenOauth2", Some(&app_scope), &config).await?;
    let result = auth_request("GET", "/cp/oauth2/apps", "app-read", &config).await?;
    assert!(result.e.is_none(), "the app scope and existing RBAC should both allow access");
    assert_forbidden(&auth_request_with_protocol("GET", "/cp/oauth2/apps", "app-read", Some("other-res"), &config).await?);

    let userinfo_scope = metadata(&["iam.userinfo.read"]);
    set_token("userinfo-read", "TokenOauth2", Some(&userinfo_scope), &config).await?;
    let result = auth_request("GET", "/cp/oauth2/userinfo", "userinfo-read", &config).await?;
    assert!(result.e.is_none(), "the userinfo scope should allow its matching endpoint");

    let role_member_scope = metadata(&["iam.app.role_member.read"]);
    set_token("role-member-scope", "TokenOauth2", Some(&role_member_scope), &config).await?;
    assert_forbidden(&auth_request("GET", "/cp/oauth2/apps", "role-member-scope", &config).await?);
    let result = auth_request("GET", "/cp/oauth2/apps/app-123/role-members", "role-member-scope", &config).await?;
    assert!(result.e.is_none(), "the role-member scope should allow one dynamic app id segment");
    assert_forbidden(&auth_request("GET", "/cp/oauth2/apps/app-123/role-members", "app-read", &config).await?);

    let all_scope = metadata(&["all"]);
    set_token("all-scope", "TokenOauth2", Some(&all_scope), &config).await?;
    assert!(auth_request("GET", "/cp/oauth2/apps", "all-scope", &config).await?.e.is_none());
    assert!(auth_request("GET", "/cp/oauth2/userinfo", "all-scope", &config).await?.e.is_none());
    assert!(auth_request("GET", "/cp/oauth2/apps/app-123/role-members", "all-scope", &config).await?.e.is_none());
    add_account_rule("iam-res://cp/oauth2/future-resource", "account1")?;
    assert!(auth_request("GET", "/cp/oauth2/future-resource", "all-scope", &config).await?.e.is_none());
    assert_forbidden(&auth_request("GET", "/cp/oauth2/future-resource", "app-read", &config).await?);
    auth_res_serv::remove_res("get", "iam-res://cp/oauth2/future-resource")?;
    add_account_rule("iam-res://cp/oauth2/future-resource", "another-account")?;
    assert_forbidden(&auth_request("GET", "/cp/oauth2/future-resource", "all-scope", &config).await?);
    auth_res_serv::remove_res("get", "iam-res://cp/oauth2/future-resource")?;
    auth_res_serv::add_res(
        "POST",
        "iam-res://cp/oauth2/future-write",
        Some(TardisFuns::json.str_to_obj(r##"{"accounts":"#account1#"}"##)?),
        false,
        false,
        false,
        false,
        true,
    )?;
    assert!(auth_request("POST", "/cp/oauth2/future-write", "all-scope", &config).await?.e.is_none());
    assert_forbidden(&auth_request("POST", "/cp/oauth2/future-write", "app-read", &config).await?);
    auth_res_serv::remove_res("post", "iam-res://cp/oauth2/future-write")?;

    assert_forbidden(&auth_request("POST", "/cp/oauth2/apps", "app-read", &config).await?);
    assert_forbidden(&auth_request("GET", "/cp/oauth2/apps/app-123/role-members/extra", "role-member-scope", &config).await?);
    assert_forbidden(&auth_request("GET", "/cp/unregistered", "app-read", &config).await?);

    set_token("wrong-scope", "TokenOauth2", Some(&userinfo_scope), &config).await?;
    assert_forbidden(&auth_request("GET", "/cp/oauth2/apps", "wrong-scope", &config).await?);

    set_token("missing-meta", "TokenOauth2", None, &config).await?;
    assert_forbidden(&auth_request("GET", "/cp/oauth2/apps", "missing-meta", &config).await?);

    let malformed_meta = r#"{"version":1,"client_id":"hub-client","scopes":"iam.app.read"}"#;
    set_token("malformed-meta", "TokenOauth2", Some(malformed_meta), &config).await?;
    assert_forbidden(&auth_request("GET", "/cp/oauth2/apps", "malformed-meta", &config).await?);

    let unknown_scope = metadata(&["iam.scope.unknown"]);
    set_token("unknown-scope", "TokenOauth2", Some(&unknown_scope), &config).await?;
    assert_forbidden(&auth_request("GET", "/cp/oauth2/apps", "unknown-scope", &config).await?);

    let wrong_version = r#"{"version":2,"client_id":"hub-client","scopes":["iam.app.read"]}"#;
    set_token("wrong-version", "TokenOauth2", Some(wrong_version), &config).await?;
    assert_forbidden(&auth_request("GET", "/cp/oauth2/apps", "wrong-version", &config).await?);

    auth_res_serv::add_res(
        "GET",
        "iam-res://cp/admin/secrets",
        Some(TardisFuns::json.str_to_obj(r##"{"accounts":"#account1#"}"##)?),
        false,
        false,
        false,
        false,
        true,
    )?;
    assert_forbidden(&auth_request("GET", "/cp/admin/secrets", "app-read", &config).await?);
    auth_res_serv::remove_res("get", "iam-res://cp/admin/secrets")?;

    cache.set(&format!("{}default-no-sidecar", config.cache_key_token_info), "TokenDefault,accountxxx").await?;
    let result = auth_request("GET", "/cp/oauth2/apps", "default-no-sidecar", &config).await?;
    assert!(result.e.is_none(), "TokenDefault must continue through its existing RBAC path");

    set_token("ci-aksk-no-sidecar", "TokenCiAkSk", None, &config).await?;
    let result = auth_request("GET", "/cp/oauth2/apps", "ci-aksk-no-sidecar", &config).await?;
    assert!(result.e.is_none(), "TokenCiAkSk must continue through its existing RBAC path without OAuth2 metadata");

    auth_res_serv::remove_res("get", "iam-res://cp/oauth2/apps")?;
    add_account_rule("iam-res://cp/oauth2/apps", "another-account")?;
    let result = auth_request("GET", "/cp/oauth2/apps", "app-read", &config).await?;
    assert_forbidden(&result);
    assert_eq!(result.e.as_ref().unwrap().message, "[Auth] Permission denied");
    auth_res_serv::remove_res("get", "iam-res://cp/oauth2/apps")?;
    auth_res_serv::remove_res("get", "iam-res://cp/oauth2/userinfo")?;
    auth_res_serv::remove_res("get", "iam-res://cp/oauth2/apps/*/role-members")?;

    Ok(())
}
