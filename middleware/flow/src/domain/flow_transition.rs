use tardis::basic::dto::TardisContext;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm;
use tardis::db::sea_orm::prelude::Json;
use tardis::db::sea_orm::*;
use tardis::{TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "flow_transition")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub name: String,

    pub from_flow_state_id: String,
    pub to_flow_state_id: String,

    // TODO
    pub transfer_by_auto: bool,
    // TODO
    pub transfer_by_timer: String,

    // guard similar to `Gateway` in BPMN
    pub guard_by_creator: bool,
    pub guard_by_his_operators: bool,
    pub guard_by_spec_account_ids: Vec<String>,
    pub guard_by_spec_role_ids: Vec<String>,
    pub guard_by_other_conds: Json,

    pub vars_collect: Json,

    // TODO
    // action similar to `Event` in BPMN
    pub action_by_pre_callback: String,
    // TODO
    pub action_by_post_callback: String,

    pub rel_flow_model_id: String,

    #[fill_ctx(own_paths)]
    pub own_paths: String,
}
