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

use async_trait::async_trait;
use log::info;
use sea_orm::entity::*;
use sea_orm::sea_query::TableCreateStatement;
use sea_orm::ActiveValue::Set;
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbBackend, DbErr, EntityTrait, ExecResult, QueryTrait, Schema, Select, Statement};
use sqlparser::ast;
use sqlparser::ast::{SetExpr, TableFactor};
use sqlparser::dialect::MySqlDialect;
use sqlparser::parser::Parser;
use url::Url;

use crate::basic::error::BIOSError;
use crate::basic::result::BIOSResult;
use crate::db::domain::{bios_db_config, bios_db_del_record};
use crate::{BIOSFuns, FrameworkConfig};

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
        client.init_basic_tables().await?;
        Ok(client)
    }

    pub fn conn(&self) -> &DatabaseConnection {
        &self.con
    }

    pub fn backend(&self) -> DbBackend {
        self.con.get_database_backend()
    }

    async fn init_basic_tables(&self) -> BIOSResult<ExecResult> {
        let config_statement = bios_db_config::create_table_statement(self.con.get_database_backend());
        self.create_table_from_statement(&config_statement).await?;
        let config_statement = bios_db_del_record::create_table_statement(self.con.get_database_backend());
        self.create_table_from_statement(&config_statement).await
    }

    /// TODO 不支持 not_null nullable  default_value  default_expr indexed, unique 等
    pub async fn create_table_from_entity<E>(&self, entity: E) -> BIOSResult<ExecResult>
    where
        E: EntityTrait,
    {
        let builder = self.con.get_database_backend();
        let schema = Schema::new(builder);
        let table_create_statement = &schema.create_table_from_entity(entity);
        self.create_table_from_statement(table_create_statement).await
    }

    pub async fn create_table_from_statement(&self, statement: &TableCreateStatement) -> BIOSResult<ExecResult> {
        let statement = self.con.get_database_backend().build(statement);
        self.execute(statement).await
    }

    pub async fn execute(&self, statement: Statement) -> BIOSResult<ExecResult> {
        let result = self.con.execute(statement).await;
        match result {
            Ok(ok) => BIOSResult::Ok(ok),
            Err(err) => BIOSResult::Err(BIOSError::Box(Box::new(err))),
        }
    }
}

#[async_trait(?Send)]
pub trait BIOSSeaORMExtend<'a> {
    async fn soft_delete<C>(self, db: &'a C, create_user: &str) -> BIOSResult<()>
    where
        C: ConnectionTrait<'a>;

    async fn soft_delete_custom<C>(self, db: &'a C, create_user: &str, custom_pk_field: &str) -> BIOSResult<()>
    where
        C: ConnectionTrait<'a>;
}

#[async_trait(?Send)]
impl<'a, E> BIOSSeaORMExtend<'a> for Select<E>
where
    E: EntityTrait,
{
    async fn soft_delete<C>(self, db: &'a C, create_user: &str) -> BIOSResult<()>
    where
        C: ConnectionTrait<'a>,
    {
        self.soft_delete_custom(db, create_user, "id").await
    }

    async fn soft_delete_custom<C>(self, db: &'a C, create_user: &str, custom_pk_field: &str) -> BIOSResult<()>
    where
        C: ConnectionTrait<'a>,
    {
        let db_backend: DbBackend = db.get_database_backend();

        let sql = self.build(db_backend).sql.replace("?", "''");
        let ast: ast::Statement = Parser::parse_sql(&MySqlDialect {}, &sql).unwrap().pop().unwrap();

        let mut table_name = String::new();
        if let ast::Statement::Query(query) = ast {
            if let SetExpr::Select(select) = (*query).body {
                if let TableFactor::Table { name, .. } = &select.from[0].relation {
                    table_name = name.0[0].value.clone();
                }
            }
        }
        if table_name.is_empty() {
            return BIOSResult::Err(BIOSError::Conflict(
                "sql parsing error, the name of the table \
            to be soft deleted was not found"
                    .to_string(),
            ));
        }

        let mut ids: Vec<Value> = Vec::new();

        let rows = self.into_json().all(db).await?;
        for row in rows {
            let id = row[custom_pk_field.clone()].clone();
            let json = BIOSFuns::json.obj_to_string(&row).unwrap();
            if id.is_string() {
                ids.push(id.as_str().as_ref().unwrap().to_string().into());
            } else {
                ids.push(id.as_u64().unwrap().into());
            }
            bios_db_del_record::ActiveModel {
                entity_name: Set(table_name.to_string()),
                record_id: Set(id.to_string()),
                content: Set(json.into()),
                creator: Set(create_user.to_string()),
                ..Default::default()
            }
            .insert(db)
            .await?;
        }
        if ids.len() == 0 {
            return Ok(());
        }
        let statement = Statement::from_sql_and_values(
            db_backend,
            match db_backend {
                DbBackend::Postgres => format!("DELETE FROM {} WHERE id in ($1)", table_name),
                _ => format!("DELETE FROM {} WHERE id in (?)", table_name),
            }
            .as_str(),
            ids,
        );
        let result = db.execute(statement).await;
        match result {
            Ok(_) => BIOSResult::Ok(()),
            Err(err) => BIOSResult::Err(BIOSError::Box(Box::new(err))),
        }
    }
}

impl From<DbErr> for BIOSError {
    fn from(error: DbErr) -> Self {
        BIOSError::Box(Box::new(error))
    }
}
