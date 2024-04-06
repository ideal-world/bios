use bios_basic::rbum::rbum_config::RbumConfig;
use bios_sdk_invoke::invoke_config::InvokeConfig;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct PluginConfig {
    pub rbum: RbumConfig,
    pub invoke: InvokeConfig,
}
