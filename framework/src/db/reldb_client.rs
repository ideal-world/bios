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

use std::time::{SystemTime, UNIX_EPOCH};

use log::info;
use sea_query::{
    DeleteStatement, InsertStatement, MysqlQueryBuilder, SelectStatement, TableCreateStatement,
    UpdateStatement, Values,
};
use serde::{Deserialize, Serialize};
use sqlx::mysql::{MySqlPoolOptions, MySqlQueryResult, MySqlRow};
use sqlx::pool::PoolConnection;
use sqlx::{FromRow, MySql, Pool, Row, Transaction};
use url::Url;

use crate::basic::config::FrameworkConfig;
use crate::basic::error::{BIOSError, BIOSResult};
use crate::db::reldb_client::sea_query_driver_mysql::{bind_query, bind_query_as};

sea_query::sea_query_driver_mysql!();

pub struct BIOSRelDBClient {
    pool: Pool<MySql>,
}

impl BIOSRelDBClient {
    pub async fn init_by_conf(conf: &FrameworkConfig) -> BIOSResult<BIOSRelDBClient> {
        BIOSRelDBClient::init(&conf.db.url, conf.db.max_connections).await
    }

    pub async fn init(str_url: &str, conn_max: u32) -> BIOSResult<BIOSRelDBClient> {
        let url = Url::parse(str_url)?;
        info!(
            "[BIOS.Framework.RelDBClient] Initializing, host:{}, port:{}, max_connections:{}",
            url.host_str().unwrap_or(""),
            url.port().unwrap_or(0),
            conn_max
        );
        let pool = MySqlPoolOptions::new()
            .max_connections(conn_max)
            .connect(str_url)
            .await
            .unwrap();
        info!(
            "[BIOS.Framework.RelDBClient] Initialized, host:{}, port:{}, max_connections:{}",
            url.host_str().unwrap_or(""),
            url.port().unwrap_or(0),
            conn_max
        );
        Ok(BIOSRelDBClient { pool })
    }

    pub async fn conn(&self) -> PoolConnection<MySql> {
        self.pool.acquire().await.unwrap()
    }

    pub async fn exec<'c>(
        &self,
        sql_builder: &BIOSSqlBuilder,
        tx: Option<&mut Transaction<'c, MySql>>,
    ) -> BIOSResult<MySqlQueryResult> {
        let result = bind_query(sqlx::query(&sql_builder.sql), &sql_builder.values);
        let result = match tx {
            Some(t) => result.execute(t).await,
            None => result.execute(&self.pool).await,
        };
        match result {
            Ok(ok) => BIOSResult::Ok(ok),
            Err(err) => BIOSResult::Err(BIOSError::Box(Box::new(err))),
        }
    }

    pub async fn fetch_all<'c, E>(
        &self,
        sql_builder: &BIOSSqlBuilder,
        tx: Option<&mut Transaction<'c, MySql>>,
    ) -> BIOSResult<Vec<E>>
    where
        E: for<'r> FromRow<'r, MySqlRow>,
        E: std::marker::Send,
        E: Unpin,
    {
        let result = bind_query_as(
            sqlx::query_as::<_, E>(&sql_builder.sql),
            &sql_builder.values,
        );
        let result = match tx {
            Some(t) => result.fetch_all(t).await,
            None => result.fetch_all(&self.pool).await,
        };
        match result {
            Ok(rows) => BIOSResult::Ok(rows),
            Err(err) => BIOSResult::Err(BIOSError::Box(Box::new(err))),
        }
    }

    pub async fn fetch_one<'c, E>(
        &self,
        sql_builder: &BIOSSqlBuilder,
        tx: Option<&mut Transaction<'c, MySql>>,
    ) -> BIOSResult<E>
    where
        E: for<'r> FromRow<'r, MySqlRow>,
        E: std::marker::Send,
        E: Unpin,
    {
        let fetch_one_sql = format!("{} LIMIT 1", sql_builder.sql);
        let result = bind_query_as(sqlx::query_as::<_, E>(&fetch_one_sql), &sql_builder.values);
        let result = match tx {
            Some(t) => result.fetch_one(t).await,
            None => result.fetch_one(&self.pool).await,
        };
        match result {
            Ok(row) => BIOSResult::Ok(row),
            Err(err) => BIOSResult::Err(BIOSError::Box(Box::new(err))),
        }
    }

    pub async fn pagination<'c, E>(
        &self,
        sql_builder: &BIOSSqlBuilder,
        page_number: u64,
        page_size: u64,
        tx: Option<&mut Transaction<'c, MySql>>,
    ) -> BIOSResult<BIOSPage<E>>
    where
        E: for<'r> FromRow<'r, MySqlRow>,
        E: std::marker::Send,
        E: Unpin,
    {
        let page_sql = format!(
            "{} LIMIT {} , {}",
            sql_builder.sql,
            (page_number - 1) * page_size,
            page_size
        );
        let result = bind_query_as(sqlx::query_as::<_, E>(&page_sql), &sql_builder.values);
        let (total_size, result) = match tx {
            Some(t) => (
                self.count(sql_builder, Some(t)).await?,
                result.fetch_all(t).await,
            ),
            None => (
                self.count(sql_builder, None).await?,
                result.fetch_all(&self.pool).await,
            ),
        };
        match result {
            Ok(rows) => BIOSResult::Ok(BIOSPage {
                page_size,
                page_number,
                total_size,
                records: rows,
            }),
            Err(err) => BIOSResult::Err(BIOSError::Box(Box::new(err))),
        }
    }

    pub async fn exists<'c>(
        &self,
        sql_builder: &BIOSSqlBuilder,
        tx: Option<&mut Transaction<'c, MySql>>,
    ) -> BIOSResult<bool> {
        match self.count(sql_builder, tx).await {
            Ok(count) => Ok(count != 0),
            Err(e) => Err(e),
        }
    }

    pub async fn count<'c>(
        &self,
        sql_builder: &BIOSSqlBuilder,
        tx: Option<&mut Transaction<'c, MySql>>,
    ) -> BIOSResult<u64> {
        let count_sql = format!(
            "SELECT COUNT(1) AS _COUNT FROM ( {} ) _{}",
            sql_builder.sql,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );
        let result = bind_query(sqlx::query(&count_sql), &sql_builder.values);
        let result = match tx {
            Some(t) => result.fetch_one(t).await,
            None => result.fetch_one(&self.pool).await,
        };
        match result {
            Ok(row) => {
                let size: i64 = row.get(0);
                BIOSResult::Ok(size as u64)
            }
            Err(err) => BIOSResult::Err(BIOSError::Box(Box::new(err))),
        }
    }
}

pub trait SqlBuilderProcess {
    fn done(&self) -> BIOSSqlBuilder;
}

impl SqlBuilderProcess for TableCreateStatement {
    fn done(&self) -> BIOSSqlBuilder {
        BIOSSqlBuilder {
            sql: (&self.build(MysqlQueryBuilder)).to_string(),
            values: Values(vec![]),
        }
    }
}

impl SqlBuilderProcess for InsertStatement {
    fn done(&self) -> BIOSSqlBuilder {
        let (sql, values) = &self.build(MysqlQueryBuilder);
        BIOSSqlBuilder {
            sql: sql.to_string(),
            values: values.clone(),
        }
    }
}

impl SqlBuilderProcess for SelectStatement {
    fn done(&self) -> BIOSSqlBuilder {
        let (sql, values) = &self.build(MysqlQueryBuilder);
        BIOSSqlBuilder {
            sql: sql.to_string(),
            values: values.clone(),
        }
    }
}

impl SqlBuilderProcess for UpdateStatement {
    fn done(&self) -> BIOSSqlBuilder {
        let (sql, values) = &self.build(MysqlQueryBuilder);
        BIOSSqlBuilder {
            sql: sql.to_string(),
            values: values.clone(),
        }
    }
}

impl SqlBuilderProcess for DeleteStatement {
    fn done(&self) -> BIOSSqlBuilder {
        let (sql, values) = &self.build(MysqlQueryBuilder);
        BIOSSqlBuilder {
            sql: sql.to_string(),
            values: values.clone(),
        }
    }
}

pub struct BIOSSqlBuilder {
    pub sql: String,
    pub values: Values,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BIOSPage<E>
where
    E: for<'r> FromRow<'r, MySqlRow>,
    E: std::marker::Send,
    E: Unpin,
{
    pub page_size: u64,
    pub page_number: u64,
    pub total_size: u64,
    pub records: Vec<E>,
}

impl From<sqlx::Error> for BIOSError {
    fn from(error: sqlx::Error) -> Self {
        BIOSError::Box(Box::new(error))
    }
}
