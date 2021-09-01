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

use actix_http::http::StatusCode;
use actix_web::post;
use actix_web::test::{call_service, read_body};
use actix_web::web::Bytes;
use actix_web::{test, App};
use serde::{Deserialize, Serialize};
use validator::Validate;

use bios::basic::config::FrameworkConfig;
use bios::basic::error::{BIOSError, BIOSResult};
use bios::basic::logger::BIOSLogger;
use bios::web::resp_handler::{BIOSResp, BIOSRespHelper};
use bios::web::validate::json::Json;
use bios::web::validate::query::Query;
use bios::web::web_server::BIOSWebServer;

mod basic;

#[actix_rt::test]
async fn test_web_server() -> BIOSResult<()> {
    BIOSLogger::init("")?;
    let app = test::init_service(
        App::new()
            //.wrap(BIOSWebServer::init_logger())
            .wrap(BIOSWebServer::init_cors(&FrameworkConfig::default()))
            .wrap(BIOSWebServer::init_error_handlers())
            .service(normal)
            .service(bus_error)
            .service(sys_error)
            .service(validation),
    )
    .await;

    // Normal
    let req = test::TestRequest::post().uri("/normal/11").to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(read_body(resp).await, Bytes::from(r#"{"code":"200","msg":"","body":"successful"}"#));

    // Business Error
    let req = test::TestRequest::post().uri("/bus_error").to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(read_body(resp).await, Bytes::from(r#"{"code":"xxx01","msg":"business error","body":null}"#),);

    // Not Found
    let req = test::TestRequest::post().uri("/not_found").to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    assert_eq!(read_body(resp).await, Bytes::from(r#"{"body":null,"code":"404","msg":"method:POST, url:/not_found"}"#),);

    // System Error
    let req = test::TestRequest::post().uri("/sys_error").to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(read_body(resp).await, Bytes::from(r#"{"body":null,"code":"500","msg":"没事，莫慌"}"#),);

    // Validation
    let req = test::TestRequest::post().uri("/validation").to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        read_body(resp).await,
        Bytes::from(r#"{"body":null,"code":"400","msg":"Query deserialize error: missing field `id`"}"#),
    );

    let req = test::TestRequest::post().uri("/validation?id=111").to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(
        read_body(resp).await,
        Bytes::from(r#"{"body":null,"code":"400","msg":"Query deserialize error: missing field `response_type`"}"#),
    );

    let req = test::TestRequest::post().uri("/validation?id=-1").to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(
        read_body(resp).await,
        Bytes::from(r#"{"body":null,"code":"400","msg":"Query deserialize error: invalid digit found in string"}"#),
    );

    let req = test::TestRequest::post().uri("/validation?id=111&response_type=XX").to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(
        read_body(resp).await,
        Bytes::from(r#"{"body":null,"code":"400","msg":"Query deserialize error: unknown variant `XX`, expected `Token` or `Code`"}"#),
    );

    let req = test::TestRequest::post()
        .uri("/validation?id=1001&response_type=Code")
        .set_json(&ItemBody {
            req: Some("req".to_owned()),
            len: "len".to_owned(),
            eq: "1234567890".to_owned(),
            range: 19,
            url: "http://idealworld.group".to_owned(),
            mail: "i@sunisle.org".to_owned(),
            phone: "18657120202".to_owned(),
            cont: "ddd@gmail.com".to_owned(),
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(read_body(resp).await, Bytes::from(r#"{"code":"200","msg":"","body":"successful"}"#),);

    let req = test::TestRequest::post()
        .uri("/validation?id=100&response_type=Code")
        .set_json(&ItemBody {
            req: Some("req".to_owned()),
            len: "len".to_owned(),
            eq: "1234567890".to_owned(),
            range: 19,
            url: "http://idealworld.group".to_owned(),
            mail: "i@sunisle.org".to_owned(),
            phone: "18657120202".to_owned(),
            cont: "ddd@gmail.com".to_owned(),
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert!(String::from_utf8(read_body(resp).await.to_vec()).unwrap().contains("ValidationErrors({\\\"id\\\""));

    let req = test::TestRequest::post()
        .uri("/validation?id=1001&response_type=Code")
        .set_json(&ItemBody {
            req: None,
            len: "len".to_owned(),
            eq: "1234567890".to_owned(),
            range: 19,
            url: "http://idealworld.group".to_owned(),
            mail: "i@sunisle.org".to_owned(),
            phone: "18657120202".to_owned(),
            cont: "ddd@gmail.com".to_owned(),
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert!(String::from_utf8(read_body(resp).await.to_vec())
        .unwrap()
        .contains("ValidationErrors({\\\"req\\\": Field([ValidationError { code: \\\"required\\\", message: None, params: {\\\"value\\\": Null} }])})\"}"));

    let req = test::TestRequest::post()
        .uri("/validation?id=1001&response_type=Code")
        .set_json(&ItemBody {
            req: Some("req".to_owned()),
            len: "".to_owned(),
            eq: "1234567890".to_owned(),
            range: 19,
            url: "http://idealworld.group".to_owned(),
            mail: "i@sunisle.org".to_owned(),
            phone: "18657120202".to_owned(),
            cont: "ddd@gmail.com".to_owned(),
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert!(String::from_utf8(read_body(resp).await.to_vec())
        .unwrap()
        .contains("ValidationErrors({\\\"len\\\": Field([ValidationError { code: \\\"length\\\", message: Some(\\\"custom msg\\\")"));

    let req = test::TestRequest::post()
        .uri("/validation?id=1001&response_type=Code")
        .set_json(&ItemBody {
            req: Some("req".to_owned()),
            len: "len".to_owned(),
            eq: "123456789".to_owned(),
            range: 19,
            url: "http://idealworld.group".to_owned(),
            mail: "i@sunisle.org".to_owned(),
            phone: "18657120202".to_owned(),
            cont: "ddd@gmail.com".to_owned(),
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert!(String::from_utf8(read_body(resp).await.to_vec())
        .unwrap()
        .contains("ValidationErrors({\\\"eq\\\": Field([ValidationError { code: \\\"length\\\","));

    let req = test::TestRequest::post()
        .uri("/validation?id=1001&response_type=Code")
        .set_json(&ItemBody {
            req: Some("req".to_owned()),
            len: "len".to_owned(),
            eq: "1234567890".to_owned(),
            range: 1,
            url: "http://idealworld.group".to_owned(),
            mail: "i@sunisle.org".to_owned(),
            phone: "18657120202".to_owned(),
            cont: "ddd@gmail.com".to_owned(),
        })
        .to_request();
    let resp = call_service(&app, req).await;

    assert!(String::from_utf8(read_body(resp).await.to_vec())
        .unwrap()
        .contains("ValidationErrors({\\\"range\\\": Field([ValidationError { code: \\\"range\\\","));

    let req = test::TestRequest::post()
        .uri("/validation?id=1001&response_type=Code")
        .set_json(&ItemBody {
            req: Some("req".to_owned()),
            len: "len".to_owned(),
            eq: "1234567890".to_owned(),
            range: 19,
            url: "idealworld.group".to_owned(),
            mail: "i@sunisle.org".to_owned(),
            phone: "18657120202".to_owned(),
            cont: "ddd@gmail.com".to_owned(),
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert!(String::from_utf8(read_body(resp).await.to_vec())
        .unwrap()
        .contains("ValidationErrors({\\\"url\\\": Field([ValidationError {"));

    let req = test::TestRequest::post()
        .uri("/validation?id=1001&response_type=Code")
        .set_json(&ItemBody {
            req: Some("req".to_owned()),
            len: "len".to_owned(),
            eq: "1234567890".to_owned(),
            range: 19,
            url: "http://idealworld.group".to_owned(),
            mail: "sunisle.org".to_owned(),
            phone: "18657120202".to_owned(),
            cont: "ddd@gmail.com".to_owned(),
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert!(String::from_utf8(read_body(resp).await.to_vec())
        .unwrap()
        .contains("ValidationErrors({\\\"mail\\\": Field([ValidationError { code: \\\"email\\\", message: None, params: {\\\"value\\\": String(\\\"sunisle.org\\\")} }])})\"}"));

    let req = test::TestRequest::post()
        .uri("/validation?id=1001&response_type=Code")
        .set_json(&ItemBody {
            req: Some("req".to_owned()),
            len: "len".to_owned(),
            eq: "1234567890".to_owned(),
            range: 19,
            url: "http://idealworld.group".to_owned(),
            mail: "i@sunisle.org".to_owned(),
            phone: "18657120202".to_owned(),
            cont: "ddd@163.com".to_owned(),
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert!(String::from_utf8(read_body(resp).await.to_vec())
        .unwrap()
        .contains("ValidationErrors({\\\"cont\\\": Field([ValidationError"));

    let req = test::TestRequest::post()
        .uri("/validation?id=1001&response_type=Code")
        .set_json(&ItemBody {
            req: Some("req".to_owned()),
            len: "len".to_owned(),
            eq: "1234567890".to_owned(),
            range: 19,
            url: "http://idealworld.group".to_owned(),
            mail: "i@sunisle.org".to_owned(),
            phone: "1865712020".to_owned(),
            cont: "ddd@gmail.com".to_owned(),
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert!(String::from_utf8(read_body(resp).await.to_vec())
        .unwrap()
        .contains("Validation error: ValidationErrors({\\\"phone\\\""));

    Ok(())
}

#[post("/normal/{id}")]
async fn normal() -> BIOSResp {
    BIOSRespHelper::ok("successful".to_owned())
}

#[post("/bus_error")]
async fn bus_error() -> BIOSResp {
    BIOSRespHelper::bus_err("xxx01", "business error")
}

#[post("/sys_error")]
async fn sys_error() -> BIOSResp {
    BIOSRespHelper::err(BIOSError::InternalError("没事，莫慌".to_owned()))
    //Err(BIOSError::InternalError("没事，莫慌".to_owned()))
}

#[derive(Debug, Deserialize)]
enum ResponseType {
    Token,
    Code,
}

#[derive(Deserialize, Validate)]
struct AuthRequest {
    #[validate(range(min = 1000, max = 9999))]
    id: u64,
    response_type: ResponseType,
}

#[derive(Deserialize, Serialize, Validate)]
struct ItemBody {
    #[validate(required)]
    req: Option<String>,
    #[validate(length(min = 1, max = 10, message = "custom msg"))]
    len: String,
    #[validate(length(equal = 10))]
    eq: String,
    #[validate(range(min = 18, max = 28))]
    range: u8,
    #[validate(url)]
    url: String,
    #[validate(email)]
    mail: String,
    #[validate(custom = "bios::web::validate::handler::validate_phone")]
    phone: String,
    #[validate(contains = "gmail")]
    cont: String,
}

#[post("/validation")]
async fn validation(_query: Query<AuthRequest>, _body: Json<ItemBody>) -> BIOSResp {
    BIOSRespHelper::ok("successful".to_owned())
}
