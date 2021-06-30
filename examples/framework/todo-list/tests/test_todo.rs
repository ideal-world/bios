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

// https://github.com/rambler-digital-solutions/actix-web-validator
// https://github.com/Keats/validator

use serde_json::Value;

use bios_framework::basic::error::BIOSResult;
use bios_framework::web::web_client::BIOSWebClient;

#[actix_rt::test]
async fn test_web_server() -> BIOSResult<()> {
    let url = "http://127.0.0.1:8080/";
    let client = BIOSWebClient::init(60, 60);
    let response = serde_json::from_str::<Value>(
        &BIOSWebClient::body_as_str(&mut client.get(url + "categories").send().await?).await?,
    );
    // assert_eq!(response., StatusCode::OK);

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

    Ok(())
}
