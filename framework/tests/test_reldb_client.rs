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

// https://github.com/SeaQL/sea-query

use chrono::{Local, NaiveDateTime};
use sea_query::{ColumnDef, Expr, Iden, Order, Query, Table};
use serde::Serialize;
use strum::EnumIter;
use strum::IntoEnumIterator;
use testcontainers::clients;

use bios::basic::config::{BIOSConfig, CacheConfig, DBConfig, FrameworkConfig, MQConfig, NoneConfig};
use bios::basic::result::BIOSResult;
use bios::db::domain::BiosDelRecord;
use bios::db::reldb_client::BIOSRelDBClient;
use bios::db::reldb_client_mysql::BIOSRelDBMysqlClient;
use bios::db::reldb_client_postgres::BIOSRelDBPostgresClient;
use bios::test::test_container::BIOSTestContainer;
use bios::BIOSFuns;

#[derive(Iden, EnumIter, PartialEq, Copy, Clone)]
enum Employees {
    Table,
    Name,
}

#[derive(sqlx::FromRow, Debug)]
struct EmployeesStruct {
    name: String,
}

#[derive(Iden, EnumIter, PartialEq, Copy, Clone)]
enum BizActivity {
    Table,
    Id,
    Name,
    Status,
    Remark,
    CreateTime,
    Version,
}

#[derive(sqlx::FromRow, Debug, Serialize)]
struct BizActivityStruct {
    id: i32,
    name: String,
    status: i16,
    remark: String,
    create_time: NaiveDateTime,
    version: f64,
}

#[tokio::test]
async fn test_reldb_client() -> BIOSResult<()> {
    let docker = clients::Cli::default();
    let mysql_container = BIOSTestContainer::mysql_custom(Some("tests/sql/"), &docker);
    let port = mysql_container.get_host_port(3306).expect("Test port acquisition error");
    let url = format!("mysql://root:123456@localhost:{}/test", port);

    let client = BIOSRelDBMysqlClient::init(&url, 10).await?;

    // Test init script
    let result = client.fetch_all::<EmployeesStruct>(Query::select().columns(vec![Employees::Name]).from(Employees::Table).limit(1), None).await?;
    assert_eq!(result[0].name, "gudaoxuri");

    // DDL
    client
        .ddl_create_table(
            Table::create()
                .table(BizActivity::Table)
                .if_not_exists()
                .col(ColumnDef::new(BizActivity::Id).integer().not_null().auto_increment().primary_key())
                .col(ColumnDef::new(BizActivity::Name).not_null().string())
                .col(ColumnDef::new(BizActivity::Status).not_null().tiny_integer())
                .col(ColumnDef::new(BizActivity::Remark).text())
                .col(ColumnDef::new(BizActivity::CreateTime).date_time())
                .col(ColumnDef::new(BizActivity::Version).not_null().double()),
            None,
        )
        .await?;

    // Create
    let result = client
        .insert(
            Query::insert()
                .into_table(BizActivity::Table)
                .columns(vec![
                    BizActivity::Name,
                    BizActivity::Status,
                    BizActivity::Remark,
                    BizActivity::CreateTime,
                    BizActivity::Version,
                ])
                .values_panic(vec!["测试".into(), 1.into(), "".into(), Local::now().naive_local().into(), 1.0.into()]),
            None,
        )
        .await?;
    let id = result.last_insert_id();
    assert_eq!(id, 1);

    // Read
    let result = client
        .fetch_all::<BizActivityStruct>(
            Query::select()
                .columns(vec![
                    BizActivity::Id,
                    BizActivity::Name,
                    BizActivity::Status,
                    BizActivity::Remark,
                    BizActivity::CreateTime,
                    BizActivity::Version,
                ])
                .from(BizActivity::Table)
                .order_by(BizActivity::Id, Order::Desc)
                .limit(1),
            None,
        )
        .await?;
    assert_eq!(result[0].name, "测试");
    assert_eq!(result[0].version, 1.0);

    // Update
    let result = client
        .update(
            Query::update().table(BizActivity::Table).values(vec![(BizActivity::Status, 2.into())]).and_where(Expr::col(BizActivity::Id).eq(id)),
            None,
        )
        .await?;
    assert_eq!(result.rows_affected(), 1);

    // Read
    let result = client
        .fetch_one::<BizActivityStruct>(
            Query::select()
                .columns(vec![
                    BizActivity::Id,
                    BizActivity::Name,
                    BizActivity::Status,
                    BizActivity::Remark,
                    BizActivity::CreateTime,
                    BizActivity::Version,
                ])
                .from(BizActivity::Table)
                .order_by(BizActivity::Id, Order::Desc),
            None,
        )
        .await?;
    assert_eq!(result.status, 2);

    let result = client
        .fetch_one_json(
            Query::select()
                .columns(vec![
                    BizActivity::Id,
                    BizActivity::Name,
                    BizActivity::Status,
                    BizActivity::Remark,
                    BizActivity::CreateTime,
                    BizActivity::Version,
                ])
                .from(BizActivity::Table)
                .order_by(BizActivity::Id, Order::Desc),
            None,
        )
        .await?;
    assert_eq!(result["name"], "测试");
    assert_eq!(result["version"], 1.0);

    // Pagination
    let result = client
        .pagination::<BizActivityStruct>(
            Query::select()
                .columns(vec![
                    BizActivity::Id,
                    BizActivity::Name,
                    BizActivity::Status,
                    BizActivity::Remark,
                    BizActivity::CreateTime,
                    BizActivity::Version,
                ])
                .from(BizActivity::Table)
                .order_by(BizActivity::Id, Order::Desc),
            1,
            10,
            None,
        )
        .await?;
    assert_eq!(result.page_number, 1);
    assert_eq!(result.page_size, 10);
    assert_eq!(result.total_size, 1);
    assert_eq!(result.records[0].status, 2);

    let result = client
        .pagination_json(
            Query::select()
                .columns(vec![
                    BizActivity::Id,
                    BizActivity::Name,
                    BizActivity::Status,
                    BizActivity::Remark,
                    BizActivity::CreateTime,
                    BizActivity::Version,
                ])
                .from(BizActivity::Table)
                .order_by(BizActivity::Id, Order::Desc),
            1,
            10,
            None,
        )
        .await?;
    assert_eq!(result.page_number, 1);
    assert_eq!(result.page_size, 10);
    assert_eq!(result.total_size, 1);
    assert_eq!(result.records[0]["name"], "测试");
    assert_eq!(result.records[0]["status"], 2);

    // Count
    let result = client.count(Query::select().columns(vec![BizActivity::Id]).from(BizActivity::Table), None).await?;
    assert_eq!(result, 1);

    // Delete
    let result = client.delete(Query::delete().from_table(BizActivity::Table).and_where(Expr::col(BizActivity::Id).eq(id)), None).await?;
    assert_eq!(result.rows_affected(), 1);

    // Default test
    BIOSFuns::init_conf(BIOSConfig {
        ws: NoneConfig {},
        fw: FrameworkConfig {
            app: Default::default(),
            web: Default::default(),
            cache: CacheConfig {
                enabled: false,
                ..Default::default()
            },
            db: DBConfig {
                enabled: true,
                url,
                max_connections: 20,
            },
            mq: MQConfig {
                enabled: false,
                ..Default::default()
            },
            adv: Default::default(),
        },
    })
    .await?;

    let result = BIOSFuns::mysql().count(Query::select().columns(vec![BizActivity::Id]).from(BizActivity::Table), None).await?;
    assert_eq!(result, 0);

    Ok(())
}

#[tokio::test]
async fn test_reldb_client_with_tx() -> BIOSResult<()> {
    let docker = clients::Cli::default();
    let mysql_container = BIOSTestContainer::mysql_custom(Some("tests/sql/"), &docker);
    let port = mysql_container.get_host_port(3306).expect("Test port acquisition error");
    let url = format!("mysql://root:123456@localhost:{}/test", port);

    let client = BIOSRelDBMysqlClient::init(&url, 10).await?;

    let mut conn = client.conn().await?;
    let mut tx = conn.begin().await?;

    client
        .ddl_create_table(
            Table::create()
                .table(BizActivity::Table)
                .if_not_exists()
                .col(ColumnDef::new(BizActivity::Id).integer().not_null().auto_increment().primary_key())
                .col(ColumnDef::new(BizActivity::Name).not_null().string())
                .col(ColumnDef::new(BizActivity::Status).not_null().tiny_integer())
                .col(ColumnDef::new(BizActivity::Remark).text())
                .col(ColumnDef::new(BizActivity::CreateTime).date_time())
                .col(ColumnDef::new(BizActivity::Version).not_null().double()),
            None,
        )
        .await?;

    // Create
    let result = client
        .insert(
            Query::insert()
                .into_table(BizActivity::Table)
                .columns(vec![
                    BizActivity::Name,
                    BizActivity::Status,
                    BizActivity::Remark,
                    BizActivity::CreateTime,
                    BizActivity::Version,
                ])
                .values_panic(vec!["测试".into(), 1.into(), "".into(), Local::now().naive_local().into(), 1.0.into()]),
            Some(&mut tx),
        )
        .await?;
    let id = result.last_insert_id();
    assert_eq!(id, 1);

    // Rollback
    tx.rollback().await?;

    // Read, return empty
    let result = client
        .fetch_all::<BizActivityStruct>(
            Query::select()
                .columns(vec![
                    BizActivity::Id,
                    BizActivity::Name,
                    BizActivity::Status,
                    BizActivity::Remark,
                    BizActivity::CreateTime,
                    BizActivity::Version,
                ])
                .from(BizActivity::Table)
                .order_by(BizActivity::Id, Order::Desc)
                .limit(1),
            None,
        )
        .await?;
    assert_eq!(result.len(), 0);

    // Again

    let mut conn = client.conn().await?;
    let mut tx = conn.begin().await?;

    let result = client
        .insert(
            Query::insert()
                .into_table(BizActivity::Table)
                .columns(vec![
                    BizActivity::Name,
                    BizActivity::Status,
                    BizActivity::Remark,
                    BizActivity::CreateTime,
                    BizActivity::Version,
                ])
                .values_panic(vec!["测试".into(), 1.into(), "".into(), Local::now().naive_local().into(), 1.0.into()]),
            Some(&mut tx),
        )
        .await?;
    let id = result.last_insert_id();
    assert_eq!(id, 2);

    // Read in TX, return one record
    let result = client
        .fetch_all::<BizActivityStruct>(
            Query::select()
                .columns(vec![
                    BizActivity::Id,
                    BizActivity::Name,
                    BizActivity::Status,
                    BizActivity::Remark,
                    BizActivity::CreateTime,
                    BizActivity::Version,
                ])
                .from(BizActivity::Table)
                .order_by(BizActivity::Id, Order::Desc)
                .limit(1),
            Some(&mut tx),
        )
        .await?;
    assert_eq!(result[0].name, "测试");
    assert_eq!(result[0].version, 1.0);

    // Read NOT in TX, return empty
    let result = client
        .fetch_all::<BizActivityStruct>(
            Query::select()
                .columns(vec![
                    BizActivity::Id,
                    BizActivity::Name,
                    BizActivity::Status,
                    BizActivity::Remark,
                    BizActivity::CreateTime,
                    BizActivity::Version,
                ])
                .from(BizActivity::Table)
                .order_by(BizActivity::Id, Order::Desc)
                .limit(1),
            None,
        )
        .await?;
    assert_eq!(result.len(), 0);

    // Commit

    tx.commit().await?;

    // Read NOT in TX, return one record
    let result = client
        .fetch_all::<BizActivityStruct>(
            Query::select()
                .columns(vec![
                    BizActivity::Id,
                    BizActivity::Name,
                    BizActivity::Status,
                    BizActivity::Remark,
                    BizActivity::CreateTime,
                    BizActivity::Version,
                ])
                .from(BizActivity::Table)
                .order_by(BizActivity::Id, Order::Desc)
                .limit(1),
            None,
        )
        .await?;
    assert_eq!(result[0].name, "测试");
    assert_eq!(result[0].version, 1.0);

    // Soft Delete
    let mut conn = client.conn().await?;
    let mut tx = conn.begin().await?;

    let result = client
        .soft_del(
            BizActivity::Table,
            BizActivity::Id,
            "gdxr",
            Query::select().columns(BizActivity::iter().filter(|i| *i != BizActivity::Table)).from(BizActivity::Table).and_where(Expr::col(BizActivity::Id).eq(id)),
            &mut tx,
        )
        .await?;
    assert_eq!(result, true);

    let result = client.count(Query::select().columns(vec![BizActivity::Id]).from(BizActivity::Table), Some(&mut tx)).await?;
    assert_eq!(result, 0);

    let result = client.count(Query::select().columns(vec![BiosDelRecord::Id]).from(BiosDelRecord::Table), Some(&mut tx)).await?;
    assert_eq!(result, 1);

    tx.commit().await?;

    Ok(())
}

#[tokio::test]
async fn test_reldb_client_with_tx_by_postgres() -> BIOSResult<()> {
    let docker = clients::Cli::default();
    let postgres_container = BIOSTestContainer::postgres_custom(Some("tests/sql/"), &docker);
    let port = postgres_container.get_host_port(5432).expect("database system is ready to accept connections");
    let url = format!("postgres://postgres:123456@localhost:{}/test", port);

    let client = BIOSRelDBPostgresClient::init(&url, 10).await?;

    let mut conn = client.conn().await?;
    let mut tx = conn.begin().await?;

    client
        .ddl_create_table(
            Table::create()
                .table(BizActivity::Table)
                .if_not_exists()
                .col(ColumnDef::new(BizActivity::Id).integer().not_null().auto_increment().primary_key())
                .col(ColumnDef::new(BizActivity::Name).not_null().string())
                .col(ColumnDef::new(BizActivity::Status).not_null().small_integer())
                .col(ColumnDef::new(BizActivity::Remark).text())
                .col(ColumnDef::new(BizActivity::CreateTime).date_time())
                .col(ColumnDef::new(BizActivity::Version).not_null().double()),
            None,
        )
        .await?;

    // Create
    client
        .insert(
            Query::insert()
                .into_table(BizActivity::Table)
                .columns(vec![
                    BizActivity::Name,
                    BizActivity::Status,
                    BizActivity::Remark,
                    BizActivity::CreateTime,
                    BizActivity::Version,
                ])
                .values_panic(vec!["测试".into(), 1.into(), "".into(), Local::now().naive_local().into(), 1.0.into()]),
            Some(&mut tx),
        )
        .await?;

    // Rollback
    tx.rollback().await?;

    // Read, return empty
    let result = client
        .fetch_all::<BizActivityStruct>(
            Query::select()
                .columns(vec![
                    BizActivity::Id,
                    BizActivity::Name,
                    BizActivity::Status,
                    BizActivity::Remark,
                    BizActivity::CreateTime,
                    BizActivity::Version,
                ])
                .from(BizActivity::Table)
                .order_by(BizActivity::Id, Order::Desc)
                .limit(1),
            None,
        )
        .await?;
    assert_eq!(result.len(), 0);

    // Again

    let mut conn = client.conn().await?;
    let mut tx = conn.begin().await?;

    client
        .insert(
            Query::insert()
                .into_table(BizActivity::Table)
                .columns(vec![
                    BizActivity::Name,
                    BizActivity::Status,
                    BizActivity::Remark,
                    BizActivity::CreateTime,
                    BizActivity::Version,
                ])
                .values_panic(vec!["测试".into(), 1.into(), "".into(), Local::now().naive_local().into(), 1.0.into()]),
            Some(&mut tx),
        )
        .await?;

    // Read in TX, return one record
    let result = client
        .fetch_all::<BizActivityStruct>(
            Query::select()
                .columns(vec![
                    BizActivity::Id,
                    BizActivity::Name,
                    BizActivity::Status,
                    BizActivity::Remark,
                    BizActivity::CreateTime,
                    BizActivity::Version,
                ])
                .from(BizActivity::Table)
                .order_by(BizActivity::Id, Order::Desc)
                .limit(1),
            Some(&mut tx),
        )
        .await?;
    assert_eq!(result[0].name, "测试");
    assert_eq!(result[0].version, 1.0);

    // Read NOT in TX, return empty
    let result = client
        .fetch_all::<BizActivityStruct>(
            Query::select()
                .columns(vec![
                    BizActivity::Id,
                    BizActivity::Name,
                    BizActivity::Status,
                    BizActivity::Remark,
                    BizActivity::CreateTime,
                    BizActivity::Version,
                ])
                .from(BizActivity::Table)
                .order_by(BizActivity::Id, Order::Desc)
                .limit(1),
            None,
        )
        .await?;
    assert_eq!(result.len(), 0);

    // Commit

    tx.commit().await?;

    // Read NOT in TX, return one record
    let result = client
        .fetch_all::<BizActivityStruct>(
            Query::select()
                .columns(vec![
                    BizActivity::Id,
                    BizActivity::Name,
                    BizActivity::Status,
                    BizActivity::Remark,
                    BizActivity::CreateTime,
                    BizActivity::Version,
                ])
                .from(BizActivity::Table)
                .order_by(BizActivity::Id, Order::Desc)
                .limit(1),
            None,
        )
        .await?;
    assert_eq!(result[0].name, "测试");
    assert_eq!(result[0].version, 1.0);

    // Soft Delete
    let mut conn = client.conn().await?;
    let mut tx = conn.begin().await?;

    let result = client
        .soft_del(
            BizActivity::Table,
            BizActivity::Id,
            "gdxr",
            Query::select().columns(BizActivity::iter().filter(|i| *i != BizActivity::Table)).from(BizActivity::Table).and_where(Expr::col(BizActivity::Id).eq(result[0].id)),
            &mut tx,
        )
        .await?;
    assert_eq!(result, true);

    let result = client.count(Query::select().columns(vec![BizActivity::Id]).from(BizActivity::Table), Some(&mut tx)).await?;
    assert_eq!(result, 0);

    let result = client.count(Query::select().columns(vec![BiosDelRecord::Id]).from(BiosDelRecord::Table), Some(&mut tx)).await?;
    assert_eq!(result, 1);

    tx.commit().await?;

    Ok(())
}
