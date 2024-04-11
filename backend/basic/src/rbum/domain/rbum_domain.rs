use tardis::basic::dto::TardisContext;
use tardis::chrono::{self, Utc};
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm;
use tardis::db::sea_orm::prelude::*;
use tardis::db::sea_orm::sea_query::{ColumnDef, IndexCreateStatement, Table, TableCreateStatement};
use tardis::db::sea_orm::*;
use tardis::{TardisCreateEntity, TardisCreateIndex, TardisEmptyBehavior, TardisEmptyRelation};

/// Resource domain model
///
/// The resource domain is the unit of ownership of the resource, indicating the owner of the resource.
/// Each resource is required to belong to a resource domain.
///
/// E.g. All menu resources are provided by IAM components, and all IaaS resources are provided by CMDB components.
/// IAM components and CMDB components are resource domains.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "rbum_domain")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    /// Resource domain code, which is required to conform to the host specification in the uri, matching the regular: ^[a-z0-9-.]+$
    #[index(unique)]
    pub code: String,
    pub name: String,
    pub note: String,
    pub icon: String,
    pub sort: i64,

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
