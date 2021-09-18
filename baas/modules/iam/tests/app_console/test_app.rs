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

use actix_web::body::AnyBody;
use actix_web::http::StatusCode;
use actix_web::test::{call_service, read_body_json};
use actix_web::{test, App};
use chrono::NaiveDate;
use testcontainers::clients;

use crate::test_basic;
use bios::basic::config::FrameworkConfig;
use bios::basic::dto::BIOSResp;
use bios::basic::result::BIOSResult;
use bios::web::web_server::BIOSWebServer;
use bios_baas_iam::process::app_console;
use bios_baas_iam::process::app_console::ac_app_dto::{AppIdentAddReq, AppIdentDetailResp, AppIdentModifyReq};

#[actix_rt::test]
async fn test_app_ident() -> BIOSResult<()> {
    let docker = clients::Cli::default();
    let _c = test_basic::init(&docker).await;
    let app = test::init_service(
        App::new()
            .wrap(BIOSWebServer::init_cors(&FrameworkConfig::default()))
            .wrap(BIOSWebServer::init_error_handlers())
            .service(app_console::ac_app_processor::add_app_ident)
            .service(app_console::ac_app_processor::modify_app_ident)
            .service(app_console::ac_app_processor::list_app_ident)
            .service(app_console::ac_app_processor::get_app_ident_sk)
            .service(app_console::ac_app_processor::delete_app_ident),
    )
    .await;

    // Add AppIdent
    let req = test::TestRequest::post()
        .insert_header(test_basic::context_account())
        .uri("/console/app/app/ident")
        .set_json(&AppIdentAddReq {
            note: "测试".to_string(),
            valid_time: NaiveDate::from_ymd(3000, 1, 1).and_hms(0, 0, 0).timestamp(),
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let id = read_body_json::<BIOSResp<String>, AnyBody>(resp).await.body.unwrap();

    // Modify AppIdent
    let req = test::TestRequest::put()
        .insert_header(test_basic::context_account())
        .uri(format!("/console/app/app/ident/{}", id.clone()).as_str())
        .set_json(&AppIdentModifyReq {
            note: Some("测试使用".to_string()),
            valid_time: None,
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let result = read_body_json::<BIOSResp<String>, AnyBody>(resp).await;
    assert_eq!(result.code, "200");

    // List AppIdent
    let req = test::TestRequest::get().insert_header(test_basic::context_account()).uri("/console/app/app/ident").to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_body_json::<BIOSResp<Vec<AppIdentDetailResp>>, AnyBody>(resp).await.body.unwrap();
    assert_eq!(body[0].note, "测试使用");
    assert_eq!(body[0].valid_time, NaiveDate::from_ymd(3000, 1, 1).and_hms(0, 0, 0).timestamp());
    assert_eq!(body[0].create_user, "平台管理员");
    assert_eq!(body[0].update_user, "平台管理员");

    // Get AppIdent SK
    let req = test::TestRequest::get().insert_header(test_basic::context_account()).uri(format!("/console/app/app/ident/{}/sk", id.clone()).as_str()).to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let sk = read_body_json::<BIOSResp<String>, AnyBody>(resp).await.body.unwrap();
    assert_ne!(sk, "");

    // Delete AppIdent
    let req = test::TestRequest::delete().insert_header(test_basic::context_account()).uri(format!("/console/app/app/ident/{}", id.clone()).as_str()).to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let result = read_body_json::<BIOSResp<String>, AnyBody>(resp).await;
    assert_eq!(result.code, "200");

    Ok(())
}
