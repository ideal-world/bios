use std::collections::HashMap;

use bios_basic::test::init_test_container;
use bios_iam::basic::dto::iam_account_dto::IamAccountInfoResp;
use bios_iam::basic::dto::iam_cert_dto::IamContextFetchReq;
use bios_iam::basic::serv::iam_key_cache_serv::IamIdentCacheServ;
use bios_iam::iam_config::IamConfig;
use bios_iam::iam_constants;
use tardis::basic::result::TardisResult;
use tardis::tokio;

#[tokio::test]
async fn test_tenant_session_isolation() -> TardisResult<()> {
    let _hold = init_test_container::init(None).await?;
    let funs = iam_constants::get_tardis_inst();
    let account_id = "tenant_session_account";
    let tenant_a = "tenant_a";
    let tenant_b = "tenant_b";
    let token_a = "token_tenant_a";
    let token_b = "token_tenant_b";

    for (token, tenant, role) in [(token_a, tenant_a, "role_a"), (token_b, tenant_b, "role_b")] {
        funs.cache()
            .set(
                &format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, token),
                &format!("TokenDefault,{account_id}"),
            )
            .await?;
        IamIdentCacheServ::add_contexts(
            &IamAccountInfoResp {
                account_id: account_id.to_string(),
                account_name: "test".to_string(),
                token: token.to_string(),
                access_token: None,
                roles: HashMap::from([(role.to_string(), role.to_string())]),
                groups: HashMap::new(),
                apps: vec![],
            },
            tenant,
            &funs,
        )
        .await?;
        IamIdentCacheServ::rebind_token_tenant(token, tenant, &funs).await?;
    }

    let ctx_a = IamIdentCacheServ::get_context(
        &IamContextFetchReq {
            token: token_a.to_string(),
            app_id: None,
        },
        &funs,
    )
    .await?;
    let ctx_b = IamIdentCacheServ::get_context(
        &IamContextFetchReq {
            token: token_b.to_string(),
            app_id: None,
        },
        &funs,
    )
    .await?;
    assert_eq!(ctx_a.own_paths, tenant_a);
    assert_eq!(ctx_a.roles, vec!["role_a"]);
    assert_eq!(ctx_b.own_paths, tenant_b);
    assert_eq!(ctx_b.roles, vec!["role_b"]);

    // 重绑一个会话的 token 不应影响另一个会话
    IamIdentCacheServ::rebind_token_tenant(token_a, tenant_b, &funs).await?;
    let ctx_b_after = IamIdentCacheServ::get_context(
        &IamContextFetchReq {
            token: token_b.to_string(),
            app_id: None,
        },
        &funs,
    )
    .await?;
    assert_eq!(ctx_b_after.own_paths, tenant_b);
    assert_eq!(ctx_b_after.roles, vec!["role_b"]);

    // 旧 token 没有租户绑定，回退到账号级共享字段
    let legacy_token = "token_legacy";
    funs.cache()
        .set(
            &format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, legacy_token),
            &format!("TokenDefault,{account_id}"),
        )
        .await?;
    funs.cache()
        .hset(
            &format!("{}{}", funs.conf::<IamConfig>().cache_key_account_info_, account_id),
            "",
            "{\"own_paths\":\"tenant_legacy\",\"owner\":\"tenant_session_account\",\"roles\":[\"role_legacy\"],\"groups\":[]}",
        )
        .await?;
    let legacy_ctx = IamIdentCacheServ::get_context(
        &IamContextFetchReq {
            token: legacy_token.to_string(),
            app_id: None,
        },
        &funs,
    )
    .await?;
    assert_eq!(legacy_ctx.own_paths, "tenant_legacy");
    assert_eq!(legacy_ctx.roles, vec!["role_legacy"]);
    Ok(())
}
