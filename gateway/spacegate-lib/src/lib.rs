#![warn(clippy::unwrap_used)]

use crate::plugin::{anti_replay, anti_xss, audit_log, auth, ip_time};

mod plugin;

pub const PACKAGE_NAME: &str = "bios_spacegate";

pub fn register_lib_filter() {
    spacegate_kernel::register_filter_def(audit_log::SgFilterAuditLogDef);
    spacegate_kernel::register_filter_def(ip_time::SgFilterIpTimeDef);
    spacegate_kernel::register_filter_def(anti_replay::SgFilterAntiReplayDef);
    spacegate_kernel::register_filter_def(anti_xss::SgFilterAntiXSSDef);
    spacegate_kernel::register_filter_def(auth::SgFilterAuthDef);
}
