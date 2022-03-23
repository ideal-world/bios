use tardis::basic::dto::TardisContext;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm::prelude::*;
use tardis::db::sea_orm::*;
use tardis::db::sea_query::{ColumnDef, Index, IndexCreateStatement, Table, TableCreateStatement};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "rbum_rel")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    // Specific
    pub tag: String,
    pub from_rbum_item_id: String,
    pub to_rbum_item_id: String,
    pub to_scope_ids: String,
    pub ext: String,
    // Basic
    pub scope_ids: String,
    pub updater_id: String,
    pub create_time: DateTime,
    pub update_time: DateTime,
}

impl TardisActiveModel for ActiveModel {
    fn fill_cxt(&mut self, cxt: &TardisContext, is_insert: bool) {
        if is_insert {
            self.scope_ids = Set(cxt.scope_ids.to_string());
        }
        self.updater_id = Set(cxt.account_id.to_string());
    }

    fn create_table_statement(_: DbBackend) -> TableCreateStatement {
        Table::create()
            .table(Entity.table_ref())
            .if_not_exists()
            .col(ColumnDef::new(Column::Id).not_null().string().primary_key())
            // Specific
            .col(ColumnDef::new(Column::Tag).not_null().string())
            .col(ColumnDef::new(Column::FromRbumItemId).not_null().string())
            .col(ColumnDef::new(Column::ToRbumItemId).not_null().string())
            .col(ColumnDef::new(Column::ToScopeIds).not_null().string())
            .col(ColumnDef::new(Column::Ext).not_null().string())
            // Basic
            .col(ColumnDef::new(Column::ScopeIds).not_null().string())
            .col(ColumnDef::new(Column::UpdaterId).not_null().string())
            .col(ColumnDef::new(Column::CreateTime).extra("DEFAULT CURRENT_TIMESTAMP".to_string()).date_time())
            .col(ColumnDef::new(Column::UpdateTime).extra("DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP".to_string()).date_time())
            .to_owned()
    }

    fn create_index_statement() -> Vec<IndexCreateStatement> {
        vec![
            Index::create().name(&format!("idx-{}-{}", Entity.table_name(), Column::ScopeIds.to_string())).table(Entity).col(Column::ScopeIds).to_owned(),
            Index::create().name(&format!("idx-{}-from", Entity.table_name())).table(Entity).col(Column::Tag).col(Column::FromRbumItemId).to_owned(),
            Index::create().name(&format!("idx-{}-to", Entity.table_name())).table(Entity).col(Column::Tag).col(Column::ToRbumItemId).to_owned(),
        ]
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
