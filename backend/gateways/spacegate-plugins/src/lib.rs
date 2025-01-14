#![warn(clippy::unwrap_used)]

pub use crate::plugin::{anti_replay, anti_xss, audit_log, auth, ip_time, rewrite_ns_b_ip, content_filter};

mod consts;
mod extension;
mod marker;
mod plugin;
mod utils;

pub const PACKAGE_NAME: &str = "spacegate_lib";
use plugin::{notify, op_redis_publisher};
use spacegate_shell::plugin::PluginRepository;
pub fn register_lib_plugins(repo: &PluginRepository) {
    repo.register::<ip_time::IpTimePlugin>();
    repo.register::<anti_replay::AntiReplayPlugin>();
    repo.register::<anti_xss::AntiXssPlugin>();
    repo.register::<rewrite_ns_b_ip::RewriteNsPlugin>();
    repo.register::<audit_log::AuditLogPlugin>();
    repo.register::<auth::AuthPlugin>();
    repo.register::<op_redis_publisher::RedisPublisherPlugin>();
    repo.register::<notify::NotifyPlugin>();
    repo.register::<content_filter::ContentFilterPlugin>();
}
