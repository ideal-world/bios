use bios_basic::rbum::rbum_config::RbumConfig;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct PluginConfig {
    pub rbum: RbumConfig,
}
