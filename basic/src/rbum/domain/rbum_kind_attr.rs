use tardis::basic::dto::TardisContext;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm::prelude::*;
use tardis::db::sea_orm::sea_query::{ColumnDef, Index, IndexCreateStatement, Table, TableCreateStatement};
use tardis::db::sea_orm::*;

/// Resource kind extended attribute definition model
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "rbum_kind_attr")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,

    pub name: String,
    pub module: String,
    pub label: String,
    pub note: String,
    pub sort: u32,
    pub main_column: bool,
    pub position: bool,
    pub capacity: bool,
    pub overload: bool,
    pub hide: bool,
    pub idx: bool,
    pub data_type: String,
    pub widget_type: String,
    pub default_value: String,
    pub options: String,
    pub required: bool,
    pub min_length: u32,
    pub max_length: u32,
    /// Custom behavior attributes \
    /// E.g. user selection function, role selection function, etc.
    /// Custom behavior needs to be bound to the corresponding function code.
    pub action: String,
    pub ext: String,
    /// Associated [resource kind](crate::rbum::domain::rbum_kind::Model) id
    pub rel_rbum_kind_id: String,

    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime,
    pub update_time: DateTime,

    pub scope_level: i8,
}

impl TardisActiveModel for ActiveModel {
    fn fill_cxt(&mut self, ctx: &TardisContext, is_insert: bool) {
        if is_insert {
            self.own_paths = Set(ctx.own_paths.to_string());
            self.owner = Set(ctx.owner.to_string());
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
            // Specific
            .col(ColumnDef::new(Column::Name).not_null().string())
            .col(ColumnDef::new(Column::Module).not_null().string())
            .col(ColumnDef::new(Column::Label).not_null().string())
            .col(ColumnDef::new(Column::Note).not_null().string())
            .col(ColumnDef::new(Column::Sort).not_null().unsigned())
            .col(ColumnDef::new(Column::MainColumn).not_null().boolean())
            .col(ColumnDef::new(Column::Position).not_null().boolean())
            .col(ColumnDef::new(Column::Capacity).not_null().boolean())
            .col(ColumnDef::new(Column::Hide).not_null().boolean())
            .col(ColumnDef::new(Column::Overload).not_null().boolean())
            .col(ColumnDef::new(Column::Idx).not_null().boolean())
            .col(ColumnDef::new(Column::DataType).not_null().string())
            .col(ColumnDef::new(Column::WidgetType).not_null().string())
            .col(ColumnDef::new(Column::DefaultValue).not_null().string())
            .col(ColumnDef::new(Column::Options).not_null().text())
            .col(ColumnDef::new(Column::Required).not_null().boolean())
            .col(ColumnDef::new(Column::MinLength).not_null().unsigned())
            .col(ColumnDef::new(Column::MaxLength).not_null().unsigned())
            .col(ColumnDef::new(Column::Action).not_null().string())
            .col(ColumnDef::new(Column::Ext).not_null().string())
            .col(ColumnDef::new(Column::RelRbumKindId).not_null().string())
            // Basic
            .col(ColumnDef::new(Column::OwnPaths).not_null().string())
            .col(ColumnDef::new(Column::Owner).not_null().string())
            .col(ColumnDef::new(Column::CreateTime).extra("DEFAULT CURRENT_TIMESTAMP".to_string()).date_time())
            .col(ColumnDef::new(Column::UpdateTime).extra("DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP".to_string()).date_time())
            // With Scope
            .col(ColumnDef::new(Column::ScopeLevel).not_null().tiny_integer())
            .to_owned()
    }

    fn create_index_statement() -> Vec<IndexCreateStatement> {
        // TODO Specified key was too long; max key length is 3072 bytes
        // vec![Index::create()
        //     .name(&format!("idx-{}-{}", Entity.table_name(), Column::RelRbumKindId.to_string()))
        //     .table(Entity)
        //     .col(Column::RelRbumKindId)
        //     .col(Column::Module)
        //     .col(Column::Name)
        //     .col(Column::OwnPaths)
        //     .unique()
        //     .to_owned()]
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
