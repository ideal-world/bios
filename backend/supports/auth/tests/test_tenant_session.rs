mod init_cache_container;

use bios_auth::auth_config::AuthConfig;
use bios_auth::auth_constants::{account_app_context_field, account_tenant_context_field, build_token_info, DOMAIN_CODE};
use bios_auth::serv::auth_kernel_serv;
use tardis::basic::result::TardisResult;
use tardis::tokio;
use tardis::TardisFuns;

#[tokio::test]
async fn test_tenant_bound_token_context() -> TardisResult<()> {
    let _hold = init_cache_container::init().await?;
    let config = TardisFuns::cs_config::<AuthConfig>(DOMAIN_CODE);
    let cache = TardisFuns::cache_by_module_or_default(DOMAIN_CODE);
    let account_id = "tenant_session_account";
    let account_key = format!("{}{}", config.cache_key_account_info, account_id);

    for (token, tenant, role) in [("token_a", "tenant_a", "role_a"), ("token_b", "tenant_b", "role_b")] {
        cache
            .set(
                &format!("{}{}", config.cache_key_token_info, token),
                &build_token_info("TokenDefault", account_id, None, Some(tenant)),
            )
            .await?;
        cache
            .hset(
                &account_key,
                &account_tenant_context_field(tenant),
                &format!("{{\"own_paths\":\"{tenant}\",\"owner\":\"{account_id}\",\"roles\":[\"{role}\"],\"groups\":[]}}"),
            )
            .await?;
        cache
            .hset(
                &account_key,
                &account_app_context_field(tenant, "app1"),
                &format!("{{\"own_paths\":\"{tenant}/app1\",\"owner\":\"{account_id}\",\"roles\":[\"app_role_{tenant}\"],\"groups\":[]}}"),
            )
            .await?;
    }
    // 账号级共享字段可能属于任意一个租户，绑定租户的会话不能读它
    cache
        .hset(
            &account_key,
            "",
            &format!("{{\"own_paths\":\"tenant_legacy\",\"owner\":\"{account_id}\",\"roles\":[\"role_legacy\"],\"groups\":[]}}"),
        )
        .await?;

    let ctx_a = auth_kernel_serv::get_token_context("token_a", "", &config, &cache).await?;
    assert_eq!(ctx_a.own_paths, "tenant_a");
    assert_eq!(ctx_a.roles, vec!["role_a"]);

    let ctx_b = auth_kernel_serv::get_token_context("token_b", "", &config, &cache).await?;
    assert_eq!(ctx_b.own_paths, "tenant_b");
    assert_eq!(ctx_b.roles, vec!["role_b"]);

    let app_ctx_a = auth_kernel_serv::get_token_context("token_a", "app1", &config, &cache).await?;
    assert_eq!(app_ctx_a.own_paths, "tenant_a/app1");
    assert_eq!(app_ctx_a.roles, vec!["app_role_tenant_a", "role_a"]);

    let app_ctx_b = auth_kernel_serv::get_token_context("token_b", "app1", &config, &cache).await?;
    assert_eq!(app_ctx_b.own_paths, "tenant_b/app1");
    assert_eq!(app_ctx_b.roles, vec!["app_role_tenant_b", "role_b"]);

    // 没有租户绑定的旧 token 回退到账号级共享字段
    cache.set(&format!("{}token_legacy", config.cache_key_token_info), &format!("TokenDefault,{account_id}")).await?;
    let legacy_ctx = auth_kernel_serv::get_token_context("token_legacy", "", &config, &cache).await?;
    assert_eq!(legacy_ctx.own_paths, "tenant_legacy");
    assert_eq!(legacy_ctx.roles, vec!["role_legacy"]);
    Ok(())
}
