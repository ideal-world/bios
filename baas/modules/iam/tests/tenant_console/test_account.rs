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

use actix_web::body::AnyBody;
use actix_web::http::StatusCode;
use actix_web::test::{call_service, read_body_json};
use actix_web::{test, App};
use chrono::Utc;
use testcontainers::clients;

use bios::basic::config::FrameworkConfig;
use bios::basic::dto::BIOSResp;
use bios::basic::result::BIOSResult;
use bios::db::reldb_client::BIOSPage;
use bios::web::web_server::BIOSWebServer;
use bios_baas_iam::process::basic_dto::{AccountIdentKind, CommonStatus};
use bios_baas_iam::process::tenant_console;
use bios_baas_iam::process::tenant_console::tc_account_dto::{
    AccountAddReq, AccountAppDetailResp, AccountDetailResp, AccountIdentAddReq, AccountIdentDetailResp, AccountIdentModifyReq, AccountModifyReq,
};
use bios_baas_iam::process::tenant_console::tc_tenant_dto::TenantIdentAddReq;

use crate::test_basic;

#[actix_rt::test]
async fn test_account() -> BIOSResult<()> {
    let docker = clients::Cli::default();
    let _c = test_basic::init(&docker).await;
    let app = test::init_service(
        App::new()
            .wrap(BIOSWebServer::init_cors(&FrameworkConfig::default()))
            .wrap(BIOSWebServer::init_error_handlers())
            .service(tenant_console::tc_account_processor::add_account)
            .service(tenant_console::tc_account_processor::modify_account)
            .service(tenant_console::tc_account_processor::list_account)
            .service(tenant_console::tc_account_processor::delete_account),
    )
    .await;

    // Add Account
    let req = test::TestRequest::post()
        .insert_header(test_basic::context_account())
        .uri("/console/tenant/account")
        .set_json(&AccountAddReq {
            name: "孤岛旭日".to_string(),
            avatar: None,
            parameters: None,
            parent_id: None,
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let id = read_body_json::<BIOSResp<String>, AnyBody>(resp).await.body.unwrap();

    // Modify Account
    let req = test::TestRequest::put()
        .insert_header(test_basic::context_account())
        .uri(format!("/console/tenant/account/{}", id.clone()).as_str())
        .set_json(&AccountModifyReq {
            name: Some("gudaoxuri".to_string()),
            avatar: None,
            parameters: None,
            parent_id: None,
            status: Some(CommonStatus::Disabled),
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let result = read_body_json::<BIOSResp<String>, AnyBody>(resp).await;
    assert_eq!(result.code, "200");

    // List Account
    let req = test::TestRequest::get().insert_header(test_basic::context_account()).uri("/console/tenant/account?name=gudaoxuri&page_number=1&page_size=10").to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_body_json::<BIOSResp<BIOSPage<AccountDetailResp>>, AnyBody>(resp).await.body.unwrap();
    assert_eq!(body.total_size, 1);
    assert_eq!(body.records[0].name, "gudaoxuri");
    assert_eq!(body.records[0].create_user, "平台管理员");
    assert_eq!(body.records[0].update_user, "平台管理员");

    // Delete Account
    let req = test::TestRequest::delete().insert_header(test_basic::context_account()).uri(format!("/console/tenant/account/{}", id.clone()).as_str()).to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let result = read_body_json::<BIOSResp<String>, AnyBody>(resp).await;
    assert_eq!(result.code, "200");

    Ok(())
}

#[actix_rt::test]
async fn test_account_ident() -> BIOSResult<()> {
    let docker = clients::Cli::default();
    let _c = test_basic::init(&docker).await;
    let app = test::init_service(
        App::new()
            .wrap(BIOSWebServer::init_cors(&FrameworkConfig::default()))
            .wrap(BIOSWebServer::init_error_handlers())
            .service(tenant_console::tc_tenant_processor::add_tenant_ident)
            .service(tenant_console::tc_account_processor::add_account)
            .service(tenant_console::tc_account_processor::add_account_ident)
            .service(tenant_console::tc_account_processor::modify_account_ident)
            .service(tenant_console::tc_account_processor::list_account_ident)
            .service(tenant_console::tc_account_processor::delete_account_ident),
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

    // Add Account
    let req = test::TestRequest::post()
        .insert_header(test_basic::context_account())
        .uri("/console/tenant/account")
        .set_json(&AccountAddReq {
            name: "孤岛旭日".to_string(),
            avatar: None,
            parameters: None,
            parent_id: None,
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let account_id = read_body_json::<BIOSResp<String>, AnyBody>(resp).await.body.unwrap();

    // Add AccountIdent
    let req = test::TestRequest::post()
        .insert_header(test_basic::context_account())
        .uri(format!("/console/tenant/account/{}/ident", account_id.clone()).as_str())
        .set_json(&AccountIdentAddReq {
            kind: AccountIdentKind::Username,
            ak: "gdxr".to_string(),
            sk: Some("1223".to_string()),
            valid_start_time: Utc::now().timestamp(),
            valid_end_time: Utc::now().timestamp() + 10000,
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let id = read_body_json::<BIOSResp<String>, AnyBody>(resp).await.body.unwrap();

    // Modify AccountIdent
    let req = test::TestRequest::put()
        .insert_header(test_basic::context_account())
        .uri(format!("/console/tenant/account/{}/ident/{}", account_id.clone(), id.clone()).as_str())
        .set_json(&AccountIdentModifyReq {
            ak: Some("gdxr2".to_string()),
            sk: Some("11111".to_string()),
            valid_start_time: None,
            valid_end_time: None,
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let result = read_body_json::<BIOSResp<String>, AnyBody>(resp).await;
    assert_eq!(result.code, "200");

    // List AccountIdent
    let req = test::TestRequest::get().insert_header(test_basic::context_account()).uri(format!("/console/tenant/account/{}/ident", account_id.clone()).as_str()).to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_body_json::<BIOSResp<Vec<AccountIdentDetailResp>>, AnyBody>(resp).await.body.unwrap();
    assert_eq!(body[0].ak, "gdxr2");
    assert_eq!(body[0].create_user, "平台管理员");
    assert_eq!(body[0].update_user, "平台管理员");

    // Delete AccountIdent
    let req = test::TestRequest::delete()
        .insert_header(test_basic::context_account())
        .uri(format!("/console/tenant/account/{}/ident/{}", account_id.clone(), id.clone()).as_str())
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let result = read_body_json::<BIOSResp<String>, AnyBody>(resp).await;
    assert_eq!(result.code, "200");

    Ok(())
}

#[actix_rt::test]
async fn test_account_app() -> BIOSResult<()> {
    let docker = clients::Cli::default();
    let _c = test_basic::init(&docker).await;
    let app = test::init_service(
        App::new()
            .wrap(BIOSWebServer::init_cors(&FrameworkConfig::default()))
            .wrap(BIOSWebServer::init_error_handlers())
            .service(tenant_console::tc_account_processor::add_account)
            .service(tenant_console::tc_account_processor::add_account_app)
            .service(tenant_console::tc_account_processor::list_account_app)
            .service(tenant_console::tc_account_processor::delete_account_app),
    )
    .await;

    // Add Account
    let req = test::TestRequest::post()
        .insert_header(test_basic::context_account())
        .uri("/console/tenant/account")
        .set_json(&AccountAddReq {
            name: "孤岛旭日".to_string(),
            avatar: None,
            parameters: None,
            parent_id: None,
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let account_id = read_body_json::<BIOSResp<String>, AnyBody>(resp).await.body.unwrap();

    // Add AccountApp
    let req =
        test::TestRequest::post().insert_header(test_basic::context_account()).uri(format!("/console/tenant/account/{}/app/{}", account_id.clone(), "app1").as_str()).to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // List AccountApp
    let req = test::TestRequest::get().insert_header(test_basic::context_account()).uri(format!("/console/tenant/account/{}/app", account_id.clone()).as_str()).to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_body_json::<BIOSResp<Vec<AccountAppDetailResp>>, AnyBody>(resp).await.body.unwrap();
    assert_eq!(body[0].rel_app_code, "app1");
    assert_eq!(body[0].rel_account_id, account_id);
    assert_eq!(body[0].create_user, "平台管理员");
    assert_eq!(body[0].update_user, "平台管理员");

    // Delete AccountApp
    let req = test::TestRequest::delete()
        .insert_header(test_basic::context_account())
        .uri(format!("/console/tenant/account/{}/app/{}", account_id.clone(), "app1").as_str())
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let result = read_body_json::<BIOSResp<String>, AnyBody>(resp).await;
    assert_eq!(result.code, "200");

    Ok(())
}
