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

// https://github.com/poem-web/poem

extern crate core;

use poem_openapi::{param::Path, payload::Json, Object, OpenApi, Tags};
use serde::{Deserialize, Serialize};

use bios::basic::config::{BIOSConfig, CacheConfig, DBConfig, FrameworkConfig, MQConfig, NoneConfig, WebServerConfig, WebServerModuleConfig};
use bios::basic::error::BIOSError;
use bios::basic::result::BIOSResult;
use bios::web::web_resp::BIOSResp;
use bios::BIOSFuns;

#[tokio::test]
async fn test_web_server() -> BIOSResult<()> {
    BIOSFuns::init_conf(BIOSConfig {
        ws: NoneConfig {},
        fw: FrameworkConfig {
            app: Default::default(),
            web_server: WebServerConfig {
                enabled: true,
                modules: vec![
                    WebServerModuleConfig {
                        code: "todo".to_string(),
                        title: "todo app".to_string(),
                        doc_urls: [
                            ("test env".to_string(), "http://localhost:8080".to_string()),
                            ("prod env".to_string(), "http://127.0.0.1:8080".to_string()),
                        ]
                        .iter()
                        .cloned()
                        .collect(),
                        ..Default::default()
                    },
                    WebServerModuleConfig {
                        code: "other".to_string(),
                        title: "other app".to_string(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            web_client: Default::default(),
            cache: CacheConfig {
                enabled: false,
                ..Default::default()
            },
            db: DBConfig {
                enabled: false,
                ..Default::default()
            },
            mq: MQConfig {
                enabled: false,
                ..Default::default()
            },
            adv: Default::default(),
        },
    })
    .await?;

    BIOSFuns::web_server().add_module("todo", (TodosApi)).add_module("other", OtherApi).start().await?;

    Ok(())
}

#[derive(Tags)]
enum FunTags {
    #[oai(rename = "Todo1测试")]
    Todo1,
    #[oai(rename = "Todo2测试")]
    Todo2,
}

#[derive(Object, Serialize, Deserialize)]
struct TodoResp {
    id: i64,
    description: String,
    done: bool,
}

#[derive(Object, Serialize, Deserialize)]
struct TodoAddReq {
    description: String,
    done: bool,
}

#[derive(Object, Serialize, Deserialize)]
struct TodoModifyReq {
    description: Option<String>,
    done: Option<bool>,
}

#[derive(Object, Serialize, Deserialize)]
struct ValidateReq {
    #[oai(validator(min_length = "1", max_length = "10"))]
    len: String,
    #[oai(validator(min_length = "5", max_length = "5"))]
    eq: String,
    #[oai(validator(minimum(value = "1", exclusive = "false"), maximum(value = "500", exclusive)))]
    range: u8,
    #[oai(validator(pattern = r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$"))]
    mail: String,
    #[oai(validator(pattern = r".*gmail.*"))]
    contain: String,
    #[oai(validator(pattern = r"^1(3\d|4[5-9]|5[0-35-9]|6[2567]|7[0-8]|8\d|9[0-35-9])\d{8}$"))]
    phone: String,
    #[oai(validator(min_items = "1", max_items = "3"))]
    item_len: Vec<String>,
    #[oai(validator(unique_items))]
    item_unique: Vec<String>,
}

struct TodosApi;

#[OpenApi(tag = "FunTags::Todo1")]
impl TodosApi {
    #[oai(path = "/todos", method = "post")]
    async fn create(&self, todo_add_req: Json<TodoAddReq>) -> BIOSResp<String> {
        BIOSResp::ok("0".into())
    }

    #[oai(path = "/todos/:id", method = "get")]
    async fn get(&self, id: Path<i64>) -> BIOSResp<TodoResp> {
        BIOSResp::ok(TodoResp {
            id: id.0,
            description: "sss".to_string(),
            done: false,
        })
    }

    #[oai(path = "/todos/:id/err", method = "get")]
    async fn get_by_error(&self, id: Path<i64>) -> BIOSResp<TodoResp> {
        BIOSResp::err(BIOSError::Conflict("ssssssss".to_string()))
    }
}

struct OtherApi;

#[OpenApi]
impl OtherApi {
    #[oai(path = "/validate", method = "post")]
    async fn test(&self, _req: Json<ValidateReq>) -> BIOSResp<String> {
        BIOSResp::ok("".into())
    }
}
