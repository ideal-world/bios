use bios_basic::rbum::rbum_config::RbumConfig;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct GraphConfig {
    pub rbum: RbumConfig,
}

impl Default for GraphConfig {
    fn default() -> Self {
        GraphConfig { rbum: Default::default() }
    }
}
