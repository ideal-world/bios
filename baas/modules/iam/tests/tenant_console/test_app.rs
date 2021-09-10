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
use testcontainers::clients;

use bios::basic::config::FrameworkConfig;
use bios::basic::error::BIOSResult;
use bios::db::reldb_client::BIOSPage;
use bios::web::resp_handler::BIOSRespHelper;
use bios::web::web_server::BIOSWebServer;
use bios::BIOSFuns;
use bios_baas_iam::process::tenant_console;
use bios_baas_iam::process::tenant_console::tc_app_dto::{AppAddReq, AppDetailResp, AppModifyReq};

#[actix_rt::test]
async fn test_app() -> BIOSResult<()> {
    let docker = clients::Cli::default();
    let _c = crate::test_basic::init(&docker).await;
    let app = test::init_service(
        App::new()
            .wrap(BIOSWebServer::init_cors(&FrameworkConfig::default()))
            .wrap(BIOSWebServer::init_error_handlers())
            .service(tenant_console::tc_app_processor::add_app)
            .service(tenant_console::tc_app_processor::modify_app)
            .service(tenant_console::tc_app_processor::list_app)
            .service(tenant_console::tc_app_processor::delete_app),
    )
    .await;

    // Add App
    let req = test::TestRequest::post()
        .insert_header((
            BIOSFuns::fw_config().web.ident_info_flag.clone(),
            bios::basic::security::digest::base64::encode(r#"{"app_id":"app1","tenant_id":"tenant1","account_id":"admin001","ak":"ak1","token":"t01"}"#),
        ))
        .uri("/console/tenant/app")
        .set_json(&AppAddReq {
            name: "测试应用".to_string(),
            icon: None,
            parameters: None,
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let id = read_body_json::<BIOSRespHelper<String>, AnyBody>(resp).await.body.unwrap();

    // Modify App
    let req = test::TestRequest::put()
        .insert_header((
            BIOSFuns::fw_config().web.ident_info_flag.clone(),
            bios::basic::security::digest::base64::encode(r#"{"app_id":"app1","tenant_id":"tenant1","account_id":"admin001","ak":"ak1","token":"t01"}"#),
        ))
        .uri(format!("/console/tenant/app/{}", id.clone()).as_str())
        .set_json(&AppModifyReq {
            name: Some("test_app".to_string()),
            icon: None,
            parameters: None,
            status: None,
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let result = read_body_json::<BIOSRespHelper<String>, AnyBody>(resp).await;
    assert_eq!(result.code, "200");

    // List App
    let req = test::TestRequest::get()
        .insert_header((
            BIOSFuns::fw_config().web.ident_info_flag.clone(),
            bios::basic::security::digest::base64::encode(r#"{"app_id":"app1","tenant_id":"tenant1","account_id":"admin001","ak":"ak1","token":"t01"}"#),
        ))
        .uri("/console/tenant/app?name=test&page_number=1&page_size=10")
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_body_json::<BIOSRespHelper<BIOSPage<AppDetailResp>>, AnyBody>(resp).await.body.unwrap();
    assert_eq!(body.total_size, 1);
    assert_eq!(body.records[0].name, "test_app");
    assert_eq!(body.records[0].create_user, "平台管理员");
    assert_eq!(body.records[0].update_user, "平台管理员");

    // Delete App
    let req = test::TestRequest::delete()
        .insert_header((
            BIOSFuns::fw_config().web.ident_info_flag.clone(),
            bios::basic::security::digest::base64::encode(r#"{"app_id":"app1","tenant_id":"tenant1","account_id":"admin001","ak":"ak1","token":"t01"}"#),
        ))
        .uri(format!("/console/tenant/app/{}", id.clone()).as_str())
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let result = read_body_json::<BIOSRespHelper<String>, AnyBody>(resp).await;
    assert_eq!(result.code, "200");

    Ok(())
}
