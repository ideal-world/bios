use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;

use bios_basic::rbum::dto::rbum_set_item_dto::RbumSetItemModifyReq;
use bios_iam::basic::dto::iam_set_dto::{IamSetCateAddReq, IamSetCateModifyReq, IamSetItemAddReq};
use bios_iam::basic::serv::iam_set_serv::IamSetServ;
use bios_iam::iam_constants;
use bios_iam::iam_constants::RBUM_SCOPE_LEVEL_APP;

pub async fn test(context1: &TardisContext, _: &TardisContext) -> TardisResult<()> {
    let mut funs = iam_constants::get_tardis_inst();
    funs.begin().await?;

    info!("【test_ca_set】 : Add Set Cate");
    let cate_id1 = IamSetServ::add_set_cate(
        &mut IamSetCateAddReq {
            bus_code: TrimString("bc1".to_string()),
            name: TrimString("xxx分公司".to_string()),
            icon: None,
            sort: None,
            ext: None,
            rbum_parent_cate_id: None,
        },
        true,
        RBUM_SCOPE_LEVEL_APP,
        &funs,
        context1,
    )
    .await?;

    let cate_id2 = IamSetServ::add_set_cate(
        &IamSetCateAddReq {
            bus_code: TrimString("bc2".to_string()),
            name: TrimString("yyy分公司".to_string()),
            icon: None,
            sort: None,
            ext: None,
            rbum_parent_cate_id: None,
        },
        true,
        RBUM_SCOPE_LEVEL_APP,
        &funs,
        context1,
    )
    .await?;

    let _cate_id3 = IamSetServ::add_set_cate(
        &IamSetCateAddReq {
            bus_code: TrimString("bc2-1".to_string()),
            name: TrimString("yyy分公司zzz部门".to_string()),
            icon: None,
            sort: None,
            ext: None,
            rbum_parent_cate_id: Some(cate_id2.clone()),
        },
        true,
        RBUM_SCOPE_LEVEL_APP,
        &funs,
        context1,
    )
    .await?;

    let cate_id4 = IamSetServ::add_set_cate(
        &IamSetCateAddReq {
            bus_code: TrimString("bc2-2".to_string()),
            name: TrimString("yyy分公司zzz部门".to_string()),
            icon: None,
            sort: None,
            ext: None,
            rbum_parent_cate_id: Some(cate_id2.clone()),
        },
        true,
        RBUM_SCOPE_LEVEL_APP,
        &funs,
        context1,
    )
    .await?;

    info!("【test_ca_set】 : Modify Set Cate By Id");
    IamSetServ::modify_set_cate(
        &cate_id4,
        &IamSetCateModifyReq {
            bus_code: Some(TrimString("bc2-xxxxx".to_string())),
            name: None,
            icon: None,
            sort: None,
            ext: None,
        },
        None,
        &funs,
        context1,
    )
    .await?;

    info!("【test_ca_set】 : Find Set Cate");
    let cates = IamSetServ::find_set_cates(true, &funs, context1).await?;
    assert_eq!(cates.len(), 4);

    info!("【test_ca_set】 : Delete Set Cate By Id");
    IamSetServ::delete_set_cate(&cate_id4, &funs, context1).await?;
    let cates = IamSetServ::find_set_cates(true, &funs, context1).await?;
    assert_eq!(cates.len(), 3);

    info!("【test_ca_set】 : Add Set Item");
    let item_id1 = IamSetServ::add_set_item(
        &cate_id1,
        &IamSetItemAddReq {
            sort: 0,
            rel_rbum_item_id: context1.owner.to_string(),
        },
        true,
        &funs,
        context1,
    )
    .await?;

    let _item_id2 = IamSetServ::add_set_item(
        &cate_id2,
        &IamSetItemAddReq {
            sort: 0,
            rel_rbum_item_id: context1.owner.to_string(),
        },
        true,
        &funs,
        context1,
    )
    .await?;

    info!("【test_ca_set】 : Modify Set Item By Id");
    IamSetServ::modify_set_item(&item_id1, &mut RbumSetItemModifyReq { sort: 10 }, &funs, context1).await?;

    info!("【test_ca_set】 : Find Set Item");
    let items = IamSetServ::find_set_items(&cate_id1, true, &funs, context1).await?;
    assert_eq!(items.len(), 1);

    info!("【test_ca_set】 : Delete Set Item By Id");
    IamSetServ::delete_set_item(&item_id1, &funs, context1).await?;
    let items = IamSetServ::find_set_items(&cate_id1, true, &funs, context1).await?;
    assert_eq!(items.len(), 0);

    funs.rollback().await?;

    Ok(())
}
