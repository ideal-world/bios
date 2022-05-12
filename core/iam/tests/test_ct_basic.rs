use std::time::Duration;

use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;
use tardis::tokio::time::sleep;

use bios_iam::basic::dto::iam_cert_conf_dto::{IamMailVCodeCertConfAddOrModifyReq, IamPhoneVCodeCertConfAddOrModifyReq, IamUserPwdCertConfAddOrModifyReq};
use bios_iam::basic::dto::iam_cert_dto::IamContextFetchReq;
use bios_iam::basic::serv::iam_cert_serv::IamCertServ;
use bios_iam::console_passport::dto::iam_cp_cert_dto::IamCpUserPwdLoginReq;
use bios_iam::console_passport::serv::iam_cp_cert_user_pwd_serv::IamCpCertUserPwdServ;
use bios_iam::console_system::dto::iam_cs_tenant_dto::IamCsTenantAddReq;
use bios_iam::console_system::serv::iam_cs_tenant_serv::IamCsTenantServ;
use bios_iam::iam_constants;

pub async fn test(context: &TardisContext) -> TardisResult<(TardisContext, TardisContext)> {
    let mut funs = iam_constants::get_tardis_inst();
    funs.begin().await?;

    info!("【test_ct】 : Prepare : IamCsTenantServ::add_tenant");
    let (tenant_id, tenant_admin_pwd) = IamCsTenantServ::add_tenant(
        &mut IamCsTenantAddReq {
            tenant_name: TrimString("测试租户1".to_string()),
            tenant_icon: None,
            tenant_contact_phone: None,
            tenant_note: None,
            admin_username: TrimString("bios".to_string()),
            disabled: None,
            admin_name: TrimString("测试管理员1".to_string()),
            admin_password: None,
            cert_conf_by_user_pwd: IamUserPwdCertConfAddOrModifyReq {
                ak_note: None,
                ak_rule: None,
                sk_note: None,
                sk_rule: None,
                repeatable: Some(true),
                expire_sec: None
            },
            cert_conf_by_phone_vcode: Some(IamPhoneVCodeCertConfAddOrModifyReq{
                ak_note: None,
                ak_rule: None
            }),

            cert_conf_by_mail_vcode: Some(IamMailVCodeCertConfAddOrModifyReq{
                ak_note: None,
                ak_rule: None
            }),
        },
        &funs,
        context,
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
    let context1 = IamCertServ::fetch_context(
        &IamContextFetchReq {
            token: account_resp.token.to_string(),
            app_id: None,
        },
        &funs,
    )
    .await?;

    let (tenant_id, tenant_admin_pwd) = IamCsTenantServ::add_tenant(
        &mut IamCsTenantAddReq {
            tenant_name: TrimString("测试租户2".to_string()),
            tenant_icon: None,
            tenant_contact_phone: None,
            tenant_note: None,
            admin_username: TrimString("bios".to_string()),
            disabled: None,
            admin_name: TrimString("测试管理员2".to_string()),
            admin_password: None,
            cert_conf_by_user_pwd: IamUserPwdCertConfAddOrModifyReq {
                ak_note: None,
                ak_rule: None,
                sk_note: None,
                sk_rule: None,
                repeatable: Some(true),
                expire_sec: None
            },
            cert_conf_by_phone_vcode: Some(IamPhoneVCodeCertConfAddOrModifyReq{
                ak_note: None,
                ak_rule: None
            }),

            cert_conf_by_mail_vcode: Some(IamMailVCodeCertConfAddOrModifyReq{
                ak_note: None,
                ak_rule: None
            }),
        },
        &funs,
        context,
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
    let context2 = IamCertServ::fetch_context(
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
