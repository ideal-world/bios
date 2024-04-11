use tardis::basic::dto::TardisContext;
use tardis::chrono::{self, Utc};
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm;
use tardis::db::sea_orm::prelude::*;
use tardis::db::sea_orm::sea_query::{ColumnDef, IndexCreateStatement, Table, TableCreateStatement};
use tardis::db::sea_orm::*;
use tardis::{TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};

/// Resource kind
///
/// A resource kind is a set of common resources. \
/// E.g. `/tenant/**` , `/app/**` these are all APIs, and these are all API-kind resources; `/tenant/list` ,
/// `/tenant/detail#more` these are all menus, and these are all  menu-kind resources.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "rbum_kind")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    /// Resource kind code, which is required to conform to the scheme specification in the uri, matching the regular: ^[a-z0-9-.]+$
    #[index(unique)]
    pub code: String,
    pub name: String,
    pub note: String,
    pub icon: String,
    pub sort: i64,
    pub module: String,
    /// Each resource kind can specify an extension table for storing customized data
    pub ext_table_name: String,

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
}
