use std::collections::HashMap;
use std::time::Duration;

use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;
use tardis::tokio::time::sleep;
use tardis::web::web_resp::{TardisPage, Void};

use bios_basic::rbum::dto::rbum_cert_conf_dto::RbumCertConfDetailResp;
use bios_basic::rbum::dto::rbum_cert_dto::RbumCertSummaryResp;
use bios_basic::rbum::dto::rbum_kind_attr_dto::{RbumKindAttrDetailResp, RbumKindAttrModifyReq, RbumKindAttrSummaryResp};
use bios_basic::rbum::dto::rbum_rel_dto::RbumRelBoneResp;
use bios_basic::rbum::dto::rbum_set_cate_dto::RbumSetTreeResp;
use bios_basic::rbum::dto::rbum_set_dto::RbumSetPathResp;
use bios_basic::rbum::dto::rbum_set_item_dto::RbumSetItemSummaryResp;
use bios_basic::rbum::rbum_enumeration::{RbumDataTypeKind, RbumWidgetTypeKind};
use bios_iam::basic::dto::iam_account_dto::{
    IamAccountAggAddReq, IamAccountAggModifyReq, IamAccountBoneResp, IamAccountDetailResp, IamAccountInfoResp, IamAccountSelfModifyReq, IamAccountSummaryResp,
};
use bios_iam::basic::dto::iam_app_dto::IamAppDetailResp;
use bios_iam::basic::dto::iam_attr_dto::IamKindAttrAddReq;
use bios_iam::basic::dto::iam_cert_conf_dto::{IamMailVCodeCertConfAddOrModifyReq, IamPhoneVCodeCertConfAddOrModifyReq, IamUserPwdCertConfAddOrModifyReq};
use bios_iam::basic::dto::iam_cert_dto::{IamUserPwdCertModifyReq, IamUserPwdCertRestReq};
use bios_iam::basic::dto::iam_res_dto::{IamResAddReq, IamResAggAddReq, IamResDetailResp, IamResModifyReq};
use bios_iam::basic::dto::iam_role_dto::{IamRoleAddReq, IamRoleAggAddReq, IamRoleAggModifyReq, IamRoleBoneResp, IamRoleDetailResp, IamRoleModifyReq, IamRoleSummaryResp};
use bios_iam::basic::dto::iam_set_dto::{IamSetCateAddReq, IamSetCateModifyReq, IamSetItemAggAddReq, IamSetItemWithDefaultSetAddReq};
use bios_iam::basic::dto::iam_tenant_dto::{IamTenantBoneResp, IamTenantDetailResp, IamTenantModifyReq, IamTenantSummaryResp};
use bios_iam::console_app::dto::iam_ca_app_dto::IamCaAppModifyReq;
use bios_iam::console_passport::dto::iam_cp_cert_dto::IamCpUserPwdLoginReq;
use bios_iam::console_system::dto::iam_cs_tenant_dto::IamCsTenantAddReq;
use bios_iam::console_tenant::dto::iam_ct_app_dto::IamCtAppAddReq;
use bios_iam::iam_constants::{RBUM_SCOPE_LEVEL_APP, RBUM_SCOPE_LEVEL_GLOBAL, RBUM_SCOPE_LEVEL_TENANT};
use bios_iam::iam_enumeration::{IamCertKind, IamCertTokenKind, IamResKind};
use bios_iam::iam_test_helper::BIOSWebTestClient;

pub async fn test(client: &mut BIOSWebTestClient, sysadmin_name: &str, sysadmin_password: &str) -> TardisResult<()> {
    login_page(client, &tenant_admin_user_name, &tenant_admin_password, Some(tenant_id.clone()), None, true).await?;
    tenant_console_tenant_mgr_page(client).await?;
    tenant_console_org_mgr_page(client, &tenant_admin_user_name, &tenant_admin_password, &tenant_id).await?;
    tenant_console_account_mgr_page(client).await?;
    tenant_console_auth_mgr_page(client).await?;
    app_console_project_mgr_page(client, &tenant_id).await?;
    app_console_auth_mgr_page(client).await?;
    passport_console_account_mgr_page(client).await?;
    passport_console_security_mgr_page(client).await?;
    common_console_opt(client).await?;
    Ok(())
}

pub async fn tenant_console_tenant_mgr_page(client: &mut BIOSWebTestClient) -> TardisResult<()> {
    info!("【tenant_console_tenant_mgr_page】");

    // Get Current Tenant
    let tenant: IamTenantDetailResp = client.get("/ct/tenant").await;
    assert_eq!(tenant.name, "测试公司_new");
    assert_eq!(tenant.icon, "https://oss.minio.io/xxx.icon");

    // Find Cert Conf by Current Tenant
    let cert_conf: Vec<RbumCertConfDetailResp> = client.get("/ct/cert-conf").await;
    let cert_conf_user_pwd = cert_conf.iter().find(|x| x.code == IamCertKind::UserPwd.to_string()).unwrap();
    let _cert_conf_mail_vcode = cert_conf.iter().find(|x| x.code == IamCertKind::MailVCode.to_string()).unwrap();
    assert_eq!(cert_conf.len(), 2);
    assert!(cert_conf_user_pwd.sk_encrypted);
    assert!(!cert_conf_user_pwd.repeatable);

    // Modify Current Tenant
    let _: Void = client
        .put(
            "/ct/tenant",
            &IamTenantModifyReq {
                name: Some(TrimString("测试公司".to_string())),
                scope_level: None,
                disabled: None,
                icon: None,
                sort: None,
                contact_phone: None,
                note: None,
            },
        )
        .await;
    let tenant: IamTenantDetailResp = client.get("/ct/tenant").await;
    assert_eq!(tenant.name, "测试公司");

    // Modify Cert Conf by User Pwd Id
    let _: Void = client
        .put(
            &format!("/ct/cert-conf/{}/user-pwd", cert_conf_user_pwd.id),
            &IamUserPwdCertConfAddOrModifyReq {
                ak_note: None,
                ak_rule: None,
                sk_note: None,
                sk_rule: None,
                repeatable: Some(false),
                expire_sec: Some(111),
            },
        )
        .await;

    // Add Cert Conf by Tenant Id
    let _: Void = client.post("/ct/cert-conf/phone-vcode", &IamPhoneVCodeCertConfAddOrModifyReq { ak_note: None, ak_rule: None }).await;
    let cert_conf: Vec<RbumCertConfDetailResp> = client.get("/ct/cert-conf").await;
    assert_eq!(cert_conf.len(), 3);

    Ok(())
}

pub async fn tenant_console_org_mgr_page(client: &mut BIOSWebTestClient, tenant_admin_user_name: &str, tenant_admin_password: &str, tenant_id: &str) -> TardisResult<()> {
    info!("【tenant_console_org_mgr_page】");

    // Find Org Cates By Current Tenant
    let res_tree: Vec<RbumSetTreeResp> = client.get("/ct/org/tree").await;
    assert_eq!(res_tree.len(), 0);

    // Add Org Cate
    let cate_node1_id: String = client
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
    let cate_node2_id: String = client
        .post(
            "/ct/org/cate",
            &IamSetCateAddReq {
                name: TrimString("综合服务".to_string()),
                scope_level: Some(RBUM_SCOPE_LEVEL_TENANT),
                bus_code: None,
                icon: None,
                sort: None,
                ext: None,
                rbum_parent_cate_id: None,
            },
        )
        .await;

    // Delete Org Cate By Org Id
    client.delete(&format!("/ct/org/cate/{}", cate_node2_id)).await;

    // Modify Org Cate By Org Id
    let _: Void = client
        .put(
            &format!("/ct/org/cate/{}", cate_node1_id),
            &IamSetCateModifyReq {
                name: Some(TrimString("综合服务中心".to_string())),
                scope_level: None,
                bus_code: None,
                icon: None,
                sort: None,
                ext: None,
            },
        )
        .await;
    let res_tree: Vec<RbumSetTreeResp> = client.get("/ct/org/tree").await;
    assert_eq!(res_tree.len(), 1);
    assert_eq!(res_tree.get(0).unwrap().name, "综合服务中心");

    // Count Accounts
    let accounts: u64 = client.get("/ct/account/total").await;
    assert_eq!(accounts, 2);

    // Find Accounts
    let accounts: TardisPage<IamAccountSummaryResp> = client.get("/ct/account?page_number=1&page_size=10").await;
    assert_eq!(accounts.total_size, 2);
    let account_id = accounts.records.iter().find(|i| i.name == "测试管理员").unwrap().id.clone();

    // Find Role By Account Id
    let roles: Vec<RbumRelBoneResp> = client.get(&format!("/ct/account/{}/role", account_id)).await;
    assert_eq!(roles.len(), 1);
    assert_eq!(roles.get(0).unwrap().rel_name, "tenant_admin");

    // Find Set Paths By Account Id
    let roles: Vec<Vec<Vec<RbumSetPathResp>>> = client.get(&format!("/ct/account/{}/set-path", account_id)).await;
    assert_eq!(roles.len(), 0);

    // Find Certs By Account Id
    let certs: Vec<RbumCertSummaryResp> = client.get(&format!("/ct/cert?account_id={}", account_id)).await;
    assert_eq!(certs.len(), 1);
    assert!(certs.into_iter().any(|i| i.rel_rbum_cert_conf_code == Some("UserPwd".to_string())));

    // Add Org Item
    let _: String = client
        .put(
            "/ct/org/item",
            &IamSetItemWithDefaultSetAddReq {
                set_cate_id: cate_node1_id.to_string(),
                sort: 0,
                rel_rbum_item_id: account_id.clone(),
            },
        )
        .await;

    // Find Org Items
    let items: Vec<RbumSetItemSummaryResp> = client.get(&format!("/ct/org/item?cate_id={}", cate_node1_id)).await;
    assert_eq!(items.len(), 1);

    login_page(client, tenant_admin_user_name, tenant_admin_password, Some(tenant_id.to_string()), None, true).await?;
    assert_eq!(client.context().groups.len(), 1);
    assert!(client.context().groups.get(0).unwrap().contains(":aaaa"));

    // Delete Org Item By Org Item Id
    client.delete(&format!("/ct/org/item/{}", items.get(0).unwrap().id)).await;
    let items: Vec<RbumSetItemSummaryResp> = client.get(&format!("/ct/org/item?cate_id={}", cate_node1_id)).await;
    assert_eq!(items.len(), 0);

    login_page(client, tenant_admin_user_name, tenant_admin_password, Some(tenant_id.to_string()), None, true).await?;
    assert_eq!(client.context().groups.len(), 0);

    Ok(())
}

pub async fn tenant_console_account_mgr_page(client: &mut BIOSWebTestClient) -> TardisResult<()> {
    info!("【tenant_console_account_mgr_page】");

    // Find Accounts
    let accounts: TardisPage<IamAccountSummaryResp> = client.get("/ct/account?page_number=1&page_size=10").await;
    assert_eq!(accounts.total_size, 2);
    let account_id = accounts.records.iter().find(|i| i.name == "测试管理员").unwrap().id.clone();

    // Find Certs By Account Id
    let certs: Vec<RbumCertSummaryResp> = client.get(&format!("/ct/cert?account_id={}", account_id)).await;
    assert_eq!(certs.len(), 1);
    assert!(certs.into_iter().any(|i| i.rel_rbum_cert_conf_code == Some("UserPwd".to_string())));

    // Find Role By Account Id
    let roles: Vec<RbumRelBoneResp> = client.get(&format!("/ct/account/{}/role", account_id)).await;
    assert_eq!(roles.len(), 1);
    assert_eq!(roles.get(0).unwrap().rel_name, "tenant_admin");

    // Find Set Paths By Account Id
    let roles: Vec<Vec<Vec<RbumSetPathResp>>> = client.get(&format!("/ct/account/{}/set-path", account_id)).await;
    assert_eq!(roles.len(), 0);

    // Find Org Cates By Current Tenant
    let res_tree: Vec<RbumSetTreeResp> = client.get("/ct/org/tree").await;
    assert_eq!(res_tree.len(), 1);

    // Find Roles
    let roles: TardisPage<IamRoleSummaryResp> = client.get("/ct/role?with_sub=true&page_number=1&page_size=10").await;
    let role_id = &roles.records.iter().find(|i| i.name == "审计管理员").unwrap().id;
    assert_eq!(roles.total_size, 2);
    assert!(!roles.records.iter().any(|i| i.name == "sys_admin"));

    // Find Account Attrs By Current Tenant
    let attrs: Vec<RbumKindAttrSummaryResp> = client.get("/ct/account/attr").await;
    assert_eq!(attrs.len(), 1);

    // Add Account
    let account_id: String = client
        .post(
            "/ct/account",
            &IamAccountAggAddReq {
                id: None,
                name: TrimString("用户3".to_string()),
                cert_user_name: TrimString("user3".to_string()),
                cert_password: TrimString("123456".to_string()),
                cert_phone: None,
                cert_mail: Some(TrimString("gudaoxuri@outlook.com".to_string())),
                role_ids: Some(vec![role_id.to_string()]),
                scope_level: Some(RBUM_SCOPE_LEVEL_TENANT),
                disabled: None,
                icon: None,
                exts: HashMap::from([("ext1_idx".to_string(), "00001".to_string())]),
            },
        )
        .await;

    // Get Account By Account Id
    let account: IamAccountDetailResp = client.get(&format!("/ct/account/{}", account_id)).await;
    assert_eq!(account.name, "用户3");
    // Find Account Attr Value By Account Id
    let account_attrs: HashMap<String, String> = client.get(&format!("/ct/account/attr/value?account_id={}", account_id)).await;
    assert_eq!(account_attrs.len(), 1);
    assert_eq!(account_attrs.get("ext1_idx"), Some(&"00001".to_string()));

    // Modify Account By Account Id
    let _: Void = client
        .put(
            &format!("/ct/account/{}", account_id),
            &IamAccountAggModifyReq {
                name: Some(TrimString("用户3_new".to_string())),
                scope_level: None,
                disabled: None,
                icon: None,
                role_ids: Some(vec![]),
                exts: HashMap::from([("ext1_idx".to_string(), "".to_string())]),
            },
        )
        .await;

    // Get Account By Account Id
    let account: IamAccountDetailResp = client.get(&format!("/ct/account/{}", account_id)).await;
    assert_eq!(account.name, "用户3_new");

    // Find Account Attr By Account Id
    let account_attrs: HashMap<String, String> = client.get(&format!("/ct/account/attr/value?account_id={}", account_id)).await;
    assert_eq!(account_attrs.len(), 1);
    assert_eq!(account_attrs.get("ext1_idx"), Some(&"".to_string()));

    // Delete Account By Account Id
    let _ = client.delete(&format!("/ct/account/{}", account_id)).await;

    // Rest Password By Account Id
    let _: Void = client
        .put(
            &format!("/ct/cert/user-pwd?account_id={}", account_id),
            &IamUserPwdCertRestReq {
                new_sk: TrimString("123456".to_string()),
            },
        )
        .await;

    Ok(())
}

pub async fn tenant_console_auth_mgr_page(client: &mut BIOSWebTestClient) -> TardisResult<()> {
    info!("【tenant_console_auth_mgr_page】");

    // Find Accounts
    let accounts: TardisPage<IamAccountSummaryResp> = client.get("/ct/account?page_number=1&page_size=10").await;
    assert_eq!(accounts.total_size, 3);
    let account_id = accounts.records.iter().find(|i| i.name == "测试管理员").unwrap().id.clone();

    // Find Roles
    let roles: TardisPage<IamRoleSummaryResp> = client.get("/ct/role?with_sub=true&page_number=1&page_size=10").await;
    assert_eq!(roles.total_size, 2);
    assert!(!roles.records.iter().any(|i| i.name == "sys_admin"));

    // Find Res Tree
    let res_tree: Vec<RbumSetTreeResp> = client.get("/ct/res/tree?sys_res=true").await;
    assert_eq!(res_tree.len(), 3);
    let res = res_tree.iter().find(|i| i.name == "个人工作台").unwrap().rbum_set_items.get(0).unwrap();
    assert_eq!(res.rel_rbum_item_name, "工作台页面");
    let res_id = res.rel_rbum_item_id.clone();

    // Add Role
    let role_id: String = client
        .post(
            "/ct/role",
            &IamRoleAggAddReq {
                role: IamRoleAddReq {
                    code: TrimString("role5".to_string()),
                    name: TrimString("角色5".to_string()),
                    scope_level: Some(RBUM_SCOPE_LEVEL_TENANT),
                    disabled: None,
                    icon: None,
                    sort: None,
                },
                res_ids: Some(vec![res_id.clone()]),
            },
        )
        .await;

    // Get Role By Role Id
    let role: IamRoleDetailResp = client.get(&format!("/ct/role/{}", role_id)).await;
    assert_eq!(role.name, "角色5");

    // Find Res By Role Id
    let res: Vec<RbumRelBoneResp> = client.get(&format!("/ct/role/{}/res", role_id)).await;
    assert_eq!(res.len(), 1);
    assert_eq!(res.get(0).unwrap().rel_name, "工作台页面");

    // Modify Role by Role Id
    let _: Void = client
        .put(
            &format!("/ct/role/{}", role_id),
            &IamRoleAggModifyReq {
                role: IamRoleModifyReq {
                    name: Some(TrimString("xx角色".to_string())),
                    scope_level: None,
                    disabled: None,
                    icon: None,
                    sort: None,
                },
                res_ids: Some(vec![]),
            },
        )
        .await;

    // Get Role By Role Id
    let role: IamRoleDetailResp = client.get(&format!("/ct/role/{}", role_id)).await;
    assert_eq!(role.name, "xx角色");

    // Find Res By Role Id
    let res: Vec<RbumRelBoneResp> = client.get(&format!("/ct/role/{}/res", role_id)).await;
    assert_eq!(res.len(), 0);

    // Add Account To Role
    let _: Void = client.put(&format!("/ct/role/{}/account/{}", role_id, account_id), &Void {}).await;

    // Find Accounts By Role Id
    let accounts: TardisPage<IamAccountSummaryResp> = client.get(&format!("/ct/account?role_id={}&with_sub=false&page_number=1&page_size=10", role_id)).await;
    assert_eq!(accounts.total_size, 1);
    assert_eq!(accounts.records.get(0).unwrap().name, "测试管理员");

    // Count Account By Role Id
    let accounts: u64 = client.get(&format!("/ct/role/{}/account/total", role_id)).await;
    assert_eq!(accounts, 1);

    // Delete Account By Res Id
    client.delete(&format!("/ct/role/{}/account/{}", role_id, account_id)).await;
    let accounts: u64 = client.get(&format!("/ct/role/{}/account/total", role_id)).await;
    assert_eq!(accounts, 0);

    Ok(())
}

pub async fn app_console_project_mgr_page(client: &mut BIOSWebTestClient, tenant_id: &str) -> TardisResult<()> {
    info!("【app_console_project_mgr_page】");

    // Add Account
    let app_account_id: String = client
        .post(
            "/ct/account",
            &IamAccountAggAddReq {
                id: None,
                name: TrimString("devops应用管理员".to_string()),
                cert_user_name: TrimString("user_dp".to_string()),
                cert_password: TrimString("123456".to_string()),
                cert_phone: None,
                cert_mail: Some(TrimString("devopsxxx@xx.com".to_string())),
                role_ids: None,
                scope_level: Some(RBUM_SCOPE_LEVEL_TENANT),
                disabled: None,
                icon: None,
                exts: HashMap::from([("ext1_idx".to_string(), "00002".to_string())]),
            },
        )
        .await;

    // Add App
    let app_id: String = client
        .post(
            "/ct/app",
            &IamCtAppAddReq {
                app_name: TrimString("devops project".to_string()),
                app_icon: None,
                app_sort: None,
                app_contact_phone: None,
                admin_id: app_account_id.clone(),
                disabled: None,
            },
        )
        .await;

    sleep(Duration::from_secs(1)).await;
    login_page(client, "user_dp", "123456", Some(tenant_id.to_string()), Some(app_id.clone()), true).await?;
    assert_eq!(client.context().roles.len(), 1);
    assert_eq!(client.context().own_paths, format!("{}/{}", tenant_id, app_id));

    // Modify App
    let _: Void = client
        .put(
            "/ca/app",
            &IamCaAppModifyReq {
                name: Some(TrimString("DevOps项目".to_string())),
                icon: None,
                sort: None,
                contact_phone: None,
                disabled: None,
            },
        )
        .await;

    // Get App
    let app: IamAppDetailResp = client.get("/ca/app").await;
    assert_eq!(app.name, "DevOps项目");

    Ok(())
}

pub async fn app_console_auth_mgr_page(client: &mut BIOSWebTestClient) -> TardisResult<()> {
    info!("【app_console_auth_mgr_page】");

    // Find Accounts
    let accounts: TardisPage<IamAccountSummaryResp> = client.get("/ca/account?page_number=1&page_size=10").await;
    assert_eq!(accounts.total_size, 1);
    let account_id = accounts.records.iter().find(|i| i.name == "devops应用管理员").unwrap().id.clone();

    // Find Roles
    let roles: TardisPage<IamRoleSummaryResp> = client.get("/ca/role?page_number=1&page_size=10").await;
    assert_eq!(roles.total_size, 4);
    assert!(roles.records.iter().any(|i| i.name == "app_admin"));

    // Find Res Tree
    let res_tree: Vec<RbumSetTreeResp> = client.get("/ca/res/tree?sys_res=true").await;
    assert_eq!(res_tree.len(), 3);
    let res = res_tree.iter().find(|i| i.name == "个人工作台").unwrap().rbum_set_items.get(0).unwrap();
    assert_eq!(res.rel_rbum_item_name, "工作台页面");
    let res_id = res.rel_rbum_item_id.clone();

    // Add Role
    let role_id: String = client
        .post(
            "/ca/role",
            &IamRoleAggAddReq {
                role: IamRoleAddReq {
                    code: TrimString("role_xxx".to_string()),
                    name: TrimString("自定义角色1".to_string()),
                    scope_level: Some(RBUM_SCOPE_LEVEL_APP),
                    disabled: None,
                    icon: None,
                    sort: None,
                },
                res_ids: Some(vec![res_id.clone()]),
            },
        )
        .await;

    // Get Role By Role Id
    let role: IamRoleDetailResp = client.get(&format!("/ca/role/{}", role_id)).await;
    assert_eq!(role.name, "自定义角色1");

    // Find Res By Role Id
    let res: Vec<RbumRelBoneResp> = client.get(&format!("/ca/role/{}/res", role_id)).await;
    assert_eq!(res.len(), 1);
    assert_eq!(res.get(0).unwrap().rel_name, "工作台页面");

    // Modify Role by Role Id
    let _: Void = client
        .put(
            &format!("/ca/role/{}", role_id),
            &IamRoleAggModifyReq {
                role: IamRoleModifyReq {
                    name: Some(TrimString("自定义角色new".to_string())),
                    scope_level: None,
                    disabled: None,
                    icon: None,
                    sort: None,
                },
                res_ids: Some(vec![]),
            },
        )
        .await;

    // Get Role By Role Id
    let role: IamRoleDetailResp = client.get(&format!("/ca/role/{}", role_id)).await;
    assert_eq!(role.name, "自定义角色new");

    // Find Res By Role Id
    let res: Vec<RbumRelBoneResp> = client.get(&format!("/ca/role/{}/res", role_id)).await;
    assert_eq!(res.len(), 0);

    // Add Account To Role
    let _: Void = client.put(&format!("/ca/role/{}/account/{}", role_id, account_id), &Void {}).await;

    // Find Accounts By Role Id
    let accounts: TardisPage<IamAccountSummaryResp> = client.get(&format!("/ca/account?role_id={}&with_sub=false&page_number=1&page_size=10", role_id)).await;
    assert_eq!(accounts.total_size, 1);
    assert_eq!(accounts.records.get(0).unwrap().name, "devops应用管理员");

    // Count Account By Role Id
    let accounts: u64 = client.get(&format!("/ca/role/{}/account/total", role_id)).await;
    assert_eq!(accounts, 1);

    // Delete Account By Res Id
    client.delete(&format!("/ca/role/{}/account/{}", role_id, account_id)).await;
    let accounts: u64 = client.get(&format!("/ca/role/{}/account/total", role_id)).await;
    assert_eq!(accounts, 0);

    Ok(())
}

pub async fn passport_console_account_mgr_page(client: &mut BIOSWebTestClient) -> TardisResult<()> {
    info!("【passport_console_account_mgr_page】");

    // Get Current Account
    let account: IamAccountDetailResp = client.get("/cp/account").await;
    assert_eq!(account.name, "devops应用管理员");

    // Get Current Tenant
    let tenant: Option<IamTenantSummaryResp> = client.get("/cp/tenant").await;
    assert_eq!(tenant.unwrap().name, "测试公司");

    // Find Certs By Current Account
    let certs: Vec<RbumCertSummaryResp> = client.get("/cp/cert").await;
    assert_eq!(certs.len(), 2);
    assert!(certs.into_iter().any(|i| i.rel_rbum_cert_conf_code == Some("UserPwd".to_string())));

    // Find Role By Current Account
    let roles: Vec<RbumRelBoneResp> = client.get("/cp/account/role").await;
    assert_eq!(roles.len(), 1);
    assert_eq!(roles.get(0).unwrap().rel_name, "app_admin");

    // Find Set Paths By Current Account
    let roles: Vec<Vec<Vec<RbumSetPathResp>>> = client.get("/cp/account/set-path?sys_org=true").await;
    assert_eq!(roles.len(), 0);

    // Find Account Attrs By Current Tenant
    let attrs: Vec<RbumKindAttrSummaryResp> = client.get("/cp/account/attr").await;
    assert_eq!(attrs.len(), 1);

    // Find Account Attr Value By Current Account
    let account_attrs: HashMap<String, String> = client.get("/cp/account/attr/value").await;
    assert_eq!(account_attrs.len(), 1);
    assert_eq!(account_attrs.get("ext1_idx"), Some(&"00002".to_string()));

    // Modify Account By Current Account
    let _: Void = client
        .put(
            "/cp/account",
            &IamAccountSelfModifyReq {
                name: Some(TrimString("测试管理员1".to_string())),
                disabled: None,
                icon: None,
                exts: HashMap::from([("ext1_idx".to_string(), "00001".to_string())]),
            },
        )
        .await;

    // Get Current Account
    let account: IamAccountDetailResp = client.get("/cp/account").await;
    assert_eq!(account.name, "测试管理员1");

    // Find Account Attr Value By Current Account
    let account_attrs: HashMap<String, String> = client.get("/cp/account/attr/value").await;
    assert_eq!(account_attrs.len(), 1);
    assert_eq!(account_attrs.get("ext1_idx"), Some(&"00001".to_string()));

    Ok(())
}

pub async fn passport_console_security_mgr_page(client: &mut BIOSWebTestClient) -> TardisResult<()> {
    info!("【passport_console_security_mgr_page】");

    // Modify Password
    let _: Void = client
        .put(
            "/cp/cert/userpwd",
            &IamUserPwdCertModifyReq {
                original_sk: TrimString("123456".to_string()),
                new_sk: TrimString("654321".to_string()),
            },
        )
        .await;

    Ok(())
}

pub async fn common_console_opt(client: &mut BIOSWebTestClient) -> TardisResult<()> {
    info!("【common_console_opt】");

    // Find Accounts
    let accounts: TardisPage<IamAccountBoneResp> = client.get("/cc/account?page_number=1&page_size=10").await;
    assert_eq!(accounts.total_size, 3);
    assert!(accounts.records.iter().any(|i| i.name == "测试管理员1"));

    // Find Roles
    let roles: TardisPage<IamRoleBoneResp> = client.get("/cc/role?page_number=1&page_size=10").await;
    assert_eq!(roles.total_size, 5);
    assert!(roles.records.iter().any(|i| i.name == "审计管理员"));

    Ok(())
}
