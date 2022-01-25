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

// https://github.com/seanmonstar/reqwest

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use bios::basic::config::{BIOSConfig, CacheConfig, DBConfig, FrameworkConfig, MQConfig, NoneConfig};
use bios::basic::result::BIOSResult;
use bios::BIOSFuns;

#[tokio::test]
async fn test_web_client() -> BIOSResult<()> {
    BIOSFuns::init_conf(BIOSConfig {
        ws: NoneConfig {},
        fw: FrameworkConfig {
            app: Default::default(),
            web_server: Default::default(),
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

    let response = BIOSFuns::web_client()
        .get_to_str(
            "https://www.baidu.com",
            Some([("User-Agent".to_string(), "Actix-web".to_string())].iter().cloned().collect()),
        )
        .await?;
    assert_eq!(response.code, StatusCode::OK.as_u16());
    assert!(response.body.unwrap().contains("baidu"));

    let response = BIOSFuns::web_client().get_to_str("https://httpbin.org/get", Some([("User-Agent".to_string(), "BIOS".to_string())].iter().cloned().collect())).await?;
    assert_eq!(response.code, StatusCode::OK.as_u16());
    assert!(response.body.unwrap().contains("BIOS"));

    let response = BIOSFuns::web_client()
        .delete(
            "https://httpbin.org/delete",
            Some([("User-Agent".to_string(), "BIOS".to_string())].iter().cloned().collect()),
        )
        .await?;
    assert_eq!(response.code, StatusCode::OK.as_u16());

    let response = BIOSFuns::web_client().post_str_to_str("https://httpbin.org/post", &"Raw body contents".to_string(), None).await?;
    assert_eq!(response.code, StatusCode::OK.as_u16());
    assert!(response.body.unwrap().contains(r#"data": "Raw body contents"#));

    let response = BIOSFuns::web_client().post_str_to_str("https://httpbin.org/post", &"Raw body contents".to_string(), None).await?;
    assert_eq!(response.code, StatusCode::OK.as_u16());
    assert!(response.body.unwrap().contains(r#"data": "Raw body contents"#));

    let request = serde_json::json!({
        "lang": "rust",
        "body": "json"
    });
    let response = BIOSFuns::web_client().post_obj_to_str("https://httpbin.org/post", &request, None).await?;
    assert_eq!(response.code, StatusCode::OK.as_u16());
    assert!(response.body.unwrap().contains(r#"data": "{\"body\":\"json\",\"lang\":\"rust\"}"#));

    let new_post = Post {
        id: None,
        title: "Reqwest.rs".into(),
        body: "https://docs.rs/reqwest".into(),
        user_id: 1,
    };
    let response = BIOSFuns::web_client().post::<Post, Post>("https://jsonplaceholder.typicode.com/posts", &new_post, None).await?;
    assert_eq!(response.code, StatusCode::CREATED.as_u16());
    assert_eq!(response.body.unwrap().body, "https://docs.rs/reqwest");

    let response = BIOSFuns::web_client().post_obj_to_str("https://jsonplaceholder.typicode.com/posts", &new_post, None).await?;
    assert_eq!(response.code, StatusCode::CREATED.as_u16());
    assert!(response.body.unwrap().contains("https://docs.rs/reqwest"));

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct Post {
    id: Option<i32>,
    title: String,
    body: String,
    #[serde(rename = "userId")]
    user_id: i32,
}
