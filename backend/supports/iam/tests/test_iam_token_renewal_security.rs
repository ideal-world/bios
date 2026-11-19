//! 续期兑换/吊销接口必须先校验服务凭据，不能把用户身份当作服务鉴权；
//! capability 只在注册响应出现一次，兑换限流按单授权计数。
use bios_basic::rbum::rbum_initializer::get_first_account_context;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use bios_basic::test::init_test_container;
use bios_iam::basic::domain::iam_token_renewal;
use bios_iam::basic::dto::iam_account_dto::IamAccountAddReq;
use bios_iam::basic::dto::iam_app_dto::IamAppAggAddReq;
use bios_iam::basic::dto::iam_role_dto::{IamRoleAddReq, IamRoleAggAddReq};
use bios_iam::basic::dto::iam_tenant_dto::IamTenantAggAddReq;
use bios_iam::basic::dto::iam_token_renewal_dto::IamTokenRenewalRegisterReq;
use bios_iam::basic::serv::iam_account_serv::IamAccountServ;
use bios_iam::basic::serv::iam_app_serv::IamAppServ;
use bios_iam::basic::serv::iam_key_cache_serv::IamIdentCacheServ;
use bios_iam::basic::serv::iam_rel_serv::IamRelServ;
use bios_iam::basic::serv::iam_role_serv::IamRoleServ;
use bios_iam::basic::serv::iam_tenant_serv::IamTenantServ;
use bios_iam::basic::serv::iam_token_renewal_serv::IamTokenRenewalServ;
use bios_iam::console_passport::api::iam_cp_token_renewal_api::IamTokenRenewalRuntimeApi;
use bios_iam::{iam_constants, iam_enumeration::{IamAccountStatusKind, IamRelKind}};
use tardis::basic::{dto::TardisContext, field::TrimString, result::TardisResult};
use tardis::db::sea_orm::EntityTrait;
use tardis::serde_json::{json, Value};
use tardis::tokio;
use tardis::web::poem::{http::Method, listener::TcpAcceptor, Body, Endpoint, EndpointExt, Request, Route, Server};
use tardis::web::poem_openapi::OpenApiService;
use tardis::web::uniform_error_mw::UniformError;
use tardis::web::web_resp::TardisResp;
use tardis::{TardisFuns, TardisFunsInst};

#[tokio::test]
#[ignore = "requires Docker for isolated PostgreSQL, Redis and RabbitMQ"]
async fn token_renewal_service_auth_limit_and_capability_hygiene() -> TardisResult<()> {
    Box::pin(token_renewal_service_auth_limit_fixture()).await
}

async fn token_renewal_service_auth_limit_fixture() -> TardisResult<()> {
    std::env::set_var("TARDIS_CSM.IAM.OAUTH2_TASK_SERVICE_TOKEN", "renewal-service-secret");
    std::env::set_var("BIOS_OAUTH2_TASK_SERVICE_TOKEN", "renewal-service-secret");
    // 单授权每小时兑换上限压到 2，便于触发限流分支。
    std::env::set_var("TARDIS_CSM.IAM.TOKEN_RENEWAL_EXCHANGE_HOURLY_LIMIT", "2");
    let _containers = Box::pin(init_test_container::init(None)).await?;
    let funs = iam_constants::get_tardis_inst();
    assert_eq!(funs.conf::<bios_iam::iam_config::IamConfig>().token_renewal_exchange_hourly_limit, 2);

    // 服务凭据守卫必须先于数据库访问：未建表时错误的服务 token 只能命中 401。
    let runtime_api = Route::new()
        .nest("/iam", OpenApiService::new(IamTokenRenewalRuntimeApi, "Token renewal runtime", "1"))
        .with(UniformError);
    for token in [None, Some("wrong"), Some("alice")] {
        for (method, uri) in [
            (Method::POST, "/iam/internal/token-renewals/credential/exchange"),
            (Method::DELETE, "/iam/internal/token-renewals/credential"),
        ] {
            let mut req = Request::builder().method(method.clone()).uri(uri.parse().unwrap()).header("Content-Type", "application/json");
            if let Some(token) = token {
                req = req.header("X-Bios-Task-Service-Token", token);
            }
            let response = runtime_api
                .get_response(
                    req.body(Body::from_json(&json!({ "run_id": "wfrun-guard", "capability": "capability" })).unwrap()),
                )
                .await;
            let body: Value = response.into_body().into_json().await.unwrap();
            assert!(body["code"].as_str().unwrap().starts_with("401"), "{body}");
        }
    }

    Box::pin(bios_iam::iam_initializer::init_db(iam_constants::get_tardis_inst())).await?;
    let system_ctx = Box::pin(get_first_account_context(iam_constants::RBUM_KIND_CODE_IAM_ACCOUNT, iam_constants::COMPONENT_CODE, &funs)).await?.unwrap();
    let fixture = Box::pin(setup_renewal_fixture(&funs, &system_ctx)).await?;

    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let address = listener.local_addr().unwrap();
    let acceptor = TcpAcceptor::from_tokio(listener).unwrap();
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    let api_task = tokio::spawn(async move {
        Server::new_with_acceptor(acceptor)
            .run_with_graceful_shutdown(
                Route::new().nest("/iam", OpenApiService::new(IamTokenRenewalRuntimeApi, "Token renewal runtime", "1")).with(UniformError),
                async move {
                    let _ = shutdown_rx.await;
                },
                Some(std::time::Duration::from_secs(1)),
            )
            .await
            .unwrap();
    });

    // HTTP 兑换成功两次后，第三次落入限流拒绝码；限流不吊销授权。
    let resp = Box::pin(register_renewal(renewal_register_req("wfrun-limit", "Bios-Token", Some(3600)), fixture.user_ctx.clone())).await?;
    let limit_key = format!("iam:renewal-exchange-count:{}", resp.credential_id);
    for attempt in 1..=3 {
        let (http_code, body) = renewal_exchange_http(address, &resp.credential_id, "wfrun-limit", &resp.capability, Some("renewal-service-secret")).await?;
        let code = body_code(&body);
        if attempt <= 2 {
            assert_eq!(http_code, 200, "{body}");
            assert!(code.starts_with("200"), "{body}");
        } else {
            // BIOS UniformError 把 429 映射回 HTTP 200、业务码保留在信封 code（照 task grant 测试的断言约定）。
            assert!(http_code == 429 || code.starts_with("429-token-renewal-exchange-limit"), "http_code={http_code}, {body}");
        }
    }
    // 限流计数是带 TTL 的一小时窗口计数。
    assert!(funs.cache().ttl(&limit_key).await? > 0);
    let limited_row = iam_token_renewal::Entity::find_by_id(&resp.credential_id).one(funs.db().raw_conn()).await?.unwrap();
    assert!(!limited_row.revoked, "the exchange limit must reject without revoking the authorization");

    // 绑定不匹配的兑换不消耗限流预算：三次错误 capability 全部 mismatch，随后两次合法兑换仍可成功。
    let mismatch_resp = Box::pin(register_renewal(renewal_register_req("wfrun-mismatch", "Bios-Token", Some(3600)), fixture.user_ctx.clone())).await?;
    for _ in 0..3 {
        let (http_code, body) = renewal_exchange_http(address, &mismatch_resp.credential_id, "wfrun-mismatch", "wrong-capability", Some("renewal-service-secret")).await?;
        assert!(http_code == 409 || body_code(&body).starts_with("409-token-renewal-principal-mismatch"), "http_code={http_code}, {body}");
    }
    let first_ok = renewal_exchange_http(address, &mismatch_resp.credential_id, "wfrun-mismatch", &mismatch_resp.capability, Some("renewal-service-secret")).await?;
    assert_eq!(first_ok.0, 200, "{:?}", first_ok.1);
    assert!(body_code(&first_ok.1).starts_with("200"), "{:?}", first_ok.1);
    let second_ok = renewal_exchange_http(address, &mismatch_resp.credential_id, "wfrun-mismatch", &mismatch_resp.capability, Some("renewal-service-secret")).await?;
    assert_eq!(second_ok.0, 200, "{:?}", second_ok.1);
    assert!(body_code(&second_ok.1).starts_with("200"), "{:?}", second_ok.1);
    let third_limited = renewal_exchange_http(address, &mismatch_resp.credential_id, "wfrun-mismatch", &mismatch_resp.capability, Some("renewal-service-secret")).await?;
    assert!(third_limited.0 == 429 || body_code(&third_limited.1).starts_with("429-token-renewal-exchange-limit"), "http_code={}, {:?}", third_limited.0, third_limited.1);

    // 兑换响应只包含 access_token / expires_at，绝不回显 capability。
    let data = second_ok.1.get("data").cloned().unwrap_or(Value::Null);
    let data_keys = data.as_object().map(|object| object.keys().cloned().collect::<Vec<_>>()).unwrap_or_default();
    assert_eq!(data_keys, vec!["access_token".to_string(), "expires_at".to_string()], "the exchange response must not echo the capability");

    // 服务端持久化只含哈希：库里没有任何 capability 明文。
    let stored = iam_token_renewal::Entity::find_by_id(&resp.credential_id).one(funs.db().raw_conn()).await?.unwrap();
    assert_eq!(stored.capability_hash, TardisFuns::crypto.digest.sha256(resp.capability.as_bytes())?);
    assert_ne!(stored.capability_hash, resp.capability);
    // 新 token 走 TokenTask 解析路径，能力与 task grant 一致。
    let access_token = data.get("access_token").and_then(Value::as_str).unwrap_or_default().to_string();
    let resolved = IamIdentCacheServ::get_context(
        &bios_iam::basic::dto::iam_cert_dto::IamContextFetchReq {
            token: access_token.clone(),
            app_id: Some(fixture.app_id.clone()),
        },
        &funs,
    )
    .await?;
    assert_eq!(resolved.owner, fixture.user_ctx.owner);

    let _ = shutdown_tx.send(());
    api_task.await.unwrap();
    Ok(())
}

struct RenewalFixture {
    user_ctx: TardisContext,
    app_id: String,
}

async fn setup_renewal_fixture(funs: &TardisFunsInst, system_ctx: &TardisContext) -> TardisResult<RenewalFixture> {
    let account_id = Box::pin(IamAccountServ::add_item(
        &mut TardisFuns::json.json_to_obj::<IamAccountAddReq>(json!({
            "name": "Renewal security user", "scope_level": iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
            "status": IamAccountStatusKind::Active
        }))?,
        funs,
        system_ctx,
    ))
    .await?;
    let (tenant_id, _, _) = Box::pin(IamTenantServ::add_tenant_agg(
        &IamTenantAggAddReq {
            name: TrimString::from("Token renewal security tenant"),
            icon: None,
            contact_phone: None,
            note: None,
            account_self_reg: None,
            disabled: None,
            admin_username: TrimString::from("token-renewal-security-admin"),
            admin_password: None,
            admin_phone: None,
            admin_mail: None,
            admin_name: TrimString::from("Token renewal security admin"),
            audit_username: TrimString::from("token-renewal-security-audit"),
            audit_password: None,
            audit_phone: None,
            audit_mail: None,
            audit_name: TrimString::from("Token renewal security auditor"),
            cert_conf_by_oauth2: None,
            cert_conf_by_ldap: None,
        },
        funs,
        system_ctx,
    ))
    .await?;
    let tenant_ctx = TardisContext {
        own_paths: tenant_id,
        owner: account_id.clone(),
        ..Default::default()
    };
    let app_id = Box::pin(IamAppServ::add_app_agg(
        &IamAppAggAddReq {
            app_name: TrimString::from("Renewal security app"),
            app_description: None,
            app_icon: None,
            app_sort: None,
            app_contact_phone: None,
            admin_ids: None,
            publish_system_ids: None,
            disabled: None,
            set_cate_id: None,
            kind: None,
            sync_apps_group: Some(false),
        },
        funs,
        &tenant_ctx,
    ))
    .await?;
    let user_ctx = TardisContext {
        own_paths: format!("{}/{app_id}", tenant_ctx.own_paths),
        owner: account_id.clone(),
        ..Default::default()
    };
    let tenant_role_id = Box::pin(IamRoleServ::tenant_add_app_role_agg(
        &mut IamRoleAggAddReq {
            role: IamRoleAddReq {
                id: None,
                code: None,
                name: TrimString::from("Renewal security user"),
                perm_kind: None,
                icon: None,
                sort: None,
                kind: Some(bios_iam::iam_enumeration::IamRoleKind::App),
                scope_level: Some(iam_constants::RBUM_SCOPE_LEVEL_APP),
                in_embed: None,
                in_base: None,
                extend_role_id: None,
                disabled: None,
                deletable: None,
            },
            res_ids: None,
        },
        funs,
        &tenant_ctx,
    ))
    .await?;
    let role_id = Box::pin(IamRoleServ::get_embed_sub_role_id(&tenant_role_id, funs, &user_ctx)).await?;
    IamAppServ::add_rel_account(&app_id, &user_ctx.owner, false, funs, &user_ctx).await?;
    IamRelServ::add_simple_rel(&IamRelKind::IamAccountRole, &user_ctx.owner, &role_id, None, None, true, false, funs, &user_ctx).await?;
    Ok(RenewalFixture { user_ctx, app_id })
}

fn renewal_register_req(run_id: &str, secret_key: &str, ttl_sec: Option<u64>) -> IamTokenRenewalRegisterReq {
    IamTokenRenewalRegisterReq {
        run_id: run_id.to_string(),
        secret_key: secret_key.to_string(),
        ttl_sec,
    }
}

async fn register_renewal(req: IamTokenRenewalRegisterReq, ctx: TardisContext) -> TardisResult<bios_iam::basic::dto::iam_token_renewal_dto::IamTokenRenewalRegisterResp> {
    let mut tx_funs = iam_constants::get_tardis_inst();
    tx_funs.begin().await?;
    match Box::pin(IamTokenRenewalServ::register(&req, &tx_funs, &ctx)).await {
        Ok(resp) => {
            tx_funs.commit().await?;
            ctx.execute_task().await?;
            Ok(resp)
        }
        Err(error) => {
            tx_funs.rollback().await?;
            Err(error)
        }
    }
}

async fn renewal_exchange_http(
    address: std::net::SocketAddr,
    credential_id: &str,
    run_id: &str,
    capability: &str,
    service_token: Option<&str>,
) -> TardisResult<(u16, Value)> {
    let headers = service_token.map(|token| vec![("X-Bios-Task-Service-Token".to_string(), token.to_string())]).unwrap_or_default();
    let funs = iam_constants::get_tardis_inst();
    let response = funs
        .web_client()
        .post::<_, TardisResp<Value>>(
            &format!("http://{address}/iam/internal/token-renewals/{credential_id}/exchange"),
            &json!({ "run_id": run_id, "capability": capability }),
            headers,
        )
        .await
        .map_err(|error| tardis::basic::error::TardisError::internal_error(&error.to_string(), "test-token-renewal-http"))?;
    let body = tardis::serde_json::to_value(response.body).map_err(|error| tardis::basic::error::TardisError::internal_error(&error.to_string(), "test-token-renewal-http-response"))?;
    Ok((response.code, body))
}

fn body_code(body: &Value) -> &str {
    body.as_object().and_then(|body| body.get("code")).and_then(Value::as_str).unwrap_or_default()
}
