use bios_basic::rbum::rbum_config::RbumConfig;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct ReldbConfig {
    pub rbum: RbumConfig,
    pub tx_clean_interval_sec: u8,
}

impl Default for ReldbConfig {
    fn default() -> Self {
        ReldbConfig {
            rbum: Default::default(),
            tx_clean_interval_sec: 5,
        }
    }
}
