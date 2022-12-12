use tardis::basic::dto::TardisContext;
use tardis::chrono::{self, Utc};
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm;
use tardis::db::sea_orm::prelude::*;
use tardis::db::sea_orm::sea_query::{ColumnDef, Index, IndexCreateStatement, Table, TableCreateStatement};
use tardis::db::sea_orm::*;

/// Resource kind extended attribute definition model
///
/// General logic for dynamic request processing:
///
/// 1. dynamic values take precedence over static values
/// 2. support calling http to get data with GET request
/// 3. request url supports attribute variable substitution, format is: `{attribute name}` .
/// 4. if no attribute variable substitution exists and secret = false, the url is called directly and the corresponding value is returned,
/// 5. if attribute variable substitution exists, then：
///  1） extract all attribute variables to be replaced
///  2） monitor changes of these attributes
///  3） substitute attribute variables with values into the url
///  4） if no longer an attribute variable substitution in the url and secret = false, call the url and return the corresponding value
/// 6. before the resource object is saved, if secret = true and an attribute variable substitution in the url, call the url and return the corresponding value
///
/// For security reasons, step 6 must be done by the server side.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "rbum_kind_attr")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,

    pub name: String,
    pub module: String,
    pub label: String,
    pub note: String,
    pub sort: i64,
    pub main_column: bool,
    pub position: bool,
    pub capacity: bool,
    pub overload: bool,
    pub hide: bool,
    /// When secret = true, the attribute information is not returned to the frontend(except in configuration)
    pub secret: bool,
    /// Display condition, json format: `{<attribute name>:<attribute value>}`, currently only support `and` operations
    pub show_by_conds: String,
    /// Whether indexing is needed
    pub idx: bool,
    /// Associated [resource kind](crate::rbum::rbum_enumeration::RbumDataTypeKind)
    pub data_type: String,
    /// Associated [resource kind](crate::rbum::rbum_enumeration::RbumWidgetTypeKind)
    pub widget_type: String,
    pub widget_columns: i16,
    pub default_value: String,
    /// Dynamic default value
    /// the return format is the same as `default_value`
    /// or `json` when `data_type` = `Json` and `widget_type` = `Control`
    /// or `array` when `data_type` = `Array` and `widget_type` = `Group`
    pub dyn_default_value: String,
    /// Fixed option, json array formatted as `[{name:<display name>:value:<corresponding value>}]`
    pub options: String,
    /// Dynamic options
    /// the return format is the same as `options`
    pub dyn_options: String,
    pub required: bool,
    pub min_length: i32,
    pub max_length: i32,
    /// Used to implement multi-level attributes, default is empty
    pub parent_attr_name: String,
    /// Custom behavior attributes \
    /// E.g. user selection function, role selection function, etc.
    /// Custom behavior needs to be bound to the corresponding function code.
    pub action: String,
    pub ext: String,
    /// Associated [resource kind](crate::rbum::domain::rbum_kind::Model) id
    pub rel_rbum_kind_id: String,

    pub own_paths: String,
    pub owner: String,
    pub create_time: chrono::DateTime<Utc>,
    pub update_time: chrono::DateTime<Utc>,

    pub scope_level: i16,
}

impl TardisActiveModel for ActiveModel {
    fn fill_ctx(&mut self, ctx: &TardisContext, is_insert: bool) {
        if is_insert {
            self.own_paths = Set(ctx.own_paths.to_string());
            self.owner = Set(ctx.owner.to_string());
        }
    }

    fn create_table_statement(db: DbBackend) -> TableCreateStatement {
        let mut builder = Table::create();
        builder
            .table(Entity.table_ref())
            .if_not_exists()
            .col(ColumnDef::new(Column::Id).not_null().string().primary_key())
            // Specific
            .col(ColumnDef::new(Column::Name).not_null().string())
            .col(ColumnDef::new(Column::Module).not_null().string())
            .col(ColumnDef::new(Column::Label).not_null().string())
            .col(ColumnDef::new(Column::Note).not_null().string().default(""))
            .col(ColumnDef::new(Column::Sort).not_null().big_integer())
            .col(ColumnDef::new(Column::MainColumn).not_null().boolean().default(false))
            .col(ColumnDef::new(Column::Position).not_null().boolean().default(false))
            .col(ColumnDef::new(Column::Capacity).not_null().boolean().default(false))
            .col(ColumnDef::new(Column::Hide).not_null().boolean().default(false))
            .col(ColumnDef::new(Column::Secret).not_null().boolean().default(false))
            .col(ColumnDef::new(Column::ShowByConds).not_null().string().default(""))
            .col(ColumnDef::new(Column::Overload).not_null().boolean().default(false))
            .col(ColumnDef::new(Column::Idx).not_null().boolean().default(false))
            .col(ColumnDef::new(Column::DataType).not_null().string())
            .col(ColumnDef::new(Column::WidgetType).not_null().string())
            .col(ColumnDef::new(Column::WidgetColumns).not_null().small_integer().default(1))
            .col(ColumnDef::new(Column::DefaultValue).not_null().string().default(""))
            .col(ColumnDef::new(Column::DynDefaultValue).not_null().string().default(""))
            .col(ColumnDef::new(Column::Options).not_null().text())
            .col(ColumnDef::new(Column::DynOptions).not_null().string().default(""))
            .col(ColumnDef::new(Column::Required).not_null().boolean().default(false))
            .col(ColumnDef::new(Column::MinLength).not_null().integer())
            .col(ColumnDef::new(Column::MaxLength).not_null().integer())
            .col(ColumnDef::new(Column::ParentAttrName).not_null().string().default(""))
            .col(ColumnDef::new(Column::Action).not_null().string().default(""))
            .col(ColumnDef::new(Column::Ext).not_null().string().default(""))
            .col(ColumnDef::new(Column::RelRbumKindId).not_null().string())
            // Basic
            .col(ColumnDef::new(Column::OwnPaths).not_null().string())
            .col(ColumnDef::new(Column::Owner).not_null().string())
            // With Scope
            .col(ColumnDef::new(Column::ScopeLevel).not_null().small_integer());
        if db == DatabaseBackend::Postgres {
            builder
                .col(ColumnDef::new(Column::CreateTime).extra("DEFAULT CURRENT_TIMESTAMP".to_string()).timestamp_with_time_zone())
                .col(ColumnDef::new(Column::UpdateTime).extra("DEFAULT CURRENT_TIMESTAMP".to_string()).timestamp_with_time_zone());
        } else {
            builder
                .engine("InnoDB")
                .character_set("utf8mb4")
                .collate("utf8mb4_0900_as_cs")
                .col(ColumnDef::new(Column::CreateTime).extra("DEFAULT CURRENT_TIMESTAMP".to_string()).timestamp())
                .col(ColumnDef::new(Column::UpdateTime).extra("DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP".to_string()).timestamp());
        }
        builder.to_owned()
    }

    fn create_index_statement() -> Vec<IndexCreateStatement> {
        vec![Index::create()
            .name(&format!("idx-{}-{}", Entity.table_name(), Column::RelRbumKindId.to_string()))
            .table(Entity)
            .col(Column::RelRbumKindId)
            .col(Column::Name)
            .col(Column::Module)
            .to_owned()]
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
