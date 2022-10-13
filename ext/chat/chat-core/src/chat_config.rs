use bios_basic::rbum::rbum_config::RbumConfig;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct ChatConfig {
    pub rbum: RbumConfig,
}

impl Default for ChatConfig {
    fn default() -> Self {
        ChatConfig { rbum: Default::default() }
    }
}
