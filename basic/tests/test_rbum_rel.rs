use std::collections::HashMap;
use std::time::Duration;

use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::chrono::Utc;
use tardis::log::info;
use tardis::TardisFuns;

use bios_basic::rbum::dto::filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::dto::rbum_domain_dto::RbumDomainAddReq;
use bios_basic::rbum::dto::rbum_item_dto::RbumItemAddReq;
use bios_basic::rbum::dto::rbum_kind_attr_dto::RbumKindAttrAddReq;
use bios_basic::rbum::dto::rbum_kind_dto::RbumKindAddReq;
use bios_basic::rbum::dto::rbum_rel_agg_dto::{RbumRelAggAddReq, RbumRelAttrAggAddReq, RbumRelEnvAggAddReq};
use bios_basic::rbum::dto::rbum_rel_attr_dto::{RbumRelAttrAddReq, RbumRelAttrModifyReq};
use bios_basic::rbum::dto::rbum_rel_dto::{RbumRelAddReq, RbumRelCheckReq, RbumRelModifyReq};
use bios_basic::rbum::dto::rbum_rel_env_dto::{RbumRelEnvAddReq, RbumRelEnvModifyReq};
use bios_basic::rbum::enumeration::{RbumDataTypeKind, RbumRelEnvKind, RbumWidgetKind};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_domain_serv::RbumDomainServ;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemServ;
use bios_basic::rbum::serv::rbum_kind_serv::{RbumKindAttrServ, RbumKindServ};
use bios_basic::rbum::serv::rbum_rel_serv::{RbumRelAttrServ, RbumRelEnvServ, RbumRelServ};

pub async fn test(context: &TardisContext) -> TardisResult<()> {
    test_rbum_rel(context).await?;
    test_rbum_rel_attr(context).await?;
    test_rbum_rel_env(context).await?;
    test_rbum_rel_use(context).await?;
    Ok(())
}

async fn test_rbum_rel(context: &TardisContext) -> TardisResult<()> {
    let mut tx = TardisFuns::reldb().conn();
    tx.begin().await?;

    info!("【test_rbum_rel】 : Prepare Kind : RbumKindServ::add_rbum");
    let kind_reldb_id = RbumKindServ::add_rbum(
        &mut RbumKindAddReq {
            uri_scheme: TrimString("reldb".to_string()),
            name: TrimString("关系型数据库".to_string()),
            note: None,
            icon: None,
            sort: None,
            ext_table_name: None,
            scope_level: 2,
        },
        &tx,
        context,
    )
    .await?;

    let kind_account_id = RbumKindServ::add_rbum(
        &mut RbumKindAddReq {
            uri_scheme: TrimString("account".to_string()),
            name: TrimString("Account".to_string()),
            note: None,
            icon: None,
            sort: None,
            ext_table_name: None,
            scope_level: 2,
        },
        &tx,
        context,
    )
    .await?;

    info!("【test_rbum_rel】 : Prepare Domain : RbumDomainServ::add_rbum");
    let domain_reldb_id = RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            uri_authority: TrimString("mysql_dev".to_string()),
            name: TrimString("Mysql测试集群".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: 2,
        },
        &tx,
        context,
    )
    .await?;

    let domain_iam_id = RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            uri_authority: TrimString("iam2".to_string()),
            name: TrimString("IAM2".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: 2,
        },
        &tx,
        context,
    )
    .await?;

    info!("【test_rbum_rel】 : Prepare Item : RbumItemServ::add_rbum");
    let item_reldb_inst1_id = RbumItemServ::add_rbum(
        &mut RbumItemAddReq {
            id: None,
            uri_path: None,
            name: TrimString("实例1".to_string()),
            icon: None,
            sort: None,
            scope_level: 2,
            disabled: None,
            rel_rbum_kind_id: kind_reldb_id.to_string(),
            rel_rbum_domain_id: domain_reldb_id.to_string(),
        },
        &tx,
        context,
    )
    .await?;

    let item_account_a1_id = RbumItemServ::add_rbum(
        &mut RbumItemAddReq {
            id: None,
            uri_path: None,
            name: TrimString("用户1".to_string()),
            icon: None,
            sort: None,
            scope_level: 2,
            disabled: None,
            rel_rbum_kind_id: kind_account_id.to_string(),
            rel_rbum_domain_id: domain_iam_id.to_string(),
        },
        &tx,
        context,
    )
    .await?;

    // -----------------------------------

    info!("【test_rbum_rel】 : Test Add : RbumRelServ::add_rbum");
    assert!(RbumRelServ::add_rbum(
        &mut RbumRelAddReq {
            tag: "bind".to_string(),
            from_rbum_item_id: "".to_string(),
            to_rbum_item_id: "".to_string(),
            to_scope_ids: "".to_string(),
            ext: None
        },
        &tx,
        context,
    )
    .await
    .is_err());

    assert!(RbumRelServ::add_rbum(
        &mut RbumRelAddReq {
            tag: "bind".to_string(),
            from_rbum_item_id: "".to_string(),
            to_rbum_item_id: "".to_string(),
            to_scope_ids: context.scope_ids.to_string(),
            ext: None
        },
        &tx,
        context,
    )
    .await
    .is_err());

    assert!(RbumRelServ::add_rbum(
        &mut RbumRelAddReq {
            tag: "bind".to_string(),
            from_rbum_item_id: item_reldb_inst1_id.to_string(),
            to_rbum_item_id: "".to_string(),
            to_scope_ids: context.scope_ids.to_string(),
            ext: None
        },
        &tx,
        context,
    )
    .await
    .is_err());

    assert!(RbumRelServ::add_rbum(
        &mut RbumRelAddReq {
            tag: "bind".to_string(),
            from_rbum_item_id: item_reldb_inst1_id.to_string(),
            to_rbum_item_id: "".to_string(),
            to_scope_ids: context.scope_ids.to_string(),
            ext: None
        },
        &tx,
        context,
    )
    .await
    .is_err());

    let id = RbumRelServ::add_rbum(
        &mut RbumRelAddReq {
            tag: "bind".to_string(),
            from_rbum_item_id: item_reldb_inst1_id.to_string(),
            to_rbum_item_id: item_account_a1_id.to_string(),
            to_scope_ids: context.scope_ids.to_string(),
            ext: None,
        },
        &tx,
        context,
    )
    .await?;

    info!("【test_rbum_rel】 : Test Get : RbumRelServ::get_rbum");
    let rbum = RbumRelServ::get_rbum(&id, &RbumBasicFilterReq::default(), &tx, context).await?;
    assert_eq!(rbum.id, id);
    assert_eq!(rbum.tag, "bind");
    assert_eq!(rbum.to_scope_ids, context.scope_ids);

    info!("【test_rbum_rel】 : Test Modify : RbumRelServ::modify_rbum");
    RbumRelServ::modify_rbum(
        &id,
        &mut RbumRelModifyReq {
            tag: Some("alloc".to_string()),
            ext: None,
        },
        &tx,
        context,
    )
    .await?;

    info!("【test_rbum_rel】 : Test Find : RbumRelServ::paginate_rbums");
    let rbums = RbumRelServ::paginate_rbums(&RbumBasicFilterReq::default(), 1, 10, None, None, &tx, context).await?;
    assert_eq!(rbums.page_number, 1);
    assert_eq!(rbums.page_size, 10);
    assert_eq!(rbums.total_size, 1);
    assert_eq!(rbums.records.get(0).unwrap().tag, "alloc");

    info!("【test_rbum_rel】 : Test Delete : RbumRelServ::delete_rbum");
    RbumRelServ::delete_rbum(&id, &tx, context).await?;
    assert!(RbumRelServ::get_rbum(&id, &RbumBasicFilterReq::default(), &tx, context).await.is_err());

    tx.rollback().await?;

    Ok(())
}

async fn test_rbum_rel_attr(context: &TardisContext) -> TardisResult<()> {
    let mut tx = TardisFuns::reldb().conn();
    tx.begin().await?;

    info!("【test_rbum_rel_attr】 : Prepare Kind : RbumKindServ::add_rbum");
    let kind_reldb_id = RbumKindServ::add_rbum(
        &mut RbumKindAddReq {
            uri_scheme: TrimString("reldb".to_string()),
            name: TrimString("关系型数据库".to_string()),
            note: None,
            icon: None,
            sort: None,
            ext_table_name: None,
            scope_level: 2,
        },
        &tx,
        context,
    )
    .await?;

    let kind_account_id = RbumKindServ::add_rbum(
        &mut RbumKindAddReq {
            uri_scheme: TrimString("account".to_string()),
            name: TrimString("Account".to_string()),
            note: None,
            icon: None,
            sort: None,
            ext_table_name: None,
            scope_level: 2,
        },
        &tx,
        context,
    )
    .await?;

    info!("【test_rbum_rel_attr】 : Prepare Kind Attr : RbumKindAttrServ::add_rbum");
    let kind_attr_db_type_id = RbumKindAttrServ::add_rbum(
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
            scope_level: 2,
            rel_rbum_kind_id: kind_reldb_id.to_string(),
        },
        &tx,
        context,
    )
    .await?;

    info!("【test_rbum_rel_attr】 : Prepare Domain : RbumDomainServ::add_rbum");
    let domain_reldb_id = RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            uri_authority: TrimString("mysql_dev".to_string()),
            name: TrimString("Mysql测试集群".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: 2,
        },
        &tx,
        context,
    )
    .await?;

    let domain_iam_id = RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            uri_authority: TrimString("iam2".to_string()),
            name: TrimString("IAM2".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: 2,
        },
        &tx,
        context,
    )
    .await?;

    info!("【test_rbum_rel_attr】 : Prepare Item : RbumItemServ::add_rbum");
    let item_reldb_inst1_id = RbumItemServ::add_rbum(
        &mut RbumItemAddReq {
            id: None,
            uri_path: None,
            name: TrimString("实例1".to_string()),
            icon: None,
            sort: None,
            scope_level: 2,
            disabled: None,
            rel_rbum_kind_id: kind_reldb_id.to_string(),
            rel_rbum_domain_id: domain_reldb_id.to_string(),
        },
        &tx,
        context,
    )
    .await?;

    let item_account_a1_id = RbumItemServ::add_rbum(
        &mut RbumItemAddReq {
            id: None,
            uri_path: None,
            name: TrimString("用户1".to_string()),
            icon: None,
            sort: None,
            scope_level: 2,
            disabled: None,
            rel_rbum_kind_id: kind_account_id.to_string(),
            rel_rbum_domain_id: domain_iam_id.to_string(),
        },
        &tx,
        context,
    )
    .await?;

    info!("【test_rbum_rel_attr】 : Prepare Rel : RbumRelServ::add_rbum");
    let rel_id = RbumRelServ::add_rbum(
        &mut RbumRelAddReq {
            tag: "bind".to_string(),
            from_rbum_item_id: item_reldb_inst1_id.to_string(),
            to_rbum_item_id: item_account_a1_id.to_string(),
            to_scope_ids: context.scope_ids.to_string(),
            ext: None,
        },
        &tx,
        context,
    )
    .await?;

    // -----------------------------------

    info!("【test_rbum_rel_attr】 : Test Add : RbumRelAttrServ::add_rbum");
    assert!(RbumRelAttrServ::add_rbum(
        &mut RbumRelAttrAddReq {
            is_from: true,
            value: "mysql".to_string(),
            rel_rbum_rel_id: "".to_string(),
            rel_rbum_kind_attr_id: "".to_string()
        },
        &tx,
        context,
    )
    .await
    .is_err());

    assert!(RbumRelAttrServ::add_rbum(
        &mut RbumRelAttrAddReq {
            is_from: true,
            value: "mysql".to_string(),
            rel_rbum_rel_id: rel_id.to_string(),
            rel_rbum_kind_attr_id: "".to_string()
        },
        &tx,
        context,
    )
    .await
    .is_err());

    let id = RbumRelAttrServ::add_rbum(
        &mut RbumRelAttrAddReq {
            is_from: true,
            value: "mysql".to_string(),
            rel_rbum_rel_id: rel_id.to_string(),
            rel_rbum_kind_attr_id: kind_attr_db_type_id.to_string(),
        },
        &tx,
        context,
    )
    .await?;

    info!("【test_rbum_rel_attr】 : Test Get : RbumRelAttrServ::get_rbum");
    let rbum = RbumRelAttrServ::get_rbum(&id, &RbumBasicFilterReq::default(), &tx, context).await?;
    assert_eq!(rbum.id, id);
    assert_eq!(rbum.value, "mysql");
    assert_eq!(rbum.name, "db_type");

    info!("【test_rbum_rel_attr】 : Test Modify : RbumRelAttrServ::modify_rbum");
    RbumRelAttrServ::modify_rbum(&id, &mut RbumRelAttrModifyReq { value: "tidb".to_string() }, &tx, context).await?;

    info!("【test_rbum_rel_attr】 : Test Find : RbumRelAttrServ::paginate_rbums");
    let rbums = RbumRelAttrServ::paginate_rbums(&RbumBasicFilterReq::default(), 1, 10, None, None, &tx, context).await?;
    assert_eq!(rbums.page_number, 1);
    assert_eq!(rbums.page_size, 10);
    assert_eq!(rbums.total_size, 1);
    assert!(rbums.records.get(0).unwrap().is_from);
    assert_eq!(rbums.records.get(0).unwrap().value, "tidb");
    assert_eq!(rbums.records.get(0).unwrap().name, "db_type");

    info!("【test_rbum_rel_attr】 : Test Delete : RbumRelAttrServ::delete_rbum");
    RbumRelAttrServ::delete_rbum(&id, &tx, context).await?;
    assert!(RbumRelAttrServ::get_rbum(&id, &RbumBasicFilterReq::default(), &tx, context).await.is_err());

    tx.rollback().await?;

    Ok(())
}

async fn test_rbum_rel_env(context: &TardisContext) -> TardisResult<()> {
    let mut tx = TardisFuns::reldb().conn();
    tx.begin().await?;

    info!("【test_rbum_rel_env】 : Prepare Kind : RbumKindServ::add_rbum");
    let kind_reldb_id = RbumKindServ::add_rbum(
        &mut RbumKindAddReq {
            uri_scheme: TrimString("reldb".to_string()),
            name: TrimString("关系型数据库".to_string()),
            note: None,
            icon: None,
            sort: None,
            ext_table_name: None,
            scope_level: 2,
        },
        &tx,
        context,
    )
    .await?;

    let kind_account_id = RbumKindServ::add_rbum(
        &mut RbumKindAddReq {
            uri_scheme: TrimString("account".to_string()),
            name: TrimString("Account".to_string()),
            note: None,
            icon: None,
            sort: None,
            ext_table_name: None,
            scope_level: 2,
        },
        &tx,
        context,
    )
    .await?;

    info!("【test_rbum_rel_env】 : Prepare Domain : RbumDomainServ::add_rbum");
    let domain_reldb_id = RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            uri_authority: TrimString("mysql_dev".to_string()),
            name: TrimString("Mysql测试集群".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: 2,
        },
        &tx,
        context,
    )
    .await?;

    let domain_iam_id = RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            uri_authority: TrimString("iam2".to_string()),
            name: TrimString("IAM2".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: 2,
        },
        &tx,
        context,
    )
    .await?;

    info!("【test_rbum_rel_env】 : Prepare Item : RbumItemServ::add_rbum");
    let item_reldb_inst1_id = RbumItemServ::add_rbum(
        &mut RbumItemAddReq {
            id: None,
            uri_path: None,
            name: TrimString("实例1".to_string()),
            icon: None,
            sort: None,
            scope_level: 2,
            disabled: None,
            rel_rbum_kind_id: kind_reldb_id.to_string(),
            rel_rbum_domain_id: domain_reldb_id.to_string(),
        },
        &tx,
        context,
    )
    .await?;

    let item_account_a1_id = RbumItemServ::add_rbum(
        &mut RbumItemAddReq {
            id: None,
            uri_path: None,
            name: TrimString("用户1".to_string()),
            icon: None,
            sort: None,
            scope_level: 2,
            disabled: None,
            rel_rbum_kind_id: kind_account_id.to_string(),
            rel_rbum_domain_id: domain_iam_id.to_string(),
        },
        &tx,
        context,
    )
    .await?;

    info!("【test_rbum_rel_env】 : Prepare Rel : RbumRelServ::add_rbum");
    let rel_id = RbumRelServ::add_rbum(
        &mut RbumRelAddReq {
            tag: "bind".to_string(),
            from_rbum_item_id: item_reldb_inst1_id.to_string(),
            to_rbum_item_id: item_account_a1_id.to_string(),
            to_scope_ids: context.scope_ids.to_string(),
            ext: None,
        },
        &tx,
        context,
    )
    .await?;

    // -----------------------------------

    info!("【test_rbum_rel_env】 : Test Add : RbumRelEnvServ::add_rbum");
    assert!(RbumRelEnvServ::add_rbum(
        &mut RbumRelEnvAddReq {
            kind: RbumRelEnvKind::DatetimeRange,
            value1: Utc::now().timestamp().to_string(),
            value2: Some((Utc::now().timestamp() + 2000).to_string()),
            rel_rbum_rel_id: "".to_string()
        },
        &tx,
        context,
    )
    .await
    .is_err());

    let start_time = Utc::now().timestamp().to_string();
    let end_time = (Utc::now().timestamp() + 2000).to_string();
    let id = RbumRelEnvServ::add_rbum(
        &mut RbumRelEnvAddReq {
            kind: RbumRelEnvKind::DatetimeRange,
            value1: start_time.clone(),
            value2: Some(end_time.clone()),
            rel_rbum_rel_id: rel_id.to_string(),
        },
        &tx,
        context,
    )
    .await?;

    info!("【test_rbum_rel_env】 : Test Get : RbumRelEnvServ::get_rbum");
    let rbum = RbumRelEnvServ::get_rbum(&id, &RbumBasicFilterReq::default(), &tx, context).await?;
    assert_eq!(rbum.id, id);
    assert_eq!(rbum.kind, "DatetimeRange");
    assert_eq!(rbum.value1, start_time);
    assert_eq!(rbum.value2, end_time);

    info!("【test_rbum_rel_env】 : Test Modify : RbumRelEnvServ::modify_rbum");
    let start_time = (Utc::now().timestamp() + 100).to_string();
    RbumRelEnvServ::modify_rbum(
        &id,
        &mut RbumRelEnvModifyReq {
            value1: Some(start_time.clone()),
            value2: None,
        },
        &tx,
        context,
    )
    .await?;

    info!("【test_rbum_rel_env】 : Test Find : RbumRelEnvServ::paginate_rbums");
    let rbums = RbumRelEnvServ::paginate_rbums(&RbumBasicFilterReq::default(), 1, 10, None, None, &tx, context).await?;
    assert_eq!(rbums.page_number, 1);
    assert_eq!(rbums.page_size, 10);
    assert_eq!(rbums.total_size, 1);
    assert_eq!(rbums.records.get(0).unwrap().kind, "DatetimeRange");
    assert_eq!(rbums.records.get(0).unwrap().value1, start_time);
    assert_eq!(rbums.records.get(0).unwrap().value2, end_time);

    info!("【test_rbum_rel_env】 : Test Delete : RbumRelEnvServ::delete_rbum");
    RbumRelEnvServ::delete_rbum(&id, &tx, context).await?;
    assert!(RbumRelEnvServ::get_rbum(&id, &RbumBasicFilterReq::default(), &tx, context).await.is_err());

    tx.rollback().await?;

    Ok(())
}

async fn test_rbum_rel_use(context: &TardisContext) -> TardisResult<()> {
    let mut tx = TardisFuns::reldb().conn();
    tx.begin().await?;

    info!("【test_rbum_rel_use】 : Prepare Kind : RbumKindServ::add_rbum");
    let kind_reldb_id = RbumKindServ::add_rbum(
        &mut RbumKindAddReq {
            uri_scheme: TrimString("reldb".to_string()),
            name: TrimString("关系型数据库".to_string()),
            note: None,
            icon: None,
            sort: None,
            ext_table_name: None,
            scope_level: 2,
        },
        &tx,
        context,
    )
    .await?;

    info!("【test_rbum_rel_use】 : Prepare Kind Attr : RbumKindAttrServ::add_rbum");
    let kind_attr_db_type_id = RbumKindAttrServ::add_rbum(
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
            scope_level: 2,
            rel_rbum_kind_id: kind_reldb_id.to_string(),
        },
        &tx,
        context,
    )
    .await?;

    let kind_account_id = RbumKindServ::add_rbum(
        &mut RbumKindAddReq {
            uri_scheme: TrimString("account".to_string()),
            name: TrimString("Account".to_string()),
            note: None,
            icon: None,
            sort: None,
            ext_table_name: None,
            scope_level: 2,
        },
        &tx,
        context,
    )
    .await?;

    info!("【test_rbum_rel_use】 : Prepare Domain : RbumDomainServ::add_rbum");
    let domain_reldb_id = RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            uri_authority: TrimString("mysql_dev".to_string()),
            name: TrimString("Mysql测试集群".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: 2,
        },
        &tx,
        context,
    )
    .await?;

    let domain_iam_id = RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            uri_authority: TrimString("iam2".to_string()),
            name: TrimString("IAM2".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: 2,
        },
        &tx,
        context,
    )
    .await?;

    info!("【test_rbum_rel_use】 : Prepare Item : RbumItemServ::add_rbum");
    let item_reldb_inst1_id = RbumItemServ::add_rbum(
        &mut RbumItemAddReq {
            id: None,
            uri_path: None,
            name: TrimString("实例1".to_string()),
            icon: None,
            sort: None,
            scope_level: 2,
            disabled: None,
            rel_rbum_kind_id: kind_reldb_id.to_string(),
            rel_rbum_domain_id: domain_reldb_id.to_string(),
        },
        &tx,
        context,
    )
    .await?;

    let item_account_a1_id = RbumItemServ::add_rbum(
        &mut RbumItemAddReq {
            id: None,
            uri_path: None,
            name: TrimString("用户1".to_string()),
            icon: None,
            sort: None,
            scope_level: 2,
            disabled: None,
            rel_rbum_kind_id: kind_account_id.to_string(),
            rel_rbum_domain_id: domain_iam_id.to_string(),
        },
        &tx,
        context,
    )
    .await?;

    // -----------------------------------

    info!("【test_rbum_rel_use】 : Test Add Agg : RbumRelServ::add_rel");
    let start_time = Utc::now().timestamp().to_string();
    let end_time = (Utc::now().timestamp() + 2).to_string();
    RbumRelServ::add_rel(
        &mut RbumRelAggAddReq {
            rel: RbumRelAddReq {
                tag: "bind".to_string(),
                from_rbum_item_id: item_reldb_inst1_id.to_string(),
                to_rbum_item_id: item_account_a1_id.to_string(),
                to_scope_ids: context.scope_ids.to_string(),
                ext: None,
            },
            attrs: vec![RbumRelAttrAggAddReq {
                is_from: true,
                value: "mysql".to_string(),
                rel_rbum_kind_attr_id: kind_attr_db_type_id.to_string(),
            }],
            envs: vec![RbumRelEnvAggAddReq {
                kind: RbumRelEnvKind::DatetimeRange,
                value1: start_time.clone(),
                value2: Some(end_time.clone()),
            }],
        },
        &tx,
        context,
    )
    .await?;

    info!("【test_rbum_rel_use】 : Test Find From Rels : RbumRelServ::find_from_rels");
    let rbums = RbumRelServ::find_from_rels("bind", kind_reldb_id.as_str(), item_reldb_inst1_id.as_str(), 1, 10, None, None, &tx, context).await?;
    assert_eq!(rbums.page_number, 1);
    assert_eq!(rbums.page_size, 10);
    assert_eq!(rbums.total_size, 1);
    assert_eq!(rbums.records.get(0).unwrap().rel.tag, "bind");
    assert_eq!(rbums.records.get(0).unwrap().rel.to_scope_ids, context.scope_ids.to_string());
    assert_eq!(rbums.records.get(0).unwrap().rel.scope_ids, context.scope_ids.to_string());
    assert_eq!(rbums.records.get(0).unwrap().attrs.len(), 1);
    assert_eq!(rbums.records.get(0).unwrap().attrs.get(0).unwrap().value, "mysql");
    assert_eq!(rbums.records.get(0).unwrap().attrs.get(0).unwrap().name, "db_type");
    assert_eq!(rbums.records.get(0).unwrap().envs.len(), 1);
    assert_eq!(rbums.records.get(0).unwrap().envs.get(0).unwrap().kind, "DatetimeRange");
    assert_eq!(rbums.records.get(0).unwrap().envs.get(0).unwrap().value1, start_time);
    assert_eq!(rbums.records.get(0).unwrap().envs.get(0).unwrap().value2, end_time);

    info!("【test_rbum_rel_use】 : Test Find To Rels : RbumRelServ::find_to_rels");
    let rbums = RbumRelServ::find_to_rels("bind", kind_account_id.as_str(), item_account_a1_id.as_str(), 1, 10, None, None, &tx, context).await?;
    assert_eq!(rbums.page_number, 1);
    assert_eq!(rbums.page_size, 10);
    assert_eq!(rbums.total_size, 1);
    assert_eq!(rbums.records.get(0).unwrap().rel.tag, "bind");
    assert_eq!(rbums.records.get(0).unwrap().rel.to_scope_ids, context.scope_ids.to_string());
    assert_eq!(rbums.records.get(0).unwrap().rel.scope_ids, context.scope_ids.as_str());
    assert_eq!(rbums.records.get(0).unwrap().attrs.len(), 1);
    assert_eq!(rbums.records.get(0).unwrap().attrs.get(0).unwrap().value, "mysql");
    assert_eq!(rbums.records.get(0).unwrap().attrs.get(0).unwrap().name, "db_type");
    assert_eq!(rbums.records.get(0).unwrap().envs.len(), 1);
    assert_eq!(rbums.records.get(0).unwrap().envs.get(0).unwrap().kind, "DatetimeRange");
    assert_eq!(rbums.records.get(0).unwrap().envs.get(0).unwrap().value1, start_time);
    assert_eq!(rbums.records.get(0).unwrap().envs.get(0).unwrap().value2, end_time);

    info!("【test_rbum_rel_use】 : Test Check Rel : RbumRelServ::check_rel");
    assert!(
        !RbumRelServ::check_rel(
            &RbumRelCheckReq {
                tag: "".to_string(),
                from_rbum_item_id: "".to_string(),
                to_rbum_item_id: "".to_string(),
                from_attrs: Default::default(),
                to_attrs: Default::default()
            },
            &tx,
            context
        )
        .await?
    );

    assert!(
        !RbumRelServ::check_rel(
            &RbumRelCheckReq {
                tag: "bind".to_string(),
                from_rbum_item_id: item_reldb_inst1_id.to_string(),
                to_rbum_item_id: item_account_a1_id.to_string(),
                from_attrs: Default::default(),
                to_attrs: Default::default()
            },
            &tx,
            context
        )
        .await?
    );

    assert!(
        !RbumRelServ::check_rel(
            &RbumRelCheckReq {
                tag: "bind".to_string(),
                from_rbum_item_id: item_reldb_inst1_id.to_string(),
                to_rbum_item_id: item_account_a1_id.to_string(),
                from_attrs: HashMap::from([("db_type".to_string(), "tidb".to_string()),]),
                to_attrs: Default::default()
            },
            &tx,
            context
        )
        .await?
    );

    assert!(
        RbumRelServ::check_rel(
            &RbumRelCheckReq {
                tag: "bind".to_string(),
                from_rbum_item_id: item_reldb_inst1_id.to_string(),
                to_rbum_item_id: item_account_a1_id.to_string(),
                from_attrs: HashMap::from([("db_type".to_string(), "mysql".to_string()),]),
                to_attrs: Default::default()
            },
            &tx,
            context
        )
        .await?
    );

    tardis::tokio::time::sleep(Duration::from_secs(3)).await;

    assert!(
        !RbumRelServ::check_rel(
            &RbumRelCheckReq {
                tag: "bind".to_string(),
                from_rbum_item_id: item_reldb_inst1_id.to_string(),
                to_rbum_item_id: item_account_a1_id.to_string(),
                from_attrs: HashMap::from([("db_type".to_string(), "mysql".to_string()),]),
                to_attrs: Default::default()
            },
            &tx,
            context
        )
        .await?
    );

    tx.rollback().await?;

    Ok(())
}
