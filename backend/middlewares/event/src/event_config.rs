use bios_basic::rbum::rbum_config::RbumConfig;

use bios_sdk_invoke::invoke_config::InvokeConfig;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, sync::Mutex};
use tardis::{
    basic::{error::TardisError, result::TardisResult},
    tardis_static,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct EventConfig {
    pub rbum: RbumConfig,
    pub enable: bool,
    pub svc: String,
    pub raft: Option<RaftConfig>,
    // default by 5000ms
    pub startup_timeout: u64,
    pub durable: bool,
    pub avatars: Vec<String>,
    pub cluster: Option<String>,
    pub invoke: InvokeConfig,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct RaftConfig {
    pub election_timeout_min: u64,
    pub election_timeout_max: u64,
    pub heartbeat_interval: u64,
    pub snapshot_chunk_size_kb: u64,
    pub snapshot_chunk_timeout_ms: u64,
}

impl Default for RaftConfig {
    fn default() -> Self {
        RaftConfig {
            election_timeout_max: 1000,
            election_timeout_min: 500,
            heartbeat_interval: 100,
            snapshot_chunk_size_kb: 1024 * 3,
            snapshot_chunk_timeout_ms: 1000,
        }
    }
}

impl EventConfig {
    pub const CLUSTER_K8S: &str = "k8s";
    pub const NO_CLUSTER: &str = "singleton";
}

impl Default for EventConfig {
    fn default() -> Self {
        EventConfig {
            rbum: Default::default(),
            enable: false,
            svc: "bios".to_string(),
            avatars: Vec::new(),
            startup_timeout: 5000,
            durable: true,
            cluster: Some(Self::CLUSTER_K8S.to_string()),
            raft: None,
            invoke: Default::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EventInfo {
    pub kind_id: String,
    pub domain_id: String,
}

tardis_static! {
    pub event_info: Mutex<Option<EventInfo>>;
}

pub struct EventInfoManager;

impl EventInfoManager {
    pub fn set(new_event_info: EventInfo) -> TardisResult<()> {
        let mut conf = event_info().lock().map_err(|e| TardisError::internal_error(&format!("{e:?}"), ""))?;
        *conf = Some(new_event_info);
        Ok(())
    }

    pub fn get_config<F, T>(fun: F) -> T
    where
        F: Fn(&EventInfo) -> T,
    {
        let conf = event_info().lock().unwrap_or_else(|e| panic!("event info lock error: {e:?}"));
        let conf = conf.as_ref().unwrap_or_else(|| panic!("rbum config not set"));
        fun(conf)
    }
}
