use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;
use tardis::TardisFuns;

use bios_basic::rbum::dto::rbum_domain_dto::RbumDomainAddReq;
use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::dto::rbum_item_attr_dto::{RbumItemAttrAddReq, RbumItemAttrModifyReq};
use bios_basic::rbum::dto::rbum_item_dto::{RbumItemAddReq, RbumItemModifyReq};
use bios_basic::rbum::dto::rbum_kind_attr_dto::RbumKindAttrAddReq;
use bios_basic::rbum::dto::rbum_kind_dto::RbumKindAddReq;
use bios_basic::rbum::rbum_enumeration::{RbumDataTypeKind, RbumScopeLevelKind, RbumWidgetTypeKind};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_domain_serv::RbumDomainServ;
use bios_basic::rbum::serv::rbum_item_serv::{RbumItemAttrServ, RbumItemServ};
use bios_basic::rbum::serv::rbum_kind_serv::{RbumKindAttrServ, RbumKindServ};

pub async fn test(context: &TardisContext) -> TardisResult<()> {
    test_rbum_item(context).await?;
    test_rbum_item_attr(context).await?;
    Ok(())
}

async fn test_rbum_item(context: &TardisContext) -> TardisResult<()> {
    let mut funs = TardisFuns::inst_with_db_conn("");
    funs.begin().await?;

    info!("【test_rbum_item】 : Prepare Kind : RbumKindServ::add_rbum");
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

    info!("【test_rbum_item】 : Prepare Domain : RbumDomainServ::add_rbum");
    let domain_id = RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            code: TrimString("mysql_dev".to_string()),
            name: TrimString("Mysql测试集群".to_string()),
            note: Some("...".to_string()),
            icon: Some("...".to_string()),
            sort: None,
            scope_level: RbumScopeLevelKind::L2,
        },
        &funs,
        context,
    )
    .await?;

    // -----------------------------------

    info!("【test_rbum_item】 : Test Add : RbumItemServ::add_rbum");
    assert!(RbumItemServ::add_rbum(
        &mut RbumItemAddReq {
            id: None,
            code: None,
            name: TrimString("".to_string()),
            scope_level: RbumScopeLevelKind::L2,
            disabled: None,
            rel_rbum_kind_id: "".to_string(),
            rel_rbum_domain_id: domain_id.to_string()
        },
        &funs,
        context,
    )
    .await
    .is_err());

    assert!(RbumItemServ::add_rbum(
        &mut RbumItemAddReq {
            id: None,
            code: None,
            name: TrimString("".to_string()),
            scope_level: RbumScopeLevelKind::L2,
            disabled: None,
            rel_rbum_kind_id: kind_id.to_string(),
            rel_rbum_domain_id: "".to_string()
        },
        &funs,
        context,
    )
    .await
    .is_err());

    assert!(RbumItemServ::add_rbum(
        &mut RbumItemAddReq {
            id: None,
            code: None,
            name: TrimString("".to_string()),
            scope_level: RbumScopeLevelKind::L2,
            disabled: None,
            rel_rbum_kind_id: "123".to_string(),
            rel_rbum_domain_id: "123".to_string()
        },
        &funs,
        context,
    )
    .await
    .is_err());

    let id = RbumItemServ::add_rbum(
        &mut RbumItemAddReq {
            id: None,
            code: None,
            name: TrimString("实例1".to_string()),
            disabled: None,
            rel_rbum_kind_id: kind_id.to_string(),
            rel_rbum_domain_id: domain_id.to_string(),
            scope_level: RbumScopeLevelKind::L2,
        },
        &funs,
        context,
    )
    .await?;

    info!("【test_rbum_item】 : Test Get : RbumItemServ::get_rbum");
    let rbum = RbumItemServ::get_rbum(&id, &RbumBasicFilterReq::default(), &funs, context).await?;
    assert_eq!(rbum.id, id);
    assert_eq!(rbum.name, "实例1");

    info!("【test_rbum_item】 : Test Modify : RbumItemServ::modify_rbum");
    RbumItemServ::modify_rbum(
        &id,
        &mut RbumItemModifyReq {
            code: None,
            name: Some(TrimString("数据库实例1".to_string())),
            disabled: None,
            scope_level: None,
        },
        &funs,
        context,
    )
    .await?;

    info!("【test_rbum_item】 : Test Find : RbumItemServ::paginate_rbums");
    let rbums = RbumItemServ::paginate_rbums(
        &RbumBasicFilterReq {
            scope_level: Some(RbumScopeLevelKind::L2),
            name: Some("%据库%".to_string()),
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
    assert_eq!(rbums.records.get(0).unwrap().name, "数据库实例1");

    info!("【test_rbum_item】 : Test Delete : RbumItemServ::delete_rbum");
    RbumItemServ::delete_rbum(&id, &funs, context).await?;
    assert!(RbumItemServ::get_rbum(&id, &RbumBasicFilterReq::default(), &funs, context).await.is_err());

    funs.rollback().await?;

    Ok(())
}

async fn test_rbum_item_attr(context: &TardisContext) -> TardisResult<()> {
    let mut funs = TardisFuns::inst_with_db_conn("");
    funs.begin().await?;

    info!("【test_rbum_item_attr】 : Prepare Kind : RbumKindServ::add_rbum");
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

    info!("【test_rbum_item_attr】 : Prepare Kind Attr : RbumKindAttrServ::add_rbum");
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

    info!("【test_rbum_item_attr】 : Prepare Domain : RbumDomainServ::add_rbum");
    let domain_id = RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            code: TrimString("mysql_dev".to_string()),
            name: TrimString("Mysql测试集群".to_string()),
            note: Some("...".to_string()),
            icon: Some("...".to_string()),
            sort: None,
            scope_level: RbumScopeLevelKind::L2,
        },
        &funs,
        context,
    )
    .await?;

    let item_id = RbumItemServ::add_rbum(
        &mut RbumItemAddReq {
            id: None,
            code: None,
            name: TrimString("实例1".to_string()),
            disabled: None,
            rel_rbum_kind_id: kind_id.to_string(),
            rel_rbum_domain_id: domain_id.to_string(),
            scope_level: RbumScopeLevelKind::L2,
        },
        &funs,
        context,
    )
    .await?;

    // -----------------------------------

    info!("【test_rbum_item_attr】 : Test Add : RbumItemAttrServ::add_rbum");
    assert!(RbumItemAttrServ::add_rbum(
        &mut RbumItemAttrAddReq {
            value: "数据1".to_string(),
            rel_rbum_item_id: "".to_string(),
            rel_rbum_kind_attr_id: "".to_string()
        },
        &funs,
        context,
    )
    .await
    .is_err());

    assert!(RbumItemAttrServ::add_rbum(
        &mut RbumItemAttrAddReq {
            value: "数据1".to_string(),
            rel_rbum_item_id: item_id.to_string(),
            rel_rbum_kind_attr_id: "".to_string()
        },
        &funs,
        context,
    )
    .await
    .is_err());

    assert!(RbumItemAttrServ::add_rbum(
        &mut RbumItemAttrAddReq {
            value: "数据1".to_string(),
            rel_rbum_item_id: "".to_string(),
            rel_rbum_kind_attr_id: kind_attr_id.to_string()
        },
        &funs,
        context,
    )
    .await
    .is_err());

    let item_attr_id = RbumItemAttrServ::add_rbum(
        &mut RbumItemAttrAddReq {
            value: "数据1".to_string(),
            rel_rbum_item_id: item_id.to_string(),
            rel_rbum_kind_attr_id: kind_attr_id.to_string(),
        },
        &funs,
        context,
    )
    .await?;

    info!("【test_rbum_item_attr】 : Test Get : RbumItemAttrServ::get_rbum");
    let rbum = RbumItemAttrServ::get_rbum(&item_attr_id, &RbumBasicFilterReq::default(), &funs, context).await?;
    assert_eq!(rbum.id, item_attr_id);
    assert_eq!(rbum.value, "数据1");
    assert_eq!(rbum.rel_rbum_item_id, item_id.to_string());
    assert_eq!(rbum.rel_rbum_item_name, "实例1".to_string());
    assert_eq!(rbum.rel_rbum_kind_attr_id, kind_attr_id.to_string());
    assert_eq!(rbum.rel_rbum_kind_attr_name, "db_type".to_string());

    info!("【test_rbum_item_attr】 : Test Modify : RbumItemAttrServ::modify_rbum");
    assert!(RbumItemAttrServ::modify_rbum("111", &mut RbumItemAttrModifyReq { value: "数据2".to_string() }, &funs, context).await.is_err());
    RbumItemAttrServ::modify_rbum(&item_attr_id, &mut RbumItemAttrModifyReq { value: "数据3".to_string() }, &funs, context).await?;

    info!("【test_rbum_item_attr】 : Test Find : RbumItemAttrServ::paginate_rbums");
    let rbums = RbumItemAttrServ::paginate_rbums(&RbumBasicFilterReq::default(), 1, 10, None, None, &funs, context).await?;
    assert_eq!(rbums.page_number, 1);
    assert_eq!(rbums.page_size, 10);
    assert_eq!(rbums.total_size, 1);
    assert_eq!(rbums.records.get(0).unwrap().value, "数据3");

    info!("【test_rbum_item_attr】 : Test Delete : RbumItemAttrServ::delete_rbum");
    RbumItemAttrServ::delete_rbum(&item_attr_id, &funs, context).await?;
    assert!(RbumItemAttrServ::get_rbum(&item_attr_id, &RbumBasicFilterReq::default(), &funs, context).await.is_err());

    funs.rollback().await?;

    Ok(())
}
