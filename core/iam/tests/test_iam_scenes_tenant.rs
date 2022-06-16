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
use bios_iam::basic::dto::iam_account_dto::{IamAccountAggAddReq, IamAccountAggModifyReq, IamAccountDetailResp, IamAccountSummaryResp};
use bios_iam::basic::dto::iam_attr_dto::IamKindAttrAddReq;
use bios_iam::basic::dto::iam_cert_conf_dto::{IamMailVCodeCertConfAddOrModifyReq, IamPhoneVCodeCertConfAddOrModifyReq, IamUserPwdCertConfAddOrModifyReq};
use bios_iam::basic::dto::iam_cert_dto::IamUserPwdCertRestReq;
use bios_iam::basic::dto::iam_role_dto::{IamRoleAddReq, IamRoleAggAddReq, IamRoleAggModifyReq, IamRoleDetailResp, IamRoleModifyReq, IamRoleSummaryResp};
use bios_iam::basic::dto::iam_set_dto::{IamSetCateAddReq, IamSetCateModifyReq, IamSetItemWithDefaultSetAddReq};
use bios_iam::basic::dto::iam_tenant_dto::{IamTenantDetailResp, IamTenantModifyReq};
use bios_iam::console_system::dto::iam_cs_tenant_dto::IamCsTenantAddReq;
use bios_iam::iam_constants::{RBUM_SCOPE_LEVEL_GLOBAL, RBUM_SCOPE_LEVEL_TENANT};
use bios_iam::iam_enumeration::IamCertKind;
use bios_iam::iam_test_helper::BIOSWebTestClient;

pub async fn test(sysadmin_name: &str, sysadmin_password: &str, client: &mut BIOSWebTestClient) -> TardisResult<()> {
    info!("【test_iam_scenes_tenant】");

    client.login(sysadmin_name, sysadmin_password, None, None, None, true).await?;

    // Add Tenant
    let tenant_id: String = client
        .post(
            "/cs/tenant",
            &IamCsTenantAddReq {
                tenant_name: TrimString("测试公司1".to_string()),
                tenant_icon: Some("https://oss.minio.io/xxx.icon".to_string()),
                tenant_contact_phone: None,
                tenant_note: None,
                admin_name: TrimString("测试管理员".to_string()),
                admin_username: TrimString("admin".to_string()),
                admin_password: Some("123456".to_string()),
                cert_conf_by_user_pwd: IamUserPwdCertConfAddOrModifyReq {
                    ak_note: None,
                    ak_rule: None,
                    // 密码长度，密码复杂度等使用前端自定义格式写入到sk_node字段
                    sk_note: None,
                    // 前端生成正则判断写入到sk_rule字段
                    sk_rule: None,
                    repeatable: Some(false),
                    expire_sec: None,
                },
                cert_conf_by_phone_vcode: None,
                cert_conf_by_mail_vcode: Some(IamMailVCodeCertConfAddOrModifyReq { ak_note: None, ak_rule: None }),
                disabled: None,
            },
        )
        .await;
    sleep(Duration::from_secs(1)).await;
    client.login("admin", "123456", Some(tenant_id.clone()), None, None, true).await?;

    tenant_console_tenant_mgr_page(client).await?;
    tenant_console_org_mgr_page("admin", "123456", &tenant_id, client).await?;
    tenant_console_account_mgr_page(client).await?;
    tenant_console_auth_mgr_page(client).await?;

    Ok(())
}

pub async fn tenant_console_tenant_mgr_page(client: &mut BIOSWebTestClient) -> TardisResult<()> {
    info!("【tenant_console_tenant_mgr_page】");

    // Get Current Tenant
    let tenant: IamTenantDetailResp = client.get("/ct/tenant").await;
    assert_eq!(tenant.name, "测试公司1");
    assert_eq!(tenant.icon, "https://oss.minio.io/xxx.icon");

    // Find Cert Conf by Current Tenant
    let cert_conf: Vec<RbumCertConfDetailResp> = client.get("/ct/cert-conf").await;
    assert_eq!(cert_conf.len(), 2);
    assert!(cert_conf.iter().any(|x| x.code == IamCertKind::MailVCode.to_string()));
    let cert_conf_user_pwd = cert_conf.iter().find(|x| x.code == IamCertKind::UserPwd.to_string()).unwrap();
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

pub async fn tenant_console_org_mgr_page(tenant_admin_user_name: &str, tenant_admin_password: &str, tenant_id: &str, client: &mut BIOSWebTestClient) -> TardisResult<()> {
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
    assert_eq!(accounts, 1);

    // Find Accounts
    let accounts: TardisPage<IamAccountSummaryResp> = client.get("/ct/account?page_number=1&page_size=10").await;
    assert_eq!(accounts.total_size, 1);
    let account_id = accounts.records.iter().find(|i| i.name == "测试管理员").unwrap().id.clone();

    // Find Role By Account Id
    let roles: Vec<RbumRelBoneResp> = client.get(&format!("/ct/account/{}/role", account_id)).await;
    assert_eq!(roles.len(), 1);
    assert_eq!(roles.get(0).unwrap().rel_name, "tenant_admin");

    // Find Set Paths By Account Id
    let org_path: Vec<Vec<RbumSetPathResp>> = client.get(&format!("/ct/account/{}/set-path", account_id)).await;
    assert_eq!(org_path.len(), 0);

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
    let org_path: Vec<Vec<RbumSetPathResp>> = client.get(&format!("/ct/account/{}/set-path", account_id)).await;
    assert!(org_path.get(0).unwrap().iter().any(|i| i.name == "综合服务中心"));

    client.login(tenant_admin_user_name, tenant_admin_password, Some(tenant_id.to_string()), None, None, true).await?;
    assert_eq!(client.context().groups.len(), 1);
    assert!(client.context().groups.get(0).unwrap().contains(":aaaa"));

    // Delete Org Item By Org Item Id
    client.delete(&format!("/ct/org/item/{}", items.get(0).unwrap().id)).await;
    let items: Vec<RbumSetItemSummaryResp> = client.get(&format!("/ct/org/item?cate_id={}", cate_node1_id)).await;
    assert_eq!(items.len(), 0);
    let org_path: Vec<Vec<RbumSetPathResp>> = client.get(&format!("/ct/account/{}/set-path", account_id)).await;
    assert_eq!(org_path.len(), 0);

    client.login(tenant_admin_user_name, tenant_admin_password, Some(tenant_id.to_string()), None, None, true).await?;
    assert_eq!(client.context().groups.len(), 0);

    Ok(())
}

pub async fn tenant_console_account_mgr_page(client: &mut BIOSWebTestClient) -> TardisResult<()> {
    info!("【tenant_console_account_mgr_page】");

    // -------------------- Account Attr --------------------

    // Add Account Attr
    let _: String = client
        .post(
            "/ct/account/attr",
            &IamKindAttrAddReq {
                name: TrimString("ext1_idx".to_string()),
                label: "工号".to_string(),
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
                options: None,
                required: Some(true),
                min_length: None,
                max_length: None,
                action: None,
                ext: None,
                scope_level: Some(RBUM_SCOPE_LEVEL_TENANT),
            },
        )
        .await;

    let attr_id: String = client
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

    // Find Account Attrs
    let attrs: Vec<RbumKindAttrSummaryResp> = client.get("/ct/account/attr").await;
    assert_eq!(attrs.len(), 2);

    // Modify Account Attrs by Attr Id
    let _: Void = client
        .put(
            &format!("/ct/account/attr/{}", attr_id),
            &RbumKindAttrModifyReq {
                label: None,
                note: None,
                sort: None,
                main_column: None,
                position: None,
                capacity: None,
                overload: None,
                hide: None,
                idx: None,
                data_type: None,
                widget_type: None,
                default_value: None,
                options: Some(r#"[{"l1":"L1","l2":"L2","l3":"L3"}]"#.to_string()),
                required: None,
                min_length: None,
                max_length: None,
                action: None,
                ext: None,
                scope_level: None,
            },
        )
        .await;

    // Get Account Attrs by Attr Id
    let attr: RbumKindAttrDetailResp = client.get(&format!("/ct/account/attr/{}", attr_id)).await;
    assert_eq!(attr.name, "ext9");
    assert_eq!(attr.label, "岗级");
    assert_eq!(attr.options, r#"[{"l1":"L1","l2":"L2","l3":"L3"}]"#);

    // Delete Account Attr By Attr Id
    client.delete(&format!("/ct/account/attr/{}", attr_id)).await;

    // -------------------- Account --------------------

    // =============== Prepare ===============
    let _: String = client
        .post(
            "/ct/role",
            &IamRoleAggAddReq {
                role: IamRoleAddReq {
                    code: TrimString("audit_admin".to_string()),
                    name: TrimString("审计管理员".to_string()),
                    // 必须设置成全局作用域（1）
                    scope_level: Some(RBUM_SCOPE_LEVEL_GLOBAL),
                    disabled: None,
                    icon: None,
                    sort: None,
                },
                res_ids: None,
            },
        )
        .await;
    // =============== Prepare ===============

    // Find Roles
    let roles: TardisPage<IamRoleSummaryResp> = client.get("/ct/role?page_number=1&page_size=10").await;
    let role_id = &roles.records.iter().find(|i| i.name == "审计管理员").unwrap().id;
    assert_eq!(roles.total_size, 2);
    assert!(roles.records.iter().any(|i| i.name == "tenant_admin"));
    assert!(roles.records.iter().any(|i| i.name == "审计管理员"));

    // Find Org Cates By Current Tenant
    let res_tree: Vec<RbumSetTreeResp> = client.get("/ct/org/tree").await;
    assert_eq!(res_tree.len(), 1);

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
                org_node_ids: Some(vec![res_tree[0].id.to_string()]),
                scope_level: Some(RBUM_SCOPE_LEVEL_TENANT),
                disabled: None,
                icon: None,
                exts: HashMap::from([("ext1_idx".to_string(), "00001".to_string())]),
            },
        )
        .await;

    // Find Accounts
    let accounts: TardisPage<IamAccountSummaryResp> = client.get("/ct/account?page_number=1&page_size=10").await;
    assert_eq!(accounts.total_size, 2);

    // Get Account By Account Id
    let account: IamAccountDetailResp = client.get(&format!("/ct/account/{}", account_id)).await;
    assert_eq!(account.name, "用户3");

    // Find Set Paths By Account Id
    let org_path: Vec<Vec<RbumSetPathResp>> = client.get(&format!("/ct/account/{}/set-path", account_id)).await;
    assert_eq!(org_path.len(), 1);
    assert!(org_path.get(0).unwrap().iter().any(|i| i.name == "综合服务中心"));

    // Find Account Attr Value By Account Id
    let account_attrs: HashMap<String, String> = client.get(&format!("/ct/account/attr/value?account_id={}", account_id)).await;
    assert_eq!(account_attrs.len(), 1);
    assert_eq!(account_attrs.get("ext1_idx"), Some(&"00001".to_string()));

    // Find Rel Role By Account Id
    let roles: Vec<RbumRelBoneResp> = client.get(&format!("/ct/account/{}/role", account_id)).await;
    assert_eq!(roles.len(), 1);
    assert_eq!(roles.get(0).unwrap().rel_name, "审计管理员");

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
                org_cate_ids: Some(vec![]),
                exts: HashMap::from([("ext1_idx".to_string(), "".to_string())]),
            },
        )
        .await;

    // Get Account By Account Id
    let account: IamAccountDetailResp = client.get(&format!("/ct/account/{}", account_id)).await;
    assert_eq!(account.name, "用户3_new");

    // Find Rel Role By Account Id
    let roles: Vec<RbumRelBoneResp> = client.get(&format!("/ct/account/{}/role", account_id)).await;
    assert_eq!(roles.len(), 0);

    // Find Set Paths By Account Id
    let org_path: Vec<Vec<RbumSetPathResp>> = client.get(&format!("/ct/account/{}/set-path", account_id)).await;
    assert_eq!(org_path.len(), 0);

    // Find Account Attr Value By Account Id
    let account_attrs: HashMap<String, String> = client.get(&format!("/ct/account/attr/value?account_id={}", account_id)).await;
    assert_eq!(account_attrs.len(), 1);
    assert_eq!(account_attrs.get("ext1_idx"), Some(&"".to_string()));

    // Find Certs By Account Id
    let certs: Vec<RbumCertSummaryResp> = client.get(&format!("/ct/cert?account_id={}", account_id)).await;
    assert_eq!(certs.len(), 2);
    assert!(certs.into_iter().any(|i| i.rel_rbum_cert_conf_code == Some("UserPwd".to_string())));

    // Rest Password By Account Id
    let _: Void = client
        .put(
            &format!("/ct/cert/user-pwd?account_id={}", account_id),
            &IamUserPwdCertRestReq {
                new_sk: TrimString("123456".to_string()),
            },
        )
        .await;

    // Delete Account By Account Id
    let _ = client.delete(&format!("/ct/account/{}", account_id)).await;

    Ok(())
}

pub async fn tenant_console_auth_mgr_page(client: &mut BIOSWebTestClient) -> TardisResult<()> {
    info!("【tenant_console_auth_mgr_page】");

    // Find Accounts
    let accounts: TardisPage<IamAccountSummaryResp> = client.get("/ct/account?page_number=1&page_size=10").await;
    assert_eq!(accounts.total_size, 2);
    let account_id = accounts.records.iter().find(|i| i.name == "测试管理员").unwrap().id.clone();

    // Find Roles
    let roles: TardisPage<IamRoleSummaryResp> = client.get("/ct/role?page_number=1&page_size=10").await;
    assert_eq!(roles.total_size, 2);
    assert!(!roles.records.iter().any(|i| i.name == "sys_admin"));

    // Find Res Tree
    let res_tree: Vec<RbumSetTreeResp> = client.get("/ct/res/tree?sys_res=true").await;
    assert_eq!(res_tree.len(), 2);
    let res = res_tree.iter().find(|i| i.name == "Menus").unwrap().rbum_set_items.get(0).unwrap();
    assert!(res.rel_rbum_item_name.contains("Console"));
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
    assert!(res.get(0).unwrap().rel_name.contains("Console"));

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
    let account_id = accounts.records.get(0).unwrap().id.clone();

    // Count Account By Role Id
    let accounts: u64 = client.get(&format!("/ct/role/{}/account/total", role_id)).await;
    assert_eq!(accounts, 1);

    // Find Certs By Account Id
    let certs: Vec<RbumCertSummaryResp> = client.get(&format!("/ct/cert?account_id={}", account_id)).await;
    assert_eq!(certs.len(), 1);
    assert!(certs.into_iter().any(|i| i.rel_rbum_cert_conf_code == Some("UserPwd".to_string())));

    // Find Set Paths By Account Id
    let org_path: Vec<Vec<RbumSetPathResp>> = client.get(&format!("/ct/account/{}/set-path", account_id)).await;
    assert_eq!(org_path.len(), 0);

    // Delete Account By Res Id
    client.delete(&format!("/ct/role/{}/account/{}", role_id, account_id)).await;
    let accounts: u64 = client.get(&format!("/ct/role/{}/account/total", role_id)).await;
    assert_eq!(accounts, 0);

    Ok(())
}
