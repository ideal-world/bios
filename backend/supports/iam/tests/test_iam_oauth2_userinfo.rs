use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;
use bios_basic::test::init_test_container;
use bios_iam::basic::dto::iam_account_dto::IamAccountAggAddReq;
use bios_iam::basic::serv::iam_account_serv::IamAccountServ;
use bios_iam::basic::serv::iam_cert_oauth2_service_serv::IamCertOAuth2ServiceServ;
use bios_iam::basic::serv::iam_cert_serv::IamCertServ;
use bios_iam::iam_constants;
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::{tokio, TardisFuns};

async fn add_contact_account(phone: Option<&str>, mail: Option<&str>, funs: &tardis::TardisFunsInst) -> TardisResult<String> {
    let account_id = TardisFuns::field.nanoid();
    let ctx = TardisContext {
        own_paths: "".to_string(),
        owner: account_id.clone(),
        ..Default::default()
    };
    IamAccountServ::add_account_agg(
        &IamAccountAggAddReq {
            id: Some(TrimString(account_id.clone())),
            name: TrimString(format!("userinfo-{account_id}")),
            cert_user_name: TrimString(format!("userinfo_{account_id}")),
            cert_password: Some(TrimString(IamCertServ::get_new_pwd())),
            cert_phone: phone.map(|phone| TrimString(phone.to_string())),
            cert_mail: mail.map(|mail| TrimString(mail.to_string())),
            role_ids: None,
            org_node_ids: None,
            scope_level: Some(RbumScopeLevelKind::Root),
            disabled: None,
            icon: None,
            exts: None,
            status: None,
            temporary: None,
            lock_status: None,
            logout_type: None,
            labor_type: None,
            id_card_no: None,
            employee_code: None,
            others_id: None,
        },
        false,
        funs,
        &ctx,
    )
    .await
}

#[tokio::test]
async fn userinfo_reads_login_name_phone_and_mail_from_kernel_certs() -> TardisResult<()> {
    let _containers = init_test_container::init(None).await?;
    let _ = bios_iam::iam_initializer::init_db(iam_constants::get_tardis_inst()).await?;
    let funs = iam_constants::get_tardis_inst();

    let account_id = add_contact_account(Some("13900000001"), Some("userinfo-contact@example.com"), &funs).await?;
    let userinfo = IamCertOAuth2ServiceServ::build_userinfo_by_account_id(&account_id, None, &funs).await?;
    assert_eq!(userinfo.phone.as_deref(), Some("13900000001"));
    assert_eq!(userinfo.mail.as_deref(), Some("userinfo-contact@example.com"));
    assert_eq!(userinfo.name, format!("userinfo-{account_id}"));
    assert_eq!(userinfo.user_name, Some(format!("userinfo_{account_id}")));

    let account_without_contact = add_contact_account(None, None, &funs).await?;
    let userinfo = IamCertOAuth2ServiceServ::build_userinfo_by_account_id(&account_without_contact, None, &funs).await?;
    assert_eq!(userinfo.phone, None);
    assert_eq!(userinfo.mail, None);
    assert_eq!(userinfo.user_name, Some(format!("userinfo_{account_without_contact}")));

    Ok(())
}
