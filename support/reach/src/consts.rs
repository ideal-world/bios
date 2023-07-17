use std::sync::OnceLock;

use tardis::{TardisFuns, TardisFunsInst};
pub const DOMAIN_CODE: &str = "reach";
pub const RBUM_KIND_CODE_REACH_MESSAGE: &str = "reach-message";
pub const RBUM_EXT_TABLE_REACH_MESSAGE: &str = "reach_message";
pub const RBUM_SET_SCHEME_REACH: &str = "reach_set_";
pub const MQ_REACH_TOPIC_MESSAGE: &str = "starsys::reach::topic::message";
pub static DOMAIN_REACH_ID: OnceLock<String> = OnceLock::new();

pub fn get_domain_reach_id() -> &'static str {
    DOMAIN_REACH_ID.get().expect("get domain id before it's initialized")
}

pub fn get_tardis_inst() -> TardisFunsInst {
    TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None)
}
