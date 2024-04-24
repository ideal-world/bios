use tardis::chrono::{self, Utc};
use tardis::db::sea_orm;
use tardis::db::sea_orm::prelude::*;
use tardis::db::sea_orm::*;
use tardis::{TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};
/// Relationship model
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "rbum_rel")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    /// Relationship label
    #[index(index_id = "from_index", repeat(index_id = "to_index"))]
    pub tag: String,
    pub note: String,
    /// The [source kind](crate::rbum::rbum_enumeration::RbumRelFromKind) of the relationship
    #[index(index_id = "from_index")]
    pub from_rbum_kind: i16,
    /// The source id of the relationship
    #[index(index_id = "from_index")]
    pub from_rbum_id: String,
    /// The target resource id of the relationship
    #[index(index_id = "to_index")]
    pub to_rbum_item_id: String,
    pub to_own_paths: String,
    /// Extended Information  \
    /// E.g. the record from or to is in another service, to avoid remote calls, you can redundantly add the required information to this field.
    pub ext: String,

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
}
