use bios_basic::rbum::rbum_config::RbumConfig;
use bios_sdk_invoke::invoke_config::InvokeConfig;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct StatsConfig {
    pub rbum: RbumConfig,
    pub base_url: String,
    pub invoke: InvokeConfig,
}
