/*
 * Copyright 2022. the original author or authors.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::env;
use std::fmt::Debug;
use std::path::Path;

use config::{Config, ConfigError, Environment, File};
use log::{debug, info};
use serde::{Deserialize, Serialize};

use crate::basic::error::{BIOSError, ERROR_DEFAULT_CODE};
use crate::basic::fetch_profile;
use crate::basic::result::BIOSResult;
use crate::BIOSFuns;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BIOSConfig<T> {
    pub ws: T,
    pub fw: FrameworkConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct FrameworkConfig {
    pub app: AppConfig,
    pub web_server: WebServerConfig,
    pub web_client: WebClientConfig,
    pub cache: CacheConfig,
    pub db: DBConfig,
    pub mq: MQConfig,
    pub adv: AdvConfig,
}

impl Default for FrameworkConfig {
    fn default() -> Self {
        FrameworkConfig {
            app: AppConfig::default(),
            web_server: WebServerConfig::default(),
            web_client: WebClientConfig::default(),
            cache: CacheConfig::default(),
            db: DBConfig::default(),
            mq: MQConfig::default(),
            adv: AdvConfig::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct AppConfig {
    pub id: String,
    pub name: String,
    pub desc: String,
    pub version: String,
    pub url: String,
    pub email: String,
    pub inst: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            id: "".to_string(),
            name: "BIOS Application".to_string(),
            desc: "This is a BIOS Application".to_string(),
            version: "0.0.1".to_string(),
            url: "".to_string(),
            email: "".to_string(),
            inst: format!("inst_{}", BIOSFuns::field.uuid()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct WebServerConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub allowed_origin: String,
    pub context_flag: String,
    pub lang_flag: String,
    pub modules: Vec<WebServerModuleConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct WebServerModuleConfig {
    pub code: String,
    pub title: String,
    pub version: String,
    pub doc_urls: Vec<(String, String)>,
    // TODO
    pub authors: Vec<(String, String)>,
    pub ui_path: Option<String>,
    pub spec_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct WebClientConfig {
    pub connect_timeout_sec: u64,
    pub request_timeout_sec: u64,
}

impl Default for WebServerConfig {
    fn default() -> Self {
        WebServerConfig {
            enabled: true,
            host: "0.0.0.0".to_string(),
            port: 8080,
            allowed_origin: "*".to_string(),
            context_flag: "BIOS-Context".to_string(),
            lang_flag: "Accept-Language".to_string(),
            modules: [WebServerModuleConfig::default()].iter().cloned().collect(),
        }
    }
}

impl Default for WebServerModuleConfig {
    fn default() -> Self {
        WebServerModuleConfig {
            code: "".to_string(),
            title: "BIOS-based application".to_string(),
            version: "1.0.0".to_string(),
            doc_urls: [("test env".to_string(), "http://localhost:8080/".to_string())].iter().cloned().collect(),
            authors: [("gudaoxuri".to_string(), "i@sunisle.org".to_string())].iter().cloned().collect(),
            ui_path: Some("ui".to_string()),
            spec_path: Some("spec".to_string()),
        }
    }
}

impl Default for WebClientConfig {
    fn default() -> Self {
        WebClientConfig {
            connect_timeout_sec: 60,
            request_timeout_sec: 60,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct CacheConfig {
    pub enabled: bool,
    pub url: String,
}

impl Default for CacheConfig {
    fn default() -> Self {
        CacheConfig {
            enabled: true,
            url: "".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct DBConfig {
    pub enabled: bool,
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout_sec: Option<u64>,
    pub idle_timeout_sec: Option<u64>,
}

impl Default for DBConfig {
    fn default() -> Self {
        DBConfig {
            enabled: true,
            url: "".to_string(),
            max_connections: 20,
            min_connections: 5,
            connect_timeout_sec: None,
            idle_timeout_sec: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct MQConfig {
    pub enabled: bool,
    pub url: String,
}

impl Default for MQConfig {
    fn default() -> Self {
        MQConfig {
            enabled: true,
            url: "".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct AdvConfig {
    pub backtrace: bool,
}

impl Default for AdvConfig {
    fn default() -> Self {
        AdvConfig { backtrace: false }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NoneConfig {}

impl<'a, T> BIOSConfig<T>
where
    T: Deserialize<'a>,
{
    pub(crate) fn init(root_path: &str) -> BIOSResult<BIOSConfig<T>> {
        let profile = fetch_profile();
        let path = Path::new(root_path);

        info!("[BIOS.Framework.Config] Initializing, root path:{:?}, profile:{}", root_path, profile);
        let mut conf = Config::default();
        conf.merge(File::from(path.join("conf-default")).required(false))?;

        conf.merge(File::from(Path::new(root_path).join(&format!("conf-{}", profile))).required(false))?;
        conf.merge(Environment::with_prefix("BIOS"))?;
        let workspace_config = conf.clone().try_into::<T>()?;
        let framework_config = conf.try_into::<FrameworkConfig>()?;

        env::set_var("RUST_BACKTRACE", if framework_config.adv.backtrace { "1" } else { "0" });

        info!("[BIOS.Framework.Config] Initialized, root path:{}, profile:{}", root_path, profile);
        debug!("=====[BIOS.Framework.Config] Content=====\n{:#?}\n=====", framework_config);

        Ok(BIOSConfig {
            ws: workspace_config,
            fw: framework_config,
        })
    }
}

impl From<ConfigError> for BIOSError {
    fn from(error: ConfigError) -> Self {
        match error {
            ConfigError::Frozen => BIOSError::IOError(error.to_string()),
            ConfigError::NotFound(_) => BIOSError::NotFound(error.to_string()),
            ConfigError::PathParse(_) => BIOSError::IOError(error.to_string()),
            ConfigError::FileParse { .. } => BIOSError::IOError(error.to_string()),
            ConfigError::Type { .. } => BIOSError::FormatError(error.to_string()),
            ConfigError::Message(s) => BIOSError::Custom(ERROR_DEFAULT_CODE.to_string(), s),
            ConfigError::Foreign(err) => BIOSError::Box(err),
        }
    }
}
