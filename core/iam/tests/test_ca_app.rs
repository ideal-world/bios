use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;

use bios_iam::console_app::dto::iam_ca_app_dto::IamCaAppModifyReq;
use bios_iam::console_app::serv::iam_ca_app_serv::IamCaAppServ;
use bios_iam::iam_constants;

pub async fn test(context1: &TardisContext, _context2: &TardisContext) -> TardisResult<()> {
    let mut funs = iam_constants::get_tardis_inst();
    funs.begin().await?;

    info!("【test_ca_app】 : Modify App By Id");
    IamCaAppServ::modify_app(
        &mut IamCaAppModifyReq {
            name: Some(TrimString("测试应用".to_string())),
            icon: None,
            sort: None,
            contact_phone: Some("13333333333".to_string()),
            disabled: None,
        },
        &funs,
        context1,
    )
    .await?;

    info!("【test_ca_app】 : Get App By Id");
    let app = IamCaAppServ::get_app(&funs, context1).await?;
    assert_eq!(app.name, "测试应用");
    assert_eq!(app.contact_phone, "13333333333");
    assert!(!app.disabled);

    funs.rollback().await?;

    Ok(())
}
