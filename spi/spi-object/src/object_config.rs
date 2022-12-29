use bios_basic::rbum::rbum_config::RbumConfig;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct ObjectConfig {
    pub rbum: RbumConfig,
}

impl Default for ObjectConfig {
    fn default() -> Self {
        ObjectConfig { rbum: Default::default() }
    }
}
