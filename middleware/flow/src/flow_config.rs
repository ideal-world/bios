use bios_basic::{process::ci_processor::AppKeyConfig, rbum::rbum_config::RbumConfig};
use bios_sdk_invoke::invoke_config::InvokeConfig;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, sync::Mutex};
use tardis::{
    basic::{error::TardisError, result::TardisResult},
    TardisFunsInst,
};
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct FlowConfig {
    pub rbum: RbumConfig,
    pub invoke: InvokeConfig,
    pub app_key: AppKeyConfig,
    pub search_url: String,
    pub log_url: String,
}

impl Default for FlowConfig {
    fn default() -> Self {
        FlowConfig {
            rbum: Default::default(),
            invoke: Default::default(),
            app_key: Default::default(),
            search_url: "http://127.0.0.1:8080/spi-search".to_string(),
            log_url: "http://127.0.0.1:8080/spi-log".to_string(),
        }
    }
}

impl FlowConfig {
    pub fn search_url(&self) -> String {
        if self.search_url.ends_with('/') {
            self.search_url.clone()
        } else {
            format!("{}/", self.search_url)
        }
    }

    pub fn log_url(&self) -> String {
        if self.log_url.ends_with('/') {
            self.log_url.clone()
        } else {
            format!("{}/", self.log_url)
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BasicInfo {
    pub kind_state_id: String,
    pub kind_model_id: String,
    pub domain_flow_id: String,
}

lazy_static! {
    static ref BASIC_INFO: Mutex<Option<BasicInfo>> = Mutex::new(None);
}

pub struct FlowBasicInfoManager;

impl FlowBasicInfoManager {
    pub fn set(basic_info: BasicInfo) -> TardisResult<()> {
        let mut conf = BASIC_INFO.lock().map_err(|e| TardisError::internal_error(&format!("{e:?}"), ""))?;
        *conf = Some(basic_info);
        Ok(())
    }

    pub fn get_config<F, T>(fun: F) -> T
    where
        F: Fn(&BasicInfo) -> T,
    {
        let conf = BASIC_INFO.lock().unwrap_or_else(|e| panic!("flow basic info lock error: {e:?}"));
        let conf = conf.as_ref().unwrap_or_else(|| panic!("flow basic info not set"));
        fun(conf)
    }
}

pub trait FlowBasicConfigApi {
    fn flow_basic_kind_state_id(&self) -> String;
    fn flow_basic_kind_model_id(&self) -> String;
    fn flow_basic_domain_flow_id(&self) -> String;
}

impl FlowBasicConfigApi for TardisFunsInst {
    fn flow_basic_kind_state_id(&self) -> String {
        FlowBasicInfoManager::get_config(|conf| conf.kind_state_id.clone())
    }
    fn flow_basic_kind_model_id(&self) -> String {
        FlowBasicInfoManager::get_config(|conf| conf.kind_model_id.clone())
    }
    fn flow_basic_domain_flow_id(&self) -> String {
        FlowBasicInfoManager::get_config(|conf| conf.domain_flow_id.clone())
    }
}
