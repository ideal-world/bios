use bios_basic::rbum::dto::rbum_domain_dto::RbumDomainAddReq;
use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumSetCateFilterReq, RbumSetFilterReq, RbumSetItemFilterReq, RbumSetTreeFilterReq};
use bios_basic::rbum::dto::rbum_item_dto::RbumItemAddReq;
use bios_basic::rbum::dto::rbum_kind_dto::RbumKindAddReq;
use bios_basic::rbum::dto::rbum_set_cate_dto::{RbumSetCateAddReq, RbumSetCateModifyReq};
use bios_basic::rbum::dto::rbum_set_dto::{RbumSetAddReq, RbumSetModifyReq};
use bios_basic::rbum::dto::rbum_set_item_dto::{RbumSetItemAddReq, RbumSetItemModifyReq};
use bios_basic::rbum::rbum_enumeration::{RbumScopeLevelKind, RbumSetCateLevelQueryKind};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_domain_serv::RbumDomainServ;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemServ;
use bios_basic::rbum::serv::rbum_kind_serv::RbumKindServ;
use bios_basic::rbum::serv::rbum_set_serv::{RbumSetCateServ, RbumSetItemServ, RbumSetServ};
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;
use tardis::TardisFuns;

pub async fn test(context: &TardisContext) -> TardisResult<()> {
    test_rbum_set(context).await?;
    test_rbum_set_cate(context).await?;
    test_rbum_set_item(context).await?;
    Ok(())
}

async fn test_rbum_set(context: &TardisContext) -> TardisResult<()> {
    let mut funs = TardisFuns::inst_with_db_conn("".to_string(), None);
    funs.begin().await?;

    info!("【test_rbum_set】 : Test Add : RbumSetServ::add_rbum");
    let id = RbumSetServ::add_rbum(
        &mut RbumSetAddReq {
            code: TrimString("test_rbum_set_code".to_string()),
            kind: TrimString("".to_string()),
            name: TrimString(" 测试集合 ".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: Some(RbumScopeLevelKind::L2),
            ext: None,
            disabled: None,
        },
        &funs,
        context,
    )
    .await?;

    info!("【test_rbum_set】 : Test Get : RbumSetServ::get_rbum");
    let rbum = RbumSetServ::get_rbum(&id, &RbumSetFilterReq::default(), &funs, context).await?;
    assert_eq!(rbum.id, id);
    assert_eq!(rbum.name, "测试集合");
    assert_eq!(rbum.scope_level, RbumScopeLevelKind::L2);

    info!("【test_rbum_set】 : Test Modify : RbumSetServ::modify_rbum");
    RbumSetServ::modify_rbum(
        &id,
        &mut RbumSetModifyReq {
            name: None,
            note: Some("remark".to_string()),
            icon: None,
            sort: None,
            scope_level: None,
            ext: None,
            disabled: None,
        },
        &funs,
        context,
    )
    .await?;

    info!("【test_rbum_set】 : Test Find : RbumSetServ::paginate_rbums");
    let rbums = RbumSetServ::paginate_rbums(&RbumSetFilterReq::default(), 1, 10, None, None, &funs, context).await?;
    assert_eq!(rbums.page_number, 1);
    assert_eq!(rbums.page_size, 10);
    assert_eq!(rbums.total_size, 1);
    assert_eq!(rbums.records.first().unwrap().name, "测试集合");

    info!("【test_rbum_set】 : Test Delete : RbumSetServ::delete_rbum");
    RbumSetServ::delete_rbum(&id, &funs, context).await?;
    assert!(RbumSetServ::get_rbum(&id, &RbumSetFilterReq::default(), &funs, context).await.is_err());

    funs.rollback().await?;

    Ok(())
}

async fn test_rbum_set_cate(context: &TardisContext) -> TardisResult<()> {
    let mut funs = TardisFuns::inst_with_db_conn("".to_string(), None);
    funs.begin().await?;

    info!("【test_rbum_set_cate】 : Prepare Set : RbumSetServ::add_rbum");
    let set_id = RbumSetServ::add_rbum(
        &mut RbumSetAddReq {
            code: TrimString("sss".to_string()),
            kind: TrimString("".to_string()),
            name: TrimString(" 测试集合 ".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: Some(RbumScopeLevelKind::L2),
            ext: None,
            disabled: None,
        },
        &funs,
        context,
    )
    .await?;
    info!("【test_rbum_set_cate】 : Prepare Set : RbumKindServ::add_rbum");
    let kind_app_id = RbumKindServ::add_rbum(
        &mut RbumKindAddReq {
            code: TrimString("app".to_string()),
            name: TrimString("APP".to_string()),
            module: None,
            note: None,
            icon: None,
            sort: None,
            ext_table_name: None,
            scope_level: Some(RbumScopeLevelKind::L2),
        },
        &funs,
        context,
    )
    .await?;

    info!("【test_rbum_set_cate】 : Prepare Domain : RbumDomainServ::add_rbum");
    let domain_iam_id = RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            code: TrimString("iam2".to_string()),
            name: TrimString("IAM2".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: Some(RbumScopeLevelKind::L2),
        },
        &funs,
        context,
    )
    .await?;

    info!("【test_rbum_set_cate】 : Prepare Item : RbumItemServ::add_rbum");
    let item_app_id = RbumItemServ::add_rbum(
        &mut RbumItemAddReq {
            id: None,
            code: None,
            name: TrimString("应用1".to_string()),
            scope_level: Some(RbumScopeLevelKind::L2),
            disabled: None,
            rel_rbum_kind_id: kind_app_id.to_string(),
            rel_rbum_domain_id: domain_iam_id.to_string(),
        },
        &funs,
        context,
    )
    .await?;

    // -----------------------------------

    info!("【test_rbum_set_cate】 : Test Add : RbumSetCateServ::add_rbum");
    assert!(RbumSetCateServ::add_rbum(
        &mut RbumSetCateAddReq {
            bus_code: TrimString("".to_string()),
            name: TrimString("".to_string()),
            icon: None,
            sort: None,
            ext: None,
            rbum_parent_cate_id: None,
            scope_level: Some(RbumScopeLevelKind::L2),
            rel_rbum_set_id: "".to_string(),
        },
        &funs,
        context,
    )
    .await
    .is_err());

    let l1_id = RbumSetCateServ::add_rbum(
        &mut RbumSetCateAddReq {
            bus_code: TrimString("".to_string()),
            name: TrimString("l1".to_string()),
            icon: None,
            sort: None,
            ext: None,
            rbum_parent_cate_id: None,
            scope_level: Some(RbumScopeLevelKind::L2),
            rel_rbum_set_id: set_id.to_string(),
        },
        &funs,
        context,
    )
    .await?;

    let l2_id = RbumSetCateServ::add_rbum(
        &mut RbumSetCateAddReq {
            bus_code: TrimString("".to_string()),
            name: TrimString("l2".to_string()),
            icon: None,
            sort: None,
            ext: None,
            rbum_parent_cate_id: None,
            scope_level: Some(RbumScopeLevelKind::L2),
            rel_rbum_set_id: set_id.to_string(),
        },
        &funs,
        context,
    )
    .await?;

    let l3_id = RbumSetCateServ::add_rbum(
        &mut RbumSetCateAddReq {
            bus_code: TrimString("".to_string()),
            name: TrimString("l3".to_string()),
            icon: None,
            sort: None,
            ext: None,
            rbum_parent_cate_id: None,
            scope_level: Some(RbumScopeLevelKind::L2),
            rel_rbum_set_id: set_id.to_string(),
        },
        &funs,
        context,
    )
    .await?;

    let l1_1_id = RbumSetCateServ::add_rbum(
        &mut RbumSetCateAddReq {
            bus_code: TrimString("".to_string()),
            name: TrimString("l1_1".to_string()),
            icon: None,
            sort: None,
            ext: None,
            rbum_parent_cate_id: Some(l1_id.to_string()),
            scope_level: Some(RbumScopeLevelKind::L2),
            rel_rbum_set_id: set_id.to_string(),
        },
        &funs,
        context,
    )
    .await?;

    let l2_1_id = RbumSetCateServ::add_rbum(
        &mut RbumSetCateAddReq {
            bus_code: TrimString("".to_string()),
            name: TrimString("l2_1".to_string()),
            icon: None,
            sort: None,
            ext: None,
            rbum_parent_cate_id: Some(l2_id.to_string()),
            scope_level: Some(RbumScopeLevelKind::L2),
            rel_rbum_set_id: set_id.to_string(),
        },
        &funs,
        context,
    )
    .await?;
    RbumSetItemServ::add_rbum(
        &mut RbumSetItemAddReq {
            sort: 0,
            rel_rbum_set_id: set_id.to_string(),
            rel_rbum_set_cate_id: l2_1_id.clone(),
            rel_rbum_item_id: item_app_id.clone(),
        },
        &funs,
        context,
    )
    .await?;

    let l2_1_1_id = RbumSetCateServ::add_rbum(
        &mut RbumSetCateAddReq {
            bus_code: TrimString("".to_string()),
            name: TrimString("l2_1_1".to_string()),
            icon: None,
            sort: None,
            ext: None,
            rbum_parent_cate_id: Some(l2_1_id.to_string()),
            scope_level: Some(RbumScopeLevelKind::L2),
            rel_rbum_set_id: set_id.to_string(),
        },
        &funs,
        context,
    )
    .await?;
    RbumSetItemServ::add_rbum(
        &mut RbumSetItemAddReq {
            sort: 0,
            rel_rbum_set_id: set_id.to_string(),
            rel_rbum_set_cate_id: l2_1_1_id.clone(),
            rel_rbum_item_id: context.owner.clone(),
        },
        &funs,
        context,
    )
    .await?;

    let l2_1_2_id = RbumSetCateServ::add_rbum(
        &mut RbumSetCateAddReq {
            bus_code: TrimString("".to_string()),
            name: TrimString("l2_1_2".to_string()),
            icon: None,
            sort: None,
            ext: None,
            rbum_parent_cate_id: Some(l2_1_id.to_string()),
            scope_level: Some(RbumScopeLevelKind::L2),
            rel_rbum_set_id: set_id.to_string(),
        },
        &funs,
        context,
    )
    .await?;

    info!("【test_rbum_set_cate】 : Test Get : RbumSetCateServ::get_rbum");
    let rbum = RbumSetCateServ::get_rbum(&l2_1_2_id, &RbumSetCateFilterReq::default(), &funs, context).await?;
    assert_eq!(rbum.id, l2_1_2_id);
    assert_eq!(rbum.name, "l2_1_2");

    info!("【test_rbum_set_cate】 : Test Modify : RbumSetCateServ::modify_rbum");
    RbumSetCateServ::modify_rbum(
        &l2_1_2_id,
        &mut RbumSetCateModifyReq {
            bus_code: Some(TrimString("dddddd".to_string())),
            name: None,
            icon: None,
            sort: None,
            ext: None,
            rbum_parent_cate_id: None,
            scope_level: None,
        },
        &funs,
        context,
    )
    .await?;

    info!("【test_rbum_set_cate】 : Test Find : RbumSetCateServ::paginate_rbums");
    let rbums = RbumSetCateServ::paginate_rbums(
        &RbumSetCateFilterReq {
            basic: RbumBasicFilterReq {
                name: Some("l2_1_2".to_string()),
                ..Default::default()
            },
            ..Default::default()
        },
        1,
        10,
        None,
        None,
        &funs,
        context,
    )
    .await?;
    assert_eq!(rbums.page_number, 1);
    assert_eq!(rbums.page_size, 10);
    assert_eq!(rbums.total_size, 1);
    assert_eq!(rbums.records[0].name, "l2_1_2");
    assert_eq!(rbums.records[0].bus_code, "dddddd");

    info!("【test_rbum_set_cate】 : Test Find By Set : RbumSetCateServ::get_tree_all");
    let rbums = RbumSetServ::get_tree(&set_id, &RbumSetTreeFilterReq::default(), &funs, context).await?;
    assert_eq!(rbums.main.len(), 7);
    assert_eq!(rbums.main[0].id, l1_id);
    assert_eq!(rbums.main[0].pid, None);
    assert_eq!(rbums.main[1].id, l1_1_id);
    assert_eq!(rbums.main[1].pid, Some(l1_id.clone()));
    assert_eq!(rbums.main[2].id, l2_id);
    assert_eq!(rbums.main[2].pid, None);
    assert_eq!(rbums.main[3].id, l2_1_id);
    assert_eq!(rbums.main[3].pid, Some(l2_id.clone()));
    assert_eq!(rbums.main[4].id, l2_1_1_id);
    assert_eq!(rbums.main[4].pid, Some(l2_1_id.clone()));
    assert_eq!(rbums.main[5].id, l2_1_2_id);
    assert_eq!(rbums.main[5].pid, Some(l2_1_id.clone()));
    assert_eq!(rbums.main[6].id, l3_id);
    assert_eq!(rbums.main[6].pid, None);
    assert!(rbums.ext.is_none());

    info!("【test_rbum_set_cate】 : Test Find By Set : RbumSetCateServ::get_tree_all, fetch_cate_item=true");
    let rbums = RbumSetServ::get_tree(
        &set_id,
        &RbumSetTreeFilterReq {
            fetch_cate_item: true,
            ..Default::default()
        },
        &funs,
        context,
    )
    .await?;
    assert_eq!(rbums.main.len(), 7);
    assert_eq!(rbums.main[0].id, l1_id);
    assert_eq!(rbums.main[0].pid, None);
    assert_eq!(rbums.main[1].id, l1_1_id);
    assert_eq!(rbums.main[1].pid, Some(l1_id.clone()));
    assert_eq!(rbums.main[2].id, l2_id);
    assert_eq!(rbums.main[2].pid, None);
    assert_eq!(rbums.main[3].id, l2_1_id);
    assert_eq!(rbums.main[3].pid, Some(l2_id.clone()));
    assert_eq!(rbums.main[4].id, l2_1_1_id);
    assert_eq!(rbums.main[4].pid, Some(l2_1_id.clone()));
    assert_eq!(rbums.main[5].id, l2_1_2_id);
    assert_eq!(rbums.main[5].pid, Some(l2_1_id.clone()));
    assert_eq!(rbums.main[6].id, l3_id);
    assert_eq!(rbums.main[6].pid, None);
    let ext = rbums.ext.unwrap();
    assert!(ext.items[&rbums.main[0].id].is_empty());
    assert!(ext.items[&rbums.main[1].id].is_empty());
    assert!(ext.items[&rbums.main[2].id].is_empty());
    assert_eq!(ext.items[&rbums.main[3].id].len(), 1);
    assert_eq!(ext.items[&rbums.main[3].id][0].rel_rbum_item_kind_id, kind_app_id);
    assert_eq!(ext.items[&rbums.main[3].id][0].rel_rbum_item_domain_id, domain_iam_id);
    assert_eq!(ext.items[&rbums.main[3].id][0].rel_rbum_item_name, "应用1");
    assert_eq!(ext.items[&rbums.main[4].id].len(), 1);
    assert_eq!(ext.items[&rbums.main[4].id][0].rel_rbum_item_id, context.owner);
    assert!(ext.items[&rbums.main[5].id].is_empty());
    assert_eq!(ext.item_kinds.len(), 2);
    assert_eq!(ext.item_kinds[&kind_app_id].name, "APP");
    let kind_account_id = ext.item_kinds.keys().find(|k| *k != &kind_app_id).unwrap();
    assert_eq!(ext.item_domains.len(), 2);
    assert_eq!(ext.item_domains[&domain_iam_id].name, "IAM2");
    assert_eq!(ext.item_number_agg[""].len(), 2);
    assert_eq!(ext.item_number_agg[""][&kind_app_id], 1);
    assert_eq!(ext.item_number_agg[""][kind_account_id], 1);
    assert_eq!(ext.item_number_agg[&rbums.main[2].id].len(), 2);
    assert_eq!(ext.item_number_agg[&rbums.main[2].id][&kind_app_id], 1);
    assert_eq!(ext.item_number_agg[&rbums.main[2].id][kind_account_id], 1);
    assert_eq!(ext.item_number_agg[&rbums.main[3].id].len(), 2);
    assert_eq!(ext.item_number_agg[&rbums.main[3].id][&kind_app_id], 1);
    assert_eq!(ext.item_number_agg[&rbums.main[4].id].len(), 1);
    assert_eq!(ext.item_number_agg[&rbums.main[4].id][kind_account_id], 1);

    info!("【test_rbum_set_cate】 : Test Find By Set : RbumSetCateServ::get_tree_all,fetch_cate_item=true,filter_cate_sys_codes=0000,0001");
    let rbums = RbumSetServ::get_tree(
        &set_id,
        &RbumSetTreeFilterReq {
            fetch_cate_item: true,
            sys_codes: Some(vec!["0000".to_string(), "0001".to_string()]),
            ..Default::default()
        },
        &funs,
        context,
    )
    .await?;
    assert_eq!(rbums.main.len(), 2);
    assert_eq!(rbums.main[0].id, l1_id);
    assert_eq!(rbums.main[1].id, l2_id);

    info!("【test_rbum_set_cate】 : Test Find By Set : RbumSetCateServ::get_tree_all,fetch_cate_item=true,filter_cate_sys_codes=0001,filter_sys_code_query_kind=CurrentAndSub");
    let rbums = RbumSetServ::get_tree(
        &set_id,
        &RbumSetTreeFilterReq {
            fetch_cate_item: true,
            sys_codes: Some(vec!["0001".to_string()]),
            sys_code_query_kind: Some(RbumSetCateLevelQueryKind::CurrentAndSub),
            ..Default::default()
        },
        &funs,
        context,
    )
    .await?;
    assert_eq!(rbums.main.len(), 4);
    assert_eq!(rbums.main[0].id, l2_id);
    assert_eq!(rbums.main[0].pid, None);
    assert_eq!(rbums.main[1].id, l2_1_id);
    assert_eq!(rbums.ext.as_ref().unwrap().items[&rbums.main[1].id].len(), 1);
    assert_eq!(rbums.ext.as_ref().unwrap().items[&rbums.main[1].id][0].rel_rbum_item_kind_id, kind_app_id);
    assert_eq!(rbums.ext.as_ref().unwrap().items[&rbums.main[1].id][0].rel_rbum_item_domain_id, domain_iam_id);
    assert_eq!(rbums.ext.as_ref().unwrap().items[&rbums.main[1].id][0].rel_rbum_item_name, "应用1");
    assert_eq!(rbums.main[1].pid, Some(l2_id.clone()));
    assert_eq!(rbums.main[2].id, l2_1_1_id);
    assert_eq!(rbums.ext.as_ref().unwrap().items[&rbums.main[2].id][0].rel_rbum_item_id, context.owner);
    assert_eq!(rbums.main[2].pid, Some(l2_1_id.clone()));
    assert_eq!(rbums.main[3].id, l2_1_2_id);
    assert_eq!(rbums.main[3].pid, Some(l2_1_id.clone()));

    info!("【test_rbum_set_cate】 : Test Find By Set : RbumSetCateServ::get_tree_all,fetch_cate_item=true,filter_cate_item_ids=some1");
    let rbums = RbumSetServ::get_tree(
        &set_id,
        &RbumSetTreeFilterReq {
            fetch_cate_item: true,
            rel_rbum_item_ids: Some(vec![item_app_id.clone()]),
            ..Default::default()
        },
        &funs,
        context,
    )
    .await?;
    assert_eq!(rbums.main.len(), 7);
    assert_eq!(rbums.main[0].id, l1_id);
    assert_eq!(rbums.main[0].pid, None);
    assert_eq!(rbums.main[1].id, l1_1_id);
    assert_eq!(rbums.main[1].pid, Some(l1_id.clone()));
    assert_eq!(rbums.main[2].id, l2_id);
    assert_eq!(rbums.main[2].pid, None);
    assert_eq!(rbums.main[3].id, l2_1_id);
    assert_eq!(rbums.ext.as_ref().unwrap().items[&rbums.main[3].id].len(), 1);
    assert_eq!(rbums.ext.as_ref().unwrap().items[&rbums.main[3].id][0].rel_rbum_item_kind_id, kind_app_id);
    assert_eq!(rbums.ext.as_ref().unwrap().items[&rbums.main[3].id][0].rel_rbum_item_domain_id, domain_iam_id);
    assert_eq!(rbums.ext.as_ref().unwrap().items[&rbums.main[3].id][0].rel_rbum_item_name, "应用1");
    assert_eq!(rbums.main[3].pid, Some(l2_id.clone()));
    assert_eq!(rbums.main[4].id, l2_1_1_id);
    assert_eq!(rbums.ext.as_ref().unwrap().items[&rbums.main[4].id].len(), 0);
    assert_eq!(rbums.main[4].pid, Some(l2_1_id.clone()));
    assert_eq!(rbums.main[5].id, l2_1_2_id);
    assert_eq!(rbums.main[5].pid, Some(l2_1_id.clone()));
    assert_eq!(rbums.main[6].id, l3_id);
    assert_eq!(rbums.main[6].pid, None);

    info!("【test_rbum_set_cate】 : Test Find By Set : RbumSetCateServ::get_tree_all,fetch_cate_item=true,filter_cate_item_ids=some2");
    let rbums = RbumSetServ::get_tree(
        &set_id,
        &RbumSetTreeFilterReq {
            fetch_cate_item: true,
            rel_rbum_item_ids: Some(vec![item_app_id.clone(), context.owner.clone()]),
            ..Default::default()
        },
        &funs,
        context,
    )
    .await?;
    assert_eq!(rbums.main.len(), 7);
    assert_eq!(rbums.main[0].id, l1_id);
    assert_eq!(rbums.main[0].pid, None);
    assert_eq!(rbums.main[1].id, l1_1_id);
    assert_eq!(rbums.main[1].pid, Some(l1_id.clone()));
    assert_eq!(rbums.main[2].id, l2_id);
    assert_eq!(rbums.main[2].pid, None);
    assert_eq!(rbums.main[3].id, l2_1_id);
    assert_eq!(rbums.ext.as_ref().unwrap().items[&rbums.main[3].id].len(), 1);
    assert_eq!(rbums.ext.as_ref().unwrap().items[&rbums.main[3].id][0].rel_rbum_item_kind_id, kind_app_id);
    assert_eq!(rbums.ext.as_ref().unwrap().items[&rbums.main[3].id][0].rel_rbum_item_domain_id, domain_iam_id);
    assert_eq!(rbums.ext.as_ref().unwrap().items[&rbums.main[3].id][0].rel_rbum_item_name, "应用1");
    assert_eq!(rbums.main[3].pid, Some(l2_id.clone()));
    assert_eq!(rbums.main[4].id, l2_1_1_id);
    assert_eq!(rbums.ext.as_ref().unwrap().items[&rbums.main[4].id][0].rel_rbum_item_id, context.owner);
    assert_eq!(rbums.main[4].pid, Some(l2_1_id.clone()));
    assert_eq!(rbums.main[5].id, l2_1_2_id);
    assert_eq!(rbums.main[5].pid, Some(l2_1_id.clone()));
    assert_eq!(rbums.main[6].id, l3_id);
    assert_eq!(rbums.main[6].pid, None);

    info!("【test_rbum_set_cate】 : Test Find By Set : RbumSetCateServ::get_tree_all,fetch_cate_item=true,filter_cate_item_ids=some2, hide");
    let rbums = RbumSetServ::get_tree(
        &set_id,
        &RbumSetTreeFilterReq {
            fetch_cate_item: true,
            hide_cate_with_empty_item: true,
            rel_rbum_item_ids: Some(vec![item_app_id.clone(), context.owner.clone()]),
            ..Default::default()
        },
        &funs,
        context,
    )
    .await?;
    assert_eq!(rbums.main.len(), 3);
    assert_eq!(rbums.main[0].id, l2_id);
    assert_eq!(rbums.main[1].id, l2_1_id);
    assert_eq!(rbums.ext.as_ref().unwrap().items[&rbums.main[1].id].len(), 1);
    assert_eq!(rbums.ext.as_ref().unwrap().items[&rbums.main[1].id][0].rel_rbum_item_kind_id, kind_app_id);
    assert_eq!(rbums.ext.as_ref().unwrap().items[&rbums.main[1].id][0].rel_rbum_item_domain_id, domain_iam_id);
    assert_eq!(rbums.ext.as_ref().unwrap().items[&rbums.main[1].id][0].rel_rbum_item_name, "应用1");
    assert_eq!(rbums.main[2].id, l2_1_1_id);
    assert_eq!(rbums.ext.as_ref().unwrap().items[&rbums.main[2].id][0].rel_rbum_item_id, context.owner);

    info!("【test_rbum_set_cate】 : Test Find By Set : RbumSetCateServ::get_tree_all,fetch_cate_item=true,filter_cate_item_ids=some2, filter_cate_item_kind_ids=some, hide");
    let rbums = RbumSetServ::get_tree(
        &set_id,
        &RbumSetTreeFilterReq {
            fetch_cate_item: true,
            hide_cate_with_empty_item: true,
            rel_rbum_item_kind_ids: Some(vec![kind_app_id.clone()]),
            rel_rbum_item_ids: Some(vec![item_app_id.clone(), context.owner.clone()]),
            ..Default::default()
        },
        &funs,
        context,
    )
    .await?;
    assert_eq!(rbums.main.len(), 2);
    assert_eq!(rbums.main[0].id, l2_id);
    assert_eq!(rbums.main[1].id, l2_1_id);
    assert_eq!(rbums.ext.as_ref().unwrap().items[&rbums.main[1].id].len(), 1);
    assert_eq!(rbums.ext.as_ref().unwrap().items[&rbums.main[1].id][0].rel_rbum_item_kind_id, kind_app_id);
    assert_eq!(rbums.ext.as_ref().unwrap().items[&rbums.main[1].id][0].rel_rbum_item_domain_id, domain_iam_id);
    assert_eq!(rbums.ext.as_ref().unwrap().items[&rbums.main[1].id][0].rel_rbum_item_name, "应用1");

    info!(
        "【test_rbum_set_cate】 : Test Find By Set : RbumSetCateServ::get_tree_all,fetch_cate_item=true,filter_cate_item_ids=some2, filter_cate_item_kind_ids=some, \
    filter_cate_item_domain_ids=some, hide"
    );
    let rbums = RbumSetServ::get_tree(
        &set_id,
        &RbumSetTreeFilterReq {
            fetch_cate_item: true,
            hide_cate_with_empty_item: true,
            rel_rbum_item_kind_ids: Some(vec![kind_app_id.clone()]),
            rel_rbum_item_domain_ids: Some(vec![domain_iam_id.clone()]),
            rel_rbum_item_ids: Some(vec![item_app_id.clone(), context.owner.clone()]),
            ..Default::default()
        },
        &funs,
        context,
    )
    .await?;
    assert_eq!(rbums.main.len(), 2);
    assert_eq!(rbums.main[0].id, l2_id);
    assert_eq!(rbums.main[1].id, l2_1_id);
    assert_eq!(rbums.ext.as_ref().unwrap().items[&rbums.main[1].id].len(), 1);
    assert_eq!(rbums.ext.as_ref().unwrap().items[&rbums.main[1].id][0].rel_rbum_item_kind_id, kind_app_id);
    assert_eq!(rbums.ext.as_ref().unwrap().items[&rbums.main[1].id][0].rel_rbum_item_domain_id, domain_iam_id);
    assert_eq!(rbums.ext.as_ref().unwrap().items[&rbums.main[1].id][0].rel_rbum_item_name, "应用1");

    info!(
        "【test_rbum_set_cate】 : Test Find By Set : RbumSetCateServ::get_tree_all,fetch_cate_item=true,filter_cate_item_ids=some3, filter_cate_item_kind_ids=some, \
    filter_cate_item_domain_ids=some, hide"
    );
    let rbums = RbumSetServ::get_tree(
        &set_id,
        &RbumSetTreeFilterReq {
            fetch_cate_item: true,
            hide_cate_with_empty_item: true,
            rel_rbum_item_kind_ids: Some(vec![kind_app_id.clone()]),
            rel_rbum_item_domain_ids: Some(vec![domain_iam_id.clone()]),
            rel_rbum_item_ids: Some(vec![context.owner.clone()]),
            ..Default::default()
        },
        &funs,
        context,
    )
    .await?;
    assert!(rbums.main.is_empty());

    info!("【test_rbum_set_cate】 : Test Find By Set : RbumSetCateServ::get_tree by_level");
    let rbums = RbumSetServ::get_tree(
        &set_id,
        &RbumSetTreeFilterReq {
            fetch_cate_item: true,
            sys_codes: Some(vec![]),
            ..Default::default()
        },
        &funs,
        context,
    )
    .await?;
    assert_eq!(rbums.main.len(), 3);
    assert_eq!(rbums.main[0].id, l1_id);
    assert_eq!(rbums.main[1].id, l2_id);
    assert_eq!(rbums.main[2].id, l3_id);
    let l1_sys_code = rbums.main[0].sys_code.clone();
    let l2_sys_code = rbums.main[1].sys_code.clone();
    let l3_sys_code = rbums.main[2].sys_code.clone();
    let rbums = RbumSetServ::get_tree(
        &set_id,
        &RbumSetTreeFilterReq {
            fetch_cate_item: true,
            sys_codes: Some(vec![l1_sys_code.clone()]),
            sys_code_query_kind: Some(RbumSetCateLevelQueryKind::Sub),
            sys_code_query_depth: Some(1),
            ..Default::default()
        },
        &funs,
        context,
    )
    .await?;
    assert_eq!(rbums.main.len(), 1);
    assert_eq!(rbums.main[0].id, l1_1_id);
    let rbums = RbumSetServ::get_tree(
        &set_id,
        &RbumSetTreeFilterReq {
            fetch_cate_item: true,
            sys_codes: Some(vec![l1_sys_code.clone(), l2_sys_code.clone()]),
            sys_code_query_kind: Some(RbumSetCateLevelQueryKind::Sub),
            sys_code_query_depth: Some(1),
            ..Default::default()
        },
        &funs,
        context,
    )
    .await?;
    assert_eq!(rbums.main.len(), 2);
    assert_eq!(rbums.main[0].id, l1_1_id);
    assert_eq!(rbums.main[1].id, l2_1_id);
    let rbums = RbumSetServ::get_tree(
        &set_id,
        &RbumSetTreeFilterReq {
            fetch_cate_item: true,
            sys_codes: Some(vec![l3_sys_code.clone()]),
            sys_code_query_kind: Some(RbumSetCateLevelQueryKind::Sub),
            sys_code_query_depth: Some(1),
            ..Default::default()
        },
        &funs,
        context,
    )
    .await?;
    assert_eq!(rbums.main.len(), 0);
    let rbums = RbumSetServ::get_tree(
        &set_id,
        &RbumSetTreeFilterReq {
            fetch_cate_item: true,
            sys_codes: Some(vec![l2_sys_code.clone()]),
            sys_code_query_kind: Some(RbumSetCateLevelQueryKind::Sub),
            sys_code_query_depth: Some(1),
            ..Default::default()
        },
        &funs,
        context,
    )
    .await?;
    assert_eq!(rbums.main.len(), 1);
    assert_eq!(rbums.main[0].id, l2_1_id);
    let l2_1_sys_code = rbums.main[0].sys_code.clone();
    let rbums = RbumSetServ::get_tree(
        &set_id,
        &RbumSetTreeFilterReq {
            fetch_cate_item: true,
            sys_codes: Some(vec![l2_1_sys_code.clone()]),
            sys_code_query_kind: Some(RbumSetCateLevelQueryKind::Sub),
            sys_code_query_depth: Some(1),
            ..Default::default()
        },
        &funs,
        context,
    )
    .await?;
    assert_eq!(rbums.main.len(), 2);
    assert_eq!(rbums.main[0].id, l2_1_1_id);
    assert_eq!(rbums.main[1].id, l2_1_2_id);

    info!("【test_rbum_set_cate】 : Test Delete : RbumSetCateServ::delete_rbum");
    assert!(RbumSetCateServ::delete_rbum(&l2_1_id, &funs, context).await.is_err());
    RbumSetCateServ::delete_rbum(&l2_1_2_id, &funs, context).await?;
    assert!(RbumSetCateServ::get_rbum(&l2_1_2_id, &RbumSetCateFilterReq::default(), &funs, context).await.is_err());

    funs.rollback().await?;

    Ok(())
}

async fn test_rbum_set_item(context: &TardisContext) -> TardisResult<()> {
    let mut funs = TardisFuns::inst_with_db_conn("".to_string(), None);
    funs.begin().await?;

    info!("【test_rbum_set_item】 : Prepare : RbumKindServ::add_rbum");
    let kind_account_id = RbumKindServ::add_rbum(
        &mut RbumKindAddReq {
            code: TrimString("account".to_string()),
            name: TrimString("Account".to_string()),
            module: None,
            note: None,
            icon: None,
            sort: None,
            ext_table_name: None,
            scope_level: Some(RbumScopeLevelKind::L2),
        },
        &funs,
        context,
    )
    .await?;

    info!("【test_rbum_set_item】 : Prepare Domain : RbumDomainServ::add_rbum");
    let domain_iam_id = RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            code: TrimString("iam2".to_string()),
            name: TrimString("IAM2".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: Some(RbumScopeLevelKind::L2),
        },
        &funs,
        context,
    )
    .await?;

    info!("【test_rbum_set_item】 : Prepare Item : RbumItemServ::add_rbum");
    let item_account_a1_id = RbumItemServ::add_rbum(
        &mut RbumItemAddReq {
            id: None,
            code: None,
            name: TrimString("用户1".to_string()),
            scope_level: Some(RbumScopeLevelKind::L2),
            disabled: None,
            rel_rbum_kind_id: kind_account_id.to_string(),
            rel_rbum_domain_id: domain_iam_id.to_string(),
        },
        &funs,
        context,
    )
    .await?;

    let item_account_a2_id = RbumItemServ::add_rbum(
        &mut RbumItemAddReq {
            id: None,
            code: None,
            name: TrimString("用户2".to_string()),
            scope_level: Some(RbumScopeLevelKind::L2),
            disabled: None,
            rel_rbum_kind_id: kind_account_id.to_string(),
            rel_rbum_domain_id: domain_iam_id.to_string(),
        },
        &funs,
        context,
    )
    .await?;

    let item_account_a3_id = RbumItemServ::add_rbum(
        &mut RbumItemAddReq {
            id: None,
            code: None,
            name: TrimString("用户3".to_string()),
            scope_level: Some(RbumScopeLevelKind::L2),
            disabled: None,
            rel_rbum_kind_id: kind_account_id.to_string(),
            rel_rbum_domain_id: domain_iam_id.to_string(),
        },
        &funs,
        context,
    )
    .await?;

    info!("【test_rbum_set_item】 : Prepare Set : RbumSetServ::add_rbum");
    let set_id = RbumSetServ::add_rbum(
        &mut RbumSetAddReq {
            code: TrimString("set_test".to_string()),
            kind: TrimString("".to_string()),
            name: TrimString(" 测试集合 ".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: Some(RbumScopeLevelKind::L2),
            ext: None,
            disabled: None,
        },
        &funs,
        context,
    )
    .await?;

    info!("【test_rbum_set_item】 : Prepare Set Cate : RbumSetCateServ::add_rbum");
    let set_cate_l1_id = RbumSetCateServ::add_rbum(
        &mut RbumSetCateAddReq {
            bus_code: TrimString("".to_string()),
            name: TrimString("l1".to_string()),
            icon: None,
            sort: None,
            ext: None,
            rbum_parent_cate_id: None,
            scope_level: Some(RbumScopeLevelKind::L2),
            rel_rbum_set_id: set_id.to_string(),
        },
        &funs,
        context,
    )
    .await?;

    let set_cate_l1_1_id = RbumSetCateServ::add_rbum(
        &mut RbumSetCateAddReq {
            bus_code: TrimString("".to_string()),
            name: TrimString("l1-1".to_string()),
            icon: None,
            sort: None,
            ext: None,
            rbum_parent_cate_id: Some(set_cate_l1_id.to_string()),
            scope_level: Some(RbumScopeLevelKind::L2),
            rel_rbum_set_id: set_id.to_string(),
        },
        &funs,
        context,
    )
    .await?;

    let set_cate_l2_id = RbumSetCateServ::add_rbum(
        &mut RbumSetCateAddReq {
            bus_code: TrimString("".to_string()),
            name: TrimString("l2".to_string()),
            icon: None,
            sort: None,
            ext: None,
            rbum_parent_cate_id: None,
            scope_level: Some(RbumScopeLevelKind::L2),
            rel_rbum_set_id: set_id.to_string(),
        },
        &funs,
        context,
    )
    .await?;

    let _set_cate_l3_id = RbumSetCateServ::add_rbum(
        &mut RbumSetCateAddReq {
            bus_code: TrimString("".to_string()),
            name: TrimString("l3".to_string()),
            icon: None,
            sort: None,
            ext: None,
            rbum_parent_cate_id: None,
            scope_level: Some(RbumScopeLevelKind::L2),
            rel_rbum_set_id: set_id.to_string(),
        },
        &funs,
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
            rel_rbum_item_id: "".to_string(),
        },
        &funs,
        context,
    )
    .await
    .is_err());

    assert!(RbumSetItemServ::add_rbum(
        &mut RbumSetItemAddReq {
            sort: 0,
            rel_rbum_set_id: set_id.to_string(),
            rel_rbum_set_cate_id: "".to_string(),
            rel_rbum_item_id: "".to_string(),
        },
        &funs,
        context,
    )
    .await
    .is_err());

    assert!(RbumSetItemServ::add_rbum(
        &mut RbumSetItemAddReq {
            sort: 0,
            rel_rbum_set_id: set_id.to_string(),
            rel_rbum_set_cate_id: "".to_string(),
            rel_rbum_item_id: item_account_a1_id.to_string(),
        },
        &funs,
        context,
    )
    .await
    .is_ok());

    RbumSetItemServ::add_rbum(
        &mut RbumSetItemAddReq {
            sort: 0,
            rel_rbum_set_id: set_id.to_string(),
            rel_rbum_set_cate_id: set_cate_l1_id.to_string(),
            rel_rbum_item_id: context.owner.to_string(),
        },
        &funs,
        context,
    )
    .await?;

    RbumSetItemServ::add_rbum(
        &mut RbumSetItemAddReq {
            sort: 0,
            rel_rbum_set_id: set_id.to_string(),
            rel_rbum_set_cate_id: set_cate_l2_id.to_string(),
            rel_rbum_item_id: item_account_a3_id.to_string(),
        },
        &funs,
        context,
    )
    .await?;

    RbumSetItemServ::add_rbum(
        &mut RbumSetItemAddReq {
            sort: 0,
            rel_rbum_set_id: set_id.to_string(),
            rel_rbum_set_cate_id: set_cate_l1_1_id.to_string(),
            rel_rbum_item_id: item_account_a2_id.to_string(),
        },
        &funs,
        context,
    )
    .await?;

    let id = RbumSetItemServ::add_rbum(
        &mut RbumSetItemAddReq {
            sort: 0,
            rel_rbum_set_id: set_id.to_string(),
            rel_rbum_set_cate_id: set_cate_l1_1_id.to_string(),
            rel_rbum_item_id: item_account_a1_id.to_string(),
        },
        &funs,
        context,
    )
    .await?;

    assert!(RbumSetItemServ::add_rbum(
        &mut RbumSetItemAddReq {
            sort: 0,
            rel_rbum_set_id: set_id.to_string(),
            rel_rbum_set_cate_id: set_cate_l1_1_id.to_string(),
            rel_rbum_item_id: item_account_a1_id.to_string(),
        },
        &funs,
        context,
    )
    .await
    .err()
    .unwrap()
    .message
    .contains("item already exists"));

    // item_account_a1_id
    // >set_cate_l1_id
    //      context.owner
    //      >set_cate_l1_1_id
    //          item_account_a2_id
    //          item_account_a1_id
    // >set_cate_l2_id
    //          item_account_a3_id
    // >set_cate_l3_id

    info!("【test_rbum_set_item】 : Test Get : RbumSetItemServ::check_a_is_parent_of_b");
    assert!(RbumSetItemServ::check_a_is_parent_of_b(&context.owner, &item_account_a1_id, &set_id, &funs, context,).await?);
    assert!(!RbumSetItemServ::check_a_is_parent_of_b("xxxx", &item_account_a1_id, &set_id, &funs, context,).await?);
    assert!(!RbumSetItemServ::check_a_is_parent_of_b(&item_account_a1_id, &item_account_a2_id, &set_id, &funs, context,).await?);

    info!("【test_rbum_set_item】 : Test Get : RbumSetItemServ::check_a_is_sibling_of_b");
    assert!(!RbumSetItemServ::check_a_is_sibling_of_b(&context.owner, &item_account_a1_id, &set_id, &funs, context,).await?);
    assert!(RbumSetItemServ::check_a_is_sibling_of_b(&item_account_a1_id, &item_account_a2_id, &set_id, &funs, context,).await?);
    assert!(!RbumSetItemServ::check_a_is_sibling_of_b(&item_account_a1_id, &item_account_a3_id, &set_id, &funs, context,).await?);

    info!("【test_rbum_set_item】 : Test Get : RbumSetItemServ::check_a_is_parent_or_sibling_of_b");
    assert!(RbumSetItemServ::check_a_is_parent_or_sibling_of_b(&context.owner, &item_account_a1_id, &set_id, &funs, context,).await?);
    assert!(RbumSetItemServ::check_a_is_parent_or_sibling_of_b(&item_account_a1_id, &item_account_a2_id, &set_id, &funs, context,).await?);

    info!("【test_rbum_set_item】 : Test Find Set Paths : RbumSetItemServ::get_rbum");
    let set_paths = RbumSetItemServ::find_set_paths(&item_account_a1_id, &set_id, &funs, context).await?;
    assert_eq!(set_paths.len(), 2);
    assert_eq!(set_paths[0].len(), 0);
    assert_eq!(set_paths[1].len(), 2);
    assert!(set_paths[1].iter().any(|i| i.name == "l1-1"));
    assert!(set_paths[1].iter().any(|i| i.name == "l1"));

    info!("【test_rbum_set_item】 : Test Get : RbumSetServ::get_tree_all");
    let set_infos = RbumSetServ::get_tree(
        &set_id,
        &RbumSetTreeFilterReq {
            fetch_cate_item: true,
            ..Default::default()
        },
        &funs,
        context,
    )
    .await?;
    assert_eq!(set_infos.main.len(), 4);
    assert_eq!(set_infos.ext.as_ref().unwrap().items[&set_infos.main[1].id].len(), 2);
    assert!(set_infos.ext.as_ref().unwrap().items[&set_infos.main[1].id].iter().any(|r| r.rel_rbum_item_name == "用户1"));

    info!("【test_rbum_set_item】 : Test Get : RbumSetServ::get_tree_all hide_cate_with_empty_item");
    let set_infos = RbumSetServ::get_tree(
        &set_id,
        &RbumSetTreeFilterReq {
            fetch_cate_item: true,
            hide_cate_with_empty_item: true,
            ..Default::default()
        },
        &funs,
        context,
    )
    .await?;
    assert_eq!(set_infos.main.len(), 3);
    assert_eq!(set_infos.ext.as_ref().unwrap().items[&set_infos.main[1].id].len(), 2);
    assert!(set_infos.ext.as_ref().unwrap().items[&set_infos.main[1].id].iter().any(|r| r.rel_rbum_item_name == "用户1"));

    let set_info_tree = set_infos.to_trees();
    assert_eq!(set_info_tree.cate_tree.len(), 2);
    assert_eq!(set_info_tree.cate_tree[0].node.len(), 1);
    assert_eq!(set_info_tree.ext.as_ref().unwrap().items[&set_info_tree.cate_tree[0].node[0].id].len(), 2);
    assert!(set_info_tree.ext.as_ref().unwrap().items[&set_info_tree.cate_tree[0].node[0].id].iter().any(|r| r.rel_rbum_item_name == "用户1"));
    let rbum = RbumSetItemServ::get_rbum(
        &id,
        &RbumSetItemFilterReq {
            basic: Default::default(),
            rel_rbum_set_id: Some(set_id.to_string()),
            ..Default::default()
        },
        &funs,
        context,
    )
    .await?;
    assert_eq!(rbum.id, id);
    assert_eq!(rbum.sort, 0);
    assert_eq!(rbum.rel_rbum_set_cate_name.unwrap_or_default(), "l1-1");
    assert_eq!(rbum.rel_rbum_item_name, "用户1");

    info!("【test_rbum_set_item】 : Test Modify : RbumSetItemServ::modify_rbum");
    RbumSetItemServ::modify_rbum(
        &id,
        &mut RbumSetItemModifyReq {
            sort: Some(10),
            rel_rbum_set_cate_id: None,
        },
        &funs,
        context,
    )
    .await?;

    info!("【test_rbum_set_item】 : Test Find : RbumSetItemServ::paginate_rbums");
    let rbums = RbumSetItemServ::paginate_rbums(
        &RbumSetItemFilterReq {
            basic: Default::default(),
            rel_rbum_set_id: Some(set_id.to_string()),
            ..Default::default()
        },
        1,
        10,
        None,
        None,
        &funs,
        context,
    )
    .await?;
    assert_eq!(rbums.page_number, 1);
    assert_eq!(rbums.page_size, 10);
    assert_eq!(rbums.total_size, 4);
    assert!(rbums.records.iter().any(|r| r.sort == 10));

    info!("【test_rbum_set_item】 : Test Delete : RbumSetItemServ::delete_rbum");
    RbumSetItemServ::delete_rbum(&id, &funs, context).await?;
    assert!(RbumSetItemServ::get_rbum(
        &id,
        &RbumSetItemFilterReq {
            basic: Default::default(),
            rel_rbum_set_id: Some(set_id.to_string()),
            ..Default::default()
        },
        &funs,
        context,
    )
    .await
    .is_err());

    funs.rollback().await?;

    Ok(())
}
