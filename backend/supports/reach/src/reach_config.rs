use bios_basic::rbum::rbum_config::RbumConfig;
use bios_sdk_invoke::invoke_config::InvokeConfig;
use serde::{Deserialize, Serialize};
mod reach_config_sms;
mod reach_config_opapi;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct ReachConfig {
    pub sms: reach_config_sms::SmsConfig,
    pub opapi: reach_config_opapi::OpApiConfig,
    pub rbum: RbumConfig,
    pub invoke: InvokeConfig,
    pub iam_get_account: String,
}
