use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::web::web_resp::{TardisPage, Void};

use bios_basic::rbum::dto::rbum_cert_conf_dto::RbumCertConfDetailResp;
use bios_iam::basic::dto::iam_account_dto::AccountInfoResp;
use bios_iam::basic::dto::iam_cert_conf_dto::{IamMailVCodeCertConfAddOrModifyReq, IamPhoneVCodeCertConfAddOrModifyReq, IamUserPwdCertConfAddOrModifyReq};
use bios_iam::basic::dto::iam_tenant_dto::{IamTenantBoneResp, IamTenantDetailResp, IamTenantModifyReq, IamTenantSummaryResp};
use bios_iam::console_passport::dto::iam_cp_cert_dto::IamCpUserPwdLoginReq;
use bios_iam::console_system::dto::iam_cs_tenant_dto::IamCsTenantAddReq;
use bios_iam::iam_constants;
use bios_iam::iam_enumeration::IamCertKind;
use bios_iam::iam_test_helper::BIOSWebTestClient;

pub async fn test(client: &mut BIOSWebTestClient, sysadmin_name: &str, sysadmin_password: &str) -> TardisResult<()> {
    login_page(client, sysadmin_name, sysadmin_password).await?;
    Ok(())
}

pub async fn login_page(client: &mut BIOSWebTestClient, sysadmin_name: &str, sysadmin_password: &str) -> TardisResult<()> {
    // Fetch Tenants
    let tenants: Vec<IamTenantBoneResp> = client.get("/cp/tenant").await;
    assert_eq!(tenants.len(), 0);
    // Login
    let account: AccountInfoResp = client
        .put(
            "/cp/login/userpwd",
            &IamCpUserPwdLoginReq {
                ak: TrimString(sysadmin_name.to_string()),
                sk: TrimString(sysadmin_password.to_string()),
                tenant_id: None,
                flag: None,
            },
        )
        .await;
    assert_eq!(account.account_name, iam_constants::RBUM_ITEM_NAME_SYS_ADMIN_ACCOUNT);
    // Fetch Context
    client.set_auth(&account.token, None).await?;
    sys_tenant_mgr(client).await?;
    Ok(())
}

pub async fn sys_tenant_mgr(client: &mut BIOSWebTestClient) -> TardisResult<()> {
    // Fetch Tenants
    let tenants: TardisPage<IamTenantSummaryResp> = client.get("/cs/tenant?page_number=1&page_size=10").await;
    assert_eq!(tenants.total_size, 0);

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
                admin_password: None,
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
                cert_conf_by_phone_vcode: Some(IamPhoneVCodeCertConfAddOrModifyReq { ak_note: None, ak_rule: None }),
                cert_conf_by_mail_vcode: None,
                disabled: None,
            },
        )
        .await;

    // Get Tenant by Id
    let tenant: IamTenantDetailResp = client.get(&format!("/cs/tenant/{}", tenant_id)).await;
    assert_eq!(tenant.name, "测试公司1");
    assert_eq!(tenant.icon, "https://oss.minio.io/xxx.icon");

    // Get Cert Conf by Tenant Id
    let cert_conf: Vec<RbumCertConfDetailResp> = client.get(&format!("/cs/cert-conf?tenant_id={}", tenant_id)).await;
    let cert_conf_user_pwd = cert_conf.iter().find(|x| x.code == IamCertKind::UserPwd.to_string()).unwrap();
    let cert_conf_phone_vcode = cert_conf.iter().find(|x| x.code == IamCertKind::PhoneVCode.to_string()).unwrap();
    assert_eq!(cert_conf.len(), 2);
    assert!(cert_conf_user_pwd.sk_encrypted);
    assert!(!cert_conf_user_pwd.repeatable);

    // Modify Tenant
    let _: Void = client
        .put(
            &format!("/cs/tenant/{}", tenant_id),
            &IamTenantModifyReq {
                name: Some(TrimString("测试公司_new".to_string())),
                scope_level: None,
                disabled: None,
                icon: None,
                sort: None,
                contact_phone: None,
                note: None,
            },
        )
        .await;

    // Modify Cert Conf by User Pwd Id
    let _: Void = client
        .put(
            &format!("/cc/cert-conf/{}/user-pwd", cert_conf_user_pwd.id),
            &IamUserPwdCertConfAddOrModifyReq {
                ak_note: None,
                ak_rule: None,
                sk_note: None,
                sk_rule: None,
                repeatable: Some(false),
                expire_sec: Some(111),
            },
        )
        .await;

    // Delete Cert Conf by Id
    client.delete(format!("/cc/cert-conf/{}", cert_conf_phone_vcode.id).as_str()).await;

    // Add Cert Conf by User Pwd
    let _: Void = client
        .post(
            &format!("/cs/cert-conf/mail-vcode?tenant_id={}", tenant_id),
            &IamMailVCodeCertConfAddOrModifyReq { ak_note: None, ak_rule: None },
        )
        .await;

    // Fetch Account Total
    let tenants: u64 = client.get(&format!("/cs/account/total?tenant_id={}", tenant_id)).await;
    assert_eq!(tenants, 0);

    Ok(())
}
