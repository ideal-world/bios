use std::time::Duration;

use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;
use tardis::tokio::time::sleep;

use bios_iam::basic::dto::iam_account_dto::IamAccountAggAddReq;
use bios_iam::basic::dto::iam_app_dto::IamAppAggAddReq;
use bios_iam::basic::dto::iam_cert_conf_dto::IamCertConfUserPwdAddOrModifyReq;
use bios_iam::basic::dto::iam_cert_dto::IamContextFetchReq;
use bios_iam::basic::dto::iam_tenant_dto::IamTenantAggAddReq;
use bios_iam::basic::serv::iam_account_serv::IamAccountServ;
use bios_iam::basic::serv::iam_app_serv::IamAppServ;
use bios_iam::basic::serv::iam_cert_serv::IamCertServ;
use bios_iam::basic::serv::iam_key_cache_serv::IamIdentCacheServ;
use bios_iam::basic::serv::iam_tenant_serv::IamTenantServ;
use bios_iam::console_passport::dto::iam_cp_cert_dto::IamCpUserPwdLoginReq;
use bios_iam::console_passport::serv::iam_cp_cert_user_pwd_serv::IamCpCertUserPwdServ;
use bios_iam::iam_constants;

pub async fn test(_context: &TardisContext) -> TardisResult<(TardisContext, TardisContext, TardisContext)> {
    let mut funs = iam_constants::get_tardis_inst();
    funs.begin().await?;

    info!("【test_ca】 : Prepare : IamCsTenantServ::add_tenant");
    let (tenant_id, tenant_admin_pwd, tenant_audit_pwd) = IamTenantServ::add_tenant_agg(
        &mut IamTenantAggAddReq {
            name: TrimString("测试租户1".to_string()),
            icon: None,
            contact_phone: None,
            note: None,
            admin_username: TrimString("bios".to_string()),
            admin_name: TrimString("测试管理员1".to_string()),
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
            //     expire_sec: 604800,
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

    let account_resp = IamCpCertUserPwdServ::login_by_user_pwd(
        &IamCpUserPwdLoginReq {
            ak: TrimString("bios".to_string()),
            sk: TrimString(tenant_admin_pwd),
            tenant_id: Some(tenant_id.clone()),
            flag: None,
        },
        &funs,
    )
    .await?;
    let tenant_context = IamIdentCacheServ::get_context(
        &IamContextFetchReq {
            token: account_resp.token.to_string(),
            app_id: None,
        },
        &funs,
    )
    .await?;

    let pwd = IamCertServ::get_new_pwd();

    let account_id1 = IamAccountServ::add_account_agg(
        &IamAccountAggAddReq {
            id: None,
            name: TrimString("应用1管理员".to_string()),
            cert_user_name: TrimString("app_admin1".to_string()),
            cert_password: Some(TrimString(pwd.to_string())),
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
        &tenant_context,
    )
    .await?;

    let account_id2 = IamAccountServ::add_account_agg(
        &IamAccountAggAddReq {
            id: None,
            name: TrimString("应用2管理员".to_string()),
            cert_user_name: TrimString("app_admin2".to_string()),
            cert_password: Some(TrimString(pwd.to_string())),
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
        &tenant_context,
    )
    .await?;

    let app_id1 = IamAppServ::add_app_agg(
        &mut IamAppAggAddReq {
            app_name: TrimString("测试应用1".to_string()),
            app_icon: None,
            app_sort: None,
            app_contact_phone: None,
            disabled: None,
            admin_ids: Some(vec![account_id1.to_string()]),
        },
        &funs,
        &tenant_context,
    )
    .await?;

    let app_id2 = IamAppServ::add_app_agg(
        &IamAppAggAddReq {
            app_name: TrimString("测试应用2".to_string()),
            app_icon: None,
            app_sort: None,
            app_contact_phone: None,
            disabled: None,
            admin_ids: Some(vec![account_id2.clone()]),
        },
        &funs,
        &tenant_context,
    )
    .await?;

    let account_resp = IamCpCertUserPwdServ::login_by_user_pwd(
        &IamCpUserPwdLoginReq {
            ak: TrimString("app_admin1".to_string()),
            sk: TrimString(pwd.clone()),
            tenant_id: Some(tenant_id.clone()),
            flag: None,
        },
        &funs,
    )
    .await?;
    let app_context1 = IamIdentCacheServ::get_context(
        &IamContextFetchReq {
            token: account_resp.token.to_string(),
            app_id: Some(app_id1.clone()),
        },
        &funs,
    )
    .await?;

    let account_resp = IamCpCertUserPwdServ::login_by_user_pwd(
        &IamCpUserPwdLoginReq {
            ak: TrimString("app_admin2".to_string()),
            sk: TrimString(pwd),
            tenant_id: Some(tenant_id.clone()),
            flag: None,
        },
        &funs,
    )
    .await?;
    let app_context2 = IamIdentCacheServ::get_context(
        &IamContextFetchReq {
            token: account_resp.token.to_string(),
            app_id: Some(app_id2.clone()),
        },
        &funs,
    )
    .await?;

    funs.commit().await?;

    Ok((app_context1, app_context2, tenant_context))
}
