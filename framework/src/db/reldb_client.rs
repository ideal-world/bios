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

use std::time::Duration;

use log::info;
use rbatis::core::db::DBPoolOptions;
use rbatis::rbatis::Rbatis;
use url::Url;

use crate::basic::config::FrameworkConfig;
use crate::basic::error::{BIOSError, BIOSResult};

lazy_static! {
    pub static ref BIOSDB: Rbatis = Rbatis::new();
}

pub struct BIOSRelDBClient {}

impl BIOSRelDBClient {
    pub async fn init_by_conf(conf: &FrameworkConfig) -> BIOSResult<()> {
        BIOSRelDBClient::init(&conf.db.url, conf.db.max_connections).await
    }

    pub async fn init(str_url: &str, conn_max: u32) -> BIOSResult<()> {
        let url = Url::parse(str_url)?;
        info!(
            "[BIOS.Framework.RelDBClient] Initializing, host:{}, port:{}, max_connections:{}",
            url.host_str().unwrap_or(""),
            url.port().unwrap_or(0),
            conn_max
        );
        BIOSDB
            .link_opt(
                str_url,
                &DBPoolOptions {
                    // pool a maximum connections to the same database
                    max_connections: conn_max,
                    // don't open connections until necessary
                    min_connections: 0,
                    // try to connect for 10 seconds before erroring
                    connect_timeout: Duration::from_secs(60),
                    // reap connections that have been alive > 30 minutes
                    // prevents unbounded live-leaking of memory due to naive prepared statement caching
                    // see src/cache.rs for context
                    max_lifetime: Some(Duration::from_secs(1800)),
                    // don't reap connections based on idle time
                    idle_timeout: None,
                    // If true, test the health of a connection on acquire
                    test_before_acquire: true,
                },
            )
            .await?;
        info!(
            "[BIOS.Framework.RelDBClient] Initialized, host:{}, port:{}, max_connections:{}",
            url.host_str().unwrap_or(""),
            url.port().unwrap_or(0),
            conn_max
        );
        Ok(())
    }
}

impl From<rbatis::Error> for BIOSError {
    fn from(error: rbatis::Error) -> Self {
        BIOSError::Box(Box::new(error))
    }
}
