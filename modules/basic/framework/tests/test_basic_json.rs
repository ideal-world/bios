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
use serde::{Deserialize, Serialize};

use bios::basic::result::BIOSResult;
use bios::BIOSFuns;

#[derive(Serialize, Deserialize)]
#[serde(default)]
struct TestConfig<T> {
    project_name: String,
    level_num: u8,
    db_proj: T,
}

impl<T: Default> Default for TestConfig<T> {
    fn default() -> Self {
        TestConfig {
            project_name: "".to_owned(),
            level_num: 0,
            db_proj: Default::default(),
        }
    }
}

#[derive(Serialize, Deserialize)]
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
async fn test_basic_json() -> BIOSResult<()> {
    let test_config = TestConfig {
        project_name: "测试".to_string(),
        level_num: 0,
        db_proj: DatabaseConfig { url: "http://xxx".to_string() },
    };

    let json_str = BIOSFuns::json.obj_to_string(&test_config).unwrap();
    assert_eq!(json_str, r#"{"project_name":"测试","level_num":0,"db_proj":{"url":"http://xxx"}}"#);

    let json_obj = BIOSFuns::json.str_to_obj::<TestConfig<DatabaseConfig>>(&json_str).unwrap();
    assert_eq!(json_obj.project_name, "测试");
    assert_eq!(json_obj.level_num, 0);
    assert_eq!(json_obj.db_proj.url, "http://xxx");

    let json_value = BIOSFuns::json.str_to_json(&json_str).unwrap();
    assert_eq!(json_value["project_name"], "测试");
    assert_eq!(json_value["level_num"], 0);
    assert_eq!(json_value["db_proj"]["url"], "http://xxx");

    let json_value = BIOSFuns::json.obj_to_json(&json_obj).unwrap();
    assert_eq!(json_value["project_name"], "测试");
    assert_eq!(json_value["level_num"], 0);
    assert_eq!(json_value["db_proj"]["url"], "http://xxx");

    let json_obj = BIOSFuns::json.json_to_obj::<TestConfig<DatabaseConfig>>(json_value).unwrap();
    assert_eq!(json_obj.project_name, "测试");
    assert_eq!(json_obj.level_num, 0);
    assert_eq!(json_obj.db_proj.url, "http://xxx");

    Ok(())
}
