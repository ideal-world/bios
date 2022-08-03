use bios_basic::rbum::dto::rbum_filer_dto::RbumSetTreeFilterReq;
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;
use tardis::TardisFunsInst;

use bios_basic::rbum::dto::rbum_set_item_dto::RbumSetItemModifyReq;
use bios_iam::basic::dto::iam_set_dto::{IamSetCateAddReq, IamSetCateModifyReq, IamSetItemAddReq};
use bios_iam::basic::serv::iam_set_serv::IamSetServ;
use bios_iam::iam_constants;
use bios_iam::iam_constants::{RBUM_SCOPE_LEVEL_GLOBAL, RBUM_SCOPE_LEVEL_PRIVATE, RBUM_SCOPE_LEVEL_TENANT};
use bios_iam::iam_enumeration::IamSetKind;

pub async fn test(
    sys_context: &TardisContext,
    t1_context: &TardisContext,
    t2_context: &TardisContext,
    t2_a1_context: &TardisContext,
    t2_a2_context: &TardisContext,
) -> TardisResult<()> {
    test_single_level(sys_context, t1_context).await?;
    test_single_level(t1_context, t2_context).await?;
    test_single_level(t2_a1_context, t2_a2_context).await?;
    test_multi_level_by_sys_context(sys_context, t1_context, t2_context, t2_a1_context, t2_a2_context).await?;
    test_multi_level_by_tenant_context(sys_context, t1_context, t2_context, t2_a1_context, t2_a2_context).await?;
    test_multi_level_by_app_context(sys_context, t1_context, t2_context, t2_a1_context, t2_a2_context).await?;
    Ok(())
}

async fn test_single_level(context: &TardisContext, another_context: &TardisContext) -> TardisResult<()> {
    let mut funs = iam_constants::get_tardis_inst();
    funs.begin().await?;
    info!("【test_cc_set】 : test_single_level : Add Set Cate");
    let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Org, &funs, context).await?;
    let set_cate_id1 = IamSetServ::add_set_cate(
        &set_id,
        &mut IamSetCateAddReq {
            bus_code: Some(TrimString("bc1".to_string())),
            name: TrimString("xxx分公司".to_string()),
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

    let _set_cate_id3 = IamSetServ::add_set_cate(
        &set_id,
        &mut IamSetCateAddReq {
            bus_code: Some(TrimString("bc2-1".to_string())),
            name: TrimString("yyy分公司zzz部门".to_string()),
            icon: None,
            sort: None,
            ext: None,
            rbum_parent_cate_id: Some(set_cate_id1.clone()),
            scope_level: None,
        },
        &funs,
        context,
    )
    .await?;

    let set_cate_id4 = IamSetServ::add_set_cate(
        &set_id,
        &mut IamSetCateAddReq {
            bus_code: Some(TrimString("bc2-2".to_string())),
            name: TrimString("yyy分公司zzz部门".to_string()),
            icon: None,
            sort: None,
            ext: None,
            rbum_parent_cate_id: Some(set_cate_id1.clone()),
            scope_level: None,
        },
        &funs,
        context,
    )
    .await?;

    info!("【test_cc_set】 : test_single_level : Modify Set Cate By Id");
    assert!(IamSetServ::modify_set_cate(
        &set_cate_id4,
        &IamSetCateModifyReq {
            bus_code: Some(TrimString("bc2-xxxxx".to_string())),
            name: None,
            icon: None,
            sort: None,
            ext: None,
            scope_level: None
        },
        &funs,
        another_context
    )
    .await
    .is_err());

    IamSetServ::modify_set_cate(
        &set_cate_id4,
        &IamSetCateModifyReq {
            bus_code: Some(TrimString("bc2-xxxxx".to_string())),
            name: None,
            icon: None,
            sort: None,
            ext: None,
            scope_level: None,
        },
        &funs,
        context,
    )
    .await?;

    info!("【test_cc_set】 : test_single_level : Find Set Cate");
    let set_cates = IamSetServ::get_tree(
        &set_id,
        None,
        &mut RbumSetTreeFilterReq {
            fetch_cate_item: true,
            ..Default::default()
        },
        &funs,
        context,
    )
    .await?;
    assert_eq!(set_cates.len(), 3);
    assert!(set_cates.iter().any(|i| i.bus_code == "bc2-xxxxx"));

    info!("【test_cc_set】 : test_single_level : Delete Set Cate By Id");
    assert!(IamSetServ::delete_set_cate(&set_cate_id4, &funs, another_context).await.is_err());
    assert!(IamSetServ::delete_set_cate(&set_cate_id1, &funs, context).await.is_err());
    IamSetServ::delete_set_cate(&set_cate_id4, &funs, context).await?;

    info!("【test_ca_set】 : test_single_level : Add Set Item");
    assert!(IamSetServ::add_set_item(
        &IamSetItemAddReq {
            set_id: set_id.clone(),
            set_cate_id: set_cate_id1.to_string(),
            sort: 0,
            rel_rbum_item_id: context.owner.to_string(),
        },
        &funs,
        another_context,
    )
    .await
    .is_err());
    let item_id1 = IamSetServ::add_set_item(
        &IamSetItemAddReq {
            set_id: set_id.clone(),
            set_cate_id: set_cate_id1.to_string(),
            sort: 0,
            rel_rbum_item_id: context.owner.to_string(),
        },
        &funs,
        context,
    )
    .await?;

    assert!(IamSetServ::add_set_item(
        &IamSetItemAddReq {
            set_id: set_id.clone(),
            set_cate_id: set_cate_id1.to_string(),
            sort: 0,
            rel_rbum_item_id: context.owner.to_string(),
        },
        &funs,
        context,
    )
    .await
    .is_err());

    info!("【test_ca_set】 : test_single_level : Modify Set Item By Id");
    assert!(IamSetServ::modify_set_item(&item_id1, &mut RbumSetItemModifyReq { sort: 10 }, &funs, another_context).await.is_err());
    IamSetServ::modify_set_item(&item_id1, &mut RbumSetItemModifyReq { sort: 10 }, &funs, context).await?;

    info!("【test_ca_set】 : test_single_level : Find Set Item");
    let items = IamSetServ::find_set_items(None, Some(set_cate_id1.clone()), None, false, &funs, context).await?;
    assert_eq!(items.len(), 1);

    info!("【test_ca_set】 : test_single_level : Delete Set Item By Id");
    assert!(IamSetServ::delete_set_item(&item_id1, &funs, another_context).await.is_err());
    IamSetServ::delete_set_item(&item_id1, &funs, context).await?;
    let items = IamSetServ::find_set_items(None, Some(set_cate_id1.clone()), None, false, &funs, context).await?;
    assert_eq!(items.len(), 0);

    funs.rollback().await?;
    Ok(())
}

async fn test_multi_level_add<'a>(
    sys_context: &TardisContext,
    t1_context: &TardisContext,
    t2_context: &TardisContext,
    t2_a1_context: &TardisContext,
    t2_a2_context: &TardisContext,
    funs: &TardisFunsInst<'a>,
) -> TardisResult<(String, String, String, String, String, String, String)> {
    info!("【test_cc_set】 : test_multi_level : Add Set Cate");

    let sys_set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Org, &funs, sys_context).await?;

    let set_cate_sys_global_id = IamSetServ::add_set_cate(
        &sys_set_id,
        &mut IamSetCateAddReq {
            bus_code: None,
            name: TrimString("xxx公司".to_string()),
            icon: None,
            sort: None,
            ext: None,
            rbum_parent_cate_id: None,
            scope_level: Some(RBUM_SCOPE_LEVEL_GLOBAL),
        },
        &funs,
        sys_context,
    )
    .await?;

    let set_cate_sys_id = IamSetServ::add_set_cate(
        &sys_set_id,
        &mut IamSetCateAddReq {
            bus_code: None,
            name: TrimString("sys私有部门".to_string()),
            icon: None,
            sort: None,
            ext: None,
            rbum_parent_cate_id: Some(set_cate_sys_global_id.clone()),
            scope_level: None,
        },
        &funs,
        sys_context,
    )
    .await?;

    let set_cate_t1_id = IamSetServ::add_set_cate(
        &sys_set_id,
        &mut IamSetCateAddReq {
            bus_code: None,
            name: TrimString("t1私有部门".to_string()),
            icon: None,
            sort: None,
            ext: None,
            rbum_parent_cate_id: Some(set_cate_sys_global_id.clone()),
            scope_level: None,
        },
        &funs,
        t1_context,
    )
    .await?;

    let set_cate_t2_id = IamSetServ::add_set_cate(
        &sys_set_id,
        &mut IamSetCateAddReq {
            bus_code: None,
            name: TrimString("t2私有部门".to_string()),
            icon: None,
            sort: None,
            ext: None,
            rbum_parent_cate_id: Some(set_cate_sys_global_id.clone()),
            scope_level: None,
        },
        funs,
        t2_context,
    )
    .await?;

    let set_cate_t2_tenant_id = IamSetServ::add_set_cate(
        &sys_set_id,
        &mut IamSetCateAddReq {
            bus_code: None,
            name: TrimString("t2共享部门".to_string()),
            icon: None,
            sort: None,
            ext: None,
            rbum_parent_cate_id: Some(set_cate_sys_global_id.clone()),
            scope_level: Some(RBUM_SCOPE_LEVEL_TENANT),
        },
        funs,
        t2_context,
    )
    .await?;

    let set_cate_t2_a1_id = IamSetServ::add_set_cate(
        &sys_set_id,
        &mut IamSetCateAddReq {
            bus_code: None,
            name: TrimString("t2_a1私有部门".to_string()),
            icon: None,
            sort: None,
            ext: None,
            rbum_parent_cate_id: Some(set_cate_t2_tenant_id.clone()),
            scope_level: None,
        },
        funs,
        t2_a1_context,
    )
    .await?;

    let set_cate_t2_a2_id = IamSetServ::add_set_cate(
        &sys_set_id,
        &mut IamSetCateAddReq {
            bus_code: None,
            name: TrimString("t2_a2私有部门".to_string()),
            icon: None,
            sort: None,
            ext: None,
            rbum_parent_cate_id: Some(set_cate_t2_tenant_id.clone()),
            scope_level: None,
        },
        funs,
        t2_a2_context,
    )
    .await?;

    Ok((
        set_cate_sys_id,
        set_cate_sys_global_id,
        set_cate_t1_id,
        set_cate_t2_id,
        set_cate_t2_tenant_id,
        set_cate_t2_a1_id,
        set_cate_t2_a2_id,
    ))
}

pub async fn test_multi_level_by_sys_context(
    sys_context: &TardisContext,
    t1_context: &TardisContext,
    t2_context: &TardisContext,
    t2_a1_context: &TardisContext,
    t2_a2_context: &TardisContext,
) -> TardisResult<()> {
    let mut funs = iam_constants::get_tardis_inst();
    funs.begin().await?;

    let sys_set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Org, &funs, sys_context).await?;

    let (set_cate_sys_id, set_cate_sys_global_id, set_cate_t1_id, set_cate_t2_id, set_cate_t2_tenant_id, set_cate_t2_a1_id, set_cate_t2_a2_id) =
        test_multi_level_add(sys_context, t1_context, t2_context, t2_a1_context, t2_a2_context, &funs).await?;

    info!("【test_cc_set】 : test_multi_level : Modify Set Cate By sys_context");
    IamSetServ::modify_set_cate(
        &set_cate_sys_global_id,
        &IamSetCateModifyReq {
            name: Some(TrimString("xxx公司_modify".to_string())),
            bus_code: None,
            icon: None,
            sort: None,
            ext: None,
            scope_level: None,
        },
        &funs,
        sys_context,
    )
    .await?;
    IamSetServ::modify_set_cate(
        &set_cate_sys_id,
        &IamSetCateModifyReq {
            name: Some(TrimString("sys私有部门_modify".to_string())),
            bus_code: None,
            icon: None,
            sort: None,
            ext: None,
            scope_level: None,
        },
        &funs,
        sys_context,
    )
    .await?;
    IamSetServ::modify_set_cate(
        &set_cate_t1_id,
        &IamSetCateModifyReq {
            name: Some(TrimString("t1私有部门_modify".to_string())),
            bus_code: None,
            icon: None,
            sort: None,
            ext: None,
            scope_level: None,
        },
        &funs,
        sys_context,
    )
    .await?;
    IamSetServ::modify_set_cate(
        &set_cate_t2_a1_id,
        &IamSetCateModifyReq {
            name: Some(TrimString("t2_a1私有部门_modify".to_string())),
            bus_code: None,
            icon: None,
            sort: None,
            ext: None,
            scope_level: None,
        },
        &funs,
        sys_context,
    )
    .await?;

    info!("【test_cc_set】 : test_multi_level : Find Set Cate By sys_context");
    let set_cates = IamSetServ::get_tree(
        &sys_set_id,
        None,
        &mut RbumSetTreeFilterReq {
            fetch_cate_item: true,
            ..Default::default()
        },
        &funs,
        sys_context,
    )
    .await?;
    assert_eq!(set_cates.len(), 7);
    assert!(set_cates.iter().find(|i| i.name == "t1私有部门_modify" && i.scope_level == RBUM_SCOPE_LEVEL_PRIVATE).is_some());

    info!("【test_cc_set】 : test_multi_level : Delete Set Cate By sys_context");
    assert!(IamSetServ::delete_set_cate(&set_cate_t2_tenant_id, &funs, sys_context).await.is_err());
    IamSetServ::delete_set_cate(&set_cate_t2_a2_id, &funs, sys_context).await?;
    IamSetServ::delete_set_cate(&set_cate_t2_a1_id, &funs, sys_context).await?;
    IamSetServ::delete_set_cate(&set_cate_t2_tenant_id, &funs, sys_context).await?;

    funs.rollback().await?;
    Ok(())
}

pub async fn test_multi_level_by_tenant_context(
    sys_context: &TardisContext,
    t1_context: &TardisContext,
    t2_context: &TardisContext,
    t2_a1_context: &TardisContext,
    t2_a2_context: &TardisContext,
) -> TardisResult<()> {
    let mut funs = iam_constants::get_tardis_inst();
    funs.begin().await?;

    let sys_set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Org, &funs, sys_context).await?;

    let (set_cate_sys_id, set_cate_sys_global_id, set_cate_t1_id, set_cate_t2_id, set_cate_t2_tenant_id, set_cate_t2_a1_id, set_cate_t2_a2_id) =
        test_multi_level_add(sys_context, t1_context, t2_context, t2_a1_context, t2_a2_context, &funs).await?;

    info!("【test_cc_set】 : test_multi_level : Modify Set By tenant_context");
    assert!(IamSetServ::modify_set_cate(
        &set_cate_sys_global_id,
        &IamSetCateModifyReq {
            name: Some(TrimString("xxx公司_modify".to_string())),
            bus_code: None,
            icon: None,
            sort: None,
            ext: None,
            scope_level: None
        },
        &funs,
        t1_context,
    )
    .await
    .is_err());
    assert!(IamSetServ::modify_set_cate(
        &set_cate_sys_id,
        &IamSetCateModifyReq {
            name: Some(TrimString("sys私有部门_modify".to_string())),
            bus_code: None,
            icon: None,
            sort: None,
            ext: None,
            scope_level: None
        },
        &funs,
        t1_context,
    )
    .await
    .is_err());
    assert!(IamSetServ::modify_set_cate(
        &set_cate_t1_id,
        &IamSetCateModifyReq {
            name: Some(TrimString("t1私有部门_modify".to_string())),
            bus_code: None,
            icon: None,
            sort: None,
            ext: None,
            scope_level: None
        },
        &funs,
        t2_context,
    )
    .await
    .is_err());
    IamSetServ::modify_set_cate(
        &set_cate_t2_id,
        &IamSetCateModifyReq {
            name: Some(TrimString("t2私有部门_modify".to_string())),
            bus_code: None,
            icon: None,
            sort: None,
            ext: None,
            scope_level: None,
        },
        &funs,
        t2_context,
    )
    .await?;
    IamSetServ::modify_set_cate(
        &set_cate_t2_tenant_id,
        &IamSetCateModifyReq {
            name: Some(TrimString("t2共享部门_modify".to_string())),
            bus_code: None,
            icon: None,
            sort: None,
            ext: None,
            scope_level: None,
        },
        &funs,
        t2_context,
    )
    .await?;
    IamSetServ::modify_set_cate(
        &set_cate_t2_a1_id,
        &IamSetCateModifyReq {
            name: Some(TrimString("t2_a1私有部门_modify".to_string())),
            bus_code: None,
            icon: None,
            sort: None,
            ext: None,
            scope_level: None,
        },
        &funs,
        t2_context,
    )
    .await?;

    info!("【test_cc_set】 : test_multi_level : Find Set Cate By tenant_context");
    let set_cates = IamSetServ::get_tree(
        &sys_set_id,
        None,
        &mut RbumSetTreeFilterReq {
            fetch_cate_item: true,
            ..Default::default()
        },
        &funs,
        t2_context,
    )
    .await?;
    assert_eq!(set_cates.len(), 7);
    assert!(set_cates.iter().find(|i| i.name == "t2私有部门_modify" && i.scope_level == RBUM_SCOPE_LEVEL_PRIVATE).is_some());

    info!("【test_cc_set】 : test_multi_level : Delete Set Cate By tenant_context");
    assert!(IamSetServ::delete_set_cate(&set_cate_t2_tenant_id, &funs, t2_context).await.is_err());
    IamSetServ::delete_set_cate(&set_cate_t2_a2_id, &funs, t2_context).await?;
    IamSetServ::delete_set_cate(&set_cate_t2_a1_id, &funs, t2_context).await?;
    IamSetServ::delete_set_cate(&set_cate_t2_tenant_id, &funs, t2_context).await?;

    funs.rollback().await?;
    Ok(())
}

pub async fn test_multi_level_by_app_context(
    sys_context: &TardisContext,
    t1_context: &TardisContext,
    t2_context: &TardisContext,
    t2_a1_context: &TardisContext,
    t2_a2_context: &TardisContext,
) -> TardisResult<()> {
    let mut funs = iam_constants::get_tardis_inst();
    funs.begin().await?;

    let sys_set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Org, &funs, sys_context).await?;

    let (set_cate_sys_id, set_cate_sys_global_id, set_cate_t1_id, set_cate_t2_id, set_cate_t2_tenant_id, set_cate_t2_a1_id, set_cate_t2_a2_id) =
        test_multi_level_add(sys_context, t1_context, t2_context, t2_a1_context, t2_a2_context, &funs).await?;

    info!("【test_cc_set】 : test_multi_level : Modify Set By app_context");
    assert!(IamSetServ::modify_set_cate(
        &set_cate_sys_global_id,
        &IamSetCateModifyReq {
            name: Some(TrimString("xxx公司_modify".to_string())),
            bus_code: None,
            icon: None,
            sort: None,
            ext: None,
            scope_level: None
        },
        &funs,
        t2_a1_context,
    )
    .await
    .is_err());
    assert!(IamSetServ::modify_set_cate(
        &set_cate_sys_id,
        &IamSetCateModifyReq {
            name: Some(TrimString("sys私有部门_modify".to_string())),
            bus_code: None,
            icon: None,
            sort: None,
            ext: None,
            scope_level: None
        },
        &funs,
        t2_a1_context,
    )
    .await
    .is_err());
    assert!(IamSetServ::modify_set_cate(
        &set_cate_t2_id,
        &IamSetCateModifyReq {
            name: Some(TrimString("t2私有部门_modify".to_string())),
            bus_code: None,
            icon: None,
            sort: None,
            ext: None,
            scope_level: None
        },
        &funs,
        t2_a1_context,
    )
    .await
    .is_err());
    assert!(IamSetServ::modify_set_cate(
        &set_cate_t2_tenant_id,
        &IamSetCateModifyReq {
            name: Some(TrimString("t2共享部门_modify".to_string())),
            bus_code: None,
            icon: None,
            sort: None,
            ext: None,
            scope_level: None
        },
        &funs,
        t2_a1_context,
    )
    .await
    .is_err());
    assert!(IamSetServ::modify_set_cate(
        &set_cate_t2_a2_id,
        &IamSetCateModifyReq {
            name: Some(TrimString("t2_a2私有部门_modify".to_string())),
            bus_code: None,
            icon: None,
            sort: None,
            ext: None,
            scope_level: None
        },
        &funs,
        t2_a1_context,
    )
    .await
    .is_err());
    IamSetServ::modify_set_cate(
        &set_cate_t2_a1_id,
        &IamSetCateModifyReq {
            name: Some(TrimString("t2_a1私有部门_modify".to_string())),
            bus_code: None,
            icon: None,
            sort: None,
            ext: None,
            scope_level: None,
        },
        &funs,
        t2_a1_context,
    )
    .await?;

    info!("【test_cc_set】 : test_multi_level : Find Set Cate By app_context");
    let set_cates = IamSetServ::get_tree(
        &sys_set_id,
        None,
        &mut RbumSetTreeFilterReq {
            fetch_cate_item: true,
            ..Default::default()
        },
        &funs,
        t2_a1_context,
    )
    .await?;
    assert_eq!(set_cates.len(), 7);
    assert!(set_cates.iter().find(|i| i.name == "t2_a1私有部门_modify" && i.scope_level == RBUM_SCOPE_LEVEL_PRIVATE).is_some());

    info!("【test_cc_set】 : test_multi_level : Delete Set Cate By app_context");
    let item_id1 = IamSetServ::add_set_item(
        &IamSetItemAddReq {
            set_id: sys_set_id.to_string(),
            set_cate_id: set_cate_t2_a1_id.to_string(),
            sort: 0,
            rel_rbum_item_id: t2_a1_context.owner.to_string(),
        },
        &funs,
        t2_a1_context,
    )
    .await?;
    assert!(IamSetServ::delete_set_cate(&set_cate_t2_a1_id, &funs, t2_a1_context).await.is_err());
    IamSetServ::delete_set_item(&item_id1, &funs, t2_a1_context).await?;
    IamSetServ::delete_set_cate(&set_cate_t2_a1_id, &funs, t2_a1_context).await?;

    funs.rollback().await?;
    Ok(())
}
