use sea_orm::entity::prelude::*;
use sea_orm::ActiveModelBehavior;
use sea_orm::ActiveValue::Set;

use bios::BIOSFuns;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "test_app")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub name: String,
    pub tenant_id: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::tenant::Entity", from = "Column::TenantId", to = "super::tenant::Column::Id")]
    Tenant,
}

impl Related<super::tenant::Entity> for super::app::Entity {
    fn to() -> RelationDef {
        Relation::Tenant.def()
    }
}

impl Related<super::account::Entity> for super::app::Entity {
    fn to() -> RelationDef {
        super::app_account_rel::Relation::Account.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::app_account_rel::Relation::App.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            id: Set(BIOSFuns::field.uuid_str()),
            ..ActiveModelTrait::default()
        }
    }
}
