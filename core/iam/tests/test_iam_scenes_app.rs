use std::collections::HashMap;
use std::time::Duration;

use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;
use tardis::tokio::time::sleep;
use tardis::web::web_resp::{TardisPage, TardisResp, Void};

use bios_basic::rbum::dto::rbum_rel_dto::RbumRelBoneResp;
use bios_basic::rbum::dto::rbum_set_dto::RbumSetTreeResp;
use bios_iam::basic::dto::iam_account_dto::{IamAccountAggAddReq, IamAccountSummaryAggResp};
use bios_iam::basic::dto::iam_app_dto::{IamAppAggAddReq, IamAppDetailResp, IamAppModifyReq};
use bios_iam::basic::dto::iam_cert_conf_dto::IamUserPwdCertConfInfo;
use bios_iam::basic::dto::iam_role_dto::{IamRoleAddReq, IamRoleAggAddReq, IamRoleAggModifyReq, IamRoleDetailResp, IamRoleModifyReq, IamRoleSummaryResp};
use bios_iam::basic::dto::iam_tenant_dto::IamTenantAggAddReq;
use bios_iam::iam_constants::{RBUM_SCOPE_LEVEL_APP, RBUM_SCOPE_LEVEL_TENANT};
use bios_iam::iam_test_helper::BIOSWebTestClient;

pub async fn test(sysadmin_name: &str, sysadmin_password: &str, client: &mut BIOSWebTestClient) -> TardisResult<()> {
    info!("【test_iam_scenes_app】");

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
                admin_password: Some("123456".to_string()),
                cert_conf_by_user_pwd: IamUserPwdCertConfInfo {
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
                cert_conf_by_mail_vcode: true,
                disabled: None,
            },
        )
        .await;
    sleep(Duration::from_secs(1)).await;
    client.login("admin", "123456", Some(tenant_id.clone()), None, None, true).await?;

    app_console_project_mgr_page(&tenant_id, client).await?;
    app_console_auth_mgr_page(client).await?;

    Ok(())
}

pub async fn app_console_project_mgr_page(tenant_id: &str, client: &mut BIOSWebTestClient) -> TardisResult<()> {
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
                org_node_ids: None,
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
            &IamAppAggAddReq {
                app_name: TrimString("devops project".to_string()),
                app_icon: None,
                app_sort: None,
                app_contact_phone: None,
                admin_ids: Some(vec![app_account_id.clone()]),
                disabled: None,
            },
        )
        .await;

    sleep(Duration::from_secs(1)).await;
    client.login("user_dp", "123456", Some(tenant_id.to_string()), Some(app_id.clone()), None, true).await?;
    assert_eq!(client.context().roles.len(), 1);
    assert_eq!(client.context().own_paths, format!("{}/{}", tenant_id, app_id));

    // Modify App
    let modify_app_resp: TardisResp<Option<String>> = client
        .put_resp(
            "/ca/app",
            &IamAppModifyReq {
                name: Some(TrimString("DevOps项目".to_string())),
                icon: None,
                sort: None,
                contact_phone: None,
                disabled: None,
                scope_level: None,
            },
        )
        .await;
    assert_eq!(modify_app_resp.code, "200");

    // Get App
    let app: IamAppDetailResp = client.get("/ca/app").await;
    assert_eq!(app.name, "DevOps项目");

    Ok(())
}

pub async fn app_console_auth_mgr_page(client: &mut BIOSWebTestClient) -> TardisResult<()> {
    info!("【app_console_auth_mgr_page】");

    // Find Roles
    let roles: TardisPage<IamRoleSummaryResp> = client.get("/ca/role?page_number=1&page_size=10").await;
    assert_eq!(roles.total_size, 1);
    assert!(roles.records.iter().any(|i| i.name == "app_admin"));

    // Find Accounts
    let accounts: TardisPage<IamAccountSummaryAggResp> = client.get("/ca/account?page_number=1&page_size=10").await;
    assert_eq!(accounts.total_size, 1);
    let accounts: TardisPage<IamAccountSummaryAggResp> = client.get("/ca/account?name=devops&page_number=1&page_size=10").await;
    assert_eq!(accounts.total_size, 1);
    assert_eq!(accounts.records[0].name, "devops应用管理员");
    assert!(accounts.records[0].certs.contains_key("UserPwd"));
    let account_id = accounts.records.iter().find(|i| i.name == "devops应用管理员").unwrap().id.clone();

    // Find Res Tree
    let res_tree: RbumSetTreeResp = client.get("/ca/res/tree").await;
    assert_eq!(res_tree.main.len(), 1);
    let res = res_tree.ext.as_ref().unwrap().items[&res_tree.main.iter().find(|i| i.name == "Menus").unwrap().id].get(0).unwrap();
    assert!(res.rel_rbum_item_name.contains("Console"));
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
    assert!(res.get(0).unwrap().rel_name.contains("Console"));

    // Modify Role by Role Id
    let modify_role_resp: TardisResp<Option<String>> = client
        .put_resp(
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
    assert_eq!(modify_role_resp.code, "200");

    // Get Role By Role Id
    let role: IamRoleDetailResp = client.get(&format!("/ca/role/{}", role_id)).await;
    assert_eq!(role.name, "自定义角色new");

    // Find Res By Role Id
    let res: Vec<RbumRelBoneResp> = client.get(&format!("/ca/role/{}/res", role_id)).await;
    assert_eq!(res.len(), 0);

    // Add Account To Role
    let _: Void = client.put(&format!("/ca/role/{}/account/{}", role_id, account_id), &Void {}).await;

    // Find Accounts By Role Id
    let accounts: TardisPage<IamAccountSummaryAggResp> = client.get(&format!("/ca/account?role_id={}&page_number=1&page_size=10", role_id)).await;
    assert_eq!(accounts.total_size, 1);
    assert_eq!(accounts.records[0].name, "devops应用管理员");
    assert_eq!(accounts.records[0].certs.len(), 2);
    assert_eq!(accounts.records[0].orgs.len(), 0);
    assert!(accounts.records[0].certs.contains_key("UserPwd"));
    let account_id = accounts.records.get(0).unwrap().id.clone();

    // Count Account By Role Id
    let accounts: u64 = client.get(&format!("/ca/role/{}/account/total", role_id)).await;
    assert_eq!(accounts, 1);

    // Delete Account By Res Id
    client.delete(&format!("/ca/role/{}/account/{}", role_id, account_id)).await;
    let accounts: u64 = client.get(&format!("/ca/role/{}/account/total", role_id)).await;
    assert_eq!(accounts, 0);

    Ok(())
}
