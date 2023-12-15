use bios_basic::rbum::rbum_config::RbumConfig;
use bios_sdk_invoke::clients::event_client::EventTopicConfig;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct LogConfig {
    pub rbum: RbumConfig,
    pub event: Option<EventTopicConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EventClientConfig {
    pub base_url: String,
    pub event_bus_sk: String,
}

impl Default for EventClientConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8080".to_string(),
            event_bus_sk: "".to_string(),
        }
    }
}
