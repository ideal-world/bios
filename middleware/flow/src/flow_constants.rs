use tardis::{TardisFuns, TardisFunsInst};

pub const DOMAIN_CODE: &str = "flow";
pub const RBUM_KIND_STATE_CODE: &str = "fw-state";
pub const RBUM_EXT_TABLE_STATE: &str = "flow_state";
pub const RBUM_KIND_MODEL_CODE: &str = "fw-model";
pub const RBUM_EXT_TABLE_MODEL: &str = "flow_model";

pub const EVENT_FRONT_CHANGE: &str = "event_front_change";
pub const EVENT_POST_CHANGE: &str = "event_post_change";
pub const EVENT_UPDATE_STATE: &str = "event_update_state";
pub const EVENT_MODIFY_FIELD: &str = "event_modify_field";
pub const EVENT_MODIFY_ASSIGNED: &str = "event_modify_assigned";

pub fn get_tardis_inst() -> TardisFunsInst {
    TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None)
}
