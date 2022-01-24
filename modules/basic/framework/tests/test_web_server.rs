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

use poem_openapi::{param::Path, payload::Json, Object, OpenApi, Tags};

use bios::basic::config::{BIOSConfig, CacheConfig, DBConfig, FrameworkConfig, MQConfig, NoneConfig, WebServerConfig, WebServerModuleConfig};
use bios::basic::result::BIOSResult;
use bios::BIOSFuns;

#[tokio::test]
async fn test_web_server() -> BIOSResult<()> {
    BIOSFuns::init_log_from_path("")?;

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
                            ("test env".to_string(), "http://localhost:8080/".to_string()),
                            ("prod env".to_string(), "http://127.0.0.1:8080/".to_string()),
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

#[derive(Object)]
struct TodoResp {
    id: i64,
    description: String,
    done: bool,
}

#[derive(Object)]
struct TodoAddReq {
    description: String,
    done: bool,
}

#[derive(Object)]
struct TodoModifyReq {
    description: Option<String>,
    done: Option<bool>,
}

struct TodosApi;

#[OpenApi(tag = "FunTags::Todo1")]
impl TodosApi {
    #[oai(path = "/todos", method = "post")]
    async fn create(&self, todo_add_req: Json<TodoAddReq>) -> poem::Result<Json<i64>> {
        Ok(Json(0))
    }

    #[oai(path = "/todos/:id", method = "get")]
    async fn get(&self, id: Path<i64>) -> poem::Result<Json<TodoResp>> {
        Ok(Json(TodoResp {
            id: id.0,
            description: "sss".to_string(),
            done: false,
        }))
    }
}

struct OtherApi;

#[OpenApi]
impl OtherApi {
    #[oai(path = "/a", method = "get")]
    async fn test(&self) {}
}
