use bios_basic::rbum::rbum_config::RbumConfig;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct KvConfig {
    pub rbum: RbumConfig,
    pub cache_key_async_task_status: String,
}

impl Default for KvConfig {
    fn default() -> Self {
        KvConfig {
            rbum: Default::default(),
            cache_key_async_task_status: "iam:cache:task:status".to_string(),
        }
    }
}
