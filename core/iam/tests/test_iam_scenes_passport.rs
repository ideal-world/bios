use std::collections::HashMap;
use std::time::Duration;

use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;
use tardis::tokio::time::sleep;
use tardis::web::web_resp::{TardisResp, Void};

use bios_basic::rbum::dto::rbum_cert_dto::RbumCertSummaryResp;
use bios_basic::rbum::dto::rbum_kind_attr_dto::RbumKindAttrSummaryResp;
use bios_basic::rbum::rbum_enumeration::{RbumDataTypeKind, RbumWidgetTypeKind};
use bios_iam::basic::dto::iam_account_dto::{IamAccountAggAddReq, IamAccountInfoResp, IamAccountSelfModifyReq};
use bios_iam::basic::dto::iam_attr_dto::IamKindAttrAddReq;
use bios_iam::basic::dto::iam_cert_conf_dto::IamUserPwdCertConfAddOrModifyReq;
use bios_iam::basic::dto::iam_cert_dto::{IamPwdNewReq, IamUserPwdCertModifyReq};
use bios_iam::basic::dto::iam_set_dto::{IamSetCateAddReq, IamSetItemWithDefaultSetAddReq};
use bios_iam::basic::dto::iam_tenant_dto::IamTenantBoneResp;
use bios_iam::console_passport::dto::iam_cp_account_dto::IamCpAccountInfoResp;
use bios_iam::console_system::dto::iam_cs_tenant_dto::IamCsTenantAddReq;
use bios_iam::console_tenant::dto::iam_ct_app_dto::IamCtAppAddReq;
use bios_iam::iam_constants::RBUM_SCOPE_LEVEL_TENANT;
use bios_iam::iam_enumeration::IamCertTokenKind;
use bios_iam::iam_test_helper::BIOSWebTestClient;

pub async fn test(sysadmin_name: &str, sysadmin_password: &str, client: &mut BIOSWebTestClient) -> TardisResult<()> {
    info!("【test_iam_scenes_passport】");

    info!("【test_iam_scenes_passport:system】");
    login_page(sysadmin_name, sysadmin_password, None, None, true, client).await?;
    account_mgr_by_sys_admin(client).await?;
    security_mgr_page_by_sys_admin(sysadmin_name, sysadmin_password, client).await?;

    info!("【test_iam_scenes_passport:tenant】");
    let tenant_id: String = client
        .post(
            "/cs/tenant",
            &IamCsTenantAddReq {
                tenant_name: TrimString("测试公司1".to_string()),
                tenant_icon: Some("https://oss.minio.io/xxx.icon".to_string()),
                tenant_contact_phone: None,
                tenant_note: None,
                admin_name: TrimString("测试管理员".to_string()),
                admin_username: TrimString("tenant_admin".to_string()),
                admin_password: Some("123456".to_string()),
                cert_conf_by_user_pwd: IamUserPwdCertConfAddOrModifyReq {
                    ak_note: None,
                    ak_rule: None,
                    sk_note: None,
                    sk_rule: None,
                    repeatable: Some(false),
                    expire_sec: None,
                },
                cert_conf_by_phone_vcode: None,
                cert_conf_by_mail_vcode: None,
                disabled: None,
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
            &IamCtAppAddReq {
                app_name: TrimString("devops project".to_string()),
                app_icon: None,
                app_sort: None,
                app_contact_phone: None,
                admin_id: account_id.clone(),
                disabled: None,
            },
        )
        .await;
    sleep(Duration::from_secs(1)).await;
    login_page("user1", "123456", Some(tenant_id.clone()), Some(app_id.clone()), true, client).await?;
    account_mgr_by_app_account(client).await?;
    security_mgr_by_app_account("user1", "123456", &tenant_id, &app_id, client).await?;
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

    // Find Certs By Current Account
    let certs: Vec<RbumCertSummaryResp> = client.get("/cp/cert").await;
    assert_eq!(certs.len(), 1);
    assert!(certs.into_iter().any(|i| i.rel_rbum_cert_conf_code == Some("UserPwd".to_string())));

    // Find Account Attrs
    let attrs: Vec<RbumKindAttrSummaryResp> = client.get("/cp/account/attr").await;
    assert_eq!(attrs.len(), 0);

    // Find Account Attr Value
    let account_attrs: HashMap<String, String> = client.get("/cp/account/attr/value").await;
    assert_eq!(account_attrs.len(), 0);

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

    // Find Account Attr Value
    let account_attrs: HashMap<String, String> = client.get("/cp/account/attr/value").await;
    assert_eq!(account_attrs.len(), 0);

    Ok(())
}

pub async fn security_mgr_page_by_sys_admin(name: &str, password: &str, client: &mut BIOSWebTestClient) -> TardisResult<()> {
    info!("【security_mgr_page_by_sys_admin】");

    // Modify Password
    let _: Void = client
        .put(
            "/cp/cert/userpwd",
            &IamUserPwdCertModifyReq {
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
            &IamPwdNewReq {
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
            &IamPwdNewReq {
                ak: TrimString(name.to_string()),
                original_sk: TrimString("654321".to_string()),
                new_sk: TrimString("xxxxx".to_string()),
                tenant_id: None,
            },
        )
        .await;
    client.login(name, "xxxxx", None, None, Some(IamCertTokenKind::TokenPc.to_string()), true).await?;

    Ok(())
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

    // Find Certs By Current Account
    let certs: Vec<RbumCertSummaryResp> = client.get("/cp/cert").await;
    assert_eq!(certs.len(), 1);
    assert!(certs.into_iter().any(|i| i.rel_rbum_cert_conf_code == Some("UserPwd".to_string())));

    // Find Account Attrs By Current Tenant
    let attrs: Vec<RbumKindAttrSummaryResp> = client.get("/cp/account/attr").await;
    assert_eq!(attrs.len(), 0);

    // Find Account Attr Value By Current Account
    let account_attrs: HashMap<String, String> = client.get("/cp/account/attr/value").await;
    assert_eq!(account_attrs.len(), 0);

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

    // Find Account Attrs By Current Tenant
    let attrs: Vec<RbumKindAttrSummaryResp> = client.get("/cp/account/attr").await;
    assert_eq!(attrs.len(), 1);
    assert_eq!(attrs.iter().any(|i| i.name == "ext9"), true);

    // Find Account Attr Value By Current Account
    let account_attrs: HashMap<String, String> = client.get("/cp/account/attr/value").await;
    assert_eq!(account_attrs.len(), 1);
    assert_eq!(account_attrs.get("ext9"), Some(&"00001".to_string()));

    Ok(())
}

pub async fn security_mgr_by_tenant_account(name: &str, password: &str, tenant_id: &str, client: &mut BIOSWebTestClient) -> TardisResult<()> {
    info!("【security_mgr_by_tenant_account】");

    // Modify Password
    let _: Void = client
        .put(
            "/cp/cert/userpwd",
            &IamUserPwdCertModifyReq {
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
            &IamPwdNewReq {
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
            &IamPwdNewReq {
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
    assert!(account.org.get(0).unwrap().iter().any(|i| i.name == "综合服务中心"));

    // Find Certs By Current Account
    let certs: Vec<RbumCertSummaryResp> = client.get("/cp/cert").await;
    assert_eq!(certs.len(), 1);
    assert!(certs.into_iter().any(|i| i.rel_rbum_cert_conf_code == Some("UserPwd".to_string())));

    // Find Account Attrs By Current Tenant
    let attrs: Vec<RbumKindAttrSummaryResp> = client.get("/cp/account/attr").await;
    assert_eq!(attrs.len(), 1);
    assert_eq!(attrs.iter().any(|i| i.name == "ext9"), true);

    // Find Account Attr Value By Current Account
    let account_attrs: HashMap<String, String> = client.get("/cp/account/attr/value").await;
    assert_eq!(account_attrs.len(), 1);
    assert_eq!(account_attrs.get("ext9"), Some(&"00001".to_string()));

    Ok(())
}

pub async fn security_mgr_by_app_account(name: &str, password: &str, tenant_id: &str, app_id: &str, client: &mut BIOSWebTestClient) -> TardisResult<()> {
    info!("【security_mgr_by_app_account】");

    // Modify Password
    let _: Void = client
        .put(
            "/cp/cert/userpwd",
            &IamUserPwdCertModifyReq {
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
            &IamPwdNewReq {
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
            &IamPwdNewReq {
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
            &IamPwdNewReq {
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
