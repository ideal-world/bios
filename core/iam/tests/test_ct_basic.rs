use std::time::Duration;

use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;
use tardis::tokio::time::sleep;

use bios_iam::basic::dto::iam_cert_conf_dto::IamCertConfUserPwdAddOrModifyReq;
use bios_iam::basic::dto::iam_cert_dto::IamContextFetchReq;
use bios_iam::basic::dto::iam_tenant_dto::IamTenantAggAddReq;
use bios_iam::basic::serv::iam_key_cache_serv::IamIdentCacheServ;
use bios_iam::basic::serv::iam_tenant_serv::IamTenantServ;
use bios_iam::console_passport::dto::iam_cp_cert_dto::IamCpUserPwdLoginReq;
use bios_iam::console_passport::serv::iam_cp_cert_user_pwd_serv::IamCpCertUserPwdServ;
use bios_iam::iam_constants;

pub async fn test(_context: &TardisContext) -> TardisResult<(TardisContext, TardisContext)> {
    let mut funs = iam_constants::get_tardis_inst();
    funs.begin().await?;

    info!("【test_ct】 : Prepare : IamCsTenantServ::add_tenant");
    let (tenant_id, tenant_admin_pwd) = IamTenantServ::add_tenant_agg(
        &mut IamTenantAggAddReq {
            name: TrimString("测试租户1".to_string()),
            icon: None,
            contact_phone: None,
            note: None,
            admin_username: TrimString("bios".to_string()),
            disabled: None,
            admin_name: TrimString("测试管理员1".to_string()),
            admin_password: None,
            cert_conf_by_user_pwd: IamCertConfUserPwdAddOrModifyReq {
                ak_rule_len_min: 2,
                ak_rule_len_max: 20,
                sk_rule_len_min: 2,
                sk_rule_len_max: 20,
                sk_rule_need_num: false,
                sk_rule_need_uppercase: false,
                sk_rule_need_lowercase: false,
                sk_rule_need_spec_char: false,
                sk_lock_cycle_sec: 0,
                sk_lock_err_times: 0,
                sk_lock_duration_sec: 0,
                repeatable: true,
                expire_sec: 604800,
            },
            cert_conf_by_phone_vcode: true,
            cert_conf_by_mail_vcode: true,
            account_self_reg: None,
            cert_conf_by_wechat_mp: None,
            cert_conf_by_ldap: Vec::new(),
        },
        &funs,
    )
    .await?;
    sleep(Duration::from_secs(1)).await;

    let account_resp = IamCpCertUserPwdServ::login_by_user_pwd(
        &mut IamCpUserPwdLoginReq {
            ak: TrimString("bios".to_string()),
            sk: TrimString(tenant_admin_pwd),
            tenant_id: Some(tenant_id.clone()),
            flag: None,
        },
        &funs,
    )
    .await?;
    let context1 = IamIdentCacheServ::get_context(
        &IamContextFetchReq {
            token: account_resp.token.to_string(),
            app_id: None,
        },
        &funs,
    )
    .await?;

    let (tenant_id, tenant_admin_pwd) = IamTenantServ::add_tenant_agg(
        &mut IamTenantAggAddReq {
            name: TrimString("测试租户2".to_string()),
            icon: None,
            contact_phone: None,
            note: None,
            admin_username: TrimString("bios".to_string()),
            disabled: None,
            admin_name: TrimString("测试管理员2".to_string()),
            admin_password: None,
            cert_conf_by_user_pwd: IamCertConfUserPwdAddOrModifyReq {
                ak_rule_len_min: 2,
                ak_rule_len_max: 20,
                sk_rule_len_min: 2,
                sk_rule_len_max: 20,
                sk_rule_need_num: false,
                sk_rule_need_uppercase: false,
                sk_rule_need_lowercase: false,
                sk_rule_need_spec_char: false,
                sk_lock_cycle_sec: 0,
                sk_lock_err_times: 0,
                sk_lock_duration_sec: 0,
                repeatable: true,
                expire_sec: 111,
            },
            cert_conf_by_phone_vcode: true,

            cert_conf_by_mail_vcode: true,
            account_self_reg: None,
            cert_conf_by_wechat_mp: None,
            cert_conf_by_ldap: Vec::new(),
            
        },
        &funs,
    )
    .await?;
    sleep(Duration::from_secs(1)).await;

    let account_resp = IamCpCertUserPwdServ::login_by_user_pwd(
        &mut IamCpUserPwdLoginReq {
            ak: TrimString("bios".to_string()),
            sk: TrimString(tenant_admin_pwd),
            tenant_id: Some(tenant_id.clone()),
            flag: None,
        },
        &funs,
    )
    .await?;
    let context2 = IamIdentCacheServ::get_context(
        &IamContextFetchReq {
            token: account_resp.token.to_string(),
            app_id: None,
        },
        &funs,
    )
    .await?;

    funs.commit().await?;

    Ok((context1, context2))
}
