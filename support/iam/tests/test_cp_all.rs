use std::time::Duration;

use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;
use tardis::tokio::time::sleep;

use bios_basic::rbum::serv::rbum_cert_serv::RbumCertServ;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use bios_iam::basic::dto::iam_account_dto::IamAccountSelfModifyReq;
use bios_iam::basic::dto::iam_cert_conf_dto::IamCertConfUserPwdAddOrModifyReq;
use bios_iam::basic::dto::iam_cert_dto::{IamCertMailVCodeAddReq, IamCertUserNameNewReq, IamCertUserPwdModifyReq, IamContextFetchReq};
use bios_iam::basic::dto::iam_filer_dto::IamAccountFilterReq;
use bios_iam::basic::dto::iam_tenant_dto::{IamTenantAggAddReq, IamTenantConfigReq};
use bios_iam::basic::serv::iam_account_serv::IamAccountServ;
use bios_iam::basic::serv::iam_cert_mail_vcode_serv::IamCertMailVCodeServ;
use bios_iam::basic::serv::iam_cert_serv::IamCertServ;
use bios_iam::basic::serv::iam_key_cache_serv::IamIdentCacheServ;
use bios_iam::basic::serv::iam_tenant_serv::IamTenantServ;
use bios_iam::console_passport::dto::iam_cp_cert_dto::{IamCpMailVCodeLoginReq, IamCpUserPwdLoginReq};
use bios_iam::console_passport::serv::iam_cp_cert_mail_vcode_serv::IamCpCertMailVCodeServ;
use bios_iam::console_passport::serv::iam_cp_cert_user_pwd_serv::IamCpCertUserPwdServ;
use bios_iam::iam_constants;

pub async fn test(sysadmin_info: (&str, &str), system_admin_context: &TardisContext) -> TardisResult<()> {
    let mut funs = iam_constants::get_tardis_inst();
    funs.begin().await?;

    info!("【test_cp_all】 : Prepare : IamCsTenantServ::add_tenant");
    let (tenant_id, tenant_admin_pwd, _tenant_audit_pwd) = IamTenantServ::add_tenant_agg(
        &IamTenantAggAddReq {
            name: TrimString("测试租户1".to_string()),
            icon: None,
            contact_phone: None,
            note: None,
            admin_username: TrimString("bios".to_string()),
            admin_name: TrimString("测试管理员".to_string()),
            admin_password: None,
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
            //     sk_lock_cycle_sec: 5,
            //     sk_lock_err_times: 2,
            //     sk_lock_duration_sec: 5,
            //     repeatable: true,
            //     expire_sec: 111,
            // },
            // cert_conf_by_phone_vcode: true,
            // cert_conf_by_mail_vcode: true,
            disabled: None,
            account_self_reg: None,
            cert_conf_by_oauth2: None,
            cert_conf_by_ldap: None,
        },
        &funs,
    )
    .await?;
    sleep(Duration::from_secs(1)).await;

    let tenant_ctx = &TardisContext {
        own_paths: tenant_id.clone(),
        ..Default::default()
    };
    IamTenantServ::modify_tenant_config_agg(
        &tenant_id,
        &mut IamTenantConfigReq {
            cert_conf_by_user_pwd: Some(IamCertConfUserPwdAddOrModifyReq {
                ak_rule_len_min: 2,
                ak_rule_len_max: 20,
                sk_rule_len_min: 2,
                sk_rule_len_max: 20,
                sk_rule_need_num: false,
                sk_rule_need_uppercase: false,
                sk_rule_need_lowercase: false,
                sk_rule_need_spec_char: false,
                sk_lock_cycle_sec: 5,
                sk_lock_err_times: 2,
                sk_lock_duration_sec: 5,
                repeatable: true,
                expire_sec: 111,
            }),
            cert_conf_by_phone_vcode: Some(true),
            cert_conf_by_mail_vcode: Some(true),
            cert_conf_by_oauth2: None,
            cert_conf_by_ldap: None,
            config: None,
        },
        &funs,
        tenant_ctx,
    )
    .await?;

    info!("【test_cp_all】 : Login by Username and Password, Password error");
    assert!(IamCpCertUserPwdServ::login_by_user_pwd(
        &IamCpUserPwdLoginReq {
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
        &IamCpUserPwdLoginReq {
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
        &IamCpUserPwdLoginReq {
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
        &IamCpUserPwdLoginReq {
            ak: TrimString("bios".to_string()),
            sk: TrimString(tenant_admin_pwd.to_string()),
            tenant_id: Some(tenant_id.clone()),
            flag: None,
        },
        &funs,
    )
    .await?;
    let tenant_admin_context = IamIdentCacheServ::get_context(
        &IamContextFetchReq {
            token: account_resp.token.to_string(),
            app_id: None,
        },
        &funs,
    )
    .await?;

    assert_eq!(account_resp.account_name, "测试管理员");
    assert_eq!(account_resp.roles.len(), 1);
    assert!(account_resp.roles.iter().any(|i| i.1 == "tenant_admin"));
    assert!(!account_resp.token.is_empty());

    info!("【test_cp_all】 : Login by Username and Password, error 2");
    assert!(IamCpCertUserPwdServ::login_by_user_pwd(
        &IamCpUserPwdLoginReq {
            ak: TrimString("bios".to_string()),
            sk: TrimString("123456".to_string()),
            tenant_id: Some(tenant_id.clone()),
            flag: None
        },
        &funs,
    )
    .await
    .is_err());
    assert!(IamCpCertUserPwdServ::login_by_user_pwd(
        &IamCpUserPwdLoginReq {
            ak: TrimString("bios".to_string()),
            sk: TrimString("123456".to_string()),
            tenant_id: Some(tenant_id.clone()),
            flag: None
        },
        &funs,
    )
    .await
    .is_err());
    info!("【test_cp_all】 : Login by Username and Password, By tenant admin lock");
    assert!(IamCpCertUserPwdServ::login_by_user_pwd(
        &IamCpUserPwdLoginReq {
            ak: TrimString("bios".to_string()),
            sk: TrimString(tenant_admin_pwd.to_string()),
            tenant_id: Some(tenant_id.clone()),
            flag: None,
        },
        &funs,
    )
    .await
    .is_err());
    info!("【test_cp_all】 : Login by Username and Password, By tenant admin wait unlock");
    // 线程休息
    sleep(Duration::from_secs(5)).await;
    let account_unlock_resp = IamCpCertUserPwdServ::login_by_user_pwd(
        &IamCpUserPwdLoginReq {
            ak: TrimString("bios".to_string()),
            sk: TrimString(tenant_admin_pwd.to_string()),
            tenant_id: Some(tenant_id.clone()),
            flag: None,
        },
        &funs,
    )
    .await?;
    assert_eq!(account_unlock_resp.account_name, "测试管理员");
    assert_eq!(account_unlock_resp.roles.len(), 1);
    assert!(account_unlock_resp.roles.iter().any(|i| i.1 == "tenant_admin"));
    assert!(!account_unlock_resp.token.is_empty());

    info!("【test_cp_all】 : Login by Username and Password, By sys admin");
    let account_resp = IamCpCertUserPwdServ::login_by_user_pwd(
        &IamCpUserPwdLoginReq {
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

    let system_admin_context = TardisContext {
        own_paths: system_admin_context.own_paths.to_string(),
        ak: sysadmin_info.0.to_string(),
        owner: system_admin_context.owner.to_string(),
        roles: account_resp.roles.iter().map(|i| i.0.to_string()).collect(),
        // TODO
        groups: vec![],
        ..Default::default()
    };

    info!("【test_cp_all】 : Modify Password, original password error");
    assert!(IamCpCertUserPwdServ::modify_cert_user_pwd(
        &system_admin_context.owner,
        &IamCertUserPwdModifyReq {
            original_sk: TrimString("12345".to_string()),
            new_sk: TrimString("123456".to_string()),
        },
        &funs,
        &system_admin_context,
    )
    .await
    .is_err());

    info!("【test_cp_all】 : Modify Password");
    IamCpCertUserPwdServ::modify_cert_user_pwd(
        &system_admin_context.owner,
        &IamCertUserPwdModifyReq {
            original_sk: TrimString(sysadmin_info.1.to_string()),
            new_sk: TrimString("123456".to_string()),
        },
        &funs,
        &system_admin_context,
    )
    .await?;

    assert!(IamCpCertUserPwdServ::login_by_user_pwd(
        &IamCpUserPwdLoginReq {
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
        &IamCpUserPwdLoginReq {
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

    info!("【test_cp_all】 : modify cert ak");

    IamCpCertUserPwdServ::new_user_name(
        &IamCertUserNameNewReq {
            original_ak: "bios".into(),
            new_ak: "bios2".into(),
            sk: tenant_admin_pwd.clone().into(),
        },
        &funs,
        &tenant_admin_context,
    )
    .await?;

    let account_resp = IamCpCertUserPwdServ::login_by_user_pwd(
        &IamCpUserPwdLoginReq {
            ak: TrimString("bios2".to_string()),
            sk: TrimString(tenant_admin_pwd.clone()),
            tenant_id: Some(tenant_admin_context.own_paths.clone()),
            flag: None,
        },
        &funs,
    )
    .await?;
    assert_eq!(account_resp.account_name, "测试管理员");

    IamCpCertUserPwdServ::new_user_name(
        &IamCertUserNameNewReq {
            original_ak: "bios".into(),
            new_ak: "bios-admin".into(),
            sk: "123456".into(),
        },
        &funs,
        &system_admin_context,
    )
    .await?;

    let account_resp = IamCpCertUserPwdServ::login_by_user_pwd(
        &IamCpUserPwdLoginReq {
            ak: TrimString("bios-admin".to_string()),
            sk: TrimString("123456".to_string()),
            tenant_id: None,
            flag: None,
        },
        &funs,
    )
    .await?;
    assert_eq!(account_resp.account_name, "bios");
    // ------------------ Mail-VCode Cert Test Start ------------------

    info!("【test_cp_all】 : Add Mail-VCode Cert");
    let mail_vcode_cert_id = IamCpCertMailVCodeServ::add_cert_mail_vocde(
        &IamCertMailVCodeAddReq {
            mail: "i@sunisle.org".to_string(),
        },
        &funs,
        &tenant_admin_context,
    )
    .await?;

    let vcode = RbumCertServ::get_and_delete_vcode_in_cache("i@sunisle.org", &tenant_admin_context.own_paths, &funs).await?;
    assert!(vcode.is_some());

    info!("【test_cp_all】 : Resend Activation Mail");
    IamCertMailVCodeServ::resend_activation_mail(&tenant_admin_context.owner, "i@sunisle.org", &funs, &tenant_admin_context).await?;

    let vcode = RbumCertServ::get_vcode_in_cache("i@sunisle.org", &tenant_admin_context.own_paths, &funs).await?;
    assert!(vcode.is_some());

    info!("【test_cp_all】 : Activate Mail");
    IamCertMailVCodeServ::activate_mail("i@sunisle.org", &vcode.unwrap(), &funs, &tenant_admin_context).await?;

    info!("【test_cp_all】 : Send Login Mail");
    IamCertMailVCodeServ::send_login_mail("i@sunisle.org", &tenant_admin_context.own_paths, &funs).await?;
    let vcode = RbumCertServ::get_vcode_in_cache("i@sunisle.org", &tenant_admin_context.own_paths, &funs).await?;
    assert!(vcode.is_some());

    sleep(Duration::from_secs(1)).await;
    info!("【test_cp_all】 : Login by Mail And Vcode");
    IamCpCertMailVCodeServ::login_by_mail_vocde(
        &IamCpMailVCodeLoginReq {
            mail: "i@sunisle.org".to_string(),
            vcode: TrimString(vcode.unwrap()),
            tenant_id: tenant_admin_context.own_paths.clone(),
            flag: None,
        },
        &funs,
    )
    .await?;

    info!("【test_cp_all】 : Login by Mail And Pwd");
    let mail_account_resp = IamCpCertUserPwdServ::login_by_user_pwd(
        &IamCpUserPwdLoginReq {
            ak: TrimString("i@sunisle.org".to_string()),
            sk: TrimString(tenant_admin_pwd.to_string()),
            tenant_id: Some(tenant_admin_context.own_paths.clone()),
            flag: None,
        },
        &funs,
    )
    .await?;
    assert_eq!(mail_account_resp.account_name, "测试管理员");
    assert_eq!(mail_account_resp.roles.len(), 1);
    assert!(mail_account_resp.roles.iter().any(|i| i.1 == "tenant_admin"));
    assert!(!mail_account_resp.token.is_empty());

    info!("【test_cp_all】 : Delete Mail-VCode Cert");
    IamCertServ::delete_cert(&mail_vcode_cert_id, &funs, &tenant_admin_context).await?;

    // ------------------ Mail-VCode Cert Test End ------------------
    info!("【test_cp_all】 : Validate User Pwd");
    assert!(IamCpCertUserPwdServ::validate_by_user_pwd(sysadmin_info.1, false, &funs, &tenant_admin_context).await.is_err());
    IamCpCertUserPwdServ::validate_by_user_pwd(tenant_admin_pwd.as_str(), false, &funs, &tenant_admin_context).await?;
    info!("【test_cp_all】 : Send Bind Mail");
    IamCertMailVCodeServ::send_bind_mail("i@sunisle.org", &funs, &tenant_admin_context).await?;

    let vcode = RbumCertServ::get_vcode_in_cache("i@sunisle.org", &tenant_admin_context.own_paths, &funs).await?;
    assert!(vcode.is_some());

    info!("【test_cp_all】 : Bind Mail");
    let mail_vcode_cert_id = IamCertMailVCodeServ::bind_mail("i@sunisle.org", &vcode.unwrap(), &funs, &tenant_admin_context).await?;

    info!("【test_cp_all】 : Send Login Mail");
    IamCertMailVCodeServ::send_login_mail("i@sunisle.org", &tenant_admin_context.own_paths, &funs).await?;
    let vcode = RbumCertServ::get_vcode_in_cache("i@sunisle.org", &tenant_admin_context.own_paths, &funs).await?;
    assert!(vcode.is_some());

    sleep(Duration::from_secs(1)).await;
    info!("【test_cp_all】 : Login by Mail And Vcode");
    IamCpCertMailVCodeServ::login_by_mail_vocde(
        &IamCpMailVCodeLoginReq {
            mail: "i@sunisle.org".to_string(),
            vcode: TrimString(vcode.unwrap()),
            tenant_id: tenant_admin_context.own_paths.clone(),
            flag: None,
        },
        &funs,
    )
    .await?;
    info!("【test_cp_all】 : Delete Mail-VCode Cert");
    IamCertServ::delete_cert(&mail_vcode_cert_id, &funs, &tenant_admin_context).await?;
    // ------------------ Mail-VCode Cert Test End ------------------

    info!("【test_cp_all】 : Modify Current Account");
    IamAccountServ::self_modify_account(
        &mut IamAccountSelfModifyReq {
            name: Some(TrimString("测试系统管理员".to_string())),
            icon: Some("/static/images/avatar.png".to_string()),
            disabled: Some(true),
            exts: Default::default(),
        },
        &funs,
        &system_admin_context,
    )
    .await?;

    info!("【test_cp_all】 : Get Current Account");
    let sysadmin = IamAccountServ::get_item(&system_admin_context.owner, &IamAccountFilterReq::default(), &funs, &system_admin_context).await?;
    assert_eq!(sysadmin.name, "测试系统管理员");
    assert_eq!(sysadmin.icon, "/static/images/avatar.png");
    assert!(sysadmin.disabled);

    info!("【test_cp_all】 : Find Rel Roles By Current Account");
    let sysadmin_roles = IamAccountServ::find_simple_rel_roles(&system_admin_context.owner, false, None, None, &funs, &system_admin_context).await?;
    assert_eq!(sysadmin_roles.len(), 1);
    assert_eq!(sysadmin_roles.get(0).unwrap().rel_name, "sys_admin");

    funs.rollback().await?;

    Ok(())
}
