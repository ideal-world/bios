use bios_auth::{
    dto::auth_kernel_dto::AuthContext,
    serv::{auth_kernel_serv, auth_res_serv},
};
use tardis::{basic::result::TardisResult, TardisFuns};

pub async fn test_match() -> TardisResult<()> {
    // public
    assert!(auth_kernel_serv::do_auth(&AuthContext {
        rbum_uri: "iam-res://iam-serv".to_string(),
        rbum_action: "get".to_string(),
        app_id: None,
        tenant_id: None,
        account_id: None,
        roles: None,
        groups: None,
        own_paths: None,
        ak: None,
    })
    .await
    .is_ok());

    // private
    auth_res_serv::add_res(
        "GET",
        "iam-res://iam-serv",
        Some(TardisFuns::json.str_to_obj(r###"{"apps":"#app1#app2#","tenants":"#tenant1#"}"###)?),
        false,
        false,
        false,
    )?;
    assert!(auth_kernel_serv::do_auth(&AuthContext {
        rbum_uri: "iam-res://iam-serv".to_string(),
        rbum_action: "get".to_string(),
        app_id: None,
        tenant_id: None,
        account_id: None,
        roles: None,
        groups: None,
        own_paths: None,
        ak: None,
    })
    .await
    .is_err());

    // match account
    auth_res_serv::add_res(
        "GET",
        "iam-res://iam-serv",
        Some(TardisFuns::json.str_to_obj(r###"{"accounts":"#acc1#acc2#"}"###)?),
        false,
        false,
        false,
    )?;
    assert!(auth_kernel_serv::do_auth(&AuthContext {
        rbum_uri: "iam-res://iam-serv".to_string(),
        rbum_action: "get".to_string(),
        app_id: None,
        tenant_id: None,
        account_id: Some("acc3".to_string()),
        roles: None,
        groups: None,
        own_paths: None,
        ak: None,
    })
    .await
    .is_err());
    assert!(auth_kernel_serv::do_auth(&AuthContext {
        rbum_uri: "iam-res://iam-serv".to_string(),
        rbum_action: "get".to_string(),
        app_id: None,
        tenant_id: None,
        account_id: Some("acc1".to_string()),
        roles: None,
        groups: None,
        own_paths: None,
        ak: None,
    })
    .await
    .is_ok());

    // match role
    auth_res_serv::add_res(
        "GET",
        "iam-res://iam-serv",
        Some(TardisFuns::json.str_to_obj(r###"{"roles":"#role1#role2#"}"###)?),
        false,
        false,
        false,
    )?;
    assert!(auth_kernel_serv::do_auth(&AuthContext {
        rbum_uri: "iam-res://iam-serv".to_string(),
        rbum_action: "get".to_string(),
        app_id: None,
        tenant_id: None,
        account_id: None,
        roles: Some(vec!["role0".to_string()]),
        groups: None,
        own_paths: None,
        ak: None,
    })
    .await
    .is_err());
    assert!(auth_kernel_serv::do_auth(&AuthContext {
        rbum_uri: "iam-res://iam-serv".to_string(),
        rbum_action: "get".to_string(),
        app_id: None,
        tenant_id: None,
        account_id: None,
        roles: Some(vec!["role1".to_string()]),
        groups: None,
        own_paths: None,
        ak: None,
    })
    .await
    .is_ok());

    // match group
    auth_res_serv::add_res(
        "GET",
        "iam-res://iam-serv",
        Some(TardisFuns::json.str_to_obj(r###"{"groups":"#g2.aaaa#g1.aaab##g1.aaaaaaaa##g1.aaaaaaab#"}"###)?),
        false,
        false,
        false,
    )?;
    assert!(auth_kernel_serv::do_auth(&AuthContext {
        rbum_uri: "iam-res://iam-serv".to_string(),
        rbum_action: "get".to_string(),
        app_id: None,
        tenant_id: None,
        account_id: None,
        roles: None,
        groups: Some(vec!["g2.bbbb".to_string()]),
        own_paths: None,
        ak: None,
    })
    .await
    .is_err());
    assert!(auth_kernel_serv::do_auth(&AuthContext {
        rbum_uri: "iam-res://iam-serv".to_string(),
        rbum_action: "get".to_string(),
        app_id: None,
        tenant_id: None,
        account_id: None,
        roles: None,
        groups: Some(vec!["g1.aaab".to_string()]),
        own_paths: None,
        ak: None,
    })
    .await
    .is_ok());
    assert!(auth_kernel_serv::do_auth(&AuthContext {
        rbum_uri: "iam-res://iam-serv".to_string(),
        rbum_action: "get".to_string(),
        app_id: None,
        tenant_id: None,
        account_id: None,
        roles: None,
        groups: Some(vec!["g1.aaaa".to_string()]),
        own_paths: None,
        ak: None,
    })
    .await
    .is_ok());

    // match app
    auth_res_serv::add_res(
        "GET",
        "iam-res://iam-serv",
        Some(TardisFuns::json.str_to_obj(r###"{"apps":"#app1#app2#"}"###)?),
        false,
        false,
        false,
    )?;
    assert!(auth_kernel_serv::do_auth(&AuthContext {
        rbum_uri: "iam-res://iam-serv".to_string(),
        rbum_action: "get".to_string(),
        app_id: Some("app0".to_string()),
        tenant_id: None,
        account_id: None,
        roles: None,
        groups: None,
        own_paths: None,
        ak: None,
    })
    .await
    .is_err());
    assert!(auth_kernel_serv::do_auth(&AuthContext {
        rbum_uri: "iam-res://iam-serv".to_string(),
        rbum_action: "get".to_string(),
        app_id: Some("app1".to_string()),
        tenant_id: None,
        account_id: None,
        roles: None,
        groups: None,
        own_paths: None,
        ak: None,
    })
    .await
    .is_ok());

    // match tenant
    auth_res_serv::add_res(
        "GET",
        "iam-res://iam-serv",
        Some(TardisFuns::json.str_to_obj(r###"{"tenants":"#tenant1#tenant2#"}"###)?),
        false,
        false,
        false,
    )?;
    assert!(auth_kernel_serv::do_auth(&AuthContext {
        rbum_uri: "iam-res://iam-serv".to_string(),
        rbum_action: "get".to_string(),
        app_id: None,
        tenant_id: Some("tenant0".to_string()),
        account_id: None,
        roles: None,
        groups: None,
        own_paths: None,
        ak: None,
    })
    .await
    .is_err());
    assert!(auth_kernel_serv::do_auth(&AuthContext {
        rbum_uri: "iam-res://iam-serv".to_string(),
        rbum_action: "get".to_string(),
        app_id: None,
        tenant_id: Some("tenant1".to_string()),
        account_id: None,
        roles: None,
        groups: None,
        own_paths: None,
        ak: None,
    })
    .await
    .is_ok());

    // match all
    auth_res_serv::add_res(
        "GET",
        "iam-res://iam-serv",
        Some(TardisFuns::json.str_to_obj(r###"{"tenants":"#tenant1#tenant2#"}"###)?),
        false,
        false,
        false,
    )?;
    assert!(auth_kernel_serv::do_auth(&AuthContext {
        rbum_uri: "iam-res://iam-serv".to_string(),
        rbum_action: "get".to_string(),
        app_id: Some("app1".to_string()),
        tenant_id: Some("tenant1".to_string()),
        account_id: Some("acc1".to_string()),
        roles: None,
        groups: None,
        own_paths: None,
        ak: None,
    })
    .await
    .is_ok());

    // match more
    auth_res_serv::add_res(
        "GET",
        "iam-res://app1/ct/account/001",
        Some(TardisFuns::json.str_to_obj(r###"{"accounts":"#acc1#acc2#"}"###)?),
        false,
        false,
        false,
    )?;
    assert!(auth_kernel_serv::do_auth(&AuthContext {
        rbum_uri: "iam-res://app1/ct/account/001".to_string(),
        rbum_action: "get".to_string(),
        app_id: None,
        tenant_id: None,
        account_id: Some("acc3".to_string()),
        roles: None,
        groups: None,
        own_paths: None,
        ak: None,
    })
    .await
    .is_err());
    assert!(auth_kernel_serv::do_auth(&AuthContext {
        rbum_uri: "iam-res://app1/ct/account/001".to_string(),
        rbum_action: "get".to_string(),
        app_id: None,
        tenant_id: None,
        account_id: Some("acc2".to_string()),
        roles: None,
        groups: None,
        own_paths: None,
        ak: None,
    })
    .await
    .is_ok());
    auth_res_serv::add_res(
        "GET",
        "iam-res://app1/ct/account/**",
        Some(TardisFuns::json.str_to_obj(r###"{"accounts":"#acc3#"}"###)?),
        false,
        false,
        false,
    )?;
    assert!(auth_kernel_serv::do_auth(&AuthContext {
        rbum_uri: "iam-res://app1/ct/account/001".to_string(),
        rbum_action: "get".to_string(),
        app_id: None,
        tenant_id: None,
        account_id: Some("acc3".to_string()),
        roles: None,
        groups: None,
        own_paths: None,
        ak: None,
    })
    .await
    .is_ok());
    auth_res_serv::add_res(
        "GET",
        "iam-res://app1/ct/**",
        Some(TardisFuns::json.str_to_obj(r###"{"roles":"#tenant_admin#"}"###)?),
        false,
        false,
        false,
    )?;
    assert!(auth_kernel_serv::do_auth(&AuthContext {
        rbum_uri: "iam-res://app1/ct/account/001".to_string(),
        rbum_action: "get".to_string(),
        app_id: None,
        tenant_id: None,
        account_id: None,
        roles: Some(vec!["tenant_admin".to_string()]),
        groups: None,
        own_paths: None,
        ak: None,
    })
    .await
    .is_ok());

    Ok(())
}
