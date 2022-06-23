use sea_orm::EntityName;
use tardis::basic::dto::TardisContext;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm::prelude::*;
use tardis::db::sea_orm::*;
use tardis::db::sea_query::{ColumnDef, Index, IndexCreateStatement, Table, TableCreateStatement};

/// Relationship environment condition model
///
/// This model is used to further qualify the conditions under which the relationship is established
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "rbum_rel_env")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    /// [Environment Kind](crate::rbum::rbum_enumeration::RbumRelEnvKind)
    pub kind: u8,
    pub value1: String,
    pub value2: String,
    /// Associated [relationship](crate::rbum::domain::rbum_rel::Model) id
    pub rel_rbum_rel_id: String,

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
            .col(ColumnDef::new(Column::Kind).not_null().tiny_unsigned())
            .col(ColumnDef::new(Column::Value1).not_null().string())
            .col(ColumnDef::new(Column::Value2).not_null().string())
            .col(ColumnDef::new(Column::RelRbumRelId).not_null().string())
            // Basic
            .col(ColumnDef::new(Column::OwnPaths).not_null().string())
            .col(ColumnDef::new(Column::Owner).not_null().string())
            .col(ColumnDef::new(Column::CreateTime).extra("DEFAULT CURRENT_TIMESTAMP".to_string()).date_time())
            .col(ColumnDef::new(Column::UpdateTime).extra("DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP".to_string()).date_time())
            .to_owned()
    }

    fn create_index_statement() -> Vec<IndexCreateStatement> {
        vec![Index::create().name(&format!("idx-{}-{}", Entity.table_name(), Column::RelRbumRelId.to_string())).table(Entity).col(Column::RelRbumRelId).to_owned()]
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
