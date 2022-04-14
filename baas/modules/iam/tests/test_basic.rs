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

use sea_query::Query;
use testcontainers::clients::Cli;
use testcontainers::images::generic::GenericImage;
use testcontainers::images::redis::Redis;
use testcontainers::Container;

use bios::basic::config::{BIOSConfig, CacheConfig, DBConfig, FrameworkConfig};
use bios::basic::logger::BIOSLogger;
use bios::db::reldb_client::SqlBuilderProcess;
use bios::test::test_container::BIOSTestContainer;
use bios::BIOSFuns;
use bios_baas_iam::domain::ident_domain::{IamAccount, IamAccountApp, IamApp, IamTenant};
use bios_baas_iam::iam_config::WorkSpaceConfig;

pub fn context_account() -> (&'static str, String) {
    (
        &BIOSFuns::fw_config().web.context_flag,
        bios::basic::security::digest::base64::encode(
            r#"{"trace":{"id":"111111"},"ident":{"app_id":"app1","tenant_id":"tenant1","account_id":"admin001","ak":"ak1","token":"t01"}}"#,
        ),
    )
}

pub fn context_pub() -> (&'static str, String) {
    (
        &BIOSFuns::fw_config().web.context_flag,
        bios::basic::security::digest::base64::encode(r#"{"trace":{"id":"111111"},"ident":{}}"#),
    )
}

pub async fn init<'a>(docker: &'a Cli) -> (Container<'a, Cli, GenericImage>, Container<'a, Cli, Redis>) {
    BIOSLogger::init("").unwrap();
    let mysql_container = BIOSTestContainer::mysql_custom(Some("sql/"), &docker);
    let redis_container = BIOSTestContainer::redis_custom(&docker);
    BIOSFuns::init(BIOSConfig {
        ws: WorkSpaceConfig::default(),
        fw: FrameworkConfig {
            app: Default::default(),
            web: Default::default(),
            cache: CacheConfig {
                enabled: true,
                url: format!("redis://127.0.0.1:{}/0", redis_container.get_host_port(6379)),
            },
            db: DBConfig {
                enabled: true,
                url: format!("mysql://root:123456@localhost:{}/iam", mysql_container.get_host_port(3306)),
                max_connections: 20,
            },
            mq: Default::default(),
            adv: Default::default(),
        },
    })
    .await
    .unwrap();

    // Init Tenant
    let sql_builder = Query::insert()
        .into_table(IamTenant::Table)
        .columns(vec![
            IamTenant::Id,
            IamTenant::CreateUser,
            IamTenant::UpdateUser,
            IamTenant::Name,
            IamTenant::Icon,
            IamTenant::Parameters,
            IamTenant::AllowAccountRegister,
            IamTenant::Status,
        ])
        .values_panic(vec![
            "tenant1".into(),
            "admin001".into(),
            "admin001".into(),
            "理想世界".into(),
            "".into(),
            "".into(),
            true.into(),
            "enabled".into(),
        ])
        .done();
    BIOSFuns::reldb().exec(&sql_builder, None).await.unwrap();
    // Init App
    let sql_builder = Query::insert()
        .into_table(IamApp::Table)
        .columns(vec![
            IamApp::Id,
            IamApp::CreateUser,
            IamApp::UpdateUser,
            IamApp::Name,
            IamApp::Icon,
            IamApp::Parameters,
            IamApp::RelTenantId,
            IamApp::Status,
        ])
        .values_panic(vec![
            "app1".into(),
            "admin001".into(),
            "admin001".into(),
            "IAM".into(),
            "".into(),
            "".into(),
            "tenant1".into(),
            "enabled".into(),
        ])
        .done();
    BIOSFuns::reldb().exec(&sql_builder, None).await.unwrap();
    // Init Account
    let sql_builder = Query::insert()
        .into_table(IamAccount::Table)
        .columns(vec![
            IamAccount::Id,
            IamAccount::CreateUser,
            IamAccount::UpdateUser,
            IamAccount::OpenId,
            IamAccount::Name,
            IamAccount::Avatar,
            IamAccount::Parameters,
            IamAccount::ParentId,
            IamAccount::RelTenantId,
            IamAccount::Status,
        ])
        .values_panic(vec![
            "admin001".into(),
            "admin001".into(),
            "admin001".into(),
            "open_id_xx".into(),
            "平台管理员".into(),
            "".into(),
            "".into(),
            "".into(),
            "".into(),
            "enabled".into(),
        ])
        .done();
    BIOSFuns::reldb().exec(&sql_builder, None).await.unwrap();
    // Init AccountApp
    let sql_builder = Query::insert()
        .into_table(IamAccountApp::Table)
        .columns(vec![
            IamAccountApp::Id,
            IamAccountApp::CreateUser,
            IamAccountApp::UpdateUser,
            IamAccountApp::RelAppId,
            IamAccountApp::RelAccountId,
        ])
        .values_panic(vec!["admin001".into(), "admin001".into(), "admin001".into(), "app1".into(), "admin001".into()])
        .done();
    BIOSFuns::reldb().exec(&sql_builder, None).await.unwrap();

    (mysql_container, redis_container)
}

pub async fn init_without_data<'a>(docker: &'a Cli) -> (Container<'a, Cli, GenericImage>, Container<'a, Cli, Redis>) {
    BIOSLogger::init("").unwrap();
    let mysql_container = BIOSTestContainer::mysql_custom(Some("sql/"), &docker);
    let redis_container = BIOSTestContainer::redis_custom(&docker);
    BIOSFuns::init(BIOSConfig {
        ws: WorkSpaceConfig::default(),
        fw: FrameworkConfig {
            app: Default::default(),
            web: Default::default(),
            cache: CacheConfig {
                enabled: true,
                url: format!("redis://127.0.0.1:{}/0", redis_container.get_host_port(6379)),
            },
            db: DBConfig {
                enabled: true,
                url: format!("mysql://root:123456@localhost:{}/iam", mysql_container.get_host_port(3306)),
                max_connections: 20,
            },
            mq: Default::default(),
            adv: Default::default(),
        },
    })
    .await
    .unwrap();
    (mysql_container, redis_container)
}
