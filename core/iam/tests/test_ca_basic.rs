use std::time::Duration;

use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;
use tardis::tokio::time::sleep;

use bios_iam::basic::dto::iam_account_dto::IamAccountAggAddReq;
use bios_iam::basic::dto::iam_cert_conf_dto::{IamMailVCodeCertConfAddOrModifyReq, IamPhoneVCodeCertConfAddOrModifyReq, IamUserPwdCertConfAddOrModifyReq};
use bios_iam::basic::dto::iam_cert_dto::IamContextFetchReq;
use bios_iam::basic::serv::iam_account_serv::IamAccountServ;
use bios_iam::basic::serv::iam_cert_serv::IamCertServ;
use bios_iam::basic::serv::iam_key_cache_serv::IamIdentCacheServ;
use bios_iam::console_passport::dto::iam_cp_cert_dto::IamCpUserPwdLoginReq;
use bios_iam::console_passport::serv::iam_cp_cert_user_pwd_serv::IamCpCertUserPwdServ;
use bios_iam::console_system::dto::iam_cs_tenant_dto::IamCsTenantAddReq;
use bios_iam::console_system::serv::iam_cs_tenant_serv::IamCsTenantServ;
use bios_iam::console_tenant::dto::iam_ct_app_dto::IamCtAppAddReq;
use bios_iam::console_tenant::serv::iam_ct_app_serv::IamCtAppServ;
use bios_iam::iam_constants;

pub async fn test(_context: &TardisContext) -> TardisResult<(TardisContext, TardisContext, TardisContext)> {
    let mut funs = iam_constants::get_tardis_inst();
    funs.begin().await?;

    info!("【test_ca】 : Prepare : IamCsTenantServ::add_tenant");
    let (tenant_id, tenant_admin_pwd) = IamCsTenantServ::add_tenant(
        &mut IamCsTenantAddReq {
            tenant_name: TrimString("测试租户1".to_string()),
            tenant_icon: None,
            tenant_contact_phone: None,
            tenant_note: None,
            admin_username: TrimString("bios".to_string()),
            admin_name: TrimString("测试管理员1".to_string()),
            admin_password: None,
            cert_conf_by_user_pwd: IamUserPwdCertConfAddOrModifyReq {
                ak_note: None,
                ak_rule: None,
                sk_note: None,
                sk_rule: None,
                repeatable: Some(true),
                expire_sec: None,
            },
            cert_conf_by_phone_vcode: Some(IamPhoneVCodeCertConfAddOrModifyReq { ak_note: None, ak_rule: None }),

            cert_conf_by_mail_vcode: Some(IamMailVCodeCertConfAddOrModifyReq { ak_note: None, ak_rule: None }),
            disabled: None,
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
            cert_password: TrimString(pwd.to_string()),
            cert_phone: None,
            cert_mail: None,
            icon: None,
            disabled: None,
            scope_level: Some(iam_constants::RBUM_SCOPE_LEVEL_TENANT),
            role_ids: None,
            org_node_ids: None,
            exts: Default::default(),
        },
        &funs,
        &tenant_context,
    )
    .await?;

    let account_id2 = IamAccountServ::add_account_agg(
        &IamAccountAggAddReq {
            id: None,
            name: TrimString("应用2管理员".to_string()),
            cert_user_name: TrimString("app_admin2".to_string()),
            cert_password: TrimString(pwd.to_string()),
            cert_phone: None,
            cert_mail: None,
            icon: None,
            disabled: None,
            scope_level: Some(iam_constants::RBUM_SCOPE_LEVEL_TENANT),
            role_ids: None,
            org_node_ids: None,
            exts: Default::default(),
        },
        &funs,
        &tenant_context,
    )
    .await?;

    let app_id1 = IamCtAppServ::add_app(
        &mut IamCtAppAddReq {
            app_name: TrimString("测试应用1".to_string()),
            app_icon: None,
            app_sort: None,
            app_contact_phone: None,
            disabled: None,
            admin_id: account_id1.clone(),
        },
        &funs,
        &tenant_context,
    )
    .await?;

    let app_id2 = IamCtAppServ::add_app(
        &mut IamCtAppAddReq {
            app_name: TrimString("测试应用2".to_string()),
            app_icon: None,
            app_sort: None,
            app_contact_phone: None,
            disabled: None,
            admin_id: account_id2.clone(),
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
