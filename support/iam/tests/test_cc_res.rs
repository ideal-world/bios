use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;
use tardis::TardisFunsInst;

use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use bios_iam::basic::dto::iam_filter_dto::IamResFilterReq;
use bios_iam::basic::dto::iam_res_dto::{IamResAddReq, IamResModifyReq};
use bios_iam::basic::dto::iam_role_dto::IamRoleAddReq;
use bios_iam::basic::serv::iam_res_serv::IamResServ;
use bios_iam::basic::serv::iam_role_serv::IamRoleServ;
use bios_iam::iam_constants;
use bios_iam::iam_constants::{RBUM_SCOPE_LEVEL_GLOBAL, RBUM_SCOPE_LEVEL_TENANT};
use bios_iam::iam_enumeration::{IamRelKind, IamResKind, IamRoleKind};

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
    info!("【test_cc_res】 : test_single_level : Prepare : Add Role");
    let role_id = IamRoleServ::add_item(
        &mut IamRoleAddReq {
            code: Some(TrimString("role1".to_string())),
            name: TrimString("角色1".to_string()),
            icon: None,
            sort: None,
            disabled: None,
            scope_level: None,
            kind: None,
        },
        &funs,
        context,
    )
    .await?;

    info!("【test_cc_res】 : test_single_level : Add Res");
    let res_id1 = IamResServ::add_item(
        &mut IamResAddReq {
            name: TrimString("测试资源1".to_string()),
            code: TrimString("test_code1".to_string()),
            method: Some(TrimString("GET".to_string())),
            kind: IamResKind::Api,
            icon: None,
            sort: None,
            hide: None,
            action: None,
            scope_level: None,
            disabled: None,
        },
        &funs,
        context,
    )
    .await?;

    let res_id2 = IamResServ::add_item(
        &mut IamResAddReq {
            name: TrimString("测试资源2".to_string()),
            code: TrimString("test_code2".to_string()),
            method: Some(TrimString("GET".to_string())),
            kind: IamResKind::Api,
            icon: None,
            sort: None,
            hide: None,
            action: None,
            scope_level: None,
            disabled: None,
        },
        &funs,
        another_context,
    )
    .await?;

    info!("【test_cc_res】 : test_single_level : Modify Res By Id");
    assert!(IamResServ::modify_item(
        &res_id1,
        &mut IamResModifyReq {
            name: Some(TrimString("测试资源".to_string())),
            icon: Some("/icon/icon.png".to_string()),
            sort: None,
            hide: None,
            action: None,
            scope_level: None,
            disabled: None
        },
        &funs,
        another_context
    )
    .await
    .is_err());

    IamResServ::modify_item(
        &res_id1,
        &mut IamResModifyReq {
            name: Some(TrimString("测试资源".to_string())),
            icon: Some("/icon/icon.png".to_string()),
            sort: None,
            hide: None,
            action: None,
            scope_level: None,
            disabled: None,
        },
        &funs,
        context,
    )
    .await?;

    info!("【test_cc_res】 : test_single_level : Get Res By Id");
    assert!(IamResServ::get_item(&res_id1, &IamResFilterReq::default(), &funs, another_context).await.is_err());
    let res = IamResServ::get_item(&res_id1, &IamResFilterReq::default(), &funs, context).await?;
    assert_eq!(res.id, res_id1);
    assert_eq!(res.name, "测试资源");
    assert_eq!(res.method, "GET");
    assert_eq!(res.icon, "/icon/icon.png");
    assert!(!res.disabled);

    info!("【test_cc_res】 : test_single_level : Find Res");
    let res = IamResServ::paginate_items(&IamResFilterReq::default(), 1, 10, Some(true), None, &funs, context).await?;
    assert_eq!(res.page_number, 1);
    assert_eq!(res.page_size, 10);
    assert!(res.records.iter().any(|i| i.name == "测试资源"));

    info!("【test_cc_res】 : test_single_level : Find Rel Roles By Res Id");
    let res_roles = IamResServ::paginate_from_simple_rel_roles(&IamRelKind::IamResRole, &res_id1, false, 1, 10, None, None, &funs, context).await?;
    assert_eq!(res_roles.total_size, 0);
    info!("【test_cc_res】 : test_single_level : Add Rel Res By Id");
    assert!(IamRoleServ::add_rel_res(&role_id, &res_id1, &funs, another_context).await.is_err());
    IamRoleServ::add_rel_res(&role_id, &res_id1, &funs, context).await?;
    let res_roles = IamResServ::paginate_from_simple_rel_roles(&IamRelKind::IamResRole, &res_id1, false, 1, 10, None, None, &funs, context).await?;
    assert_eq!(res_roles.total_size, 1);

    info!("【test_cc_res】 : test_single_level : Delete Res By Id");
    assert!(IamResServ::delete_item_with_all_rels("11111", &funs, context).await.is_err());
    assert!(IamResServ::delete_item_with_all_rels(&res_id1, &funs, another_context).await.is_err());
    IamResServ::delete_item_with_all_rels(&res_id1, &funs, context).await?;
    assert_eq!(
        IamResServ::paginate_items(
            &IamResFilterReq {
                basic: RbumBasicFilterReq {
                    ids: Some(vec![res_id1]),
                    ..Default::default()
                },
                ..Default::default()
            },
            1,
            10,
            None,
            None,
            &funs,
            context
        )
        .await?
        .total_size,
        0
    );

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
    info!("【test_cc_res】 : test_multi_level : Add Res");

    let res_sys_id = IamResServ::add_item(
        &mut IamResAddReq {
            name: TrimString("res_sys".to_string()),
            code: TrimString("res_sys_id".to_string()),
            method: Some(TrimString("GET".to_string())),
            kind: IamResKind::Api,
            icon: None,
            sort: None,
            hide: None,
            action: None,
            scope_level: None,
            disabled: None,
        },
        funs,
        sys_context,
    )
    .await?;

    let res_sys_global_id = IamResServ::add_item(
        &mut IamResAddReq {
            name: TrimString("res_sys_global".to_string()),
            code: TrimString("res_sys_global_id".to_string()),
            method: Some(TrimString("GET".to_string())),
            kind: IamResKind::Api,
            icon: None,
            sort: None,
            hide: None,
            action: None,
            scope_level: Some(RBUM_SCOPE_LEVEL_GLOBAL),
            disabled: None,
        },
        funs,
        sys_context,
    )
    .await?;

    let res_t1_id = IamResServ::add_item(
        &mut IamResAddReq {
            name: TrimString("res_t1".to_string()),
            code: TrimString("res_t1_id".to_string()),
            method: Some(TrimString("GET".to_string())),
            kind: IamResKind::Api,
            icon: None,
            sort: None,
            hide: None,
            action: None,
            scope_level: None,
            disabled: None,
        },
        funs,
        t1_context,
    )
    .await?;

    let res_t2_id = IamResServ::add_item(
        &mut IamResAddReq {
            name: TrimString("res_t2".to_string()),
            code: TrimString("res_t2_id".to_string()),
            method: Some(TrimString("GET".to_string())),
            kind: IamResKind::Api,
            icon: None,
            sort: None,
            hide: None,
            action: None,
            scope_level: None,
            disabled: None,
        },
        funs,
        t2_context,
    )
    .await?;

    let res_t2_tenant_id = IamResServ::add_item(
        &mut IamResAddReq {
            name: TrimString("res_t2_tenant".to_string()),
            code: TrimString("res_t2_tenant_id".to_string()),
            method: Some(TrimString("GET".to_string())),
            kind: IamResKind::Api,
            icon: None,
            sort: None,
            hide: None,
            action: None,
            scope_level: Some(RBUM_SCOPE_LEVEL_TENANT),
            disabled: None,
        },
        funs,
        t2_context,
    )
    .await?;

    let res_t2_a1_id = IamResServ::add_item(
        &mut IamResAddReq {
            name: TrimString("res_t2_a1".to_string()),
            code: TrimString("res_t2_a1_id".to_string()),
            method: Some(TrimString("GET".to_string())),
            kind: IamResKind::Api,
            icon: None,
            sort: None,
            hide: None,
            action: None,
            scope_level: None,
            disabled: None,
        },
        funs,
        t2_a1_context,
    )
    .await?;

    let res_t2_a2_id = IamResServ::add_item(
        &mut IamResAddReq {
            name: TrimString("res_t2_a2".to_string()),
            code: TrimString("res_t2_a2_id".to_string()),
            method: Some(TrimString("GET".to_string())),
            kind: IamResKind::Api,
            icon: None,
            sort: None,
            hide: None,
            action: None,
            scope_level: None,
            disabled: None,
        },
        funs,
        t2_a2_context,
    )
    .await?;

    Ok((res_sys_id, res_sys_global_id, res_t1_id, res_t2_id, res_t2_tenant_id, res_t2_a1_id, res_t2_a2_id))
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

    info!("【test_cc_res】 : test_multi_level : Prepare : Add Role");
    let role_id = IamRoleServ::add_item(
        &mut IamRoleAddReq {
            code: Some(TrimString("role1".to_string())),
            name: TrimString("角色1".to_string()),
            icon: None,
            sort: None,
            disabled: None,
            scope_level: Some(RBUM_SCOPE_LEVEL_GLOBAL),
            kind: None,
        },
        &funs,
        sys_context,
    )
    .await?;

    let (res_sys_id, res_sys_global_id, res_t1_id, res_t2_id, res_t2_tenant_id, res_t2_a1_id, res_t2_a2_id) =
        test_multi_level_add(sys_context, t1_context, t2_context, t2_a1_context, t2_a2_context, &funs).await?;

    info!("【test_cc_res】 : test_multi_level : Modify Res By sys_context");
    IamResServ::modify_item(
        &res_sys_id,
        &mut IamResModifyReq {
            name: Some(TrimString("res_sys_modify".to_string())),
            icon: None,
            sort: None,
            hide: None,
            action: None,
            scope_level: None,
            disabled: None,
        },
        &funs,
        sys_context,
    )
    .await?;
    IamResServ::modify_item(
        &res_t1_id,
        &mut IamResModifyReq {
            name: Some(TrimString("res_t1_modify".to_string())),
            icon: None,
            sort: None,
            hide: None,
            action: None,
            scope_level: None,
            disabled: None,
        },
        &funs,
        sys_context,
    )
    .await?;
    IamResServ::modify_item(
        &res_t2_a1_id,
        &mut IamResModifyReq {
            name: Some(TrimString("res_t2_a1_modify".to_string())),
            icon: None,
            sort: None,
            hide: None,
            action: None,
            scope_level: None,
            disabled: None,
        },
        &funs,
        sys_context,
    )
    .await?;
    info!("【test_cc_res】 : test_multi_level : Get Res By sys_context");
    assert_eq!(
        IamResServ::get_item(&res_sys_id, &IamResFilterReq::default(), &funs, sys_context).await?.name,
        "res_sys_modify"
    );
    assert!(IamResServ::get_item(&res_t1_id, &IamResFilterReq::default(), &funs, sys_context).await.is_err());
    assert_eq!(
        IamResServ::get_item(
            &res_t1_id,
            &IamResFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            &funs,
            sys_context
        )
        .await?
        .name,
        "res_t1_modify"
    );
    assert!(IamResServ::get_item(&res_t2_a1_id, &IamResFilterReq::default(), &funs, sys_context).await.is_err());
    assert_eq!(
        IamResServ::get_item(
            &res_t2_a1_id,
            &IamResFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            &funs,
            sys_context
        )
        .await?
        .name,
        "res_t2_a1_modify"
    );

    info!("【test_cc_res】 : test_multi_level : Test Rel Roles By sys_context");
    assert_eq!(
        IamResServ::paginate_from_simple_rel_roles(&IamRelKind::IamResRole, &res_sys_id, false, 1, 10, None, None, &funs, sys_context).await?.total_size,
        0
    );
    IamRoleServ::add_rel_res(&role_id, &res_sys_id, &funs, sys_context).await?;
    assert_eq!(
        IamResServ::paginate_from_simple_rel_roles(&IamRelKind::IamResRole, &res_sys_id, false, 1, 10, None, None, &funs, sys_context).await?.total_size,
        1
    );
    assert_eq!(
        IamResServ::paginate_from_simple_rel_roles(&IamRelKind::IamResRole, &res_sys_global_id, false, 1, 10, None, None, &funs, sys_context).await?.total_size,
        0
    );
    IamRoleServ::add_rel_res(&role_id, &res_sys_global_id, &funs, sys_context).await?;
    assert_eq!(
        IamResServ::paginate_from_simple_rel_roles(&IamRelKind::IamResRole, &res_sys_global_id, false, 1, 10, None, None, &funs, sys_context).await?.total_size,
        1
    );
    // assert!(IamRoleServ::add_rel_res(&role_id, &res_t1_id, &funs, sys_context).await.is_err());
    // assert!(IamRoleServ::add_rel_res(&role_id, &res_t2_tenant_id, &funs, sys_context).await.is_err());

    info!("【test_cc_res】 : test_multi_level : Delete Res By sys_context");
    IamResServ::delete_item_with_all_rels(&res_sys_id, &funs, sys_context).await?;
    IamResServ::delete_item_with_all_rels(&res_t1_id, &funs, sys_context).await?;
    IamResServ::delete_item_with_all_rels(&res_t2_a1_id, &funs, sys_context).await?;

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

    info!("【test_cc_res】 : test_multi_level : Prepare : Add Role");
    let role_id = IamRoleServ::add_item(
        &mut IamRoleAddReq {
            code: Some(TrimString("role1".to_string())),
            name: TrimString("角色1".to_string()),
            icon: None,
            sort: None,
            disabled: None,
            scope_level: Some(RBUM_SCOPE_LEVEL_GLOBAL),
            kind: Some(IamRoleKind::Tenant),
        },
        &funs,
        sys_context,
    )
    .await?;

    let (res_sys_id, res_sys_global_id, res_t1_id, res_t2_id, res_t2_tenant_id, res_t2_a1_id, res_t2_a2_id) =
        test_multi_level_add(sys_context, t1_context, t2_context, t2_a1_context, t2_a2_context, &funs).await?;

    info!("【test_cc_res】 : test_multi_level : Modify Res By tenant_context");
    assert!(IamResServ::modify_item(
        &res_sys_id,
        &mut IamResModifyReq {
            name: Some(TrimString("res_sys_modify".to_string())),
            icon: None,
            sort: None,
            hide: None,
            action: None,
            scope_level: None,
            disabled: None,
        },
        &funs,
        t1_context,
    )
    .await
    .is_err());
    assert!(IamResServ::modify_item(
        &res_sys_global_id,
        &mut IamResModifyReq {
            name: Some(TrimString("res_sys_global_modify".to_string())),
            icon: None,
            sort: None,
            hide: None,
            action: None,
            scope_level: None,
            disabled: None,
        },
        &funs,
        t1_context,
    )
    .await
    .is_err());
    assert!(IamResServ::modify_item(
        &res_t1_id,
        &mut IamResModifyReq {
            name: Some(TrimString("res_t1_modify".to_string())),
            icon: None,
            sort: None,
            hide: None,
            action: None,
            scope_level: None,
            disabled: None,
        },
        &funs,
        t2_context,
    )
    .await
    .is_err());
    IamResServ::modify_item(
        &res_t2_id,
        &mut IamResModifyReq {
            name: Some(TrimString("res_t2_modify".to_string())),
            icon: None,
            sort: None,
            hide: None,
            action: None,
            scope_level: None,
            disabled: None,
        },
        &funs,
        t2_context,
    )
    .await?;
    IamResServ::modify_item(
        &res_t2_a1_id,
        &mut IamResModifyReq {
            name: Some(TrimString("res_t2_a1_modify".to_string())),
            icon: None,
            sort: None,
            hide: None,
            action: None,
            scope_level: None,
            disabled: None,
        },
        &funs,
        t2_context,
    )
    .await?;
    info!("【test_cc_res】 : test_multi_level : Get Res By tenant_context");
    assert!(IamResServ::get_item(&res_sys_id, &IamResFilterReq::default(), &funs, t2_context).await.is_err());
    assert!(IamResServ::get_item(&res_t1_id, &IamResFilterReq::default(), &funs, t2_context).await.is_err());
    assert_eq!(
        IamResServ::get_item(&res_sys_global_id, &IamResFilterReq::default(), &funs, t2_context).await?.name,
        "res_sys_global"
    );
    assert_eq!(
        IamResServ::get_item(&res_t2_id, &IamResFilterReq::default(), &funs, t2_context).await?.name,
        "res_t2_modify"
    );
    assert!(IamResServ::get_item(&res_t2_a1_id, &IamResFilterReq::default(), &funs, t2_context).await.is_err());
    assert_eq!(
        IamResServ::get_item(
            &res_t2_a1_id,
            &IamResFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            &funs,
            t2_context
        )
        .await?
        .name,
        "res_t2_a1_modify"
    );

    info!("【test_cc_res】 : test_multi_level : Test Rel Roles By tenant_context");
    assert_eq!(
        IamResServ::paginate_from_simple_rel_roles(&IamRelKind::IamResRole, &res_sys_global_id, false, 1, 10, None, None, &funs, t2_context).await?.total_size,
        0
    );
    IamRoleServ::add_rel_res(&role_id, &res_sys_global_id, &funs, t2_context).await?;
    assert_eq!(
        IamResServ::paginate_from_simple_rel_roles(&IamRelKind::IamResRole, &res_sys_global_id, false, 1, 10, None, None, &funs, t2_context).await?.total_size,
        1
    );
    assert_eq!(
        IamResServ::paginate_from_simple_rel_roles(&IamRelKind::IamResRole, &res_t2_id, false, 1, 10, None, None, &funs, t2_context).await?.total_size,
        0
    );
    IamRoleServ::add_rel_res(&role_id, &res_t2_id, &funs, t2_context).await?;
    assert_eq!(
        IamResServ::paginate_from_simple_rel_roles(&IamRelKind::IamResRole, &res_t2_id, false, 1, 10, None, None, &funs, t2_context).await?.total_size,
        1
    );
    assert!(IamRoleServ::add_rel_res(&role_id, &res_sys_id, &funs, t2_context).await.is_err());
    assert!(IamRoleServ::add_rel_res(&role_id, &res_t1_id, &funs, t2_context).await.is_err());

    info!("【test_cc_res】 : test_multi_level : Delete Res By tenant_context");
    assert!(IamResServ::delete_item_with_all_rels(&res_sys_id, &funs, t2_context).await.is_err());
    assert!(IamResServ::delete_item_with_all_rels(&res_sys_global_id, &funs, t2_context).await.is_err());
    assert!(IamResServ::delete_item_with_all_rels(&res_t1_id, &funs, t2_context).await.is_err());
    IamResServ::delete_item_with_all_rels(&res_t2_id, &funs, t2_context).await?;
    IamResServ::delete_item_with_all_rels(&res_t2_a1_id, &funs, t2_context).await?;

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

    info!("【test_cc_res】 : test_multi_level : Prepare : Add Role");
    let role_id = IamRoleServ::add_item(
        &mut IamRoleAddReq {
            code: Some(TrimString("role1".to_string())),
            name: TrimString("角色1".to_string()),
            icon: None,
            sort: None,
            disabled: None,
            scope_level: Some(RBUM_SCOPE_LEVEL_GLOBAL),
            kind: Some(IamRoleKind::System),
        },
        &funs,
        sys_context,
    )
    .await?;

    let (res_sys_id, res_sys_global_id, res_t1_id, res_t2_id, res_t2_tenant_id, res_t2_a1_id, res_t2_a2_id) =
        test_multi_level_add(sys_context, t1_context, t2_context, t2_a1_context, t2_a2_context, &funs).await?;

    info!("【test_cc_res】 : test_multi_level : Modify Res By app_context");
    assert!(IamResServ::modify_item(
        &res_sys_id,
        &mut IamResModifyReq {
            name: Some(TrimString("res_sys_modify".to_string())),
            icon: None,
            sort: None,
            hide: None,
            action: None,
            scope_level: None,
            disabled: None,
        },
        &funs,
        t2_a1_context,
    )
    .await
    .is_err());
    assert!(IamResServ::modify_item(
        &res_t1_id,
        &mut IamResModifyReq {
            name: Some(TrimString("res_t1_modify".to_string())),
            icon: None,
            sort: None,
            hide: None,
            action: None,
            scope_level: None,
            disabled: None,
        },
        &funs,
        t2_a1_context,
    )
    .await
    .is_err());
    assert!(IamResServ::modify_item(
        &res_t2_tenant_id,
        &mut IamResModifyReq {
            name: Some(TrimString("res_t2_tenant_modify".to_string())),
            icon: None,
            sort: None,
            hide: None,
            action: None,
            scope_level: None,
            disabled: None,
        },
        &funs,
        t2_a1_context,
    )
    .await
    .is_err());
    assert!(IamResServ::modify_item(
        &res_t2_id,
        &mut IamResModifyReq {
            name: Some(TrimString("res_t2_modify".to_string())),
            icon: None,
            sort: None,
            hide: None,
            action: None,
            scope_level: None,
            disabled: None,
        },
        &funs,
        t2_a1_context,
    )
    .await
    .is_err());
    assert!(IamResServ::modify_item(
        &res_t2_a2_id,
        &mut IamResModifyReq {
            name: Some(TrimString("res_t2_a2_modify".to_string())),
            icon: None,
            sort: None,
            hide: None,
            action: None,
            scope_level: None,
            disabled: None,
        },
        &funs,
        t2_a1_context,
    )
    .await
    .is_err());
    IamResServ::modify_item(
        &res_t2_a1_id,
        &mut IamResModifyReq {
            name: Some(TrimString("res_t2_a1_modify".to_string())),
            icon: None,
            sort: None,
            hide: None,
            action: None,
            scope_level: None,
            disabled: None,
        },
        &funs,
        t2_a1_context,
    )
    .await?;

    info!("【test_cc_res】 : test_multi_level : Get Res By app_context");
    assert!(IamResServ::get_item(&res_sys_id, &IamResFilterReq::default(), &funs, t2_a1_context).await.is_err());
    assert_eq!(
        IamResServ::get_item(&res_sys_global_id, &IamResFilterReq::default(), &funs, t2_a1_context).await?.name,
        "res_sys_global"
    );
    assert_eq!(
        IamResServ::get_item(&res_t2_tenant_id, &IamResFilterReq::default(), &funs, t2_a1_context).await?.name,
        "res_t2_tenant"
    );
    assert!(IamResServ::get_item(&res_t1_id, &IamResFilterReq::default(), &funs, t2_a1_context).await.is_err());
    assert!(IamResServ::get_item(&res_t2_id, &IamResFilterReq::default(), &funs, t2_a1_context).await.is_err());
    assert!(IamResServ::get_item(&res_t2_a2_id, &IamResFilterReq::default(), &funs, t2_a1_context).await.is_err());
    assert_eq!(
        IamResServ::get_item(&res_t2_a1_id, &IamResFilterReq::default(), &funs, t2_a1_context).await?.name,
        "res_t2_a1_modify"
    );

    info!("【test_cc_res】 : test_multi_level : Test Rel Roles By app_context");
    assert_eq!(
        IamResServ::paginate_from_simple_rel_roles(&IamRelKind::IamResRole, &res_sys_global_id, false, 1, 10, None, None, &funs, t2_a1_context).await?.total_size,
        0
    );
    IamRoleServ::add_rel_res(&role_id, &res_sys_global_id, &funs, t2_a1_context).await?;
    assert_eq!(
        IamResServ::paginate_from_simple_rel_roles(&IamRelKind::IamResRole, &res_sys_global_id, false, 1, 10, None, None, &funs, t2_a1_context).await?.total_size,
        1
    );
    assert_eq!(
        IamResServ::paginate_from_simple_rel_roles(&IamRelKind::IamResRole, &res_t2_a1_id, false, 1, 10, None, None, &funs, t2_a1_context).await?.total_size,
        0
    );
    IamRoleServ::add_rel_res(&role_id, &res_t2_a1_id, &funs, t2_a1_context).await?;
    assert_eq!(
        IamResServ::paginate_from_simple_rel_roles(&IamRelKind::IamResRole, &res_t2_a1_id, false, 1, 10, None, None, &funs, t2_a1_context).await?.total_size,
        1
    );
    assert_eq!(
        IamResServ::paginate_from_simple_rel_roles(&IamRelKind::IamResRole, &res_sys_global_id, false, 1, 10, None, None, &funs, t2_a1_context).await?.total_size,
        1
    );
    assert_eq!(
        IamResServ::paginate_from_simple_rel_roles(&IamRelKind::IamResRole, &res_t2_tenant_id, false, 1, 10, None, None, &funs, t2_a1_context).await?.total_size,
        0
    );
    IamRoleServ::add_rel_res(&role_id, &res_t2_tenant_id, &funs, t2_a1_context).await?;
    assert_eq!(
        IamResServ::paginate_from_simple_rel_roles(&IamRelKind::IamResRole, &res_t2_tenant_id, false, 1, 10, None, None, &funs, t2_a1_context).await?.total_size,
        1
    );
    assert!(IamRoleServ::add_rel_res(&role_id, &res_sys_id, &funs, t2_a1_context).await.is_err());
    assert!(IamRoleServ::add_rel_res(&role_id, &res_t1_id, &funs, t2_a1_context).await.is_err());
    assert!(IamRoleServ::add_rel_res(&role_id, &res_t2_id, &funs, t2_a1_context).await.is_err());

    info!("【test_cc_res】 : test_multi_level : Delete Res By app_context");
    assert!(IamResServ::delete_item_with_all_rels(&res_sys_id, &funs, t2_a1_context).await.is_err());
    assert!(IamResServ::delete_item_with_all_rels(&res_t1_id, &funs, t2_a1_context).await.is_err());
    assert!(IamResServ::delete_item_with_all_rels(&res_t2_id, &funs, t2_a1_context).await.is_err());
    assert!(IamResServ::delete_item_with_all_rels(&res_t2_tenant_id, &funs, t2_a1_context).await.is_err());
    assert!(IamResServ::delete_item_with_all_rels(&res_t2_a2_id, &funs, t2_a1_context).await.is_err());
    IamResServ::delete_item_with_all_rels(&res_t2_a1_id, &funs, t2_a1_context).await?;

    funs.rollback().await?;
    Ok(())
}
