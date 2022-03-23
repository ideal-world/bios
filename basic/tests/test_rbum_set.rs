use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;
use tardis::TardisFuns;

use bios_basic::rbum::dto::filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::dto::rbum_domain_dto::RbumDomainAddReq;
use bios_basic::rbum::dto::rbum_item_dto::RbumItemAddReq;
use bios_basic::rbum::dto::rbum_kind_dto::RbumKindAddReq;
use bios_basic::rbum::dto::rbum_set_cate_dto::{RbumSetCateAddReq, RbumSetCateModifyReq};
use bios_basic::rbum::dto::rbum_set_dto::{RbumSetAddReq, RbumSetModifyReq};
use bios_basic::rbum::dto::rbum_set_item_dto::{RbumSetItemAddReq, RbumSetItemModifyReq};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_domain_serv::RbumDomainServ;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemServ;
use bios_basic::rbum::serv::rbum_kind_serv::RbumKindServ;
use bios_basic::rbum::serv::rbum_set_serv::{RbumSetCateServ, RbumSetItemServ, RbumSetServ};

pub async fn test(context: &TardisContext) -> TardisResult<()> {
    test_rbum_set(context).await?;
    test_rbum_set_cate(context).await?;
    test_rbum_set_item(context).await?;
    Ok(())
}

async fn test_rbum_set(context: &TardisContext) -> TardisResult<()> {
    let mut tx = TardisFuns::reldb().conn();
    tx.begin().await?;

    info!("【test_rbum_set】 : Test Add : RbumSetServ::add_rbum");
    let id = RbumSetServ::add_rbum(
        &mut RbumSetAddReq {
            name: TrimString(" 测试集合 ".to_string()),
            note: None,
            icon: None,
            sort: None,
            tags: None,
            scope_level: 2,
        },
        &tx,
        context,
    )
    .await?;

    info!("【test_rbum_set】 : Test Get : RbumSetServ::get_rbum");
    let rbum = RbumSetServ::get_rbum(&id, &RbumBasicFilterReq::default(), &tx, context).await?;
    assert_eq!(rbum.id, id);
    assert_eq!(rbum.name, "测试集合");
    assert_eq!(rbum.scope_level, 2);

    info!("【test_rbum_set】 : Test Modify : RbumSetServ::modify_rbum");
    RbumSetServ::modify_rbum(
        &id,
        &mut RbumSetModifyReq {
            name: None,
            note: Some("remark".to_string()),
            icon: None,
            sort: None,
            tags: None,
            scope_level: None,
        },
        &tx,
        context,
    )
    .await?;

    info!("【test_rbum_set】 : Test Find : RbumSetServ::paginate_rbums");
    let rbums = RbumSetServ::paginate_rbums(&RbumBasicFilterReq::default(), 1, 10, None, None, &tx, context).await?;
    assert_eq!(rbums.page_number, 1);
    assert_eq!(rbums.page_size, 10);
    assert_eq!(rbums.total_size, 1);
    assert_eq!(rbums.records.get(0).unwrap().name, "测试集合");

    info!("【test_rbum_set】 : Test Delete : RbumSetServ::delete_rbum");
    RbumSetServ::delete_rbum(&id, &tx, context).await?;
    assert!(RbumSetServ::get_rbum(&id, &RbumBasicFilterReq::default(), &tx, context).await.is_err());

    tx.rollback().await?;

    Ok(())
}

async fn test_rbum_set_cate(context: &TardisContext) -> TardisResult<()> {
    let mut tx = TardisFuns::reldb().conn();
    tx.begin().await?;

    info!("【test_rbum_set_cate】 : Prepare Set : RbumSetServ::add_rbum");
    let set_id = RbumSetServ::add_rbum(
        &mut RbumSetAddReq {
            name: TrimString(" 测试集合 ".to_string()),
            note: None,
            icon: None,
            sort: None,
            tags: None,
            scope_level: 2,
        },
        &tx,
        context,
    )
    .await?;

    // -----------------------------------

    info!("【test_rbum_set_cate】 : Test Add : RbumSetCateServ::add_rbum");
    assert!(RbumSetCateServ::add_rbum(
        &mut RbumSetCateAddReq {
            bus_code: TrimString("".to_string()),
            name: TrimString("".to_string()),
            sort: None,
            rbum_sibling_cate_id: None,
            rbum_parent_cate_id: None,
            scope_level: 2,
            rel_rbum_set_id: "".to_string()
        },
        &tx,
        context,
    )
    .await
    .is_err());

    let l1_id = RbumSetCateServ::add_rbum(
        &mut RbumSetCateAddReq {
            bus_code: TrimString("".to_string()),
            name: TrimString("l1".to_string()),
            sort: None,
            rbum_sibling_cate_id: None,
            rbum_parent_cate_id: None,
            scope_level: 2,
            rel_rbum_set_id: set_id.to_string(),
        },
        &tx,
        context,
    )
    .await?;

    let l2_id = RbumSetCateServ::add_rbum(
        &mut RbumSetCateAddReq {
            bus_code: TrimString("".to_string()),
            name: TrimString("l2".to_string()),
            sort: None,
            rbum_sibling_cate_id: Some(l1_id.to_string()),
            rbum_parent_cate_id: None,
            scope_level: 2,
            rel_rbum_set_id: set_id.to_string(),
        },
        &tx,
        context,
    )
    .await?;

    let l3_id = RbumSetCateServ::add_rbum(
        &mut RbumSetCateAddReq {
            bus_code: TrimString("".to_string()),
            name: TrimString("l3".to_string()),
            sort: None,
            rbum_sibling_cate_id: Some(l1_id.to_string()),
            rbum_parent_cate_id: None,
            scope_level: 2,
            rel_rbum_set_id: set_id.to_string(),
        },
        &tx,
        context,
    )
    .await?;

    let l1_1_id = RbumSetCateServ::add_rbum(
        &mut RbumSetCateAddReq {
            bus_code: TrimString("".to_string()),
            name: TrimString("l1_1".to_string()),
            sort: None,
            rbum_sibling_cate_id: None,
            rbum_parent_cate_id: Some(l1_id.to_string()),
            scope_level: 2,
            rel_rbum_set_id: set_id.to_string(),
        },
        &tx,
        context,
    )
    .await?;

    let l2_1_id = RbumSetCateServ::add_rbum(
        &mut RbumSetCateAddReq {
            bus_code: TrimString("".to_string()),
            name: TrimString("l2_1".to_string()),
            sort: None,
            rbum_sibling_cate_id: None,
            rbum_parent_cate_id: Some(l2_id.to_string()),
            scope_level: 2,
            rel_rbum_set_id: set_id.to_string(),
        },
        &tx,
        context,
    )
    .await?;

    let l2_1_1_id = RbumSetCateServ::add_rbum(
        &mut RbumSetCateAddReq {
            bus_code: TrimString("".to_string()),
            name: TrimString("l2_1_1".to_string()),
            sort: None,
            rbum_sibling_cate_id: None,
            rbum_parent_cate_id: Some(l2_1_id.to_string()),
            scope_level: 2,
            rel_rbum_set_id: set_id.to_string(),
        },
        &tx,
        context,
    )
    .await?;

    let l2_1_2_id = RbumSetCateServ::add_rbum(
        &mut RbumSetCateAddReq {
            bus_code: TrimString("".to_string()),
            name: TrimString("l2_1_2".to_string()),
            sort: None,
            rbum_sibling_cate_id: Some(l2_1_1_id.to_string()),
            rbum_parent_cate_id: None,
            scope_level: 2,
            rel_rbum_set_id: set_id.to_string(),
        },
        &tx,
        context,
    )
    .await?;

    info!("【test_rbum_set_cate】 : Test Get : RbumSetCateServ::get_rbum");
    let rbum = RbumSetCateServ::get_rbum(&l2_1_2_id, &RbumBasicFilterReq::default(), &tx, context).await?;
    assert_eq!(rbum.id, l2_1_2_id);
    assert_eq!(rbum.name, "l2_1_2");

    info!("【test_rbum_set_cate】 : Test Modify : RbumSetCateServ::modify_rbum");
    RbumSetCateServ::modify_rbum(
        &l2_1_2_id,
        &mut RbumSetCateModifyReq {
            bus_code: Some(TrimString("dddddd".to_string())),
            name: None,
            sort: None,
            scope_level: None,
        },
        &tx,
        context,
    )
    .await?;

    info!("【test_rbum_set_cate】 : Test Find : RbumSetCateServ::paginate_rbums");
    let rbums = RbumSetCateServ::paginate_rbums(
        &RbumBasicFilterReq {
            name: Some("l2_1_2".to_string()),
            ..Default::default()
        },
        1,
        10,
        None,
        None,
        &tx,
        context,
    )
    .await?;
    assert_eq!(rbums.page_number, 1);
    assert_eq!(rbums.page_size, 10);
    assert_eq!(rbums.total_size, 1);
    assert_eq!(rbums.records.get(0).unwrap().name, "l2_1_2");
    assert_eq!(rbums.records.get(0).unwrap().bus_code, "dddddd");

    info!("【test_rbum_set_cate】 : Test Find By Set : RbumSetCateServ::get_tree_all");
    let rbums = RbumSetServ::get_tree_all(&set_id, &tx, context).await?;
    assert_eq!(rbums.len(), 7);
    assert_eq!(rbums.get(0).unwrap().id, l1_id);
    assert_eq!(rbums.get(0).unwrap().pid, None);
    assert_eq!(rbums.get(1).unwrap().id, l1_1_id);
    assert_eq!(rbums.get(1).unwrap().pid, Some(l1_id.clone()));
    assert_eq!(rbums.get(2).unwrap().id, l2_id);
    assert_eq!(rbums.get(2).unwrap().pid, None);
    assert_eq!(rbums.get(3).unwrap().id, l2_1_id);
    assert_eq!(rbums.get(3).unwrap().pid, Some(l2_id.clone()));
    assert_eq!(rbums.get(4).unwrap().id, l2_1_1_id);
    assert_eq!(rbums.get(4).unwrap().pid, Some(l2_1_id.clone()));
    assert_eq!(rbums.get(5).unwrap().id, l2_1_2_id);
    assert_eq!(rbums.get(5).unwrap().pid, Some(l2_1_id.clone()));
    assert_eq!(rbums.get(6).unwrap().id, l3_id);
    assert_eq!(rbums.get(6).unwrap().pid, None);

    info!("【test_rbum_set_cate】 : Test Find By Set : RbumSetCateServ::get_tree_by_level");
    let rbums = RbumSetServ::get_tree_by_level(&set_id, None, &tx, context).await?;
    assert_eq!(rbums.len(), 3);
    assert_eq!(rbums.get(0).unwrap().id, l1_id);
    assert_eq!(rbums.get(1).unwrap().id, l2_id);
    assert_eq!(rbums.get(2).unwrap().id, l3_id);
    let rbums = RbumSetServ::get_tree_by_level(&set_id, Some(&l1_id), &tx, context).await?;
    assert_eq!(rbums.len(), 1);
    assert_eq!(rbums.get(0).unwrap().id, l1_1_id);
    let rbums = RbumSetServ::get_tree_by_level(&set_id, Some(&l3_id), &tx, context).await?;
    assert_eq!(rbums.len(), 0);
    let rbums = RbumSetServ::get_tree_by_level(&set_id, Some(&l2_id), &tx, context).await?;
    assert_eq!(rbums.len(), 1);
    assert_eq!(rbums.get(0).unwrap().id, l2_1_id);
    let rbums = RbumSetServ::get_tree_by_level(&set_id, Some(&l2_1_id), &tx, context).await?;
    assert_eq!(rbums.len(), 2);
    assert_eq!(rbums.get(0).unwrap().id, l2_1_1_id);
    assert_eq!(rbums.get(1).unwrap().id, l2_1_2_id);

    info!("【test_rbum_set_cate】 : Test Delete : RbumSetCateServ::delete_rbum");
    RbumSetCateServ::delete_rbum(&l2_1_2_id, &tx, context).await?;
    assert!(RbumSetCateServ::get_rbum(&l2_1_2_id, &RbumBasicFilterReq::default(), &tx, context).await.is_err());

    tx.rollback().await?;

    Ok(())
}

async fn test_rbum_set_item(context: &TardisContext) -> TardisResult<()> {
    let mut tx = TardisFuns::reldb().conn();
    tx.begin().await?;

    info!("【test_rbum_set_item】 : Prepare Kind : RbumKindServ::add_rbum");
    let kind_account_id = RbumKindServ::add_rbum(
        &mut RbumKindAddReq {
            uri_scheme: TrimString("account".to_string()),
            name: TrimString("Account".to_string()),
            note: None,
            icon: None,
            sort: None,
            ext_table_name: None,
            scope_level: 2,
        },
        &tx,
        context,
    )
    .await?;

    info!("【test_rbum_set_item】 : Prepare Domain : RbumDomainServ::add_rbum");
    let domain_iam_id = RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            uri_authority: TrimString("iam2".to_string()),
            name: TrimString("IAM2".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: 2,
        },
        &tx,
        context,
    )
    .await?;

    info!("【test_rbum_set_item】 : Prepare Item : RbumItemServ::add_rbum");
    let item_account_a1_id = RbumItemServ::add_rbum(
        &mut RbumItemAddReq {
            id: None,
            uri_path: None,
            name: TrimString("用户1".to_string()),
            icon: None,
            sort: None,
            scope_level: 2,
            disabled: None,
            rel_rbum_kind_id: kind_account_id.to_string(),
            rel_rbum_domain_id: domain_iam_id.to_string(),
        },
        &tx,
        context,
    )
    .await?;

    info!("【test_rbum_set_item】 : Prepare Set : RbumSetServ::add_rbum");
    let set_id = RbumSetServ::add_rbum(
        &mut RbumSetAddReq {
            name: TrimString(" 测试集合 ".to_string()),
            note: None,
            icon: None,
            sort: None,
            tags: None,
            scope_level: 2,
        },
        &tx,
        context,
    )
    .await?;

    info!("【test_rbum_set_item】 : Prepare Set Cate : RbumSetCateServ::add_rbum");
    let set_cate_id = RbumSetCateServ::add_rbum(
        &mut RbumSetCateAddReq {
            bus_code: TrimString("".to_string()),
            name: TrimString("l1".to_string()),
            sort: None,
            rbum_sibling_cate_id: None,
            rbum_parent_cate_id: None,
            scope_level: 2,
            rel_rbum_set_id: set_id.to_string(),
        },
        &tx,
        context,
    )
    .await?;

    // -----------------------------------

    info!("【test_rbum_set_item】 : Test Add : RbumSetItemServ::add_rbum");
    assert!(RbumSetItemServ::add_rbum(
        &mut RbumSetItemAddReq {
            sort: 0,
            rel_rbum_set_id: "".to_string(),
            rel_rbum_set_cate_id: "".to_string(),
            rel_rbum_item_id: "".to_string()
        },
        &tx,
        context,
    )
    .await
    .is_err());

    assert!(RbumSetItemServ::add_rbum(
        &mut RbumSetItemAddReq {
            sort: 0,
            rel_rbum_set_id: set_id.to_string(),
            rel_rbum_set_cate_id: "".to_string(),
            rel_rbum_item_id: "".to_string()
        },
        &tx,
        context,
    )
    .await
    .is_err());

    assert!(RbumSetItemServ::add_rbum(
        &mut RbumSetItemAddReq {
            sort: 0,
            rel_rbum_set_id: set_id.to_string(),
            rel_rbum_set_cate_id: "".to_string(),
            rel_rbum_item_id: item_account_a1_id.to_string()
        },
        &tx,
        context,
    )
    .await
    .is_err());

    let id = RbumSetItemServ::add_rbum(
        &mut RbumSetItemAddReq {
            sort: 0,
            rel_rbum_set_id: set_id.to_string(),
            rel_rbum_set_cate_id: set_cate_id.to_string(),
            rel_rbum_item_id: item_account_a1_id.to_string(),
        },
        &tx,
        context,
    )
    .await?;

    info!("【test_rbum_set_item】 : Test Get : RbumSetItemServ::get_rbum");
    let rbum = RbumSetItemServ::get_rbum(&id, &RbumBasicFilterReq::default(), &tx, context).await?;
    assert_eq!(rbum.id, id);
    assert_eq!(rbum.sort, 0);
    assert_eq!(rbum.rel_rbum_set_cate_name, "l1");
    assert_eq!(rbum.rel_rbum_item_name, "用户1");

    info!("【test_rbum_set_item】 : Test Modify : RbumSetItemServ::modify_rbum");
    RbumSetItemServ::modify_rbum(&id, &mut RbumSetItemModifyReq { sort: 10 }, &tx, context).await?;

    info!("【test_rbum_set_item】 : Test Find : RbumSetItemServ::paginate_rbums");
    let rbums = RbumSetItemServ::paginate_rbums(&RbumBasicFilterReq::default(), 1, 10, None, None, &tx, context).await?;
    assert_eq!(rbums.page_number, 1);
    assert_eq!(rbums.page_size, 10);
    assert_eq!(rbums.total_size, 1);
    assert_eq!(rbums.records.get(0).unwrap().sort, 10);

    info!("【test_rbum_set_item】 : Test Delete : RbumSetItemServ::delete_rbum");
    RbumSetItemServ::delete_rbum(&id, &tx, context).await?;
    assert!(RbumSetItemServ::get_rbum(&id, &RbumBasicFilterReq::default(), &tx, context).await.is_err());

    tx.rollback().await?;

    Ok(())
}
