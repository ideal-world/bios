use bios_iam::basic::dto::iam_cert_conf_dto::IamCertConfOAuth2AddOrModifyReq;
use bios_iam::basic::dto::iam_tenant_dto::IamTenantAggModifyReq;
use bios_iam::basic::serv::iam_cert_oauth2_serv::IamCertOAuth2Serv;
use bios_iam::basic::serv::iam_tenant_serv::IamTenantServ;
use bios_iam::iam_constants;
use bios_iam::iam_enumeration::IamCertOAuth2Supplier;
use ldap3::log::info;
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::db::sea_orm::IntoActiveValue;

pub async fn test(tenant1_admin_context: &TardisContext) -> TardisResult<()> {
    const GITHUB_OAUTH2_AK: &str = "";
    const GITHUB_OAUTH2_SK: &str = "";
    // Manually splicing address to obtain code
    // https://github.com/login/oauth/authorize?client_id={GITHUB_OAUTH2_AK}&redirect_uri=http://localhost/
    let code = "";

    let mut funs = iam_constants::get_tardis_inst();
    funs.begin().await?;
    let id = &IamTenantServ::get_id_by_ctx(tenant1_admin_context, &funs)?;
    IamTenantServ::modify_tenant_agg(
        id,
        &IamTenantAggModifyReq {
            name: None,
            icon: None,
            sort: None,
            contact_phone: None,
            note: None,
            account_self_reg: None,
            disabled: None,
            cert_conf_by_user_pwd: None,
            cert_conf_by_phone_vcode: None,
            cert_conf_by_mail_vcode: None,
            cert_conf_by_oauth2: Some(vec![IamCertConfOAuth2AddOrModifyReq {
                supplier: "Github".into(),
                ak: GITHUB_OAUTH2_AK.into(),
                sk: GITHUB_OAUTH2_SK.into(),
            }]),
            cert_conf_by_ldap: None,
        },
        &funs,
        tenant1_admin_context,
    )
    .await?;

    let account = IamCertOAuth2Serv::get_or_add_account(IamCertOAuth2Supplier::Github, code, id, &funs).await?;
    info!("account info= {:?}", account);
    Ok(())
}
