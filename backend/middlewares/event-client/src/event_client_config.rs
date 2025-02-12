use bios_sdk_invoke::invoke_config::InvokeConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct EventClientConfig {
    pub max_retry_times: Option<usize>,
    pub enable: bool,
    pub retry_duration_ms: u32,
    pub invoke: InvokeConfig,
}

impl Default for EventClientConfig {
    fn default() -> Self {
        Self {
            max_retry_times: None,
            enable: false,
            retry_duration_ms: 5000,
            invoke: InvokeConfig::default(),
        }
    }
}
