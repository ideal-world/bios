use tardis::basic::dto::TardisContext;
use tardis::chrono::{self, Utc};
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm;
use tardis::db::sea_orm::prelude::*;
use tardis::db::sea_orm::sea_query::{ColumnDef, IndexCreateStatement, Table, TableCreateStatement};
use tardis::db::sea_orm::*;
use tardis::TardisCreateIndex;

/// Resource extended attribute value model
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, TardisCreateIndex)]
#[sea_orm(table_name = "rbum_item_attr")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    /// Extended attribute value
    pub value: String,
    /// Associated [resource](crate::rbum::domain::rbum_item::Model) id
    #[index(index_id = "id", unique)]
    pub rel_rbum_item_id: String,
    /// Associated [resource kind extended attribute](crate::rbum::domain::rbum_kind_attr::Model) id
    #[index(index_id = "id", unique)]
    pub rel_rbum_kind_attr_id: String,

    pub own_paths: String,
    pub owner: String,
    pub create_time: chrono::DateTime<Utc>,
    pub update_time: chrono::DateTime<Utc>,
    pub create_by: String,
    pub update_by: String,
}

impl TardisActiveModel for ActiveModel {
    fn fill_ctx(&mut self, ctx: &TardisContext, is_insert: bool) {
        if is_insert {
            self.own_paths = Set(ctx.own_paths.to_string());
            self.owner = Set(ctx.owner.to_string());
            self.create_by = Set(ctx.owner.to_string());
        }
        self.update_by = Set(ctx.owner.to_string());
    }

    fn create_table_statement(db: DbBackend) -> TableCreateStatement {
        let mut builder = Table::create();
        builder
            .table(Entity.table_ref())
            .if_not_exists()
            .col(ColumnDef::new(Column::Id).not_null().string().primary_key())
            // Specific
            .col(ColumnDef::new(Column::Value).not_null().string())
            .col(ColumnDef::new(Column::RelRbumItemId).not_null().string())
            .col(ColumnDef::new(Column::RelRbumKindAttrId).not_null().string())
            // Basic
            .col(ColumnDef::new(Column::OwnPaths).not_null().string())
            .col(ColumnDef::new(Column::Owner).not_null().string())
            .col(ColumnDef::new(Column::CreateBy).not_null().string())
            .col(ColumnDef::new(Column::UpdateBy).not_null().string());
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
        tardis_create_index_statement()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
