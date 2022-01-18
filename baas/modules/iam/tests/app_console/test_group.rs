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
use bios::db::reldb_client::BIOSPage;
use bios::web::web_server::BIOSWebServer;
use bios_baas_iam::process::app_console;
use bios_baas_iam::process::app_console::ac_group_dto::{
    GroupAddReq, GroupDetailResp, GroupModifyReq, GroupNodeAddReq, GroupNodeDetailResp, GroupNodeModifyReq, GroupNodeOverviewResp,
};
use bios_baas_iam::process::basic_dto::GroupKind;

#[actix_rt::test]
async fn test_group() -> BIOSResult<()> {
    let docker = clients::Cli::default();
    let _c = test_basic::init(&docker).await;
    let app = test::init_service(
        App::new()
            .wrap(BIOSWebServer::init_cors(&FrameworkConfig::default()))
            .wrap(BIOSWebServer::init_error_handlers())
            .service(app_console::ac_group_processor::add_group)
            .service(app_console::ac_group_processor::modify_group)
            .service(app_console::ac_group_processor::list_group)
            .service(app_console::ac_group_processor::delete_group),
    )
    .await;

    // Add Group
    let req = test::TestRequest::post()
        .insert_header(test_basic::context_account())
        .uri("/console/app/group")
        .set_json(&GroupAddReq {
            code: " g001".to_string(),
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
    let result = read_body_json::<BIOSResp<String>, AnyBody>(resp).await;
    assert_eq!(result.code, "400000000000");

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
    let id = read_body_json::<BIOSResp<String>, AnyBody>(resp).await.body.unwrap();

    // Modify Group
    let req = test::TestRequest::put()
        .insert_header(test_basic::context_account())
        .uri(format!("/console/app/group/{}", id.clone()).as_str())
        .set_json(&GroupModifyReq {
            name: Some("测试行政树".to_string()),
            kind: None,
            sort: None,
            icon: None,
            rel_group_id: None,
            rel_group_node_id: None,
            expose_kind: None,
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let result = read_body_json::<BIOSResp<String>, AnyBody>(resp).await;
    assert_eq!(result.code, "200");

    // List Group
    let req = test::TestRequest::get().insert_header(test_basic::context_account()).uri("/console/app/group?expose=false&page_number=1&page_size=10").to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_body_json::<BIOSResp<BIOSPage<GroupDetailResp>>, AnyBody>(resp).await.body.unwrap();
    assert_eq!(body.total_size, 1);
    assert_eq!(body.records[0].code, "g001");
    assert_eq!(body.records[0].name, "测试行政树");
    assert_eq!(body.records[0].create_user, "平台管理员");
    assert_eq!(body.records[0].update_user, "平台管理员");

    // Delete Group
    let req = test::TestRequest::delete().insert_header(test_basic::context_account()).uri(format!("/console/app/group/{}", id.clone()).as_str()).to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let result = read_body_json::<BIOSResp<String>, AnyBody>(resp).await;
    assert_eq!(result.code, "200");

    Ok(())
}

#[actix_rt::test]
async fn test_group_node() -> BIOSResult<()> {
    let docker = clients::Cli::default();
    let _c = test_basic::init(&docker).await;
    let app = test::init_service(
        App::new()
            .wrap(BIOSWebServer::init_cors(&FrameworkConfig::default()))
            .wrap(BIOSWebServer::init_error_handlers())
            .service(app_console::ac_group_processor::add_group)
            .service(app_console::ac_group_processor::add_group_node)
            .service(app_console::ac_group_processor::modify_group_node)
            .service(app_console::ac_group_processor::list_group_node)
            .service(app_console::ac_group_processor::delete_group_node),
    )
    .await;

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

    let req = test::TestRequest::post()
        .insert_header(test_basic::context_account())
        .uri(format!("/console/app/group/{}/node", group_id.clone()).as_str())
        .set_json(&GroupNodeAddReq {
            bus_code: None,
            name: "yy公司".to_string(),
            parameters: None,
            parent_code: "".to_string(),
            sort: 0,
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let id_yy = read_body_json::<BIOSResp<GroupNodeOverviewResp>, AnyBody>(resp).await.body.unwrap().id;

    // Modify GroupNode
    let req = test::TestRequest::put()
        .insert_header(test_basic::context_account())
        .uri(format!("/console/app/group/{}/node/{}", group_id.clone(), id_yy.clone()).as_str())
        .set_json(&GroupNodeModifyReq {
            bus_code: Some("b001".to_string()),
            name: None,
            parameters: None,
            sort: None,
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let result = read_body_json::<BIOSResp<String>, AnyBody>(resp).await;
    assert_eq!(result.code, "200");

    // List GroupNode
    let req = test::TestRequest::get().insert_header(test_basic::context_account()).uri(format!("/console/app/group/{}/node", group_id.clone()).as_str()).to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_body_json::<BIOSResp<Vec<GroupNodeDetailResp>>, AnyBody>(resp).await.body.unwrap();
    assert!(body[0].code == "aaab" || body[1].code == "aaab");

    // Delete GroupNode
    let req = test::TestRequest::delete()
        .insert_header(test_basic::context_account())
        .uri(format!("/console/app/group/{}/node/{}", group_id.clone(), id_yy.clone()).as_str())
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let result = read_body_json::<BIOSResp<String>, AnyBody>(resp).await;
    assert_eq!(result.code, "200");

    // Test GroupNode Code
    let req = test::TestRequest::post()
        .insert_header(test_basic::context_account())
        .uri(format!("/console/app/group/{}/node", group_id.clone()).as_str())
        .set_json(&GroupNodeAddReq {
            bus_code: None,
            name: "yy公司aa子公司".to_string(),
            parameters: None,
            parent_code: "aaaa".to_string(),
            sort: 0,
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let req = test::TestRequest::post()
        .insert_header(test_basic::context_account())
        .uri(format!("/console/app/group/{}/node", group_id.clone()).as_str())
        .set_json(&GroupNodeAddReq {
            bus_code: None,
            name: "yy公司bb子公司".to_string(),
            parameters: None,
            parent_code: "aaaa".to_string(),
            sort: 0,
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let req = test::TestRequest::post()
        .insert_header(test_basic::context_account())
        .uri(format!("/console/app/group/{}/node", group_id.clone()).as_str())
        .set_json(&GroupNodeAddReq {
            bus_code: None,
            name: "yy公司bb子公司00部门".to_string(),
            parameters: None,
            parent_code: "aaaa.aaab".to_string(),
            sort: 0,
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let code = read_body_json::<BIOSResp<GroupNodeOverviewResp>, AnyBody>(resp).await.body.unwrap().code;
    assert_eq!(code, "aaaa.aaab.aaaa");

    Ok(())
}
