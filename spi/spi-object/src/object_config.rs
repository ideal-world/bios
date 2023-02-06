use bios_basic::rbum::rbum_config::RbumConfig;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Serialize, Deserialize,Default, Clone)]
#[serde(default)]
pub struct ObjectConfig {
    pub rbum: RbumConfig,
}


