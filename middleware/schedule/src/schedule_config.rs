use bios_basic::{process::ci_processor::AppKeyConfig, rbum::rbum_config::RbumConfig};
use bios_sdk_invoke::clients::event_client::EventTopicConfig;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct ScheduleConfig {
    pub rbum: RbumConfig,
    pub app_key: AppKeyConfig,
    pub spi_app_id: String,
    pub kv_url: String,
    pub log_url: String,
    pub cache_key_job_changed_info: String,
    pub cache_key_job_changed_timer_sec: u32,
    /// The expire time of the distributed lock on a certain task, in seconds, defualt 1 seconds
    pub distributed_lock_expire_sec: u32,
    /// The expire key prefix of the distributed lock, default "schedual:job:lock:"
    pub distributed_lock_key_prefix: String,
    pub event: EventTopicConfig,
}

impl Default for ScheduleConfig {
    fn default() -> Self {
        ScheduleConfig {
            rbum: RbumConfig::default(),
            app_key: AppKeyConfig::default(),
            spi_app_id: "".to_string(),
            kv_url: "http://127.0.0.1:8080/spi-kv".to_string(),
            log_url: "http://127.0.0.1:8080/spi-log".to_string(),
            cache_key_job_changed_info: "spi:job:changed:info:".to_string(),
            cache_key_job_changed_timer_sec: 30,
            distributed_lock_expire_sec: 1,
            distributed_lock_key_prefix: "schedual:job:lock:".to_string(),
            event: EventTopicConfig::default()
        }
    }
}
