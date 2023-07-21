use crate::invoke_enumeration::InvokeModuleKind;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::{collections::HashMap, fmt::Debug};
use tardis::basic::{error::TardisError, result::TardisResult};
use tardis::TardisFunsInst;
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

lazy_static! {
    static ref INVOKE_CONFIG: Mutex<HashMap<String, InvokeConfig>> = Mutex::new(HashMap::new());
}

pub struct InvokeConfigManager;

impl InvokeConfigManager {
    pub fn add(code: &str, config: InvokeConfig) -> TardisResult<()> {
        let mut conf = INVOKE_CONFIG.lock().map_err(|e| TardisError::internal_error(&format!("{e:?}"), ""))?;
        conf.insert(code.to_string(), config);
        Ok(())
    }

    pub fn match_module_url(code: &str, module_url: &str) -> bool {
        Self::get_config(code, |conf| conf.module_urls.iter().any(|(k, _v)| module_url.eq(k)))
    }

    pub fn get_config<F, T>(code: &str, fun: F) -> T
    where
        F: Fn(&InvokeConfig) -> T,
    {
        let conf = INVOKE_CONFIG.lock().unwrap_or_else(|e| panic!("invoke config lock error: {e:?}"));
        let conf = conf.get(code).unwrap_or_else(|| panic!("not found rbum config code {code}"));
        fun(conf)
    }
}

pub trait InvokeConfigApi {
    fn invoke_conf_spi_app_id(&self) -> String;
    fn invoke_conf_module_url(&self) -> HashMap<String, String>;
    fn invoke_conf_match_module_url(&self, module_url: &str) -> bool;
}

impl InvokeConfigApi for TardisFunsInst {
    fn invoke_conf_spi_app_id(&self) -> String {
        InvokeConfigManager::get_config(self.module_code(), |conf| conf.spi_app_id.clone())
    }

    fn invoke_conf_module_url(&self) -> HashMap<String, String> {
        InvokeConfigManager::get_config(self.module_code(), |conf| conf.module_urls.clone())
    }

    fn invoke_conf_match_module_url(&self, module_url: &str) -> bool {
        InvokeConfigManager::match_module_url(self.module_code(), module_url)
    }
}
