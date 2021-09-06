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
use bios_baas_iam::process::app_console;
use bios_baas_iam::process::app_console::ac_role_dto::{RoleAddReq, RoleDetailResp, RoleModifyReq};

#[actix_rt::test]
async fn test_role() -> BIOSResult<()> {
    let docker = clients::Cli::default();
    let _c = crate::test_basic::init(&docker).await;
    let app = test::init_service(
        App::new()
            .wrap(BIOSWebServer::init_cors(&FrameworkConfig::default()))
            .wrap(BIOSWebServer::init_error_handlers())
            .service(app_console::ac_role_processor::add_role)
            .service(app_console::ac_role_processor::modify_role)
            .service(app_console::ac_role_processor::list_role)
            .service(app_console::ac_role_processor::delete_role),
    )
    .await;

    // Add Role
    let req = test::TestRequest::post()
        .insert_header((
            BIOSFuns::fw_config().web.ident_info_flag.clone(),
            bios::basic::security::digest::base64::encode(r#"{"app_id":"app1","tenant_id":"tenant1","account_id":"admin001","ak":"ak1","token":"t01"}"#),
        ))
        .uri("/console/app/role")
        .set_json(&RoleAddReq {
            code: "admin".to_string(),
            name: "管理员".to_string(),
            sort: 1,
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let id = read_body_json::<BIOSRespHelper<String>, AnyBody>(resp).await.body.unwrap();

    // Add Role again
    let req = test::TestRequest::post()
        .insert_header((
            BIOSFuns::fw_config().web.ident_info_flag.clone(),
            bios::basic::security::digest::base64::encode(r#"{"app_id":"app1","tenant_id":"tenant1","account_id":"admin001","ak":"ak1","token":"t01"}"#),
        ))
        .uri("/console/app/role")
        .set_json(&RoleAddReq {
            code: "admin".to_string(),
            name: "管理员".to_string(),
            sort: 1,
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let result = read_body_json::<BIOSRespHelper<String>, AnyBody>(resp).await;
    assert_eq!(result.code, "409");
    assert_eq!(result.msg, "Role [code] already exists");

    // Modify Role
    let req = test::TestRequest::put()
        .insert_header((
            BIOSFuns::fw_config().web.ident_info_flag.clone(),
            bios::basic::security::digest::base64::encode(r#"{"app_id":"app1","tenant_id":"tenant1","account_id":"admin001","ak":"ak1","token":"t01"}"#),
        ))
        .uri(format!("/console/app/role/{}", id.clone()).as_str())
        .set_json(&RoleModifyReq {
            code: Some("test_admin".to_string()),
            name: Some("测试管理员".to_string()),
            sort: None,
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let result = read_body_json::<BIOSRespHelper<String>, AnyBody>(resp).await;
    assert_eq!(result.code, "200");

    // List Role
    let req = test::TestRequest::get()
        .insert_header((
            BIOSFuns::fw_config().web.ident_info_flag.clone(),
            bios::basic::security::digest::base64::encode(r#"{"app_id":"app1","tenant_id":"tenant1","account_id":"admin001","ak":"ak1","token":"t01"}"#),
        ))
        .uri("/console/app/role?page_number=1&page_size=10&code=admin")
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_body_json::<BIOSRespHelper<BIOSPage<RoleDetailResp>>, AnyBody>(resp).await.body.unwrap();
    assert_eq!(body.total_size, 1);
    assert_eq!(body.records[0].code, "test_admin");
    assert_eq!(body.records[0].name, "测试管理员");
    assert_eq!(body.records[0].create_user, "平台管理员");
    assert_eq!(body.records[0].update_user, "平台管理员");

    // Delete Role
    let req = test::TestRequest::delete()
        .insert_header((
            BIOSFuns::fw_config().web.ident_info_flag.clone(),
            bios::basic::security::digest::base64::encode(r#"{"app_id":"app1","tenant_id":"tenant1","account_id":"admin001","ak":"ak1","token":"t01"}"#),
        ))
        .uri(format!("/console/app/role/{}", id.clone()).as_str())
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let result = read_body_json::<BIOSRespHelper<String>, AnyBody>(resp).await;
    assert_eq!(result.code, "200");

    Ok(())
}
