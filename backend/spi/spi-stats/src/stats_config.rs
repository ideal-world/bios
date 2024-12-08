use bios_basic::rbum::rbum_config::RbumConfig;
use bios_sdk_invoke::invoke_config::InvokeConfig;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct StatsConfig {
    pub rbum: RbumConfig,
    pub base_url: String,
    pub invoke: InvokeConfig,
    pub cache_key_async_task_status: String,
}

impl Default for StatsConfig {
    fn default() -> Self {
        StatsConfig {
            rbum: Default::default(),
            invoke: InvokeConfig::default(),
            cache_key_async_task_status: "iam:cache:task:status".to_string(),
            base_url: "http://127.0.0.1:8080/spi-stats".to_string(),
        }
    }
}
