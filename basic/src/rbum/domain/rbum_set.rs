use tardis::basic::dto::TardisContext;
use tardis::chrono::{self, Utc};
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm;
use tardis::db::sea_orm::prelude::*;
use tardis::db::sea_orm::sea_query::{ColumnDef, Index, IndexCreateStatement, Table, TableCreateStatement};
use tardis::db::sea_orm::*;

/// Resource set model
///
/// Resource set is essentially a general tree structure processing model
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "rbum_set")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    // Set code
    pub code: String,
    // Set kind
    pub kind: String,
    pub name: String,
    pub note: String,
    pub icon: String,
    pub sort: u32,
    pub ext: String,

    pub own_paths: String,
    pub owner: String,
    pub create_time: chrono::DateTime<Utc>,
    pub update_time: chrono::DateTime<Utc>,

    pub scope_level: i8,

    pub disabled: bool,
}

impl TardisActiveModel for ActiveModel {
    fn fill_ctx(&mut self, ctx: &TardisContext, is_insert: bool) {
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
            .col(ColumnDef::new(Column::Code).not_null().string())
            .col(ColumnDef::new(Column::Kind).not_null().string())
            .col(ColumnDef::new(Column::Name).not_null().string())
            .col(ColumnDef::new(Column::Note).not_null().string())
            .col(ColumnDef::new(Column::Icon).not_null().string())
            .col(ColumnDef::new(Column::Sort).not_null().unsigned())
            .col(ColumnDef::new(Column::Ext).not_null().string())
            // Basic
            .col(ColumnDef::new(Column::OwnPaths).not_null().string())
            .col(ColumnDef::new(Column::Owner).not_null().string())
            .col(ColumnDef::new(Column::CreateTime).extra("DEFAULT CURRENT_TIMESTAMP".to_string()).timestamp())
            .col(ColumnDef::new(Column::UpdateTime).extra("DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP".to_string()).timestamp())
            // With Scope
            .col(ColumnDef::new(Column::ScopeLevel).not_null().tiny_integer())
            // With Status
            .col(ColumnDef::new(Column::Disabled).not_null().boolean())
            .to_owned()
    }

    fn create_index_statement() -> Vec<IndexCreateStatement> {
        vec![
            Index::create().name(&format!("idx-{}-{}", Entity.table_name(), Column::OwnPaths.to_string())).table(Entity).col(Column::OwnPaths).to_owned(),
            Index::create().name(&format!("idx-{}-{}", Entity.table_name(), Column::Code.to_string())).table(Entity).col(Column::Code).unique().to_owned(),
            Index::create().name(&format!("idx-{}-{}", Entity.table_name(), Column::Kind.to_string())).table(Entity).col(Column::Kind).to_owned(),
        ]
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
