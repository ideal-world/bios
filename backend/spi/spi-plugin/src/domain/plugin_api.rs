use tardis::basic::dto::TardisContext;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm;
use tardis::db::sea_orm::sea_query::{ColumnDef, IndexCreateStatement, Table, TableCreateStatement};
use tardis::db::sea_orm::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "plugin_api")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub kind: String,
    pub path_and_query: String,
    pub content_type: String,
    pub timeout: i32,
    pub callback: String,
    pub http_method: String,
    pub ext: String,
    pub save_message: bool,

    pub own_paths: String,
}

impl TardisActiveModel for ActiveModel {
    fn fill_ctx(&mut self, ctx: &TardisContext, is_insert: bool) {
        if is_insert {
            self.own_paths = Set(ctx.own_paths.to_string());
        }
    }

    fn create_table_statement(db: DbBackend) -> TableCreateStatement {
        let mut builder = Table::create();
        builder
            .table(Entity.table_ref())
            .if_not_exists()
            .col(ColumnDef::new(Column::Id).not_null().string().primary_key())
            .col(ColumnDef::new(Column::Kind).not_null().string())
            .col(ColumnDef::new(Column::PathAndQuery).not_null().string())
            .col(ColumnDef::new(Column::ContentType).not_null().string())
            .col(ColumnDef::new(Column::Timeout).not_null().integer())
            .col(ColumnDef::new(Column::Callback).not_null().string())
            .col(ColumnDef::new(Column::HttpMethod).not_null().string())
            .col(ColumnDef::new(Column::Ext).not_null().string())
            .col(ColumnDef::new(Column::SaveMessage).not_null().boolean())
            .col(ColumnDef::new(Column::OwnPaths).not_null().string());
        if db == DatabaseBackend::MySql {
            builder.engine("InnoDB").character_set("utf8mb4").collate("utf8mb4_0900_as_cs");
        }
        builder.to_owned()
    }

    fn create_index_statement() -> Vec<IndexCreateStatement> {
        vec![]
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
