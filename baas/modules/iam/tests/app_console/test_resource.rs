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
use bios_baas_iam::process::app_console::ac_resource_dto::{
    ResourceAddReq, ResourceDetailResp, ResourceModifyReq, ResourceSubjectAddReq, ResourceSubjectDetailResp, ResourceSubjectModifyReq,
};
use bios_baas_iam::process::basic_dto::ResourceKind;

#[actix_rt::test]
async fn test_resource_subject() -> BIOSResult<()> {
    let docker = clients::Cli::default();
    let _c = crate::test_basic::init(&docker).await;
    let app = test::init_service(
        App::new()
            .wrap(BIOSWebServer::init_cors(&FrameworkConfig::default()))
            .wrap(BIOSWebServer::init_error_handlers())
            .service(app_console::ac_resource_processor::add_resource_subject)
            .service(app_console::ac_resource_processor::modify_resource_subject)
            .service(app_console::ac_resource_processor::list_resource_subject)
            .service(app_console::ac_resource_processor::delete_resource_subject),
    )
    .await;

    // Add ResourceSubject
    let req = test::TestRequest::post()
        .insert_header((
            BIOSFuns::fw_config().web.ident_info_flag.clone(),
            bios::basic::security::digest::base64::encode(r#"{"app_id":"app1","tenant_id":"tenant1","account_id":"admin001","ak":"ak1","token":"t01"}"#),
        ))
        .uri("/console/app/resource/subject")
        .set_json(&ResourceSubjectAddReq {
            code_postfix: "httpbin".to_string(),
            name: "测试Http请求".to_string(),
            sort: 0,
            kind: ResourceKind::Api,
            uri: "http://httpbin.org".to_string(),
            ak: None,
            sk: None,
            platform_account: None,
            platform_project_id: None,
            timeout_ms: None,
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let id = read_body_json::<BIOSRespHelper<String>, AnyBody>(resp).await.body.unwrap();

    // Modify ResourceSubject
    let req = test::TestRequest::put()
        .insert_header((
            BIOSFuns::fw_config().web.ident_info_flag.clone(),
            bios::basic::security::digest::base64::encode(r#"{"app_id":"app1","tenant_id":"tenant1","account_id":"admin001","ak":"ak1","token":"t01"}"#),
        ))
        .uri(format!("/console/app/resource/subject/{}", id.clone()).as_str())
        .set_json(&ResourceSubjectModifyReq {
            code_postfix: Some("httpbin_test".to_string()),
            name: Some("测试Http请求1".to_string()),
            sort: None,
            kind: Some(ResourceKind::Api),
            uri: Some("https://httpbin.org".to_string()),
            ak: None,
            sk: None,
            platform_account: None,
            platform_project_id: None,
            timeout_ms: None,
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let result = read_body_json::<BIOSRespHelper<String>, AnyBody>(resp).await;
    assert_eq!(result.code, "200");

    // List ResourceSubject
    let req = test::TestRequest::get()
        .insert_header((
            BIOSFuns::fw_config().web.ident_info_flag.clone(),
            bios::basic::security::digest::base64::encode(r#"{"app_id":"app1","tenant_id":"tenant1","account_id":"admin001","ak":"ak1","token":"t01"}"#),
        ))
        .uri("/console/app/resource/subject?page_number=1&page_size=10&name=Http")
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_body_json::<BIOSRespHelper<BIOSPage<ResourceSubjectDetailResp>>, AnyBody>(resp).await.body.unwrap();
    assert_eq!(body.total_size, 1);
    assert_eq!(body.records[0].name, "测试Http请求1");
    assert_eq!(body.records[0].code, "app1.api.httpbin_test");
    assert_eq!(body.records[0].create_user, "平台管理员");
    assert_eq!(body.records[0].update_user, "平台管理员");

    // Delete ResourceSubject
    let req = test::TestRequest::delete()
        .insert_header((
            BIOSFuns::fw_config().web.ident_info_flag.clone(),
            bios::basic::security::digest::base64::encode(r#"{"app_id":"app1","tenant_id":"tenant1","account_id":"admin001","ak":"ak1","token":"t01"}"#),
        ))
        .uri(format!("/console/app/resource/subject/{}", id.clone()).as_str())
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let result = read_body_json::<BIOSRespHelper<String>, AnyBody>(resp).await;
    assert_eq!(result.code, "200");

    Ok(())
}

#[actix_rt::test]
async fn test_resource() -> BIOSResult<()> {
    let docker = clients::Cli::default();
    let _c = crate::test_basic::init(&docker).await;
    let app = test::init_service(
        App::new()
            .wrap(BIOSWebServer::init_cors(&FrameworkConfig::default()))
            .wrap(BIOSWebServer::init_error_handlers())
            .service(app_console::ac_resource_processor::add_resource_subject)
            .service(app_console::ac_resource_processor::add_resource)
            .service(app_console::ac_resource_processor::modify_resource)
            .service(app_console::ac_resource_processor::list_resource)
            .service(app_console::ac_resource_processor::delete_resource),
    )
    .await;

    // Add Resource NOT found
    let req = test::TestRequest::post()
        .insert_header((
            BIOSFuns::fw_config().web.ident_info_flag.clone(),
            bios::basic::security::digest::base64::encode(r#"{"app_id":"app1","tenant_id":"tenant1","account_id":"admin001","ak":"ak1","token":"t01"}"#),
        ))
        .uri("/console/app/resource")
        .set_json(&ResourceAddReq {
            name: "测试Get请求".to_string(),
            path_and_query: "/get".to_string(),
            icon: "xxx.png".to_string(),
            action: None,
            sort: 1,
            res_group: false,
            parent_id: None,
            rel_resource_subject_id: "r1".to_string(),
            expose_kind: None,
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let result = read_body_json::<BIOSRespHelper<String>, AnyBody>(resp).await;
    assert_eq!(result.code, "404");
    assert_eq!(result.msg, "Resource [rel_resource_subject_id] not exists");

    // Add ResourceSubject
    let req = test::TestRequest::post()
        .insert_header((
            BIOSFuns::fw_config().web.ident_info_flag.clone(),
            bios::basic::security::digest::base64::encode(r#"{"app_id":"app1","tenant_id":"tenant1","account_id":"admin001","ak":"ak1","token":"t01"}"#),
        ))
        .uri("/console/app/resource/subject")
        .set_json(&ResourceSubjectAddReq {
            code_postfix: "httpbin".to_string(),
            name: "测试Http请求".to_string(),
            sort: 0,
            kind: ResourceKind::Api,
            uri: "http://httpbin.org".to_string(),
            ak: None,
            sk: None,
            platform_account: None,
            platform_project_id: None,
            timeout_ms: None,
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let subject_id = read_body_json::<BIOSRespHelper<String>, AnyBody>(resp).await.body.unwrap();

    // Add Resource
    let req = test::TestRequest::post()
        .insert_header((
            BIOSFuns::fw_config().web.ident_info_flag.clone(),
            bios::basic::security::digest::base64::encode(r#"{"app_id":"app1","tenant_id":"tenant1","account_id":"admin001","ak":"ak1","token":"t01"}"#),
        ))
        .uri("/console/app/resource")
        .set_json(&ResourceAddReq {
            name: "测试Get请求".to_string(),
            path_and_query: "/get".to_string(),
            icon: "xxx.png".to_string(),
            action: None,
            sort: 1,
            res_group: false,
            parent_id: None,
            rel_resource_subject_id: subject_id.clone(),
            expose_kind: None,
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let id = read_body_json::<BIOSRespHelper<String>, AnyBody>(resp).await.body.unwrap();

    // Add Resource again
    let req = test::TestRequest::post()
        .insert_header((
            BIOSFuns::fw_config().web.ident_info_flag.clone(),
            bios::basic::security::digest::base64::encode(r#"{"app_id":"app1","tenant_id":"tenant1","account_id":"admin001","ak":"ak1","token":"t01"}"#),
        ))
        .uri("/console/app/resource")
        .set_json(&ResourceAddReq {
            name: "测试Get请求".to_string(),
            path_and_query: "/get".to_string(),
            icon: "xxx.png".to_string(),
            action: None,
            sort: 1,
            res_group: false,
            parent_id: None,
            rel_resource_subject_id: subject_id.clone(),
            expose_kind: None,
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let result = read_body_json::<BIOSRespHelper<String>, AnyBody>(resp).await;
    assert_eq!(result.code, "409");
    assert_eq!(result.msg, "Resource [path_and_query] already exists");

    // Modify Resource
    let req = test::TestRequest::put()
        .insert_header((
            BIOSFuns::fw_config().web.ident_info_flag.clone(),
            bios::basic::security::digest::base64::encode(r#"{"app_id":"app1","tenant_id":"tenant1","account_id":"admin001","ak":"ak1","token":"t01"}"#),
        ))
        .uri(format!("/console/app/resource/{}", id.clone()).as_str())
        .set_json(&ResourceModifyReq {
            name: Some("测试Http GET请求".to_string()),
            path_and_query: Some("/get".to_string()),
            icon: None,
            action: None,
            sort: None,
            res_group: None,
            parent_id: None,
            expose_kind: None,
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let result = read_body_json::<BIOSRespHelper<String>, AnyBody>(resp).await;
    assert_eq!(result.code, "200");

    // List Resource
    let req = test::TestRequest::get()
        .insert_header((
            BIOSFuns::fw_config().web.ident_info_flag.clone(),
            bios::basic::security::digest::base64::encode(r#"{"app_id":"app1","tenant_id":"tenant1","account_id":"admin001","ak":"ak1","token":"t01"}"#),
        ))
        .uri("/console/app/resource?page_number=1&page_size=10&name=Http&expose=false")
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_body_json::<BIOSRespHelper<BIOSPage<ResourceDetailResp>>, AnyBody>(resp).await.body.unwrap();
    assert_eq!(body.total_size, 1);
    assert_eq!(body.records[0].name, "测试Http GET请求");
    assert_eq!(body.records[0].path_and_query, "/get");
    assert_eq!(body.records[0].create_user, "平台管理员");
    assert_eq!(body.records[0].update_user, "平台管理员");

    let req = test::TestRequest::get()
        .insert_header((
            BIOSFuns::fw_config().web.ident_info_flag.clone(),
            bios::basic::security::digest::base64::encode(r#"{"app_id":"app1","tenant_id":"tenant1","account_id":"admin001","ak":"ak1","token":"t01"}"#),
        ))
        .uri("/console/app/resource?page_number=1&page_size=10&name=Http&expose=true")
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_body_json::<BIOSRespHelper<BIOSPage<ResourceDetailResp>>, AnyBody>(resp).await.body.unwrap();
    assert_eq!(body.total_size, 0);

    // Delete Resource
    let req = test::TestRequest::delete()
        .insert_header((
            BIOSFuns::fw_config().web.ident_info_flag.clone(),
            bios::basic::security::digest::base64::encode(r#"{"app_id":"app1","tenant_id":"tenant1","account_id":"admin001","ak":"ak1","token":"t01"}"#),
        ))
        .uri(format!("/console/app/resource/{}", id.clone()).as_str())
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let result = read_body_json::<BIOSRespHelper<String>, AnyBody>(resp).await;
    assert_eq!(result.code, "200");

    Ok(())
}
