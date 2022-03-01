use tardis::basic::dto::TardisContext;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm::*;
use tardis::db::sea_query::{ColumnDef, IndexCreateStatement, Table, TableCreateStatement};
use tardis::TardisFuns;

pub const RBUM_KIND_ID: &str = "iam_tenant";

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "iam_tenant")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
}

impl TardisActiveModel for ActiveModel {
    fn fill_cxt(&mut self, _: &TardisContext, is_insert: bool) {
        if is_insert {
            if self.id == ActiveValue::NotSet {
                self.id = Set(TardisFuns::field.uuid_str());
            }
        }
    }

    fn create_table_statement(_: DbBackend) -> TableCreateStatement {
        Table::create().table(Entity.table_ref()).if_not_exists().col(ColumnDef::new(Column::Id).not_null().string().primary_key()).to_owned()
    }

    fn create_index_statement() -> Vec<IndexCreateStatement> {
        vec![]
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
