use tardis::basic::dto::TardisContext;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm::prelude::*;
use tardis::db::sea_orm::*;
use tardis::db::sea_query::{ColumnDef, Index, IndexCreateStatement, Table, TableCreateStatement};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "rbum_set_cate_item")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    // Specific
    pub rel_rbum_set_id: String,
    pub rel_rbum_set_cate_code: String,
    pub rel_rbum_item_id: String,
    pub sort: i32,
    // Basic
    pub scope_paths: String,
    pub updater_id: String,
    pub create_time: DateTime,
    pub update_time: DateTime,
}

impl TardisActiveModel for ActiveModel {
    fn fill_cxt(&mut self, cxt: &TardisContext, is_insert: bool) {
        if is_insert {
            self.scope_paths = Set(cxt.scope_paths.to_string());
        }
        self.updater_id = Set(cxt.account_id.to_string());
    }

    fn create_table_statement(_: DbBackend) -> TableCreateStatement {
        Table::create()
            .table(Entity.table_ref())
            .if_not_exists()
            .col(ColumnDef::new(Column::Id).not_null().string().primary_key())
            // Specific
            .col(ColumnDef::new(Column::RelRbumSetId).not_null().string())
            .col(ColumnDef::new(Column::RelRbumSetCateCode).not_null().string())
            .col(ColumnDef::new(Column::RelRbumItemId).not_null().string())
            .col(ColumnDef::new(Column::Sort).not_null().integer())
            // Basic
            .col(ColumnDef::new(Column::ScopePaths).not_null().string())
            .col(ColumnDef::new(Column::UpdaterId).not_null().string())
            .col(ColumnDef::new(Column::CreateTime).extra("DEFAULT CURRENT_TIMESTAMP".to_string()).date_time())
            .col(ColumnDef::new(Column::UpdateTime).extra("DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP".to_string()).date_time())
            .to_owned()
    }

    fn create_index_statement() -> Vec<IndexCreateStatement> {
        vec![
            Index::create()
                .name(&format!("idx-{}-{}", Entity.table_name(), Column::RelRbumSetId.to_string()))
                .table(Entity)
                .col(Column::RelRbumSetId)
                .col(Column::RelRbumSetCateCode)
                .to_owned(),
            Index::create().name(&format!("idx-{}-{}", Entity.table_name(), Column::RelRbumItemId.to_string())).table(Entity).col(Column::RelRbumItemId).to_owned(),
        ]
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
