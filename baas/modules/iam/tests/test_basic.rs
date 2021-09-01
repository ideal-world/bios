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

use testcontainers::clients::Cli;
use testcontainers::Container;
use testcontainers::images::generic::GenericImage;
use testcontainers::images::redis::Redis;

use bios::basic::config::{BIOSConfig, CacheConfig, DBConfig, FrameworkConfig};
use bios::basic::logger::BIOSLogger;
use bios::BIOSFuns;
use bios::test::test_container::BIOSTestContainer;
use bios_baas_iam::iam_config::WorkSpaceConfig;

pub async fn init<'a>(
    docker: &'a Cli,
) -> (Container<'a, Cli, GenericImage>, Container<'a, Cli, Redis>) {
    BIOSLogger::init("").unwrap();
    let mysql_container = BIOSTestContainer::mysql_custom(Some("sql/"), &docker);
    let redis_container = BIOSTestContainer::redis_custom(&docker);
    BIOSFuns::init(BIOSConfig {
        ws: WorkSpaceConfig::default(),
        fw: FrameworkConfig {
            app: Default::default(),
            web: Default::default(),
            cache: CacheConfig {
                url: format!(
                    "redis://127.0.0.1:{}/0",
                    redis_container
                        .get_host_port(6379)
                        .expect("Test port acquisition error")
                ),
            },
            db: DBConfig {
                url: format!(
                    "mysql://root:123456@localhost:{}/iam",
                    mysql_container
                        .get_host_port(3306)
                        .expect("Test port acquisition error")
                ),
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
