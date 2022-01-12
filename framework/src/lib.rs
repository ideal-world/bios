/*
 * Copyright 2021. gudaoxuri
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
use crate::db::reldb_client_mysql::BIOSRelDBMysqlClient;
#[cfg(feature = "reldb")]
use crate::db::reldb_client_postgres::BIOSRelDBPostgresClient;
#[cfg(feature = "mq")]
use crate::mq::mq_client::BIOSMQClient;
#[cfg(feature = "web-client")]
use crate::web::web_client::BIOSWebClient;

pub struct BIOSFuns {
    workspace_config: Option<Box<dyn Any>>,
    framework_config: Option<FrameworkConfig>,
    #[cfg(feature = "reldb")]
    reldb_mysql: Option<BIOSRelDBMysqlClient>,
    #[cfg(feature = "reldb")]
    reldb_postgres: Option<BIOSRelDBPostgresClient>,
    #[cfg(feature = "cache")]
    cache: Option<BIOSCacheClient>,
    #[cfg(feature = "mq")]
    mq: Option<BIOSMQClient>,
    #[cfg(feature = "web-client")]
    web_client: Option<BIOSWebClient>,
}

static mut BIOS_INST: BIOSFuns = BIOSFuns {
    workspace_config: None,
    framework_config: None,
    #[cfg(feature = "reldb")]
    reldb_mysql: None,
    #[cfg(feature = "reldb")]
    reldb_postgres: None,
    #[cfg(feature = "cache")]
    cache: None,
    #[cfg(feature = "mq")]
    mq: None,
    #[cfg(feature = "web-client")]
    web_client: None,
};

#[allow(unsafe_code)]
impl BIOSFuns {
    pub async fn init<T: 'static + Deserialize<'static>>(root_path: &str) -> BIOSResult<()> {
        BIOSLogger::init(root_path)?;
        let config = BIOSConfig::<T>::init(root_path)?;
        BIOSFuns::init_conf::<T>(config).await
    }

    pub async fn init_conf_from_path<T: 'static + Deserialize<'static>>(root_path: &str) -> BIOSResult<()> {
        let config = BIOSConfig::<T>::init(root_path)?;
        BIOSFuns::init_conf::<T>(config).await
    }

    pub fn init_log_from_path(root_path: &str) -> BIOSResult<()> {
        BIOSLogger::init(root_path)
    }

    pub async fn init_conf<T: 'static>(conf: BIOSConfig<T>) -> BIOSResult<()> {
        unsafe {
            replace(&mut BIOS_INST.workspace_config, Some(Box::new(conf.ws)));
            replace(&mut BIOS_INST.framework_config, Some(conf.fw));
        };
        #[cfg(feature = "reldb")]
        {
            if BIOSFuns::fw_config().db.enabled {
                match &BIOSFuns::fw_config().db.url.to_lowercase() {
                    url if url.starts_with("mysql") => {
                        let reldb_client = BIOSRelDBMysqlClient::init_by_conf(&BIOSFuns::fw_config()).await?;
                        unsafe {
                            replace(&mut BIOS_INST.reldb_mysql, Some(reldb_client));
                        };
                    }
                    url if url.starts_with("postgres") => {
                        let reldb_client = BIOSRelDBPostgresClient::init_by_conf(&BIOSFuns::fw_config()).await?;
                        unsafe {
                            replace(&mut BIOS_INST.reldb_postgres, Some(reldb_client));
                        };
                    }
                    _ => panic!("Doesn't support [{}] driver", Url::parse(&BIOSFuns::fw_config().db.url.as_str()).unwrap().scheme()),
                }
            }
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
        #[cfg(feature = "web-client")]
        {
            let web_client = BIOSWebClient::init_by_conf(&BIOSFuns::fw_config())?;
            unsafe {
                replace(&mut BIOS_INST.web_client, Some(web_client));
            };
        }
        BIOSResult::Ok(())
    }

    pub fn ws_config<T>() -> &'static T {
        unsafe {
            match &BIOS_INST.workspace_config {
                None => panic!("Raw Workspace Config doesn't exist"),
                Some(conf) => match conf.downcast_ref::<T>() {
                    None => panic!("Workspace Config doesn't exist"),
                    Some(t) => t,
                },
            }
        }
    }

    pub fn fw_config() -> &'static FrameworkConfig {
        unsafe {
            match &BIOS_INST.framework_config {
                None => panic!("Framework Config doesn't exist"),
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
    pub fn mysql() -> &'static BIOSRelDBMysqlClient {
        unsafe {
            match &BIOS_INST.reldb_mysql {
                None => panic!("RelDB default instance doesn't exist"),
                Some(t) => t,
            }
        }
    }

    #[cfg(feature = "reldb")]
    pub fn postgres() -> &'static BIOSRelDBPostgresClient {
        unsafe {
            match &BIOS_INST.reldb_postgres {
                None => panic!("RelDB default instance doesn't exist"),
                Some(t) => t,
            }
        }
    }

    #[cfg(feature = "cache")]
    pub fn cache() -> &'static mut BIOSCacheClient {
        unsafe {
            match &mut BIOS_INST.cache {
                None => panic!("Cache default instance doesn't exist"),
                Some(t) => t,
            }
        }
    }

    #[cfg(feature = "mq")]
    pub fn mq() -> &'static mut BIOSMQClient {
        unsafe {
            match &mut BIOS_INST.mq {
                None => panic!("MQ default instance doesn't exist"),
                Some(t) => t,
            }
        }
    }

    #[cfg(feature = "web-client")]
    pub fn web_client() -> &'static BIOSWebClient {
        unsafe {
            match &BIOS_INST.web_client {
                None => panic!("Web Client default instance doesn't exist"),
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
