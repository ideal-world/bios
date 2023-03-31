use bios_basic::{process::ci_processor::AppKeyConfig, rbum::rbum_config::RbumConfig};
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
        }
    }
}
