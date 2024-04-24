use tardis::chrono::{self, Utc};
use tardis::db::sea_orm;
use tardis::db::sea_orm::prelude::*;
use tardis::db::sea_orm::*;
use tardis::{TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};

/// Resource model
///
/// Used to represent a specific resource,
/// Each resource corresponds to a [resource kind](crate::rbum::domain::rbum_kind::Model)  and [resource domain](crate::rbum::domain::rbum_domain::Model).
///
/// Each resource corresponds to a unique uri,
/// and the uri consists of `<resource kind>://<resource domain>/<resource code>`
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "rbum_item")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    /// Resource code
    #[index(index_id = "unique_id", unique)]
    pub code: String,
    pub name: String,
    /// Associated [resource kind](crate::rbum::domain::rbum_kind::Model) id
    #[index(repeat(index_id = "unique_id", unique))]
    pub rel_rbum_kind_id: String,
    /// Associated [resource domain](crate::rbum::domain::rbum_domain::Model) id
    #[index(repeat(index_id = "unique_id", unique))]
    pub rel_rbum_domain_id: String,

    pub scope_level: i16,

    #[index]
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

    pub disabled: bool,
}
