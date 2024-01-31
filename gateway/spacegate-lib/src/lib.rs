#![warn(clippy::unwrap_used)]

use crate::plugin::{anti_replay, anti_xss, audit_log, auth, ip_time, rewrite_ns_b_ip};

mod plugin;
mod extension;
pub const PACKAGE_NAME: &str = "spacegate_lib";

pub fn register_lib_filter() {
    spacegate_shell::register_filter_def(audit_log::SgFilterAuditLogDef);
    spacegate_shell::register_filter_def(ip_time::SgFilterIpTimeDef);
    spacegate_shell::register_filter_def(anti_replay::SgFilterAntiReplayDef);
    spacegate_shell::register_filter_def(anti_xss::SgFilterAntiXSSDef);
    spacegate_shell::register_filter_def(auth::SgFilterAuthDef);
    spacegate_shell::register_filter_def(rewrite_ns_b_ip::SgFilterRewriteNsDef);
}
