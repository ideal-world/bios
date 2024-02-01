#![warn(clippy::unwrap_used)]

use crate::plugin::{anti_replay, anti_xss, ip_time, rewrite_ns_b_ip};

mod extension;
mod plugin;
pub const PACKAGE_NAME: &str = "spacegate_lib";
use spacegate_shell::plugin::SgPluginRepository;
pub fn register_lib_plugins(repo: &SgPluginRepository) {
    repo.register::<ip_time::SgIpTimePlugin>();
    repo.register::<anti_replay::AntiReplayPlugin>();
    repo.register::<anti_xss::AntiXssPlugin>();
    repo.register::<rewrite_ns_b_ip::RewriteNsPlugin>();
    // spacegate_shell::register_filter_def(audit_log::SgFilterAuditLogDef);
    // spacegate_shell::register_filter_def(ip_time::SgFilterIpTimeDef);
    // spacegate_shell::register_filter_def(anti_replay::SgFilterAntiReplayDef);
    // spacegate_shell::register_filter_def(anti_xss::SgFilterAntiXSSDef);
    // spacegate_shell::register_filter_def(auth::SgFilterAuthDef);
    // spacegate_shell::register_filter_def(rewrite_ns_b_ip::SgFilterRewriteNsDef);
}
