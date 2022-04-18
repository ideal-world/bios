use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;

use bios_iam::console_tenant::dto::iam_ct_app_dto::{IamCtAppAddReq, IamCtAppModifyReq};
use bios_iam::console_tenant::serv::iam_ct_app_serv::IamCtAppServ;
use bios_iam::iam_constants;

pub async fn test(context1: &TardisContext, context2: &TardisContext) -> TardisResult<()> {
    let mut funs = iam_constants::get_tardis_inst();
    funs.begin().await?;

    info!("【test_ct_app】 : Add App");
    let app_id1 = IamCtAppServ::add_app(
        &mut IamCtAppAddReq {
            name: TrimString("测试应用1".to_string()),
            icon: None,
            sort: None,
            contact_phone: None,
            disabled: None,
        },
        &funs,
        context1,
    )
    .await?;

    IamCtAppServ::add_app(
        &mut IamCtAppAddReq {
            name: TrimString("测试应用2".to_string()),
            icon: None,
            sort: None,
            contact_phone: None,
            disabled: None,
        },
        &funs,
        context2,
    )
    .await?;

    info!("【test_ct_app】 : Modify App By Id, with err");
    assert!(IamCtAppServ::modify_app(
        &app_id1,
        &mut IamCtAppModifyReq {
            name: Some(TrimString("测试应用3".to_string())),
            icon: None,
            sort: None,
            contact_phone: Some("13333333333".to_string()),
            disabled: None
        },
        &funs,
        context2
    )
    .await
    .is_err());
    info!("【test_ct_app】 : Modify App By Id");
    IamCtAppServ::modify_app(
        &app_id1,
        &mut IamCtAppModifyReq {
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

    info!("【test_ct_app】 : Get App By Id, with err");
    assert!(IamCtAppServ::get_app(&app_id1, &funs, context2).await.is_err());
    info!("【test_ct_app】 : Get App By Id");
    let app = IamCtAppServ::get_app(&app_id1, &funs, context1).await?;
    assert_eq!(app.id, app_id1);
    assert_eq!(app.name, "测试应用");
    assert_eq!(app.contact_phone, "13333333333");
    assert!(!app.disabled);

    info!("【test_ct_app】 : Find Apps");
    let apps = IamCtAppServ::paginate_apps(None, None, 1, 10, None, None, &funs, context1).await?;
    assert_eq!(apps.page_number, 1);
    assert_eq!(apps.page_size, 10);
    assert_eq!(apps.total_size, 1);
    assert!(apps.records.iter().any(|i| i.name == "测试应用"));

    info!("【test_ct_app】 : Delete App By Id, with err");
    assert!(IamCtAppServ::delete_app("11111", &funs, &context1).await.is_err());
    info!("【test_ct_app】 : Delete App By Id, with err");
    assert!(IamCtAppServ::delete_app(&app_id1, &funs, &context2).await.is_err());
    info!("【test_ct_app】 : Delete App By Id");
    assert_eq!(
        IamCtAppServ::paginate_apps(Some(app_id1.clone()), None, 1, 10, None, None, &funs, context1).await?.total_size,
        1
    );
    IamCtAppServ::delete_app(&app_id1, &funs, &context1).await?;
    assert_eq!(
        IamCtAppServ::paginate_apps(Some(app_id1.clone()), None, 1, 10, None, None, &funs, context1).await?.total_size,
        0
    );

    funs.rollback().await?;

    Ok(())
}
