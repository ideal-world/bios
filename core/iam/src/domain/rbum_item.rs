use tardis::db::entity::prelude::*;
use tardis::db::prelude::DateTime;
use tardis::db::ActiveModelBehavior;
use tardis::db::ActiveValue::Set;
use tardis::TardisFuns;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "rbum_item")]
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
    pub uri_part: String,
    pub icon: String,
    pub sort: i32,

    pub disabled: bool,
    pub rel_rbum_kind_id: String,
    pub rel_rbum_domain_id: String,
}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            id: Set(TardisFuns::field.uuid_str()),
            ..ActiveModelTrait::default()
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
