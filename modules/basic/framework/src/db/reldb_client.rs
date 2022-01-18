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

use std::time::Duration;

use log::info;
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbErr, EntityTrait, ExecResult, Schema};
use url::Url;

use crate::basic::error::BIOSError;
use crate::basic::result::BIOSResult;
use crate::db::domain::bios_db_config;
use crate::FrameworkConfig;

pub struct BIOSRelDBClient {
    con: DatabaseConnection,
}

impl BIOSRelDBClient {
    pub async fn init_by_conf(conf: &FrameworkConfig) -> BIOSResult<BIOSRelDBClient> {
        BIOSRelDBClient::init(
            &conf.db.url,
            conf.db.max_connections,
            conf.db.min_connections,
            conf.db.connect_timeout_sec,
            conf.db.idle_timeout_sec,
        )
        .await
    }

    pub async fn init(str_url: &str, max_connections: u32, min_connections: u32, connect_timeout_sec: Option<u64>, idle_timeout_sec: Option<u64>) -> BIOSResult<BIOSRelDBClient> {
        // TODO
        // tracing_subscriber::fmt()
        //     .with_max_level(tracing::Level::DEBUG)
        //     .with_test_writer()
        //     .init();
        let url = Url::parse(str_url)?;
        info!(
            "[BIOS.Framework.RelDBClient] Initializing, host:{}, port:{}, max_connections:{}",
            url.host_str().unwrap_or(""),
            url.port().unwrap_or(0),
            max_connections
        );
        let mut opt = ConnectOptions::new(str_url.to_owned());
        opt.max_connections(max_connections).min_connections(min_connections).sqlx_logging(true);
        if let Some(connect_timeout_sec) = connect_timeout_sec {
            opt.connect_timeout(Duration::from_secs(connect_timeout_sec));
        }
        if let Some(idle_timeout_sec) = idle_timeout_sec {
            opt.idle_timeout(Duration::from_secs(idle_timeout_sec));
        }
        let con = Database::connect(opt).await?;
        info!(
            "[BIOS.Framework.RelDBClient] Initialized, host:{}, port:{}, max_connections:{}",
            url.host_str().unwrap_or(""),
            url.port().unwrap_or(0),
            min_connections
        );
        let client = BIOSRelDBClient { con };
        client.create_table(bios_db_config::Entity);
        Ok(client)
    }

    pub async fn create_table<E>(&self, entity: E) -> BIOSResult<ExecResult>
    where
        E: EntityTrait,
    {
        let builder = self.con.get_database_backend();
        let schema = Schema::new(builder);
        let table_create_statement = &schema.create_table_from_entity(entity);
        let result = self.con.execute(builder.build(table_create_statement)).await;
        match result {
            Ok(ok) => BIOSResult::Ok(ok),
            Err(err) => BIOSResult::Err(BIOSError::Box(Box::new(err))),
        }
    }
}

impl From<DbErr> for BIOSError {
    fn from(error: DbErr) -> Self {
        BIOSError::Box(Box::new(error))
    }
}
