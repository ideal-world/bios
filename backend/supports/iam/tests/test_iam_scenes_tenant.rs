use std::collections::HashMap;
use std::time::Duration;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;
use tardis::tokio::time::sleep;
use tardis::web::web_resp::{TardisPage, TardisResp, Void};

use bios_basic::rbum::dto::rbum_kind_attr_dto::{RbumKindAttrDetailResp, RbumKindAttrModifyReq, RbumKindAttrSummaryResp};
use bios_basic::rbum::dto::rbum_rel_dto::RbumRelBoneResp;
use bios_basic::rbum::dto::rbum_set_dto::RbumSetTreeResp;
use bios_basic::rbum::dto::rbum_set_item_dto::RbumSetItemDetailResp;
use bios_basic::rbum::rbum_enumeration::{RbumDataTypeKind, RbumWidgetTypeKind};
use bios_iam::basic::dto::iam_account_dto::{IamAccountAggAddReq, IamAccountAggModifyReq, IamAccountDetailAggResp, IamAccountSummaryAggResp};
use bios_iam::basic::dto::iam_app_dto::IamAppAggAddReq;
use bios_iam::basic::dto::iam_attr_dto::IamKindAttrAddReq;

use bios_iam::basic::dto::iam_cert_dto::IamCertUserPwdRestReq;
use bios_iam::basic::dto::iam_role_dto::{IamRoleAddReq, IamRoleAggAddReq, IamRoleAggModifyReq, IamRoleDetailResp, IamRoleModifyReq, IamRoleSummaryResp};
use bios_iam::basic::dto::iam_set_dto::{IamSetCateAddReq, IamSetCateModifyReq, IamSetItemWithDefaultSetAddReq};
use bios_iam::basic::dto::iam_tenant_dto::{IamTenantAggAddReq, IamTenantAggDetailResp, IamTenantAggModifyReq};
use bios_iam::iam_constants::{RBUM_SCOPE_LEVEL_GLOBAL, RBUM_SCOPE_LEVEL_TENANT};
use bios_iam::iam_test_helper::BIOSWebTestClient;

pub async fn test(sysadmin_name: &str, sysadmin_password: &str, client: &mut BIOSWebTestClient) -> TardisResult<()> {
    info!("【test_iam_scenes_tenant】");

    client.login(sysadmin_name, sysadmin_password, None, None, None, true).await?;

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
                admin_password: Some(TrimString("123456".to_string())),
                admin_phone: None,
                admin_mail: None,
                audit_username: TrimString("audit".to_string()),
                audit_name: TrimString("审计管理员".to_string()),
                audit_password: None,
                audit_phone: None,
                audit_mail: None,
                // cert_conf_by_user_pwd: IamCertConfUserPwdAddOrModifyReq {
                //     ak_rule_len_min: 2,
                //     ak_rule_len_max: 20,
                //     sk_rule_len_min: 2,
                //     sk_rule_len_max: 20,
                //     sk_rule_need_num: false,
                //     sk_rule_need_uppercase: false,
                //     sk_rule_need_lowercase: false,
                //     sk_rule_need_spec_char: false,
                //     sk_lock_cycle_sec: 60,
                //     sk_lock_err_times: 2,
                //     sk_lock_duration_sec: 60,
                //     repeatable: false,
                //     expire_sec: 6000,
                // },
                // cert_conf_by_phone_vcode: false,
                // cert_conf_by_mail_vcode: true,
                disabled: None,
                account_self_reg: None,
                cert_conf_by_oauth2: None,
                cert_conf_by_ldap: None,
            },
        )
        .await;
    sleep(Duration::from_secs(1)).await;
    client.login("admin", "123456", Some(tenant_id.clone()), None, None, true).await?;

    tenant_console_tenant_mgr_page(client).await?;
    tenant_console_org_mgr_page("admin", "123456", &tenant_id, client).await?;
    tenant_console_account_mgr_page(client).await?;
    tenant_console_auth_mgr_page(client).await?;
    tenant_console_app_set_mgr_page(client).await?;

    Ok(())
}

pub async fn tenant_console_tenant_mgr_page(client: &mut BIOSWebTestClient) -> TardisResult<()> {
    info!("【tenant_console_tenant_mgr_page】");

    // Get Current Tenant
    let tenant: IamTenantAggDetailResp = client.get("/ct/tenant").await;
    assert_eq!(tenant.name, "测试公司1");
    assert_eq!(tenant.icon, "https://oss.minio.io/xxx.icon");
    assert!(!tenant.cert_conf_by_user_pwd.repeatable);
    assert!(!tenant.cert_conf_by_phone_vcode);
    assert!(tenant.cert_conf_by_mail_vcode);

    // Modify Current Tenant
    let modify_tenant_resp: TardisResp<Option<String>> = client
        .put_resp(
            "/ct/tenant",
            &IamTenantAggModifyReq {
                name: Some(TrimString("测试公司".to_string())),
                disabled: None,
                icon: None,
                sort: None,
                contact_phone: None,
                note: None,
                // cert_conf_by_user_pwd: Some(IamCertConfUserPwdAddOrModifyReq {
                //     ak_rule_len_min: 2,
                //     ak_rule_len_max: 20,
                //     sk_rule_len_min: 2,
                //     sk_rule_len_max: 20,
                //     sk_rule_need_num: false,
                //     sk_rule_need_uppercase: false,
                //     sk_rule_need_lowercase: false,
                //     sk_rule_need_spec_char: false,
                //     sk_lock_cycle_sec: 60,
                //     sk_lock_err_times: 2,
                //     sk_lock_duration_sec: 60,
                //     repeatable: true,
                //     expire_sec: 111,
                // }),
                // cert_conf_by_phone_vcode: Some(true),
                // cert_conf_by_mail_vcode: Some(true),
                account_self_reg: None,
                // cert_conf_by_oauth2: None,
                // cert_conf_by_ldap: None,
            },
        )
        .await;
    assert_eq!(modify_tenant_resp.code, "200");

    let tenant: IamTenantAggDetailResp = client.get("/ct/tenant").await;
    assert_eq!(tenant.name, "测试公司");
    assert!(tenant.cert_conf_by_user_pwd.repeatable);
    assert_eq!(tenant.cert_conf_by_user_pwd.expire_sec, 111);
    assert!(tenant.cert_conf_by_phone_vcode);
    assert!(tenant.cert_conf_by_mail_vcode);

    Ok(())
}

pub async fn tenant_console_org_mgr_page(tenant_admin_user_name: &str, tenant_admin_password: &str, tenant_id: &str, client: &mut BIOSWebTestClient) -> TardisResult<()> {
    info!("【tenant_console_org_mgr_page】");

    // Find Org Cates By Current Tenant
    let res_tree: RbumSetTreeResp = client.get("/ct/org/tree").await;
    assert_eq!(res_tree.main.len(), 0);

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
    let res_tree: RbumSetTreeResp = client.get("/ct/org/tree").await;
    assert_eq!(res_tree.main.len(), 1);
    assert_eq!(res_tree.main.first().unwrap().name, "综合服务中心");

    // Count Accounts
    let accounts: u64 = client.get("/ct/account/total").await;
    assert_eq!(accounts, 1);

    // Find Accounts
    let accounts: TardisPage<IamAccountSummaryAggResp> = client.get("/ct/account?page_number=1&page_size=10").await;
    assert_eq!(accounts.total_size, 1);
    let account = accounts.records.into_iter().find(|i| i.name == "测试管理员").unwrap();
    assert_eq!(account.roles.len(), 1);
    assert!(account.roles.iter().any(|i| i.1 == "tenant_admin"));
    assert!(account.orgs.is_empty());
    assert_eq!(account.certs.len(), 1);
    assert!(account.certs.contains_key("UserPwd"));
    let account_id = account.id.clone();

    // Add Org Item
    let _: String = client
        .put(
            "/ct/org/item",
            &IamSetItemWithDefaultSetAddReq {
                set_cate_id: Some(cate_node1_id.to_string()),
                sort: 0,
                rel_rbum_item_id: account_id.clone(),
            },
        )
        .await;

    // Find Org Items
    let items: Vec<RbumSetItemDetailResp> = client.get(&format!("/ct/org/item?cate_id={}", cate_node1_id)).await;
    assert_eq!(items.len(), 1);
    assert_eq!(items.first().unwrap().rel_rbum_item_name, "测试管理员");
    let account: IamAccountDetailAggResp = client.get(&format!("/ct/account/{}", account_id)).await;
    assert!(account.orgs.contains(&("综合服务中心".to_string())));

    client.login(tenant_admin_user_name, tenant_admin_password, Some(tenant_id.to_string()), None, None, true).await?;
    assert_eq!(client.context().groups.len(), 1);
    assert!(client.context().groups.first().unwrap().contains(":0000"));

    // Delete Org Item By Org Item Id
    client.delete(&format!("/ct/org/item/{}", items.first().unwrap().id)).await;
    let items: Vec<RbumSetItemDetailResp> = client.get(&format!("/ct/org/item?cate_id={}", cate_node1_id)).await;
    assert_eq!(items.len(), 0);
    let account: IamAccountDetailAggResp = client.get(&format!("/ct/account/{}", account_id)).await;
    assert!(account.orgs.is_empty());

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
                    id: None,
                    code: Some(TrimString("audit_admin".to_string())),
                    name: TrimString("审计管理员".to_string()),
                    // 必须设置成全局作用域（1）
                    scope_level: Some(RBUM_SCOPE_LEVEL_GLOBAL),
                    disabled: None,
                    icon: None,
                    sort: None,
                    kind: None,
                    in_embed: None,
                    extend_role_id: None,
                    in_base: None,
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
    let res_tree: RbumSetTreeResp = client.get("/ct/org/tree").await;
    assert_eq!(res_tree.main.len(), 1);

    // Add Account
    let account_id: String = client
        .post(
            "/ct/account",
            &IamAccountAggAddReq {
                id: None,
                name: TrimString("用户3".to_string()),
                cert_user_name: TrimString("user3".to_string()),
                cert_password: Some(TrimString("123456".to_string())),
                cert_phone: None,
                cert_mail: Some(TrimString("gudaoxuri@outlook.com".to_string())),
                role_ids: Some(vec![role_id.to_string()]),
                org_node_ids: Some(vec![res_tree.main[0].id.to_string()]),
                scope_level: Some(RBUM_SCOPE_LEVEL_TENANT),
                disabled: None,
                icon: None,
                exts: HashMap::from([("ext1_idx".to_string(), "00001".to_string())]),
                status: None,
                temporary: None,
                lock_status: None,
                logout_type: None,
                labor_type: None,
            },
        )
        .await;

    // Find Accounts
    let accounts: TardisPage<IamAccountSummaryAggResp> = client.get("/ct/account?page_number=1&page_size=10").await;
    assert_eq!(accounts.total_size, 2);

    // Get Account By Account Id
    let account: IamAccountDetailAggResp = client.get(&format!("/ct/account/{}", account_id)).await;
    assert_eq!(account.name, "用户3");
    assert_eq!(account.orgs.len(), 1);
    assert!(account.orgs.contains(&("综合服务中心".to_string())));
    assert_eq!(account.exts.len(), 1);
    assert_eq!(account.exts.into_iter().find(|r| r.name == "ext1_idx").unwrap().value, "00001");
    assert_eq!(account.roles.len(), 1);
    assert!(account.roles.into_iter().any(|r| r.1 == "审计管理员"));

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
                exts: Some(HashMap::from([("ext1_idx".to_string(), "".to_string())])),
                status: None,
                cert_phone: None,
                cert_mail: None,
                temporary: None,
                logout_type: None,
                labor_type: None,
            },
        )
        .await;

    // Get Account By Account Id
    let account: IamAccountDetailAggResp = client.get(&format!("/ct/account/{}", account_id)).await;
    assert_eq!(account.name, "用户3_new");
    assert_eq!(account.roles.len(), 0);
    assert_eq!(account.orgs.len(), 0);
    assert_eq!(account.exts.len(), 1);
    assert_eq!(account.exts.into_iter().find(|r| r.name == "ext1_idx").unwrap().value, "");
    assert_eq!(account.certs.len(), 2);
    assert!(account.certs.contains_key("UserPwd"));

    // Rest Password By Account Id
    let _: Void = client
        .put(
            &format!("/ct/cert/user-pwd?account_id={}", account_id),
            &IamCertUserPwdRestReq {
                new_sk: Some(TrimString("1234567".to_string())),
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
    let accounts: TardisPage<IamAccountSummaryAggResp> = client.get("/ct/account?page_number=1&page_size=10").await;
    assert_eq!(accounts.total_size, 1);
    let account_id = accounts.records.iter().find(|i| i.name == "测试管理员").unwrap().id.clone();

    // Find Roles
    let roles: TardisPage<IamRoleSummaryResp> = client.get("/ct/role?page_number=1&page_size=10").await;
    assert_eq!(roles.total_size, 2);
    assert!(!roles.records.iter().any(|i| i.name == "sys_admin"));

    // Find Menu Tree
    let res_tree: RbumSetTreeResp = client.get("/ct/res/tree").await;
    assert_eq!(res_tree.main.len(), 3);
    let res = res_tree.ext.as_ref().unwrap().items[&res_tree.main.iter().find(|i| i.name == "Menus").unwrap().id].first().unwrap();
    assert!(res.rel_rbum_item_name.contains("Console"));
    let res_id = res.rel_rbum_item_id.clone();

    // Add Role
    let role_id: String = client
        .post(
            "/ct/role",
            &IamRoleAggAddReq {
                role: IamRoleAddReq {
                    id: None,
                    code: Some(TrimString("role5".to_string())),
                    name: TrimString("角色5".to_string()),
                    scope_level: Some(RBUM_SCOPE_LEVEL_TENANT),
                    disabled: None,
                    icon: None,
                    sort: None,
                    kind: None,
                    in_embed: None,
                    extend_role_id: None,
                    in_base: None,
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
    assert!(res.first().unwrap().rel_name.contains("Console"));

    // Modify Role by Role Id
    let modify_role_resp: TardisResp<Option<String>> = client
        .put_resp(
            &format!("/ct/role/{}", role_id),
            &IamRoleAggModifyReq {
                role: Some(IamRoleModifyReq {
                    name: Some(TrimString("xx角色".to_string())),
                    scope_level: None,
                    disabled: None,
                    icon: None,
                    sort: None,
                    kind: None,
                }),
                res_ids: Some(vec![]),
            },
        )
        .await;
    assert_eq!(modify_role_resp.code, "200");

    // Get Role By Role Id
    let role: IamRoleDetailResp = client.get(&format!("/ct/role/{}", role_id)).await;
    assert_eq!(role.name, "xx角色");

    // Find Res By Role Id
    let res: Vec<RbumRelBoneResp> = client.get(&format!("/ct/role/{}/res", role_id)).await;
    assert_eq!(res.len(), 0);

    // Add Account To Role
    let _: Void = client.put(&format!("/ct/role/{}/account/{}", role_id, account_id), &Void {}).await;

    // Find Accounts By Role Id
    let accounts: TardisPage<IamAccountSummaryAggResp> = client.get(&format!("/ct/account?role_id={}&with_sub=false&page_number=1&page_size=10", role_id)).await;
    assert_eq!(accounts.total_size, 1);
    assert_eq!(accounts.records.first().unwrap().name, "测试管理员");
    let account = accounts.records.first().unwrap();
    assert_eq!(account.certs.len(), 1);
    assert!(account.certs.contains_key("UserPwd"));
    assert!(account.orgs.is_empty());
    let account_id = account.id.clone();

    // Count Account By Role Id
    let accounts: u64 = client.get(&format!("/ct/role/{}/account/total", role_id)).await;
    assert_eq!(accounts, 1);

    // Delete Account By Res Id
    client.delete(&format!("/ct/role/{}/account/{}", role_id, account_id)).await;
    let accounts: u64 = client.get(&format!("/ct/role/{}/account/total", role_id)).await;
    assert_eq!(accounts, 0);

    Ok(())
}

pub async fn tenant_console_app_set_mgr_page(client: &mut BIOSWebTestClient) -> TardisResult<()> {
    info!("【tenant_console_app_set_mgr_page】");

    // =============== Prepare ===============
    // Add Account
    let app_account1_id: String = client
        .post(
            "/ct/account",
            &IamAccountAggAddReq {
                id: None,
                name: TrimString("admin1".to_string()),
                cert_user_name: TrimString("apps_test1".to_string()),
                cert_password: Some(TrimString("123456".to_string())),
                cert_phone: None,
                cert_mail: None,
                role_ids: None,
                org_node_ids: None,
                scope_level: Some(RBUM_SCOPE_LEVEL_TENANT),
                disabled: None,
                icon: None,
                exts: HashMap::from([("ext1_idx".to_string(), "00001".to_string())]),
                status: None,
                temporary: None,
                lock_status: None,
                logout_type: None,
                labor_type: None,
            },
        )
        .await;
    let app_account2_id: String = client
        .post(
            "/ct/account",
            &IamAccountAggAddReq {
                id: None,
                name: TrimString("admin2".to_string()),
                cert_user_name: TrimString("apps_test2".to_string()),
                cert_password: Some(TrimString("123456".to_string())),
                cert_phone: None,
                cert_mail: None,
                role_ids: None,
                org_node_ids: None,
                scope_level: Some(RBUM_SCOPE_LEVEL_TENANT),
                disabled: None,
                icon: None,
                exts: HashMap::from([("ext1_idx".to_string(), "00001".to_string())]),
                status: None,
                temporary: None,
                lock_status: None,
                logout_type: None,
                labor_type: None,
            },
        )
        .await;

    // Add App
    let app1_id: String = client
        .post(
            "/ct/app",
            &IamAppAggAddReq {
                app_name: TrimString("app1".to_string()),
                app_icon: None,
                app_sort: None,
                app_contact_phone: None,
                disabled: None,
                admin_ids: Some(vec![app_account1_id.clone()]),
                set_cate_id: None,
                kind: None,
                sync_apps_group: None,
            },
        )
        .await;
    let app2_id: String = client
        .post(
            "/ct/app",
            &IamAppAggAddReq {
                app_name: TrimString("app2".to_string()),
                app_icon: None,
                app_sort: None,
                app_contact_phone: None,
                disabled: None,
                admin_ids: Some(vec![app_account2_id.clone()]),
                set_cate_id: None,
                kind: None,
                sync_apps_group: None,
            },
        )
        .await;
    // =============== Prepare ===============

    // Find App Set Cates By Current Tenant
    let app_tree: RbumSetTreeResp = client.get("/ct/apps/tree").await;
    assert_eq!(app_tree.main.len(), 0);

    // Add App Set Cate
    let cate_node1_id: String = client
        .post(
            "/ct/apps/cate",
            &IamSetCateAddReq {
                name: TrimString("x事业部".to_string()),
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
            "/ct/apps/cate",
            &IamSetCateAddReq {
                name: TrimString("yy事业部".to_string()),
                scope_level: Some(RBUM_SCOPE_LEVEL_TENANT),
                bus_code: None,
                icon: None,
                sort: None,
                ext: None,
                rbum_parent_cate_id: None,
            },
        )
        .await;
    let cate_node2_1_id: String = client
        .post(
            "/ct/apps/cate",
            &IamSetCateAddReq {
                name: TrimString("yy事业部aa中心".to_string()),
                scope_level: Some(RBUM_SCOPE_LEVEL_TENANT),
                bus_code: None,
                icon: None,
                sort: None,
                ext: None,
                rbum_parent_cate_id: Some(cate_node2_id.clone()),
            },
        )
        .await;
    let cate_node2_1_1_id: String = client
        .post(
            "/ct/apps/cate",
            &IamSetCateAddReq {
                name: TrimString("yy事业部aa中心bb组".to_string()),
                scope_level: Some(RBUM_SCOPE_LEVEL_TENANT),
                bus_code: None,
                icon: None,
                sort: None,
                ext: None,
                rbum_parent_cate_id: Some(cate_node2_1_id.clone()),
            },
        )
        .await;
    let cate_node3_id: String = client
        .post(
            "/ct/apps/cate",
            &IamSetCateAddReq {
                name: TrimString("z事业部".to_string()),
                scope_level: Some(RBUM_SCOPE_LEVEL_TENANT),
                bus_code: None,
                icon: None,
                sort: None,
                ext: None,
                rbum_parent_cate_id: None,
            },
        )
        .await;

    // Delete App Set Cate By App Set Id
    client.delete(&format!("/ct/apps/cate/{}", cate_node3_id)).await;

    // Modify App Set Cate By App Set Id
    let _: Void = client
        .put(
            &format!("/ct/apps/cate/{}", cate_node1_id),
            &IamSetCateModifyReq {
                name: Some(TrimString("xx事业部".to_string())),
                scope_level: None,
                bus_code: None,
                icon: None,
                sort: None,
                ext: None,
            },
        )
        .await;
    let res_tree: RbumSetTreeResp = client.get("/ct/apps/tree").await;
    assert_eq!(res_tree.main.len(), 4);
    assert!(res_tree.main.iter().any(|cate| cate.name == "yy事业部aa中心" && cate.pid == Some(cate_node2_id.clone())));

    // Add App Set Item
    let _: String = client
        .put(
            "/ct/apps/item",
            &IamSetItemWithDefaultSetAddReq {
                set_cate_id: Some(cate_node2_id.to_string()),
                sort: 0,
                rel_rbum_item_id: app1_id.clone(),
            },
        )
        .await;
    let _: String = client
        .put(
            "/ct/apps/item",
            &IamSetItemWithDefaultSetAddReq {
                set_cate_id: Some(cate_node2_id.to_string()),
                sort: 0,
                rel_rbum_item_id: app_account1_id.clone(),
            },
        )
        .await;
    let item_2_id_with_current_account: String = client
        .put(
            "/ct/apps/item",
            &IamSetItemWithDefaultSetAddReq {
                set_cate_id: Some(cate_node2_id.to_string()),
                sort: 0,
                rel_rbum_item_id: client.context().owner.clone(),
            },
        )
        .await;
    let _: String = client
        .put(
            "/ct/apps/item",
            &IamSetItemWithDefaultSetAddReq {
                set_cate_id: Some(cate_node2_1_id.to_string()),
                sort: 0,
                rel_rbum_item_id: app2_id.clone(),
            },
        )
        .await;
    let _: String = client
        .put(
            "/ct/apps/item",
            &IamSetItemWithDefaultSetAddReq {
                set_cate_id: Some(cate_node2_1_id.to_string()),
                sort: 0,
                rel_rbum_item_id: app_account2_id.clone(),
            },
        )
        .await;
    let _: String = client
        .put(
            "/ct/apps/item",
            &IamSetItemWithDefaultSetAddReq {
                set_cate_id: Some(cate_node2_1_1_id.to_string()),
                sort: 0,
                rel_rbum_item_id: app2_id.clone(),
            },
        )
        .await;
    let item_2_1_1_id_with_current_account: String = client
        .put(
            "/ct/apps/item",
            &IamSetItemWithDefaultSetAddReq {
                set_cate_id: Some(cate_node2_1_1_id.to_string()),
                sort: 0,
                rel_rbum_item_id: client.context().owner.clone(),
            },
        )
        .await;

    // Find App Set Items
    let items: Vec<RbumSetItemDetailResp> = client.get(&format!("/ct/apps/item?cate_id={}", cate_node1_id)).await;
    assert_eq!(items.len(), 0);
    let items: Vec<RbumSetItemDetailResp> = client.get(&format!("/ct/apps/item?cate_id={}", cate_node2_id)).await;
    assert_eq!(items.len(), 3);
    assert!(items.iter().any(|item| item.rel_rbum_item_name == "admin1"));
    assert!(items.iter().any(|item| item.rel_rbum_item_name == "app1"));
    assert!(items.iter().any(|item| item.rel_rbum_item_id == client.context().owner));
    let items: Vec<RbumSetItemDetailResp> = client.get(&format!("/ct/apps/item?cate_id={}", cate_node2_1_id)).await;
    assert_eq!(items.len(), 2);
    assert!(items.iter().any(|item| item.rel_rbum_item_name == "admin2"));
    assert!(items.iter().any(|item| item.rel_rbum_item_name == "app2"));
    let items: Vec<RbumSetItemDetailResp> = client.get("/ct/apps/item").await;
    assert_eq!(items.len(), 7);
    assert!(items.iter().any(|item| item.rel_rbum_item_name == "admin1"));
    assert!(items.iter().any(|item| item.rel_rbum_item_name == "app1"));
    assert!(items.iter().any(|item| item.rel_rbum_item_name == "admin2"));
    assert!(items.iter().any(|item| item.rel_rbum_item_name == "app2"));
    assert!(items.iter().any(|item| item.rel_rbum_item_id == client.context().owner));

    // Find App Set Cates By Current Tenant
    let app_tree: RbumSetTreeResp = client.get("/ct/apps/tree").await;
    assert_eq!(app_tree.main.len(), 4);
    assert!(app_tree.main.iter().any(|cate| cate.name == "xx事业部" && cate.pid.is_none() && app_tree.ext.as_ref().unwrap().items[&cate.id].is_empty()));
    assert!(app_tree.main.iter().any(|cate| cate.name == "yy事业部" && cate.pid.is_none() && app_tree.ext.as_ref().unwrap().items[&cate.id].len() == 3));
    assert!(app_tree.main.iter().any(|cate| cate.name == "yy事业部aa中心" && cate.pid == Some(cate_node2_id.clone()) && app_tree.ext.as_ref().unwrap().items[&cate.id].len() == 2));
    assert!(app_tree
        .main
        .iter()
        .any(|cate| cate.name == "yy事业部aa中心bb组" && cate.pid == Some(cate_node2_1_id.clone()) && app_tree.ext.as_ref().unwrap().items[&cate.id].len() == 2));
    let app_tree: RbumSetTreeResp = client.get("/ct/apps/tree?only_related=true").await;
    assert_eq!(app_tree.main.len(), 3);
    assert!(app_tree.main.iter().any(|cate| cate.name == "yy事业部" && cate.pid.is_none() && app_tree.ext.as_ref().unwrap().items[&cate.id].len() == 3));
    assert!(app_tree.main.iter().any(|cate| cate.name == "yy事业部aa中心" && cate.pid == Some(cate_node2_id.clone()) && app_tree.ext.as_ref().unwrap().items[&cate.id].len() == 2));
    assert!(app_tree
        .main
        .iter()
        .any(|cate| cate.name == "yy事业部aa中心bb组" && cate.pid == Some(cate_node2_1_id.clone()) && app_tree.ext.as_ref().unwrap().items[&cate.id].len() == 2));

    // Check App Scope with Account
    assert!(client.get(&format!("/ct/apps/scope?app_id={}&account={}", app1_id, app_account1_id)).await);
    assert!(client.get(&format!("/ct/apps/scope?app_id={}&account={}", app1_id, app_account2_id)).await);
    assert!(client.get(&format!("/ct/apps/scope?app_id={}&account={}", app2_id, app_account1_id)).await);
    assert!(client.get(&format!("/ct/apps/scope?app_id={}", app1_id)).await);
    assert!(client.get(&format!("/ct/apps/scope?app_id={}", app2_id)).await);

    // Delete App Set Item By App Set Item Id
    client.delete(&format!("/ct/apps/item/{}", item_2_1_1_id_with_current_account)).await;
    let items: Vec<RbumSetItemDetailResp> = client.get(&format!("/ct/apps/item?cate_id={}", cate_node2_1_1_id)).await;
    assert_eq!(items.len(), 1);
    let app_tree: RbumSetTreeResp = client.get("/ct/apps/tree?only_related=true").await;
    assert_eq!(app_tree.main.len(), 3);

    client.delete(&format!("/ct/apps/item/{}", item_2_id_with_current_account)).await;
    let app_tree: RbumSetTreeResp = client.get("/ct/apps/tree?only_related=true").await;
    assert_eq!(app_tree.main.len(), 0);
    assert!(!client.get(&format!("/ct/apps/scope?app_id={}&account={}", app1_id, app_account2_id)).await);
    assert!(!client.get(&format!("/ct/apps/scope?app_id={}", app1_id)).await);
    assert!(!client.get(&format!("/ct/apps/scope?app_id={}", app2_id)).await);

    Ok(())
}
