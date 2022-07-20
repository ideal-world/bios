use tardis::basic::dto::TardisContext;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm::prelude::*;
use tardis::db::sea_orm::sea_query::{ColumnDef, Index, IndexCreateStatement, Table, TableCreateStatement};
use tardis::db::sea_orm::*;

/// Relationship model
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "rbum_rel")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    /// Relationship label
    pub tag: String,
    pub note: String,
    /// The [source kind](crate::rbum::rbum_enumeration::RbumRelFromKind) of the relationship
    pub from_rbum_kind: u8,
    /// The source id of the relationship
    pub from_rbum_id: String,
    /// The target resource id of the relationship
    pub to_rbum_item_id: String,
    pub to_own_paths: String,
    /// Extended Information  \
    /// E.g. the record from or to is in another service, to avoid remote calls, you can redundantly add the required information to this field.
    pub ext: String,

    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime,
    pub update_time: DateTime,
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
            .col(ColumnDef::new(Column::Tag).not_null().string())
            .col(ColumnDef::new(Column::Note).not_null().string())
            .col(ColumnDef::new(Column::FromRbumKind).not_null().tiny_unsigned())
            .col(ColumnDef::new(Column::FromRbumId).not_null().string())
            .col(ColumnDef::new(Column::ToRbumItemId).not_null().string())
            .col(ColumnDef::new(Column::ToOwnPaths).not_null().string())
            .col(ColumnDef::new(Column::Ext).not_null().string())
            // Basic
            .col(ColumnDef::new(Column::OwnPaths).not_null().string())
            .col(ColumnDef::new(Column::Owner).not_null().string())
            .col(ColumnDef::new(Column::CreateTime).extra("DEFAULT CURRENT_TIMESTAMP".to_string()).date_time())
            .col(ColumnDef::new(Column::UpdateTime).extra("DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP".to_string()).date_time())
            .to_owned()
    }

    fn create_index_statement() -> Vec<IndexCreateStatement> {
        vec![
            Index::create().name(&format!("idx-{}-{}", Entity.table_name(), Column::OwnPaths.to_string())).table(Entity).col(Column::OwnPaths).to_owned(),
            Index::create().name(&format!("idx-{}-from", Entity.table_name())).table(Entity).col(Column::Tag).col(Column::FromRbumKind).col(Column::FromRbumId).to_owned(),
            Index::create().name(&format!("idx-{}-to", Entity.table_name())).table(Entity).col(Column::Tag).col(Column::ToRbumItemId).to_owned(),
        ]
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
