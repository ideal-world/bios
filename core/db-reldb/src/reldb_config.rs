use bios_basic::rbum::rbum_config::RbumConfig;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct RelDbConfig {
    pub rbum: RbumConfig,
}

impl Default for RelDbConfig {
    fn default() -> Self {
        RelDbConfig { rbum: Default::default() }
    }
}
