use tardis::{TardisFuns, TardisFunsInst};

pub const DOMAIN_CODE: &str = "flow";
pub const RBUM_KIND_STATE_CODE: &str = "fw-state";
pub const RBUM_EXT_TABLE_STATE: &str = "flow_state";
pub const RBUM_KIND_MODEL_CODE: &str = "fw-model";
pub const RBUM_EXT_TABLE_MODEL: &str = "flow_model";

pub fn get_tardis_inst() -> TardisFunsInst {
    TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None)
}
