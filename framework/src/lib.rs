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

#[macro_use]
extern crate lazy_static;

use std::any::Any;
use std::ptr::replace;

use serde::Deserialize;

use basic::result::BIOSResult;

use crate::basic::config::{BIOSConfig, FrameworkConfig};
use crate::basic::field::BIOSField;
use crate::basic::json::BIOSJson;
use crate::basic::logger::BIOSLogger;
use crate::basic::security::BIOSSecurity;
use crate::basic::security::BIOSSecurityBase64;
use crate::basic::security::BIOSSecurityKey;
use crate::basic::uri::BIOSUri;
#[cfg(feature = "cache")]
use crate::cache::cache_client::BIOSCacheClient;
#[cfg(feature = "reldb")]
use crate::db::reldb_client::BIOSRelDBClient;
#[cfg(feature = "mq")]
use crate::mq::mq_client::BIOSMQClient;
#[cfg(feature = "web-client")]
use crate::web::web_client::BIOSWebClient;
#[cfg(feature = "web-server")]
use crate::web::web_server::BIOSWebServer;

pub struct BIOSFuns {
    workspace_config: Option<Box<dyn Any>>,
    framework_config: Option<FrameworkConfig>,
    #[cfg(feature = "reldb")]
    reldb: Option<BIOSRelDBClient>,
    #[cfg(feature = "web-server")]
    web_server: Option<BIOSWebServer>,
    #[cfg(feature = "web-client")]
    web_client: Option<BIOSWebClient>,
    #[cfg(feature = "cache")]
    cache: Option<BIOSCacheClient>,
    #[cfg(feature = "mq")]
    mq: Option<BIOSMQClient>,
}

static mut BIOS_INST: BIOSFuns = BIOSFuns {
    workspace_config: None,
    framework_config: None,
    #[cfg(feature = "reldb")]
    reldb: None,
    #[cfg(feature = "web-server")]
    web_server: None,
    #[cfg(feature = "web-client")]
    web_client: None,
    #[cfg(feature = "cache")]
    cache: None,
    #[cfg(feature = "mq")]
    mq: None,
};

#[allow(unsafe_code)]
impl BIOSFuns {
    pub async fn init<T: 'static + Deserialize<'static>>(relative_path: &str) -> BIOSResult<()> {
        BIOSLogger::init()?;
        let config = BIOSConfig::<T>::init(relative_path)?;
        BIOSFuns::init_conf::<T>(config).await
    }

    pub fn init_log() -> BIOSResult<()> {
        BIOSLogger::init()
    }

    pub async fn init_conf<T: 'static>(conf: BIOSConfig<T>) -> BIOSResult<()> {
        BIOSLogger::init()?;
        unsafe {
            replace(&mut BIOS_INST.workspace_config, Some(Box::new(conf.ws)));
            replace(&mut BIOS_INST.framework_config, Some(conf.fw));
        };
        #[cfg(feature = "reldb")]
        {
            if BIOSFuns::fw_config().db.enabled {
                let reldb_client = BIOSRelDBClient::init_by_conf(&BIOSFuns::fw_config()).await?;
                unsafe {
                    replace(&mut BIOS_INST.reldb, Some(reldb_client));
                };
            }
        }
        #[cfg(feature = "web-server")]
        {
            if BIOSFuns::fw_config().web_server.enabled {
                let web_server = BIOSWebServer::init_by_conf(&BIOSFuns::fw_config()).await?;
                unsafe {
                    replace(&mut BIOS_INST.web_server, Some(web_server));
                };
            }
        }
        #[cfg(feature = "web-client")]
        {
            let web_client = BIOSWebClient::init_by_conf(&BIOSFuns::fw_config())?;
            unsafe {
                replace(&mut BIOS_INST.web_client, Some(web_client));
            };
        }
        #[cfg(feature = "cache")]
        {
            if BIOSFuns::fw_config().cache.enabled {
                let cache_client = BIOSCacheClient::init_by_conf(&BIOSFuns::fw_config()).await?;
                unsafe {
                    replace(&mut BIOS_INST.cache, Some(cache_client));
                };
            }
        }
        #[cfg(feature = "mq")]
        {
            if BIOSFuns::fw_config().mq.enabled {
                let mq_client = BIOSMQClient::init_by_conf(&BIOSFuns::fw_config()).await?;
                unsafe {
                    replace(&mut BIOS_INST.mq, Some(mq_client));
                };
            }
        }
        BIOSResult::Ok(())
    }

    pub fn ws_config<T>() -> &'static T {
        unsafe {
            match &BIOS_INST.workspace_config {
                None => panic!("[BIOS.Framework.Config] Raw Workspace Config doesn't exist"),
                Some(conf) => match conf.downcast_ref::<T>() {
                    None => panic!("[BIOS.Framework.Config] Workspace Config doesn't exist"),
                    Some(t) => t,
                },
            }
        }
    }

    pub fn fw_config() -> &'static FrameworkConfig {
        unsafe {
            match &BIOS_INST.framework_config {
                None => panic!("[BIOS.Framework.Config] Framework Config doesn't exist"),
                Some(t) => t,
            }
        }
    }

    #[allow(non_upper_case_globals)]
    pub const field: BIOSField = BIOSField {};

    #[allow(non_upper_case_globals)]
    pub const json: BIOSJson = BIOSJson {};

    #[allow(non_upper_case_globals)]
    pub const uri: BIOSUri = BIOSUri {};

    #[allow(non_upper_case_globals)]
    pub const security: BIOSSecurity = BIOSSecurity {
        base64: BIOSSecurityBase64 {},
        key: BIOSSecurityKey {},
    };

    #[cfg(feature = "reldb")]
    pub fn reldb() -> &'static BIOSRelDBClient {
        unsafe {
            match &BIOS_INST.reldb {
                None => panic!("[BIOS.Framework.Config] RelDB default instance doesn't exist"),
                Some(t) => t,
            }
        }
    }

    #[cfg(feature = "web-server")]
    pub fn web_server() -> &'static mut BIOSWebServer {
        unsafe {
            match &mut BIOS_INST.web_server {
                None => panic!("[BIOS.Framework.Config] Web Server default instance doesn't exist"),
                Some(t) => t,
            }
        }
    }

    #[cfg(feature = "web-client")]
    pub fn web_client() -> &'static BIOSWebClient {
        unsafe {
            match &BIOS_INST.web_client {
                None => panic!("[BIOS.Framework.Config] Web Client default instance doesn't exist"),
                Some(t) => t,
            }
        }
    }

    #[cfg(feature = "cache")]
    pub fn cache() -> &'static mut BIOSCacheClient {
        unsafe {
            match &mut BIOS_INST.cache {
                None => panic!("[BIOS.Framework.Config] Cache default instance doesn't exist"),
                Some(t) => t,
            }
        }
    }

    #[cfg(feature = "mq")]
    pub fn mq() -> &'static mut BIOSMQClient {
        unsafe {
            match &mut BIOS_INST.mq {
                None => panic!("[BIOS.Framework.Config] MQ default instance doesn't exist"),
                Some(t) => t,
            }
        }
    }
}

pub mod basic;
#[cfg(feature = "cache")]
pub mod cache;
#[cfg(feature = "reldb")]
pub mod db;
#[cfg(feature = "mq")]
pub mod mq;
#[cfg(feature = "test")]
pub mod test;
pub mod web;
