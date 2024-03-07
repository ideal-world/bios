#![warn(clippy::unwrap_used)]

use crate::plugin::{anti_replay, anti_xss, audit_log, auth, ip_time, rewrite_ns_b_ip};

mod extension;
mod plugin;
pub const PACKAGE_NAME: &str = "spacegate_lib";
use spacegate_shell::plugin::SgPluginRepository;
pub fn register_lib_plugins(repo: &SgPluginRepository) {
    repo.register::<ip_time::SgIpTimePlugin>();
    repo.register::<anti_replay::AntiReplayPlugin>();
    repo.register::<anti_xss::AntiXssPlugin>();
    repo.register::<rewrite_ns_b_ip::RewriteNsPlugin>();
    repo.register::<audit_log::AuditLogPlugin>();
    repo.register::<auth::AuthPlugin>();
}
