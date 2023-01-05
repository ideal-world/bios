use bios_basic::rbum::rbum_config::RbumConfig;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct LogConfig {
    pub rbum: RbumConfig,
}

impl Default for LogConfig {
    fn default() -> Self {
        LogConfig { rbum: Default::default() }
    }
}
