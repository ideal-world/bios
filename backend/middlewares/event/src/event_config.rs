use bios_basic::rbum::rbum_config::RbumConfig;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, sync::Mutex};
use tardis::basic::{error::TardisError, result::TardisResult};
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct EventConfig {
    pub rbum: RbumConfig,
    pub enable: bool,
    pub svc: String,
    pub event_url: String,
    pub base_url: String,
    pub event_bus_sk: String,
    pub avatars: Vec<String>,
    pub resend_threshold: u32,
    pub resend_interval_sec: Option<u32>,
}

impl Default for EventConfig {
    fn default() -> Self {
        EventConfig {
            rbum: Default::default(),
            enable: false,
            base_url: "http://localhost:8080/event".to_string(),
            svc: "bios".to_string(),
            avatars: Vec::new(),
            event_url: "".to_string(),
            event_bus_sk: "".to_string(),
            resend_threshold: 3,
            resend_interval_sec: None,
        }
    }
}

impl EventConfig {
    pub fn event_url(&self) -> String {
        if self.event_url.ends_with('/') {
            self.event_url.clone()
        } else {
            format!("{}/", self.event_url)
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
        let mut conf = EVENT_INFO.lock().map_err(|e| TardisError::internal_error(&format!("{e:?}"), ""))?;
        *conf = Some(event_info);
        Ok(())
    }

    pub fn get_config<F, T>(fun: F) -> T
    where
        F: Fn(&EventInfo) -> T,
    {
        let conf = EVENT_INFO.lock().unwrap_or_else(|e| panic!("event info lock error: {e:?}"));
        let conf = conf.as_ref().unwrap_or_else(|| panic!("rbum config not set"));
        fun(conf)
    }
}
