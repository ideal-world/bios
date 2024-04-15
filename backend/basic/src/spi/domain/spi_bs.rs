use tardis::basic::dto::TardisContext;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm;
use tardis::db::sea_orm::sea_query::{ColumnDef, IndexCreateStatement, Table, TableCreateStatement};
use tardis::db::sea_orm::*;

/// SPI backend service model, extended from [`crate::rbum::domain::rbum_item`]
///
/// SPI后端服务模型，扩展于[`crate::rbum::domain::rbum_item`]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "spi_bs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    /// Is private. When it is private, this service is exclusive. Can only be used by one subject of request (tenant or application).
    ///
    /// 是否私有。当为私有时，这个服务为独占式的。只能给一个请求主体（租户或应用）使用。
    pub private: bool,

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
            .col(ColumnDef::new(Column::Private).not_null().boolean())
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
