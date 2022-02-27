use tardis::basic::dto::TardisContext;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm::*;
use tardis::db::sea_orm::prelude::*;
use tardis::db::sea_query::{ColumnDef, Table, TableCreateStatement};
use tardis::TardisFuns;

use crate::enumeration::RbumScopeKind;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "rbum_item")]
pub struct Model {
    // Basic
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    #[sea_orm(indexed)]
    pub rel_app_id: String,
    #[sea_orm(indexed)]
    pub rel_tenant_id: String,
    pub creator_id: String,
    pub updater_id: String,
    pub create_time: DateTime,
    pub update_time: DateTime,

    // With Scope
    pub scope_kind: String,

    // With Status
    pub disabled: bool,

    // Specific
    #[sea_orm(indexed)]
    pub code: String,
    pub name: String,
    pub uri_part: String,
    pub icon: String,
    pub sort: i32,
    pub rel_rbum_kind_id: String,
    pub rel_rbum_domain_id: String,
}

impl TardisActiveModel for ActiveModel {
    type Entity = Entity;

    fn fill_cxt(&mut self, cxt: &TardisContext, is_insert: bool) {
        if is_insert {
            self.id = Set(TardisFuns::field.uuid_str());
            self.rel_app_id = Set(cxt.app_id.to_string());
            self.rel_tenant_id = Set(cxt.tenant_id.to_string());
            self.creator_id = Set(cxt.account_id.to_string());
            self.updater_id = Set(cxt.account_id.to_string());
            self.scope_kind = Set(RbumScopeKind::APP.to_string());
        } else {
            self.updater_id = Set(cxt.account_id.to_string());
        }
    }

    fn create_table_statement(_: DbBackend) -> TableCreateStatement {
        Table::create()
            .table(Entity.table_ref())
            .if_not_exists()
            // Basic
            .col(ColumnDef::new(Column::Id).not_null().string().primary_key())
            .col(ColumnDef::new(Column::RelAppId).not_null().string())
            .col(ColumnDef::new(Column::RelTenantId).not_null().string())
            .col(ColumnDef::new(Column::CreatorId).not_null().string())
            .col(ColumnDef::new(Column::CreatorId).not_null().string())
            .col(ColumnDef::new(Column::CreateTime).extra("DEFAULT CURRENT_TIMESTAMP".to_string()).date_time())
            .col(ColumnDef::new(Column::UpdateTime).extra("DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP".to_string()).date_time())
            // With Scope
            .col(ColumnDef::new(Column::ScopeKind).not_null().string())
            // With Status
            .col(ColumnDef::new(Column::Disabled).not_null().boolean())
            // Specific
            .col(ColumnDef::new(Column::Code).not_null().string().unique_key())
            .col(ColumnDef::new(Column::Name).not_null().string())
            .col(ColumnDef::new(Column::UriPart).not_null().string())
            .col(ColumnDef::new(Column::Icon).not_null().string())
            .col(ColumnDef::new(Column::Sort).not_null().integer())
            .col(ColumnDef::new(Column::RelRbumKindId).not_null().string())
            .col(ColumnDef::new(Column::RelRbumDomainId).not_null().string())
            .to_owned()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
