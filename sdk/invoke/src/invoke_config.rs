use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Debug};

use crate::invoke_enumeration::InvokeModuleKind;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct InvokeConfig {
    pub spi_app_id: String,
    pub module_urls: HashMap<String, String>,
}

impl Default for InvokeConfig {
    fn default() -> Self {
        InvokeConfig {
            spi_app_id: Default::default(),
            module_urls: HashMap::from([
                (InvokeModuleKind::Kv.to_string(), "http://127.0.0.1:8080/spi-kv".to_string()),
                (InvokeModuleKind::Log.to_string(), "http://127.0.0.1:8080/spi-log".to_string()),
                (InvokeModuleKind::Search.to_string(), "http://127.0.0.1:8080/spi-search".to_string()),
                (InvokeModuleKind::Schedule.to_string(), "http://127.0.0.1:8080/schedule".to_string()),
            ]),
        }
    }
}
