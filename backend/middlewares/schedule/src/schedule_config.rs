use bios_basic::{process::ci_processor::AppKeyConfig, rbum::rbum_config::RbumConfig};
use bios_sdk_invoke::invoke_config::InvokeConfig;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct ScheduleConfig {
    pub rbum: RbumConfig,
    pub app_key: AppKeyConfig,
    pub invoke: InvokeConfig,
    pub cache_key_job_changed_info: String,
    pub cache_key_job_changed_timer_sec: u32,
    /// The expire time of the distributed lock on a certain task, in seconds, defualt 1 seconds
    pub distributed_lock_expire_sec: u32,
    /// The expire key prefix of the distributed lock, default "schedual:job:lock:"
    pub distributed_lock_key_prefix: String,
    /// interval to force sync jobs from database
    pub force_sync_interval_sec: u32,
}

impl Default for ScheduleConfig {
    fn default() -> Self {
        ScheduleConfig {
            rbum: RbumConfig::default(),
            app_key: AppKeyConfig::default(),
            invoke: InvokeConfig::default(),
            cache_key_job_changed_info: "spi:job:changed:info:".to_string(),
            cache_key_job_changed_timer_sec: 30,
            distributed_lock_expire_sec: 30,
            force_sync_interval_sec: 30,
            distributed_lock_key_prefix: "schedual:job:lock:".to_string(),
        }
    }
}
