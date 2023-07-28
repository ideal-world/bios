use bios_basic::rbum::rbum_config::RbumConfig;
use bios_sdk_invoke::{
    invoke_config::{InvokeConfig, InvokeConfigTrait},
    invoke_enumeration::InvokeModuleKind,
};
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
    pub iam_get_account: String
}

impl InvokeConfigTrait for ReachConfig {
    fn get_spi_app_id(&self) -> &str {
        self.invoke.get_spi_app_id()
    }

    fn get_module_opt_url(&self, module: InvokeModuleKind) -> Option<&str> {
        self.invoke.get_module_opt_url(module)
    }
}
