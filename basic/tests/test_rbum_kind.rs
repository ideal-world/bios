use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;
use tardis::TardisFuns;

use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::dto::rbum_kind_attr_dto::{RbumKindAttrAddReq, RbumKindAttrModifyReq};
use bios_basic::rbum::dto::rbum_kind_dto::{RbumKindAddReq, RbumKindModifyReq};
use bios_basic::rbum::rbum_enumeration::{RbumDataTypeKind, RbumScopeLevelKind, RbumWidgetTypeKind};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_kind_serv::{RbumKindAttrServ, RbumKindServ};

pub async fn test(context: &TardisContext) -> TardisResult<()> {
    test_rbum_kind(context).await?;
    test_rbum_kind_attr(context).await?;
    Ok(())
}

async fn test_rbum_kind(context: &TardisContext) -> TardisResult<()> {
    let mut funs = TardisFuns::inst_with_db_conn("");
    funs.begin().await?;

    info!("【test_rbum_kind】 : Test Add : RbumKindServ::add_rbum");
    let id = RbumKindServ::add_rbum(
        &mut RbumKindAddReq {
            code: TrimString("db".to_string()),
            name: TrimString("关系型数据库".to_string()),
            note: None,
            icon: None,
            sort: None,
            ext_table_name: Some("reldb_mgr".to_string()),
            scope_level: RbumScopeLevelKind::L2,
        },
        &funs,
        context,
    )
    .await?;

    info!("【test_rbum_kind】 : Test Get : RbumKindServ::get_rbum");
    let rbum = RbumKindServ::get_rbum(&id, &RbumBasicFilterReq::default(), &funs, context).await?;
    assert_eq!(rbum.id, id);
    assert_eq!(rbum.code, "db");
    assert_eq!(rbum.name, "关系型数据库");

    info!("【test_rbum_kind】 : Test Modify : RbumKindServ::modify_rbum");
    RbumKindServ::modify_rbum(
        &id,
        &mut RbumKindModifyReq {
            name: Some(TrimString("关系型数据库_new".to_string())),
            note: None,
            icon: None,
            sort: None,
            ext_table_name: None,
            scope_level: None,
        },
        &funs,
        context,
    )
    .await?;

    info!("【test_rbum_kind】 : Test Find : RbumKindServ::paginate_rbums");
    let rbums = RbumKindServ::paginate_rbums(
        &RbumBasicFilterReq {
            scope_level: Some(RbumScopeLevelKind::L2),
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
    assert_eq!(rbums.records.get(0).unwrap().name, "关系型数据库_new");

    info!("【test_rbum_kind】 : Test Delete : RbumKindServ::delete_rbum");
    RbumKindServ::delete_rbum(&id, &funs, context).await?;
    assert!(RbumKindServ::get_rbum(&id, &RbumBasicFilterReq::default(), &funs, context).await.is_err());

    funs.rollback().await?;

    Ok(())
}

async fn test_rbum_kind_attr(context: &TardisContext) -> TardisResult<()> {
    let mut funs = TardisFuns::inst_with_db_conn("");
    funs.begin().await?;

    info!("【test_rbum_kind_attr】 : Prepare Kind : RbumKindServ::add_rbum");
    let kind_id = RbumKindServ::add_rbum(
        &mut RbumKindAddReq {
            code: TrimString("reldb".to_string()),
            name: TrimString("关系型数据库".to_string()),
            note: None,
            icon: None,
            sort: None,
            ext_table_name: None,
            scope_level: RbumScopeLevelKind::L2,
        },
        &funs,
        context,
    )
    .await?;

    // -----------------------------------

    info!("【test_rbum_kind_attr】 : Test Add : RbumKindAttrServ::add_rbum");
    assert!(RbumKindAttrServ::add_rbum(
        &mut RbumKindAttrAddReq {
            name: TrimString("db_type".to_string()),
            label: "数据库类型".to_string(),
            data_type: RbumDataTypeKind::String,
            widget_type: RbumWidgetTypeKind::InputTxt,
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
            rel_rbum_kind_id: "".to_string(),
            scope_level: RbumScopeLevelKind::L2,
        },
        &funs,
        context,
    )
    .await
    .is_err());

    assert!(RbumKindAttrServ::add_rbum(
        &mut RbumKindAttrAddReq {
            name: TrimString("db_type".to_string()),
            label: "数据库类型".to_string(),
            data_type: RbumDataTypeKind::String,
            widget_type: RbumWidgetTypeKind::InputTxt,
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
            rel_rbum_kind_id: "11".to_string(),
            scope_level: RbumScopeLevelKind::L2,
        },
        &funs,
        context,
    )
    .await
    .is_err());

    let kind_attr_id = RbumKindAttrServ::add_rbum(
        &mut RbumKindAttrAddReq {
            name: TrimString("db_type".to_string()),
            label: "数据库类型".to_string(),
            data_type: RbumDataTypeKind::String,
            widget_type: RbumWidgetTypeKind::InputTxt,
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
            rel_rbum_kind_id: kind_id.to_string(),
            scope_level: RbumScopeLevelKind::L2,
        },
        &funs,
        context,
    )
    .await?;

    info!("【test_rbum_kind_attr】 : Test Get : RbumKindAttrServ::get_rbum");
    let rbum = RbumKindAttrServ::get_rbum(&kind_attr_id, &RbumBasicFilterReq::default(), &funs, context).await?;
    assert_eq!(rbum.id, kind_attr_id);
    assert_eq!(rbum.name, "db_type");
    assert_eq!(rbum.label, "数据库类型");
    assert_eq!(rbum.data_type, RbumDataTypeKind::String);
    assert_eq!(rbum.widget_type, RbumWidgetTypeKind::InputTxt);
    assert!(!rbum.overload);

    info!("【test_rbum_kind_attr】 : Test Modify : RbumKindAttrServ::modify_rbum");
    assert!(RbumKindAttrServ::modify_rbum(
        "111",
        &mut RbumKindAttrModifyReq {
            name: None,
            label: None,
            data_type: None,
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
            scope_level: None
        },
        &funs,
        context
    )
    .await
    .is_err());

    RbumKindAttrServ::modify_rbum(
        &kind_attr_id,
        &mut RbumKindAttrModifyReq {
            name: None,
            label: None,
            data_type: None,
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
            scope_level: None,
        },
        &funs,
        context,
    )
    .await?;

    info!("【test_rbum_kind_attr】 : Test Find : RbumKindAttrServ::paginate_rbums");
    let rbums = RbumKindAttrServ::paginate_rbums(&RbumBasicFilterReq::default(), 1, 10, None, None, &funs, context).await?;
    assert_eq!(rbums.page_number, 1);
    assert_eq!(rbums.page_size, 10);
    assert_eq!(rbums.total_size, 1);
    assert!(rbums.records.get(0).unwrap().overload);

    info!("【test_rbum_kind_attr】 : Test Delete : RbumKindAttrServ::delete_rbum");
    RbumKindAttrServ::delete_rbum(&kind_attr_id, &funs, context).await?;
    assert!(RbumKindAttrServ::get_rbum(&kind_attr_id, &RbumBasicFilterReq::default(), &funs, context).await.is_err());

    funs.rollback().await?;

    Ok(())
}
