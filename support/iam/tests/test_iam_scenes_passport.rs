use std::collections::HashMap;
use std::env;
use std::time::Duration;
use tardis::basic::dto::TardisContext;

use bios_iam::console_passport::dto::iam_cp_cert_dto::{
    IamCpLdapLoginReq, IamCpOAuth2LoginReq, IamCpUserPwdBindReq, IamCpUserPwdBindWithLdapReq, IamCpUserPwdCheckReq, IamCpUserPwdLoginReq,
};
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;
use tardis::tokio::time::sleep;
use tardis::web::web_resp::{TardisResp, Void};

use bios_basic::rbum::dto::rbum_cert_dto::RbumCertSummaryResp;
use bios_basic::rbum::rbum_enumeration::{RbumDataTypeKind, RbumWidgetTypeKind};
use bios_iam::basic::dto::iam_account_dto::{IamAccountAggAddReq, IamAccountInfoResp, IamAccountInfoWithUserPwdAkResp, IamAccountSelfModifyReq, IamCpUserPwdBindResp};
use bios_iam::basic::dto::iam_app_dto::IamAppAggAddReq;
use bios_iam::basic::dto::iam_attr_dto::IamKindAttrAddReq;
use bios_iam::basic::dto::iam_cert_conf_dto::{IamCertConfLdapAddOrModifyReq, IamCertConfOAuth2AddOrModifyReq, IamCertConfUserPwdAddOrModifyReq};
use bios_iam::basic::dto::iam_cert_dto::{IamCertPwdNewReq, IamCertUserPwdModifyReq, IamCertUserPwdRestReq};
use bios_iam::basic::dto::iam_set_dto::{IamSetCateAddReq, IamSetItemWithDefaultSetAddReq};
use bios_iam::basic::dto::iam_tenant_dto::{IamTenantAggAddReq, IamTenantBoneResp};
use bios_iam::basic::serv::iam_cert_ldap_serv::IamCertLdapServ;
use bios_iam::console_passport::dto::iam_cp_account_dto::IamCpAccountInfoResp;
use bios_iam::iam_constants;
use bios_iam::iam_constants::RBUM_SCOPE_LEVEL_TENANT;
use bios_iam::iam_enumeration::{IamCertOAuth2Supplier, IamCertTokenKind};
use bios_iam::iam_test_helper::BIOSWebTestClient;

pub async fn test(sysadmin_name: &str, sysadmin_password: &str, client: &mut BIOSWebTestClient) -> TardisResult<()> {
    info!("【test_iam_scenes_passport】");

    info!("【test_iam_scenes_passport:system】");
    login_page(sysadmin_name, sysadmin_password, None, None, true, client).await?;
    account_mgr_by_sys_admin(client).await?;
    let sysadmin_password = security_mgr_page_by_sys_admin(sysadmin_name, sysadmin_password, client).await?;

    info!("【test_iam_scenes_passport:tenant】");
    let tenant_id: String = client
        .post(
            "/cs/tenant",
            &IamTenantAggAddReq {
                name: TrimString("测试公司1".to_string()),
                icon: Some("https://oss.minio.io/xxx.icon".to_string()),
                contact_phone: None,
                note: None,
                admin_name: TrimString("测试管理员".to_string()),
                admin_username: TrimString("tenant_admin".to_string()),
                admin_password: Some("123456".to_string()),
                cert_conf_by_user_pwd: IamCertConfUserPwdAddOrModifyReq {
                    ak_rule_len_min: 2,
                    ak_rule_len_max: 20,
                    sk_rule_len_min: 2,
                    sk_rule_len_max: 20,
                    sk_rule_need_num: false,
                    sk_rule_need_uppercase: false,
                    sk_rule_need_lowercase: false,
                    sk_rule_need_spec_char: false,
                    sk_lock_cycle_sec: 60,
                    sk_lock_err_times: 2,
                    sk_lock_duration_sec: 60,
                    repeatable: false,
                    expire_sec: 6000,
                },
                cert_conf_by_phone_vcode: false,
                cert_conf_by_mail_vcode: false,
                disabled: None,
                account_self_reg: None,
                cert_conf_by_oauth2: None,
                cert_conf_by_ldap: None,
            },
        )
        .await;
    sleep(Duration::from_secs(1)).await;
    login_page("tenant_admin", "123456", Some(tenant_id.clone()), None, true, client).await?;
    let cate_node_id: String = client
        .post(
            "/ct/org/cate",
            &IamSetCateAddReq {
                name: TrimString("综合服务中心".to_string()),
                scope_level: Some(RBUM_SCOPE_LEVEL_TENANT),
                bus_code: None,
                icon: None,
                sort: None,
                ext: None,
                rbum_parent_cate_id: None,
            },
        )
        .await;
    account_mgr_by_tenant_account(client).await?;
    security_mgr_by_tenant_account("tenant_admin", "123456", &tenant_id, client).await?;

    info!("【test_iam_scenes_passport:app】");
    let account_id: String = client
        .post(
            "/ct/account",
            &IamAccountAggAddReq {
                id: None,
                name: TrimString("用户1".to_string()),
                cert_user_name: TrimString("user1".to_string()),
                cert_password: TrimString("123456".to_string()),
                cert_phone: None,
                cert_mail: None,
                role_ids: None,
                org_node_ids: None,
                scope_level: Some(RBUM_SCOPE_LEVEL_TENANT),
                disabled: None,
                icon: None,
                exts: HashMap::from([("ext9".to_string(), "00001".to_string())]),
                status: None,
            },
        )
        .await;
    let _: String = client
        .put(
            "/ct/org/item",
            &IamSetItemWithDefaultSetAddReq {
                set_cate_id: cate_node_id.to_string(),
                sort: 0,
                rel_rbum_item_id: account_id.clone(),
            },
        )
        .await;
    let app_id: String = client
        .post(
            "/ct/app",
            &IamAppAggAddReq {
                app_name: TrimString("devops project".to_string()),
                app_icon: None,
                app_sort: None,
                app_contact_phone: None,
                admin_ids: Some(vec![account_id.clone()]),
                disabled: None,
            },
        )
        .await;
    sleep(Duration::from_secs(1)).await;
    login_page("user1", "123456", Some(tenant_id.clone()), Some(app_id.clone()), true, client).await?;
    account_mgr_by_app_account(client).await?;
    security_mgr_by_app_account("user1", "123456", &tenant_id, &app_id, client).await?;
    login_page(sysadmin_name, &sysadmin_password, None, None, true, client).await?;
    security_password(client).await?;
    // login_by_oauth2(client).await?;
    login_page(sysadmin_name, &sysadmin_password, None, None, true, client).await?;
    login_by_ldap(client).await?;
    Ok(())
}

pub async fn login_page(
    user_name: &str,
    password: &str,
    tenant_id: Option<String>,
    app_id: Option<String>,
    set_auth: bool,
    client: &mut BIOSWebTestClient,
) -> TardisResult<IamAccountInfoResp> {
    info!("【login_page】");

    // Find Tenants
    let _: Vec<IamTenantBoneResp> = client.get("/cp/tenant/all").await;
    // Login
    client.login(user_name, password, tenant_id, app_id, Some(IamCertTokenKind::TokenPc.to_string()), set_auth).await
}

pub async fn account_mgr_by_sys_admin(client: &mut BIOSWebTestClient) -> TardisResult<()> {
    info!("【account_mgr_by_sys_admin】");

    // Get Current Account
    let account: IamCpAccountInfoResp = client.get("/cp/account").await;
    assert_eq!(account.account_name, "bios");
    assert!(account.tenant_name.is_none());
    assert!(account.apps.is_empty());
    assert_eq!(account.roles.len(), 1);
    assert!(account.roles.iter().any(|(_, v)| v == "sys_admin"));
    assert!(account.org.is_empty());
    assert_eq!(account.certs.len(), 1);
    assert!(account.certs.contains_key("UserPwd"));
    assert!(account.exts.is_empty());

    // Find Certs By Current Account
    let certs: Vec<RbumCertSummaryResp> = client.get("/cp/cert").await;
    assert_eq!(certs.len(), 1);
    assert!(certs.into_iter().any(|i| i.rel_rbum_cert_conf_code == Some("UserPwd".to_string())));

    // Modify Account By Current Account
    let _: Void = client
        .put(
            "/cp/account",
            &IamAccountSelfModifyReq {
                name: Some(TrimString("租户admin".to_string())),
                disabled: None,
                icon: None,
                exts: HashMap::new(),
            },
        )
        .await;

    // Get Current Account
    let account: IamCpAccountInfoResp = client.get("/cp/account").await;
    assert_eq!(account.account_name, "租户admin");

    Ok(())
}

pub async fn security_mgr_page_by_sys_admin(name: &str, password: &str, client: &mut BIOSWebTestClient) -> TardisResult<String> {
    info!("【security_mgr_page_by_sys_admin】");

    // Modify Password
    let _: Void = client
        .put(
            "/cp/cert/userpwd",
            &IamCertUserPwdModifyReq {
                original_sk: TrimString(password.to_string()),
                new_sk: TrimString("654321".to_string()),
            },
        )
        .await;
    client.login(name, "654321", None, None, Some(IamCertTokenKind::TokenPc.to_string()), true).await?;

    // Modify Password without login
    let result: TardisResp<Void> = client
        .put_resp(
            "/cp/cert/userpwd/new",
            &IamCertPwdNewReq {
                ak: TrimString(name.to_string()),
                original_sk: TrimString(password.to_string()),
                new_sk: TrimString("654321".to_string()),
                tenant_id: None,
            },
        )
        .await;
    assert!(result.code.starts_with("401"));

    let _: Void = client
        .put(
            "/cp/cert/userpwd/new",
            &IamCertPwdNewReq {
                ak: TrimString(name.to_string()),
                original_sk: TrimString("654321".to_string()),
                new_sk: TrimString("xxxxx".to_string()),
                tenant_id: None,
            },
        )
        .await;
    client.login(name, "xxxxx", None, None, Some(IamCertTokenKind::TokenPc.to_string()), true).await?;

    Ok("xxxxx".to_string())
}

pub async fn account_mgr_by_tenant_account(client: &mut BIOSWebTestClient) -> TardisResult<()> {
    info!("【account_mgr_by_tenant_account】");

    // Get Current Account
    let account: IamCpAccountInfoResp = client.get("/cp/account").await;
    assert_eq!(account.account_name, "测试管理员");
    assert_eq!(account.tenant_name, Some("测试公司1".to_string()));
    assert!(account.apps.is_empty());
    assert_eq!(account.roles.len(), 1);
    assert!(account.roles.iter().any(|(_, v)| v == "tenant_admin"));
    assert!(account.org.is_empty());
    assert_eq!(account.certs.len(), 1);
    assert!(account.certs.contains_key("UserPwd"));
    assert!(account.exts.is_empty());

    // Find Certs By Current Account
    let certs: Vec<RbumCertSummaryResp> = client.get("/cp/cert").await;
    assert_eq!(certs.len(), 1);
    assert!(certs.into_iter().any(|i| i.rel_rbum_cert_conf_code == Some("UserPwd".to_string())));

    let _: String = client
        .post(
            "/ct/account/attr",
            &IamKindAttrAddReq {
                name: TrimString("ext9".to_string()),
                label: "岗级".to_string(),
                note: None,
                sort: None,
                main_column: Some(true),
                position: None,
                capacity: None,
                overload: None,
                idx: None,
                data_type: RbumDataTypeKind::String,
                widget_type: RbumWidgetTypeKind::Input,
                default_value: None,
                options: Some(r#"[{"l1":"L1","l2":"L2"}]"#.to_string()),
                required: None,
                min_length: None,
                max_length: None,
                action: None,
                ext: None,
                scope_level: Some(RBUM_SCOPE_LEVEL_TENANT),
            },
        )
        .await;

    // Modify Account By Current Account
    let _: Void = client
        .put(
            "/cp/account",
            &IamAccountSelfModifyReq {
                name: Some(TrimString("租户管理员".to_string())),
                disabled: None,
                icon: None,
                exts: HashMap::from([("ext9".to_string(), "00001".to_string())]),
            },
        )
        .await;

    // Get Current Account
    let account: IamCpAccountInfoResp = client.get("/cp/account").await;
    assert_eq!(account.account_name, "租户管理员");

    // Get Current Account
    let account: IamCpAccountInfoResp = client.get("/cp/account").await;
    assert_eq!(account.account_name, "租户管理员");
    assert_eq!(account.tenant_name, Some("测试公司1".to_string()));
    assert!(account.apps.is_empty());
    assert_eq!(account.roles.len(), 1);
    assert!(account.roles.iter().any(|(_, v)| v == "tenant_admin"));
    assert!(account.org.is_empty());
    assert_eq!(account.certs.len(), 1);
    assert!(account.certs.contains_key("UserPwd"));
    assert_eq!(account.exts.len(), 1);
    assert_eq!(account.exts[0].name, "ext9");
    assert_eq!(account.exts[0].label, "岗级");
    assert_eq!(account.exts[0].value, "00001");

    Ok(())
}

pub async fn security_mgr_by_tenant_account(name: &str, password: &str, tenant_id: &str, client: &mut BIOSWebTestClient) -> TardisResult<()> {
    info!("【security_mgr_by_tenant_account】");

    // Modify Password
    let _: Void = client
        .put(
            "/cp/cert/userpwd",
            &IamCertUserPwdModifyReq {
                original_sk: TrimString(password.to_string()),
                new_sk: TrimString("654321".to_string()),
            },
        )
        .await;
    client.login(name, "654321", Some(tenant_id.to_string()), None, Some(IamCertTokenKind::TokenPc.to_string()), true).await?;

    // Modify Password without login
    let result: TardisResp<Void> = client
        .put_resp(
            "/cp/cert/userpwd/new",
            &IamCertPwdNewReq {
                ak: TrimString(name.to_string()),
                original_sk: TrimString(password.to_string()),
                new_sk: TrimString("654321".to_string()),
                tenant_id: Some(tenant_id.to_string()),
            },
        )
        .await;
    assert!(result.code.starts_with("401"));

    let _: Void = client
        .put(
            "/cp/cert/userpwd/new",
            &IamCertPwdNewReq {
                ak: TrimString(name.to_string()),
                original_sk: TrimString("654321".to_string()),
                new_sk: TrimString("xxxxx".to_string()),
                tenant_id: Some(tenant_id.to_string()),
            },
        )
        .await;
    client.login(name, "xxxxx", Some(tenant_id.to_string()), None, Some(IamCertTokenKind::TokenPc.to_string()), true).await?;

    Ok(())
}

pub async fn account_mgr_by_app_account(client: &mut BIOSWebTestClient) -> TardisResult<()> {
    info!("【account_mgr_by_app_account】");

    // Get Current Account
    let account: IamCpAccountInfoResp = client.get("/cp/account").await;
    assert_eq!(account.account_name, "用户1");
    assert_eq!(account.tenant_name, Some("测试公司1".to_string()));
    assert_eq!(account.apps.len(), 1);
    assert!(account.apps.iter().any(|i| i.app_name == "devops project"));
    assert_eq!(account.roles.len(), 1);
    assert_eq!(account.roles.len(), 1);
    assert_eq!(account.org.len(), 1);
    assert_eq!(account.org[0], "综合服务中心");
    assert_eq!(account.certs.len(), 1);
    assert!(account.certs.contains_key("UserPwd"));
    assert_eq!(account.exts.len(), 1);
    assert_eq!(account.exts[0].name, "ext9");
    assert_eq!(account.exts[0].label, "岗级");
    assert_eq!(account.exts[0].value, "00001");

    // Find Certs By Current Account
    let certs: Vec<RbumCertSummaryResp> = client.get("/cp/cert").await;
    assert_eq!(certs.len(), 1);
    assert!(certs.into_iter().any(|i| i.rel_rbum_cert_conf_code == Some("UserPwd".to_string())));

    Ok(())
}

pub async fn security_mgr_by_app_account(name: &str, password: &str, tenant_id: &str, app_id: &str, client: &mut BIOSWebTestClient) -> TardisResult<()> {
    info!("【security_mgr_by_app_account】");

    // Modify Password
    let _: Void = client
        .put(
            "/cp/cert/userpwd",
            &IamCertUserPwdModifyReq {
                original_sk: TrimString(password.to_string()),
                new_sk: TrimString("654321".to_string()),
            },
        )
        .await;
    client
        .login(
            name,
            "654321",
            Some(tenant_id.to_string()),
            Some(app_id.to_string()),
            Some(IamCertTokenKind::TokenPc.to_string()),
            true,
        )
        .await?;

    // Modify Password without login
    let result: TardisResp<Void> = client
        .put_resp(
            "/cp/cert/userpwd/new",
            &IamCertPwdNewReq {
                ak: TrimString(name.to_string()),
                original_sk: TrimString(password.to_string()),
                new_sk: TrimString("654321".to_string()),
                tenant_id: Some(tenant_id.to_string()),
            },
        )
        .await;
    assert!(result.code.starts_with("401"));

    let result: TardisResp<Void> = client
        .put_resp(
            "/cp/cert/userpwd/new",
            &IamCertPwdNewReq {
                ak: TrimString(name.to_string()),
                original_sk: TrimString("654321".to_string()),
                new_sk: TrimString("654321".to_string()),
                tenant_id: Some(tenant_id.to_string()),
            },
        )
        .await;
    assert!(result.code.starts_with("400"));

    let _: Void = client
        .put(
            "/cp/cert/userpwd/new",
            &IamCertPwdNewReq {
                ak: TrimString(name.to_string()),
                original_sk: TrimString("654321".to_string()),
                new_sk: TrimString("xxxxx".to_string()),
                tenant_id: Some(tenant_id.to_string()),
            },
        )
        .await;
    client
        .login(
            name,
            "xxxxx",
            Some(tenant_id.to_string()),
            Some(app_id.to_string()),
            Some(IamCertTokenKind::TokenPc.to_string()),
            true,
        )
        .await?;

    Ok(())
}

pub async fn security_password(client: &mut BIOSWebTestClient) -> TardisResult<()> {
    info!("【security_password】");

    assert!(client
        .post_resp::<IamTenantAggAddReq, String>(
            "/cs/tenant",
            &IamTenantAggAddReq {
                name: TrimString("测试公司1".to_string()),
                icon: Some("https://oss.minio.io/xxx.icon".to_string()),
                contact_phone: None,
                note: None,
                admin_name: TrimString("测试管理员".to_string()),
                admin_username: TrimString("tenant_admin".to_string()),
                admin_password: Some("aaaa".to_string()),
                cert_conf_by_user_pwd: IamCertConfUserPwdAddOrModifyReq {
                    ak_rule_len_min: 2,
                    ak_rule_len_max: 20,
                    sk_rule_len_min: 2,
                    sk_rule_len_max: 20,
                    sk_rule_need_num: true,
                    sk_rule_need_uppercase: false,
                    sk_rule_need_lowercase: false,
                    sk_rule_need_spec_char: false,
                    sk_lock_cycle_sec: 60,
                    sk_lock_err_times: 2,
                    sk_lock_duration_sec: 60,
                    repeatable: false,
                    expire_sec: 6000,
                },
                cert_conf_by_phone_vcode: false,
                cert_conf_by_mail_vcode: false,
                disabled: None,
                account_self_reg: None,
                cert_conf_by_oauth2: None,
                cert_conf_by_ldap: None,
            },
        )
        .await
        .code
        .starts_with("400"));

    assert!(client
        .post_resp::<IamTenantAggAddReq, String>(
            "/cs/tenant",
            &IamTenantAggAddReq {
                name: TrimString("测试公司1".to_string()),
                icon: Some("https://oss.minio.io/xxx.icon".to_string()),
                contact_phone: None,
                note: None,
                admin_name: TrimString("测试管理员".to_string()),
                admin_username: TrimString("tenant_admin".to_string()),
                admin_password: Some("aa22".to_string()),
                cert_conf_by_user_pwd: IamCertConfUserPwdAddOrModifyReq {
                    ak_rule_len_min: 2,
                    ak_rule_len_max: 20,
                    sk_rule_len_min: 2,
                    sk_rule_len_max: 20,
                    sk_rule_need_num: true,
                    sk_rule_need_uppercase: true,
                    sk_rule_need_lowercase: false,
                    sk_rule_need_spec_char: false,
                    sk_lock_cycle_sec: 60,
                    sk_lock_err_times: 2,
                    sk_lock_duration_sec: 60,
                    repeatable: false,
                    expire_sec: 6000,
                },
                cert_conf_by_phone_vcode: false,
                cert_conf_by_mail_vcode: false,
                disabled: None,
                account_self_reg: None,
                cert_conf_by_oauth2: None,
                cert_conf_by_ldap: None,
            },
        )
        .await
        .code
        .starts_with("400"));

    assert!(client
        .post_resp::<IamTenantAggAddReq, String>(
            "/cs/tenant",
            &IamTenantAggAddReq {
                name: TrimString("测试公司1".to_string()),
                icon: Some("https://oss.minio.io/xxx.icon".to_string()),
                contact_phone: None,
                note: None,
                admin_name: TrimString("测试管理员".to_string()),
                admin_username: TrimString("tenant_admin".to_string()),
                admin_password: Some("aa22A".to_string()),
                cert_conf_by_user_pwd: IamCertConfUserPwdAddOrModifyReq {
                    ak_rule_len_min: 2,
                    ak_rule_len_max: 20,
                    sk_rule_len_min: 2,
                    sk_rule_len_max: 20,
                    sk_rule_need_num: true,
                    sk_rule_need_uppercase: true,
                    sk_rule_need_lowercase: true,
                    sk_rule_need_spec_char: true,
                    sk_lock_cycle_sec: 60,
                    sk_lock_err_times: 2,
                    sk_lock_duration_sec: 60,
                    repeatable: false,
                    expire_sec: 6000,
                },
                cert_conf_by_phone_vcode: false,
                cert_conf_by_mail_vcode: false,
                disabled: None,
                account_self_reg: None,
                cert_conf_by_oauth2: None,
                cert_conf_by_ldap: None,
            },
        )
        .await
        .code
        .starts_with("400"));

    assert!(client
        .post_resp::<IamTenantAggAddReq, String>(
            "/cs/tenant",
            &IamTenantAggAddReq {
                name: TrimString("测试公司1".to_string()),
                icon: Some("https://oss.minio.io/xxx.icon".to_string()),
                contact_phone: None,
                note: None,
                admin_name: TrimString("测试管理员".to_string()),
                admin_username: TrimString("tenant_admin".to_string()),
                admin_password: Some("aa22A#".to_string()),
                cert_conf_by_user_pwd: IamCertConfUserPwdAddOrModifyReq {
                    ak_rule_len_min: 2,
                    ak_rule_len_max: 20,
                    sk_rule_len_min: 7,
                    sk_rule_len_max: 20,
                    sk_rule_need_num: true,
                    sk_rule_need_uppercase: true,
                    sk_rule_need_lowercase: true,
                    sk_rule_need_spec_char: true,
                    sk_lock_cycle_sec: 60,
                    sk_lock_err_times: 2,
                    sk_lock_duration_sec: 60,
                    repeatable: false,
                    expire_sec: 6000,
                },
                cert_conf_by_phone_vcode: false,
                cert_conf_by_mail_vcode: false,
                disabled: None,
                account_self_reg: None,
                cert_conf_by_oauth2: None,
                cert_conf_by_ldap: None,
            },
        )
        .await
        .code
        .starts_with("400"));

    let tenant_id: String = client
        .post(
            "/cs/tenant",
            &IamTenantAggAddReq {
                name: TrimString("测试公司1".to_string()),
                icon: Some("https://oss.minio.io/xxx.icon".to_string()),
                contact_phone: None,
                note: None,
                admin_name: TrimString("测试管理员".to_string()),
                admin_username: TrimString("tenant_admin".to_string()),
                admin_password: Some("A3a#f".to_string()),
                cert_conf_by_user_pwd: IamCertConfUserPwdAddOrModifyReq {
                    ak_rule_len_min: 2,
                    ak_rule_len_max: 20,
                    sk_rule_len_min: 2,
                    sk_rule_len_max: 20,
                    sk_rule_need_num: true,
                    sk_rule_need_uppercase: true,
                    sk_rule_need_lowercase: true,
                    sk_rule_need_spec_char: true,
                    sk_lock_cycle_sec: 60,
                    sk_lock_err_times: 2,
                    sk_lock_duration_sec: 60,
                    repeatable: false,
                    expire_sec: 6000,
                },
                cert_conf_by_phone_vcode: false,
                cert_conf_by_mail_vcode: false,
                disabled: None,
                account_self_reg: None,
                cert_conf_by_oauth2: None,
                cert_conf_by_ldap: None,
            },
        )
        .await;
    sleep(Duration::from_secs(1)).await;
    login_page("tenant_admin", "A3a#f", Some(tenant_id.clone()), None, true, client).await?;

    Ok(())
}

pub async fn login_by_oauth2(client: &mut BIOSWebTestClient) -> TardisResult<()> {
    info!("【login_by_oauth2】");

    let app_id = "";
    let secret = "";
    let code = "";

    let tenant_id: String = client
        .post(
            "/cs/tenant",
            &IamTenantAggAddReq {
                name: TrimString("测试公司2".to_string()),
                icon: None,
                contact_phone: None,
                note: None,
                admin_name: TrimString("测试管理员".to_string()),
                admin_username: TrimString("tenant_admin".to_string()),
                admin_password: Some("123456".to_string()),
                cert_conf_by_user_pwd: IamCertConfUserPwdAddOrModifyReq {
                    ak_rule_len_min: 2,
                    ak_rule_len_max: 20,
                    sk_rule_len_min: 2,
                    sk_rule_len_max: 20,
                    sk_rule_need_num: false,
                    sk_rule_need_uppercase: false,
                    sk_rule_need_lowercase: false,
                    sk_rule_need_spec_char: false,
                    sk_lock_cycle_sec: 60,
                    sk_lock_err_times: 2,
                    sk_lock_duration_sec: 60,
                    repeatable: false,
                    expire_sec: 6000,
                },
                cert_conf_by_phone_vcode: false,
                cert_conf_by_mail_vcode: false,
                disabled: None,
                account_self_reg: Some(true),
                cert_conf_by_oauth2: Some(vec![IamCertConfOAuth2AddOrModifyReq {
                    supplier: TrimString(IamCertOAuth2Supplier::WechatMp.to_string()),
                    ak: TrimString(app_id.to_string()),
                    sk: TrimString(secret.to_string()),
                }]),
                cert_conf_by_ldap: None,
            },
        )
        .await;
    sleep(Duration::from_secs(1)).await;
    let ak: String = client.get(&format!("/cp/ak/wechat-mp/{}", tenant_id)).await;
    assert_eq!(app_id, ak);

    let account: IamAccountInfoResp = client
        .put(
            "/cp/login/wechat-mp",
            &IamCpOAuth2LoginReq {
                code: TrimString(code.to_string()),
                tenant_id,
            },
        )
        .await;
    assert_eq!(account.account_name, "");
    assert!(account.access_token.is_some());
    assert!(account.roles.is_empty());
    Ok(())
}

pub async fn login_by_ldap(client: &mut BIOSWebTestClient) -> TardisResult<()> {
    const LDAP_SUPPLIER: &str = "TEST_LDAP";
    const LDAP_SUPPLIER2: &str = "TEST_LDAP2";
    let ldap_user1 = "Barbara";
    let ldap_user1_pwd = "123456";
    let ldap_user2 = "testUser";
    let ldap_user2_pwd = "123456";
    let ldap_user3 = "testUser1";
    let ldap_user3_pwd = "123456";
    let ldap_user4 = "testUser2";
    let ldap_user4_pwd = "123456";

    let global_user1 = "global_user1";

    info!("【login_by_ldap】prepare");
    //=======preparation area===========
    let funs = iam_constants::get_tardis_inst();
    let ctx = TardisContext {
        own_paths: "".to_string(),
        ak: "".to_string(),
        owner: "".to_string(),
        roles: vec![],
        groups: vec![],
        ..Default::default()
    };
    //add global account ldap login config
    IamCertLdapServ::add_cert_conf(
        &&IamCertConfLdapAddOrModifyReq {
            supplier: TrimString(LDAP_SUPPLIER.into()),
            name: LDAP_SUPPLIER.into(),
            conn_uri: env::var("TARDIS_FW.LDAP.URL").unwrap(),
            is_tls: false,
            principal: TrimString(env::var("TARDIS_FW.LDAP.ADMIN_CN").unwrap_or("".to_string())),
            credentials: TrimString(env::var("TARDIS_FW.LDAP.ADMIN_PASSWORD").unwrap_or("".to_string())),
            base_dn: env::var("TARDIS_FW.LDAP.BASE_DN").unwrap_or("".to_string()),
            field_display_name: "displayName".to_string(),
            search_base_filter: "objectClass=*".to_string(),
        },
        None,
        &funs,
        &ctx,
    )
    .await?;

    let global_account_id: String = client
        .post(
            "/ct/account",
            &IamAccountAggAddReq {
                id: None,
                name: TrimString("用户2".to_string()),
                cert_user_name: TrimString(global_user1.to_string()),
                cert_password: TrimString("1234567".to_string()),
                cert_phone: None,
                cert_mail: None,
                role_ids: None,
                org_node_ids: None,
                scope_level: Some(RBUM_SCOPE_LEVEL_TENANT),
                disabled: None,
                icon: None,
                exts: HashMap::from([("ext9".to_string(), "00001".to_string())]),
                status: None,
            },
        )
        .await;

    let tenant1_id: String = client
        .post(
            "/cs/tenant",
            &IamTenantAggAddReq {
                name: TrimString("测试公司2".to_string()),
                icon: None,
                contact_phone: None,
                note: None,
                admin_name: TrimString("测试管理员2".to_string()),
                admin_username: TrimString("tenant_admin2".to_string()),
                admin_password: Some("123456".to_string()),
                cert_conf_by_user_pwd: IamCertConfUserPwdAddOrModifyReq {
                    ak_rule_len_min: 2,
                    ak_rule_len_max: 20,
                    sk_rule_len_min: 2,
                    sk_rule_len_max: 20,
                    sk_rule_need_num: false,
                    sk_rule_need_uppercase: false,
                    sk_rule_need_lowercase: false,
                    sk_rule_need_spec_char: false,
                    sk_lock_cycle_sec: 60,
                    sk_lock_err_times: 2,
                    sk_lock_duration_sec: 60,
                    repeatable: false,
                    expire_sec: 6000,
                },
                cert_conf_by_phone_vcode: false,
                cert_conf_by_mail_vcode: false,
                disabled: None,
                account_self_reg: Some(true),
                cert_conf_by_oauth2: None,
                cert_conf_by_ldap: Some(vec![IamCertConfLdapAddOrModifyReq {
                    supplier: TrimString(LDAP_SUPPLIER2.to_string()),
                    name: LDAP_SUPPLIER2.to_string(),
                    conn_uri: env::var("TARDIS_FW.LDAP.URL").unwrap(),
                    is_tls: false,
                    principal: TrimString(env::var("TARDIS_FW.LDAP.ADMIN_CN").unwrap_or("".to_string())),
                    credentials: TrimString(env::var("TARDIS_FW.LDAP.ADMIN_PASSWORD").unwrap_or("".to_string())),
                    base_dn: env::var("TARDIS_FW.LDAP.BASE_DN").unwrap_or("".to_string()),
                    field_display_name: "displayName".to_string(),
                    search_base_filter: "objectClass=*".to_string(),
                }]),
            },
        )
        .await;
    sleep(Duration::from_secs(1)).await;
    login_page("tenant_admin2", "123456", Some(tenant1_id.clone()), None, true, client).await?;

    let tenant1_user1 = "user3";
    let tenant1_user1_pwd = "1234567";
    let tenant1_account_id: String = client
        .post(
            "/ct/account",
            &IamAccountAggAddReq {
                id: None,
                name: TrimString("用户3".to_string()),
                cert_user_name: TrimString(tenant1_user1.to_string()),
                cert_password: TrimString(tenant1_user1_pwd.to_string()),
                cert_phone: None,
                cert_mail: None,
                role_ids: None,
                org_node_ids: None,
                scope_level: Some(RBUM_SCOPE_LEVEL_TENANT),
                disabled: None,
                icon: None,
                exts: HashMap::from([("ext9".to_string(), "00001".to_string())]),
                status: None,
            },
        )
        .await;
    sleep(Duration::from_secs(1)).await;

    //=======end preparation area===========

    //=======Test multiple area============
    info!("test global_user1 login and bind test");
    do_test_ldap(
        client,
        LDAP_SUPPLIER,
        Some(tenant1_id.clone()),
        ldap_user1,
        ldap_user1_pwd,
        ldap_user2,
        ldap_user2_pwd,
        global_user1,
    )
    .await?;

    info!("test tenant level ldap user login and bind test");
    do_test_ldap(
        client,
        LDAP_SUPPLIER2,
        Some(tenant1_id),
        ldap_user3,
        ldap_user3_pwd,
        ldap_user4,
        ldap_user4_pwd,
        tenant1_user1,
    )
    .await?;

    //=======End test multiple area============

    info!("ldap login test succeed");
    Ok(())
}

pub async fn do_test_ldap(
    client: &mut BIOSWebTestClient,
    ldap_supplier: &str,
    tenant_id: Option<String>,
    ldap_user: &str,
    ldap_user_pwd: &str,
    ldap_user2: &str,
    ldap_user2_pwd: &str,
    ak_user: &str,
) -> TardisResult<()> {
    assert!(client
        .post_resp::<IamCpUserPwdCheckReq, String>(
            "/cp/ldap/check-bind",
            &IamCpUserPwdCheckReq {
                ak: "tugherugfqeyvfb".into(),
                code: ldap_supplier.into(),
                tenant_id: tenant_id.clone(),
            },
        )
        .await
        .code
        .starts_with("404"));

    assert!(client
        .post_resp::<IamCpUserPwdCheckReq, String>(
            "/cp/ldap/check-bind",
            &IamCpUserPwdCheckReq {
                ak: "user1".into(),
                code: ldap_supplier.into(),
                tenant_id: tenant_id.clone(),
            },
        )
        .await
        .code
        .starts_with("404"));

    info!("test error login");
    assert!(client
        .put_resp::<IamCpLdapLoginReq, String>(
            "/cp/ldap/login",
            &IamCpLdapLoginReq {
                code: TrimString(ldap_supplier.to_string()),
                name: "dftrvtr".into(),
                password: ldap_user_pwd.to_string(),
                tenant_id: tenant_id.clone(),
            },
        )
        .await
        .code
        .starts_with("401"));

    info!("test ldap_user:{} login,and not bind", ldap_user);
    let account: IamAccountInfoWithUserPwdAkResp = client
        .put(
            "/cp/ldap/login",
            &IamCpLdapLoginReq {
                code: TrimString(ldap_supplier.to_string()),
                name: ldap_user.into(),
                password: ldap_user_pwd.to_string(),
                tenant_id: tenant_id.clone(),
            },
        )
        .await;

    assert_eq!(account.iam_account_info_resp.account_name, "");
    assert!(account.iam_account_info_resp.access_token.is_none());

    info!("test ak_user:{} check-bind", ak_user);
    let user_pwd_bind_resp: IamCpUserPwdBindResp = client
        .post(
            "/cp/ldap/check-bind",
            &IamCpUserPwdCheckReq {
                ak: ak_user.into(),
                code: ldap_supplier.into(),
                tenant_id: tenant_id.clone(),
            },
        )
        .await;

    assert!(!user_pwd_bind_resp.is_bind);

    info!("ldap_user:{} create aksk ", ldap_user);
    let user1_account: IamAccountInfoWithUserPwdAkResp = client
        .put(
            "/cp/ldap/bind-or-create-userpwd",
            &IamCpUserPwdBindWithLdapReq {
                bind_user_pwd: IamCpUserPwdBindReq { ak: None, sk: "123456".into() },
                ldap_login: IamCpLdapLoginReq {
                    code: ldap_supplier.into(),
                    name: ldap_user.into(),
                    password: ldap_user_pwd.to_string(),
                    tenant_id: tenant_id.clone(),
                },
                tenant_id: tenant_id.clone(),
            },
        )
        .await;

    assert!(user1_account.iam_account_info_resp.access_token.is_some());
    assert!(!user1_account.iam_account_info_resp.account_name.is_empty());
    assert!(!user1_account.iam_account_info_resp.account_id.is_empty());
    assert_eq!(user1_account.status, "Pending");

    //set user1 auth
    client.set_auth(&user1_account.iam_account_info_resp.token, None).await?;
    //rest user1 pwd
    let rest_user1_pwd = "34dfe31";
    let rest_user1_pwd_resp: TardisResp<Void> = client
        .put_resp(
            &format!("/cp/cert/userpwd/reset?account_id={}", user1_account.iam_account_info_resp.account_id),
            &IamCertUserPwdRestReq { new_sk: rest_user1_pwd.into() },
        )
        .await;
    assert_eq!(rest_user1_pwd_resp.code, "200");

    sleep(Duration::from_secs(1)).await;
    //relogin user1 by userpwd
    let account: IamAccountInfoResp = client
        .put(
            "/cp/login/userpwd",
            &IamCpUserPwdLoginReq {
                ak: TrimString(user1_account.ak.to_string()),
                sk: TrimString(rest_user1_pwd.to_string()),
                tenant_id: tenant_id.clone(),
                flag: None,
            },
        )
        .await;
    info!("relogin user1 by userpwd resp:{:?}", account);

    //relogin user1 by ldap
    let account: IamAccountInfoWithUserPwdAkResp = client
        .put(
            "/cp/ldap/login",
            &IamCpLdapLoginReq {
                code: TrimString(ldap_supplier.to_string()),
                name: ldap_user.into(),
                password: ldap_user_pwd.to_string(),
                tenant_id: tenant_id.clone(),
            },
        )
        .await;
    info!("relogin user1 by ldap resp:{:?}", account);
    assert!(account.iam_account_info_resp.access_token.is_some());
    assert!(!account.iam_account_info_resp.account_name.is_empty());
    assert!(!account.iam_account_info_resp.account_id.is_empty());
    assert_eq!(account.status, "Enabled");

    info!("bind ldap_user2 with ak_user:{}", ak_user);
    let account: IamAccountInfoWithUserPwdAkResp = client
        .put(
            "/cp/ldap/bind-or-create-userpwd",
            &IamCpUserPwdBindWithLdapReq {
                bind_user_pwd: IamCpUserPwdBindReq {
                    ak: Some(ak_user.into()),
                    sk: "1234567".into(),
                },
                ldap_login: IamCpLdapLoginReq {
                    code: ldap_supplier.into(),
                    name: ldap_user2.into(),
                    password: ldap_user2_pwd.to_string(),
                    tenant_id: tenant_id.clone(),
                },
                tenant_id: tenant_id.clone(),
            },
        )
        .await;
    info!("relogin user2 by ldap resp:{:?}", account);

    assert!(!account.iam_account_info_resp.account_id.is_empty());
    assert!(account.iam_account_info_resp.access_token.is_some());
    assert!(!account.iam_account_info_resp.account_name.is_empty());
    assert_eq!(account.status, "Enabled");

    let user_pwd_bind_resp: IamCpUserPwdBindResp = client
        .post(
            "/cp/ldap/check-bind",
            &IamCpUserPwdCheckReq {
                ak: ak_user.into(),
                code: ldap_supplier.into(),
                tenant_id: tenant_id.clone(),
            },
        )
        .await;

    assert!(user_pwd_bind_resp.is_bind);

    Ok(())
}
