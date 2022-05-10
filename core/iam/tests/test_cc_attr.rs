use tardis::basic::dto::{TardisContext, TardisFunsInst};
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;

use bios_basic::rbum::dto::rbum_kind_attr_dto::RbumKindAttrModifyReq;
use bios_basic::rbum::rbum_enumeration::{RbumDataTypeKind, RbumScopeLevelKind, RbumWidgetTypeKind};
use bios_iam::basic::dto::iam_attr_dto::IamKindAttrAddReq;
use bios_iam::basic::serv::iam_attr_serv::IamAttrServ;
use bios_iam::iam_constants;

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
    info!("【test_cc_attr】 : test_single_level : Add Account Attr");
    let attr1 = IamAttrServ::add_account_attr(&package_test_attr_add_req("attr1", None), &funs, context).await?;

    info!("【test_cc_attr】 : test_single_level : Modify Account Attr By Id");
    assert!(IamAttrServ::modify_account_attr(&attr1, &mut package_test_attr_modify_req("attr1_modify"), &funs, another_context).await.is_err());
    IamAttrServ::modify_account_attr(&attr1, &mut package_test_attr_modify_req("attr1_modify"), &funs, context).await?;

    info!("【test_cc_attr】 : test_single_level : Get Account Attr By Id");
    assert!(IamAttrServ::get_account_attr(&attr1, &funs, another_context).await.is_err());
    assert_eq!(IamAttrServ::get_account_attr(&attr1, &funs, context).await?.name, "attr1_modify");

    info!("【test_cc_attr】 : test_single_level : Find Account Attrs");
    assert_eq!(IamAttrServ::find_account_attrs(&funs, another_context).await?.len(), 0);
    assert_eq!(IamAttrServ::find_account_attrs(&funs, context).await?.len(), 1);

    info!("【test_cc_attr】 : test_single_level : Add Account Attr value");
    assert!(IamAttrServ::add_account_attr_value("x".to_string(), &attr1, &another_context.owner, &funs, another_context).await.is_err());
    assert!(IamAttrServ::add_account_attr_value("x".to_string(), &attr1, &context.owner, &funs, another_context).await.is_err());
    let attr1_value = IamAttrServ::add_account_attr_value("attr1_value".to_string(), &attr1, &context.owner, &funs, context).await?;

    info!("【test_cc_attr】 : test_single_level : Modify Account Attr value");
    assert!(IamAttrServ::modify_account_attr_value(&attr1_value, "attr1_value_modify".to_string(), &funs, another_context).await.is_err());
    IamAttrServ::modify_account_attr_value(&attr1_value, "attr1_value_modify".to_string(), &funs, context).await?;

    info!("【test_cc_attr】 : test_single_level : Get Account Attr value");
    assert!(IamAttrServ::get_account_attr_value(&attr1_value, false, &funs, another_context).await.is_err());
    assert_eq!(IamAttrServ::get_account_attr_value(&attr1_value, false, &funs, context).await?.value, "attr1_value_modify");

    info!("【test_cc_attr】 : test_single_level : Delete Account Attr value");
    assert!(IamAttrServ::delete_account_attr_value(&attr1_value, &funs, another_context).await.is_err());
    IamAttrServ::delete_account_attr_value(&attr1_value, &funs, context).await?;

    info!("【test_cc_attr】 : test_single_level : Delete Account Attr");
    assert!(IamAttrServ::delete_account_attr(&attr1, &funs, another_context).await.is_err());
    IamAttrServ::delete_account_attr(&attr1, &funs, context).await?;

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
) -> TardisResult<(String, String, String, String, String, String, String, String, String, String)> {
    info!("【test_cc_attr】 : test_multi_level : Add Account Attr");
    let sys_attr1_id = IamAttrServ::add_account_attr(&package_test_attr_add_req("sys_attr1", None), funs, sys_context).await?;
    let sys_attr2_global_id = IamAttrServ::add_account_attr(
        &package_test_attr_add_req("sys_attr2_global", Some(iam_constants::RBUM_SCOPE_LEVEL_GLOBAL)),
        funs,
        sys_context,
    )
    .await?;
    let t1_attr1_id = IamAttrServ::add_account_attr(&package_test_attr_add_req("t1_attr1", None), funs, t1_context).await?;
    let t2_attr1_id = IamAttrServ::add_account_attr(&package_test_attr_add_req("t2_attr1", None), funs, t2_context).await?;
    let t2_attr2_tenant_id = IamAttrServ::add_account_attr(
        &package_test_attr_add_req("t2_attr2_tenant", Some(iam_constants::RBUM_SCOPE_LEVEL_TENANT)),
        funs,
        t2_context,
    )
    .await?;
    let t2_a1_attr1_id = IamAttrServ::add_account_attr(&package_test_attr_add_req("t2_a1_attr1", None), funs, t2_a1_context).await?;
    let t2_a1_attr2_app_id = IamAttrServ::add_account_attr(
        &package_test_attr_add_req("t2_a1_attr2_app", Some(iam_constants::RBUM_SCOPE_LEVEL_APP)),
        funs,
        t2_a1_context,
    )
    .await?;
    let t2_a1_attr2_tenant_id = IamAttrServ::add_account_attr(
        &package_test_attr_add_req("t2_a1_attr2_tenant", Some(iam_constants::RBUM_SCOPE_LEVEL_TENANT)),
        funs,
        t2_a1_context,
    )
    .await?;
    let t2_a1_attr3_global_id = IamAttrServ::add_account_attr(
        &package_test_attr_add_req("t2_a1_attr3_global", Some(iam_constants::RBUM_SCOPE_LEVEL_GLOBAL)),
        funs,
        t2_a1_context,
    )
    .await?;
    let t2_a2_attr1_id = IamAttrServ::add_account_attr(&package_test_attr_add_req("t2_a2_attr1", None), funs, t2_a2_context).await?;

    Ok((
        sys_attr1_id,
        sys_attr2_global_id,
        t1_attr1_id,
        t2_attr1_id,
        t2_attr2_tenant_id,
        t2_a1_attr1_id,
        t2_a1_attr2_app_id,
        t2_a1_attr2_tenant_id,
        t2_a1_attr3_global_id,
        t2_a2_attr1_id,
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

    let (
        sys_attr1_id,
        sys_attr2_global_id,
        t1_attr1_id,
        t2_attr1_id,
        t2_attr2_tenant_id,
        t2_a1_attr1_id,
        t2_a1_attr2_app_id,
        t2_a1_attr2_tenant_id,
        t2_a1_attr3_global_id,
        t2_a2_attr1_id,
    ) = test_multi_level_add(sys_context, t1_context, t2_context, t2_a1_context, t2_a2_context, &funs).await?;

    info!("【test_cc_attr】 : test_multi_level : Modify Account Attr By sys_context");
    IamAttrServ::modify_account_attr(&sys_attr1_id, &mut package_test_attr_modify_req("sys_attr1_modify"), &funs, sys_context).await?;
    IamAttrServ::modify_account_attr(&sys_attr2_global_id, &mut package_test_attr_modify_req("sys_attr2_global_modify"), &funs, sys_context).await?;
    IamAttrServ::modify_account_attr(&t2_attr1_id, &mut package_test_attr_modify_req("t2_attr1_modify"), &funs, sys_context).await?;
    IamAttrServ::modify_account_attr(&t2_attr2_tenant_id, &mut package_test_attr_modify_req("t2_attr2_tenant_modify"), &funs, sys_context).await?;
    IamAttrServ::modify_account_attr(&t2_a1_attr1_id, &mut package_test_attr_modify_req("t2_a1_attr1_modify"), &funs, sys_context).await?;
    IamAttrServ::modify_account_attr(&t2_a1_attr2_app_id, &mut package_test_attr_modify_req("t2_a1_attr2_app_modify"), &funs, sys_context).await?;
    IamAttrServ::modify_account_attr(&t2_a1_attr2_tenant_id, &mut package_test_attr_modify_req("t2_a1_attr2_tenant_modify"), &funs, sys_context).await?;
    IamAttrServ::modify_account_attr(&t2_a1_attr3_global_id, &mut package_test_attr_modify_req("t2_a1_attr3_global_modify"), &funs, sys_context).await?;
    IamAttrServ::modify_account_attr(&t2_a2_attr1_id, &mut package_test_attr_modify_req("t2_a2_attr1_modify"), &funs, sys_context).await?;

    info!("【test_cc_attr】 : test_multi_level : Get Account Attr By sys_context");
    assert_eq!(IamAttrServ::get_account_attr(&sys_attr1_id, &funs, sys_context).await?.name, "sys_attr1_modify");
    assert_eq!(
        IamAttrServ::get_account_attr(&sys_attr2_global_id, &funs, sys_context).await?.name,
        "sys_attr2_global_modify"
    );
    assert!(IamAttrServ::get_account_attr(&t2_attr1_id, &funs, sys_context).await.is_err());
    assert!(IamAttrServ::get_account_attr(&t2_attr2_tenant_id, &funs, sys_context).await.is_err());
    assert!(IamAttrServ::get_account_attr(&t2_a1_attr1_id, &funs, sys_context).await.is_err());
    assert!(IamAttrServ::get_account_attr(&t2_a1_attr2_app_id, &funs, sys_context).await.is_err());
    assert!(IamAttrServ::get_account_attr(&t2_a1_attr2_tenant_id, &funs, sys_context).await.is_err());
    assert_eq!(
        IamAttrServ::get_account_attr(&t2_a1_attr3_global_id, &funs, sys_context).await?.name,
        "t2_a1_attr3_global_modify"
    );
    assert!(IamAttrServ::get_account_attr(&t2_a2_attr1_id, &funs, sys_context).await.is_err());

    info!("【test_cc_attr】 : test_multi_level : Find Account Attrs By sys_context");
    assert_eq!(IamAttrServ::find_account_attrs(&funs, sys_context).await?.len(), 3);

    info!("【test_cc_attr】 : test_multi_level : Add Account Attr value By sys_context");
    let sys_attr1_value = IamAttrServ::add_account_attr_value("sys_attr1_value".to_string(), &sys_attr1_id, &sys_context.owner, &funs, sys_context).await?;
    let sys_attr2_global_value = IamAttrServ::add_account_attr_value("sys_attr2_global_value".to_string(), &sys_attr2_global_id, &sys_context.owner, &funs, sys_context).await?;
    assert!(IamAttrServ::add_account_attr_value("t2_attr1_value".to_string(), &t2_attr1_id, &sys_context.owner, &funs, sys_context).await.is_err());
    assert!(IamAttrServ::add_account_attr_value("t2_attr2_tenant_value".to_string(), &t2_attr2_tenant_id, &sys_context.owner, &funs, sys_context).await.is_err());
    assert!(IamAttrServ::add_account_attr_value("t2_a1_attr1_value".to_string(), &t2_a1_attr1_id, &sys_context.owner, &funs, sys_context).await.is_err());
    assert!(IamAttrServ::add_account_attr_value("t2_a1_attr2_app_value".to_string(), &t2_a1_attr2_app_id, &sys_context.owner, &funs, sys_context).await.is_err());
    let t2_a1_attr3_global_value =
        IamAttrServ::add_account_attr_value("t2_a1_attr3_global_value".to_string(), &t2_a1_attr3_global_id, &sys_context.owner, &funs, sys_context).await?;

    info!("【test_cc_attr】 : test_multi_level : Modify Account Attr value By sys_context");
    IamAttrServ::modify_account_attr_value(&sys_attr1_value, "sys_attr1_value_modify".to_string(), &funs, sys_context).await?;
    IamAttrServ::modify_account_attr_value(&sys_attr2_global_value, "sys_attr2_global_value_modify".to_string(), &funs, sys_context).await?;
    IamAttrServ::modify_account_attr_value(&t2_a1_attr3_global_value, "t2_a1_attr3_global_value_modify".to_string(), &funs, sys_context).await?;

    info!("【test_cc_attr】 : test_multi_level : Get Account Attr value By sys_context");
    assert_eq!(
        IamAttrServ::get_account_attr_value(&sys_attr1_value, false, &funs, sys_context).await?.value,
        "sys_attr1_value_modify"
    );
    assert_eq!(
        IamAttrServ::get_account_attr_value(&sys_attr2_global_value, false, &funs, sys_context).await?.value,
        "sys_attr2_global_value_modify"
    );
    assert_eq!(
        IamAttrServ::get_account_attr_value(&t2_a1_attr3_global_value, false, &funs, sys_context).await?.value,
        "t2_a1_attr3_global_value_modify"
    );

    info!("【test_cc_attr】 : test_multi_level : Delete Account Attr value By sys_context");
    IamAttrServ::delete_account_attr_value(&sys_attr1_value, &funs, sys_context).await?;
    IamAttrServ::delete_account_attr_value(&sys_attr2_global_value, &funs, sys_context).await?;
    IamAttrServ::delete_account_attr_value(&t2_a1_attr3_global_value, &funs, sys_context).await?;

    info!("【test_cc_attr】 : test_multi_level : Delete Account Attr By sys_context");
    IamAttrServ::delete_account_attr(&sys_attr1_id, &funs, sys_context).await?;
    IamAttrServ::delete_account_attr(&sys_attr2_global_id, &funs, sys_context).await?;
    IamAttrServ::delete_account_attr(&t2_attr1_id, &funs, sys_context).await?;
    IamAttrServ::delete_account_attr(&t2_attr2_tenant_id, &funs, sys_context).await?;
    IamAttrServ::delete_account_attr(&t2_a1_attr1_id, &funs, sys_context).await?;
    IamAttrServ::delete_account_attr(&t2_a1_attr2_app_id, &funs, sys_context).await?;
    IamAttrServ::delete_account_attr(&t2_a1_attr2_tenant_id, &funs, sys_context).await?;
    IamAttrServ::delete_account_attr(&t2_a1_attr3_global_id, &funs, sys_context).await?;
    IamAttrServ::delete_account_attr(&t2_a2_attr1_id, &funs, sys_context).await?;

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

    let (
        sys_attr1_id,
        sys_attr2_global_id,
        t1_attr1_id,
        t2_attr1_id,
        t2_attr2_tenant_id,
        t2_a1_attr1_id,
        t2_a1_attr2_app_id,
        t2_a1_attr2_tenant_id,
        t2_a1_attr3_global_id,
        t2_a2_attr1_id,
    ) = test_multi_level_add(sys_context, t1_context, t2_context, t2_a1_context, t2_a2_context, &funs).await?;

    info!("【test_cc_attr】 : test_multi_level : Modify Account Attr By tenant_context");
    assert!(IamAttrServ::modify_account_attr(&sys_attr1_id, &mut package_test_attr_modify_req("sys_attr1_modify"), &funs, t2_context).await.is_err());
    assert!(IamAttrServ::modify_account_attr(&sys_attr2_global_id, &mut package_test_attr_modify_req("sys_attr2_global_modify"), &funs, t2_context).await.is_err());
    IamAttrServ::modify_account_attr(&t2_attr1_id, &mut package_test_attr_modify_req("t2_attr1_modify"), &funs, t2_context).await?;
    IamAttrServ::modify_account_attr(&t2_attr2_tenant_id, &mut package_test_attr_modify_req("t2_attr2_tenant_modify"), &funs, t2_context).await?;
    IamAttrServ::modify_account_attr(&t2_a1_attr1_id, &mut package_test_attr_modify_req("t2_a1_attr1_modify"), &funs, t2_context).await?;
    IamAttrServ::modify_account_attr(&t2_a1_attr2_app_id, &mut package_test_attr_modify_req("t2_a1_attr2_app_modify"), &funs, t2_context).await?;
    IamAttrServ::modify_account_attr(&t2_a1_attr2_tenant_id, &mut package_test_attr_modify_req("t2_a1_attr2_tenant_modify"), &funs, t2_context).await?;
    IamAttrServ::modify_account_attr(&t2_a1_attr3_global_id, &mut package_test_attr_modify_req("t2_a1_attr3_global_modify"), &funs, t2_context).await?;
    IamAttrServ::modify_account_attr(&t2_a2_attr1_id, &mut package_test_attr_modify_req("t2_a2_attr1_modify"), &funs, t2_context).await?;

    info!("【test_cc_attr】 : test_multi_level : Get Account Attr By tenant_context");
    assert!(IamAttrServ::get_account_attr(&sys_attr1_id, &funs, t2_context).await.is_err());
    assert_eq!(IamAttrServ::get_account_attr(&sys_attr2_global_id, &funs, t2_context).await?.name, "sys_attr2_global");
    assert_eq!(IamAttrServ::get_account_attr(&t2_attr1_id, &funs, t2_context).await?.name, "t2_attr1_modify");
    assert_eq!(IamAttrServ::get_account_attr(&t2_attr2_tenant_id, &funs, t2_context).await?.name, "t2_attr2_tenant_modify");
    assert!(IamAttrServ::get_account_attr(&t2_a1_attr1_id, &funs, t2_context).await.is_err());
    assert!(IamAttrServ::get_account_attr(&t2_a1_attr2_app_id, &funs, t2_context).await.is_err());
    assert_eq!(
        IamAttrServ::get_account_attr(&t2_a1_attr2_tenant_id, &funs, t2_context).await?.name,
        "t2_a1_attr2_tenant_modify"
    );
    assert_eq!(
        IamAttrServ::get_account_attr(&t2_a1_attr3_global_id, &funs, t2_context).await?.name,
        "t2_a1_attr3_global_modify"
    );
    assert!(IamAttrServ::get_account_attr(&t2_a2_attr1_id, &funs, t2_context).await.is_err());

    info!("【test_cc_attr】 : test_multi_level : Find Account Attrs By tenant_context");
    assert_eq!(IamAttrServ::find_account_attrs(&funs, t2_context).await?.len(), 5);

    info!("【test_cc_attr】 : test_multi_level : Add Account Attr value By tenant_context");
    assert!(IamAttrServ::add_account_attr_value("sys_attr1_value".to_string(), &sys_attr1_id, &t2_context.owner, &funs, t2_context).await.is_err());
    let sys_attr2_global_value = IamAttrServ::add_account_attr_value("sys_attr2_global_value".to_string(), &sys_attr2_global_id, &t2_context.owner, &funs, t2_context).await?;
    let t2_attr1_value = IamAttrServ::add_account_attr_value("t2_attr1_value".to_string(), &t2_attr1_id, &t2_context.owner, &funs, t2_context).await?;
    let t2_attr2_tenant_value = IamAttrServ::add_account_attr_value("t2_attr2_tenant_value".to_string(), &t2_attr2_tenant_id, &t2_context.owner, &funs, t2_context).await?;
    assert!(IamAttrServ::add_account_attr_value("t2_a1_attr1_value".to_string(), &t2_a1_attr1_id, &t2_context.owner, &funs, t2_context).await.is_err());
    assert!(IamAttrServ::add_account_attr_value("t2_a1_attr2_app_value".to_string(), &t2_a1_attr2_app_id, &t2_context.owner, &funs, t2_context).await.is_err());
    let t2_a1_attr3_global_value =
        IamAttrServ::add_account_attr_value("t2_a1_attr3_global_value".to_string(), &t2_a1_attr3_global_id, &t2_context.owner, &funs, t2_context).await?;

    info!("【test_cc_attr】 : test_multi_level : Modify Account Attr value By tenant_context");
    IamAttrServ::modify_account_attr_value(&sys_attr2_global_value, "sys_attr2_global_value_modify".to_string(), &funs, t2_context).await?;
    IamAttrServ::modify_account_attr_value(&t2_attr1_value, "t2_attr1_value_modify".to_string(), &funs, t2_context).await?;
    IamAttrServ::modify_account_attr_value(&t2_attr2_tenant_value, "t2_attr2_tenant_value_modify".to_string(), &funs, t2_context).await?;
    IamAttrServ::modify_account_attr_value(&t2_a1_attr3_global_value, "t2_a1_attr3_global_value_modify".to_string(), &funs, t2_context).await?;

    info!("【test_cc_attr】 : test_multi_level : Get Account Attr value By tenant_context");
    assert_eq!(
        IamAttrServ::get_account_attr_value(&sys_attr2_global_value, false, &funs, t2_context).await?.value,
        "sys_attr2_global_value_modify"
    );
    assert_eq!(
        IamAttrServ::get_account_attr_value(&t2_attr1_value, false, &funs, t2_context).await?.value,
        "t2_attr1_value_modify"
    );
    assert_eq!(
        IamAttrServ::get_account_attr_value(&t2_attr2_tenant_value, false, &funs, t2_context).await?.value,
        "t2_attr2_tenant_value_modify"
    );
    assert_eq!(
        IamAttrServ::get_account_attr_value(&t2_a1_attr3_global_value, false, &funs, t2_context).await?.value,
        "t2_a1_attr3_global_value_modify"
    );

    info!("【test_cc_attr】 : test_multi_level : Delete Account Attr value By tenant_context");
    IamAttrServ::delete_account_attr_value(&sys_attr2_global_value, &funs, t2_context).await?;
    IamAttrServ::delete_account_attr_value(&t2_attr1_value, &funs, t2_context).await?;
    IamAttrServ::delete_account_attr_value(&t2_attr2_tenant_value, &funs, t2_context).await?;
    IamAttrServ::delete_account_attr_value(&t2_a1_attr3_global_value, &funs, t2_context).await?;

    info!("【test_cc_attr】 : test_multi_level : Delete Account Attr By tenant_context");
    assert!(IamAttrServ::delete_account_attr(&sys_attr1_id, &funs, t2_context).await.is_err());
    assert!(IamAttrServ::delete_account_attr(&sys_attr2_global_id, &funs, t2_context).await.is_err());
    IamAttrServ::delete_account_attr(&t2_attr1_id, &funs, t2_context).await?;
    IamAttrServ::delete_account_attr(&t2_attr2_tenant_id, &funs, t2_context).await?;
    IamAttrServ::delete_account_attr(&t2_a1_attr1_id, &funs, t2_context).await?;
    IamAttrServ::delete_account_attr(&t2_a1_attr2_app_id, &funs, t2_context).await?;
    IamAttrServ::delete_account_attr(&t2_a1_attr2_tenant_id, &funs, t2_context).await?;
    IamAttrServ::delete_account_attr(&t2_a1_attr3_global_id, &funs, t2_context).await?;
    IamAttrServ::delete_account_attr(&t2_a2_attr1_id, &funs, t2_context).await?;

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

    let (
        sys_attr1_id,
        sys_attr2_global_id,
        t1_attr1_id,
        t2_attr1_id,
        t2_attr2_tenant_id,
        t2_a1_attr1_id,
        t2_a1_attr2_app_id,
        t2_a1_attr2_tenant_id,
        t2_a1_attr3_global_id,
        t2_a2_attr1_id,
    ) = test_multi_level_add(sys_context, t1_context, t2_context, t2_a1_context, t2_a2_context, &funs).await?;

    info!("【test_cc_attr】 : test_multi_level : Modify Account Attr By app_context");
    assert!(IamAttrServ::modify_account_attr(&sys_attr1_id, &mut package_test_attr_modify_req("sys_attr1_modify"), &funs, t2_a1_context).await.is_err());
    assert!(IamAttrServ::modify_account_attr(&sys_attr2_global_id, &mut package_test_attr_modify_req("sys_attr2_global_modify"), &funs, t2_a1_context).await.is_err());
    assert!(IamAttrServ::modify_account_attr(&t2_attr1_id, &mut package_test_attr_modify_req("t2_attr1_modify"), &funs, t2_a1_context).await.is_err());
    assert!(IamAttrServ::modify_account_attr(&t2_attr2_tenant_id, &mut package_test_attr_modify_req("t2_attr2_tenant_modify"), &funs, t2_a1_context).await.is_err());
    IamAttrServ::modify_account_attr(&t2_a1_attr1_id, &mut package_test_attr_modify_req("t2_a1_attr1_modify"), &funs, t2_a1_context).await?;
    IamAttrServ::modify_account_attr(&t2_a1_attr2_app_id, &mut package_test_attr_modify_req("t2_a1_attr2_app_modify"), &funs, t2_a1_context).await?;
    IamAttrServ::modify_account_attr(&t2_a1_attr2_tenant_id, &mut package_test_attr_modify_req("t2_a1_attr2_tenant_modify"), &funs, t2_a1_context).await?;
    IamAttrServ::modify_account_attr(&t2_a1_attr3_global_id, &mut package_test_attr_modify_req("t2_a1_attr3_global_modify"), &funs, t2_a1_context).await?;
    assert!(IamAttrServ::modify_account_attr(&t2_a2_attr1_id, &mut package_test_attr_modify_req("t2_a2_attr1_modify"), &funs, t2_a1_context).await.is_err());

    info!("【test_cc_attr】 : test_multi_level : Get Account Attr By app_context");
    assert!(IamAttrServ::get_account_attr(&sys_attr1_id, &funs, t2_a1_context).await.is_err());
    assert_eq!(IamAttrServ::get_account_attr(&sys_attr2_global_id, &funs, t2_a1_context).await?.name, "sys_attr2_global");
    assert!(IamAttrServ::get_account_attr(&t2_attr1_id, &funs, t2_a1_context).await.is_err());
    assert_eq!(IamAttrServ::get_account_attr(&t2_attr2_tenant_id, &funs, t2_a1_context).await?.name, "t2_attr2_tenant");
    assert_eq!(IamAttrServ::get_account_attr(&t2_a1_attr1_id, &funs, t2_a1_context).await?.name, "t2_a1_attr1_modify");
    assert_eq!(
        IamAttrServ::get_account_attr(&t2_a1_attr2_app_id, &funs, t2_a1_context).await?.name,
        "t2_a1_attr2_app_modify"
    );
    assert_eq!(
        IamAttrServ::get_account_attr(&t2_a1_attr2_tenant_id, &funs, t2_a1_context).await?.name,
        "t2_a1_attr2_tenant_modify"
    );
    assert_eq!(
        IamAttrServ::get_account_attr(&t2_a1_attr3_global_id, &funs, t2_a1_context).await?.name,
        "t2_a1_attr3_global_modify"
    );
    assert!(IamAttrServ::get_account_attr(&t2_a2_attr1_id, &funs, t2_a1_context).await.is_err());

    info!("【test_cc_attr】 : test_multi_level : Find Account Attrs By app_context");
    assert_eq!(IamAttrServ::find_account_attrs(&funs, t2_a1_context).await?.len(), 6);

    info!("【test_cc_attr】 : test_multi_level : Add Account Attr value By app_context");
    assert!(IamAttrServ::add_account_attr_value("sys_attr1_value".to_string(), &sys_attr1_id, &t2_a1_context.owner, &funs, t2_a1_context).await.is_err());
    let sys_attr2_global_value =
        IamAttrServ::add_account_attr_value("sys_attr2_global_value".to_string(), &sys_attr2_global_id, &t2_a1_context.owner, &funs, t2_a1_context).await?;
    assert!(IamAttrServ::add_account_attr_value("t2_attr1_value".to_string(), &t2_attr1_id, &t2_a1_context.owner, &funs, t2_a1_context).await.is_err());
    let t2_attr2_tenant_value = IamAttrServ::add_account_attr_value("t2_attr2_tenant_value".to_string(), &t2_attr2_tenant_id, &t2_a1_context.owner, &funs, t2_a1_context).await?;
    let t2_a1_attr1_value = IamAttrServ::add_account_attr_value("t2_a1_attr1_value".to_string(), &t2_a1_attr1_id, &t2_a1_context.owner, &funs, t2_a1_context).await?;
    let t2_a1_attr2_app_value = IamAttrServ::add_account_attr_value("t2_a1_attr2_app_value".to_string(), &t2_a1_attr2_app_id, &t2_a1_context.owner, &funs, t2_a1_context).await?;
    let t2_a1_attr3_global_value =
        IamAttrServ::add_account_attr_value("t2_a1_attr3_global_value".to_string(), &t2_a1_attr3_global_id, &t2_a1_context.owner, &funs, t2_a1_context).await?;

    info!("【test_cc_attr】 : test_multi_level : Modify Account Attr value By app_context");
    IamAttrServ::modify_account_attr_value(&sys_attr2_global_value, "sys_attr2_global_value_modify".to_string(), &funs, t2_a1_context).await?;
    IamAttrServ::modify_account_attr_value(&t2_attr2_tenant_value, "t2_attr2_tenant_value_modify".to_string(), &funs, t2_a1_context).await?;
    IamAttrServ::modify_account_attr_value(&t2_a1_attr1_value, "t2_a1_attr1_value_modify".to_string(), &funs, t2_a1_context).await?;
    IamAttrServ::modify_account_attr_value(&t2_a1_attr2_app_value, "t2_a1_attr2_app_value_modify".to_string(), &funs, t2_a1_context).await?;
    IamAttrServ::modify_account_attr_value(&t2_a1_attr3_global_value, "t2_a1_attr3_global_value_modify".to_string(), &funs, t2_a1_context).await?;

    info!("【test_cc_attr】 : test_multi_level : Get Account Attr value By app_context");
    assert_eq!(
        IamAttrServ::get_account_attr_value(&sys_attr2_global_value, false, &funs, t2_a1_context).await?.value,
        "sys_attr2_global_value_modify"
    );
    assert_eq!(
        IamAttrServ::get_account_attr_value(&t2_attr2_tenant_value, false, &funs, t2_a1_context).await?.value,
        "t2_attr2_tenant_value_modify"
    );
    assert_eq!(
        IamAttrServ::get_account_attr_value(&t2_a1_attr1_value, false, &funs, t2_a1_context).await?.value,
        "t2_a1_attr1_value_modify"
    );
    assert_eq!(
        IamAttrServ::get_account_attr_value(&t2_a1_attr2_app_value, false, &funs, t2_a1_context).await?.value,
        "t2_a1_attr2_app_value_modify"
    );
    assert_eq!(
        IamAttrServ::get_account_attr_value(&t2_a1_attr3_global_value, false, &funs, t2_a1_context).await?.value,
        "t2_a1_attr3_global_value_modify"
    );

    info!("【test_cc_attr】 : test_multi_level : Delete Account Attr value By app_context");
    IamAttrServ::delete_account_attr_value(&sys_attr2_global_value, &funs, t2_a1_context).await?;
    IamAttrServ::delete_account_attr_value(&t2_attr2_tenant_value, &funs, t2_a1_context).await?;
    IamAttrServ::delete_account_attr_value(&t2_a1_attr1_value, &funs, t2_a1_context).await?;
    IamAttrServ::delete_account_attr_value(&t2_a1_attr2_app_value, &funs, t2_a1_context).await?;
    IamAttrServ::delete_account_attr_value(&t2_a1_attr3_global_value, &funs, t2_a1_context).await?;

    info!("【test_cc_attr】 : test_multi_level : Delete Account Attr By app_context");
    assert!(IamAttrServ::delete_account_attr(&sys_attr1_id, &funs, t2_a1_context).await.is_err());
    assert!(IamAttrServ::delete_account_attr(&sys_attr2_global_id, &funs, t2_a1_context).await.is_err());
    assert!(IamAttrServ::delete_account_attr(&t2_attr1_id, &funs, t2_a1_context).await.is_err());
    assert!(IamAttrServ::delete_account_attr(&t2_attr2_tenant_id, &funs, t2_a1_context).await.is_err());
    IamAttrServ::delete_account_attr(&t2_a1_attr1_id, &funs, t2_a1_context).await?;
    IamAttrServ::delete_account_attr(&t2_a1_attr2_app_id, &funs, t2_a1_context).await?;
    IamAttrServ::delete_account_attr(&t2_a1_attr2_tenant_id, &funs, t2_a1_context).await?;
    IamAttrServ::delete_account_attr(&t2_a1_attr3_global_id, &funs, t2_a1_context).await?;
    assert!(IamAttrServ::delete_account_attr(&t2_a2_attr1_id, &funs, t2_a1_context).await.is_err());

    funs.rollback().await?;
    Ok(())
}

fn package_test_attr_add_req(name: &str, scope_level: Option<RbumScopeLevelKind>) -> IamKindAttrAddReq {
    IamKindAttrAddReq {
        name: TrimString(name.to_string()),
        label: "".to_string(),
        note: None,
        sort: None,
        main_column: None,
        position: None,
        capacity: None,
        overload: None,
        idx: None,
        data_type: RbumDataTypeKind::String,
        widget_type: RbumWidgetTypeKind::Input,
        default_value: None,
        options: None,
        required: None,
        min_length: None,
        max_length: None,
        action: None,
        ext: None,
        scope_level,
    }
}

fn package_test_attr_modify_req(name: &str) -> RbumKindAttrModifyReq {
    RbumKindAttrModifyReq {
        name: Some(TrimString(name.to_string())),
        label: None,
        note: None,
        sort: None,
        main_column: None,
        position: None,
        capacity: None,
        overload: None,
        idx: None,
        data_type: None,
        widget_type: None,
        default_value: None,
        options: None,
        required: None,
        min_length: None,
        max_length: None,
        action: None,
        ext: None,
        scope_level: None,
    }
}
