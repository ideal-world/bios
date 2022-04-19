use std::time::Duration;
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;
use tardis::tokio::time::sleep;

use bios_iam::console_passport::dto::iam_cp_cert_dto::IamCpUserPwdLoginReq;
use bios_iam::console_passport::serv::iam_cp_cert_user_pwd_serv::IamCpCertUserPwdServ;
use bios_iam::console_system::dto::iam_cs_tenant_dto::IamCsTenantAddReq;
use bios_iam::console_system::serv::iam_cs_tenant_serv::IamCsTenantServ;
use bios_iam::iam_constants;
use bios_iam::iam_enumeration::IamCertTokenKind;

pub async fn test(context: &TardisContext) -> TardisResult<(TardisContext, TardisContext)> {
    let mut funs = iam_constants::get_tardis_inst();
    funs.begin().await?;

    info!("【test_ct】 : Prepare : IamCsTenantServ::add_tenant");
    let (tenant_id, tenant_admin_pwd) = IamCsTenantServ::add_tenant(
        &mut IamCsTenantAddReq {
            tenant_name: TrimString("测试租户1".to_string()),
            tenant_icon: None,
            tenant_contact_phone: None,
            admin_username: TrimString("bios".to_string()),
            disabled: None,
            admin_name: TrimString("测试管理员1".to_string()),
        },
        &funs,
        context,
    )
    .await?;
    sleep(Duration::from_secs(1)).await;

    let login_resp = IamCpCertUserPwdServ::login_by_user_pwd(
        &mut IamCpUserPwdLoginReq {
            ak: TrimString("bios".to_string()),
            sk: TrimString(tenant_admin_pwd),
            tenant_id: Some(tenant_id.clone()),
            flag: None,
        },
        &funs,
    )
    .await?;
    let context1 = TardisContext {
        own_paths: tenant_id.to_string(),
        ak: "bios".to_string(),
        owner: login_resp.id.to_string(),
        token: login_resp.token.to_string(),
        token_kind: IamCertTokenKind::TokenDefault.to_string(),
        roles: login_resp.roles.iter().map(|i| i.0.to_string()).collect(),
        // TODO
        groups: vec![],
    };

    let (tenant_id, tenant_admin_pwd) = IamCsTenantServ::add_tenant(
        &mut IamCsTenantAddReq {
            tenant_name: TrimString("测试租户2".to_string()),
            tenant_icon: None,
            tenant_contact_phone: None,
            admin_username: TrimString("bios".to_string()),
            disabled: None,
            admin_name: TrimString("测试管理员2".to_string()),
        },
        &funs,
        context,
    )
    .await?;
    sleep(Duration::from_secs(1)).await;

    let login_resp = IamCpCertUserPwdServ::login_by_user_pwd(
        &mut IamCpUserPwdLoginReq {
            ak: TrimString("bios".to_string()),
            sk: TrimString(tenant_admin_pwd),
            tenant_id: Some(tenant_id.clone()),
            flag: None,
        },
        &funs,
    )
    .await?;
    let context2 = TardisContext {
        own_paths: tenant_id.to_string(),
        ak: "bios".to_string(),
        owner: login_resp.id.to_string(),
        token: login_resp.token.to_string(),
        token_kind: IamCertTokenKind::TokenDefault.to_string(),
        roles: login_resp.roles.iter().map(|i| i.0.to_string()).collect(),
        // TODO
        groups: vec![],
    };

    funs.commit().await?;

    Ok((context1, context2))
}
