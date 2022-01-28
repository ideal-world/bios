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

use std::env;

use bios::basic::config::NoneConfig;
use log::info;
use sea_orm::entity::*;
use sea_orm::Set;
use testcontainers::clients;

use bios::basic::result::BIOSResult;
use bios::test::test_container::BIOSTestContainer;
use bios::BIOSFuns;

mod domain;

#[tokio::main]
async fn main() -> BIOSResult<()> {
    // Here is a demonstration of using docker to start a mysql simulation scenario.
    let docker = clients::Cli::default();
    let mysql_container = BIOSTestContainer::mysql_custom(None, &docker);
    let port = mysql_container.get_host_port(3306).expect("Test port acquisition error");
    let url = format!("mysql://root:123456@localhost:{}/test", port);
    env::set_var("BIOS_DB.URL", url);

    env::set_var("RUST_LOG", "debug");
    env::set_var("PROFILE", "default");

    // Initial configuration
    BIOSFuns::init::<NoneConfig>("config").await?;

    let client = BIOSFuns::reldb();

    // --------------------------------------------------

    // Create table
    client.create_table_from_entity(domain::tenant::Entity).await?;
    client.create_table_from_entity(domain::tenant_conf::Entity).await?;
    client.create_table_from_entity(domain::app::Entity).await?;
    client.create_table_from_entity(domain::account::Entity).await?;
    client.create_table_from_entity(domain::app_account_rel::Entity).await?;

    // Insert some records
    domain::tenant::ActiveModel {
        name: Set("tenant1".to_string()),
        ..Default::default()
    }
    .insert(client.conn())
    .await?;

    let tenant = domain::tenant::Entity::find().one(client.conn()).await?.unwrap();

    domain::tenant_conf::ActiveModel {
        name: Set("conf1".to_string()),
        tenant_id: Set(tenant.id.clone()),
        ..Default::default()
    }
    .insert(client.conn())
    .await?;

    domain::app::ActiveModel {
        name: Set("app1".to_string()),
        tenant_id: Set(tenant.id.clone()),
        ..Default::default()
    }
    .insert(client.conn())
    .await?;

    domain::app::ActiveModel {
        name: Set("app2".to_string()),
        tenant_id: Set(tenant.id.clone()),
        ..Default::default()
    }
    .insert(client.conn())
    .await?;

    let tenant = domain::tenant::Entity::find_by_id(tenant.id.clone()).one(client.conn()).await?.unwrap();

    info!("----------------- One To One -----------------");
    let config = tenant.find_related(domain::tenant_conf::Entity).one(client.conn()).await?.unwrap();
    assert_eq!(config.name, "conf1");
    let tenant = config.find_related(domain::tenant::Entity).one(client.conn()).await?.unwrap();
    assert_eq!(tenant.name, "tenant1");

    info!("----------------- One To Many -----------------");
    let apps = tenant.find_related(domain::app::Entity).all(client.conn()).await?;
    assert_eq!(apps.len(), 2);
    info!("----------------- Many To One -----------------");
    let tenant = apps[0].find_related(domain::tenant::Entity).one(client.conn()).await?.unwrap();
    assert_eq!(tenant.name, "tenant1");

    info!("----------------- Many To Many -----------------");
    let accounts = apps[0].find_related(domain::account::Entity).all(client.conn()).await?;
    assert_eq!(accounts.len(), 0);

    let account = domain::account::ActiveModel {
        name: Set("account1".to_string()),
        ..Default::default()
    }
    .insert(client.conn())
    .await?;
    domain::app_account_rel::ActiveModel {
        app_id: Set(apps[0].id.to_string()),
        account_id: Set(account.id.to_string()),
    }
    .insert(client.conn())
    .await?;

    let accounts = apps[0].find_related(domain::account::Entity).all(client.conn()).await?;
    assert_eq!(accounts.len(), 1);

    Ok(())
}
