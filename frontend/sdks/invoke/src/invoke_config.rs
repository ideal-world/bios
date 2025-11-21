use crate::invoke_enumeration::InvokeModuleKind;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::{collections::HashMap, fmt::Debug};
use tardis::basic::dto::TardisContext;
use tardis::basic::{error::TardisError, result::TardisResult};
use tardis::TardisFunsInst;
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct InvokeConfig {
    pub spi_app_id: String,
    pub module_urls: HashMap<String, String>,
    pub module_configs: HashMap<String, InvokeModuleConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InvokeModuleConfig {
    pub in_event: bool,
}

impl Default for InvokeConfig {
    fn default() -> Self {
        InvokeConfig {
            spi_app_id: Default::default(),
            module_urls: HashMap::from([
                (InvokeModuleKind::Kv.to_string(), "http://127.0.0.1:8080/spi-kv".to_string()),
                (InvokeModuleKind::Object.to_string(), "http://127.0.0.1:8080/spi-object".to_string()),
                (InvokeModuleKind::Log.to_string(), "http://127.0.0.1:8080/spi-log".to_string()),
                (InvokeModuleKind::Search.to_string(), "http://127.0.0.1:8080/spi-search".to_string()),
                (InvokeModuleKind::Schedule.to_string(), "http://127.0.0.1:8080/schedule".to_string()),
                (InvokeModuleKind::Iam.to_string(), "http://127.0.0.1:8080/iam".to_string()),
                (InvokeModuleKind::Stats.to_string(), "http://127.0.0.1:8080/spi-stats".to_string()),
                (InvokeModuleKind::Event.to_string(), "http://bios-event:8080/event".to_string()),
                (InvokeModuleKind::Reach.to_string(), "http://127.0.0.1:8080/reach".to_string()),
            ]),
            module_configs: HashMap::from([
                (InvokeModuleKind::Kv.to_string(), InvokeModuleConfig { in_event: false }),
                (InvokeModuleKind::Object.to_string(), InvokeModuleConfig { in_event: false }),
                (InvokeModuleKind::Log.to_string(), InvokeModuleConfig { in_event: false }),
                (InvokeModuleKind::Search.to_string(), InvokeModuleConfig { in_event: false }),
                (InvokeModuleKind::Schedule.to_string(), InvokeModuleConfig { in_event: false }),
                (InvokeModuleKind::Stats.to_string(), InvokeModuleConfig { in_event: false }),
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
        let conf = conf.get(code).unwrap_or_else(|| panic!("not found invoke config code {code}"));
        fun(conf)
    }

    pub fn get_module_config(code: &str, module: InvokeModuleKind) -> Option<InvokeModuleConfig> {
        let conf = INVOKE_CONFIG.lock().unwrap_or_else(|e| panic!("invoke config lock error: {e:?}"));
        let conf = conf.get(code).unwrap_or_else(|| panic!("not found invoke config code {code}"));
        conf.module_configs.get(&module.to_string()).cloned()
    }
}

pub trait InvokeConfigApi {
    fn invoke_conf_spi_app_id(&self) -> String;
    fn invoke_conf_module_url(&self) -> HashMap<String, String>;
    fn invoke_conf_match_module_url(&self, module_url: &str) -> bool;
    fn invoke_conf_inject_context(&self, context: &TardisContext) -> TardisContext;
    fn invoke_conf_in_event(&self, module: InvokeModuleKind) -> bool;
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

    fn invoke_conf_inject_context(&self, context: &TardisContext) -> TardisContext {
        let mut ctx = context.clone();
        ctx.ak = self.invoke_conf_spi_app_id();
        ctx
    }

    fn invoke_conf_in_event(&self, module: InvokeModuleKind) -> bool {
        InvokeConfigManager::get_module_config(self.module_code(), module).map_or(false, |conf| conf.in_event)
    }
}
