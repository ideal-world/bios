use tardis::basic::dto::TardisContext;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm::prelude::*;
use tardis::db::sea_orm::*;
use tardis::db::sea_query::{ColumnDef, Index, IndexCreateStatement, Table, TableCreateStatement};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "rbum_item")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    // Specific
    pub code: String,
    pub uri_path: String,
    pub name: String,
    pub icon: String,
    pub sort: i32,
    pub rel_rbum_kind_id: String,
    pub rel_rbum_domain_id: String,
    // Basic
    pub rel_app_code: String,
    pub updater_code: String,
    pub create_time: DateTime,
    pub update_time: DateTime,
    // With Scope
    pub scope_kind: String,
    // With Status
    pub disabled: bool,
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
            .col(ColumnDef::new(Column::Code).not_null().string())
            .col(ColumnDef::new(Column::Name).not_null().string())
            .col(ColumnDef::new(Column::UriPath).not_null().string())
            .col(ColumnDef::new(Column::Icon).not_null().string())
            .col(ColumnDef::new(Column::Sort).not_null().integer())
            .col(ColumnDef::new(Column::RelRbumKindId).not_null().string())
            .col(ColumnDef::new(Column::RelRbumDomainId).not_null().string())
            // Basic
            .col(ColumnDef::new(Column::RelAppCode).not_null().string())
            .col(ColumnDef::new(Column::UpdaterCode).not_null().string())
            .col(ColumnDef::new(Column::CreateTime).extra("DEFAULT CURRENT_TIMESTAMP".to_string()).date_time())
            .col(ColumnDef::new(Column::UpdateTime).extra("DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP".to_string()).date_time())
            // With Scope
            .col(ColumnDef::new(Column::ScopeKind).not_null().string())
            // With Status
            .col(ColumnDef::new(Column::Disabled).not_null().boolean())
            .to_owned()
    }

    fn create_index_statement() -> Vec<IndexCreateStatement> {
        vec![
            Index::create().name(&format!("idx-{}-{}", Entity.table_name(), Column::RelAppCode.to_string())).table(Entity).col(Column::RelAppCode).to_owned(),
            Index::create()
                .name(&format!("idx-{}-{}-{}", Entity.table_name(), Column::RelAppCode.to_string(), Column::Code.to_string()))
                .table(Entity)
                .col(Column::RelAppCode)
                .col(Column::Code)
                .unique()
                .to_owned(),
            Index::create().name(&format!("idx-{}-{}", Entity.table_name(), Column::UpdaterCode.to_string())).table(Entity).col(Column::UpdaterCode).to_owned(),
            Index::create().name(&format!("idx-{}-{}", Entity.table_name(), Column::ScopeKind.to_string())).table(Entity).col(Column::ScopeKind).to_owned(),
            Index::create().name(&format!("idx-{}-{}", Entity.table_name(), Column::Disabled.to_string())).table(Entity).col(Column::Disabled).to_owned(),
            Index::create().name(&format!("idx-{}-{}", Entity.table_name(), Column::Name.to_string())).table(Entity).col(Column::Name).to_owned(),
        ]
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
