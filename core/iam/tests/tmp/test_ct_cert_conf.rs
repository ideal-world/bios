use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::log::info;

use bios_iam::basic::dto::iam_cert_conf_dto::{IamMailVCodeCertConfAddOrModifyReq, IamPhoneVCodeCertConfAddOrModifyReq, IamUserPwdCertConfAddOrModifyReq};
use bios_iam::console_tenant::serv::iam_ct_cert_serv::IamCtCertServ;
use bios_iam::iam_constants;
use bios_iam::iam_enumeration::IamCertKind;

pub async fn test(context1: &TardisContext, context2: &TardisContext) -> TardisResult<()> {
    let mut funs = iam_constants::get_tardis_inst();
    funs.begin().await?;

    info!("【test_ct_cert_conf】 : Find Cert Conf By UserPwd");
    let user_pwd_cert_conf = IamCtCertServ::paginate_cert_conf(None, Some(IamCertKind::UserPwd.to_string()), 1, 10, None, None, &funs, context1).await?;
    assert_eq!(user_pwd_cert_conf.page_number, 1);
    assert_eq!(user_pwd_cert_conf.page_size, 10);
    assert_eq!(user_pwd_cert_conf.total_size, 1);
    let cert_conf_user_pwd_id = user_pwd_cert_conf.records.get(0).unwrap().id.clone();

    info!("【test_ct_cert_conf】 : Modify Cert Config By UserPwd Kind");
    IamCtCertServ::modify_cert_conf_user_pwd(
        &cert_conf_user_pwd_id,
        &mut IamUserPwdCertConfAddOrModifyReq {
            ak_note: Some("ddddd1".to_string()),
            ak_rule: None,
            sk_note: None,
            sk_rule: None,
            repeatable: None,
            expire_sec: None,
        },
        &funs,
        context1,
    )
    .await?;

    info!("【test_ct_cert_conf】 : Add Cert Config By MailVCode Kind");
    let cert_conf_mail_vcode_id = IamCtCertServ::add_cert_conf_mail_vcode(
        &mut IamMailVCodeCertConfAddOrModifyReq {
            ak_note: Some("ddddd1".to_string()),
            ak_rule: None,
        },
        &funs,
        context1,
    )
    .await?;

    info!("【test_ct_cert_conf】 : Modify Cert Config By MailVCode Kind");
    IamCtCertServ::modify_cert_conf_mail_vcode(
        &cert_conf_mail_vcode_id,
        &mut IamMailVCodeCertConfAddOrModifyReq {
            ak_note: Some("ddddd1".to_string()),
            ak_rule: None,
        },
        &funs,
        context1,
    )
    .await?;

    info!("【test_ct_cert_conf】 : Add Cert Config By PhoneVCode Kind");
    let cert_conf_phone_vcode_id = IamCtCertServ::add_cert_conf_phone_vcode(
        &mut IamPhoneVCodeCertConfAddOrModifyReq {
            ak_note: Some("ddddd1".to_string()),
            ak_rule: None,
        },
        &funs,
        context1,
    )
    .await?;

    info!("【test_ct_cert_conf】 : Modify Cert Config By PhoneVCode Kind");
    IamCtCertServ::modify_cert_conf_phone_vcode(
        &cert_conf_phone_vcode_id,
        &mut IamPhoneVCodeCertConfAddOrModifyReq {
            ak_note: Some("ddddd1".to_string()),
            ak_rule: None,
        },
        &funs,
        context1,
    )
    .await?;

    info!("【test_ct_cert_conf】 : Get Cert Conf By Id, with err");
    assert!(IamCtCertServ::get_cert_conf(&cert_conf_phone_vcode_id, &funs, context2).await.is_err());
    info!("【test_ct_cert_conf】 : Get Cert Conf By Id");
    let cert_conf = IamCtCertServ::get_cert_conf(&cert_conf_user_pwd_id, &funs, context1).await?;
    assert_eq!(cert_conf.id, cert_conf_user_pwd_id);
    assert_eq!(cert_conf.ak_note, "ddddd1");
    let cert_conf = IamCtCertServ::get_cert_conf(&cert_conf_mail_vcode_id, &funs, context1).await?;
    assert_eq!(cert_conf.id, cert_conf_mail_vcode_id);
    assert_eq!(cert_conf.ak_note, "ddddd1");
    let cert_conf = IamCtCertServ::get_cert_conf(&cert_conf_phone_vcode_id, &funs, context1).await?;
    assert_eq!(cert_conf.id, cert_conf_phone_vcode_id);
    assert_eq!(cert_conf.ak_note, "ddddd1");

    info!("【test_ct_cert_conf】 : Find Cert Conf");
    let cert_conf = IamCtCertServ::paginate_cert_conf(None, None, 1, 10, None, None, &funs, context1).await?;
    assert_eq!(cert_conf.page_number, 1);
    assert_eq!(cert_conf.page_size, 10);
    assert_eq!(cert_conf.total_size, 7);

    info!("【test_ct_cert_conf】 : Delete Cert Conf By Id, with err");
    assert!(IamCtCertServ::delete_cert_conf("11111", &funs, &context1).await.is_err());
    info!("【test_ct_cert_conf】 : Delete Cert Conf By Id, with err");
    assert!(IamCtCertServ::delete_cert_conf(&cert_conf_phone_vcode_id, &funs, &context2).await.is_err());
    info!("【test_ct_cert_conf】 : Delete Cert Conf By Id");
    assert_eq!(
        IamCtCertServ::paginate_cert_conf(Some(cert_conf_phone_vcode_id.clone()), None, 1, 10, None, None, &funs, context1).await?.total_size,
        1
    );
    IamCtCertServ::delete_cert_conf(&cert_conf_phone_vcode_id, &funs, &context1).await?;
    assert_eq!(
        IamCtCertServ::paginate_cert_conf(Some(cert_conf_phone_vcode_id.clone()), None, 1, 10, None, None, &funs, context1).await?.total_size,
        0
    );

    funs.rollback().await?;

    Ok(())
}
