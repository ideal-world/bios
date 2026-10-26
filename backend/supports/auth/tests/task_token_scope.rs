//! 任务凭据独立于交互会话，只携带授权用户在目标应用内的身份。
use bios_auth::{auth_config::AuthConfig, auth_constants::DOMAIN_CODE, serv::auth_kernel_serv::get_token_context};
use tardis::{basic::{dto::TardisContext, result::TardisResult}, tokio, TardisFuns};
mod init_cache_container;

#[tokio::test]
async fn task_tokens_do_not_inherit_the_interactive_tenant_or_platform_roles() -> TardisResult<()> {
    let _redis = init_cache_container::init().await?;
    let config = TardisFuns::cs_config::<AuthConfig>(DOMAIN_CODE);
    let cache = TardisFuns::cache_by_module_or_default(DOMAIN_CODE);
    for owner in ["alice", "bob"] {
        let token = format!("task-token-{owner}");
        let context = TardisContext {
            owner: owner.into(), own_paths: "tenant/app".into(), roles: vec!["app-member".into()], ..Default::default()
        };
        cache.set_ex(&format!("{}{}", config.cache_key_token_info, token), &format!("TokenTask,{owner}"), 60).await?;
        cache.set_ex(&format!("iam:task-token:{token}"), &TardisFuns::json.obj_to_string(&context)?, 60).await?;
        // 用户的交互会话已切换到其他租户且具有平台角色，后台任务不应受它影响。
        cache.hset(&format!("{}{}", config.cache_key_account_info, owner), "", &TardisFuns::json.obj_to_string(&TardisContext {
            owner: owner.into(), own_paths: "other-tenant".into(), roles: vec!["platform-admin".into()], ..Default::default()
        })?).await?;
        let resolved = get_token_context(&token, "app", &config, &cache).await?;
        assert_eq!(resolved.owner, owner);
        assert_eq!(resolved.own_paths, "tenant/app");
        assert_eq!(resolved.roles, ["app-member"]);
        for app in ["", "other-app"] {
            assert!(get_token_context(&token, app, &config, &cache).await.is_err());
        }
        cache.del(&format!("iam:task-token:{token}")).await?;
        assert!(get_token_context(&token, "app", &config, &cache).await.is_err());
    }
    Ok(())
}
