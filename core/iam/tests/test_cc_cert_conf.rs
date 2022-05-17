use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::log::info;

use bios_basic::rbum::helper::rbum_scope_helper::get_max_level_id_by_context;
use bios_iam::basic::dto::iam_cert_conf_dto::{IamMailVCodeCertConfAddOrModifyReq, IamPhoneVCodeCertConfAddOrModifyReq, IamUserPwdCertConfAddOrModifyReq};
use bios_iam::basic::serv::iam_cert_mail_vcode_serv::IamCertMailVCodeServ;
use bios_iam::basic::serv::iam_cert_phone_vcode_serv::IamCertPhoneVCodeServ;
use bios_iam::basic::serv::iam_cert_serv::IamCertServ;
use bios_iam::basic::serv::iam_cert_user_pwd_serv::IamCertUserPwdServ;
use bios_iam::iam_constants;
use bios_iam::iam_enumeration::IamCertKind;

pub async fn test(
    sys_context: &TardisContext,
    t1_context: &TardisContext,
    t2_context: &TardisContext,
    t2_a1_context: &TardisContext,
    t2_a2_context: &TardisContext,
) -> TardisResult<()> {
    test_single_level(sys_context, t1_context).await?;
    test_single_level(t1_context, t2_context).await?;
    // test_single_level(t2_a1_context, t2_a2_context).await?;
    Ok(())
}

async fn test_single_level(context: &TardisContext, another_context: &TardisContext) -> TardisResult<()> {
    let mut funs = iam_constants::get_tardis_inst();
    funs.begin().await?;

    info!("【test_ct_cert_conf】 : test_single_level : Find Cert Conf By UserPwd");
    let user_pwd_cert_conf = IamCertServ::paginate_cert_conf(None, Some(IamCertKind::UserPwd.to_string()), None, None, None, 1, 10, None, None, &funs, context).await?;
    assert_eq!(user_pwd_cert_conf.page_number, 1);
    assert_eq!(user_pwd_cert_conf.page_size, 10);
    assert_eq!(user_pwd_cert_conf.total_size, 1);
    let cert_conf_user_pwd_id = user_pwd_cert_conf.records.get(0).unwrap().id.clone();

    info!("【test_ct_cert_conf】 : test_single_level : Modify Cert Conf By UserPwd Kind");
    assert!(IamCertUserPwdServ::modify_cert_conf(
        &cert_conf_user_pwd_id,
        &IamUserPwdCertConfAddOrModifyReq {
            ak_note: Some("ddddd1".to_string()),
            ak_rule: None,
            sk_note: None,
            sk_rule: None,
            repeatable: None,
            expire_sec: None,
        },
        &funs,
        another_context,
    )
    .await
    .is_err());
    IamCertUserPwdServ::modify_cert_conf(
        &cert_conf_user_pwd_id,
        &IamUserPwdCertConfAddOrModifyReq {
            ak_note: Some("ddddd1".to_string()),
            ak_rule: None,
            sk_note: None,
            sk_rule: None,
            repeatable: None,
            expire_sec: None,
        },
        &funs,
        context,
    )
    .await?;

    info!("【test_ct_cert_conf】 : test_single_level : Modify Cert Conf By MailVCode Kind");
    let cert_conf_mail_vcode = IamCertServ::paginate_cert_conf(None, Some(IamCertKind::MailVCode.to_string()), None, None, None, 1, 10, None, None, &funs, context).await?;
    let cert_conf_mail_vcode_id = cert_conf_mail_vcode.records.get(0).unwrap().id.clone();
    IamCertMailVCodeServ::modify_cert_conf(
        &cert_conf_mail_vcode_id,
        &IamMailVCodeCertConfAddOrModifyReq {
            ak_note: Some("ddddd1".to_string()),
            ak_rule: None,
        },
        &funs,
        context,
    )
    .await?;

    info!("【test_ct_cert_conf】 : test_single_level : Modify Cert Conf By PhoneVCode Kind");
    let cert_conf_phone_vcode = IamCertServ::paginate_cert_conf(None, Some(IamCertKind::PhoneVCode.to_string()), None, None, None, 1, 10, None, None, &funs, context).await?;
    let cert_conf_phone_vcode_id = cert_conf_phone_vcode.records.get(0).unwrap().id.clone();
    IamCertPhoneVCodeServ::modify_cert_conf(
        &cert_conf_phone_vcode_id,
        &IamPhoneVCodeCertConfAddOrModifyReq {
            ak_note: Some("ddddd1".to_string()),
            ak_rule: None,
        },
        &funs,
        context,
    )
    .await?;

    info!("【test_ct_cert_conf】 : test_single_level : Get Cert Conf By Id");
    assert!(IamCertServ::get_cert_conf(&cert_conf_user_pwd_id, get_max_level_id_by_context(&context), &funs, another_context).await.is_err());
    let cert_conf = IamCertServ::get_cert_conf(&cert_conf_user_pwd_id, get_max_level_id_by_context(&context), &funs, context).await?;
    assert_eq!(cert_conf.id, cert_conf_user_pwd_id);
    assert_eq!(cert_conf.ak_note, "ddddd1");
    let cert_conf = IamCertServ::get_cert_conf(&cert_conf_mail_vcode_id, get_max_level_id_by_context(&context), &funs, context).await?;
    assert_eq!(cert_conf.id, cert_conf_mail_vcode_id);
    assert_eq!(cert_conf.ak_note, "ddddd1");
    let cert_conf = IamCertServ::get_cert_conf(&cert_conf_phone_vcode_id, get_max_level_id_by_context(&context), &funs, context).await?;
    assert_eq!(cert_conf.id, cert_conf_phone_vcode_id);
    assert_eq!(cert_conf.ak_note, "ddddd1");

    info!("【test_ct_cert_conf】 : test_single_level : Find Cert Conf");
    let cert_conf = IamCertServ::paginate_cert_conf(None, None, None, None, None, 1, 10, None, None, &funs, context).await?;
    assert_eq!(cert_conf.page_number, 1);
    assert_eq!(cert_conf.page_size, 10);
    assert_eq!(cert_conf.total_size, 7);

    info!("【test_ct_cert_conf】 : test_single_level : Delete Cert Conf By Id");
    assert!(IamCertServ::delete_cert_conf("11111", &funs, &context).await.is_err());
    assert!(IamCertServ::delete_cert_conf(&cert_conf_phone_vcode_id, &funs, &another_context).await.is_err());
    assert_eq!(
        IamCertServ::paginate_cert_conf(Some(cert_conf_phone_vcode_id.clone()), None, None, None, None, 1, 10, None, None, &funs, context).await?.total_size,
        1
    );
    IamCertServ::delete_cert_conf(&cert_conf_phone_vcode_id, &funs, &context).await?;
    assert_eq!(
        IamCertServ::paginate_cert_conf(Some(cert_conf_phone_vcode_id.clone()), None, None, None, None, 1, 10, None, None, &funs, context).await?.total_size,
        0
    );

    funs.rollback().await?;
    Ok(())
}
