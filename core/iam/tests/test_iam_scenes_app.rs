use std::collections::HashMap;
use std::time::Duration;

use bios_basic::rbum::dto::rbum_cert_dto::RbumCertSummaryResp;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;
use tardis::tokio::time::sleep;
use tardis::web::web_resp::{TardisPage, Void};

use bios_basic::rbum::dto::rbum_rel_dto::RbumRelBoneResp;
use bios_basic::rbum::dto::rbum_set_cate_dto::RbumSetTreeResp;
use bios_basic::rbum::dto::rbum_set_dto::RbumSetPathResp;
use bios_iam::basic::dto::iam_account_dto::{IamAccountAggAddReq, IamAccountSummaryResp};
use bios_iam::basic::dto::iam_app_dto::IamAppDetailResp;
use bios_iam::basic::dto::iam_cert_conf_dto::{IamMailVCodeCertConfAddOrModifyReq, IamUserPwdCertConfAddOrModifyReq};
use bios_iam::basic::dto::iam_role_dto::{IamRoleAddReq, IamRoleAggAddReq, IamRoleAggModifyReq, IamRoleDetailResp, IamRoleModifyReq, IamRoleSummaryResp};
use bios_iam::console_app::dto::iam_ca_app_dto::IamCaAppModifyReq;
use bios_iam::console_system::dto::iam_cs_tenant_dto::IamCsTenantAddReq;
use bios_iam::console_tenant::dto::iam_ct_app_dto::IamCtAppAddReq;
use bios_iam::iam_constants::{RBUM_SCOPE_LEVEL_APP, RBUM_SCOPE_LEVEL_TENANT};
use bios_iam::iam_test_helper::BIOSWebTestClient;

pub async fn test(sysadmin_name: &str, sysadmin_password: &str, client: &mut BIOSWebTestClient) -> TardisResult<()> {
    info!("【test_iam_scenes_app】");

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
    client.login("user_dp", "123456", Some(tenant_id.to_string()), Some(app_id.clone()), None, true).await?;
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
    assert_eq!(roles.total_size, 1);
    assert!(roles.records.iter().any(|i| i.name == "app_admin"));

    // Find Res Tree
    let res_tree: Vec<RbumSetTreeResp> = client.get("/ca/res/tree?sys_res=true").await;
    assert_eq!(res_tree.len(), 2);
    let res = res_tree.iter().find(|i| i.name == "Menus").unwrap().rbum_set_items.get(0).unwrap();
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
    let accounts: TardisPage<IamAccountSummaryResp> = client.get(&format!("/ca/account?role_id={}&page_number=1&page_size=10", role_id)).await;
    assert_eq!(accounts.total_size, 1);
    assert_eq!(accounts.records.get(0).unwrap().name, "devops应用管理员");
    let account_id = accounts.records.get(0).unwrap().id.clone();

    // Count Account By Role Id
    let accounts: u64 = client.get(&format!("/ca/role/{}/account/total", role_id)).await;
    assert_eq!(accounts, 1);

    // Find Certs By Account Id
    let certs: Vec<RbumCertSummaryResp> = client.get(&format!("/ca/cert?account_id={}", account_id)).await;
    assert_eq!(certs.len(), 2);
    assert!(certs.into_iter().any(|i| i.rel_rbum_cert_conf_code == Some("UserPwd".to_string())));

    // Find Set Paths By Account Id
    let org_path: Vec<Vec<RbumSetPathResp>> = client.get(&format!("/ca/account/{}/set-path", account_id)).await;
    assert_eq!(org_path.len(), 0);

    // Delete Account By Res Id
    client.delete(&format!("/ca/role/{}/account/{}", role_id, account_id)).await;
    let accounts: u64 = client.get(&format!("/ca/role/{}/account/total", role_id)).await;
    assert_eq!(accounts, 0);

    Ok(())
}
