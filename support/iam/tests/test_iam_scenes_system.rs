use std::collections::HashMap;
use std::time::Duration;

use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;
use tardis::tokio::time::sleep;
use tardis::web::web_resp::{TardisPage, TardisResp, Void};

use bios_basic::rbum::dto::rbum_cert_dto::RbumCertSummaryResp;
use bios_basic::rbum::dto::rbum_kind_attr_dto::{RbumKindAttrDetailResp, RbumKindAttrModifyReq, RbumKindAttrSummaryResp};
use bios_basic::rbum::dto::rbum_rel_dto::RbumRelBoneResp;
use bios_basic::rbum::dto::rbum_set_dto::RbumSetTreeResp;
use bios_basic::rbum::rbum_enumeration::{RbumDataTypeKind, RbumWidgetTypeKind};
use bios_iam::basic::dto::iam_account_dto::{IamAccountAggAddReq, IamAccountAggModifyReq, IamAccountDetailAggResp, IamAccountSummaryAggResp};
use bios_iam::basic::dto::iam_attr_dto::IamKindAttrAddReq;
use bios_iam::basic::dto::iam_cert_conf_dto::IamCertConfUserPwdAddOrModifyReq;
use bios_iam::basic::dto::iam_cert_dto::IamCertUserPwdRestReq;
use bios_iam::basic::dto::iam_res_dto::{IamResAddReq, IamResAggAddReq, IamResDetailResp, IamResModifyReq};
use bios_iam::basic::dto::iam_role_dto::{IamRoleAddReq, IamRoleAggAddReq, IamRoleAggModifyReq, IamRoleDetailResp, IamRoleModifyReq, IamRoleSummaryResp};
use bios_iam::basic::dto::iam_set_dto::{IamSetCateAddReq, IamSetCateModifyReq, IamSetItemAggAddReq, IamSetItemWithDefaultSetAddReq};
use bios_iam::basic::dto::iam_tenant_dto::{IamTenantAggAddReq, IamTenantAggDetailResp, IamTenantAggModifyReq, IamTenantSummaryResp};
use bios_iam::iam_constants::{RBUM_SCOPE_LEVEL_GLOBAL, RBUM_SCOPE_LEVEL_TENANT};
use bios_iam::iam_enumeration::IamResKind;
use bios_iam::iam_test_helper::BIOSWebTestClient;

pub async fn test(sysadmin_name: &str, sysadmin_password: &str, client: &mut BIOSWebTestClient) -> TardisResult<()> {
    info!("【test_iam_scenes_system】");

    client.login(sysadmin_name, sysadmin_password, None, None, None, true).await?;

    sys_console_tenant_mgr_page(sysadmin_name, sysadmin_password, client).await?;
    sys_console_account_mgr_page(client).await?;
    let res_menu_id = sys_console_res_mgr_page(client).await?;
    sys_console_auth_mgr_page(&res_menu_id, client).await?;

    Ok(())
}

pub async fn sys_console_tenant_mgr_page(sysadmin_name: &str, sysadmin_password: &str, client: &mut BIOSWebTestClient) -> TardisResult<()> {
    info!("【sys_console_tenant_mgr_page】");
    // Add Tenant
    let tenant_id: String = client
        .post(
            "/cs/tenant",
            &IamTenantAggAddReq {
                name: TrimString("测试公司1".to_string()),
                icon: Some("https://oss.minio.io/xxx.icon".to_string()),
                contact_phone: None,
                note: None,
                admin_name: TrimString("测试管理员".to_string()),
                admin_username: TrimString("admin".to_string()),
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
                cert_conf_by_phone_vcode: true,
                cert_conf_by_mail_vcode: false,
                disabled: None,
                account_self_reg: None,
                cert_conf_by_oauth2: None,
                cert_conf_by_ldap: None,
            },
        )
        .await;
    sleep(Duration::from_secs(1)).await;

    // =============== Prepare ===============
    client.login("admin", "123456", Some(tenant_id.clone()), None, None, true).await?;
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
                set_cate_id: Some(cate_node_id.to_string()),
                sort: 0,
                rel_rbum_item_id: account_id.clone(),
            },
        )
        .await;
    client.login(sysadmin_name, sysadmin_password, None, None, None, true).await?;
    // =============== Prepare ===============

    // Find Tenants
    let tenants: TardisPage<IamTenantSummaryResp> = client.get("/cs/tenant?page_number=1&page_size=10").await;
    assert_eq!(tenants.total_size, 1);

    // Count Accounts by Tenant Id
    let tenants: u64 = client.get(&format!("/cs/account/total?tenant_id={}", tenant_id)).await;
    assert_eq!(tenants, 2);

    // Get Tenant by Tenant Id
    let tenant: IamTenantAggDetailResp = client.get(&format!("/cs/tenant/{}", tenant_id)).await;
    assert_eq!(tenant.name, "测试公司1");
    assert_eq!(tenant.icon, "https://oss.minio.io/xxx.icon");
    assert!(!tenant.cert_conf_by_user_pwd.repeatable);
    assert!(tenant.cert_conf_by_phone_vcode);
    assert!(!tenant.cert_conf_by_mail_vcode);

    // Modify Tenant by Tenant Id
    let modify_tenant_resp: TardisResp<Option<String>> = client
        .put_resp(
            &format!("/cs/tenant/{}?tenant_id={}", tenant_id, tenant_id),
            &IamTenantAggModifyReq {
                name: Some(TrimString("测试公司_new".to_string())),
                disabled: None,
                icon: None,
                sort: None,
                contact_phone: None,
                note: None,
                cert_conf_by_user_pwd: Some(IamCertConfUserPwdAddOrModifyReq {
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
                    repeatable: true,
                    expire_sec: 111,
                }),
                cert_conf_by_phone_vcode: Some(false),
                cert_conf_by_mail_vcode: Some(true),
                account_self_reg: None,
                cert_conf_by_oauth2: None,
                cert_conf_by_ldap: None,
            },
        )
        .await;

    assert!(modify_tenant_resp.code == "200" || modify_tenant_resp.code == "202");
    if modify_tenant_resp.code == "202" {
        let task_id = modify_tenant_resp.data.unwrap().unwrap();
        print!("modify tenant task id: {}", task_id);

        let task_status: bool = client.get(&format!("/cc/system/task/{}", task_id)).await;
    }
    // Get Tenant by Tenant Id
    let tenant: IamTenantAggDetailResp = client.get(&format!("/cs/tenant/{}?tenant_id={}", tenant_id, tenant_id)).await;
    assert_eq!(tenant.name, "测试公司_new");
    assert_eq!(tenant.icon, "https://oss.minio.io/xxx.icon");
    assert!(tenant.cert_conf_by_user_pwd.repeatable);
    assert_eq!(tenant.cert_conf_by_user_pwd.expire_sec, 111);
    assert!(!tenant.cert_conf_by_phone_vcode);
    assert!(tenant.cert_conf_by_mail_vcode);

    // Find Roles By Tenant Id
    let roles: TardisPage<IamRoleSummaryResp> = client.get(&format!("/cs/role?tenant_id={}&with_sub=true&page_number=1&page_size=15", tenant_id)).await;
    assert_eq!(roles.total_size, 13);
    let sys_admin_role_id = &roles.records.iter().find(|i| i.name == "tenant_admin").unwrap().id;

    // Count Accounts By Role Id
    let accounts: u64 = client.get(&format!("/cs/role/{}/account/total?tenant_id={}", sys_admin_role_id, tenant_id)).await;
    assert_eq!(accounts, 1);

    // Find Accounts By Tenant Id
    let accounts: TardisPage<IamAccountSummaryAggResp> = client.get(&format!("/cs/account?tenant_id={}&with_sub=true&page_number=1&page_size=10", tenant_id)).await;
    assert_eq!(accounts.total_size, 2);
    assert!(accounts.records.iter().find(|r| r.id == account_id).unwrap().orgs.contains(&("综合服务中心".to_string())));

    // Find Accounts By Role Id
    let accounts: TardisPage<IamAccountSummaryAggResp> = client
        .get(&format!(
            "/cs/account?role_id={}&tenant_id={}&with_sub=true&page_number=1&page_size=10",
            sys_admin_role_id, tenant_id
        ))
        .await;
    let sys_admin_account_id = &accounts.records.get(0).unwrap().id;
    assert_eq!(accounts.total_size, 1);
    assert_eq!(accounts.records.get(0).unwrap().name, "测试管理员");
    assert_eq!(accounts.records.get(0).unwrap().roles.len(), 1);
    assert!(accounts.records.get(0).unwrap().roles.iter().any(|i| i.1 == "tenant_admin"));
    assert_eq!(accounts.records.get(0).unwrap().orgs.len(), 0);
    assert_eq!(accounts.records.get(0).unwrap().certs.len(), 1);
    assert!(accounts.records.get(0).unwrap().certs.contains_key("UserPwd"));

    // Lock/Unlock Account By Account Id
    let _: Void = client
        .put(
            &format!("/cs/account/{}?tenant_id={}", sys_admin_account_id, tenant_id),
            &IamAccountAggModifyReq {
                name: None,
                scope_level: None,
                disabled: Some(true),
                icon: None,
                role_ids: None,
                org_cate_ids: None,
                exts: None,
            },
        )
        .await;

    // Rest Password By Account Id
    let _: Void = client
        .put(
            &format!("/cs/cert/user-pwd?account_id={}&tenant_id={}", sys_admin_account_id, tenant_id),
            &IamCertUserPwdRestReq {
                new_sk: TrimString("1234567".to_string()),
            },
        )
        .await;

    // Delete By Account Id
    client.delete(&format!("/cs/account/{}?tenant_id={}", account_id, tenant_id)).await;
    let accounts: TardisPage<IamAccountSummaryAggResp> = client.get(&format!("/cs/account?tenant_id={}&with_sub=true&page_number=1&page_size=10", tenant_id)).await;
    assert_eq!(accounts.total_size, 1);

    // Offline By Account Id
    client.delete(&format!("/cs/account/{}/token?tenant_id={}", sys_admin_account_id, tenant_id)).await;

    Ok(())
}

pub async fn sys_console_account_mgr_page(client: &mut BIOSWebTestClient) -> TardisResult<()> {
    info!("【sys_console_account_mgr_page】");
    // -------------------- Account Attr --------------------

    // Add Account Attr
    let _: String = client
        .post(
            "/cs/account/attr",
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
            "/cs/account/attr",
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
    let attrs: Vec<RbumKindAttrSummaryResp> = client.get("/cs/account/attr").await;
    assert_eq!(attrs.len(), 2);

    // Modify Account Attrs by Attr Id
    let _: Void = client
        .put(
            &format!("/cs/account/attr/{}", attr_id),
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
                secret: None,
                show_by_conds: None,
                widget_columns: None,
                dyn_default_value: None,
                dyn_options: None,
                parent_attr_name: None,
            },
        )
        .await;

    // Get Account Attrs by Attr Id
    let attr: RbumKindAttrDetailResp = client.get(&format!("/cs/account/attr/{}", attr_id)).await;
    assert_eq!(attr.name, "ext9");
    assert_eq!(attr.label, "岗级");
    assert_eq!(attr.options, r#"[{"l1":"L1","l2":"L2","l3":"L3"}]"#);

    // Delete Account Attr By Attr Id
    client.delete(&format!("/cs/account/attr/{}", attr_id)).await;

    // -------------------- Account --------------------

    // =============== Prepare ===============
    let _: String = client
        .post(
            "/cs/role",
            &IamRoleAggAddReq {
                role: IamRoleAddReq {
                    code: TrimString("audit_admin".to_string()),
                    name: TrimString("审计管理员".to_string()),
                    // 必须设置成全局作用域（1）
                    scope_level: Some(RBUM_SCOPE_LEVEL_GLOBAL),
                    disabled: None,
                    icon: None,
                    sort: None,
                    kind: None,
                },
                res_ids: None,
            },
        )
        .await;
    // =============== Prepare ===============

    // Find Roles
    let roles: TardisPage<IamRoleSummaryResp> = client.get("/cs/role?page_number=1&page_size=15").await;
    let role_id = &roles.records.iter().find(|i| i.name == "审计管理员").unwrap().id;
    assert_eq!(roles.total_size, 15);
    assert!(roles.records.iter().any(|i| i.name == "sys_admin"));
    assert!(roles.records.iter().any(|i| i.name == "审计管理员"));

    // Add Account
    let account_id: String = client
        .post(
            "/cs/account",
            &IamAccountAggAddReq {
                id: None,
                name: TrimString("系统用户1".to_string()),
                cert_user_name: TrimString("user1".to_string()),
                cert_password: TrimString("123456".to_string()),
                cert_phone: None,
                cert_mail: Some(TrimString("i@sunisle.org".to_string())),
                role_ids: Some(vec![role_id.to_string()]),
                org_node_ids: None,
                scope_level: None,
                disabled: None,
                icon: None,
                exts: HashMap::from([("ext1_idx".to_string(), "00001".to_string())]),
                status: None,
            },
        )
        .await;

    // Find Accounts
    let accounts: TardisPage<IamAccountSummaryAggResp> = client.get("/cs/account?page_number=1&page_size=10").await;
    assert_eq!(accounts.total_size, 2);

    // Get Account By Account Id
    let account: IamAccountDetailAggResp = client.get(&format!("/cs/account/{}", account_id)).await;
    assert_eq!(account.name, "系统用户1");
    assert_eq!(account.roles.len(), 1);
    assert!(account.roles.into_iter().any(|r| r.1 == "审计管理员"));
    assert_eq!(account.exts.len(), 1);
    assert_eq!(account.exts.into_iter().find(|r| r.name == "ext1_idx").unwrap().value, "00001".to_string());

    // Modify Account By Account Id
    let _: Void = client
        .put(
            &format!("/cs/account/{}", account_id),
            &IamAccountAggModifyReq {
                name: Some(TrimString("用户2".to_string())),
                scope_level: None,
                disabled: None,
                icon: None,
                role_ids: Some(vec![]),
                org_cate_ids: None,
                exts: Some(HashMap::from([("ext1_idx".to_string(), "".to_string())])),
            },
        )
        .await;

    // Get Account By Account Id
    let account: IamAccountDetailAggResp = client.get(&format!("/cs/account/{}", account_id)).await;
    assert_eq!(account.name, "用户2");
    assert_eq!(account.roles.len(), 0);
    assert_eq!(account.exts.len(), 1);
    assert_eq!(account.exts.into_iter().find(|r| r.name == "ext1_idx").unwrap().value, "".to_string());
    assert_eq!(account.certs.len(), 2);
    assert!(account.certs.contains_key(&("UserPwd".to_string())));

    // Find Certs By Account Id
    let certs: Vec<RbumCertSummaryResp> = client.get(&format!("/cs/cert?account_id={}", account_id)).await;
    assert_eq!(certs.len(), 2);
    assert!(certs.into_iter().any(|i| i.rel_rbum_cert_conf_code == Some("UserPwd".to_string())));

    // Rest Password By Account Id
    let _: Void = client
        .put(
            &format!("/cs/cert/user-pwd?account_id={}", account_id),
            &IamCertUserPwdRestReq {
                new_sk: TrimString("1234567".to_string()),
            },
        )
        .await;

    // Delete Account By Account Id
    let _ = client.delete(&format!("/cs/account/{}", account_id)).await;

    Ok(())
}

pub async fn sys_console_res_mgr_page(client: &mut BIOSWebTestClient) -> TardisResult<String> {
    info!("【sys_console_res_mgr_page】");

    // Find Menu Tree
    let res_tree: RbumSetTreeResp = client.get("/cs/res/tree/menu").await;
    assert_eq!(res_tree.main.len(), 3);
    let cate_menus_id = res_tree.main.iter().find(|i| i.bus_code == "__menus__").map(|i| i.id.clone()).unwrap();

    // Find Api Tree
    let res_tree: RbumSetTreeResp = client.get("/cs/res/tree/api").await;
    assert_eq!(res_tree.main.len(), 1);
    let cate_apis_id = res_tree.main.iter().find(|i| i.bus_code == "__apis__").map(|i| i.id.clone()).unwrap();

    // Add Res Cate
    let cate_work_spaces_id: String = client
        .post(
            "/cs/res/cate",
            &IamSetCateAddReq {
                name: TrimString("工作台".to_string()),
                scope_level: Some(RBUM_SCOPE_LEVEL_GLOBAL),
                bus_code: None,
                icon: None,
                sort: None,
                ext: None,
                rbum_parent_cate_id: Some(cate_menus_id.clone()),
            },
        )
        .await;
    let cate_collaboration_id: String = client
        .post(
            "/cs/res/cate",
            &IamSetCateAddReq {
                name: TrimString("协作空间".to_string()),
                scope_level: Some(RBUM_SCOPE_LEVEL_GLOBAL),
                bus_code: None,
                icon: None,
                sort: None,
                ext: None,
                rbum_parent_cate_id: Some(cate_menus_id.clone()),
            },
        )
        .await;

    // Delete Res Cate By Res Cate Id
    client.delete(&format!("/cs/res/cate/{}", cate_collaboration_id)).await;

    // Modify Res Cate By Res Cate Id
    let _: Void = client
        .put(
            &format!("/cs/res/cate/{}", cate_work_spaces_id),
            &IamSetCateModifyReq {
                name: Some(TrimString("个人工作台".to_string())),
                scope_level: None,
                bus_code: None,
                icon: None,
                sort: None,
                ext: None,
            },
        )
        .await;

    // Add Menu Res
    let res_menu_id: String = client
        .post(
            "/cs/res",
            &IamResAggAddReq {
                res: IamResAddReq {
                    code: TrimString("work_spaces".to_string()),
                    name: TrimString("工作台页面".to_string()),
                    kind: IamResKind::Menu,
                    icon: None,
                    sort: None,
                    method: None,
                    hide: None,
                    action: None,
                    scope_level: Some(RBUM_SCOPE_LEVEL_GLOBAL),
                    disabled: None,
                },
                set: IamSetItemAggAddReq {
                    set_cate_id: cate_work_spaces_id.to_string(),
                },
            },
        )
        .await;

    // Add Element Res
    let _: String = client
        .post(
            "/cs/res",
            &IamResAggAddReq {
                res: IamResAddReq {
                    code: TrimString("work_spaces#btn1".to_string()),
                    name: TrimString("xx按钮".to_string()),
                    kind: IamResKind::Ele,
                    icon: None,
                    sort: None,
                    method: None,
                    hide: None,
                    action: None,
                    scope_level: Some(RBUM_SCOPE_LEVEL_GLOBAL),
                    disabled: None,
                },
                set: IamSetItemAggAddReq {
                    set_cate_id: cate_work_spaces_id.to_string(),
                },
            },
        )
        .await;
    let res_ele2_id: String = client
        .post(
            "/cs/res",
            &IamResAggAddReq {
                res: IamResAddReq {
                    code: TrimString("work_spaces#btn2".to_string()),
                    name: TrimString("yy按钮".to_string()),
                    kind: IamResKind::Ele,
                    icon: None,
                    sort: None,
                    method: None,
                    hide: None,
                    action: None,
                    scope_level: Some(RBUM_SCOPE_LEVEL_GLOBAL),
                    disabled: None,
                },
                set: IamSetItemAggAddReq {
                    set_cate_id: cate_work_spaces_id.to_string(),
                },
            },
        )
        .await;

    // Delete Res By Res Id
    client.delete(&format!("/cs/res/{}", res_ele2_id)).await;

    // Add Api Res
    let res_api_id: String = client
        .post(
            "/cs/res",
            &IamResAggAddReq {
                res: IamResAddReq {
                    code: TrimString("cs-test/**".to_string()),
                    name: TrimString("系统控制台功能".to_string()),
                    kind: IamResKind::Api,
                    icon: None,
                    sort: None,
                    method: None,
                    hide: None,
                    action: None,
                    scope_level: Some(RBUM_SCOPE_LEVEL_GLOBAL),
                    disabled: None,
                },
                set: IamSetItemAggAddReq {
                    set_cate_id: cate_apis_id.to_string(),
                },
            },
        )
        .await;

    // Modify Res By Res Id
    let _: Void = client
        .put(
            &format!("/cs/res/{}", res_api_id),
            &IamResModifyReq {
                name: None,
                icon: Some("/static/img/icon/api.png".to_string()),
                sort: None,
                hide: None,
                action: None,
                scope_level: None,
                disabled: None,
            },
        )
        .await;

    // Get Res by Res Id
    let res: IamResDetailResp = client.get(&format!("/cs/res/{}", res_api_id)).await;
    assert_eq!(res.code, "cs-test/**");
    assert_eq!(res.icon, "/static/img/icon/api.png");

    // Add Rel Res
    let _: Void = client.put(&format!("/cs/res/{}/res/{}", res_menu_id, res_api_id), &Void {}).await;

    // Find Api Res By Res Id
    let rel_api_res: Vec<RbumRelBoneResp> = client.get(&format!("/cs/res/{}/res", res_menu_id)).await;
    assert_eq!(rel_api_res.len(), 1);
    assert_eq!(rel_api_res.get(0).unwrap().rel_name, "系统控制台功能");

    // Count Api Res By Res Id
    let rel_api_res: u64 = client.get(&format!("/cs/res/{}/res/total", res_menu_id)).await;
    assert_eq!(rel_api_res, 1);

    // Delete Api Res By Res Id
    client.delete(&format!("/cs/res/{}/res/{}", res_menu_id, res_api_id)).await;
    let rel_api_res: u64 = client.get(&format!("/cs/res/{}/res/total", res_menu_id)).await;
    assert_eq!(rel_api_res, 0);

    Ok(res_menu_id)
}

pub async fn sys_console_auth_mgr_page(res_menu_id: &str, client: &mut BIOSWebTestClient) -> TardisResult<()> {
    info!("【sys_console_auth_mgr_page】");

    // Add Role
    let role_id: String = client
        .post(
            "/cs/role",
            &IamRoleAggAddReq {
                role: IamRoleAddReq {
                    code: TrimString("test_role".to_string()),
                    name: TrimString("测试角色".to_string()),
                    scope_level: Some(RBUM_SCOPE_LEVEL_GLOBAL),
                    disabled: None,
                    icon: None,
                    sort: None,
                    kind: None,
                },
                res_ids: None,
            },
        )
        .await;

    let modify_role_resp: TardisResp<Option<String>> = client
        .put_resp(
            &format!("/cs/role/{}", role_id),
            &IamRoleAggModifyReq {
                role: IamRoleModifyReq {
                    name: Some(TrimString("测试角色2".to_string())),
                    scope_level: None,
                    disabled: None,
                    icon: None,
                    sort: None,
                    kind: None,
                },
                res_ids: Some(vec![res_menu_id.to_string()]),
            },
        )
        .await;
    assert_eq!(modify_role_resp.code, "200");

    // Find Roles
    let roles: TardisPage<IamRoleSummaryResp> = client.get("/cs/role?with_sub=true&page_number=1&page_size=16").await;
    let sys_admin_role_id = &roles.records.iter().find(|i| i.name == "sys_admin").unwrap().id;
    let test_role_id = &roles.records.iter().find(|i| i.name == "测试角色2").unwrap().id;
    assert_eq!(roles.total_size, 16);
    assert!(roles.records.iter().any(|i| i.name == "审计管理员"));
    assert!(roles.records.iter().any(|i| i.name == "测试角色2"));

    // Get Role By Role Id
    let role: IamRoleDetailResp = client.get(&format!("/cs/role/{}", sys_admin_role_id)).await;
    assert_eq!(role.name, "sys_admin");

    // Count Res By Role Id
    let res: u64 = client.get(&format!("/cs/role/{}/res/total", sys_admin_role_id)).await;
    assert_eq!(res, 2);
    let res: u64 = client.get(&format!("/cs/role/{}/res/total", test_role_id)).await;
    assert_eq!(res, 1);

    // Find Res Tree
    let res_tree: RbumSetTreeResp = client.get("/cs/res/tree").await;
    assert_eq!(res_tree.main.len(), 5);

    // Add Res To Role
    let _: Void = client.put(&format!("/cs/role/{}/res/{}", sys_admin_role_id, res_menu_id), &Void {}).await;

    // Count Res By Role Id
    let res: u64 = client.get(&format!("/cs/role/{}/res/total", sys_admin_role_id)).await;
    assert_eq!(res, 3);

    // Find Res By Role Id
    let res: Vec<RbumRelBoneResp> = client.get(&format!("/cs/role/{}/res", sys_admin_role_id)).await;
    assert_eq!(res.len(), 3);
    assert!(res.iter().any(|r| r.rel_name == "工作台页面"));

    // Delete Res By Res Id
    client.delete(&format!("/cs/role/{}/res/{}", sys_admin_role_id, res_menu_id)).await;
    let res: u64 = client.get(&format!("/cs/role/{}/res/total", sys_admin_role_id)).await;
    assert_eq!(res, 2);

    // Find Accounts
    let accounts: TardisPage<IamAccountSummaryAggResp> = client.get("/cs/account?with_sub=true&page_number=1&page_size=10").await;
    let account_id = accounts.records.iter().find(|r| r.name == "bios").unwrap().id.clone();
    assert_eq!(accounts.total_size, 2);

    // Add Rel Account
    let _: Void = client.put(&format!("/cs/role/{}/account/{}", test_role_id, account_id), &Void {}).await;

    // Find Rel Account By Role Id
    let rel_accounts: TardisPage<IamAccountSummaryAggResp> = client.get(&format!("/cs/account?role_id={}&page_number=1&page_size=10", test_role_id)).await;
    assert_eq!(rel_accounts.total_size, 1);
    assert_eq!(rel_accounts.records.get(0).unwrap().id, account_id);

    // Count Rel Res By Role Id
    let rel_accounts: u64 = client.get(&format!("/cs/role/{}/account/total", test_role_id)).await;
    assert_eq!(rel_accounts, 1);

    // Delete Rel Res
    client.delete(&format!("/cs/role/{}/account/{}", test_role_id, account_id)).await;
    let rel_api_res: u64 = client.get(&format!("/cs/role/{}/account/total", test_role_id)).await;
    assert_eq!(rel_api_res, 0);

    Ok(())
}
