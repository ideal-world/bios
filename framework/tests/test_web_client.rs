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

// https://docs.rs/awc

use awc::http::StatusCode;

use bios::basic::config::{BIOSConfig, CacheConfig, DBConfig, FrameworkConfig, MQConfig, NoneConfig};
use bios::basic::result::BIOSResult;
use bios::web::web_client::BIOSWebClient;
use bios::BIOSFuns;

#[actix_rt::test]
async fn test_web_client() -> BIOSResult<()> {
    BIOSFuns::init_log_from_path("")?;
    let client = BIOSWebClient::init(60, 60)?;
    let client = client.raw();
    let response = client.get("https://www.baidu.com").insert_header(("User-Agent", "Actix-web")).send().await?;
    assert_eq!(response.status(), StatusCode::OK);

    let mut response = client.post("http://httpbin.org/post").send_body("Raw body contents").await?;

    assert!(BIOSWebClient::body_as_str(&mut response).await?.contains(r#"data": "Raw body contents"#));

    let request = serde_json::json!({
        "lang": "rust",
        "body": "json"
    });
    let mut response = client.post("http://httpbin.org/post").send_json(&request).await?;
    assert!(BIOSWebClient::body_as_str(&mut response).await?.contains(r#"data": "{\"body\":\"json\",\"lang\":\"rust\"}"#));

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

    let mut response = BIOSFuns::web_client().raw().post("http://httpbin.org/post").send_json(&request).await?;
    assert!(BIOSWebClient::body_as_str(&mut response).await?.contains(r#"data": "{\"body\":\"json\",\"lang\":\"rust\"}"#));

    Ok(())
}
