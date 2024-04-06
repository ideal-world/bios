use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;

use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use bios_iam::basic::dto::iam_app_dto::{IamAppAggAddReq, IamAppModifyReq};
use bios_iam::basic::dto::iam_filer_dto::IamAppFilterReq;
use bios_iam::basic::serv::iam_app_serv::IamAppServ;
use bios_iam::iam_constants;

pub async fn test(context1: &TardisContext, context2: &TardisContext) -> TardisResult<()> {
    let mut funs = iam_constants::get_tardis_inst();
    funs.begin().await?;

    info!("【test_ct_app】 : Add App");
    let app_id1 = IamAppServ::add_app_agg(
        &IamAppAggAddReq {
            app_name: TrimString("测试应用1".to_string()),
            app_icon: None,
            app_sort: None,
            app_contact_phone: None,
            disabled: None,
            admin_ids: Some(vec![context1.owner.to_string()]),
            set_cate_id: None,
        },
        &funs,
        context1,
    )
    .await?;

    IamAppServ::add_app_agg(
        &IamAppAggAddReq {
            app_name: TrimString("测试应用2".to_string()),
            app_icon: None,
            app_sort: None,
            app_contact_phone: None,
            disabled: None,
            admin_ids: Some(vec![context2.owner.to_string()]),
            set_cate_id: None,
        },
        &funs,
        context2,
    )
    .await?;

    info!("【test_ct_app】 : Modify App By Id, with err");
    assert!(IamAppServ::modify_item(
        &app_id1,
        &mut IamAppModifyReq {
            name: Some(TrimString("测试应用3".to_string())),
            icon: None,
            sort: None,
            contact_phone: Some("13333333333".to_string()),
            disabled: None,
            scope_level: None,
        },
        &funs,
        context2
    )
    .await
    .is_err());
    info!("【test_ct_app】 : Modify App By Id");
    IamAppServ::modify_item(
        &app_id1,
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

    info!("【test_ct_app】 : Get App By Id, with err");
    assert!(IamAppServ::get_item(&app_id1, &IamAppFilterReq::default(), &funs, context2).await.is_err());
    info!("【test_ct_app】 : Get App By Id");
    let app = IamAppServ::get_item(&app_id1, &IamAppFilterReq::default(), &funs, context1).await?;
    assert_eq!(app.id, app_id1);
    assert_eq!(app.name, "测试应用");
    assert_eq!(app.contact_phone, "13333333333");
    assert!(!app.disabled);

    info!("【test_ct_app】 : Find Apps");
    let apps = IamAppServ::paginate_items(
        &IamAppFilterReq {
            basic: RbumBasicFilterReq {
                ids: None,
                name: None,
                with_sub_own_paths: true,
                ..Default::default()
            },
            ..Default::default()
        },
        1,
        10,
        None,
        None,
        &funs,
        context1,
    )
    .await?;
    assert_eq!(apps.page_number, 1);
    assert_eq!(apps.page_size, 10);
    assert_eq!(apps.total_size, 1);
    assert!(apps.records.iter().any(|i| i.name == "测试应用"));

    info!("【test_ct_app】 : Delete App By Id, with err");
    assert!(IamAppServ::delete_item_with_all_rels("11111", &funs, context1).await.is_err());
    info!("【test_ct_app】 : Delete App By Id, with err");
    assert!(IamAppServ::delete_item_with_all_rels(&app_id1, &funs, context2).await.is_err());
    info!("【test_ct_app】 : Delete App By Id");
    assert_eq!(
        IamAppServ::paginate_items(
            &IamAppFilterReq {
                basic: RbumBasicFilterReq {
                    ids: Some(vec![app_id1.clone()]),
                    name: None,
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            1,
            10,
            None,
            None,
            &funs,
            context1,
        )
        .await?
        .total_size,
        1
    );
    assert!(IamAppServ::delete_item_with_all_rels(&app_id1, &funs, context1).await.is_err());

    funs.rollback().await?;

    Ok(())
}
