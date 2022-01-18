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
use testcontainers::clients;

use crate::test_basic;
use bios::basic::config::FrameworkConfig;
use bios::basic::dto::BIOSResp;
use bios::basic::result::BIOSResult;
use bios::web::web_server::BIOSWebServer;
use bios_baas_iam::process::app_console::ac_account_dto::{AccountGroupDetailResp, AccountRoleDetailResp};
use bios_baas_iam::process::app_console::ac_group_dto::{GroupAddReq, GroupNodeAddReq};
use bios_baas_iam::process::app_console::ac_role_dto::RoleAddReq;
use bios_baas_iam::process::basic_dto::GroupKind;
use bios_baas_iam::process::tenant_console::tc_account_dto::AccountAddReq;
use bios_baas_iam::process::{app_console, tenant_console};
use serde_json::Value;

#[actix_rt::test]
async fn test_account_role() -> BIOSResult<()> {
    let docker = clients::Cli::default();
    let _c = test_basic::init(&docker).await;
    let app = test::init_service(
        App::new()
            .wrap(BIOSWebServer::init_cors(&FrameworkConfig::default()))
            .wrap(BIOSWebServer::init_error_handlers())
            .service(tenant_console::tc_account_processor::add_account)
            .service(app_console::ac_role_processor::add_role)
            .service(app_console::ac_account_processor::add_account_role)
            .service(app_console::ac_account_processor::list_account_role)
            .service(app_console::ac_account_processor::delete_account_role),
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

    // Add Role
    let req = test::TestRequest::post()
        .insert_header(test_basic::context_account())
        .uri("/console/app/role")
        .set_json(&RoleAddReq {
            code: "admin".to_string(),
            name: "管理员".to_string(),
            sort: 1,
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let role_id = read_body_json::<BIOSResp<String>, AnyBody>(resp).await.body.unwrap();

    // Add AccountRole
    let req = test::TestRequest::post()
        .insert_header(test_basic::context_account())
        .uri(format!("/console/app/account/{}/role/{}", account_id.clone(), role_id.clone()).as_str())
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // List AccountRole
    let req = test::TestRequest::get().insert_header(test_basic::context_account()).uri(format!("/console/app/account/{}/role", account_id.clone()).as_str()).to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_body_json::<BIOSResp<Vec<AccountRoleDetailResp>>, AnyBody>(resp).await.body.unwrap();
    assert_eq!(body[0].rel_role_id, role_id);
    assert_eq!(body[0].rel_account_id, account_id);
    assert_eq!(body[0].create_user, "平台管理员");
    assert_eq!(body[0].update_user, "平台管理员");

    // Delete AccountRole
    let req = test::TestRequest::delete()
        .insert_header(test_basic::context_account())
        .uri(format!("/console/app/account/{}/role/{}", account_id.clone(), role_id.clone()).as_str())
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let result = read_body_json::<BIOSResp<String>, AnyBody>(resp).await;
    assert_eq!(result.code, "200");

    Ok(())
}

#[actix_rt::test]
async fn test_account_group() -> BIOSResult<()> {
    let docker = clients::Cli::default();
    let _c = test_basic::init(&docker).await;
    let app = test::init_service(
        App::new()
            .wrap(BIOSWebServer::init_cors(&FrameworkConfig::default()))
            .wrap(BIOSWebServer::init_error_handlers())
            .service(tenant_console::tc_account_processor::add_account)
            .service(app_console::ac_role_processor::add_role)
            .service(app_console::ac_group_processor::add_group)
            .service(app_console::ac_group_processor::add_group_node)
            .service(app_console::ac_account_processor::add_account_group)
            .service(app_console::ac_account_processor::list_account_group)
            .service(app_console::ac_account_processor::delete_account_group),
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

    // Add Group
    let req = test::TestRequest::post()
        .insert_header(test_basic::context_account())
        .uri("/console/app/group")
        .set_json(&GroupAddReq {
            code: "g001".to_string(),
            name: "测试群组".to_string(),
            kind: GroupKind::Administration,
            sort: 0,
            icon: None,
            rel_group_id: None,
            rel_group_node_id: None,
            expose_kind: None,
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let group_id = read_body_json::<BIOSResp<String>, AnyBody>(resp).await.body.unwrap();

    // Add GroupNode
    let req = test::TestRequest::post()
        .insert_header(test_basic::context_account())
        .uri(format!("/console/app/group/{}/node", group_id.clone()).as_str())
        .set_json(&GroupNodeAddReq {
            bus_code: None,
            name: "xx公司".to_string(),
            parameters: None,
            parent_code: "".to_string(),
            sort: 0,
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let group_node_id = read_body_json::<BIOSResp<Value>, AnyBody>(resp).await.body.unwrap();
    let group_node_id = group_node_id["id"].as_str().unwrap();

    // Add AccountGroup
    let req = test::TestRequest::post()
        .insert_header(test_basic::context_account())
        .uri(format!("/console/app/account/{}/group/{}", account_id.clone(), group_node_id.clone()).as_str())
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // List AccountGroup
    let req = test::TestRequest::get().insert_header(test_basic::context_account()).uri(format!("/console/app/account/{}/group", account_id.clone()).as_str()).to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_body_json::<BIOSResp<Vec<AccountGroupDetailResp>>, AnyBody>(resp).await.body.unwrap();
    assert_eq!(body[0].rel_group_node_id, group_node_id);
    assert_eq!(body[0].rel_account_id, account_id);
    assert_eq!(body[0].create_user, "平台管理员");
    assert_eq!(body[0].update_user, "平台管理员");

    // Delete AccountGroup
    let req = test::TestRequest::delete()
        .insert_header(test_basic::context_account())
        .uri(format!("/console/app/account/{}/group/{}", account_id.clone(), group_node_id.clone()).as_str())
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let result = read_body_json::<BIOSResp<String>, AnyBody>(resp).await;
    assert_eq!(result.code, "200");

    Ok(())
}
