use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;

use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use bios_iam::basic::dto::iam_app_dto::IamAppModifyReq;
use bios_iam::basic::dto::iam_filer_dto::IamAppFilterReq;
use bios_iam::basic::serv::iam_app_serv::IamAppServ;
use bios_iam::iam_constants;

pub async fn test(context1: &TardisContext, _context2: &TardisContext) -> TardisResult<()> {
    let mut funs = iam_constants::get_tardis_inst();
    funs.begin().await?;

    info!("【test_ca_app】 : Modify App By Id");
    IamAppServ::modify_item(
        &IamAppServ::get_id_by_cxt(context1, &funs)?,
        &mut IamAppModifyReq {
            name: Some(TrimString("测试应用".to_string())),
            icon: None,
            sort: None,
            contact_phone: Some("13333333333".to_string()),
            disabled: None,
            scope_level: None,
        },
        &funs,
        context1,
    )
    .await?;

    info!("【test_ca_app】 : Get App By Id");
    let app = IamAppServ::get_item(&IamAppServ::get_id_by_cxt(context1, &funs)?, &IamAppFilterReq::default(), &funs, context1).await?;
    assert_eq!(app.name, "测试应用");
    assert_eq!(app.contact_phone, "13333333333");
    assert!(!app.disabled);

    funs.rollback().await?;

    Ok(())
}
