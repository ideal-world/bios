use tardis::basic::dto::TardisContext;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm::prelude::*;
use tardis::db::sea_orm::*;
use tardis::db::sea_query::{ColumnDef, Index, IndexCreateStatement, Table, TableCreateStatement};
use tardis::TardisFuns;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "rbum_cert_conf")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    // Specific
    pub name: String,
    pub note: String,
    pub ak_note: String,
    pub ak_rule: String,
    pub sk_note: String,
    pub sk_rule: String,
    pub repeatable: bool,
    pub is_basic: bool,
    pub rest_by_kinds: String,
    pub expire_sec: i32,
    pub coexist_num: i32,
    pub rel_rbum_domain_id: String,
    // Basic
    pub rel_app_id: String,
    pub rel_tenant_id: String,
    pub updater_id: String,
    pub create_time: DateTime,
    pub update_time: DateTime,
}

impl TardisActiveModel for ActiveModel {
    fn fill_cxt(&mut self, cxt: &TardisContext, is_insert: bool) {
        if is_insert {
            self.id = Set(TardisFuns::field.uuid_str());
            self.rel_app_id = Set(cxt.app_id.to_string());
            self.rel_tenant_id = Set(cxt.tenant_id.to_string());
        }
        self.updater_id = Set(cxt.account_id.to_string());
    }

    fn create_table_statement(_: DbBackend) -> TableCreateStatement {
        Table::create()
            .table(Entity.table_ref())
            .if_not_exists()
            .col(ColumnDef::new(Column::Id).not_null().string().primary_key())
            // Specific
            .col(ColumnDef::new(Column::Name).not_null().string())
            .col(ColumnDef::new(Column::Note).not_null().string())
            .col(ColumnDef::new(Column::AkNote).not_null().string())
            .col(ColumnDef::new(Column::AkRule).not_null().string())
            .col(ColumnDef::new(Column::SkNote).not_null().string())
            .col(ColumnDef::new(Column::SkRule).not_null().string())
            .col(ColumnDef::new(Column::Repeatable).not_null().boolean())
            .col(ColumnDef::new(Column::IsBasic).not_null().boolean())
            .col(ColumnDef::new(Column::RestByKinds).not_null().string())
            .col(ColumnDef::new(Column::ExpireSec).not_null().integer())
            .col(ColumnDef::new(Column::CoexistNum).not_null().integer())
            .col(ColumnDef::new(Column::RelRbumDomainId).not_null().string())
            // Basic
            .col(ColumnDef::new(Column::RelAppId).not_null().string())
            .col(ColumnDef::new(Column::RelTenantId).not_null().string())
            .col(ColumnDef::new(Column::UpdaterId).not_null().string())
            .col(ColumnDef::new(Column::CreateTime).extra("DEFAULT CURRENT_TIMESTAMP".to_string()).date_time())
            .col(ColumnDef::new(Column::UpdateTime).extra("DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP".to_string()).date_time())
            .to_owned()
    }

    fn create_index_statement() -> Vec<IndexCreateStatement> {
        vec![Index::create()
            .name(&format!("idx-{}-{}-{}", Entity.table_name(), Column::RelAppId.to_string(), Column::RelTenantId.to_string()))
            .table(Entity)
            .col(Column::RelAppId)
            .col(Column::RelTenantId)
            .to_owned()]
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
