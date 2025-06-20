#![warn(clippy::unwrap_used)]

pub use crate::plugin::{anti_replay, anti_xss, audit_log, auth, bios_error_limit, content_filter, ip_time, license, rewrite_ns_b_ip};

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
    repo.register::<bios_error_limit::BiosErrorLimitPlugin>();
    repo.register::<license::LicensePlugin>();
}

// fix `instrument` find tracing error [issue](https://github.com/tokio-rs/tracing/issues/3309)
use tardis::tracing::*;
extern crate self as tracing;
