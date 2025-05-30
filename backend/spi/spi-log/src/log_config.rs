use bios_basic::rbum::rbum_config::RbumConfig;
use bios_sdk_invoke::invoke_config::InvokeConfig;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct LogConfig {
    pub rbum: RbumConfig,
    pub invoke: InvokeConfig,
    pub cache_key_async_task_status: String,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            rbum: RbumConfig::default(),
            invoke: InvokeConfig::default(),
            cache_key_async_task_status: "iam:cache:task:status".to_string(),
        }
    }
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
