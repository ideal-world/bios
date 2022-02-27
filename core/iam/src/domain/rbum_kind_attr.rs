use tardis::basic::dto::TardisContext;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm::prelude::*;
use tardis::db::sea_orm::*;
use tardis::db::sea_query::{ColumnRef, Table, TableCreateStatement};
use tardis::TardisFuns;

use crate::enumeration::RbumScopeKind;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "rbum_kind_attr")]
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

    // Specific
    #[sea_orm(indexed)]
    pub code: String,
    pub name: String,
    pub note: String,
    pub sort: i32,
    pub main_column: bool,
    pub position: bool,
    pub capacity: bool,
    pub overload: bool,
    pub data_type_kind: String,
    pub widget_type: String,
    pub default_value: String,
    pub options: String,
    pub required: bool,
    pub min_length: u8,
    pub max_length: u8,
    pub action: String,
    pub rel_rbum_kind_id: String,
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
            // Specific
            .col(ColumnDef::new(Column::Code).not_null().string().unique_key())
            .col(ColumnDef::new(Column::Name).not_null().string())
            .col(ColumnDef::new(Column::note).not_null().string())
            .col(ColumnDef::new(Column::Sort).not_null().integer())
            .col(ColumnDef::new(Column::MainColumn).not_null().boolean())
            .col(ColumnDef::new(Column::Position).not_null().boolean())
            .col(ColumnDef::new(Column::Capacity).not_null().boolean())
            .col(ColumnDef::new(Column::Overload).not_null().boolean())
            .col(ColumnDef::new(Column::DataTypeKind).not_null().string())
            .col(ColumnDef::new(Column::WidgetType).not_null().string())
            .col(ColumnDef::new(Column::DefaultValue).not_null().string())
            .col(ColumnDef::new(Column::Options).not_null().text())
            .col(ColumnDef::new(Column::Required).not_null().boolean())
            .col(ColumnDef::new(Column::MinLength).not_null().integer())
            .col(ColumnDef::new(Column::MaxLength).not_null().integer())
            .col(ColumnDef::new(Column::Action).not_null().string())
            .col(ColumnDef::new(Column::RelRbumKindId).not_null().string())
            .to_owned()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
