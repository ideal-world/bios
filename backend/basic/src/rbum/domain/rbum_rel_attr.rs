use tardis::chrono::{self, Utc};
use tardis::db::sea_orm;
use tardis::db::sea_orm::prelude::*;
use tardis::db::sea_orm::*;
use tardis::{TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};

/// Relationship attribute condition model
///
/// This model is used to further qualify the conditions under which the relationship is established
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "rbum_rel_attr")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    /// Condition qualifier \
    /// if true, it means the limitation of the relationship source,
    /// otherwise it is the limitation of the relationship target resource
    pub is_from: bool,
    /// Attribute value
    pub value: String,
    /// Attribute name, redundant field
    pub name: String,
    /// Is it for record only \
    /// if true, this condition is only used for records and does not participate in the judgment of whether the relationship is established
    pub record_only: bool,
    /// Associated [resource kind attribute](crate::rbum::domain::rbum_kind_attr::Model) id
    pub rel_rbum_kind_attr_id: String,
    /// Associated [relationship](crate::rbum::domain::rbum_rel::Model) id
    #[index]
    pub rel_rbum_rel_id: String,

    #[fill_ctx(fill = "own_paths")]
    pub own_paths: String,
    #[fill_ctx]
    pub owner: String,
    #[sea_orm(extra = "DEFAULT CURRENT_TIMESTAMP")]
    pub create_time: chrono::DateTime<Utc>,
    #[sea_orm(extra = "DEFAULT CURRENT_TIMESTAMP")]
    pub update_time: chrono::DateTime<Utc>,
    #[fill_ctx]
    pub create_by: String,
    #[fill_ctx(insert_only = false)]
    pub update_by: String,
}
