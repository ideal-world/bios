use std::collections::HashMap;

use sea_orm::sea_query::Query;
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm::sea_query::Expr;
use tardis::db::sea_orm::*;
use tardis::log::info;
use tardis::TardisFuns;

use bios_basic::rbum::dto::rbum_domain_dto::RbumDomainAddReq;
use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumItemAttrFilterReq, RbumKindAttrFilterReq};
use bios_basic::rbum::dto::rbum_item_attr_dto::{RbumItemAttrAddReq, RbumItemAttrModifyReq, RbumItemAttrsAddOrModifyReq};
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
    test_rbum_item_attr_has_main_table(context).await?;
    Ok(())
}

async fn test_rbum_item(context: &TardisContext) -> TardisResult<()> {
    let mut funs = TardisFuns::inst_with_db_conn("".to_string(), None);
    funs.begin().await?;

    info!("【test_rbum_item】 : Prepare : RbumKindServ::add_rbum");
    let kind_id = RbumKindServ::add_rbum(
        &mut RbumKindAddReq {
            code: TrimString("reldb".to_string()),
            name: TrimString("关系型数据库".to_string()),
            note: None,
            icon: None,
            sort: None,
            ext_table_name: None,
            scope_level: Some(RbumScopeLevelKind::L2),
        },
        &funs,
        context,
    )
    .await?;

    info!("【test_rbum_item】 : Prepare Domain : RbumDomainServ::add_rbum");
    let domain_id = RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            code: TrimString("mysql-dev".to_string()),
            name: TrimString("Mysql测试集群".to_string()),
            note: Some("...".to_string()),
            icon: Some("...".to_string()),
            sort: None,
            scope_level: Some(RbumScopeLevelKind::L2),
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
            scope_level: Some(RbumScopeLevelKind::L2),
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
            scope_level: Some(RbumScopeLevelKind::L2),
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
            scope_level: Some(RbumScopeLevelKind::L2),
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
            scope_level: Some(RbumScopeLevelKind::L2),
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
    let mut funs = TardisFuns::inst_with_db_conn("".to_string(), None);
    funs.begin().await?;

    info!("【test_rbum_item_attr】 : Prepare : RbumKindServ::add_rbum");
    let kind_id = RbumKindServ::add_rbum(
        &mut RbumKindAddReq {
            code: TrimString("reldb".to_string()),
            name: TrimString("关系型数据库".to_string()),
            note: None,
            icon: None,
            sort: None,
            ext_table_name: None,
            scope_level: Some(RbumScopeLevelKind::L2),
        },
        &funs,
        context,
    )
    .await?;

    info!("【test_rbum_item_attr】 : Prepare Kind Attr : RbumKindAttrServ::add_rbum");
    let kind_attr_id = RbumKindAttrServ::add_rbum(
        &mut RbumKindAttrAddReq {
            name: TrimString("db_type".to_string()),
            module: None,
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
            ext: None,
            rel_rbum_kind_id: kind_id.to_string(),
            scope_level: Some(RbumScopeLevelKind::L2),
            idx: None,
            hide: None,
        },
        &funs,
        context,
    )
    .await?;

    info!("【test_rbum_item_attr】 : Prepare Domain : RbumDomainServ::add_rbum");
    let domain_id = RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            code: TrimString("mysql-dev".to_string()),
            name: TrimString("Mysql测试集群".to_string()),
            note: Some("...".to_string()),
            icon: Some("...".to_string()),
            sort: None,
            scope_level: Some(RbumScopeLevelKind::L2),
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
            scope_level: Some(RbumScopeLevelKind::L2),
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
    let rbum = RbumItemAttrServ::get_rbum(&item_attr_id, &RbumItemAttrFilterReq::default(), &funs, context).await?;
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
    let rbums = RbumItemAttrServ::paginate_rbums(&RbumItemAttrFilterReq::default(), 1, 10, None, None, &funs, context).await?;
    assert_eq!(rbums.page_number, 1);
    assert_eq!(rbums.page_size, 10);
    assert_eq!(rbums.total_size, 1);
    assert_eq!(rbums.records.get(0).unwrap().value, "数据3");

    info!("【test_rbum_item_attr】 : Test Delete : RbumItemAttrServ::delete_rbum");
    RbumItemAttrServ::delete_rbum(&item_attr_id, &funs, context).await?;
    assert!(RbumItemAttrServ::get_rbum(&item_attr_id, &RbumItemAttrFilterReq::default(), &funs, context).await.is_err());

    funs.rollback().await?;

    Ok(())
}

async fn test_rbum_item_attr_has_main_table(context: &TardisContext) -> TardisResult<()> {
    let mut funs = TardisFuns::inst_with_db_conn("".to_string(), None);
    funs.begin().await?;

    TardisFuns::inst_with_db_conn("".to_string(), None)
        .db()
        .create_table_and_index(&test_iam_account::ActiveModel::create_table_and_index_statement(TardisFuns::reldb().backend()))
        .await?;

    info!("【test_rbum_item_attr】 : Prepare : RbumKindServ::add_rbum");
    let kind_id = RbumKindServ::add_rbum(
        &mut RbumKindAddReq {
            code: TrimString("account".to_string()),
            name: TrimString("账号".to_string()),
            note: None,
            icon: None,
            sort: None,
            ext_table_name: Some("iam_account".to_string()),
            scope_level: Some(RbumScopeLevelKind::L2),
        },
        &funs,
        context,
    )
    .await?;

    info!("【test_rbum_item_attr】 : Prepare Kind Attr : RbumKindAttrServ::add_rbum");
    RbumKindAttrServ::add_rbum(
        &mut RbumKindAttrAddReq {
            name: TrimString("ext2".to_string()),
            module: None,
            label: "图标".to_string(),
            data_type: RbumDataTypeKind::String,
            widget_type: RbumWidgetTypeKind::InputTxt,
            note: None,
            sort: None,
            main_column: Some(true),
            position: None,
            capacity: None,
            overload: None,
            default_value: None,
            options: None,
            required: None,
            min_length: None,
            max_length: None,
            action: None,
            ext: None,
            rel_rbum_kind_id: kind_id.to_string(),
            scope_level: Some(RbumScopeLevelKind::L2),
            idx: None,
            hide: None,
        },
        &funs,
        context,
    )
    .await?;

    RbumKindAttrServ::add_rbum(
        &mut RbumKindAttrAddReq {
            name: TrimString("ext1_idx".to_string()),
            module: None,
            label: "是否临时账号".to_string(),
            data_type: RbumDataTypeKind::Boolean,
            widget_type: RbumWidgetTypeKind::Checkbox,
            note: None,
            sort: None,
            main_column: Some(true),
            position: None,
            capacity: None,
            overload: None,
            default_value: None,
            options: None,
            required: None,
            min_length: None,
            max_length: None,
            action: None,
            ext: None,
            rel_rbum_kind_id: kind_id.to_string(),
            scope_level: Some(RbumScopeLevelKind::L2),
            idx: None,
            hide: None,
        },
        &funs,
        context,
    )
    .await?;

    RbumKindAttrServ::add_rbum(
        &mut RbumKindAttrAddReq {
            name: TrimString("addr".to_string()),
            module: None,
            label: "住址".to_string(),
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
            ext: None,
            rel_rbum_kind_id: kind_id.to_string(),
            scope_level: Some(RbumScopeLevelKind::L2),
            idx: None,
            hide: None,
        },
        &funs,
        context,
    )
    .await?;

    info!("【test_rbum_item_attr】 : Prepare Domain : RbumDomainServ::add_rbum");
    let domain_id = RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            code: TrimString("test-iam".to_string()),
            name: TrimString("IAM".to_string()),
            note: Some("...".to_string()),
            icon: Some("...".to_string()),
            sort: None,
            scope_level: Some(RbumScopeLevelKind::L2),
        },
        &funs,
        context,
    )
    .await?;

    let item_id = RbumItemServ::add_rbum(
        &mut RbumItemAddReq {
            id: None,
            code: None,
            name: TrimString("用户1".to_string()),
            disabled: None,
            rel_rbum_kind_id: kind_id.to_string(),
            rel_rbum_domain_id: domain_id.to_string(),
            scope_level: Some(RbumScopeLevelKind::L2),
        },
        &funs,
        context,
    )
    .await?;

    funs.db()
        .execute(
            Query::insert()
                .into_table(test_iam_account::Entity)
                .columns(vec![test_iam_account::Column::Id, test_iam_account::Column::Ext1Idx, test_iam_account::Column::Ext2])
                .values_panic(vec![item_id.clone().into(), "".into(), "".into()]),
        )
        .await?;

    // -----------------------------------

    info!("【test_rbum_item_attr】 : Test RbumItemAttrServ::find_item_attr_defs");
    let attr_defs = RbumKindAttrServ::find_rbums(
        &RbumKindAttrFilterReq {
            basic: RbumBasicFilterReq {
                rbum_kind_id: Some(kind_id.to_string()),
                desc_by_sort: Some(true),
                ..Default::default()
            },
            ..Default::default()
        },
        None,
        None,
        &funs,
        context,
    )
    .await?;

    assert_eq!(attr_defs.len(), 3);

    info!("【test_rbum_item_attr】 : Test Add : RbumItemAttrServ::add_or_modify_item_attrs");
    RbumItemAttrServ::add_or_modify_item_attrs(
        &RbumItemAttrsAddOrModifyReq {
            values: HashMap::from([
                ("ext1_idx".to_string(), "true".to_string()),
                ("ext2".to_string(), "/c/c/d/".to_string()),
                ("addr".to_string(), "中国杭州".to_string()),
            ]),
            rel_rbum_item_id: item_id.to_string(),
        },
        &funs,
        context,
    )
    .await?;

    let ext_values = RbumItemAttrServ::find_rbums(
        &RbumItemAttrFilterReq {
            basic: Default::default(),
            rel_rbum_item_id: Some(item_id.to_string()),
            ..Default::default()
        },
        None,
        None,
        &funs,
        context,
    )
    .await?;
    assert_eq!(ext_values.len(), 1);
    assert_eq!(ext_values.get(0).unwrap().value, "中国杭州");

    let main_values = funs
        .db()
        .get_dto::<IamAccountResp>(
            Query::select()
                .column(test_iam_account::Column::Id)
                .column(test_iam_account::Column::Ext1Idx)
                .column(test_iam_account::Column::Ext2)
                .from(test_iam_account::Entity)
                .and_where(Expr::col(test_iam_account::Column::Id).eq(item_id.as_str())),
        )
        .await?
        .unwrap();
    assert_eq!(main_values.ext1_idx, "true");
    assert_eq!(main_values.ext2, "/c/c/d/");

    info!("【test_rbum_item_attr】 : Test Modify : RbumItemAttrServ::add_or_modify_item_attrs");
    RbumItemAttrServ::add_or_modify_item_attrs(
        &RbumItemAttrsAddOrModifyReq {
            values: HashMap::from([("ext1_idx".to_string(), "false".to_string()), ("addr".to_string(), "杭州".to_string())]),
            rel_rbum_item_id: item_id.to_string(),
        },
        &funs,
        context,
    )
    .await?;

    let ext_values = RbumItemAttrServ::find_rbums(
        &RbumItemAttrFilterReq {
            basic: Default::default(),
            rel_rbum_item_id: Some(item_id.to_string()),
            ..Default::default()
        },
        None,
        None,
        &funs,
        context,
    )
    .await?;
    assert_eq!(ext_values.len(), 1);
    assert_eq!(ext_values.get(0).unwrap().value, "杭州");

    let main_values = funs
        .db()
        .get_dto::<IamAccountResp>(
            &Query::select()
                .column(test_iam_account::Column::Id)
                .column(test_iam_account::Column::Ext1Idx)
                .column(test_iam_account::Column::Ext2)
                .from(test_iam_account::Entity)
                .and_where(Expr::col(test_iam_account::Column::Id).eq(item_id.as_str())),
        )
        .await?
        .unwrap();
    assert_eq!(main_values.ext1_idx, "false");
    assert_eq!(main_values.ext2, "/c/c/d/");

    funs.rollback().await?;

    Ok(())
}

#[derive(Debug, FromQueryResult)]
pub struct IamAccountResp {
    pub id: String,
    pub ext1_idx: String,
    pub ext2: String,
}

mod test_iam_account {
    use tardis::basic::dto::TardisContext;
    use tardis::db::reldb_client::TardisActiveModel;
    use tardis::db::sea_orm::sea_query::{ColumnDef, Index, IndexCreateStatement, Table, TableCreateStatement};
    use tardis::db::sea_orm::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "iam_account")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: String,
        pub ext1_idx: String,
        pub ext2: String,

        pub own_paths: String,
    }

    impl TardisActiveModel for ActiveModel {
        fn fill_cxt(&mut self, ctx: &TardisContext, is_insert: bool) {
            if is_insert {
                self.own_paths = Set(ctx.own_paths.to_string());
            }
        }

        fn create_table_statement(_: DbBackend) -> TableCreateStatement {
            Table::create()
                .table(Entity.table_ref())
                .if_not_exists()
                .engine("InnoDB")
                .character_set("utf8mb4")
                .collate("utf8mb4_0900_as_cs")
                .col(ColumnDef::new(Column::Id).not_null().string().primary_key())
                .col(ColumnDef::new(Column::Ext1Idx).not_null().string())
                .col(ColumnDef::new(Column::Ext2).not_null().string())
                .to_owned()
        }

        fn create_index_statement() -> Vec<IndexCreateStatement> {
            vec![Index::create().name(&format!("idx-{}-idx1", Entity.table_name())).table(Entity).col(Column::Ext1Idx).to_owned()]
        }
    }

    impl ActiveModelBehavior for ActiveModel {}

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}
}
