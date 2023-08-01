use bios_basic::rbum::rbum_config::RbumConfig;
use bios_sdk_invoke::invoke_config::InvokeConfig;
use serde::{Deserialize, Serialize};

mod mail;
mod sms;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReachConfig {
    pub sms: sms::HwSmsConfig,
    #[serde(default)]
    pub rbum: RbumConfig,
    #[serde(default)]
    pub invoke: InvokeConfig,
    pub iam_get_account: String,
}
