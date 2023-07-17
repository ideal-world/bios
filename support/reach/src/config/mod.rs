use bios_basic::rbum::rbum_config::RbumConfig;
use serde::{Deserialize, Serialize};

mod mail;
mod sms;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReachConfig {
    pub sms: sms::HwSmsConfig,
    pub rbum: RbumConfig,
}
