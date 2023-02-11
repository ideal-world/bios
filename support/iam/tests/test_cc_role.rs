use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;
use tardis::TardisFunsInst;

use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use bios_iam::basic::dto::iam_filer_dto::IamRoleFilterReq;
use bios_iam::basic::dto::iam_res_dto::IamResAddReq;
use bios_iam::basic::dto::iam_role_dto::{IamRoleAddReq, IamRoleModifyReq};
use bios_iam::basic::serv::iam_res_serv::IamResServ;
use bios_iam::basic::serv::iam_role_serv::IamRoleServ;
use bios_iam::iam_constants;
use bios_iam::iam_constants::{RBUM_ITEM_NAME_SYS_ADMIN_ACCOUNT, RBUM_SCOPE_LEVEL_GLOBAL, RBUM_SCOPE_LEVEL_TENANT};
use bios_iam::iam_enumeration::{IamResKind, IamRoleKind};

pub async fn test(
    sys_context: &TardisContext,
    t1_context: &TardisContext,
    t2_context: &TardisContext,
    t2_a1_context: &TardisContext,
    t2_a2_context: &TardisContext,
) -> TardisResult<()> {
    test_single_level(sys_context, RBUM_ITEM_NAME_SYS_ADMIN_ACCOUNT, t1_context).await?;
    test_single_level(t1_context, "测试管理员1", t2_context).await?;
    test_single_level(t2_a1_context, "应用1管理员", t2_a2_context).await?;
    test_multi_level_by_sys_context(sys_context, t1_context, t2_context, t2_a1_context, t2_a2_context).await?;
    test_multi_level_by_tenant_context(sys_context, t1_context, t2_context, t2_a1_context, t2_a2_context).await?;
    test_multi_level_by_app_context(sys_context, t1_context, t2_context, t2_a1_context, t2_a2_context).await?;
    Ok(())
}

async fn test_single_level(context: &TardisContext, account_name: &str, another_context: &TardisContext) -> TardisResult<()> {
    let mut funs = iam_constants::get_tardis_inst();
    funs.begin().await?;

    info!("【test_cc_role】 : test_single_level : Add Role");
    let role_id1 = IamRoleServ::add_item(
        &mut IamRoleAddReq {
            code: TrimString("role1".to_string()),
            name: TrimString("角色1".to_string()),
            icon: None,
            scope_level: None,
            disabled: None,
            sort: None,
            kind: Some(IamRoleKind::Tenant),
        },
        &funs,
        context,
    )
    .await?;
    let role_id2 = IamRoleServ::add_item(
        &mut IamRoleAddReq {
            code: TrimString("role2".to_string()),
            name: TrimString("角色2".to_string()),
            icon: None,
            scope_level: None,
            disabled: None,
            sort: None,
            kind: Some(IamRoleKind::Tenant),
        },
        &funs,
        another_context,
    )
    .await?;

    info!("【test_cc_role】 : test_single_level : Modify Role By Id");
    assert!(IamRoleServ::modify_item(
        &role_id1,
        &mut IamRoleModifyReq {
            name: Some(TrimString("角色3".to_string())),
            icon: Some("/icon/icon.png".to_string()),
            scope_level: None,
            disabled: None,
            sort: None,
            kind: None,
        },
        &funs,
        another_context
    )
    .await
    .is_err());
    IamRoleServ::modify_item(
        &role_id1,
        &mut IamRoleModifyReq {
            name: Some(TrimString("角色3".to_string())),
            icon: Some("/icon/icon.png".to_string()),
            scope_level: None,
            disabled: None,
            sort: None,
            kind: None,
        },
        &funs,
        context,
    )
    .await?;

    info!("【test_cc_role】 : test_single_level : Get Role By Id");
    assert!(IamRoleServ::get_item(&role_id1, &IamRoleFilterReq::default(), &funs, another_context).await.is_err());
    let role = IamRoleServ::get_item(&role_id1, &IamRoleFilterReq::default(), &funs, context).await?;
    assert_eq!(role.id, role_id1);
    assert_eq!(role.name, "角色3");
    assert_eq!(role.icon, "/icon/icon.png");
    assert!(!role.disabled);

    info!("【test_cc_role】 : test_single_level : Find Roles");
    let roles = IamRoleServ::paginate_items(&IamRoleFilterReq::default(), 1, 15, None, None, &funs, context).await?;
    assert_eq!(roles.page_number, 1);
    assert_eq!(roles.page_size, 15);
    assert!(roles.records.iter().any(|i| i.name == "角色3"));

    // ----------------------- Rel Account -----------------------

    info!("【test_cc_role】 : test_single_level : Find Rel Accounts By Role Id");
    let role_accounts = IamRoleServ::paginate_simple_rel_accounts(&role_id1, 1, 10, None, None, &funs, context).await?;
    assert_eq!(role_accounts.page_number, 1);
    assert_eq!(role_accounts.page_size, 10);
    assert_eq!(role_accounts.total_size, 0);

    info!("【test_cc_role】 : test_single_level : Add Rel Account By Id");
    IamRoleServ::add_rel_account(&role_id1, &context.owner, None, &funs, context).await?;

    let role_accounts = IamRoleServ::paginate_simple_rel_accounts(&role_id1, 1, 10, None, None, &funs, context).await?;
    assert_eq!(role_accounts.page_number, 1);
    assert_eq!(role_accounts.page_size, 10);
    assert_eq!(role_accounts.total_size, 1);
    assert_eq!(role_accounts.records.len(), 1);
    assert_eq!(role_accounts.records.get(0).unwrap().rel_name, account_name);

    info!("【test_cc_role】 : test_single_level : Delete Rel Account By Id");
    IamRoleServ::delete_rel_account(&role_id1, &role_accounts.records.get(0).unwrap().rel_id, None, &funs, context).await?;
    let role_accounts = IamRoleServ::paginate_simple_rel_accounts(&role_id1, 1, 10, None, None, &funs, context).await?;
    assert_eq!(role_accounts.total_size, 0);

    // ----------------------- Rel Res -----------------------
    let res_id = IamResServ::add_item(
        &mut IamResAddReq {
            name: TrimString("测试资源".to_string()),
            code: TrimString("test_code".to_string()),
            method: Some(TrimString("GET".to_string())),
            hide: None,
            sort: None,
            icon: None,
            disabled: None,
            kind: IamResKind::Api,
            action: None,
            scope_level: None,
        },
        &funs,
        context,
    )
    .await?;

    info!("【test_cc_role】 : test_single_level : Find Rel Res By Role Id");
    let role_res = IamRoleServ::paginate_simple_rel_res(&role_id1, 1, 10, None, None, &funs, context).await?;
    assert_eq!(role_res.page_number, 1);
    assert_eq!(role_res.page_size, 10);
    assert_eq!(role_res.total_size, 0);

    info!("【test_cc_role】 : test_single_level : Add Rel Res By Id");
    IamRoleServ::add_rel_res(&role_id1, &res_id, &funs, context).await?;

    let role_res = IamRoleServ::paginate_simple_rel_res(&role_id1, 1, 10, None, None, &funs, context).await?;
    assert_eq!(role_res.page_number, 1);
    assert_eq!(role_res.page_size, 10);
    assert_eq!(role_res.total_size, 1);
    assert_eq!(role_res.records.len(), 1);
    assert_eq!(role_res.records.get(0).unwrap().rel_name, "测试资源");

    info!("【test_cc_role】 : test_single_level : Delete Rel Res By Id");
    IamRoleServ::delete_rel_res(&role_id1, &role_res.records.get(0).unwrap().rel_id, &funs, context).await?;
    let role_res = IamRoleServ::paginate_simple_rel_res(&role_id1, 1, 10, None, None, &funs, context).await?;
    assert_eq!(role_res.total_size, 0);

    // ---------------------------

    info!("【test_cc_role】 : test_single_level : Delete Role By Id");
    assert!(IamRoleServ::delete_item_with_all_rels("11111", &funs, context).await.is_err());
    assert!(IamRoleServ::delete_item_with_all_rels(&role_id1, &funs, another_context).await.is_err());
    IamRoleServ::delete_item_with_all_rels(&role_id1, &funs, context).await?;
    assert_eq!(
        IamRoleServ::paginate_items(
            &IamRoleFilterReq {
                basic: RbumBasicFilterReq {
                    ids: Some(vec![role_id1]),
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
    info!("【test_cc_role】 : test_multi_level : Add Role");

    let role_sys_id = IamRoleServ::add_item(
        &mut IamRoleAddReq {
            code: TrimString("role_sys".to_string()),
            name: TrimString("role_sys".to_string()),
            icon: None,
            scope_level: None,
            disabled: None,
            sort: None,
            kind: None,
        },
        funs,
        sys_context,
    )
    .await?;

    let role_sys_global_id = IamRoleServ::add_item(
        &mut IamRoleAddReq {
            code: TrimString("role_sys_global".to_string()),
            name: TrimString("role_sys_global".to_string()),
            icon: None,
            scope_level: Some(RBUM_SCOPE_LEVEL_GLOBAL),
            disabled: None,
            sort: None,
            kind: Some(IamRoleKind::System),
        },
        funs,
        sys_context,
    )
    .await?;

    let role_t1_id = IamRoleServ::add_item(
        &mut IamRoleAddReq {
            code: TrimString("role_t1".to_string()),
            name: TrimString("role_t1".to_string()),
            icon: None,
            scope_level: None,
            disabled: None,
            sort: None,
            kind: None,
        },
        funs,
        t1_context,
    )
    .await?;

    let role_t2_id = IamRoleServ::add_item(
        &mut IamRoleAddReq {
            code: TrimString("role_t2".to_string()),
            name: TrimString("role_t2".to_string()),
            icon: None,
            scope_level: None,
            disabled: None,
            sort: None,
            kind: None,
        },
        funs,
        t2_context,
    )
    .await?;

    let role_t2_tenant_id = IamRoleServ::add_item(
        &mut IamRoleAddReq {
            code: TrimString("role_t2_tenant".to_string()),
            name: TrimString("role_t2_tenant".to_string()),
            icon: None,
            scope_level: Some(RBUM_SCOPE_LEVEL_TENANT),
            disabled: None,
            sort: None,
            kind: Some(IamRoleKind::Tenant),
        },
        funs,
        t2_context,
    )
    .await?;

    let role_t2_a1_id = IamRoleServ::add_item(
        &mut IamRoleAddReq {
            code: TrimString("role_t2_a1".to_string()),
            name: TrimString("role_t2_a1".to_string()),
            icon: None,
            scope_level: None,
            disabled: None,
            sort: None,
            kind: None,
        },
        funs,
        t2_a1_context,
    )
    .await?;

    let role_t2_a2_id = IamRoleServ::add_item(
        &mut IamRoleAddReq {
            code: TrimString("role_t2_a2".to_string()),
            name: TrimString("role_t2_a2".to_string()),
            icon: None,
            scope_level: None,
            disabled: None,
            sort: None,
            kind: None,
        },
        funs,
        t2_a2_context,
    )
    .await?;

    Ok((role_sys_id, role_sys_global_id, role_t1_id, role_t2_id, role_t2_tenant_id, role_t2_a1_id, role_t2_a2_id))
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

    let (role_sys_id, role_sys_global_id, role_t1_id, role_t2_id, role_t2_tenant_id, role_t2_a1_id, role_t2_a2_id) =
        test_multi_level_add(sys_context, t1_context, t2_context, t2_a1_context, t2_a2_context, &funs).await?;

    info!("【test_cc_role】 : test_multi_level : Modify Role By sys_context");
    IamRoleServ::modify_item(
        &role_sys_id,
        &mut IamRoleModifyReq {
            name: None,
            icon: Some("/icon/icon.png".to_string()),
            scope_level: None,
            disabled: None,
            sort: None,
            kind: None,
        },
        &funs,
        sys_context,
    )
    .await?;
    IamRoleServ::modify_item(
        &role_t1_id,
        &mut IamRoleModifyReq {
            name: None,
            icon: Some("/icon/icon.png".to_string()),
            scope_level: None,
            disabled: None,
            sort: None,
            kind: None,
        },
        &funs,
        sys_context,
    )
    .await?;
    IamRoleServ::modify_item(
        &role_t2_a1_id,
        &mut IamRoleModifyReq {
            name: None,
            icon: Some("/icon/icon.png".to_string()),
            scope_level: None,
            disabled: None,
            sort: None,
            kind: None,
        },
        &funs,
        sys_context,
    )
    .await?;
    info!("【test_cc_role】 : test_multi_level : Get Role By sys_context");
    assert_eq!(
        IamRoleServ::get_item(&role_sys_id, &IamRoleFilterReq::default(), &funs, sys_context).await?.name,
        "role_sys"
    );
    assert_eq!(
        IamRoleServ::get_item(&role_sys_global_id, &IamRoleFilterReq::default(), &funs, sys_context).await?.name,
        "role_sys_global"
    );
    assert!(IamRoleServ::get_item(&role_t2_id, &IamRoleFilterReq::default(), &funs, sys_context).await.is_err());
    assert!(IamRoleServ::get_item(&role_t2_tenant_id, &IamRoleFilterReq::default(), &funs, sys_context).await.is_err());
    assert_eq!(
        IamRoleServ::get_item(
            &role_t2_id,
            &IamRoleFilterReq {
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
        "role_t2"
    );
    assert_eq!(
        IamRoleServ::get_item(
            &role_t2_tenant_id,
            &IamRoleFilterReq {
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
        "role_t2_tenant"
    );
    assert!(IamRoleServ::get_item(&role_t2_a1_id, &IamRoleFilterReq::default(), &funs, sys_context).await.is_err());
    assert_eq!(
        IamRoleServ::get_item(
            &role_t2_a1_id,
            &IamRoleFilterReq {
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
        "role_t2_a1"
    );

    info!("【test_cc_role】 : test_multi_level : Test Rel Accounts By sys_context");
    assert_eq!(
        IamRoleServ::paginate_simple_rel_accounts(&role_sys_id, 1, 10, None, None, &funs, sys_context).await?.total_size,
        0
    );
    IamRoleServ::add_rel_account(&role_sys_id, &sys_context.owner, None, &funs, sys_context).await?;
    assert_eq!(
        IamRoleServ::paginate_simple_rel_accounts(&role_sys_id, 1, 10, None, None, &funs, sys_context).await?.total_size,
        1
    );
    assert_eq!(
        IamRoleServ::paginate_simple_rel_accounts(&role_sys_global_id, 1, 10, None, None, &funs, sys_context).await?.total_size,
        0
    );
    IamRoleServ::add_rel_account(&role_sys_global_id, &sys_context.owner, None, &funs, sys_context).await?;
    assert_eq!(
        IamRoleServ::paginate_simple_rel_accounts(&role_sys_global_id, 1, 10, None, None, &funs, sys_context).await?.total_size,
        1
    );
    // assert!(IamRoleServ::add_rel_account(&role_t1_id, &sys_context.owner, None, &funs, sys_context).await.is_err());
    // assert!(IamRoleServ::add_rel_account(&role_t2_tenant_id, &sys_context.owner, None, &funs, sys_context).await.is_err());

    info!("【test_cc_role】 : test_multi_level : Delete Role By sys_context");
    IamRoleServ::delete_item_with_all_rels(&role_sys_id, &funs, sys_context).await?;
    IamRoleServ::delete_item_with_all_rels(&role_t1_id, &funs, sys_context).await?;
    IamRoleServ::delete_item_with_all_rels(&role_t2_a1_id, &funs, sys_context).await?;

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

    let (role_sys_id, role_sys_global_id, role_t1_id, role_t2_id, role_t2_tenant_id, role_t2_a1_id, role_t2_a2_id) =
        test_multi_level_add(sys_context, t1_context, t2_context, t2_a1_context, t2_a2_context, &funs).await?;

    info!("【test_cc_role】 : test_multi_level : Modify Role By tenant_context");
    assert!(IamRoleServ::modify_item(
        &role_sys_id,
        &mut IamRoleModifyReq {
            name: None,
            icon: Some("/icon/icon.png".to_string()),
            scope_level: None,
            disabled: None,
            sort: None,
            kind: None,
        },
        &funs,
        t1_context,
    )
    .await
    .is_err());
    assert!(IamRoleServ::modify_item(
        &role_sys_global_id,
        &mut IamRoleModifyReq {
            name: None,
            icon: Some("/icon/icon.png".to_string()),
            scope_level: None,
            disabled: None,
            sort: None,
            kind: None,
        },
        &funs,
        t1_context,
    )
    .await
    .is_err());
    assert!(IamRoleServ::modify_item(
        &role_t1_id,
        &mut IamRoleModifyReq {
            name: None,
            icon: Some("/icon/icon.png".to_string()),
            scope_level: None,
            disabled: None,
            sort: None,
            kind: None,
        },
        &funs,
        t2_context,
    )
    .await
    .is_err());
    IamRoleServ::modify_item(
        &role_t2_id,
        &mut IamRoleModifyReq {
            name: None,
            icon: Some("/icon/icon.png".to_string()),
            scope_level: None,
            disabled: None,
            sort: None,
            kind: None,
        },
        &funs,
        t2_context,
    )
    .await?;
    IamRoleServ::modify_item(
        &role_t2_a1_id,
        &mut IamRoleModifyReq {
            name: None,
            icon: Some("/icon/icon.png".to_string()),
            scope_level: None,
            disabled: None,
            sort: None,
            kind: None,
        },
        &funs,
        t2_context,
    )
    .await?;
    info!("【test_cc_role】 : test_multi_level : Get Role By tenant_context");
    assert!(IamRoleServ::get_item(&role_sys_id, &IamRoleFilterReq::default(), &funs, t2_context).await.is_err());
    assert!(IamRoleServ::get_item(&role_t1_id, &IamRoleFilterReq::default(), &funs, t2_context).await.is_err());
    assert_eq!(
        IamRoleServ::get_item(&role_sys_global_id, &IamRoleFilterReq::default(), &funs, t2_context).await?.name,
        "role_sys_global"
    );
    assert_eq!(IamRoleServ::get_item(&role_t2_id, &IamRoleFilterReq::default(), &funs, t2_context).await?.name, "role_t2");
    assert!(IamRoleServ::get_item(&role_t2_a1_id, &IamRoleFilterReq::default(), &funs, t2_context).await.is_err());
    assert_eq!(
        IamRoleServ::get_item(
            &role_t2_a1_id,
            &IamRoleFilterReq {
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
        "role_t2_a1"
    );

    info!("【test_cc_role】 : test_multi_level : Test Rel Accounts By tenant_context");
    assert_eq!(
        IamRoleServ::paginate_simple_rel_accounts(&role_sys_global_id, 1, 10, None, None, &funs, t2_context).await?.total_size,
        0
    );
    IamRoleServ::add_rel_account(&role_sys_global_id, &t2_context.owner, None, &funs, t2_context).await?;
    assert_eq!(
        IamRoleServ::paginate_simple_rel_accounts(&role_sys_global_id, 1, 10, None, None, &funs, t2_context).await?.total_size,
        1
    );
    assert_eq!(
        IamRoleServ::paginate_simple_rel_accounts(&role_t2_tenant_id, 1, 10, None, None, &funs, t2_context).await?.total_size,
        0
    );
    IamRoleServ::add_rel_account(&role_t2_tenant_id, &t2_context.owner, None, &funs, t2_context).await?;
    assert_eq!(
        IamRoleServ::paginate_simple_rel_accounts(&role_t2_tenant_id, 1, 10, None, None, &funs, t2_context).await?.total_size,
        1
    );
    assert!(IamRoleServ::add_rel_account(&role_sys_id, &t2_context.owner, None, &funs, t2_context).await.is_err());
    assert!(IamRoleServ::add_rel_account(&role_t1_id, &t2_context.owner, None, &funs, t2_context).await.is_err());

    info!("【test_cc_role】 : test_multi_level : Delete Role By tenant_context");
    assert!(IamRoleServ::delete_item_with_all_rels(&role_sys_id, &funs, t2_context).await.is_err());
    assert!(IamRoleServ::delete_item_with_all_rels(&role_sys_global_id, &funs, t2_context).await.is_err());
    assert!(IamRoleServ::delete_item_with_all_rels(&role_t1_id, &funs, t2_context).await.is_err());
    IamRoleServ::delete_item_with_all_rels(&role_t2_id, &funs, t2_context).await?;
    IamRoleServ::delete_item_with_all_rels(&role_t2_a1_id, &funs, t2_context).await?;

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

    let (role_sys_id, role_sys_global_id, role_t1_id, role_t2_id, role_t2_tenant_id, role_t2_a1_id, role_t2_a2_id) =
        test_multi_level_add(sys_context, t1_context, t2_context, t2_a1_context, t2_a2_context, &funs).await?;

    info!("【test_cc_role】 : test_multi_level : Modify Role By app_context");
    assert!(IamRoleServ::modify_item(
        &role_sys_id,
        &mut IamRoleModifyReq {
            name: None,
            icon: Some("/icon/icon.png".to_string()),
            scope_level: None,
            disabled: None,
            sort: None,
            kind: None,
        },
        &funs,
        t2_a1_context,
    )
    .await
    .is_err());
    assert!(IamRoleServ::modify_item(
        &role_sys_global_id,
        &mut IamRoleModifyReq {
            name: None,
            icon: Some("/icon/icon.png".to_string()),
            scope_level: None,
            disabled: None,
            sort: None,
            kind: None,
        },
        &funs,
        t2_a1_context,
    )
    .await
    .is_err());
    assert!(IamRoleServ::modify_item(
        &role_t1_id,
        &mut IamRoleModifyReq {
            name: None,
            icon: Some("/icon/icon.png".to_string()),
            scope_level: None,
            disabled: None,
            sort: None,
            kind: None,
        },
        &funs,
        t2_a1_context,
    )
    .await
    .is_err());
    assert!(IamRoleServ::modify_item(
        &role_t2_id,
        &mut IamRoleModifyReq {
            name: None,
            icon: Some("/icon/icon.png".to_string()),
            scope_level: None,
            disabled: None,
            sort: None,
            kind: None,
        },
        &funs,
        t2_a1_context,
    )
    .await
    .is_err());
    assert!(IamRoleServ::modify_item(
        &role_t2_tenant_id,
        &mut IamRoleModifyReq {
            name: None,
            icon: Some("/icon/icon.png".to_string()),
            scope_level: None,
            disabled: None,
            sort: None,
            kind: None,
        },
        &funs,
        t2_a1_context,
    )
    .await
    .is_err());
    assert!(IamRoleServ::modify_item(
        &role_t2_a2_id,
        &mut IamRoleModifyReq {
            name: None,
            icon: Some("/icon/icon.png".to_string()),
            scope_level: None,
            disabled: None,
            sort: None,
            kind: None,
        },
        &funs,
        t2_a1_context,
    )
    .await
    .is_err());
    IamRoleServ::modify_item(
        &role_t2_a1_id,
        &mut IamRoleModifyReq {
            name: None,
            icon: Some("/icon/icon.png".to_string()),
            scope_level: None,
            disabled: None,
            sort: None,
            kind: None,
        },
        &funs,
        t2_a1_context,
    )
    .await?;

    info!("【test_cc_role】 : test_multi_level : Get Role By app_context");
    assert!(IamRoleServ::get_item(&role_sys_id, &IamRoleFilterReq::default(), &funs, t2_a1_context).await.is_err());
    assert!(IamRoleServ::get_item(&role_t1_id, &IamRoleFilterReq::default(), &funs, t2_a1_context).await.is_err());
    assert!(IamRoleServ::get_item(&role_t2_id, &IamRoleFilterReq::default(), &funs, t2_a1_context).await.is_err());
    assert_eq!(
        IamRoleServ::get_item(&role_sys_global_id, &IamRoleFilterReq::default(), &funs, t2_a1_context).await?.name,
        "role_sys_global"
    );
    assert_eq!(
        IamRoleServ::get_item(&role_t2_tenant_id, &IamRoleFilterReq::default(), &funs, t2_a1_context).await?.name,
        "role_t2_tenant"
    );
    assert_eq!(
        IamRoleServ::get_item(&role_t2_a1_id, &IamRoleFilterReq::default(), &funs, t2_a1_context).await?.name,
        "role_t2_a1"
    );

    info!("【test_cc_role】 : test_multi_level : Test Rel Accounts By app_context");
    assert_eq!(
        IamRoleServ::paginate_simple_rel_accounts(&role_sys_global_id, 1, 10, None, None, &funs, t2_a1_context).await?.total_size,
        0
    );
    IamRoleServ::add_rel_account(&role_sys_global_id, &t2_a1_context.owner, None, &funs, t2_a1_context).await?;
    assert_eq!(
        IamRoleServ::paginate_simple_rel_accounts(&role_sys_global_id, 1, 10, None, None, &funs, t2_a1_context).await?.total_size,
        1
    );
    assert_eq!(
        IamRoleServ::paginate_simple_rel_accounts(&role_t2_tenant_id, 1, 10, None, None, &funs, t2_a1_context).await?.total_size,
        0
    );
    IamRoleServ::add_rel_account(&role_t2_tenant_id, &t2_a1_context.owner, None, &funs, t2_a1_context).await?;
    assert_eq!(
        IamRoleServ::paginate_simple_rel_accounts(&role_t2_tenant_id, 1, 10, None, None, &funs, t2_a1_context).await?.total_size,
        1
    );

    info!("【test_cc_role】 : test_multi_level : Delete Role By app_context");
    assert!(IamRoleServ::delete_item_with_all_rels(&role_sys_id, &funs, t2_a1_context).await.is_err());
    assert!(IamRoleServ::delete_item_with_all_rels(&role_t1_id, &funs, t2_a1_context).await.is_err());
    assert!(IamRoleServ::delete_item_with_all_rels(&role_t2_id, &funs, t2_a1_context).await.is_err());
    IamRoleServ::delete_item_with_all_rels(&role_t2_a1_id, &funs, t2_a1_context).await?;

    funs.rollback().await?;
    Ok(())
}
