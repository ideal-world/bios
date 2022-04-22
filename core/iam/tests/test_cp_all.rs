use std::time::Duration;
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;
use tardis::tokio::time::sleep;

use bios_iam::basic::dto::iam_cert_dto::IamUserPwdCertModifyReq;
use bios_iam::console_passport::dto::iam_cp_account_dto::IamCpAccountModifyReq;
use bios_iam::console_passport::dto::iam_cp_cert_dto::IamCpUserPwdLoginReq;
use bios_iam::console_passport::serv::iam_cp_account_serv::IamCpAccountServ;
use bios_iam::console_passport::serv::iam_cp_cert_user_pwd_serv::IamCpCertUserPwdServ;
use bios_iam::console_system::dto::iam_cs_tenant_dto::IamCsTenantAddReq;
use bios_iam::console_system::serv::iam_cs_tenant_serv::IamCsTenantServ;
use bios_iam::iam_constants;

pub async fn test(sysadmin_info: (&str, &str), context: &TardisContext) -> TardisResult<()> {
    let mut funs = iam_constants::get_tardis_inst();
    funs.begin().await?;

    info!("【test_cp_all】 : Prepare : IamCsTenantServ::add_tenant");
    let (tenant_id, tenant_admin_pwd) = IamCsTenantServ::add_tenant(
        &mut IamCsTenantAddReq {
            tenant_name: TrimString("测试租户1".to_string()),
            tenant_icon: None,
            tenant_contact_phone: None,
            admin_username: TrimString("bios".to_string()),
            disabled: None,
            admin_name: TrimString("测试管理员".to_string()),
        },
        &funs,
        context,
    )
    .await?;
    sleep(Duration::from_secs(1)).await;

    info!("【test_cp_all】 : Login by Username and Password, Password error");
    assert!(IamCpCertUserPwdServ::login_by_user_pwd(
        &mut IamCpUserPwdLoginReq {
            ak: TrimString("bios".to_string()),
            sk: TrimString("123456".to_string()),
            tenant_id: None,
            flag: None
        },
        &funs,
    )
    .await
    .is_err());

    info!("【test_cp_all】 : Login by Username and Password, Tenant error");
    assert!(IamCpCertUserPwdServ::login_by_user_pwd(
        &mut IamCpUserPwdLoginReq {
            ak: TrimString("bios".to_string()),
            sk: TrimString(sysadmin_info.1.to_string()),
            tenant_id: Some(tenant_id.clone()),
            flag: None
        },
        &funs,
    )
    .await
    .is_err());

    info!("【test_cp_all】 : Login by Username and Password, Tenant error");
    assert!(IamCpCertUserPwdServ::login_by_user_pwd(
        &mut IamCpUserPwdLoginReq {
            ak: TrimString("bios".to_string()),
            sk: TrimString(tenant_admin_pwd.to_string()),
            tenant_id: None,
            flag: None
        },
        &funs,
    )
    .await
    .is_err());

    info!("【test_cp_all】 : Login by Username and Password, By tenant admin");
    let account_resp = IamCpCertUserPwdServ::login_by_user_pwd(
        &mut IamCpUserPwdLoginReq {
            ak: TrimString("bios".to_string()),
            sk: TrimString(tenant_admin_pwd.to_string()),
            tenant_id: Some(tenant_id),
            flag: None,
        },
        &funs,
    )
    .await?;

    assert_eq!(account_resp.account_name, "测试管理员");
    assert_eq!(account_resp.roles.len(), 1);
    assert!(account_resp.roles.iter().any(|i| i.1 == "tenant_admin"));
    assert!(!account_resp.token.is_empty());

    info!("【test_cp_all】 : Login by Username and Password, By sys admin");
    let account_resp = IamCpCertUserPwdServ::login_by_user_pwd(
        &mut IamCpUserPwdLoginReq {
            ak: TrimString("bios".to_string()),
            sk: TrimString(sysadmin_info.1.to_string()),
            tenant_id: None,
            flag: None,
        },
        &funs,
    )
    .await?;

    assert_eq!(account_resp.account_name, "bios");
    assert_eq!(account_resp.roles.len(), 1);
    assert!(account_resp.roles.iter().any(|i| i.1 == "sys_admin"));
    assert!(!account_resp.token.is_empty());

    let context = TardisContext {
        own_paths: context.own_paths.to_string(),
        ak: sysadmin_info.0.to_string(),
        owner: context.owner.to_string(),
        token: context.token.to_string(),
        token_kind: context.token_kind.to_string(),
        roles: account_resp.roles.iter().map(|i| i.0.to_string()).collect(),
        // TODO
        groups: vec![],
    };

    info!("【test_cp_all】 : Modify Password, original password error");
    assert!(IamCpCertUserPwdServ::modify_cert_user_pwd(
        &mut IamUserPwdCertModifyReq {
            original_sk: TrimString("12345".to_string()),
            new_sk: TrimString("123456".to_string()),
        },
        &funs,
        &context,
    )
    .await
    .is_err());

    info!("【test_cp_all】 : Modify Password");
    IamCpCertUserPwdServ::modify_cert_user_pwd(
        &mut IamUserPwdCertModifyReq {
            original_sk: TrimString(sysadmin_info.1.to_string()),
            new_sk: TrimString("123456".to_string()),
        },
        &funs,
        &context,
    )
    .await?;

    assert!(IamCpCertUserPwdServ::login_by_user_pwd(
        &mut IamCpUserPwdLoginReq {
            ak: TrimString("bios".to_string()),
            sk: TrimString(sysadmin_info.1.to_string()),
            tenant_id: None,
            flag: None
        },
        &funs,
    )
    .await
    .is_err());

    let account_resp = IamCpCertUserPwdServ::login_by_user_pwd(
        &mut IamCpUserPwdLoginReq {
            ak: TrimString("bios".to_string()),
            sk: TrimString("123456".to_string()),
            tenant_id: None,
            flag: None,
        },
        &funs,
    )
    .await?;

    assert_eq!(account_resp.account_name, "bios");
    assert_eq!(account_resp.roles.len(), 1);
    assert!(account_resp.roles.iter().any(|i| i.1 == "sys_admin"));
    assert!(!account_resp.token.is_empty());

    info!("【test_cp_all】 : Modify Current Account");
    IamCpAccountServ::modify_account(
        &mut IamCpAccountModifyReq {
            name: Some(TrimString("测试系统管理员".to_string())),
            icon: Some("/static/images/avatar.png".to_string()),
            disabled: Some(true),
        },
        &funs,
        &context,
    )
    .await?;

    info!("【test_cp_all】 : Get Current Account");
    let sysadmin = IamCpAccountServ::get_account(&funs, &context).await?;
    assert_eq!(sysadmin.name, "测试系统管理员");
    assert_eq!(sysadmin.icon, "/static/images/avatar.png");
    assert!(sysadmin.disabled);

    info!("【test_cp_all】 : Find Rel Roles By Current Account");
    let sysadmin_roles = IamCpAccountServ::paginate_rel_roles(1, 10, None, None, &funs, &context).await?;
    assert_eq!(sysadmin_roles.page_number, 1);
    assert_eq!(sysadmin_roles.page_size, 10);
    assert_eq!(sysadmin_roles.total_size, 1);
    assert_eq!(sysadmin_roles.records.len(), 1);
    assert_eq!(sysadmin_roles.records.get(0).unwrap().rel.from_rbum_item_name, "测试系统管理员");
    assert_eq!(sysadmin_roles.records.get(0).unwrap().rel.to_rbum_item_name, "sys_admin");

    funs.rollback().await?;

    Ok(())
}
