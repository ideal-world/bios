#![warn(clippy::unwrap_used)]

use crate::plugin::{anti_replay, anti_xss, audit_log, auth, ip_time, rewrite_ns_b_ip};

mod consts;
mod extension;
mod marker;
mod plugin;

pub const PACKAGE_NAME: &str = "spacegate_lib";
use plugin::op_redis_publisher;
use spacegate_shell::plugin::PluginRepository;
pub fn register_lib_plugins(repo: &PluginRepository) {
    repo.register::<ip_time::IpTimePlugin>();
    repo.register::<anti_replay::AntiReplayPlugin>();
    repo.register::<anti_xss::AntiXssPlugin>();
    repo.register::<rewrite_ns_b_ip::RewriteNsPlugin>();
    repo.register::<audit_log::AuditLogPlugin>();
    repo.register::<auth::AuthPlugin>();
    repo.register::<op_redis_publisher::RedisPublisherPlugin>();
}
