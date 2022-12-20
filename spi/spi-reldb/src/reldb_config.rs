use bios_basic::rbum::rbum_config::RbumConfig;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct ReldbConfig {
    pub rbum: RbumConfig,
}

impl Default for ReldbConfig {
    fn default() -> Self {
        ReldbConfig { rbum: Default::default() }
    }
}
