use tardis::basic::dto::TardisContext;
use tardis::chrono::{self, Utc};
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm;
use tardis::db::sea_orm::prelude::*;
use tardis::db::sea_orm::sea_query::{ColumnDef, Index, IndexCreateStatement, Table, TableCreateStatement};
use tardis::db::sea_orm::*;
use tardis::{TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};
/// Resource set category model
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "rbum_set_cate")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    /// System (internal) code \
    /// using regular hierarchical code to avoid recursive tree queries
    #[index(index_id = "unique_sys_code", unique)]
    pub sys_code: String,
    /// Business code for custom
    #[index(index_id = "bus_code")]
    pub bus_code: String,
    pub name: String,
    pub icon: String,
    pub sort: i64,
    pub ext: String,
    /// Associated [resource set](crate::rbum::domain::rbum_set::Model) id
    #[index(index_id = "unique_sys_code", repeat(index_id = "bus_code"))]
    pub rel_rbum_set_id: String,

    pub scope_level: i16,

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
