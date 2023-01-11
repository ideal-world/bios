use bios_basic::rbum::rbum_config::RbumConfig;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct CacheConfig {
    pub rbum: RbumConfig,
}

impl Default for CacheConfig {
    fn default() -> Self {
        CacheConfig { rbum: Default::default() }
    }
}
