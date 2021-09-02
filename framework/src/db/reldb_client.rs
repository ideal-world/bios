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
    ColumnDef, DeleteStatement, Expr, InsertStatement, IntoColumnRef, IntoTableRef, MysqlQueryBuilder, Query, SelectStatement, Table, TableCreateStatement, UpdateStatement, Values,
};
use serde::{Deserialize, Serialize};
use sqlx::mysql::{MySqlPoolOptions, MySqlQueryResult, MySqlRow};
use sqlx::pool::PoolConnection;
use sqlx::{FromRow, MySql, Pool, Row, Transaction};
use url::Url;

use crate::basic::config::FrameworkConfig;
use crate::basic::error::{BIOSError, BIOSResult};
use crate::basic::json::{obj_to_json, obj_to_string};
use crate::db::domain::{BiosConfig, BiosDelRecord};
use crate::db::reldb_client::sea_query_driver_mysql::{bind_query, bind_query_as};
use rust_decimal::prelude::ToPrimitive;

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
        let pool = MySqlPoolOptions::new().max_connections(conn_max).connect(str_url).await.unwrap();
        info!(
            "[BIOS.Framework.RelDBClient] Initialized, host:{}, port:{}, max_connections:{}",
            url.host_str().unwrap_or(""),
            url.port().unwrap_or(0),
            conn_max
        );

        let sql_builder = Table::create()
            .table(BiosConfig::Table)
            .if_not_exists()
            .col(ColumnDef::new(BiosConfig::Id).integer().not_null().auto_increment().primary_key())
            .col(ColumnDef::new(BiosConfig::K).not_null().string())
            .col(ColumnDef::new(BiosConfig::V).not_null().string())
            .col(ColumnDef::new(BiosConfig::CreateUser).not_null().string())
            .col(ColumnDef::new(BiosConfig::UpdateUser).not_null().string())
            .col(ColumnDef::new(BiosConfig::CreateTime).extra("DEFAULT CURRENT_TIMESTAMP".to_string()).timestamp())
            .col(
                ColumnDef::new(BiosConfig::UpdateTime)
                    .extra("DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP".to_string())
                    .timestamp(),
            )
            .done();
        let result = bind_query(sqlx::query(&sql_builder.sql), &sql_builder.values);
        result.execute(&pool).await?;

        let sql_builder = Table::create()
            .table(BiosDelRecord::Table)
            .if_not_exists()
            .col(ColumnDef::new(BiosDelRecord::Id).integer().not_null().auto_increment().primary_key())
            .col(ColumnDef::new(BiosDelRecord::EntityName).not_null().string())
            .col(ColumnDef::new(BiosDelRecord::RecordId).not_null().string())
            .col(ColumnDef::new(BiosDelRecord::Content).not_null().text())
            .col(ColumnDef::new(BiosDelRecord::CreateUser).not_null().string())
            .col(ColumnDef::new(BiosDelRecord::CreateTime).extra("DEFAULT CURRENT_TIMESTAMP".to_string()).timestamp())
            .done();
        let result = bind_query(sqlx::query(&sql_builder.sql), &sql_builder.values);
        result.execute(&pool).await?;

        Ok(BIOSRelDBClient { pool })
    }

    pub async fn conn(&self) -> PoolConnection<MySql> {
        self.pool.acquire().await.unwrap()
    }

    pub async fn exec<'c>(&self, sql_builder: &BIOSSqlBuilder, tx: Option<&mut Transaction<'c, MySql>>) -> BIOSResult<MySqlQueryResult> {
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

    pub async fn fetch_all<'c, E>(&self, sql_builder: &BIOSSqlBuilder, tx: Option<&mut Transaction<'c, MySql>>) -> BIOSResult<Vec<E>>
    where
        E: for<'r> FromRow<'r, MySqlRow>,
        E: std::marker::Send,
        E: Unpin,
    {
        let result = bind_query_as(sqlx::query_as::<_, E>(&sql_builder.sql), &sql_builder.values);
        let result = match tx {
            Some(t) => result.fetch_all(t).await,
            None => result.fetch_all(&self.pool).await,
        };
        match result {
            Ok(rows) => BIOSResult::Ok(rows),
            Err(err) => BIOSResult::Err(BIOSError::Box(Box::new(err))),
        }
    }

    pub async fn fetch_one<'c, E>(&self, sql_builder: &BIOSSqlBuilder, tx: Option<&mut Transaction<'c, MySql>>) -> BIOSResult<E>
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

    pub async fn pagination<'c, E>(&self, sql_builder: &BIOSSqlBuilder, page_number: u64, page_size: u64, tx: Option<&mut Transaction<'c, MySql>>) -> BIOSResult<BIOSPage<E>>
    where
        E: for<'r> FromRow<'r, MySqlRow>,
        E: std::marker::Send,
        E: Unpin,
    {
        let page_sql = format!("{} LIMIT {} , {}", sql_builder.sql, (page_number - 1) * page_size, page_size);
        let result = bind_query_as(sqlx::query_as::<_, E>(&page_sql), &sql_builder.values);
        let (total_size, result) = match tx {
            Some(t) => (self.count(sql_builder, Some(t)).await?, result.fetch_all(t).await),
            None => (self.count(sql_builder, None).await?, result.fetch_all(&self.pool).await),
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

    pub async fn exists<'c>(&self, sql_builder: &BIOSSqlBuilder, tx: Option<&mut Transaction<'c, MySql>>) -> BIOSResult<bool> {
        match self.count(sql_builder, tx).await {
            Ok(count) => Ok(count != 0),
            Err(e) => Err(e),
        }
    }

    pub async fn soft_del<'c, E, R, T>(&self, table: R, id_column: T, create_user: &str, sql_builder: &BIOSSqlBuilder, tx: &mut Transaction<'c, MySql>) -> BIOSResult<bool>
    where
        R: IntoTableRef + Copy,
        T: IntoColumnRef + Copy,
        E: for<'r> FromRow<'r, MySqlRow>,
        E: std::marker::Send,
        E: Unpin,
        E: Serialize,
    {
        let table_name = format!("{:?}", &table.into_table_ref().clone());
        let table_name = table_name.as_str()[table_name.find("(").unwrap() + 1..table_name.len() - 1].to_string();
        let id_name = format!("{:?}", &id_column.into_column_ref().clone());
        let id_name = id_name.as_str()[id_name.find("(").unwrap() + 1..id_name.len() - 1].to_string();
        let mut str_ids = Vec::new();
        let mut num_ids = Vec::new();

        let rows: Vec<E> = self.fetch_all::<E>(sql_builder, Some(tx)).await?;
        for row in rows {
            let json = obj_to_json(&row).unwrap();
            let id = json[id_name.clone()].clone();
            let json = obj_to_string(&json).unwrap();
            if id.is_string() {
                str_ids.push(id.as_str().as_ref().unwrap().to_string());
            } else {
                num_ids.push(id.as_i64().as_ref().unwrap().to_i64());
            }
            let sql_builder = Query::insert()
                .into_table(BiosDelRecord::Table)
                .columns(vec![BiosDelRecord::EntityName, BiosDelRecord::RecordId, BiosDelRecord::Content, BiosDelRecord::CreateUser])
                .values_panic(vec![
                    table_name.clone().into(),
                    if id.is_string() { id.as_str().unwrap().into() } else { id.as_i64().unwrap().into() },
                    json.into(),
                    create_user.clone().into(),
                ])
                .done();
            self.exec(&sql_builder, Some(tx)).await?;
        }
        if str_ids.len() > 0 {
            let sql_builder = Query::delete().from_table(table).and_where(Expr::col(id_column).is_in(str_ids)).done();
            self.exec(&sql_builder, Some(tx)).await?;
        } else if num_ids.len() > 0 {
            let sql_builder = Query::delete().from_table(table).and_where(Expr::col(id_column).is_in(num_ids)).done();
            self.exec(&sql_builder, Some(tx)).await?;
        }
        BIOSResult::Ok(true)
    }

    pub async fn count<'c>(&self, sql_builder: &BIOSSqlBuilder, tx: Option<&mut Transaction<'c, MySql>>) -> BIOSResult<u64> {
        let count_sql = format!(
            "SELECT COUNT(1) AS _COUNT FROM ( {} ) _{}",
            sql_builder.sql,
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()
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
