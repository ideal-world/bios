use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::TardisFuns;

use bios_basic::rbum::dto::filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::dto::rbum_kind_attr_dto::{RbumKindAttrAddReq, RbumKindAttrModifyReq};
use bios_basic::rbum::dto::rbum_kind_dto::{RbumKindAddReq, RbumKindModifyReq};
use bios_basic::rbum::enumeration::{RbumDataTypeKind, RbumScopeKind, RbumWidgetKind};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_kind_serv::{RbumKindAttrServ, RbumKindServ};

pub async fn test() -> TardisResult<()> {
    test_rbum_kind().await?;
    test_rbum_kind_attr().await?;
    Ok(())
}

async fn test_rbum_kind() -> TardisResult<()> {
    let context = bios_basic::rbum::initializer::get_sys_admin_context().await?;
    let mut tx = TardisFuns::reldb().conn();
    tx.begin().await?;

    // Test Add
    let id = RbumKindServ::add_rbum(
        &mut RbumKindAddReq {
            uri_scheme: TrimString("db".to_string()),
            name: TrimString("关系型数据库".to_string()),
            note: None,
            icon: None,
            sort: None,
            ext_table_name: Some("reldb_mgr".to_string()),
            scope_kind: None,
        },
        &tx,
        &context,
    )
    .await?;

    // Test Get
    let rbum = RbumKindServ::get_rbum(&id, &RbumBasicFilterReq::default(), &tx, &context).await?;
    assert_eq!(rbum.id, id);
    assert_eq!(rbum.uri_scheme, "db");
    assert_eq!(rbum.name, "关系型数据库");

    // Test Modify
    RbumKindServ::modify_rbum(
        &id,
        &mut RbumKindModifyReq {
            uri_scheme: Some(TrimString("reldb".to_string())),
            name: None,
            note: None,
            icon: None,
            sort: None,
            ext_table_name: None,
            scope_kind: None,
        },
        &tx,
        &context,
    )
    .await?;

    // Test Find
    let rbums = RbumKindServ::paginate_rbums(
        &RbumBasicFilterReq {
            scope_kind: Some(RbumScopeKind::App),
            ..Default::default()
        },
        1,
        10,
        None,
        &tx,
        &context,
    )
    .await?;
    assert_eq!(rbums.page_number, 1);
    assert_eq!(rbums.page_size, 10);
    assert_eq!(rbums.total_size, 1);
    assert_eq!(rbums.records.get(0).unwrap().uri_scheme, "reldb");

    // Test Delete
    RbumKindServ::delete_rbum(&id, &tx, &context).await?;
    assert!(RbumKindServ::get_rbum(&id, &RbumBasicFilterReq::default(), &tx, &context).await.is_err());

    tx.rollback().await?;

    Ok(())
}

async fn test_rbum_kind_attr() -> TardisResult<()> {
    let context = bios_basic::rbum::initializer::get_sys_admin_context().await?;
    let mut tx = TardisFuns::reldb().conn();
    tx.begin().await?;

    // Prepare Kind
    let kind_id = RbumKindServ::add_rbum(
        &mut RbumKindAddReq {
            uri_scheme: TrimString("reldb".to_string()),
            name:TrimString( "关系型数据库".to_string()),
            note: None,
            icon: None,
            sort: None,
            ext_table_name: None,
            scope_kind: None,
        },
        &tx,
        &context,
    )
    .await?;

    // -----------------------------------

    // Test Add
    assert!(RbumKindAttrServ::add_rbum(
        &mut RbumKindAttrAddReq {
            name: TrimString("db_type".to_string()),
            label: "数据库类型".to_string(),
            data_type_kind: RbumDataTypeKind::String,
            widget_type: RbumWidgetKind::InputTxt,
            note: None,
            sort: None,
            main_column: None,
            position: None,
            capacity: None,
            overload: None,
            default_value: None,
            options: None,
            required: None,
            min_length: None,
            max_length: None,
            action: None,
            scope_kind: None,
            rel_rbum_kind_id: "".to_string(),
        },
        &tx,
        &context,
    )
    .await
    .is_err());

    assert!(RbumKindAttrServ::add_rbum(
        &mut RbumKindAttrAddReq {
            name: TrimString("db_type".to_string()),
            label: "数据库类型".to_string(),
            data_type_kind: RbumDataTypeKind::String,
            widget_type: RbumWidgetKind::InputTxt,
            note: None,
            sort: None,
            main_column: None,
            position: None,
            capacity: None,
            overload: None,
            default_value: None,
            options: None,
            required: None,
            min_length: None,
            max_length: None,
            action: None,
            scope_kind: None,
            rel_rbum_kind_id: "11".to_string(),
        },
        &tx,
        &context,
    )
    .await
    .is_err());

    let kind_attr_id = RbumKindAttrServ::add_rbum(
        &mut RbumKindAttrAddReq {
            name: TrimString("db_type".to_string()),
            label: "数据库类型".to_string(),
            data_type_kind: RbumDataTypeKind::String,
            widget_type: RbumWidgetKind::InputTxt,
            note: None,
            sort: None,
            main_column: None,
            position: None,
            capacity: None,
            overload: None,
            default_value: None,
            options: None,
            required: None,
            min_length: None,
            max_length: None,
            action: None,
            scope_kind: None,
            rel_rbum_kind_id: kind_id.to_string(),
        },
        &tx,
        &context,
    )
    .await?;

    // Test Get
    let rbum = RbumKindAttrServ::get_rbum(&kind_attr_id, &RbumBasicFilterReq::default(), &tx, &context).await?;
    assert_eq!(rbum.id, kind_attr_id);
    assert_eq!(rbum.name, "db_type");
    assert_eq!(rbum.label, "数据库类型");
    assert_eq!(rbum.data_type_kind, RbumDataTypeKind::String.to_string());
    assert_eq!(rbum.widget_type, RbumWidgetKind::InputTxt.to_string());
    assert!(!rbum.overload);

    // Test Modify
    assert!(RbumKindAttrServ::modify_rbum(
        "111",
        &mut RbumKindAttrModifyReq {
            name: None,
            label: None,
            data_type_kind: None,
            widget_type: None,
            note: None,
            sort: None,
            main_column: None,
            position: None,
            capacity: None,
            overload: Some(true),
            default_value: None,
            options: None,
            required: None,
            min_length: None,
            max_length: None,
            action: None,
            scope_kind: None
        },
        &tx,
        &context
    )
    .await
    .is_err());

    RbumKindAttrServ::modify_rbum(
        &kind_attr_id,
        &mut RbumKindAttrModifyReq {
            name: None,
            label: None,
            data_type_kind: None,
            widget_type: None,
            note: None,
            sort: None,
            main_column: None,
            position: None,
            capacity: None,
            overload: Some(true),
            default_value: None,
            options: None,
            required: None,
            min_length: None,
            max_length: None,
            action: None,
            scope_kind: None,
        },
        &tx,
        &context,
    )
    .await?;

    // Test Find
    let rbums = RbumKindAttrServ::paginate_rbums(&RbumBasicFilterReq::default(), 1, 10, None, &tx, &context).await?;
    assert_eq!(rbums.page_number, 1);
    assert_eq!(rbums.page_size, 10);
    assert_eq!(rbums.total_size, 1);
    assert!(rbums.records.get(0).unwrap().overload);

    // Test Delete
    RbumKindAttrServ::delete_rbum(&kind_attr_id, &tx, &context).await?;
    assert!(RbumKindAttrServ::get_rbum(&kind_attr_id, &RbumBasicFilterReq::default(), &tx, &context).await.is_err());

    tx.rollback().await?;

    Ok(())
}
