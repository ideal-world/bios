use bios_basic::{process::ci_processor::AppKeyConfig, rbum::rbum_config::RbumConfig};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, sync::Mutex};
use tardis::basic::{error::TardisError, result::TardisResult};
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct EventConfig {
    pub rbum: RbumConfig,
    pub app_key: AppKeyConfig,
    pub event_url: String,
    pub log_url: String,
}

impl Default for EventConfig {
    fn default() -> Self {
        EventConfig {
            rbum: Default::default(),
            app_key: Default::default(),
            event_url: "".to_string(),
            log_url: "".to_string(),
        }
    }
}

impl EventConfig {
    pub fn event_url(&self) -> String {
        if self.event_url.ends_with("/") {
            self.event_url.clone()
        } else {
            format!("{}/", self.event_url)
        }
    }

    pub fn log_url(&self) -> String {
        if self.log_url.ends_with("/") {
            self.log_url.clone()
        } else {
            format!("{}/", self.log_url)
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EventInfo {
    pub kind_id: String,
    pub domain_id: String,
}

lazy_static! {
    static ref EVENT_INFO: Mutex<Option<EventInfo>> = Mutex::new(None);
}

pub struct EventInfoManager;

impl EventInfoManager {
    pub fn set(event_info: EventInfo) -> TardisResult<()> {
        let mut conf = EVENT_INFO.lock().map_err(|e| TardisError::internal_error(&format!("{:?}", e), ""))?;
        *conf = Some(event_info);
        Ok(())
    }

    pub fn get_config<F, T>(fun: F) -> T
    where
        F: Fn(&EventInfo) -> T,
    {
        let conf = EVENT_INFO.lock().unwrap_or_else(|e| panic!("event info lock error: {:?}", e));
        let conf = conf.as_ref().unwrap_or_else(|| panic!("rbum config not set"));
        fun(conf)
    }
}
