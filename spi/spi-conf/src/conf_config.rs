use bios_basic::rbum::rbum_config::RbumConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct ConfConfig { 
    pub rbum: RbumConfig,
}