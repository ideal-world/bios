use std::time::Duration;

use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;
use tardis::tokio::time::sleep;
use tardis::TardisFuns;

use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use bios_iam::basic::dto::iam_account_dto::IamAccountAggAddReq;
use bios_iam::basic::dto::iam_app_dto::{IamAppAggAddReq, IamAppModifyReq};
use bios_iam::basic::dto::iam_cert_conf_dto::{IamCertConfTokenModifyReq, IamCertConfUserPwdAddOrModifyReq};
use bios_iam::basic::dto::iam_cert_dto::{IamCertUserPwdModifyReq, IamCertUserPwdRestReq, IamContextFetchReq};
use bios_iam::basic::dto::iam_res_dto::{IamResAddReq, IamResModifyReq};
use bios_iam::basic::dto::iam_role_dto::{IamRoleAddReq, IamRoleAggModifyReq, IamRoleModifyReq};
use bios_iam::basic::dto::iam_tenant_dto::{IamTenantAggAddReq, IamTenantModifyReq};
use bios_iam::basic::serv::iam_account_serv::IamAccountServ;
use bios_iam::basic::serv::iam_app_serv::IamAppServ;
use bios_iam::basic::serv::iam_cert_serv::IamCertServ;
use bios_iam::basic::serv::iam_cert_token_serv::IamCertTokenServ;
use bios_iam::basic::serv::iam_cert_user_pwd_serv::IamCertUserPwdServ;
use bios_iam::basic::serv::iam_key_cache_serv::{IamIdentCacheServ, IamResCacheServ};
use bios_iam::basic::serv::iam_res_serv::IamResServ;
use bios_iam::basic::serv::iam_role_serv::IamRoleServ;
use bios_iam::basic::serv::iam_tenant_serv::IamTenantServ;
use bios_iam::console_passport::dto::iam_cp_cert_dto::IamCpUserPwdLoginReq;
use bios_iam::console_passport::serv::iam_cp_cert_user_pwd_serv::IamCpCertUserPwdServ;
use bios_iam::iam_config::IamConfig;
use bios_iam::iam_constants;
use bios_iam::iam_constants::{RBUM_SCOPE_LEVEL_APP, RBUM_SCOPE_LEVEL_GLOBAL};
use bios_iam::iam_enumeration::{IamCertKernelKind, IamCertTokenKind, IamResKind};

pub async fn test(system_admin_context: &TardisContext) -> TardisResult<()> {
    let funs = iam_constants::get_tardis_inst();
    info!("【test_cc_cert】 : test_key_cache");
    let (tenant_id, tenant_admin_pwd, tenant_audit_pwd) = IamTenantServ::add_tenant_agg(
        &IamTenantAggAddReq {
            name: TrimString("缓存测试租户".to_string()),
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
            //     sk_lock_cycle_sec: 0,
            //     sk_lock_err_times: 0,
            //     repeatable: true,
            //     expire_sec: 111,
            //     sk_lock_duration_sec: 0,
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
    IamCertTokenServ::modify_cert_conf(
        &IamCertServ::get_cert_conf_id_by_kind(IamCertTokenKind::TokenDefault.to_string().as_str(), Some(tenant_id.clone()), &funs).await?,
        &IamCertConfTokenModifyReq {
            name: None,
            coexist_num: Some(2),
            expire_sec: None,
        },
        &funs,
        &IamCertServ::try_use_tenant_ctx(system_admin_context.clone(), Some(tenant_id.clone()))?,
    )
    .await?;
    sleep(Duration::from_secs(1)).await;

    info!("【test_key_cache】 Login by tenant admin, expected one token record");
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

    assert_eq!(
        TardisFuns::cache().get(&format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, account_resp.token)).await?.unwrap(),
        format!("TokenDefault,{}", account_resp.account_id)
    );
    assert_eq!(
        funs.cache().hlen(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, account_resp.account_id).as_str()).await?,
        1
    );
    assert!(funs
        .cache()
        .hget(
            format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, account_resp.account_id).as_str(),
            &account_resp.token,
        )
        .await?
        .unwrap()
        .contains("TokenDefault"));
    assert_eq!(
        funs.cache().hlen(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_info_, account_resp.account_id).as_str()).await?,
        1
    );

    info!("【test_key_cache】 Change cert, expected no token record");
    IamCpCertUserPwdServ::modify_cert_user_pwd(
        &account_resp.account_id,
        &IamCertUserPwdModifyReq {
            original_sk: TrimString(tenant_admin_pwd.clone()),
            new_sk: TrimString("123456".to_string()),
        },
        &funs,
        &tenant_admin_context,
    )
    .await?;
    assert!(TardisFuns::cache().get(&format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, account_resp.token)).await?.is_none());
    assert_eq!(
        funs.cache().hlen(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, account_resp.account_id).as_str()).await?,
        0
    );
    assert_eq!(
        funs.cache().hlen(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_info_, account_resp.account_id).as_str()).await?,
        0
    );

    info!("【test_key_cache】 Login by tenant admin, expected one token record");
    let account_resp = IamCpCertUserPwdServ::login_by_user_pwd(
        &IamCpUserPwdLoginReq {
            ak: TrimString("bios".to_string()),
            sk: TrimString("123456".to_string()),
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

    assert_eq!(
        TardisFuns::cache().get(&format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, account_resp.token)).await?.unwrap(),
        format!("TokenDefault,{}", account_resp.account_id)
    );
    assert_eq!(
        funs.cache().hlen(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, account_resp.account_id).as_str()).await?,
        1
    );
    assert!(funs
        .cache()
        .hget(
            format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, account_resp.account_id).as_str(),
            &account_resp.token,
        )
        .await?
        .unwrap()
        .contains("TokenDefault"));
    assert_eq!(
        funs.cache().hlen(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_info_, account_resp.account_id).as_str()).await?,
        1
    );

    info!("【test_key_cache】 Rest cert, expected no token record");
    IamCertUserPwdServ::reset_sk(
        &IamCertUserPwdRestReq {
            new_sk: TrimString("45678".to_string()),
        },
        &account_resp.account_id,
        &IamCertServ::get_cert_conf_id_by_kind(IamCertKernelKind::UserPwd.to_string().as_str(), Some(tenant_id.clone()), &funs).await?,
        &funs,
        &tenant_admin_context,
    )
    .await?;
    assert!(TardisFuns::cache().get(&format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, account_resp.token)).await?.is_none());
    assert_eq!(
        funs.cache().hlen(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, account_resp.account_id).as_str()).await?,
        0
    );
    assert_eq!(
        funs.cache().hlen(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_info_, account_resp.account_id).as_str()).await?,
        0
    );

    info!("【test_key_cache】 Login by tenant admin, expected one token record");
    let account_resp = IamCpCertUserPwdServ::login_by_user_pwd(
        &IamCpUserPwdLoginReq {
            ak: TrimString("bios".to_string()),
            sk: TrimString("45678".to_string()),
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

    assert_eq!(
        TardisFuns::cache().get(&format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, account_resp.token)).await?.unwrap(),
        format!("TokenDefault,{}", account_resp.account_id)
    );
    assert_eq!(
        funs.cache().hlen(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, account_resp.account_id).as_str()).await?,
        1
    );
    assert!(funs
        .cache()
        .hget(
            format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, account_resp.account_id).as_str(),
            &account_resp.token,
        )
        .await?
        .unwrap()
        .contains("TokenDefault"));
    assert_eq!(
        funs.cache().hlen(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_info_, account_resp.account_id).as_str()).await?,
        1
    );

    let account_id = IamAccountServ::add_account_agg(
        &IamAccountAggAddReq {
            id: None,
            name: TrimString("缓存应用管理员".to_string()),
            cert_user_name: TrimString("app_admin".to_string()),
            cert_password: Some(TrimString("123456".to_string())),
            cert_phone: None,
            cert_mail: None,
            icon: None,
            disabled: None,
            scope_level: Some(iam_constants::RBUM_SCOPE_LEVEL_TENANT),
            role_ids: None,
            org_node_ids: None,
            exts: Default::default(),
            status: None,
            temporary: None,
            lock_status: None,
        },
        false,
        &funs,
        &tenant_admin_context,
    )
    .await?;
    sleep(Duration::from_secs(1)).await;

    let app_id = IamAppServ::add_app_agg(
        &IamAppAggAddReq {
            app_name: TrimString("缓存测试应用".to_string()),
            app_icon: None,
            app_sort: None,
            app_contact_phone: None,
            disabled: None,
            admin_ids: Some(vec![account_id.clone()]),
        },
        &funs,
        &tenant_admin_context,
    )
    .await?;

    info!("【test_key_cache】 Delete token, expected no token record");
    IamCertTokenServ::delete_cert(&account_resp.token, &funs).await?;
    assert!(IamIdentCacheServ::get_context(
        &IamContextFetchReq {
            token: account_resp.token.to_string(),
            app_id: None,
        },
        &funs,
    )
    .await
    .is_err());

    assert!(TardisFuns::cache().get(&format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, account_resp.token)).await?.is_none());
    assert_eq!(
        funs.cache().hlen(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, account_id).as_str(),).await?,
        0
    );

    info!("【test_key_cache】 Login by tenant again, expected one token record");
    let account_resp1 = IamCpCertUserPwdServ::login_by_user_pwd(
        &IamCpUserPwdLoginReq {
            ak: TrimString("app_admin".to_string()),
            sk: TrimString("123456".to_string()),
            tenant_id: Some(tenant_id.clone()),
            flag: None,
        },
        &funs,
    )
    .await?;
    assert_eq!(
        TardisFuns::cache().get(&format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, account_resp1.token)).await?.unwrap(),
        format!("TokenDefault,{}", account_id)
    );
    assert_eq!(
        funs.cache().hlen(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, account_id).as_str(),).await?,
        1
    );
    assert!(funs
        .cache()
        .hget(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, account_id).as_str(), &account_resp1.token,)
        .await?
        .unwrap()
        .contains("TokenDefault"));
    assert_eq!(
        funs.cache().hlen(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_info_, account_id).as_str(),).await?,
        2
    );

    info!("【test_key_cache】 Login by tenant again, expected two token records");
    let account_resp2 = IamCpCertUserPwdServ::login_by_user_pwd(
        &IamCpUserPwdLoginReq {
            ak: TrimString("app_admin".to_string()),
            sk: TrimString("123456".to_string()),
            tenant_id: Some(tenant_id.clone()),
            flag: None,
        },
        &funs,
    )
    .await?;
    assert_eq!(
        TardisFuns::cache().get(&format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, account_resp2.token)).await?.unwrap(),
        format!("TokenDefault,{}", account_id)
    );
    assert_eq!(
        funs.cache().hlen(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, account_id).as_str(),).await?,
        2
    );
    assert!(funs
        .cache()
        .hget(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, account_id).as_str(), &account_resp2.token,)
        .await?
        .unwrap()
        .contains("TokenDefault"));
    assert_eq!(
        funs.cache().hlen(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_info_, account_id).as_str(),).await?,
        2
    );

    info!("【test_key_cache】 Login by tenant again, expected two token records");
    let account_resp3 = IamCpCertUserPwdServ::login_by_user_pwd(
        &IamCpUserPwdLoginReq {
            ak: TrimString("app_admin".to_string()),
            sk: TrimString("123456".to_string()),
            tenant_id: Some(tenant_id.clone()),
            flag: None,
        },
        &funs,
    )
    .await?;
    assert!(IamIdentCacheServ::get_context(
        &IamContextFetchReq {
            token: account_resp1.token.to_string(),
            app_id: None,
        },
        &funs,
    )
    .await
    .is_err());
    assert!(TardisFuns::cache().get(&format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, account_resp1.token)).await?.is_none());
    assert_eq!(
        TardisFuns::cache().get(&format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, account_resp3.token)).await?.unwrap(),
        format!("TokenDefault,{}", account_id)
    );
    assert_eq!(
        funs.cache().hlen(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, account_id).as_str(),).await?,
        2
    );
    assert!(funs
        .cache()
        .hget(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, account_id).as_str(), &account_resp3.token,)
        .await?
        .unwrap()
        .contains("TokenDefault"));
    assert_eq!(
        funs.cache().hlen(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_info_, account_id).as_str(),).await?,
        2
    );

    let app_admin_context = IamIdentCacheServ::get_context(
        &IamContextFetchReq {
            token: account_resp3.token.to_string(),
            app_id: None,
        },
        &funs,
    )
    .await?;
    assert!(app_admin_context.roles.is_empty());
    let app_admin_context = IamIdentCacheServ::get_context(
        &IamContextFetchReq {
            token: account_resp3.token.to_string(),
            app_id: Some(app_id.clone()),
        },
        &funs,
    )
    .await?;
    assert_eq!(app_admin_context.roles.len(), 1);

    //---------------------------------- Test Role ----------------------------------

    let role_id = app_admin_context.roles.get(0).unwrap();
    info!("【test_key_cache】 Disable role, expected no token record");
    IamRoleServ::modify_role_agg(
        role_id,
        &mut IamRoleAggModifyReq {
            role: IamRoleModifyReq {
                name: None,
                scope_level: None,
                disabled: Some(true),
                icon: None,
                sort: None,
                kind: None,
            },
            res_ids: None,
        },
        &funs,
        system_admin_context,
    )
    .await?;
    sleep(Duration::from_secs(1)).await;
    assert!(TardisFuns::cache().get(&format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, account_resp3.token)).await?.is_none());
    assert_eq!(
        funs.cache().hlen(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, account_id).as_str(),).await?,
        0
    );
    assert_eq!(
        funs.cache().hlen(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_info_, account_id).as_str(),).await?,
        0
    );

    info!("【test_key_cache】 Login again with disabled role, expected one token record");
    let account_resp = IamCpCertUserPwdServ::login_by_user_pwd(
        &IamCpUserPwdLoginReq {
            ak: TrimString("app_admin".to_string()),
            sk: TrimString("123456".to_string()),
            tenant_id: Some(tenant_id.clone()),
            flag: None,
        },
        &funs,
    )
    .await?;
    let app_admin_context = IamIdentCacheServ::get_context(
        &IamContextFetchReq {
            token: account_resp.token.to_string(),
            app_id: Some(app_id.clone()),
        },
        &funs,
    )
    .await?;
    assert_eq!(app_admin_context.roles.len(), 0);
    assert_eq!(
        TardisFuns::cache().get(&format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, account_resp.token)).await?.unwrap(),
        format!("TokenDefault,{}", account_id)
    );
    assert_eq!(
        funs.cache().hlen(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, account_id).as_str(),).await?,
        1
    );
    assert!(funs
        .cache()
        .hget(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, account_id).as_str(), &account_resp.token,)
        .await?
        .unwrap()
        .contains("TokenDefault"));
    assert_eq!(
        funs.cache().hlen(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_info_, account_id).as_str(),).await?,
        2
    );

    info!("【test_key_cache】 Enable role and login, expected two token records");
    IamRoleServ::modify_role_agg(
        role_id,
        &mut IamRoleAggModifyReq {
            role: IamRoleModifyReq {
                name: None,
                scope_level: None,
                disabled: Some(false),
                icon: None,
                sort: None,
                kind: None,
            },
            res_ids: None,
        },
        &funs,
        system_admin_context,
    )
    .await?;
    let account_resp = IamCpCertUserPwdServ::login_by_user_pwd(
        &IamCpUserPwdLoginReq {
            ak: TrimString("app_admin".to_string()),
            sk: TrimString("123456".to_string()),
            tenant_id: Some(tenant_id.clone()),
            flag: None,
        },
        &funs,
    )
    .await?;
    let app_admin_context = IamIdentCacheServ::get_context(
        &IamContextFetchReq {
            token: account_resp.token.to_string(),
            app_id: Some(app_id.clone()),
        },
        &funs,
    )
    .await?;
    assert_eq!(app_admin_context.roles.len(), 1);
    assert_eq!(
        TardisFuns::cache().get(&format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, account_resp.token)).await?.unwrap(),
        format!("TokenDefault,{}", account_id)
    );
    assert_eq!(
        funs.cache().hlen(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, account_id).as_str(),).await?,
        2
    );
    assert!(funs
        .cache()
        .hget(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, account_id).as_str(), &account_resp.token,)
        .await?
        .unwrap()
        .contains("TokenDefault"));
    assert_eq!(
        funs.cache().hlen(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_info_, account_id).as_str(),).await?,
        2
    );

    // todo: test account disabled
    info!("【test_key_cache】 Delete role rel, expected no token record");
    assert!(IamRoleServ::delete_rel_account(role_id, &account_id, None, &funs, &app_admin_context).await.is_err());
    // assert!(TardisFuns::cache().get(&format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, account_resp.token)).await?.is_none());
    // assert_eq!(
    //     funs.cache().hlen(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, account_id).as_str(),).await?,
    //     0
    // );
    // assert_eq!(
    //     funs.cache().hlen(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_info_, account_id).as_str(),).await?,
    //     0
    // );

    // info!("【test_key_cache】 Login again without role, expected one token record");
    // let account_resp = IamCpCertUserPwdServ::login_by_user_pwd(
    //     &IamCpUserPwdLoginReq {
    //         ak: TrimString("app_admin".to_string()),
    //         sk: TrimString("123456".to_string()),
    //         tenant_id: Some(tenant_id.clone()),
    //         flag: None,
    //     },
    //     &funs,
    // )
    // .await?;
    // let app_admin_context = IamIdentCacheServ::get_context(
    //     &IamContextFetchReq {
    //         token: account_resp.token.to_string(),
    //         app_id: Some(app_id.clone()),
    //     },
    //     &funs,
    // )
    // .await?;
    // assert_eq!(app_admin_context.roles.len(), 0);
    // assert_eq!(
    //     TardisFuns::cache().get(&format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, account_resp.token)).await?.unwrap(),
    //     format!("TokenDefault,{}", account_id)
    // );
    // assert_eq!(
    //     funs.cache().hlen(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, account_id).as_str(),).await?,
    //     1
    // );
    // assert!(funs
    //     .cache()
    //     .hget(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, account_id).as_str(), &account_resp.token,)
    //     .await?
    //     .unwrap()
    //     .contains("TokenDefault"));
    // assert_eq!(
    //     funs.cache().hlen(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_info_, account_id).as_str(),).await?,
    //     2
    // );

    //---------------------------------- Test App ----------------------------------

    info!("【test_key_cache】 Disable app, expected no token record");
    IamAppServ::modify_item(
        &app_id,
        &mut IamAppModifyReq {
            name: None,
            scope_level: None,
            disabled: Some(true),
            icon: None,
            sort: None,
            contact_phone: None,
        },
        &funs,
        system_admin_context,
    )
    .await?;
    sleep(Duration::from_secs(1)).await;

    assert!(TardisFuns::cache().get(&format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, account_resp.token)).await?.is_some());
    assert_eq!(
        funs.cache().hlen(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, account_id).as_str(),).await?,
        2
    );
    assert_eq!(
        funs.cache().hlen(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_info_, account_id).as_str(),).await?,
        2
    );

    info!("【test_key_cache】 Login again with disabled app, expected one token record");
    let account_resp = IamCpCertUserPwdServ::login_by_user_pwd(
        &IamCpUserPwdLoginReq {
            ak: TrimString("app_admin".to_string()),
            sk: TrimString("123456".to_string()),
            tenant_id: Some(tenant_id.clone()),
            flag: None,
        },
        &funs,
    )
    .await?;
    assert!(IamIdentCacheServ::get_context(
        &IamContextFetchReq {
            token: account_resp.token.to_string(),
            app_id: Some(app_id.clone()),
        },
        &funs,
    )
    .await
    .is_ok());
    let app_admin_context = IamIdentCacheServ::get_context(
        &IamContextFetchReq {
            token: account_resp.token.to_string(),
            app_id: None,
        },
        &funs,
    )
    .await?;
    assert_eq!(
        TardisFuns::cache().get(&format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, account_resp.token)).await?.unwrap(),
        format!("TokenDefault,{}", account_id)
    );
    assert_eq!(
        funs.cache().hlen(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, account_id).as_str(),).await?,
        2
    );
    assert!(funs
        .cache()
        .hget(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, account_id).as_str(), &account_resp.token,)
        .await?
        .unwrap()
        .contains("TokenDefault"));
    assert_eq!(
        funs.cache().hlen(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_info_, account_id).as_str(),).await?,
        2
    );

    info!("【test_key_cache】 Enable app and login, expected two token records");
    IamAppServ::modify_item(
        &app_id,
        &mut IamAppModifyReq {
            name: None,
            scope_level: None,
            disabled: Some(false),
            icon: None,
            sort: None,
            contact_phone: None,
        },
        &funs,
        system_admin_context,
    )
    .await?;
    let account_resp = IamCpCertUserPwdServ::login_by_user_pwd(
        &IamCpUserPwdLoginReq {
            ak: TrimString("app_admin".to_string()),
            sk: TrimString("123456".to_string()),
            tenant_id: Some(tenant_id.clone()),
            flag: None,
        },
        &funs,
    )
    .await?;
    let app_admin_context = IamIdentCacheServ::get_context(
        &IamContextFetchReq {
            token: account_resp.token.to_string(),
            app_id: Some(app_id.clone()),
        },
        &funs,
    )
    .await?;
    assert_eq!(
        TardisFuns::cache().get(&format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, account_resp.token)).await?.unwrap(),
        format!("TokenDefault,{}", account_id)
    );
    assert_eq!(
        funs.cache().hlen(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, account_id).as_str(),).await?,
        2
    );
    assert!(funs
        .cache()
        .hget(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, account_id).as_str(), &account_resp.token,)
        .await?
        .unwrap()
        .contains("TokenDefault"));
    assert_eq!(
        funs.cache().hlen(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_info_, account_id).as_str(),).await?,
        2
    );

    //---------------------------------- Test Tenant ----------------------------------

    info!("【test_key_cache】 Disable tenant, expected no token record");
    let tenant_ctx = IamCertServ::try_use_tenant_ctx(system_admin_context.clone(), Some(tenant_id.clone()))?;
    IamTenantServ::modify_item(
        &tenant_id,
        &mut IamTenantModifyReq {
            name: None,
            scope_level: None,
            disabled: Some(true),
            icon: None,
            sort: None,
            contact_phone: None,
            note: None,
            account_self_reg: None,
        },
        &funs,
        &tenant_ctx,
    )
    .await?;
    sleep(Duration::from_secs(1)).await;

    assert!(TardisFuns::cache().get(&format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, account_resp.token)).await?.is_none());
    assert_eq!(
        funs.cache().hlen(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, account_id).as_str(),).await?,
        0
    );
    assert_eq!(
        funs.cache().hlen(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_info_, account_id).as_str(),).await?,
        0
    );

    info!("【test_key_cache】 Login again with disabled tenant");
    assert!(IamCpCertUserPwdServ::login_by_user_pwd(
        &IamCpUserPwdLoginReq {
            ak: TrimString("app_admin".to_string()),
            sk: TrimString("123456".to_string()),
            tenant_id: Some(tenant_id.clone()),
            flag: None,
        },
        &funs,
    )
    .await
    .is_err());

    info!("【test_key_cache】 Enable tenant and login, expected one token record");
    IamTenantServ::modify_item(
        &tenant_id,
        &mut IamTenantModifyReq {
            name: None,
            scope_level: None,
            disabled: Some(false),
            icon: None,
            sort: None,
            contact_phone: None,
            note: None,
            account_self_reg: None,
        },
        &funs,
        &tenant_ctx,
    )
    .await?;
    let account_resp = IamCpCertUserPwdServ::login_by_user_pwd(
        &IamCpUserPwdLoginReq {
            ak: TrimString("app_admin".to_string()),
            sk: TrimString("123456".to_string()),
            tenant_id: Some(tenant_id.clone()),
            flag: None,
        },
        &funs,
    )
    .await?;
    let app_admin_context = IamIdentCacheServ::get_context(
        &IamContextFetchReq {
            token: account_resp.token.to_string(),
            app_id: Some(app_id.clone()),
        },
        &funs,
    )
    .await?;
    assert_eq!(
        TardisFuns::cache().get(&format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, account_resp.token)).await?.unwrap(),
        format!("TokenDefault,{}", account_id)
    );

    assert_eq!(
        funs.cache().hlen(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, account_id).as_str(),).await?,
        1
    );
    assert!(funs
        .cache()
        .hget(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, account_id).as_str(), &account_resp.token,)
        .await?
        .unwrap()
        .contains("TokenDefault"));
    assert_eq!(
        funs.cache().hlen(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_info_, account_id).as_str(),).await?,
        2
    );

    //---------------------------------- Test Res ----------------------------------

    let exists_res_counter = funs.cache().hlen(&funs.conf::<IamConfig>().cache_key_res_info).await?;

    info!("【test_key_cache】 Add res, expected two res records");
    let res_cs_id = IamResServ::add_item(
        &mut IamResAddReq {
            code: TrimString("iam/cs-2/**".to_string()),
            name: TrimString("系统控制台".to_string()),
            kind: IamResKind::Api,
            icon: None,
            sort: None,
            method: None,
            hide: None,
            action: None,
            scope_level: Some(RBUM_SCOPE_LEVEL_GLOBAL),
            disabled: None,
            crypto_req: Some(false),
            crypto_resp: Some(false),
            double_auth: Some(false),
            double_auth_msg: None,
        },
        &funs,
        system_admin_context,
    )
    .await?;
    let res_ca_id = IamResServ::add_item(
        &mut IamResAddReq {
            code: TrimString("iam/ca-2/**".to_string()),
            name: TrimString("应用控制台".to_string()),
            kind: IamResKind::Api,
            icon: None,
            sort: None,
            method: None,
            hide: None,
            action: None,
            scope_level: Some(RBUM_SCOPE_LEVEL_APP),
            disabled: None,
            crypto_req: Some(false),
            crypto_resp: Some(false),
            double_auth: Some(false),
            double_auth_msg: None,
        },
        &funs,
        system_admin_context,
    )
    .await?;
    assert_eq!(funs.cache().hlen(&funs.conf::<IamConfig>().cache_key_res_info).await?, exists_res_counter + 2);
    assert!(funs
        .cache()
        .hget(&funs.conf::<IamConfig>().cache_key_res_info, &IamResCacheServ::package_uri_mixed("iam/cs-2/**", "*"))
        .await?
        .unwrap()
        .contains(r#""need_double_auth":false"#));

    info!("【test_key_cache】 Disable res, expected one res record");
    IamResServ::modify_item(
        &res_cs_id,
        &mut IamResModifyReq {
            name: None,
            icon: None,
            sort: None,
            hide: None,
            action: None,
            scope_level: None,
            disabled: Some(true),
            crypto_req: None,
            crypto_resp: None,
            double_auth: None,
            double_auth_msg: None,
        },
        &funs,
        system_admin_context,
    )
    .await?;
    assert_eq!(funs.cache().hlen(&funs.conf::<IamConfig>().cache_key_res_info).await?, exists_res_counter + 1);
    assert!(funs.cache().hget(&funs.conf::<IamConfig>().cache_key_res_info, &IamResCacheServ::package_uri_mixed("iam/cs-2/**", "*")).await?.is_none());

    info!("【test_key_cache】 Enable res, expected two res records");
    IamResServ::modify_item(
        &res_cs_id,
        &mut IamResModifyReq {
            name: None,
            icon: None,
            sort: None,
            hide: None,
            action: None,
            scope_level: None,
            disabled: Some(false),
            crypto_req: None,
            crypto_resp: None,
            double_auth: None,
            double_auth_msg: None,
        },
        &funs,
        system_admin_context,
    )
    .await?;
    assert_eq!(funs.cache().hlen(&funs.conf::<IamConfig>().cache_key_res_info).await?, exists_res_counter + 2);
    assert!(funs
        .cache()
        .hget(&funs.conf::<IamConfig>().cache_key_res_info, &IamResCacheServ::package_uri_mixed("iam/cs-2/**", "*"))
        .await?
        .unwrap()
        .contains(r#"need_double_auth":false"#));

    info!("【test_key_cache】 Delete res, expected one res record");
    IamResServ::delete_item(&res_cs_id, &funs, system_admin_context).await?;
    assert_eq!(funs.cache().hlen(&funs.conf::<IamConfig>().cache_key_res_info).await?, exists_res_counter + 1);
    assert!(funs.cache().hget(&funs.conf::<IamConfig>().cache_key_res_info, &IamResCacheServ::package_uri_mixed("iam/cs-2/**", "*")).await?.is_none());

    info!("【test_key_cache】 Add role rel, expected one role rel record");
    IamRoleServ::add_rel_res(role_id, &res_ca_id, &funs, &app_admin_context).await?;
    assert_eq!(funs.cache().hlen(&funs.conf::<IamConfig>().cache_key_res_info).await?, exists_res_counter + 1);
    assert!(funs
        .cache()
        .hget(&funs.conf::<IamConfig>().cache_key_res_info, &IamResCacheServ::package_uri_mixed("iam/ca-2/**", "*"))
        .await?
        .unwrap()
        .contains(&format!(r##""roles":"#{}#""##, role_id)));

    info!("【test_key_cache】 Add role rel, expected two role rel records");
    let role_id1 = IamRoleServ::add_item(
        &mut IamRoleAddReq {
            code: Some(TrimString("role1".to_string())),
            name: TrimString("角色1".to_string()),
            icon: None,
            scope_level: None,
            disabled: None,
            sort: None,
            kind: None,
        },
        &funs,
        &app_admin_context,
    )
    .await?;
    IamRoleServ::add_rel_res(&role_id1, &res_ca_id, &funs, &app_admin_context).await?;
    assert_eq!(funs.cache().hlen(&funs.conf::<IamConfig>().cache_key_res_info).await?, exists_res_counter + 1);
    assert!(funs
        .cache()
        .hget(&funs.conf::<IamConfig>().cache_key_res_info, &IamResCacheServ::package_uri_mixed("iam/ca-2/**", "*"))
        .await?
        .unwrap()
        .contains(&format!(r##""roles":"#{}#{}#""##, role_id1, role_id)));

    info!("【test_key_cache】 Remove role rel, expected no role rel record");
    IamRoleServ::delete_rel_res(role_id, &res_ca_id, &funs, &app_admin_context).await?;
    assert_eq!(funs.cache().hlen(&funs.conf::<IamConfig>().cache_key_res_info).await?, exists_res_counter + 1);
    assert!(funs
        .cache()
        .hget(&funs.conf::<IamConfig>().cache_key_res_info, &IamResCacheServ::package_uri_mixed("iam/ca-2/**", "*"))
        .await?
        .unwrap()
        .contains(&format!(r##""roles":"#{}#""##, role_id1)));
    IamRoleServ::delete_rel_res(&role_id1, &res_ca_id, &funs, &app_admin_context).await?;
    assert_eq!(funs.cache().hlen(&funs.conf::<IamConfig>().cache_key_res_info).await?, exists_res_counter + 1);
    assert!(funs.cache().hget(&funs.conf::<IamConfig>().cache_key_res_info, &IamResCacheServ::package_uri_mixed("iam/ca-2/**", "*")).await?.unwrap().contains(r##""roles":"#""##));

    // ====================global account cache test===============================
    info!("【test_key_cache】 global account cache test, expected is_global is true");
    let mock_ctx = TardisContext { ..Default::default() };
    let account_id2 = IamAccountServ::add_account_agg(
        &IamAccountAggAddReq {
            id: None,
            name: TrimString("全局账号".to_string()),
            cert_user_name: TrimString("global".to_string()),
            cert_password: Some(TrimString("123456".to_string())),
            cert_phone: None,
            cert_mail: None,
            icon: None,
            disabled: None,
            scope_level: Some(iam_constants::RBUM_SCOPE_LEVEL_TENANT),
            role_ids: None,
            org_node_ids: None,
            exts: Default::default(),
            status: None,
            temporary: None,
            lock_status: None,
        },
        false,
        &funs,
        &mock_ctx,
    )
    .await?;

    let (tenant_id, _, _) = IamTenantServ::add_tenant_agg(
        &IamTenantAggAddReq {
            name: TrimString("测试租户1".to_string()),
            icon: None,
            contact_phone: None,
            note: None,
            admin_username: TrimString("admin".to_string()),
            disabled: None,
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
            //     sk_lock_cycle_sec: 0,
            //     sk_lock_err_times: 0,
            //     sk_lock_duration_sec: 0,
            //     repeatable: true,
            //     expire_sec: 111111111,
            // },
            // cert_conf_by_phone_vcode: true,
            // cert_conf_by_mail_vcode: true,
            account_self_reg: None,
            cert_conf_by_oauth2: None,
            cert_conf_by_ldap: None,
        },
        &funs,
    )
    .await?;

    let account_resp4 = IamCpCertUserPwdServ::login_by_user_pwd(
        &IamCpUserPwdLoginReq {
            ak: TrimString("global".to_string()),
            sk: TrimString("123456".to_string()),
            tenant_id: Some(tenant_id.clone()),
            flag: None,
        },
        &funs,
    )
    .await?;
    let tenant_admin_context = IamIdentCacheServ::get_context(
        &IamContextFetchReq {
            token: account_resp4.token.to_string(),
            app_id: None,
        },
        &funs,
    )
    .await?;
    let is_global = IamAccountServ::is_global_account(&account_resp4.account_id, &funs, &tenant_admin_context).await?;
    assert_eq!(is_global, true);
    assert_eq!(
        funs.cache().hlen(&format!("{}{}", funs.conf::<IamConfig>().cache_key_account_info_, account_resp4.account_id).as_str()).await?,
        2
    );

    info!("【test_key_cache】 global account cache test, expected is_global is flase");
    let account_id2 = IamAccountServ::add_account_agg(
        &IamAccountAggAddReq {
            id: None,
            name: TrimString("非全局账号".to_string()),
            cert_user_name: TrimString("not_global".to_string()),
            cert_password: Some(TrimString("123456".to_string())),
            cert_phone: None,
            cert_mail: None,
            icon: None,
            disabled: None,
            scope_level: Some(iam_constants::RBUM_SCOPE_LEVEL_TENANT),
            role_ids: None,
            org_node_ids: None,
            exts: Default::default(),
            status: None,
            temporary: None,
            lock_status: None,
        },
        false,
        &funs,
        &TardisContext {
            own_paths: tenant_id.clone(),
            owner: TardisFuns::field.nanoid(),
            ..Default::default()
        },
    )
    .await?;

    let account_resp4 = IamCpCertUserPwdServ::login_by_user_pwd(
        &IamCpUserPwdLoginReq {
            ak: TrimString("not_global".to_string()),
            sk: TrimString("123456".to_string()),
            tenant_id: Some(tenant_id.clone()),
            flag: None,
        },
        &funs,
    )
    .await?;
    let ctx = IamIdentCacheServ::get_context(
        &IamContextFetchReq {
            token: account_resp4.token.to_string(),
            app_id: None,
        },
        &funs,
    )
    .await?;
    let is_global = IamAccountServ::is_global_account(&account_id2, &funs, &ctx).await?;
    assert_eq!(is_global, false);
    assert_eq!(
        funs.cache().hlen(&format!("{}{}", funs.conf::<IamConfig>().cache_key_account_info_, account_id2).as_str()).await?,
        2
    );
    Ok(())
}
