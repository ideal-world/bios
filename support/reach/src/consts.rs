use std::sync::OnceLock;

use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;
use tardis::{TardisFuns, TardisFunsInst};
pub const DOMAIN_CODE: &str = "reach";
pub const MODULE_CODE: &str = "reach";
pub const RBUM_KIND_CODE_REACH_MESSAGE: &str = "reach-message";
pub const RBUM_EXT_TABLE_REACH_MESSAGE: &str = "reach_message";
pub const RBUM_SET_SCHEME_REACH: &str = "reach_set_";
pub const MQ_REACH_TOPIC_MESSAGE: &str = "starsys::reach::topic::message";

pub const RBUM_SCOPE_LEVEL_PRIVATE: RbumScopeLevelKind = RbumScopeLevelKind::Private;
pub const RBUM_SCOPE_LEVEL_GLOBAL: RbumScopeLevelKind = RbumScopeLevelKind::Root;
pub const RBUM_SCOPE_LEVEL_TENANT: RbumScopeLevelKind = RbumScopeLevelKind::L1;
pub const RBUM_SCOPE_LEVEL_APP: RbumScopeLevelKind = RbumScopeLevelKind::L2;

pub const REACH_INIT_OWNER: &str = "ReachInit";

pub const IAM_KEY_PHONE_V_CODE: &str = "PhoneVCode";

pub const ACCOUNT_SPLIT: char = ';';

pub static DOMAIN_REACH_ID: OnceLock<String> = OnceLock::new();

pub fn get_domain_reach_id() -> &'static str {
    DOMAIN_REACH_ID.get().expect("get domain id before it's initialized")
}

pub fn get_tardis_inst() -> TardisFunsInst {
    TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None)
}
