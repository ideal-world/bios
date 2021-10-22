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

use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, NaiveTime, SecondsFormat};
use log::info;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use sea_query::{
    ColumnDef, DeleteStatement, Expr, InsertStatement, IntoColumnRef, IntoTableRef, MysqlQueryBuilder, Query, SelectStatement, Table, TableCreateStatement, UpdateStatement, Values,
};
use serde_json::Value;
use sqlx::mysql::{MySqlPoolOptions, MySqlQueryResult, MySqlRow};
use sqlx::types::chrono::Utc;
use sqlx::{Column, FromRow, MySql, Pool, Row, TypeInfo};
use url::Url;

use crate::basic::config::FrameworkConfig;
use crate::basic::error::BIOSError;
use crate::basic::result::BIOSResult;
use crate::db::domain::{BiosConfig, BiosDelRecord};
use crate::db::reldb_client::{BIOSPage, BIOSRelDBClient, BIOSRelDBTransaction};
use crate::db::reldb_client_mysql::sea_query_driver_mysql::{bind_query as bind_query_mysql, bind_query_as as bind_query_as_mysql};
use crate::BIOSFuns;

sea_query::sea_query_driver_mysql!();

pub struct BIOSRelDBMysqlClient {
    pool: Pool<MySql>,
}

impl BIOSRelDBMysqlClient {
    pub async fn init_by_conf(conf: &FrameworkConfig) -> BIOSResult<Self> {
        BIOSRelDBMysqlClient::init(&conf.db.url, conf.db.max_connections).await
    }

    pub async fn init(str_url: &str, conn_max: u32) -> BIOSResult<Self> {
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

        let sql = Table::create()
            .table(BiosConfig::Table)
            .if_not_exists()
            .col(ColumnDef::new(BiosConfig::Id).integer().not_null().auto_increment().primary_key())
            .col(ColumnDef::new(BiosConfig::K).not_null().string())
            .col(ColumnDef::new(BiosConfig::V).not_null().string())
            .col(ColumnDef::new(BiosConfig::CreateUser).not_null().string())
            .col(ColumnDef::new(BiosConfig::UpdateUser).not_null().string())
            .col(ColumnDef::new(BiosConfig::CreateTime).extra("DEFAULT CURRENT_TIMESTAMP".to_string()).timestamp())
            .col(ColumnDef::new(BiosConfig::UpdateTime).extra("DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP".to_string()).timestamp())
            .build(MysqlQueryBuilder)
            .to_string();
        bind_query_mysql(sqlx::query(&sql), &Values(vec![])).execute(&pool).await?;

        let sql = Table::create()
            .table(BiosDelRecord::Table)
            .if_not_exists()
            .col(ColumnDef::new(BiosDelRecord::Id).integer().not_null().auto_increment().primary_key())
            .col(ColumnDef::new(BiosDelRecord::EntityName).not_null().string())
            .col(ColumnDef::new(BiosDelRecord::RecordId).not_null().string())
            .col(ColumnDef::new(BiosDelRecord::Content).not_null().text())
            .col(ColumnDef::new(BiosDelRecord::CreateUser).not_null().string())
            .col(ColumnDef::new(BiosDelRecord::CreateTime).extra("DEFAULT CURRENT_TIMESTAMP".to_string()).timestamp())
            .build(MysqlQueryBuilder)
            .to_string();
        bind_query_mysql(sqlx::query(&sql), &Values(vec![])).execute(&pool).await?;

        Ok(BIOSRelDBMysqlClient { pool })
    }
}

#[async_trait(?Send)]
impl BIOSRelDBClient<MySql, MySqlRow> for BIOSRelDBMysqlClient {
    fn pool(&self) -> &Pool<MySql> {
        &self.pool
    }

    async fn ddl_create_table<'c>(&self, sql_builder: &mut TableCreateStatement, tx: Option<&mut BIOSRelDBTransaction<'c, MySql>>) -> BIOSResult<MySqlQueryResult> {
        let sql = sql_builder.build(MysqlQueryBuilder).to_string();
        self.exec(&sql, &Values(vec![]), tx).await
    }

    async fn insert<'c>(&self, sql_builder: &mut InsertStatement, tx: Option<&mut BIOSRelDBTransaction<'c, MySql>>) -> BIOSResult<MySqlQueryResult> {
        let (sql, values) = sql_builder.build(MysqlQueryBuilder);
        self.exec(&sql, &values, tx).await
    }

    async fn update<'c>(&self, sql_builder: &mut UpdateStatement, tx: Option<&mut BIOSRelDBTransaction<'c, MySql>>) -> BIOSResult<MySqlQueryResult> {
        let (sql, values) = sql_builder.build(MysqlQueryBuilder);
        self.exec(&sql, &values, tx).await
    }

    async fn delete<'c>(&self, sql_builder: &mut DeleteStatement, tx: Option<&mut BIOSRelDBTransaction<'c, MySql>>) -> BIOSResult<MySqlQueryResult> {
        let (sql, values) = sql_builder.build(MysqlQueryBuilder);
        self.exec(&sql, &values, tx).await
    }

    async fn exec<'c>(&self, sql: &str, values: &Values, tx: Option<&mut BIOSRelDBTransaction<'c, MySql>>) -> BIOSResult<MySqlQueryResult> {
        let result = bind_query_mysql(sqlx::query(sql), values);
        let result = match tx {
            Some(t) => result.execute(&mut t.tx).await,
            None => result.execute(&self.pool).await,
        };
        match result {
            Ok(ok) => BIOSResult::Ok(ok),
            Err(err) => BIOSResult::Err(BIOSError::Box(Box::new(err))),
        }
    }

    async fn fetch_all<'c, E>(&self, sql_builder: &mut SelectStatement, tx: Option<&mut BIOSRelDBTransaction<'c, MySql>>) -> BIOSResult<Vec<E>>
    where
        E: for<'r> FromRow<'r, MySqlRow>,
        E: std::marker::Send,
        E: Unpin,
    {
        let (sql, values) = sql_builder.build(MysqlQueryBuilder);
        let result = bind_query_as_mysql(sqlx::query_as::<_, E>(&sql), &values);
        let result = match tx {
            Some(t) => result.fetch_all(&mut t.tx).await,
            None => result.fetch_all(&self.pool).await,
        };
        match result {
            Ok(rows) => BIOSResult::Ok(rows),
            Err(err) => BIOSResult::Err(BIOSError::Box(Box::new(err))),
        }
    }

    async fn fetch_all_json<'c>(&self, sql_builder: &mut SelectStatement, tx: Option<&mut BIOSRelDBTransaction<'c, MySql>>) -> BIOSResult<Vec<Value>> {
        let (sql, values) = sql_builder.build(MysqlQueryBuilder);
        let result = bind_query_mysql(sqlx::query(&sql), &values);
        let result = match tx {
            Some(t) => result.fetch_all(&mut t.tx).await,
            None => result.fetch_all(&self.pool).await,
        };
        match result {
            Ok(rows) => BIOSResult::Ok(rows.iter().map(|row| self.convert_row_to_json(row)).collect::<Vec<Value>>()),
            Err(err) => BIOSResult::Err(BIOSError::Box(Box::new(err))),
        }
    }

    async fn fetch_one<'c, E>(&self, sql_builder: &mut SelectStatement, tx: Option<&mut BIOSRelDBTransaction<'c, MySql>>) -> BIOSResult<E>
    where
        E: for<'r> FromRow<'r, MySqlRow>,
        E: std::marker::Send,
        E: Unpin,
    {
        match self.fetch_optional::<E>(sql_builder, tx).await? {
            Some(row) => BIOSResult::Ok(row),
            None => BIOSResult::Err(BIOSError::NotFound("Record not exists".to_string())),
        }
    }

    async fn fetch_one_json<'c>(&self, sql_builder: &mut SelectStatement, tx: Option<&mut BIOSRelDBTransaction<'c, MySql>>) -> BIOSResult<Value> {
        match self.fetch_optional_json(sql_builder, tx).await? {
            Some(row) => BIOSResult::Ok(row),
            None => BIOSResult::Err(BIOSError::NotFound("Record not exists".to_string())),
        }
    }

    async fn fetch_optional<'c, E>(&self, sql_builder: &mut SelectStatement, tx: Option<&mut BIOSRelDBTransaction<'c, MySql>>) -> BIOSResult<Option<E>>
    where
        E: for<'r> FromRow<'r, MySqlRow>,
        E: std::marker::Send,
        E: Unpin,
    {
        let (sql, values) = sql_builder.build(MysqlQueryBuilder);
        let fetch_one_sql = format!("{} LIMIT 1", sql);
        let result = bind_query_as_mysql(sqlx::query_as::<_, E>(&fetch_one_sql), &values);
        let result = match tx {
            Some(t) => result.fetch_optional(&mut t.tx).await,
            None => result.fetch_optional(&self.pool).await,
        };
        match result {
            Ok(row_opt) => BIOSResult::Ok(row_opt),
            Err(err) => BIOSResult::Err(BIOSError::Box(Box::new(err))),
        }
    }

    async fn fetch_optional_json<'c>(&self, sql_builder: &mut SelectStatement, tx: Option<&mut BIOSRelDBTransaction<'c, MySql>>) -> BIOSResult<Option<Value>> {
        let (sql, values) = sql_builder.build(MysqlQueryBuilder);
        let fetch_one_sql = format!("{} LIMIT 1", sql);
        let result = bind_query_mysql(sqlx::query(&fetch_one_sql), &values);
        let result = match tx {
            Some(t) => result.fetch_optional(&mut t.tx).await,
            None => result.fetch_optional(&self.pool).await,
        };
        match result {
            Ok(row_opt) => match row_opt {
                Some(row) => BIOSResult::Ok(Some(self.convert_row_to_json(&row))),
                None => BIOSResult::Ok(None),
            },
            Err(err) => BIOSResult::Err(BIOSError::Box(Box::new(err))),
        }
    }

    async fn pagination<'c, E>(
        &self,
        sql_builder: &mut SelectStatement,
        page_number: u64,
        page_size: u64,
        tx: Option<&mut BIOSRelDBTransaction<'c, MySql>>,
    ) -> BIOSResult<BIOSPage<E>>
    where
        E: for<'r> FromRow<'r, MySqlRow>,
        E: std::marker::Send,
        E: Unpin,
    {
        let (sql, values) = sql_builder.build(MysqlQueryBuilder);
        let page_sql = format!("{} LIMIT {} , {}", sql, (page_number - 1) * page_size, page_size);
        let result = bind_query_as_mysql(sqlx::query_as::<_, E>(&page_sql), &values);
        let (total_size, result) = match tx {
            Some(t) => (self.count(sql_builder, Some(t)).await?, result.fetch_all(&mut t.tx).await),
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

    async fn pagination_json<'c>(
        &self,
        sql_builder: &mut SelectStatement,
        page_number: u64,
        page_size: u64,
        tx: Option<&mut BIOSRelDBTransaction<'c, MySql>>,
    ) -> BIOSResult<BIOSPage<Value>> {
        let (sql, values) = sql_builder.build(MysqlQueryBuilder);
        let page_sql = format!("{} LIMIT {} , {}", sql, (page_number - 1) * page_size, page_size);
        let result = bind_query_mysql(sqlx::query(&page_sql), &values);
        let (total_size, result) = match tx {
            Some(t) => (self.count(sql_builder, Some(t)).await?, result.fetch_all(&mut t.tx).await),
            None => (self.count(sql_builder, None).await?, result.fetch_all(&self.pool).await),
        };
        match result {
            Ok(rows) => BIOSResult::Ok(BIOSPage {
                page_size,
                page_number,
                total_size,
                records: rows.iter().map(|row| self.convert_row_to_json(row)).collect::<Vec<Value>>(),
            }),
            Err(err) => BIOSResult::Err(BIOSError::Box(Box::new(err))),
        }
    }

    async fn exists<'c>(&self, sql_builder: &mut SelectStatement, tx: Option<&mut BIOSRelDBTransaction<'c, MySql>>) -> BIOSResult<bool> {
        match self.count(sql_builder, tx).await {
            Ok(count) => Ok(count != 0),
            Err(e) => Err(e),
        }
    }

    async fn soft_del<'c, R, T>(&self, table: R, id_column: T, create_user: &str, sql_builder: &mut SelectStatement, tx: &mut BIOSRelDBTransaction<'c, MySql>) -> BIOSResult<bool>
    where
        R: IntoTableRef + Copy,
        T: IntoColumnRef + Copy,
    {
        let table_name = format!("{:?}", &table.into_table_ref().clone());
        let table_name = table_name.as_str()[table_name.find("(").unwrap() + 1..table_name.len() - 1].to_string();
        let id_name = format!("{:?}", &id_column.into_column_ref().clone());
        let id_name = id_name.as_str()[id_name.find("(").unwrap() + 1..id_name.len() - 1].to_string();
        let mut str_ids = Vec::new();
        let mut num_ids = Vec::new();

        let rows: Vec<Value> = self.fetch_all_json(sql_builder, Some(tx)).await?;
        for row in rows {
            let id = row[id_name.clone()].clone();
            let json = BIOSFuns::json.obj_to_string(&row).unwrap();
            if id.is_string() {
                str_ids.push(id.as_str().as_ref().unwrap().to_string());
            } else {
                num_ids.push(id.as_i64().as_ref().unwrap().to_i64());
            }
            self.insert(
                Query::insert()
                    .into_table(BiosDelRecord::Table)
                    .columns(vec![BiosDelRecord::EntityName, BiosDelRecord::RecordId, BiosDelRecord::Content, BiosDelRecord::CreateUser])
                    .values_panic(vec![
                        table_name.clone().into(),
                        if id.is_string() { id.as_str().unwrap().into() } else { id.as_i64().unwrap().into() },
                        json.into(),
                        create_user.clone().into(),
                    ]),
                Some(tx),
            )
            .await?;
        }
        if str_ids.len() > 0 {
            self.delete(Query::delete().from_table(table).and_where(Expr::col(id_column).is_in(str_ids)), Some(tx)).await?;
        } else if num_ids.len() > 0 {
            self.delete(Query::delete().from_table(table).and_where(Expr::col(id_column).is_in(num_ids)), Some(tx)).await?;
        }
        BIOSResult::Ok(true)
    }

    async fn count<'c>(&self, sql_builder: &mut SelectStatement, tx: Option<&mut BIOSRelDBTransaction<'c, MySql>>) -> BIOSResult<u64> {
        let (sql, values) = sql_builder.build(MysqlQueryBuilder);
        let count_sql = format!(
            "SELECT COUNT(1) AS _COUNT FROM ( {} ) _{}",
            sql,
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()
        );
        let result = bind_query_mysql(sqlx::query(&count_sql), &values);
        let result = match tx {
            Some(t) => result.fetch_one(&mut t.tx).await,
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

    fn convert_row_to_json(&self, row: &MySqlRow) -> Value {
        let mut json = BTreeMap::new();

        for col in row.columns() {
            match col.type_info().name() {
                "CHAR" | "VARCHAR" | "TINYTEXT" | "TEXT" | "MEDIUMTEXT" | "LONGTEXT" | "ENUM" => {
                    let v: String = row.get(col.name());
                    json.insert(col.name(), Value::from(v));
                }
                "BOOLEAN" => {
                    let v: bool = row.get(col.name());
                    json.insert(col.name(), Value::from(v));
                }
                "TINYINT UNSIGNED" => {
                    let v: u8 = row.get(col.name());
                    json.insert(col.name(), Value::from(v));
                }
                "SMALLINT UNSIGNED" => {
                    let v: u16 = row.get(col.name());
                    json.insert(col.name(), Value::from(v));
                }
                "MEDIUMINT UNSIGNED" | "INT UNSIGNED" => {
                    let v: u32 = row.get(col.name());
                    json.insert(col.name(), Value::from(v));
                }
                "BIGINT UNSIGNED" => {
                    let v: u64 = row.get(col.name());
                    json.insert(col.name(), Value::from(v));
                }
                "TINYINT" => {
                    let v: i8 = row.get(col.name());
                    json.insert(col.name(), Value::from(v));
                }
                "SMALLINT" => {
                    let v: i16 = row.get(col.name());
                    json.insert(col.name(), Value::from(v));
                }
                "MEDIUMINT" | "INT" => {
                    let v: i32 = row.get(col.name());
                    json.insert(col.name(), Value::from(v));
                }
                "BIGINT" => {
                    let v: i64 = row.get(col.name());
                    json.insert(col.name(), Value::from(v));
                }
                "FLOAT" | "DOUBLE" => {
                    let v: f64 = row.get(col.name());
                    json.insert(col.name(), Value::from(v));
                }
                "NULL" => {
                    json.insert(col.name(), Value::Null);
                }
                "DATETIME" | "TIMESTAMP" => {
                    let v: DateTime<Utc> = row.get(col.name());
                    json.insert(col.name(), Value::from(v.to_rfc3339_opts(SecondsFormat::Millis, true)));
                }
                "DATE" => {
                    let v: NaiveDate = row.get(col.name());
                    json.insert(col.name(), Value::from(v.to_string()));
                }
                "TIME" => {
                    let v: NaiveTime = row.get(col.name());
                    json.insert(col.name(), Value::from(v.to_string()));
                }
                "DECIMAL" => {
                    let v: Decimal = row.get(col.name());
                    json.insert(col.name(), Value::from(v.to_string()));
                }
                // TODO
                // YEAR | BIT | SET | GEOMETRY | JSON | BINARY | VARBINARY | TINYBLOB | BLOB | MEDIUMBLOB | LONGBLOB
                _ => panic!("Unsupported data type [{}]", col.type_info().name()),
            }
        }
        BIOSFuns::json.obj_to_json(&json).expect("Json parse error")
    }
}
