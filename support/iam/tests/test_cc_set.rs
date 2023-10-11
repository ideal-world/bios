use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumRelFilterReq, RbumSetTreeFilterReq};
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use bios_basic::rbum::serv::rbum_set_serv::RbumSetServ;
use bios_iam::basic::dto::iam_account_dto::IamAccountAddReq;
use bios_iam::basic::serv::iam_account_serv::IamAccountServ;
use bios_iam::basic::serv::iam_cert_serv::IamCertServ;

use bios_iam::console_system::serv::iam_cs_org_serv::IamCsOrgServ;
use itertools::Itertools;
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;
use tardis::TardisFunsInst;

use bios_basic::rbum::dto::rbum_set_item_dto::RbumSetItemModifyReq;
use bios_basic::rbum::rbum_enumeration::{RbumRelFromKind, RbumScopeLevelKind, RbumSetCateLevelQueryKind};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_rel_serv::RbumRelServ;
use bios_iam::basic::dto::iam_set_dto::{IamSetCateAddReq, IamSetCateModifyReq, IamSetItemAddReq};
use bios_iam::basic::serv::iam_set_serv::IamSetServ;
use bios_iam::iam_constants;
use bios_iam::iam_constants::{RBUM_SCOPE_LEVEL_GLOBAL, RBUM_SCOPE_LEVEL_PRIVATE, RBUM_SCOPE_LEVEL_TENANT};
use bios_iam::iam_enumeration::{IamRelKind, IamSetKind};

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
    test_bind_platform_to_tenant_node(sys_context, t1_context, t2_context, t2_a1_context, t2_a2_context).await?;
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

    let set_cate_id3 = IamSetServ::add_set_cate(
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
    let tree = IamSetServ::get_tree(
        &set_id,
        &mut RbumSetTreeFilterReq {
            fetch_cate_item: true,
            ..Default::default()
        },
        &funs,
        context,
    )
    .await?;
    assert_eq!(tree.main.len(), 3);
    assert!(tree.main.iter().any(|i| i.bus_code == "bc2-xxxxx"));

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
    assert!(IamSetServ::modify_set_item(
        &item_id1,
        &mut RbumSetItemModifyReq {
            sort: Some(10),
            rel_rbum_set_cate_code: None,
        },
        &funs,
        another_context
    )
    .await
    .is_err());
    IamSetServ::modify_set_item(
        &item_id1,
        &mut RbumSetItemModifyReq {
            sort: Some(10),
            rel_rbum_set_cate_code: None,
        },
        &funs,
        context,
    )
    .await?;

    info!("【test_ca_set】 : test_single_level : Find Set Item");
    let items = IamSetServ::find_set_items(None, Some(set_cate_id1.clone()), None, None, false, None, &funs, context).await?;
    assert_eq!(items.len(), 1);

    info!("【test_ca_set】 : test_single_level : Delete Set Item By Id");
    assert!(IamSetServ::delete_set_item(&item_id1, &funs, another_context).await.is_err());
    IamSetServ::delete_set_item(&item_id1, &funs, context).await?;
    let items = IamSetServ::find_set_items(None, Some(set_cate_id1.clone()), None, None, false, None, &funs, context).await?;
    assert_eq!(items.len(), 0);

    info!("【test_cc_set】 : test_single_level : copy_tree_to_new_set");
    // preparation

    let pri_account_id = IamAccountServ::add_item(
        &mut IamAccountAddReq {
            id: None,
            name: TrimString("私有".to_string()),
            scope_level: None,
            disabled: None,
            icon: None,
            temporary: None,
            status: None,
            lock_status: None,
        },
        &funs,
        context,
    )
    .await?;

    let g_account_id = IamAccountServ::add_item(
        &mut IamAccountAddReq {
            id: None,
            name: TrimString("root".to_string()),
            scope_level: Some(RbumScopeLevelKind::Root),
            disabled: None,
            icon: None,
            temporary: None,
            status: None,
            lock_status: None,
        },
        &funs,
        context,
    )
    .await?;

    let _item_id = IamSetServ::add_set_item(
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
    let _item_id1 = IamSetServ::add_set_item(
        &IamSetItemAddReq {
            set_id: set_id.clone(),
            set_cate_id: set_cate_id3.to_string(),
            sort: 0,
            rel_rbum_item_id: pri_account_id.to_string(),
        },
        &funs,
        context,
    )
    .await?;
    let _item_id1 = IamSetServ::add_set_item(
        &IamSetItemAddReq {
            set_id: set_id.clone(),
            set_cate_id: set_cate_id3.to_string(),
            sort: 0,
            rel_rbum_item_id: g_account_id.to_string(),
        },
        &funs,
        context,
    )
    .await?;

    let another_set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Org, &funs, another_context).await?;
    let another_set_cate_id1 = IamSetServ::add_set_cate(
        &another_set_id,
        &IamSetCateAddReq {
            bus_code: Some(TrimString("bc1".to_string())),
            name: TrimString("another_xxx分公司".to_string()),
            icon: None,
            sort: None,
            ext: None,
            rbum_parent_cate_id: None,
            scope_level: None,
        },
        &funs,
        another_context,
    )
    .await?;

    let tree = RbumSetServ::get_tree(
        &set_id,
        &RbumSetTreeFilterReq {
            sys_code_query_kind: Some(RbumSetCateLevelQueryKind::CurrentAndSub),
            fetch_cate_item: true,
            ..Default::default()
        },
        &funs,
        context,
    )
    .await?;
    println!("raw...tree======={:?}", tree);
    let cates_size = tree.main.len();

    IamSetServ::copy_tree_to_new_set(&tree, &another_set_id, None, Some(another_set_cate_id1.clone()), &funs, another_context).await?;

    let another_tree = RbumSetServ::get_tree(
        &another_set_id,
        &RbumSetTreeFilterReq {
            sys_code_query_kind: Some(RbumSetCateLevelQueryKind::CurrentAndSub),
            fetch_cate_item: true,
            ..Default::default()
        },
        &funs,
        another_context,
    )
    .await?;

    println!("tree======={:?}", another_tree);
    assert_eq!(another_tree.main.len(), cates_size + 1);

    IamSetServ::delete_tree(&another_tree, Some(another_set_cate_id1), &funs, another_context).await?;

    let another_tree = RbumSetServ::get_tree(
        &another_set_id,
        &RbumSetTreeFilterReq {
            sys_code_query_kind: Some(RbumSetCateLevelQueryKind::CurrentAndSub),
            fetch_cate_item: true,
            ..Default::default()
        },
        &funs,
        another_context,
    )
    .await?;

    println!("after_delete_tree======={:?}", another_tree);
    assert_eq!(another_tree.main.len(), 1);

    funs.rollback().await?;
    Ok(())
}

async fn test_multi_level_add<'a>(
    sys_context: &TardisContext,
    t1_context: &TardisContext,
    t2_context: &TardisContext,
    t2_a1_context: &TardisContext,
    t2_a2_context: &TardisContext,
    funs: &TardisFunsInst,
) -> TardisResult<(String, String, String, String, String, String, String)> {
    info!("【test_cc_set】 : test_multi_level : Add Set Cate");

    let sys_set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Org, funs, sys_context).await?;

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
        funs,
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
        funs,
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
        funs,
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

    let (set_cate_sys_id, set_cate_sys_global_id, set_cate_t1_id, _set_cate_t2_id, set_cate_t2_tenant_id, set_cate_t2_a1_id, set_cate_t2_a2_id) =
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
    let tree = IamSetServ::get_tree(
        &sys_set_id,
        &mut RbumSetTreeFilterReq {
            fetch_cate_item: true,
            ..Default::default()
        },
        &funs,
        sys_context,
    )
    .await?;
    assert_eq!(tree.main.len(), 7);
    assert!(tree.main.iter().any(|i| i.name == "t1私有部门_modify" && i.scope_level == RBUM_SCOPE_LEVEL_PRIVATE));

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
    let _t1_set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Org, &funs, t1_context).await?;
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
    let tree = IamSetServ::get_tree(
        &sys_set_id,
        &mut RbumSetTreeFilterReq {
            fetch_cate_item: true,
            ..Default::default()
        },
        &funs,
        t2_context,
    )
    .await?;
    assert_eq!(tree.main.len(), 5);
    assert!(tree.main.iter().any(|i| i.name == "t2私有部门_modify" && i.scope_level == RBUM_SCOPE_LEVEL_PRIVATE));

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

    let (set_cate_sys_id, set_cate_sys_global_id, _set_cate_t1_id, set_cate_t2_id, set_cate_t2_tenant_id, set_cate_t2_a1_id, set_cate_t2_a2_id) =
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
    let tree = IamSetServ::get_tree(
        &sys_set_id,
        &mut RbumSetTreeFilterReq {
            fetch_cate_item: true,
            ..Default::default()
        },
        &funs,
        t2_a1_context,
    )
    .await?;
    assert_eq!(tree.main.len(), 3);
    assert!(tree.main.iter().any(|i| i.name == "t2_a1私有部门_modify" && i.scope_level == RBUM_SCOPE_LEVEL_PRIVATE));

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

pub async fn test_bind_platform_to_tenant_node(
    sys_context: &TardisContext,
    t1_context: &TardisContext,
    t2_context: &TardisContext,
    _t2_a1_context: &TardisContext,
    _t2_a2_context: &TardisContext,
) -> TardisResult<()> {
    let mut funs = iam_constants::get_tardis_inst();
    funs.begin().await?;

    let sys_set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Org, &funs, sys_context).await?;
    let t1_set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Org, &funs, t1_context).await?;
    let t2_set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Org, &funs, t2_context).await?;

    let set_cate_xxx_global_id = IamSetServ::add_set_cate(
        &sys_set_id,
        &IamSetCateAddReq {
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

    let _set_cate_sys_pri_id = IamSetServ::add_set_cate(
        &sys_set_id,
        &IamSetCateAddReq {
            bus_code: None,
            name: TrimString("sys私有部门".to_string()),
            icon: None,
            sort: None,
            ext: None,
            rbum_parent_cate_id: Some(set_cate_xxx_global_id.clone()),
            scope_level: None,
        },
        &funs,
        sys_context,
    )
    .await?;
    info!("【test_cc_set】 : test_bind : bind platform node");
    let set_cate_xxx_sub_id = IamSetServ::add_set_cate(
        &sys_set_id,
        &IamSetCateAddReq {
            name: TrimString("xxx公司_子部门".to_string()),
            scope_level: None,
            bus_code: None,
            icon: None,
            sort: None,
            ext: None,
            rbum_parent_cate_id: Some(set_cate_xxx_global_id.clone()),
        },
        &funs,
        sys_context,
    )
    .await?;
    let _set_cate_xxx_sub_sub_id = IamSetServ::add_set_cate(
        &sys_set_id,
        &IamSetCateAddReq {
            name: TrimString("xxx公司_子部门_子部门".to_string()),
            scope_level: None,
            bus_code: None,
            icon: None,
            sort: None,
            ext: None,
            rbum_parent_cate_id: Some(set_cate_xxx_sub_id.clone()),
        },
        &funs,
        sys_context,
    )
    .await?;

    let _new_t1_set_cate_id = IamSetServ::add_set_cate(
        &t1_set_id,
        &IamSetCateAddReq {
            name: TrimString("t1_子部门".to_string()),
            scope_level: None,
            bus_code: None,
            icon: None,
            sort: None,
            ext: None,
            rbum_parent_cate_id: None,
        },
        &funs,
        t1_context,
    )
    .await?;

    IamCsOrgServ::bind_cate_with_tenant(&set_cate_xxx_global_id, &t1_context.own_paths, &IamSetKind::Org, &funs, sys_context).await?;
    let set_cate_o = RbumRelServ::find_one_rbum(
        &RbumRelFilterReq {
            basic: RbumBasicFilterReq {
                with_sub_own_paths: true,
                ..Default::default()
            },
            tag: Some(IamRelKind::IamOrgRel.to_string()),
            from_rbum_kind: Some(RbumRelFromKind::SetCate),
            from_rbum_id: Some(set_cate_xxx_global_id.clone()),
            from_rbum_scope_levels: None,
            to_rbum_item_scope_levels: None,
            to_rbum_item_id: Some(t1_context.own_paths.clone()),
            to_own_paths: Some(t1_context.own_paths.clone()),
            ..Default::default()
        },
        &funs,
        sys_context,
    )
    .await?;
    assert!(set_cate_o.is_some());

    let mut resp = IamSetServ::get_tree(
        &sys_set_id,
        &mut RbumSetTreeFilterReq {
            fetch_cate_item: false,
            sys_code_query_kind: Some(RbumSetCateLevelQueryKind::Sub),
            sys_code_query_depth: Some(1),
            ..Default::default()
        },
        &funs,
        sys_context,
    )
    .await?;
    assert_eq!(resp.main.len(), 5);
    assert!(resp.main.clone().iter().find(|r| r.id == set_cate_xxx_global_id).unwrap().rel.is_some());
    resp.main.retain(|r| r.ext.contains("set_id"));
    assert_eq!(resp.main.len(), 4);

    let mut resp = IamSetServ::get_tree(
        &t1_set_id,
        &mut RbumSetTreeFilterReq {
            fetch_cate_item: false,
            sys_code_query_kind: Some(RbumSetCateLevelQueryKind::Sub),
            sys_code_query_depth: Some(1),
            ..Default::default()
        },
        &funs,
        t1_context,
    )
    .await?;
    assert_eq!(resp.main.len(), 4);
    resp.main.retain(|r| r.ext.contains("set_id"));
    assert_eq!(resp.main.len(), 0);

    assert!(IamCsOrgServ::bind_cate_with_tenant(&set_cate_xxx_sub_id, &t1_context.own_paths, &IamSetKind::Org, &funs, sys_context).await.is_err());
    info!("【test_cc_set】 : test_bind : unbind test");
    //删除原来的关联
    let old_rel = RbumRelServ::find_one_rbum(
        &RbumRelFilterReq {
            basic: Default::default(),
            tag: Some(IamRelKind::IamOrgRel.to_string()),
            from_rbum_kind: Some(RbumRelFromKind::SetCate),
            from_rbum_id: Some(set_cate_xxx_global_id.clone()),
            from_rbum_scope_levels: None,
            to_rbum_item_id: Some(t1_context.own_paths.clone()),
            to_own_paths: Some(t1_context.own_paths.clone()),
            ext_eq: None,
            ext_like: None,
            to_rbum_item_scope_levels: None,
        },
        &funs,
        sys_context,
    )
    .await?;
    assert!(old_rel.is_some());

    IamCsOrgServ::unbind_cate_with_tenant(old_rel.unwrap(), &funs, sys_context).await?;

    let mut resp = IamSetServ::get_tree(
        &sys_set_id,
        &mut RbumSetTreeFilterReq {
            fetch_cate_item: false,
            sys_code_query_kind: Some(RbumSetCateLevelQueryKind::Sub),
            sys_code_query_depth: Some(1),
            ..Default::default()
        },
        &funs,
        sys_context,
    )
    .await?;
    assert_eq!(resp.main.len(), 5);
    let set_cate_xxx_sub_id = &resp.main.iter().find(|r| r.name == "xxx公司_子部门").unwrap().id.clone();
    resp.main.retain(|r| r.ext.contains("set_id"));
    assert_eq!(resp.main.len(), 0);

    let mut resp = IamSetServ::get_tree(
        &t1_set_id,
        &mut RbumSetTreeFilterReq {
            fetch_cate_item: false,
            sys_code_query_kind: Some(RbumSetCateLevelQueryKind::Sub),
            sys_code_query_depth: Some(1),
            ..Default::default()
        },
        &funs,
        t1_context,
    )
    .await?;
    assert_eq!(resp.main.len(), 4);
    resp.main.retain(|r| r.ext.contains("set_id"));
    assert_eq!(resp.main.len(), 0);

    info!("【test_cc_set】 : test_multi_level : rebind test");
    IamCsOrgServ::bind_cate_with_tenant(set_cate_xxx_sub_id, &t1_context.own_paths, &IamSetKind::Org, &funs, sys_context).await.unwrap();

    let mut resp = IamSetServ::get_tree(
        &sys_set_id,
        &mut RbumSetTreeFilterReq {
            fetch_cate_item: false,
            sys_code_query_kind: Some(RbumSetCateLevelQueryKind::Sub),
            sys_code_query_depth: Some(1),
            ..Default::default()
        },
        &funs,
        sys_context,
    )
    .await?;
    assert_eq!(resp.main.len(), 9);
    assert!(resp.main.clone().iter().find(|r| r.id == *set_cate_xxx_sub_id).unwrap().rel.is_some());
    assert!(resp.main.clone().iter().find(|r| r.id == *set_cate_xxx_global_id).unwrap().ext.contains("disable_import"));
    resp.main.retain(|r| r.ext.contains("set_id"));
    assert_eq!(resp.main.len(), 5);

    let mut resp = IamSetServ::get_tree(
        &t1_set_id,
        &mut RbumSetTreeFilterReq {
            fetch_cate_item: false,
            sys_code_query_kind: Some(RbumSetCateLevelQueryKind::Sub),
            sys_code_query_depth: Some(1),
            ..Default::default()
        },
        &funs,
        t1_context,
    )
    .await?;
    assert_eq!(resp.main.len(), 5);
    resp.main.retain(|r| r.ext.contains("set_id"));
    assert_eq!(resp.main.len(), 0);

    info!("【test_cc_set】 : test_bind : t2 bind node and add rel");

    let set_cate_sys_xxx2_id = IamSetServ::add_set_cate(
        &sys_set_id,
        &IamSetCateAddReq {
            bus_code: None,
            name: TrimString("xxx2公司".to_string()),
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
    let set_cate_sys_xxx2_sub_id = IamSetServ::add_set_cate(
        &sys_set_id,
        &IamSetCateAddReq {
            bus_code: None,
            name: TrimString("xxx2_sub公司".to_string()),
            icon: None,
            sort: None,
            ext: None,
            rbum_parent_cate_id: Some(set_cate_sys_xxx2_id.clone()),
            scope_level: Some(RBUM_SCOPE_LEVEL_GLOBAL),
        },
        &funs,
        sys_context,
    )
    .await?;
    let g_account_id = IamAccountServ::add_item(
        &mut IamAccountAddReq {
            id: None,
            name: TrimString("root".to_string()),
            scope_level: Some(RbumScopeLevelKind::Root),
            disabled: None,
            icon: None,
            temporary: None,
            status: None,
            lock_status: None,
        },
        &funs,
        sys_context,
    )
    .await?;

    let t2_account_id1 = IamAccountServ::add_item(
        &mut IamAccountAddReq {
            id: None,
            name: TrimString("t2_user_1".to_string()),
            scope_level: Some(RBUM_SCOPE_LEVEL_TENANT),
            disabled: None,
            icon: None,
            temporary: None,
            status: None,
            lock_status: None,
        },
        &funs,
        t2_context,
    )
    .await?;

    IamSetServ::add_set_item(
        &IamSetItemAddReq {
            set_id: sys_set_id.clone(),
            set_cate_id: set_cate_sys_xxx2_sub_id.clone(),
            sort: 1,
            rel_rbum_item_id: g_account_id.clone(),
        },
        &funs,
        sys_context,
    )
    .await?;

    let _set_cate_t2_xxx_id = IamSetServ::add_set_cate(
        &t2_set_id,
        &IamSetCateAddReq {
            bus_code: None,
            name: TrimString("t2_xxx公司".to_string()),
            icon: None,
            sort: None,
            ext: None,
            rbum_parent_cate_id: None,
            scope_level: Some(RBUM_SCOPE_LEVEL_GLOBAL),
        },
        &funs,
        t2_context,
    )
    .await?;

    assert!(IamCsOrgServ::bind_cate_with_tenant(set_cate_xxx_sub_id, &t2_context.own_paths, &IamSetKind::Org, &funs, sys_context).await.is_err());
    IamCsOrgServ::bind_cate_with_tenant(&set_cate_sys_xxx2_id, &t2_context.own_paths, &IamSetKind::Org, &funs, sys_context).await.unwrap();

    let mut resp = IamSetServ::get_tree(
        &sys_set_id,
        &mut RbumSetTreeFilterReq {
            fetch_cate_item: true,
            sys_code_query_kind: Some(RbumSetCateLevelQueryKind::Sub),
            sys_code_query_depth: Some(1),
            ..Default::default()
        },
        &funs,
        sys_context,
    )
    .await?;
    assert_eq!(resp.main.len(), 12);
    assert!(resp.main.clone().iter().find(|r| r.id == set_cate_sys_xxx2_id).unwrap().rel.is_some());
    resp.main.retain(|r| r.ext.contains("set_id"));
    assert_eq!(resp.main.len(), 7);
    let set_cate_sys_xxx2_sub_id = &resp.main.clone().iter().find(|r| r.name == "xxx2_sub公司").unwrap().id.clone();
    assert_eq!(
        resp.ext.unwrap().items.get(set_cate_sys_xxx2_sub_id).unwrap().first().unwrap().rel_rbum_item_id,
        g_account_id.clone()
    );

    let resp = IamSetServ::get_tree(
        &t2_set_id,
        &mut RbumSetTreeFilterReq {
            fetch_cate_item: true,
            sys_code_query_kind: Some(RbumSetCateLevelQueryKind::Sub),
            sys_code_query_depth: Some(1),
            ..Default::default()
        },
        &funs,
        t2_context,
    )
    .await?;
    assert_eq!(resp.main.len(), 2);
    assert_eq!(
        resp.ext.unwrap().items.get(set_cate_sys_xxx2_sub_id).unwrap().first().unwrap().rel_rbum_item_id,
        g_account_id.clone()
    );

    // sys cc查询第一层 ->xxx
    let mut resp = IamSetServ::get_tree(
        &sys_set_id,
        &mut RbumSetTreeFilterReq {
            fetch_cate_item: true,
            sys_codes: Some(vec!["".to_string()]),
            sys_code_query_kind: Some(RbumSetCateLevelQueryKind::CurrentAndSub),
            sys_code_query_depth: Some(1),
            ..Default::default()
        },
        &funs,
        sys_context,
    )
    .await?;

    assert_eq!(resp.main.len(), 2);
    let set_cate_sys_xxx2 = &resp.main.iter().find(|r| r.name == "xxx2公司").unwrap().clone();

    resp.main.retain(|r| r.ext.contains("set_id"));
    assert_eq!(resp.main.len(), 0);

    // sys cc查询第二层 ->t2_xxx xxx2_sub
    let query_ctx = IamCertServ::try_use_tenant_ctx(sys_context.clone(), set_cate_sys_xxx2.rel.clone())?;
    let query_code = if query_ctx.own_paths.is_empty() {
        IamSetServ::get_default_org_code_by_system()
    } else {
        IamSetServ::get_default_org_code_by_tenant(&funs, &query_ctx)?
    };
    let query_set_id = IamSetServ::get_set_id_by_code(&query_code, true, &funs, &query_ctx).await?;

    let resp = IamSetServ::get_tree(
        &query_set_id,
        &mut RbumSetTreeFilterReq {
            fetch_cate_item: true,
            sys_codes: Some(vec![]),
            sys_code_query_kind: Some(RbumSetCateLevelQueryKind::CurrentAndSub),
            sys_code_query_depth: Some(1),
            ..Default::default()
        },
        &funs,
        &query_ctx,
    )
    .await?;

    assert_eq!(resp.main.len(), 2);
    let set_cate_sys_xxx2_sub_id = &resp.main.clone().iter().find(|r| r.name == "xxx2_sub公司").unwrap().id.clone();
    assert_eq!(
        resp.ext.unwrap().items.get(set_cate_sys_xxx2_sub_id).unwrap().first().unwrap().rel_rbum_item_id,
        g_account_id.clone()
    );

    let set_cate_t2_xxx_id = &resp.main.clone().iter().find(|r| r.name == "t2_xxx公司").unwrap().id.clone();
    let t2_xxx_own_paths = &resp.main.clone().iter().find(|r| r.name == "t2_xxx公司").unwrap().own_paths.clone();
    let query_ctx = IamCertServ::try_use_tenant_ctx(sys_context.clone(), Some(t2_xxx_own_paths.to_owned()))?;
    let query_set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Org, &funs, &query_ctx).await?;
    IamSetServ::add_set_item(
        &IamSetItemAddReq {
            set_id: query_set_id.clone(),
            set_cate_id: set_cate_t2_xxx_id.clone(),
            sort: 1,
            rel_rbum_item_id: g_account_id.clone(),
        },
        &funs,
        &query_ctx,
    )
    .await?;
    IamSetServ::add_set_item(
        &IamSetItemAddReq {
            set_id: query_set_id.clone(),
            set_cate_id: set_cate_t2_xxx_id.clone(),
            sort: 2,
            rel_rbum_item_id: t2_account_id1.clone(),
        },
        &funs,
        &query_ctx,
    )
    .await?;

    let resp = IamSetServ::get_tree(
        &t2_set_id,
        &mut RbumSetTreeFilterReq {
            fetch_cate_item: true,
            sys_code_query_kind: Some(RbumSetCateLevelQueryKind::Sub),
            sys_code_query_depth: Some(1),
            ..Default::default()
        },
        &funs,
        t2_context,
    )
    .await?;
    assert_eq!(resp.main.len(), 2);
    let set_cate_t2_xxx_id = &resp.main.clone().iter().find(|r| r.name == "t2_xxx公司").unwrap().id.clone();
    let cate_t2_items = resp.ext.unwrap().items.get(set_cate_t2_xxx_id).unwrap().clone();
    assert!(cate_t2_items.iter().any(|r| r.rel_rbum_item_id == g_account_id));
    assert!(cate_t2_items.iter().any(|r| r.rel_rbum_item_id == t2_account_id1));

    let resp = IamSetServ::get_tree(
        &sys_set_id,
        &mut RbumSetTreeFilterReq {
            fetch_cate_item: true,
            sys_code_query_kind: Some(RbumSetCateLevelQueryKind::Sub),
            sys_code_query_depth: Some(1),
            ..Default::default()
        },
        &funs,
        sys_context,
    )
    .await?;
    println!("resp.main.len()======{}", resp.main.len());
    assert_eq!(resp.main.len(), 12);
    let set_cate_t2_xxx_id = &resp.main.clone().iter().find(|r| r.name == "t2_xxx公司").unwrap().id.clone();
    let cate_t2_items = resp.ext.unwrap().items.get(set_cate_t2_xxx_id).unwrap().clone();
    assert!(cate_t2_items.iter().any(|r| r.rel_rbum_item_id == g_account_id));
    assert!(cate_t2_items.iter().any(|r| r.rel_rbum_item_id == t2_account_id1));

    let search_vec = resp.main.clone();
    for ele in resp.main.clone() {
        for sub_ele in search_vec.iter().filter(|m| m.pid == Some(ele.id.clone())).collect_vec() {
            assert!(sub_ele.sys_code.contains(&ele.sys_code));
        }
    }

    funs.rollback().await?;
    Ok(())
}
