use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Debug};

use crate::invoke_enumeration::InvokeModuleKind;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct InvokeConfig {
    pub spi_app_id: String,
    pub module_urls: HashMap<InvokeModuleKind, String>,
}
pub trait InvokeConfigTrait: 
for <'a> Deserialize<'a> {
    fn get_spi_app_id(&self) -> &str;
    fn get_module_url(&self, module: InvokeModuleKind) -> &str {
        self.get_module_opt_url(module).unwrap_or_else(|| panic!("invoke config missing invoke url for module [{module}]"))
    }
    fn get_module_opt_url(&self, module: InvokeModuleKind) -> Option<&str>;
}

impl InvokeConfigTrait for InvokeConfig {
    fn get_spi_app_id(&self) -> &str {
        &self.spi_app_id
    }

    fn get_module_opt_url(&self, module: InvokeModuleKind) -> Option<&str> {
        self.module_urls.get(&module).map(|x|x.as_str())
    }
}
impl Default for InvokeConfig {
    fn default() -> Self {
        InvokeConfig {
            spi_app_id: Default::default(),
            module_urls: HashMap::from([
                (InvokeModuleKind::Kv, "http://127.0.0.1:8080/spi-kv".to_string()),
                (InvokeModuleKind::Log, "http://127.0.0.1:8080/spi-log".to_string()),
                (InvokeModuleKind::Search, "http://127.0.0.1:8080/spi-search".to_string()),
                (InvokeModuleKind::Schedule, "http://127.0.0.1:8080/schedule".to_string()),
                (InvokeModuleKind::Iam, "http://127.0.0.1:8080/iam".to_string()),
            ]),
        }
    }
}
