use bios_basic::rbum::rbum_config::RbumConfig;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct PluginConfig {
    pub rbum: RbumConfig,
}

impl Default for PluginConfig {
    fn default() -> Self {
        PluginConfig { rbum: Default::default() }
    }
}
