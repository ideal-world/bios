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

use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime, SecondsFormat, TimeZone};
use log::info;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use sea_query::{
    ColumnDef, DeleteStatement, Expr, InsertStatement, IntoColumnRef, IntoTableRef, PostgresQueryBuilder, Query, SelectStatement, Table, TableCreateStatement, UpdateStatement,
    Values,
};
use serde_json::Value;
use sqlx::postgres::{PgPoolOptions, PgQueryResult, PgRow};
use sqlx::types::chrono::Utc;
use sqlx::{Column, FromRow, Pool, Postgres, Row, TypeInfo};
use url::Url;

use crate::basic::config::FrameworkConfig;
use crate::basic::error::BIOSError;
use crate::basic::result::BIOSResult;
use crate::db::domain::{BiosConfig, BiosDelRecord};
use crate::db::reldb_client::{BIOSPage, BIOSRelDBClient, BIOSRelDBTransaction};
use crate::db::reldb_client_postgres::sea_query_driver_postgres::{bind_query as bind_query_postgres, bind_query_as as bind_query_as_postgres};
use crate::BIOSFuns;

sea_query::sea_query_driver_postgres!();

pub struct BIOSRelDBPostgresClient {
    pool: Pool<Postgres>,
}

impl BIOSRelDBPostgresClient {
    pub async fn init_by_conf(conf: &FrameworkConfig) -> BIOSResult<Self> {
        BIOSRelDBPostgresClient::init(&conf.db.url, conf.db.max_connections).await
    }

    pub async fn init(str_url: &str, conn_max: u32) -> BIOSResult<Self> {
        let url = Url::parse(str_url)?;
        info!(
            "[BIOS.Framework.RelDBClient] Initializing, host:{}, port:{}, max_connections:{}",
            url.host_str().unwrap_or(""),
            url.port().unwrap_or(0),
            conn_max
        );
        let pool = PgPoolOptions::new().max_connections(conn_max).connect(str_url).await.unwrap();
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
            // TODO update time
            .col(ColumnDef::new(BiosConfig::UpdateTime).extra("DEFAULT CURRENT_TIMESTAMP".to_string()).timestamp())
            .build(PostgresQueryBuilder)
            .to_string();
        bind_query_postgres(sqlx::query(&sql), &Values(vec![])).execute(&pool).await?;

        let sql = Table::create()
            .table(BiosDelRecord::Table)
            .if_not_exists()
            .col(ColumnDef::new(BiosDelRecord::Id).integer().not_null().auto_increment().primary_key())
            .col(ColumnDef::new(BiosDelRecord::EntityName).not_null().string())
            .col(ColumnDef::new(BiosDelRecord::RecordId).not_null().string())
            .col(ColumnDef::new(BiosDelRecord::Content).not_null().text())
            .col(ColumnDef::new(BiosDelRecord::CreateUser).not_null().string())
            .col(ColumnDef::new(BiosDelRecord::CreateTime).extra("DEFAULT CURRENT_TIMESTAMP".to_string()).timestamp())
            .build(PostgresQueryBuilder)
            .to_string();
        bind_query_postgres(sqlx::query(&sql), &Values(vec![])).execute(&pool).await?;

        Ok(BIOSRelDBPostgresClient { pool })
    }
}

#[async_trait(?Send)]
impl BIOSRelDBClient<Postgres, PgRow> for BIOSRelDBPostgresClient {
    fn pool(&self) -> &Pool<Postgres> {
        &self.pool
    }

    async fn ddl_create_table<'c>(&self, sql_builder: &mut TableCreateStatement, tx: Option<&mut BIOSRelDBTransaction<'c, Postgres>>) -> BIOSResult<PgQueryResult> {
        let sql = sql_builder.build(PostgresQueryBuilder).to_string();
        self.exec(&sql, &Values(vec![]), tx).await
    }

    async fn insert<'c>(&self, sql_builder: &mut InsertStatement, tx: Option<&mut BIOSRelDBTransaction<'c, Postgres>>) -> BIOSResult<PgQueryResult> {
        let (sql, values) = sql_builder.build(PostgresQueryBuilder);
        self.exec(&sql, &values, tx).await
    }

    async fn update<'c>(&self, sql_builder: &mut UpdateStatement, tx: Option<&mut BIOSRelDBTransaction<'c, Postgres>>) -> BIOSResult<PgQueryResult> {
        let (sql, values) = sql_builder.build(PostgresQueryBuilder);
        self.exec(&sql, &values, tx).await
    }

    async fn delete<'c>(&self, sql_builder: &mut DeleteStatement, tx: Option<&mut BIOSRelDBTransaction<'c, Postgres>>) -> BIOSResult<PgQueryResult> {
        let (sql, values) = sql_builder.build(PostgresQueryBuilder);
        self.exec(&sql, &values, tx).await
    }

    async fn exec<'c>(&self, sql: &str, values: &Values, tx: Option<&mut BIOSRelDBTransaction<'c, Postgres>>) -> BIOSResult<PgQueryResult> {
        let result = bind_query_postgres(sqlx::query(sql), values);
        let result = match tx {
            Some(t) => result.execute(&mut t.tx).await,
            None => result.execute(&self.pool).await,
        };
        match result {
            Ok(ok) => BIOSResult::Ok(ok),
            Err(err) => BIOSResult::Err(BIOSError::Box(Box::new(err))),
        }
    }

    async fn fetch_all<'c, E>(&self, sql_builder: &mut SelectStatement, tx: Option<&mut BIOSRelDBTransaction<'c, Postgres>>) -> BIOSResult<Vec<E>>
    where
        E: for<'r> FromRow<'r, PgRow>,
        E: std::marker::Send,
        E: Unpin,
    {
        let (sql, values) = sql_builder.build(PostgresQueryBuilder);
        let result = bind_query_as_postgres(sqlx::query_as::<_, E>(&sql), &values);
        let result = match tx {
            Some(t) => result.fetch_all(&mut t.tx).await,
            None => result.fetch_all(&self.pool).await,
        };
        match result {
            Ok(rows) => BIOSResult::Ok(rows),
            Err(err) => BIOSResult::Err(BIOSError::Box(Box::new(err))),
        }
    }

    async fn fetch_all_json<'c>(&self, sql_builder: &mut SelectStatement, tx: Option<&mut BIOSRelDBTransaction<'c, Postgres>>) -> BIOSResult<Vec<Value>> {
        let (sql, values) = sql_builder.build(PostgresQueryBuilder);
        let result = bind_query_postgres(sqlx::query(&sql), &values);
        let result = match tx {
            Some(t) => result.fetch_all(&mut t.tx).await,
            None => result.fetch_all(&self.pool).await,
        };
        match result {
            Ok(rows) => BIOSResult::Ok(rows.iter().map(|row| self.convert_row_to_json(row)).collect::<Vec<Value>>()),
            Err(err) => BIOSResult::Err(BIOSError::Box(Box::new(err))),
        }
    }

    async fn fetch_one<'c, E>(&self, sql_builder: &mut SelectStatement, tx: Option<&mut BIOSRelDBTransaction<'c, Postgres>>) -> BIOSResult<E>
    where
        E: for<'r> FromRow<'r, PgRow>,
        E: std::marker::Send,
        E: Unpin,
    {
        match self.fetch_optional::<E>(sql_builder, tx).await? {
            Some(row) => BIOSResult::Ok(row),
            None => BIOSResult::Err(BIOSError::NotFound("Record not exists".to_string())),
        }
    }

    async fn fetch_one_json<'c>(&self, sql_builder: &mut SelectStatement, tx: Option<&mut BIOSRelDBTransaction<'c, Postgres>>) -> BIOSResult<Value> {
        match self.fetch_optional_json(sql_builder, tx).await? {
            Some(row) => BIOSResult::Ok(row),
            None => BIOSResult::Err(BIOSError::NotFound("Record not exists".to_string())),
        }
    }

    async fn fetch_optional<'c, E>(&self, sql_builder: &mut SelectStatement, tx: Option<&mut BIOSRelDBTransaction<'c, Postgres>>) -> BIOSResult<Option<E>>
    where
        E: for<'r> FromRow<'r, PgRow>,
        E: std::marker::Send,
        E: Unpin,
    {
        let (sql, values) = sql_builder.build(PostgresQueryBuilder);
        let fetch_one_sql = format!("{} LIMIT 1", sql);
        let result = bind_query_as_postgres(sqlx::query_as::<_, E>(&fetch_one_sql), &values);
        let result = match tx {
            Some(t) => result.fetch_optional(&mut t.tx).await,
            None => result.fetch_optional(&self.pool).await,
        };
        match result {
            Ok(row_opt) => BIOSResult::Ok(row_opt),
            Err(err) => BIOSResult::Err(BIOSError::Box(Box::new(err))),
        }
    }

    async fn fetch_optional_json<'c>(&self, sql_builder: &mut SelectStatement, tx: Option<&mut BIOSRelDBTransaction<'c, Postgres>>) -> BIOSResult<Option<Value>> {
        let (sql, values) = sql_builder.build(PostgresQueryBuilder);
        let fetch_one_sql = format!("{} LIMIT 1", sql);
        let result = bind_query_postgres(sqlx::query(&fetch_one_sql), &values);
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
        tx: Option<&mut BIOSRelDBTransaction<'c, Postgres>>,
    ) -> BIOSResult<BIOSPage<E>>
    where
        E: for<'r> FromRow<'r, PgRow>,
        E: std::marker::Send,
        E: Unpin,
    {
        let (sql, values) = sql_builder.build(PostgresQueryBuilder);
        let page_sql = format!("{} LIMIT {} , {}", sql, (page_number - 1) * page_size, page_size);
        let result = bind_query_as_postgres(sqlx::query_as::<_, E>(&page_sql), &values);
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
        tx: Option<&mut BIOSRelDBTransaction<'c, Postgres>>,
    ) -> BIOSResult<BIOSPage<Value>> {
        let (sql, values) = sql_builder.build(PostgresQueryBuilder);
        let page_sql = format!("{} LIMIT {} , {}", sql, (page_number - 1) * page_size, page_size);
        let result = bind_query_postgres(sqlx::query(&page_sql), &values);
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

    async fn exists<'c>(&self, sql_builder: &mut SelectStatement, tx: Option<&mut BIOSRelDBTransaction<'c, Postgres>>) -> BIOSResult<bool> {
        match self.count(sql_builder, tx).await {
            Ok(count) => Ok(count != 0),
            Err(e) => Err(e),
        }
    }

    async fn soft_del<'c, R, T>(
        &self,
        table: R,
        id_column: T,
        create_user: &str,
        sql_builder: &mut SelectStatement,
        tx: &mut BIOSRelDBTransaction<'c, Postgres>,
    ) -> BIOSResult<bool>
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

    async fn count<'c>(&self, sql_builder: &mut SelectStatement, tx: Option<&mut BIOSRelDBTransaction<'c, Postgres>>) -> BIOSResult<u64> {
        let (sql, values) = sql_builder.build(PostgresQueryBuilder);
        let count_sql = format!(
            "SELECT COUNT(1) AS _COUNT FROM ( {} ) _{}",
            sql,
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()
        );
        let result = bind_query_postgres(sqlx::query(&count_sql), &values);
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

    fn convert_row_to_json(&self, row: &PgRow) -> Value {
        let mut json = BTreeMap::new();

        for col in row.columns() {
            match col.type_info().name() {
                "CHAR" | "VARCHAR" | "CIDR" | "TEXT" | "NAME" | "OID" => {
                    let v: String = row.get(col.name());
                    json.insert(col.name(), Value::from(v));
                }
                "BOOLEAN" => {
                    let v: bool = row.get(col.name());
                    json.insert(col.name(), Value::from(v));
                }
                "INT2" => {
                    let v: i16 = row.get(col.name());
                    json.insert(col.name(), Value::from(v));
                }
                "INT4" => {
                    let v: i32 = row.get(col.name());
                    json.insert(col.name(), Value::from(v));
                }
                "INT8" => {
                    let v: i64 = row.get(col.name());
                    json.insert(col.name(), Value::from(v));
                }
                "FLOAT4" | "FLOAT8" => {
                    let v: f64 = row.get(col.name());
                    json.insert(col.name(), Value::from(v));
                }
                "VOID" => {
                    json.insert(col.name(), Value::Null);
                }
                "TIMESTAMP" => {
                    let v: NaiveDateTime = row.get(col.name());
                    let v = Utc.from_utc_datetime(&v);
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
                "MONEY" => {
                    let v: Decimal = row.get(col.name());
                    json.insert(col.name(), Value::from(v.to_string()));
                }
                // TODO
                _ => panic!("Unsupported data type [{}]", col.type_info().name()),
            }
        }
        BIOSFuns::json.obj_to_json(&json).expect("Json parse error")
    }
}
