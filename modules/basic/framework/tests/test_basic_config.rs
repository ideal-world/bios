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

// https://github.com/mehcode/config-rs

use std::env;

use serde::{Deserialize, Serialize};

use bios::basic::config::NoneConfig;
use bios::basic::result::BIOSResult;
use bios::BIOSFuns;

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
struct TestConfig {
    project_name: String,
    level_num: u8,
    db_proj: DatabaseConfig,
}

impl Default for TestConfig {
    fn default() -> Self {
        TestConfig {
            project_name: "".to_string(),
            level_num: 0,
            db_proj: DatabaseConfig::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
struct DatabaseConfig {
    url: String,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        DatabaseConfig { url: "".to_string() }
    }
}

#[tokio::test]
async fn test_basic_config() -> BIOSResult<()> {
    BIOSFuns::init::<NoneConfig>("").await?;
    assert_eq!(BIOSFuns::fw_config().db.url, "");
    assert_eq!(BIOSFuns::fw_config().db.enabled, true);
    assert_eq!(BIOSFuns::fw_config().app.name, "BIOS Application");

    BIOSFuns::init::<TestConfig>("tests/config").await?;
    assert_eq!(BIOSFuns::ws_config::<TestConfig>().project_name, "测试");
    assert_eq!(BIOSFuns::fw_config().db.url, "postgres://postgres@test");
    assert_eq!(BIOSFuns::ws_config::<TestConfig>().db_proj.url, "postgres://postgres@test.proj");
    assert_eq!(BIOSFuns::fw_config().app.name, "APP1");

    env::set_var("PROFILE", "prod");

    BIOSFuns::init::<TestConfig>("tests/config").await?;
    assert_eq!(BIOSFuns::fw_config().db.url, "postgres://postgres@prod");
    assert_eq!(BIOSFuns::ws_config::<TestConfig>().db_proj.url, "postgres://postgres@prod.proj");
    assert_eq!(BIOSFuns::fw_config().app.name, "BIOS Application");

    // cli example: env BIOS_DB.URL=test BIOS_app.name=xx ./xxx
    env::set_var("BIOS_DB.URL", "test");
    BIOSFuns::init::<TestConfig>("tests/config").await?;
    assert_eq!(BIOSFuns::fw_config().db.url, "test");
    assert_eq!(BIOSFuns::fw_config().app.name, "BIOS Application");

    /*let mut  config=BIOSConfig{
        ws: BIOSFuns::ws_config::<TestConfig>(),
        fw: BIOSFuns::fw_config(),
    };

    BIOSFuns::fw_config().db = "";
    config.fw.cache.enabled = false;
    config.fw.db.enabled = false;
    config.fw.mq.enabled = false;
    BIOSFuns::init(config).await?;

    assert_eq!(BIOSFuns::ws_config::<TestConfig>().db_proj.url, "postgres://postgres@prod.proj");
    assert_eq!(BIOSFuns::fw_config().db.url, "test");*/

    Ok(())
}
