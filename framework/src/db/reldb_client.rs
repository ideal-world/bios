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

use async_trait::async_trait;
use sea_query::{DeleteStatement, InsertStatement, IntoColumnRef, IntoTableRef, SelectStatement, TableCreateStatement, UpdateStatement, Values};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::{Connection, Database, FromRow, Pool, Row, Transaction};

use crate::basic::error::BIOSError;
use crate::basic::result::BIOSResult;

#[async_trait(?Send)]
pub trait BIOSRelDBClient<DB: Database, ROW: Row> {
    fn pool(&self) -> &Pool<DB>;

    async fn conn(&self) -> BIOSResult<BIOSRelDBConnection<DB>> {
        let connection = self.pool().acquire().await;
        match connection {
            Ok(conn) => BIOSResult::Ok(BIOSRelDBConnection { conn }),
            Err(err) => BIOSResult::Err(BIOSError::Box(Box::new(err))),
        }
    }

    async fn ddl_create_table<'c>(&self, sql_builder: &mut TableCreateStatement, tx: Option<&mut BIOSRelDBTransaction<'c, DB>>) -> BIOSResult<DB::QueryResult>;

    async fn insert<'c>(&self, sql_builder: &mut InsertStatement, tx: Option<&mut BIOSRelDBTransaction<'c, DB>>) -> BIOSResult<DB::QueryResult>;

    async fn update<'c>(&self, sql_builder: &mut UpdateStatement, tx: Option<&mut BIOSRelDBTransaction<'c, DB>>) -> BIOSResult<DB::QueryResult>;

    async fn delete<'c>(&self, sql_builder: &mut DeleteStatement, tx: Option<&mut BIOSRelDBTransaction<'c, DB>>) -> BIOSResult<DB::QueryResult>;

    async fn exec<'c>(&self, sql: &str, values: &Values, tx: Option<&mut BIOSRelDBTransaction<'c, DB>>) -> BIOSResult<DB::QueryResult>;

    async fn fetch_all<'c, E>(&self, sql_builder: &mut SelectStatement, tx: Option<&mut BIOSRelDBTransaction<'c, DB>>) -> BIOSResult<Vec<E>>
    where
        E: for<'r> FromRow<'r, ROW>,
        E: std::marker::Send,
        E: Unpin;

    async fn fetch_all_json<'c>(&self, sql_builder: &mut SelectStatement, tx: Option<&mut BIOSRelDBTransaction<'c, DB>>) -> BIOSResult<Vec<Value>>;

    async fn fetch_one<'c, E>(&self, sql_builder: &mut SelectStatement, tx: Option<&mut BIOSRelDBTransaction<'c, DB>>) -> BIOSResult<E>
    where
        E: for<'r> FromRow<'r, ROW>,
        E: std::marker::Send,
        E: Unpin;

    async fn fetch_one_json<'c>(&self, sql_builder: &mut SelectStatement, tx: Option<&mut BIOSRelDBTransaction<'c, DB>>) -> BIOSResult<Value>;

    async fn fetch_optional<'c, E>(&self, sql_builder: &mut SelectStatement, tx: Option<&mut BIOSRelDBTransaction<'c, DB>>) -> BIOSResult<Option<E>>
    where
        E: for<'r> FromRow<'r, ROW>,
        E: std::marker::Send,
        E: Unpin;

    async fn fetch_optional_json<'c>(&self, sql_builder: &mut SelectStatement, tx: Option<&mut BIOSRelDBTransaction<'c, DB>>) -> BIOSResult<Option<Value>>;

    async fn pagination<'c, E>(
        &self,
        sql_builder: &mut SelectStatement,
        page_number: u64,
        page_size: u64,
        tx: Option<&mut BIOSRelDBTransaction<'c, DB>>,
    ) -> BIOSResult<BIOSPage<E>>
    where
        E: for<'r> FromRow<'r, ROW>,
        E: std::marker::Send,
        E: Unpin;

    async fn pagination_json<'c>(
        &self,
        sql_builder: &mut SelectStatement,
        page_number: u64,
        page_size: u64,
        tx: Option<&mut BIOSRelDBTransaction<'c, DB>>,
    ) -> BIOSResult<BIOSPage<Value>>;

    async fn exists<'c>(&self, sql_builder: &mut SelectStatement, tx: Option<&mut BIOSRelDBTransaction<'c, DB>>) -> BIOSResult<bool>;

    async fn soft_del<'c, R, T>(&self, table: R, id_column: T, create_user: &str, sql_builder: &mut SelectStatement, tx: &mut BIOSRelDBTransaction<'c, DB>) -> BIOSResult<bool>
    where
        R: IntoTableRef + Copy,
        T: IntoColumnRef + Copy;

    async fn count<'c>(&self, sql_builder: &mut SelectStatement, tx: Option<&mut BIOSRelDBTransaction<'c, DB>>) -> BIOSResult<u64>;

    fn convert_row_to_json(&self, row: &ROW) -> Value;
}

pub struct BIOSRelDBConnection<DB: Database> {
    conn: PoolConnection<DB>,
}

pub struct BIOSRelDBTransaction<'c, DB: Database> {
    pub tx: Transaction<'c, DB>,
}

impl<DB: Database> BIOSRelDBConnection<DB> {
    pub async fn begin<'c>(&'c mut self) -> BIOSResult<BIOSRelDBTransaction<'c, DB>> {
        let result = self.conn.begin().await;
        match result {
            Ok(tx) => BIOSResult::Ok(BIOSRelDBTransaction::<'c, DB> { tx }),
            Err(err) => BIOSResult::Err(BIOSError::Box(Box::new(err))),
        }
    }
}

impl<'c, DB: Database> BIOSRelDBTransaction<'c, DB> {
    pub async fn commit(self) -> BIOSResult<()> {
        match self.tx.commit().await {
            Ok(_) => BIOSResult::Ok(()),
            Err(err) => BIOSResult::Err(BIOSError::Box(Box::new(err))),
        }
    }

    pub async fn rollback(self) -> BIOSResult<()> {
        match self.tx.rollback().await {
            Ok(_) => BIOSResult::Ok(()),
            Err(err) => BIOSResult::Err(BIOSError::Box(Box::new(err))),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BIOSPage<E> {
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
