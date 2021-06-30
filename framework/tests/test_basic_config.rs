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

// https://github.com/mehcode/config-rs

use std::env;

use serde::{Deserialize, Serialize};

use bios_framework::basic::config::BIOSConfig;
use bios_framework::basic::error::BIOSResult;
use bios_framework::basic::logger::BIOSLogger;

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
            project_name: "".to_owned(),
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
        DatabaseConfig { url: "".to_owned() }
    }
}

#[tokio::test]
async fn test_basic_config() -> BIOSResult<()> {
    BIOSLogger::init("")?;
    let mut config = BIOSConfig::<TestConfig>::init("tests/config")?;
    assert_eq!(config.ws.project_name, "测试");
    assert_eq!(config.fw.db.url, "postgres://postgres@test");
    assert_eq!(config.ws.db_proj.url, "postgres://postgres@test.proj");
    assert_eq!(config.fw.app.name, "APP1");

    env::set_var("PROFILE", "prod");

    config = BIOSConfig::<TestConfig>::init("tests/config")?;
    assert_eq!(config.fw.db.url, "postgres://postgres@prod");
    assert_eq!(config.ws.db_proj.url, "postgres://postgres@prod.proj");
    assert_eq!(config.fw.app.name, "BIOS Application");

    // cli example: env BIOS_DB.URL=test BIOS_app.name=xx ./xxx
    env::set_var("BIOS_DB.URL", "test");
    config = BIOSConfig::<TestConfig>::init("tests/config")?;
    assert_eq!(config.fw.db.url, "test");
    assert_eq!(config.fw.app.name, "BIOS Application");

    Ok(())
}
