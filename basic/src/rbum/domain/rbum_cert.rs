use tardis::basic::dto::TardisContext;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm::prelude::*;
use tardis::db::sea_orm::*;
use tardis::db::sea_query::{ColumnDef, Index, IndexCreateStatement, Table, TableCreateStatement};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "rbum_cert")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    // Specific
    pub ak: String,
    pub sk: String,
    pub ext: String,
    pub start_time: DateTime,
    pub end_time: DateTime,
    pub coexist_flag: String,
    pub status: String,
    pub rel_rbum_cert_conf_id: String,
    pub rel_rbum_item_id: String,
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
            .col(ColumnDef::new(Column::Ak).not_null().string())
            .col(ColumnDef::new(Column::Sk).not_null().string())
            .col(ColumnDef::new(Column::Ext).not_null().string())
            .col(ColumnDef::new(Column::StartTime).extra("DEFAULT CURRENT_TIMESTAMP".to_string()).date_time())
            .col(ColumnDef::new(Column::EndTime).extra("DEFAULT CURRENT_TIMESTAMP".to_string()).date_time())
            .col(ColumnDef::new(Column::CoexistFlag).not_null().string())
            .col(ColumnDef::new(Column::Status).not_null().string())
            .col(ColumnDef::new(Column::RelRbumCertConfId).not_null().string())
            .col(ColumnDef::new(Column::RelRbumItemId).not_null().string())
            // Basic
            .col(ColumnDef::new(Column::ScopePaths).not_null().string())
            .col(ColumnDef::new(Column::UpdaterId).not_null().string())
            .col(ColumnDef::new(Column::CreateTime).extra("DEFAULT CURRENT_TIMESTAMP".to_string()).date_time())
            .col(ColumnDef::new(Column::UpdateTime).extra("DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP".to_string()).date_time())
            .to_owned()
    }

    fn create_index_statement() -> Vec<IndexCreateStatement> {
        vec![
            Index::create().name(&format!("idx-{}-{}", Entity.table_name(), Column::ScopePaths.to_string())).table(Entity).col(Column::ScopePaths).to_owned(),
            Index::create().name(&format!("idx-{}-ak", Entity.table_name())).table(Entity).col(Column::RelRbumCertConfId).col(Column::Ak).to_owned(),
            Index::create().name(&format!("idx-{}-{}", Entity.table_name(), Column::RelRbumItemId.to_string())).table(Entity).col(Column::RelRbumItemId).to_owned(),
        ]
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
