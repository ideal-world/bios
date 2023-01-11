use bios_basic::rbum::rbum_config::RbumConfig;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct KvConfig {
    pub rbum: RbumConfig,
}

impl Default for KvConfig {
    fn default() -> Self {
        KvConfig { rbum: Default::default() }
    }
}
