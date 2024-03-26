use spacegate_plugin::{plugins, Plugin, PluginSchemaExt};
use tardis::serde_json;
fn export_plugin<P: PluginSchemaExt + Plugin>(dir: std::path::PathBuf) {
    let schema = P::schema();
    let json = serde_json::to_string_pretty(&schema).unwrap();
    let filename = format!("{}.json", P::CODE);
    let path = dir.join(filename);
    std::fs::write(path, json).unwrap();
}

macro_rules! export_plugins {
    ($path: literal : $($plugin:ty)*) => {
        let dir = std::path::PathBuf::from($path);
        std::fs::create_dir_all(&dir).unwrap();
        $(export_plugin::<$plugin>(dir.clone());)*
    };
}

#[test]
fn export_schema() {
    use spacegate_lib::plugin::{
        anti_replay::AntiReplayPlugin, anti_xss::AntiXssPlugin, audit_log::AuditLogPlugin, auth::AuthPlugin, ip_time::SgIpTimePlugin, rewrite_ns_b_ip::RewriteNsPlugin,
    };
    export_plugins!("schema":
        AntiReplayPlugin
        AntiXssPlugin
        AuditLogPlugin
        // AuthPlugin
        // SgIpTimePlugin
        RewriteNsPlugin
    );
}
