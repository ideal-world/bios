use tardis::basic::dto::TardisContext;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm::prelude::*;
use tardis::db::sea_orm::*;
use tardis::db::sea_query::{ColumnDef, Index, IndexCreateStatement, Table, TableCreateStatement};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "rbum_rel_attr")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    // Specific
    pub is_from: bool,
    pub value: String,
    pub name: String,
    pub rel_rbum_kind_attr_id: String,
    pub rel_rbum_rel_id: String,
    // Basic
    pub rel_app_code: String,
    pub updater_code: String,
    pub create_time: DateTime,
    pub update_time: DateTime,
}

impl TardisActiveModel for ActiveModel {
    fn fill_cxt(&mut self, cxt: &TardisContext, is_insert: bool) {
        if is_insert {
            self.rel_app_code = Set(cxt.app_code.to_string());
        }
        self.updater_code = Set(cxt.account_code.to_string());
    }

    fn create_table_statement(_: DbBackend) -> TableCreateStatement {
        Table::create()
            .table(Entity.table_ref())
            .if_not_exists()
            .col(ColumnDef::new(Column::Id).not_null().string().primary_key())
            // Specific
            .col(ColumnDef::new(Column::IsFrom).not_null().boolean())
            .col(ColumnDef::new(Column::Value).not_null().string())
            .col(ColumnDef::new(Column::Name).not_null().string())
            .col(ColumnDef::new(Column::RelRbumKindAttrId).not_null().string())
            .col(ColumnDef::new(Column::RelRbumRelId).not_null().string())
            // Basic
            .col(ColumnDef::new(Column::RelAppCode).not_null().string())
            .col(ColumnDef::new(Column::UpdaterCode).not_null().string())
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
