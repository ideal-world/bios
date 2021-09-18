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
use bios::basic::dto::BIOSResp;
use bios::basic::result::BIOSResult;
use bios::web::web_server::BIOSWebServer;
use bios_baas_iam::process::basic_dto::AccountIdentKind;
use bios_baas_iam::process::tenant_console;
use bios_baas_iam::process::tenant_console::tc_tenant_dto::{
    TenantCertAddReq, TenantCertDetailResp, TenantCertModifyReq, TenantDetailResp, TenantIdentAddReq, TenantIdentDetailResp, TenantIdentModifyReq, TenantModifyReq,
};

use crate::test_basic;

#[actix_rt::test]
async fn test_tenant() -> BIOSResult<()> {
    let docker = clients::Cli::default();
    let _c = test_basic::init(&docker).await;
    let app = test::init_service(
        App::new()
            .wrap(BIOSWebServer::init_cors(&FrameworkConfig::default()))
            .wrap(BIOSWebServer::init_error_handlers())
            .service(tenant_console::tc_tenant_processor::modify_tenant)
            .service(tenant_console::tc_tenant_processor::get_tenant),
    )
    .await;

    // Modify Tenant
    let req = test::TestRequest::put()
        .insert_header(test_basic::context_account())
        .uri("/console/tenant/tenant")
        .set_json(&TenantModifyReq {
            name: Some("ideal world".to_string()),
            icon: None,
            allow_account_register: Some(true),
            parameters: None,
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let result = read_body_json::<BIOSResp<String>, AnyBody>(resp).await;
    assert_eq!(result.code, "200");

    // Get Tenant
    let req = test::TestRequest::get().insert_header(test_basic::context_account()).uri("/console/tenant/tenant").to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_body_json::<BIOSResp<TenantDetailResp>, AnyBody>(resp).await.body.unwrap();
    assert_eq!(body.name, "ideal world");
    assert_eq!(body.allow_account_register, true);
    assert_eq!(body.create_user, "平台管理员");
    assert_eq!(body.update_user, "平台管理员");

    Ok(())
}

#[actix_rt::test]
async fn test_tenant_cert() -> BIOSResult<()> {
    let docker = clients::Cli::default();
    let _c = test_basic::init(&docker).await;
    let app = test::init_service(
        App::new()
            .wrap(BIOSWebServer::init_cors(&FrameworkConfig::default()))
            .wrap(BIOSWebServer::init_error_handlers())
            .service(tenant_console::tc_tenant_processor::add_tenant_cert)
            .service(tenant_console::tc_tenant_processor::modify_tenant_cert)
            .service(tenant_console::tc_tenant_processor::list_tenant_cert)
            .service(tenant_console::tc_tenant_processor::delete_tenant_cert),
    )
    .await;

    // Add TenantCert
    let req = test::TestRequest::post()
        .insert_header(test_basic::context_account())
        .uri("/console/tenant/tenant/cert")
        .set_json(&TenantCertAddReq {
            category: "app".to_string(),
            version: 2,
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let id = read_body_json::<BIOSResp<String>, AnyBody>(resp).await.body.unwrap();

    // Modify TenantCert
    let req = test::TestRequest::put()
        .insert_header(test_basic::context_account())
        .uri(format!("/console/tenant/tenant/cert/{}", id.clone()).as_str())
        .set_json(&TenantCertModifyReq { version: Some(3) })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let result = read_body_json::<BIOSResp<String>, AnyBody>(resp).await;
    assert_eq!(result.code, "200");

    // List TenantCert
    let req = test::TestRequest::get().insert_header(test_basic::context_account()).uri("/console/tenant/tenant/cert").to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_body_json::<BIOSResp<Vec<TenantCertDetailResp>>, AnyBody>(resp).await.body.unwrap();
    assert_eq!(body[0].category, "app");
    assert_eq!(body[0].version, 3);
    assert_eq!(body[0].create_user, "平台管理员");
    assert_eq!(body[0].update_user, "平台管理员");

    // Delete TenantCert
    let req = test::TestRequest::delete().insert_header(test_basic::context_account()).uri(format!("/console/tenant/tenant/cert/{}", id.clone()).as_str()).to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let result = read_body_json::<BIOSResp<String>, AnyBody>(resp).await;
    assert_eq!(result.code, "200");

    Ok(())
}

#[actix_rt::test]
async fn test_tenant_ident() -> BIOSResult<()> {
    let docker = clients::Cli::default();
    let _c = test_basic::init(&docker).await;
    let app = test::init_service(
        App::new()
            .wrap(BIOSWebServer::init_cors(&FrameworkConfig::default()))
            .wrap(BIOSWebServer::init_error_handlers())
            .service(tenant_console::tc_tenant_processor::add_tenant_ident)
            .service(tenant_console::tc_tenant_processor::modify_tenant_ident)
            .service(tenant_console::tc_tenant_processor::list_tenant_ident)
            .service(tenant_console::tc_tenant_processor::delete_tenant_ident),
    )
    .await;

    // Add TenantIdent
    let req = test::TestRequest::post()
        .insert_header(test_basic::context_account())
        .uri("/console/tenant/tenant/ident")
        .set_json(&TenantIdentAddReq {
            kind: AccountIdentKind::Username,
            valid_ak_rule_note: None,
            valid_ak_rule: None,
            valid_sk_rule_note: None,
            valid_sk_rule: None,
            valid_time: 36000,
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let id = read_body_json::<BIOSResp<String>, AnyBody>(resp).await.body.unwrap();

    // Modify TenantIdent
    let req = test::TestRequest::put()
        .insert_header(test_basic::context_account())
        .uri(format!("/console/tenant/tenant/ident/{}", id.clone()).as_str())
        .set_json(&TenantIdentModifyReq {
            valid_ak_rule_note: None,
            valid_ak_rule: None,
            valid_sk_rule_note: None,
            valid_sk_rule: None,
            valid_time: Some(1000),
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let result = read_body_json::<BIOSResp<String>, AnyBody>(resp).await;
    assert_eq!(result.code, "200");

    // List TenantIdent
    let req = test::TestRequest::get().insert_header(test_basic::context_account()).uri("/console/tenant/tenant/ident").to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_body_json::<BIOSResp<Vec<TenantIdentDetailResp>>, AnyBody>(resp).await.body.unwrap();
    assert_eq!(body[0].valid_time, 1000);
    assert_eq!(body[0].create_user, "平台管理员");
    assert_eq!(body[0].update_user, "平台管理员");

    // Delete TenantIdent
    let req = test::TestRequest::delete().insert_header(test_basic::context_account()).uri(format!("/console/tenant/tenant/ident/{}", id.clone()).as_str()).to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let result = read_body_json::<BIOSResp<String>, AnyBody>(resp).await;
    assert_eq!(result.code, "200");

    Ok(())
}
