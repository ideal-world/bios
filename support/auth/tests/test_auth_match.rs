use bios_auth::{
    dto::auth_dto::AuthContext,
    serv::{auth_kernel_serv, auth_res_serv},
};
use tardis::{basic::result::TardisResult, TardisFuns};

pub fn test_match() -> TardisResult<()> {
    // public
    assert!(auth_kernel_serv::do_auth(&AuthContext {
        rbum_uri: "iam-res://iam-serv".to_string(),
        rbum_action: "get".to_string(),
        iam_app_id: None,
        iam_tenant_id: None,
        iam_account_id: None,
        iam_roles: None,
        iam_groups: None,
        own_paths: None,
        ak: None,
    })
    .is_ok());

    // private
    auth_res_serv::add_res(
        "GET",
        "iam-res://iam-serv",
        &TardisFuns::json.str_to_obj(r###"{"apps":"#app1#app2#","tenants":"#tenant1#"}"###)?,
    )?;
    assert!(auth_kernel_serv::do_auth(&AuthContext {
        rbum_uri: "iam-res://iam-serv".to_string(),
        rbum_action: "get".to_string(),
        iam_app_id: None,
        iam_tenant_id: None,
        iam_account_id: None,
        iam_roles: None,
        iam_groups: None,
        own_paths: None,
        ak: None,
    })
    .is_err());

    // match account
    auth_res_serv::add_res("GET", "iam-res://iam-serv", &TardisFuns::json.str_to_obj(r###"{"accounts":"#acc1#acc2#"}"###)?)?;
    assert!(auth_kernel_serv::do_auth(&AuthContext {
        rbum_uri: "iam-res://iam-serv".to_string(),
        rbum_action: "get".to_string(),
        iam_app_id: None,
        iam_tenant_id: None,
        iam_account_id: Some("acc3".to_string()),
        iam_roles: None,
        iam_groups: None,
        own_paths: None,
        ak: None,
    })
    .is_err());
    assert!(auth_kernel_serv::do_auth(&AuthContext {
        rbum_uri: "iam-res://iam-serv".to_string(),
        rbum_action: "get".to_string(),
        iam_app_id: None,
        iam_tenant_id: None,
        iam_account_id: Some("acc1".to_string()),
        iam_roles: None,
        iam_groups: None,
        own_paths: None,
        ak: None,
    })
    .is_ok());

    // match role
    auth_res_serv::add_res("GET", "iam-res://iam-serv", &TardisFuns::json.str_to_obj(r###"{"roles":"#role1#role2#"}"###)?)?;
    assert!(auth_kernel_serv::do_auth(&AuthContext {
        rbum_uri: "iam-res://iam-serv".to_string(),
        rbum_action: "get".to_string(),
        iam_app_id: None,
        iam_tenant_id: None,
        iam_account_id: None,
        iam_roles: Some(vec!["role0".to_string()]),
        iam_groups: None,
        own_paths: None,
        ak: None,
    })
    .is_err());
    assert!(auth_kernel_serv::do_auth(&AuthContext {
        rbum_uri: "iam-res://iam-serv".to_string(),
        rbum_action: "get".to_string(),
        iam_app_id: None,
        iam_tenant_id: None,
        iam_account_id: None,
        iam_roles: Some(vec!["role1".to_string()]),
        iam_groups: None,
        own_paths: None,
        ak: None,
    })
    .is_ok());

    // match group
    auth_res_serv::add_res(
        "GET",
        "iam-res://iam-serv",
        &TardisFuns::json.str_to_obj(r###"{"groups":"#g2.aaaa#g1.aaab##g1.aaaaaaaa##g1.aaaaaaab#"}"###)?,
    )?;
    assert!(auth_kernel_serv::do_auth(&AuthContext {
        rbum_uri: "iam-res://iam-serv".to_string(),
        rbum_action: "get".to_string(),
        iam_app_id: None,
        iam_tenant_id: None,
        iam_account_id: None,
        iam_roles: None,
        iam_groups: Some(vec!["g2.bbbb".to_string()]),
        own_paths: None,
        ak: None,
    })
    .is_err());
    assert!(auth_kernel_serv::do_auth(&AuthContext {
        rbum_uri: "iam-res://iam-serv".to_string(),
        rbum_action: "get".to_string(),
        iam_app_id: None,
        iam_tenant_id: None,
        iam_account_id: None,
        iam_roles: None,
        iam_groups: Some(vec!["g1.aaab".to_string()]),
        own_paths: None,
        ak: None,
    })
    .is_ok());
    assert!(auth_kernel_serv::do_auth(&AuthContext {
        rbum_uri: "iam-res://iam-serv".to_string(),
        rbum_action: "get".to_string(),
        iam_app_id: None,
        iam_tenant_id: None,
        iam_account_id: None,
        iam_roles: None,
        iam_groups: Some(vec!["g1.aaaa".to_string()]),
        own_paths: None,
        ak: None,
    })
    .is_ok());

    // match app
    auth_res_serv::add_res("GET", "iam-res://iam-serv", &TardisFuns::json.str_to_obj(r###"{"apps":"#app1#app2#"}"###)?)?;
    assert!(auth_kernel_serv::do_auth(&AuthContext {
        rbum_uri: "iam-res://iam-serv".to_string(),
        rbum_action: "get".to_string(),
        iam_app_id: Some("app0".to_string()),
        iam_tenant_id: None,
        iam_account_id: None,
        iam_roles: None,
        iam_groups: None,
        own_paths: None,
        ak: None,
    })
    .is_err());
    assert!(auth_kernel_serv::do_auth(&AuthContext {
        rbum_uri: "iam-res://iam-serv".to_string(),
        rbum_action: "get".to_string(),
        iam_app_id: Some("app1".to_string()),
        iam_tenant_id: None,
        iam_account_id: None,
        iam_roles: None,
        iam_groups: None,
        own_paths: None,
        ak: None,
    })
    .is_ok());

    // match tenant
    auth_res_serv::add_res("GET", "iam-res://iam-serv", &TardisFuns::json.str_to_obj(r###"{"tenants":"#tenant1#tenant2#"}"###)?)?;
    assert!(auth_kernel_serv::do_auth(&AuthContext {
        rbum_uri: "iam-res://iam-serv".to_string(),
        rbum_action: "get".to_string(),
        iam_app_id: None,
        iam_tenant_id: Some("tenant0".to_string()),
        iam_account_id: None,
        iam_roles: None,
        iam_groups: None,
        own_paths: None,
        ak: None,
    })
    .is_err());
    assert!(auth_kernel_serv::do_auth(&AuthContext {
        rbum_uri: "iam-res://iam-serv".to_string(),
        rbum_action: "get".to_string(),
        iam_app_id: None,
        iam_tenant_id: Some("tenant1".to_string()),
        iam_account_id: None,
        iam_roles: None,
        iam_groups: None,
        own_paths: None,
        ak: None,
    })
    .is_ok());

    // match all
    auth_res_serv::add_res("GET", "iam-res://iam-serv", &TardisFuns::json.str_to_obj(r###"{"tenants":"#tenant1#tenant2#"}"###)?)?;
    assert!(auth_kernel_serv::do_auth(&AuthContext {
        rbum_uri: "iam-res://iam-serv".to_string(),
        rbum_action: "get".to_string(),
        iam_app_id: Some("app1".to_string()),
        iam_tenant_id: Some("tenant1".to_string()),
        iam_account_id: Some("acc1".to_string()),
        iam_roles: None,
        iam_groups: None,
        own_paths: None,
        ak: None,
    })
    .is_ok());

    // match more
    auth_res_serv::add_res("GET", "iam-res://app1/ct/account/001", &TardisFuns::json.str_to_obj(r###"{"accounts":"#acc1#acc2#"}"###)?)?;
    assert!(auth_kernel_serv::do_auth(&AuthContext {
        rbum_uri: "iam-res://app1/ct/account/001".to_string(),
        rbum_action: "get".to_string(),
        iam_app_id: None,
        iam_tenant_id: None,
        iam_account_id: Some("acc3".to_string()),
        iam_roles: None,
        iam_groups: None,
        own_paths: None,
        ak: None,
    })
    .is_err());
    assert!(auth_kernel_serv::do_auth(&AuthContext {
        rbum_uri: "iam-res://app1/ct/account/001".to_string(),
        rbum_action: "get".to_string(),
        iam_app_id: None,
        iam_tenant_id: None,
        iam_account_id: Some("acc2".to_string()),
        iam_roles: None,
        iam_groups: None,
        own_paths: None,
        ak: None,
    })
    .is_ok());
    auth_res_serv::add_res("GET", "iam-res://app1/ct/account/**", &TardisFuns::json.str_to_obj(r###"{"accounts":"#acc3#"}"###)?)?;
    assert!(auth_kernel_serv::do_auth(&AuthContext {
        rbum_uri: "iam-res://app1/ct/account/001".to_string(),
        rbum_action: "get".to_string(),
        iam_app_id: None,
        iam_tenant_id: None,
        iam_account_id: Some("acc3".to_string()),
        iam_roles: None,
        iam_groups: None,
        own_paths: None,
        ak: None,
    })
    .is_ok());
    auth_res_serv::add_res("GET", "iam-res://app1/ct/**", &TardisFuns::json.str_to_obj(r###"{"roles":"#tenant_admin#"}"###)?)?;
    assert!(auth_kernel_serv::do_auth(&AuthContext {
        rbum_uri: "iam-res://app1/ct/account/001".to_string(),
        rbum_action: "get".to_string(),
        iam_app_id: None,
        iam_tenant_id: None,
        iam_account_id: None,
        iam_roles: Some(vec!["tenant_admin".to_string()]),
        iam_groups: None,
        own_paths: None,
        ak: None,
    })
    .is_ok());

    Ok(())
}
