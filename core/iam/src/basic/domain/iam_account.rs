use tardis::basic::dto::TardisContext;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm::*;
use tardis::db::sea_query::{ColumnDef, IndexCreateStatement, Table, TableCreateStatement};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "iam_account")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub icon: String,

    pub own_paths: String,
}

impl TardisActiveModel for ActiveModel {
    fn fill_cxt(&mut self, cxt: &TardisContext, is_insert: bool) {
        if is_insert {
            self.own_paths = Set(cxt.own_paths.to_string());
        }
    }

    fn create_table_statement(_: DbBackend) -> TableCreateStatement {
        Table::create()
            .table(Entity.table_ref())
            .if_not_exists()
            .col(ColumnDef::new(Column::Id).not_null().string().primary_key())
            .col(ColumnDef::new(Column::Icon).not_null().string())
            .col(ColumnDef::new(Column::OwnPaths).not_null().string())
            .to_owned()
    }

    fn create_index_statement() -> Vec<IndexCreateStatement> {
        vec![]
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
