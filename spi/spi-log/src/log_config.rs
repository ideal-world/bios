use bios_basic::rbum::rbum_config::RbumConfig;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct LogConfig {
    pub rbum: RbumConfig,
    pub event: Option<EventClientConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EventClientConfig {
    pub base_url: String,
    pub log_sk: String,
}

impl Default for EventClientConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8080".to_string(),
            log_sk: "".to_string(),
        }
    }
}
