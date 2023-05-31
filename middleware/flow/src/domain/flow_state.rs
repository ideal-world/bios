use tardis::{TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};
use tardis::basic::dto::TardisContext;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm;
use tardis::db::sea_orm::prelude::Json;
use tardis::db::sea_orm::*;

/// Similar to `Task` in BPMN
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "flow_state")]
pub struct Model {
    // Basic
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub icon: String,
    pub info: String,
    #[index(index_id = "idx_201")]
    pub sys_state: String,
    // E.g.
    // Simple: Do anything
    // Form: Fill in the form
    // Mail: Send an email
    // Callback: Callback url
    // Script: Execute a script
    // ......
    #[index(index_id = "idx_202")]
    pub state_kind: String,
    
    pub vars: Json,
    pub kind_conf: Json,

    #[index(index_id = "idx_203")]
    pub template: bool,
    #[index(index_id = "idx_204")]
    pub rel_state_id: String,

    #[index(index_id = "idx_205")]
    pub tag: String,

    #[fill_ctx(own_paths)]
    pub own_paths: String,
}
