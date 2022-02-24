use async_trait::async_trait;
use chrono::Utc;
use sea_orm::IntoActiveModel;
use tardis::basic::dto::TardisContext;
use tardis::db::entity::prelude::*;
use tardis::db::prelude::DateTime;
use tardis::db::ActiveModelBehavior;
use tardis::db::ActiveValue::Set;
use tardis::TardisFuns;

use crate::domain::BiosSeaORMExtend;
use crate::enumeration::RbumScopeKind;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "rbum_kind")]
pub struct Model {
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
    pub scope_kind: String,

    #[sea_orm(indexed)]
    pub code: String,
    pub name: String,
    pub note: String,
    pub icon: String,
    pub sort: i32,

    pub ext_table_name: String,
}

#[async_trait]
impl BiosSeaORMExtend for ActiveModel {
    type Entity = Entity;

    fn insert_cxt(&mut self, cxt: &TardisContext) {
        self.id = Set(TardisFuns::field.uuid_str());
        self.scope_kind = Set(RbumScopeKind::APP.to_string());
        self.rel_app_id = Set(cxt.app_id.to_string());
        self.rel_tenant_id = Set(cxt.tenant_id.to_string());
        self.creator_id = Set(cxt.account_id.to_string());
        self.updater_id = Set(cxt.account_id.to_string());
        self.create_time = Set(Utc::now().naive_utc());
        self.update_time = Set(Utc::now().naive_utc());
    }

    fn update_cxt(&mut self, cxt: &TardisContext) {
        self.updater_id = Set(cxt.account_id.to_string());
        self.update_time = Set(Utc::now().naive_utc());
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
