use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;

use bios_basic::rbum::dto::rbum_kind_attr_dto::RbumKindAttrModifyReq;
use bios_basic::rbum::rbum_enumeration::{RbumDataTypeKind, RbumWidgetTypeKind};
use bios_iam::basic::dto::iam_attr_dto::IamKindAttrAddReq;
use bios_iam::basic::serv::iam_attr_serv::IamAttrServ;
use bios_iam::iam_constants;
use bios_iam::iam_constants::{RBUM_SCOPE_LEVEL_APP, RBUM_SCOPE_LEVEL_TENANT};

pub async fn test(tenant1_context: &TardisContext, app1_context: &TardisContext, app2_context: &TardisContext, tenant_another_context: &TardisContext) -> TardisResult<()> {
    let mut funs = iam_constants::get_tardis_inst();
    funs.begin().await?;

    info!("【test_ct_ca_attr】 : Add Account Attr");
    let tenant1_attr1 = IamAttrServ::add_account_attr(&package_test_attr_add_req("tenant1_attr1"), RBUM_SCOPE_LEVEL_TENANT, &funs, tenant1_context).await?;
    let app1_attr1 = IamAttrServ::add_account_attr(&package_test_attr_add_req("app1_attr1"), RBUM_SCOPE_LEVEL_APP, &funs, app1_context).await?;
    let _app2_attr1 = IamAttrServ::add_account_attr(&package_test_attr_add_req("app2_attr1"), RBUM_SCOPE_LEVEL_APP, &funs, app2_context).await?;
    let _tenant_another_attr1 = IamAttrServ::add_account_attr(&package_test_attr_add_req("tenant_another_attr1"), RBUM_SCOPE_LEVEL_TENANT, &funs, tenant_another_context).await?;

    info!("【test_ct_ca_attr】 : Modify Account Attr, with err");
    assert!(IamAttrServ::modify_account_attr(&tenant1_attr1, &mut package_test_attr_modify_req("tenant1_attr1_modify"), &funs, tenant_another_context).await.is_err());
    assert!(IamAttrServ::modify_account_attr(&tenant1_attr1, &mut package_test_attr_modify_req("tenant1_attr1_modify"), &funs, app1_context).await.is_err());
    assert!(IamAttrServ::modify_account_attr(&app1_attr1, &mut package_test_attr_modify_req("app1_attr1_modify"), &funs, app2_context).await.is_err());
    info!("【test_ct_ca_attr】 : Modify Account Attr");
    IamAttrServ::modify_account_attr(&tenant1_attr1, &mut package_test_attr_modify_req("tenant1_attr1_modify"), &funs, tenant1_context).await?;
    IamAttrServ::modify_account_attr(&app1_attr1, &mut package_test_attr_modify_req("app1_attr1_modify"), &funs, app1_context).await?;
    IamAttrServ::modify_account_attr(&app1_attr1, &mut package_test_attr_modify_req("app1_attr1_modify"), &funs, tenant1_context).await?;

    info!("【test_ct_ca_attr】 : Get Account Attr, with err");
    assert!(IamAttrServ::get_account_attr(&tenant1_attr1, false, &funs, tenant_another_context).await.is_err());
    assert!(IamAttrServ::get_account_attr(&app1_attr1, false, &funs, tenant1_context).await.is_err());
    info!("【test_ct_ca_attr】 : Get Account Attr");
    assert_eq!(
        IamAttrServ::get_account_attr(&tenant1_attr1, false, &funs, tenant1_context).await?.name,
        "tenant1_attr1_modify"
    );
    assert_eq!(IamAttrServ::get_account_attr(&app1_attr1, false, &funs, app1_context).await?.name, "app1_attr1_modify");
    assert_eq!(
        IamAttrServ::get_account_attr(&tenant1_attr1, false, &funs, app1_context).await?.name,
        "tenant1_attr1_modify"
    );

    info!("【test_ct_ca_attr】 : Find Account Attrs");
    assert_eq!(IamAttrServ::find_account_attrs(false, &funs, tenant1_context).await?.len(), 1);
    // scope = 2 for attr created by app
    assert_eq!(IamAttrServ::find_account_attrs(true, &funs, tenant1_context).await?.len(), 1);
    assert_eq!(IamAttrServ::find_account_attrs(false, &funs, app1_context).await?.len(), 2);

    info!("【test_ct_ca_attr】 : Add Account Attr value, with err");
    assert!(IamAttrServ::add_account_attr_value("x".to_string(), &tenant1_attr1, &tenant_another_context.owner, &funs, tenant_another_context).await.is_err());
    assert!(IamAttrServ::add_account_attr_value("x".to_string(), &app1_attr1, &tenant_another_context.owner, &funs, tenant_another_context).await.is_err());
    assert!(IamAttrServ::add_account_attr_value("x".to_string(), &app1_attr1, &app2_context.owner, &funs, app2_context).await.is_err());
    assert!(IamAttrServ::add_account_attr_value("x".to_string(), &tenant1_attr1, &tenant1_context.owner, &funs, tenant_another_context).await.is_err());
    info!("【test_ct_ca_attr】 : Add Account Attr value");
    let tenant1_attr1_value_to_tenant1 =
        IamAttrServ::add_account_attr_value("tenant1_attr1_value_to_tenant1".to_string(), &tenant1_attr1, &tenant1_context.owner, &funs, tenant1_context).await?;
    let tenant1_attr1_value_to_app1 =
        IamAttrServ::add_account_attr_value("tenant1_attr1_value_to_app1".to_string(), &tenant1_attr1, &app1_context.owner, &funs, app1_context).await?;
    let _app1_attr1_value_to_app1 = IamAttrServ::add_account_attr_value("app1_attr1_value_to_app1".to_string(), &app1_attr1, &app1_context.owner, &funs, app1_context).await?;

    info!("【test_ct_ca_attr】 : Modify Account Attr value, with err");
    assert!(IamAttrServ::modify_account_attr_value(
        &tenant1_attr1_value_to_tenant1,
        "tenant1_attr1_value_to_tenant1_modify".to_string(),
        &funs,
        tenant_another_context
    )
    .await
    .is_err());
    assert!(IamAttrServ::modify_account_attr_value(&tenant1_attr1_value_to_tenant1, "tenant1_attr1_value_to_tenant1_modify".to_string(), &funs, app1_context).await.is_err());
    info!("【test_ct_ca_attr】 : Modify Account Attr value");
    IamAttrServ::modify_account_attr_value(&tenant1_attr1_value_to_tenant1, "tenant1_attr1_value_to_tenant1_modify".to_string(), &funs, tenant1_context).await?;
    IamAttrServ::modify_account_attr_value(&tenant1_attr1_value_to_app1, "tenant1_attr1_value_to_app1_modify".to_string(), &funs, app1_context).await?;
    IamAttrServ::modify_account_attr_value(&tenant1_attr1_value_to_app1, "tenant1_attr1_value_to_app1_modify".to_string(), &funs, tenant1_context).await?;

    info!("【test_ct_ca_attr】 : Get Account Attr value, with err");
    assert!(IamAttrServ::get_account_attr_value(&tenant1_attr1_value_to_tenant1, false, &funs, tenant_another_context).await.is_err());
    assert!(IamAttrServ::get_account_attr_value(&tenant1_attr1_value_to_tenant1, false, &funs, app1_context).await.is_err());
    assert!(IamAttrServ::get_account_attr_value(&tenant1_attr1_value_to_app1, false, &funs, tenant1_context).await.is_err());
    info!("【test_ct_ca_attr】 : Get Account Attr value");
    assert_eq!(
        IamAttrServ::get_account_attr_value(&tenant1_attr1_value_to_tenant1, false, &funs, tenant1_context).await?.value,
        "tenant1_attr1_value_to_tenant1_modify"
    );
    assert_eq!(
        IamAttrServ::get_account_attr_value(&tenant1_attr1_value_to_app1, false, &funs, app1_context).await?.value,
        "tenant1_attr1_value_to_app1_modify"
    );

    info!("【test_ct_ca_attr】 : Delete Account Attr, with err");
    assert!(IamAttrServ::delete_account_attr(&tenant1_attr1, &funs, tenant_another_context).await.is_err());
    assert!(IamAttrServ::delete_account_attr(&app1_attr1, &funs, tenant1_context).await.is_err());
    assert!(IamAttrServ::delete_account_attr(&tenant1_attr1, &funs, tenant1_context).await.is_err());
    info!("【test_ct_ca_attr】 : Delete Account Attr");
    IamAttrServ::delete_account_attr_value(&tenant1_attr1_value_to_tenant1, &funs, tenant1_context).await?;
    IamAttrServ::delete_account_attr_value(&tenant1_attr1_value_to_app1, &funs, app1_context).await?;
    IamAttrServ::delete_account_attr(&tenant1_attr1, &funs, tenant1_context).await?;

    funs.rollback().await?;

    Ok(())
}

fn package_test_attr_add_req(name: &str) -> IamKindAttrAddReq {
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
