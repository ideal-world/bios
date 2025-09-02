use bios_basic::rbum::rbum_config::RbumConfig;
use bios_sdk_invoke::invoke_config::InvokeConfig;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct PluginConfig {
    pub rbum: RbumConfig,
    pub invoke: InvokeConfig,
    pub kv_plugin_prefix: String,
}

impl Default for PluginConfig {
    fn default() -> Self {
        PluginConfig {
            rbum: RbumConfig::default(),
            invoke: InvokeConfig::default(),
            kv_plugin_prefix: "spi_plugin".to_string(),
        }
    }
}
