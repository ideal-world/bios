use spacegate_shell::plugin::Plugin;
use tardis::{TardisFuns, TardisFunsInst};

pub mod anti_replay;
pub mod anti_xss;
pub mod audit_log;
pub mod auth;
pub mod bios_error_limit;
pub mod content_filter;
pub mod ip_time;
pub mod license;
pub mod notify;
pub mod op_redis;
pub mod op_redis_publisher;
pub mod rewrite_ns_b_ip;
pub trait PluginBiosExt {
    fn get_funs_inst_by_plugin_code() -> TardisFunsInst;
}

impl<T: Plugin> PluginBiosExt for T {
    fn get_funs_inst_by_plugin_code() -> TardisFunsInst {
        TardisFuns::inst(Self::CODE, None)
    }
}
