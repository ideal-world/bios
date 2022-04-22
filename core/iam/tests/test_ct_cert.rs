use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;

use bios_iam::basic::dto::iam_cert_dto::IamUserPwdCertRestReq;
use bios_iam::console_passport::dto::iam_cp_cert_dto::IamCpUserPwdLoginReq;
use bios_iam::console_passport::serv::iam_cp_cert_user_pwd_serv::IamCpCertUserPwdServ;
use bios_iam::console_tenant::serv::iam_ct_cert_serv::IamCtCertServ;
use bios_iam::iam_constants;

pub async fn test(context1: &TardisContext, context2: &TardisContext) -> TardisResult<()> {
    let mut funs = iam_constants::get_tardis_inst();
    funs.begin().await?;

    info!("【test_ct_cert】 : Rest Password, with err");
    assert!(IamCtCertServ::rest_password(
        &context2.owner,
        &mut IamUserPwdCertRestReq {
            new_sk: TrimString("sssssssssss".to_string())
        },
        &funs,
        context1
    )
    .await
    .is_err());
    assert!(IamCpCertUserPwdServ::login_by_user_pwd(
        &mut IamCpUserPwdLoginReq {
            ak: TrimString("bios".to_string()),
            sk: TrimString("sssssssssss".to_string()),
            tenant_id: Some(context1.own_paths.clone()),
            flag: None
        },
        &funs,
    )
    .await
    .is_err());

    info!("【test_ct_cert】 : Rest Password");
    IamCtCertServ::rest_password(
        &context1.owner,
        &mut IamUserPwdCertRestReq {
            new_sk: TrimString("sssssssssss".to_string()),
        },
        &funs,
        context1,
    )
    .await?;
    IamCpCertUserPwdServ::login_by_user_pwd(
        &mut IamCpUserPwdLoginReq {
            ak: TrimString("bios".to_string()),
            sk: TrimString("sssssssssss".to_string()),
            tenant_id: Some(context1.own_paths.clone()),
            flag: None,
        },
        &funs,
    )
    .await?;

    funs.rollback().await?;

    Ok(())
}
