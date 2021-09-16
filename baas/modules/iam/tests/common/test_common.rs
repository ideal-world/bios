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
use bios::basic::dto::IdentAccountInfo;
use bios::basic::error::BIOSResult;
use bios::web::resp_handler::BIOSRespHelper;
use bios::web::web_server::BIOSWebServer;
use bios::BIOSFuns;
use bios_baas_iam::iam_config::WorkSpaceConfig;
use bios_baas_iam::iam_initializer;
use bios_baas_iam::process::basic_dto::AccountIdentKind;
use bios_baas_iam::process::common;
use bios_baas_iam::process::common::com_account_dto::{AccountChangeReq, AccountIdentChangeReq, AccountInfoDetailResp, AccountLoginReq, AccountRegisterReq};
use bios_baas_iam::process::common::com_resource_dto::ResourceDetailResp;
use bios_baas_iam::process::common::com_tenant_dto::TenantRegisterReq;

#[actix_rt::test]
async fn test_flow() -> BIOSResult<()> {
    let docker = clients::Cli::default();
    let _c = crate::test_basic::init_without_data(&docker).await;
    iam_initializer::init().await?;
    let app = test::init_service(
        App::new()
            .wrap(BIOSWebServer::init_cors(&FrameworkConfig::default()))
            .wrap(BIOSWebServer::init_error_handlers())
            .service(common::common_processor::register_tenant)
            .service(common::common_processor::register_account)
            .service(common::common_processor::login)
            .service(common::common_processor::logout)
            .service(common::common_processor::fetch_login_info)
            .service(common::common_processor::change_account)
            .service(common::common_processor::change_account_ident)
            .service(common::common_processor::fetch_menu_resources)
            .service(common::common_processor::fetch_element_resources),
    )
    .await;

    // Register Tenant
    let req = test::TestRequest::post()
        .uri("/common/tenant")
        .set_json(&TenantRegisterReq {
            name: "测试租户".to_string(),
            icon: None,
            allow_account_register: true,
            parameters: None,
            app_name: "测试应用".to_string(),
            account_username: "gudaoxuri".to_string(),
            account_password: "123456".to_string(),
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let result = read_body_json::<BIOSRespHelper<String>, AnyBody>(resp).await;
    assert_eq!(result.code, "400");
    assert_eq!(result.msg, "AccountIdent [sk] invalid format");

    let req = test::TestRequest::post()
        .uri("/common/tenant")
        .set_json(&TenantRegisterReq {
            name: "测试租户".to_string(),
            icon: None,
            allow_account_register: true,
            parameters: None,
            app_name: "测试应用".to_string(),
            account_username: "gudaoxuri".to_string(),
            account_password: "83j#@$sS".to_string(),
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let ident_info = read_body_json::<BIOSRespHelper<IdentAccountInfo>, AnyBody>(resp).await.body.unwrap();
    assert!(!ident_info.tenant_id.is_empty());
    assert!(!ident_info.app_id.is_empty());
    assert!(!ident_info.account_id.is_empty());
    assert!(!ident_info.token.is_empty());
    assert_eq!(ident_info.roles.len(), 2);
    assert_eq!(ident_info.groups.len(), 0);

    // Register Account
    let req = test::TestRequest::post()
        .uri("/common/account")
        .set_json(&AccountRegisterReq {
            name: "孤岛旭日".to_string(),
            avatar: None,
            parameters: None,
            kind: AccountIdentKind::Username,
            ak: "gudaoxuri".to_string(),
            sk: "123456".to_string(),
            rel_app_id: ident_info.app_id.to_string(),
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let result = read_body_json::<BIOSRespHelper<String>, AnyBody>(resp).await;
    assert_eq!(result.code, "409");
    assert_eq!(result.msg, "AccountIdent [kind] and [ak] already exists");

    let req = test::TestRequest::post()
        .uri("/common/account")
        .set_json(&AccountRegisterReq {
            name: "孤岛旭日".to_string(),
            avatar: None,
            parameters: None,
            kind: AccountIdentKind::Username,
            ak: "gudaoxuri1".to_string(),
            sk: "123456".to_string(),
            rel_app_id: ident_info.app_id.to_string(),
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let result = read_body_json::<BIOSRespHelper<String>, AnyBody>(resp).await;
    assert_eq!(result.code, "400");
    assert_eq!(result.msg, "AccountIdent [sk] invalid format");

    let req = test::TestRequest::post()
        .uri("/common/account")
        .set_json(&AccountRegisterReq {
            name: "孤岛旭日".to_string(),
            avatar: None,
            parameters: None,
            kind: AccountIdentKind::Username,
            ak: "gudaoxuri1".to_string(),
            sk: "39d*32fSd".to_string(),
            rel_app_id: ident_info.app_id.to_string(),
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let ident_info = read_body_json::<BIOSRespHelper<IdentAccountInfo>, AnyBody>(resp).await.body.unwrap();
    assert!(!ident_info.tenant_id.is_empty());
    assert!(!ident_info.app_id.is_empty());
    assert!(!ident_info.account_id.is_empty());
    assert!(!ident_info.token.is_empty());
    assert_eq!(ident_info.roles.len(), 0);
    assert_eq!(ident_info.groups.len(), 0);

    // Login
    let req = test::TestRequest::post()
        .uri("/common/login")
        .set_json(&AccountLoginReq {
            kind: AccountIdentKind::Username,
            ak: "gudaoxuri2".to_string(),
            sk: "123456".to_string(),
            cert_category: None,
            rel_app_id: ident_info.app_id.to_string(),
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let result = read_body_json::<BIOSRespHelper<String>, AnyBody>(resp).await;
    assert_eq!(result.code, "404");
    assert_eq!(result.msg, "Account not exists");

    let req = test::TestRequest::post()
        .uri("/common/login")
        .set_json(&AccountLoginReq {
            kind: AccountIdentKind::Username,
            ak: "gudaoxuri".to_string(),
            sk: "123456".to_string(),
            cert_category: None,
            rel_app_id: ident_info.app_id.to_string(),
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let result = read_body_json::<BIOSRespHelper<String>, AnyBody>(resp).await;
    assert_eq!(result.code, "409");
    assert_eq!(result.msg, "Username or Password error");

    let req = test::TestRequest::post()
        .uri("/common/login")
        .set_json(&AccountLoginReq {
            kind: AccountIdentKind::Username,
            ak: "gudaoxuri".to_string(),
            sk: "83j#@$sS".to_string(),
            cert_category: None,
            rel_app_id: ident_info.app_id.to_string(),
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let ident_info = read_body_json::<BIOSRespHelper<IdentAccountInfo>, AnyBody>(resp).await.body.unwrap();
    assert!(!ident_info.tenant_id.is_empty());
    assert!(!ident_info.app_id.is_empty());
    assert!(!ident_info.account_id.is_empty());
    assert!(!ident_info.token.is_empty());
    assert_eq!(ident_info.roles.len(), 2);
    assert_eq!(ident_info.groups.len(), 0);

    let token_rels = BIOSFuns::cache().hgetall(format!("{}{}", BIOSFuns::ws_config::<WorkSpaceConfig>().iam.cache.token_rel, ident_info.account_id).as_str()).await?;
    assert_eq!(token_rels.len(), 1);
    assert!(token_rels.contains_key(&ident_info.token));

    // Re-Login and check token version
    let req = test::TestRequest::post()
        .uri("/common/login")
        .set_json(&AccountLoginReq {
            kind: AccountIdentKind::Username,
            ak: "gudaoxuri".to_string(),
            sk: "83j#@$sS".to_string(),
            cert_category: None,
            rel_app_id: ident_info.app_id.to_string(),
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let ident_info = read_body_json::<BIOSRespHelper<IdentAccountInfo>, AnyBody>(resp).await.body.unwrap();
    assert!(!ident_info.tenant_id.is_empty());
    assert!(!ident_info.app_id.is_empty());
    assert!(!ident_info.account_id.is_empty());
    assert!(!ident_info.token.is_empty());
    assert_eq!(ident_info.roles.len(), 2);
    assert_eq!(ident_info.groups.len(), 0);

    let token_rels = BIOSFuns::cache().hgetall(format!("{}{}", BIOSFuns::ws_config::<WorkSpaceConfig>().iam.cache.token_rel, ident_info.account_id).as_str()).await?;
    assert_eq!(token_rels.len(), 1);
    assert!(token_rels.contains_key(&ident_info.token));

    let ident_info_in_header = bios::basic::security::digest::base64::encode(&bios::basic::json::obj_to_string(&ident_info)?);

    // Fetch login info
    let req = test::TestRequest::get().uri("/common/login").to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let result = read_body_json::<BIOSRespHelper<String>, AnyBody>(resp).await;
    assert_eq!(result.code, "401");
    assert_eq!(result.msg, "Ident Info doesn't exists");

    let req = test::TestRequest::get().insert_header((BIOSFuns::fw_config().web.ident_info_flag.clone(), ident_info_in_header.clone())).uri("/common/login").to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let ident_info = read_body_json::<BIOSRespHelper<AccountInfoDetailResp>, AnyBody>(resp).await.body.unwrap();
    assert!(!ident_info.tenant_id.is_empty());
    assert_eq!(ident_info.tenant_name, "测试租户");
    assert!(!ident_info.app_id.is_empty());
    assert_eq!(ident_info.app_name, "测试应用");
    assert!(!ident_info.account_id.is_empty());
    assert_eq!(ident_info.account_name, "测试租户管理员");
    assert!(!ident_info.token.is_empty());
    assert_eq!(ident_info.roles.len(), 2);
    assert!(ident_info.roles.iter().find(|(_, name)| name == "租户管理员").is_some());
    assert_eq!(ident_info.groups.len(), 0);

    // Change Account
    let req = test::TestRequest::put()
        .insert_header((BIOSFuns::fw_config().web.ident_info_flag.clone(), ident_info_in_header.clone()))
        .uri("/common/account")
        .set_json(&AccountChangeReq {
            name: Some("理想世界".to_string()),
            avatar: None,
            parameters: None,
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let result = read_body_json::<BIOSRespHelper<String>, AnyBody>(resp).await;
    assert_eq!(result.code, "200");

    // Change AccountIdent
    let req = test::TestRequest::put()
        .insert_header((BIOSFuns::fw_config().web.ident_info_flag.clone(), ident_info_in_header.clone()))
        .uri("/common/account/ident")
        .set_json(&AccountIdentChangeReq {
            kind: AccountIdentKind::Username,
            ak: "gudaoxuri1".to_string(),
            sk: "1234".to_string(),
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let result = read_body_json::<BIOSRespHelper<String>, AnyBody>(resp).await;
    assert_eq!(result.msg, "AccountIdent [kind] and [ak] already exists");

    let req = test::TestRequest::put()
        .insert_header((BIOSFuns::fw_config().web.ident_info_flag.clone(), ident_info_in_header.clone()))
        .uri("/common/account/ident")
        .set_json(&AccountIdentChangeReq {
            kind: AccountIdentKind::Username,
            ak: "gudaoxuri2".to_string(),
            sk: "1234".to_string(),
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let result = read_body_json::<BIOSRespHelper<String>, AnyBody>(resp).await;
    assert_eq!(result.msg, "AccountIdent [sk] invalid format");

    let req = test::TestRequest::put()
        .insert_header((BIOSFuns::fw_config().web.ident_info_flag.clone(), ident_info_in_header.clone()))
        .uri("/common/account/ident")
        .set_json(&AccountIdentChangeReq {
            kind: AccountIdentKind::Username,
            ak: "gudaoxuri".to_string(),
            sk: "dfok&i34".to_string(),
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let result = read_body_json::<BIOSRespHelper<String>, AnyBody>(resp).await;
    assert_eq!(result.code, "200");

    // Re-Login
    let req = test::TestRequest::post()
        .uri("/common/login")
        .set_json(&AccountLoginReq {
            kind: AccountIdentKind::Username,
            ak: "gudaoxuri".to_string(),
            sk: "dfok&i34".to_string(),
            cert_category: None,
            rel_app_id: ident_info.app_id.to_string(),
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let ident_info = read_body_json::<BIOSRespHelper<IdentAccountInfo>, AnyBody>(resp).await.body.unwrap();

    let token_rels = BIOSFuns::cache().hgetall(format!("{}{}", BIOSFuns::ws_config::<WorkSpaceConfig>().iam.cache.token_rel, ident_info.account_id).as_str()).await?;
    assert_eq!(token_rels.len(), 1);
    assert!(token_rels.contains_key(&ident_info.token));

    let ident_info_in_header = bios::basic::security::digest::base64::encode(&bios::basic::json::obj_to_string(&ident_info)?);

    // Fetch menu
    let req = test::TestRequest::get().insert_header((BIOSFuns::fw_config().web.ident_info_flag.clone(), ident_info_in_header.clone())).uri("/common/resource/menu").to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let result = read_body_json::<BIOSRespHelper<Vec<ResourceDetailResp>>, AnyBody>(resp).await.body.unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "测试应用菜单-租户共享菜单");
    assert_eq!(result[0].uri, format!("menu://{}/pub/**", ident_info.app_id));

    // Fetch element
    let req =
        test::TestRequest::get().insert_header((BIOSFuns::fw_config().web.ident_info_flag.clone(), ident_info_in_header.clone())).uri("/common/resource/element").to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let result = read_body_json::<BIOSRespHelper<Vec<ResourceDetailResp>>, AnyBody>(resp).await.body.unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "测试应用元素-租户共享元素");
    assert_eq!(result[0].uri, format!("element://{}/pub/**", ident_info.app_id));

    // Logout
    let req = test::TestRequest::delete().insert_header((BIOSFuns::fw_config().web.ident_info_flag.clone(), ident_info_in_header.clone())).uri("/common/logout").to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let result = read_body_json::<BIOSRespHelper<String>, AnyBody>(resp).await;
    assert_eq!(result.code, "200");

    Ok(())
}
