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

use bios_framework::basic::config::{BIOSConfig, NoneConfig};
use bios_framework::basic::error::BIOSResult;
use bios_framework::basic::logger::BIOSLogger;
use bios_framework::web::web_client::BIOSWebClient;
use bios_framework::BIOSFuns;

#[actix_rt::test]
async fn test_web_client() -> BIOSResult<()> {
    BIOSLogger::init("")?;
    let client = BIOSWebClient::init(60, 60)?;
    let client = client.raw();
    let response = client
        .get("https://www.baidu.com")
        .insert_header(("User-Agent", "Actix-web"))
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    let mut response = client
        .post("http://httpbin.org/post")
        .send_body("Raw body contents")
        .await?;

    assert!(BIOSWebClient::body_as_str(&mut response)
        .await?
        .contains(r#"data": "Raw body contents"#));

    let request = serde_json::json!({
        "lang": "rust",
        "body": "json"
    });
    let mut response = client
        .post("http://httpbin.org/post")
        .send_json(&request)
        .await?;
    assert!(BIOSWebClient::body_as_str(&mut response)
        .await?
        .contains(r#"data": "{\"body\":\"json\",\"lang\":\"rust\"}"#));

    // Default test
    BIOSFuns::init(BIOSConfig {
        ws: NoneConfig {},
        fw: Default::default(),
    })
    .await?;

    let mut response = BIOSFuns::web_client()
        .raw()
        .post("http://httpbin.org/post")
        .send_json(&request)
        .await?;
    assert!(BIOSWebClient::body_as_str(&mut response)
        .await?
        .contains(r#"data": "{\"body\":\"json\",\"lang\":\"rust\"}"#));

    Ok(())
}
