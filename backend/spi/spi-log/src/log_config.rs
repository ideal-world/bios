use bios_basic::rbum::rbum_config::RbumConfig;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct LogConfig {
    pub rbum: RbumConfig
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EventClientConfig {
    pub base_url: String,
    pub event_bus_sk: String,
}

impl Default for EventClientConfig {
    fn default() -> Self {
        Self {
            base_url: "http://127.0.0.1:8080".to_string(),
            event_bus_sk: "".to_string(),
        }
    }
}
