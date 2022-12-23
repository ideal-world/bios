use bios_basic::rbum::rbum_config::RbumConfig;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct SearchConfig {
    pub rbum: RbumConfig,
}

impl Default for SearchConfig {
    fn default() -> Self {
        SearchConfig { rbum: Default::default() }
    }
}
