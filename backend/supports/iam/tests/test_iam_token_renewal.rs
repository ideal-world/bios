use std::time::Duration;

use bios_basic::rbum::helper::rbum_event_helper;
use bios_basic::rbum::rbum_initializer::get_first_account_context;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use bios_basic::test::init_test_container;
use bios_iam::basic::domain::iam_token_renewal;
use bios_iam::basic::dto::iam_account_dto::IamAccountAddReq;
use bios_iam::basic::dto::iam_app_dto::IamAppAggAddReq;
use bios_iam::basic::dto::iam_cert_dto::IamContextFetchReq;
use bios_iam::basic::dto::iam_role_dto::{IamRoleAddReq, IamRoleAggAddReq};
use bios_iam::basic::dto::iam_tenant_dto::IamTenantAggAddReq;
use bios_iam::basic::dto::iam_token_renewal_dto::{IamTokenRenewalExchangeReq, IamTokenRenewalRegisterReq, IamTokenRenewalRevokeReq};
use bios_iam::basic::serv::iam_account_serv::IamAccountServ;
use bios_iam::basic::serv::iam_cert_serv::IamCertServ;
use bios_iam::basic::serv::iam_cert_user_pwd_serv::IamCertUserPwdServ;
use bios_iam::basic::serv::iam_app_serv::IamAppServ;
use bios_iam::basic::serv::iam_key_cache_serv::IamIdentCacheServ;
use bios_iam::basic::serv::iam_rel_serv::IamRelServ;
use bios_iam::basic::serv::iam_role_serv::IamRoleServ;
use bios_iam::basic::serv::iam_tenant_serv::IamTenantServ;
use bios_iam::basic::serv::iam_token_renewal_serv::IamTokenRenewalServ;
use bios_iam::console_passport::api::iam_cp_token_renewal_api::{IamCpTokenRenewalApi, IamTokenRenewalRuntimeApi};
use bios_iam::{iam_constants, iam_enumeration::{IamAccountLockStateKind, IamRelKind}};
use tardis::basic::{dto::TardisContext, field::TrimString, result::TardisResult};
use tardis::chrono::{DateTime, Utc};
use tardis::db::sea_orm::EntityTrait;
use tardis::serde_json::{json, Value};
use tardis::tokio;
use tardis::web::poem::{listener::TcpAcceptor, EndpointExt, Route, Server};
use tardis::web::poem_openapi::OpenApiService;
use tardis::web::uniform_error_mw::UniformError;
use tardis::web::web_resp::TardisResp;
use tardis::{TardisFuns, TardisFunsInst};

mod common {
    pub mod token_renewal_smoke_harness;
}

fn renewal_register_req(run_id: &str, secret_key: &str, ttl_sec: Option<u64>) -> IamTokenRenewalRegisterReq {
    IamTokenRenewalRegisterReq {
        run_id: run_id.to_string(),
        secret_key: secret_key.to_string(),
        ttl_sec,
    }
}

fn renewal_exchange_req(run_id: &str, capability: &str) -> IamTokenRenewalExchangeReq {
    IamTokenRenewalExchangeReq {
        run_id: run_id.to_string(),
        capability: capability.to_string(),
    }
}

fn renewal_revoke_req(run_id: &str, capability: &str) -> IamTokenRenewalRevokeReq {
    IamTokenRenewalRevokeReq {
        run_id: run_id.to_string(),
        capability: capability.to_string(),
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

async fn exchange_renewal(
    credential_id: &str,
    req: &IamTokenRenewalExchangeReq,
    funs: &TardisFunsInst,
) -> TardisResult<bios_iam::basic::dto::iam_token_renewal_dto::IamTokenRenewalExchangeResp> {
    Box::pin(IamTokenRenewalServ::exchange(credential_id, req, funs)).await
}

async fn revoke_renewal(credential_id: &str, req: &IamTokenRenewalRevokeReq, funs: &TardisFunsInst) -> TardisResult<()> {
    Box::pin(IamTokenRenewalServ::revoke(credential_id, req, funs)).await
}

struct TokenRenewalFixture {
    system_ctx: TardisContext,
    user_ctx: TardisContext,
    tenant_user_ctx: TardisContext,
    app_id: String,
    role_id: String,
    login_username: String,
    login_password: String,
}

#[tokio::test]
#[ignore = "requires Docker for isolated PostgreSQL, Redis and RabbitMQ"]
async fn token_renewal_registration_exchange_and_revocation_lifecycle() -> TardisResult<()> {
    Box::pin(token_renewal_lifecycle_fixture()).await
}

async fn token_renewal_lifecycle_fixture() -> TardisResult<()> {
    std::env::set_var("TARDIS_CSM.IAM.OAUTH2_TASK_SERVICE_TOKEN", "scheduler");
    std::env::set_var("BIOS_OAUTH2_TASK_SERVICE_TOKEN", "scheduler");
    let _containers = Box::pin(init_test_container::init(None)).await?;
    let funs = iam_constants::get_tardis_inst();
    Box::pin(bios_iam::iam_initializer::init_db(iam_constants::get_tardis_inst())).await?;
    let config = funs.conf::<bios_iam::iam_config::IamConfig>();
    assert_eq!(config.token_renewal_ttl_cap_sec, 7 * 24 * 3600);
    assert_eq!(config.token_renewal_mint_ttl_sec, 3600);
    assert_eq!(config.token_renewal_exchange_hourly_limit, 240);
    let (runtime_api_address, runtime_api_shutdown, runtime_api_task) = start_renewal_runtime_api().await;
    let (cp_api_address, cp_api_shutdown, cp_api_task) = start_renewal_cp_api().await;
    // 服务 token 缺失或错误必须先于数据库访问被拒绝，与 task runtime 端点同一守卫。
    for service_token in [None, Some("wrong"), Some("alice")] {
        let (http_code, body) = renewal_exchange_http(runtime_api_address, "missing-credential", "wfrun-missing", "missing-capability", service_token)
            .await
            .unwrap();
        assert!(http_code == 401 || body_code(&body).starts_with("401"), "{body}");
    }
    for service_token in [None, Some("wrong")] {
        let (http_code, body) = renewal_revoke_http(runtime_api_address, "missing-credential", "wfrun-missing", "missing-capability", service_token).await.unwrap();
        assert!(http_code == 401 || body_code(&body).starts_with("401"), "{body}");
    }
    let fixture = Box::pin(setup_token_renewal_fixture(&funs)).await?;
    // 可选联调挂钩：设置 BIOS_WF_RENEWAL_SMOKE_CREDENTIALS 后停留在真实 HTTP fixture，
    // 默认（未设置）完全不改变测试行为并立即退出。
    Box::pin(common::token_renewal_smoke_harness::serve_if_requested(&fixture)).await?;
    Box::pin(verify_registration_binds_principal_and_stores_only_hash(&funs, &fixture)).await?;
    Box::pin(verify_replacement_revokes_previous_capability(&funs, &fixture)).await?;
    Box::pin(verify_exchange_mints_resolvable_task_token(&funs, &fixture, runtime_api_address, cp_api_address)).await?;
    Box::pin(verify_principal_revalidation_rejections(&funs, &fixture)).await?;
    Box::pin(verify_absolute_expiry(&funs, &fixture)).await?;
    Box::pin(verify_revocation(&funs, &fixture, runtime_api_address)).await?;
    let _ = runtime_api_shutdown.send(());
    runtime_api_task.await.expect("token renewal runtime API server task");
    let _ = cp_api_shutdown.send(());
    cp_api_task.await.expect("token renewal CP API server task");
    Ok(())
}

async fn start_renewal_runtime_api() -> (std::net::SocketAddr, tokio::sync::oneshot::Sender<()>, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let address = listener.local_addr().unwrap();
    let acceptor = TcpAcceptor::from_tokio(listener).unwrap();
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    let api = Route::new()
        .nest("/iam", OpenApiService::new(IamTokenRenewalRuntimeApi, "Token renewal runtime", "1"))
        .with(UniformError);
    let task = tokio::spawn(async move {
        Server::new_with_acceptor(acceptor)
            .run_with_graceful_shutdown(
                api,
                async move {
                    let _ = shutdown_rx.await;
                },
                Some(Duration::from_secs(1)),
            )
            .await
            .unwrap();
    });
    (address, shutdown_tx, task)
}

async fn start_renewal_cp_api() -> (std::net::SocketAddr, tokio::sync::oneshot::Sender<()>, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let address = listener.local_addr().unwrap();
    let acceptor = TcpAcceptor::from_tokio(listener).unwrap();
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    let api = Route::new()
        .nest("/iam", OpenApiService::new(IamCpTokenRenewalApi, "Token renewal registration", "1"))
        .with(UniformError);
    let task = tokio::spawn(async move {
        Server::new_with_acceptor(acceptor)
            .run_with_graceful_shutdown(
                api,
                async move {
                    let _ = shutdown_rx.await;
                },
                Some(Duration::from_secs(1)),
            )
            .await
            .unwrap();
    });
    (address, shutdown_tx, task)
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

async fn renewal_revoke_http(
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
        .delete_with_body::<TardisResp<Value>, _>(
            &format!("http://{address}/iam/internal/token-renewals/{credential_id}"),
            headers,
            &json!({ "run_id": run_id, "capability": capability }),
        )
        .await
        .map_err(|error| tardis::basic::error::TardisError::internal_error(&error.to_string(), "test-token-renewal-http"))?;
    let body = tardis::serde_json::to_value(response.body).map_err(|error| tardis::basic::error::TardisError::internal_error(&error.to_string(), "test-token-renewal-http-response"))?;
    Ok((response.code, body))
}

async fn cp_register_http(address: std::net::SocketAddr, ctx: &TardisContext, run_id: &str, secret_key: &str, ttl_sec: u64) -> TardisResult<(u16, Value)> {
    // 照 iam_test_helper 的鉴权方式：网关在生产链路把 Bios-Token 解析为上下文后以
    // `Tardis-Context` 头转发，测试直接构造同样的上下文头。
    let context = TardisFuns::crypto.base64.encode(TardisFuns::json.obj_to_string(ctx)?);
    let context_header_name = TardisFuns::fw_config().web_server().context_conf.context_header_name.clone();
    let headers = vec![(context_header_name, context), ("X-Bios-Task-Service-Token".to_string(), "scheduler".to_string())];
    let funs = iam_constants::get_tardis_inst();
    let response = funs
        .web_client()
        .put::<_, TardisResp<Value>>(
            &format!("http://{address}/iam/cp/token-renewals"),
            &json!({ "run_id": run_id, "secret_key": secret_key, "ttl_sec": ttl_sec }),
            headers,
        )
        .await
        .map_err(|error| tardis::basic::error::TardisError::internal_error(&error.to_string(), "test-token-renewal-cp-http"))?;
    let body = tardis::serde_json::to_value(response.body).map_err(|error| tardis::basic::error::TardisError::internal_error(&error.to_string(), "test-token-renewal-cp-http-response"))?;
    Ok((response.code, body))
}

fn body_code(body: &Value) -> &str {
    body.as_object().and_then(|body| body.get("code")).and_then(Value::as_str).unwrap_or_default()
}

fn renewal_data(body: &Value) -> Value {
    body.get("data").cloned().unwrap_or(Value::Null)
}

async fn setup_token_renewal_fixture(funs: &TardisFunsInst) -> TardisResult<TokenRenewalFixture> {
    let system_ctx = Box::pin(get_first_account_context(iam_constants::RBUM_KIND_CODE_IAM_ACCOUNT, iam_constants::COMPONENT_CODE, funs)).await?.unwrap();
    let account_id = Box::pin(IamAccountServ::add_item(
        &mut TardisFuns::json.json_to_obj::<IamAccountAddReq>(json!({
            "name": "Renewal workflow user", "scope_level": iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
            "status": bios_iam::iam_enumeration::IamAccountStatusKind::Active
        }))?,
        funs,
        &system_ctx,
    ))
    .await?;
    let (tenant_id, _, _) = Box::pin(IamTenantServ::add_tenant_agg(
        &IamTenantAggAddReq {
            name: TrimString::from("Token renewal fixture tenant"),
            icon: None,
            contact_phone: None,
            note: None,
            account_self_reg: None,
            disabled: None,
            admin_username: TrimString::from("token-renewal-admin"),
            admin_password: None,
            admin_phone: None,
            admin_mail: None,
            admin_name: TrimString::from("Token renewal fixture admin"),
            audit_username: TrimString::from("token-renewal-audit"),
            audit_password: None,
            audit_phone: None,
            audit_mail: None,
            audit_name: TrimString::from("Token renewal fixture auditor"),
            cert_conf_by_oauth2: None,
            cert_conf_by_ldap: None,
        },
        funs,
        &system_ctx,
    ))
    .await?;
    let tenant_ctx = TardisContext {
        own_paths: tenant_id,
        owner: account_id.clone(),
        ..Default::default()
    };
    let app_id = Box::pin(IamAppServ::add_app_agg(
        &IamAppAggAddReq {
            app_name: TrimString::from("Renewal workflow"),
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
                name: TrimString::from("Renewal workflow user"),
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
    // 真实登录凭证：给 fixture 账号加 userpwd cert，供 HTTP 冒烟通过真实登录服务发行初始 token。
    let login_username = "wf-renewal-smoke-user".to_string();
    let login_password = TardisFuns::crypto.key.rand_64_hex();
    let cert_conf = IamCertServ::get_cert_conf_id_and_ext_opt_by_kind(
        &bios_iam::iam_enumeration::IamCertKernelKind::UserPwd.to_string(),
        Some(tenant_ctx.own_paths.clone()),
        funs,
    )
    .await?
    .ok_or_else(|| tardis::basic::error::TardisError::internal_error("userpwd cert conf missing in fixture tenant", "test-token-renewal-userpwd-conf"))?;
    IamCertUserPwdServ::add_cert(
        &bios_iam::basic::dto::iam_cert_dto::IamCertUserPwdAddReq {
            ak: TrimString::from(login_username.clone()),
            sk: TrimString::from(login_password.clone()),
            is_ignore_check_sk: true,
            status: None,
        },
        &account_id,
        Some(cert_conf.id),
        funs,
        &tenant_ctx,
    )
    .await?;
    let tenant_user_ctx = TardisContext {
        owner: user_ctx.owner.clone(),
        own_paths: tenant_ctx.own_paths.clone(),
        ..Default::default()
    };
    Ok(TokenRenewalFixture {
        system_ctx,
        user_ctx,
        tenant_user_ctx,
        app_id,
        role_id,
        login_username,
        login_password,
    })
}

async fn verify_registration_binds_principal_and_stores_only_hash(funs: &TardisFunsInst, fixture: &TokenRenewalFixture) -> TardisResult<()> {
    let user_ctx = &fixture.user_ctx;
    let resp = Box::pin(register_renewal(renewal_register_req("wfrun-main", "Bios-Token", Some(86400)), user_ctx.clone())).await?;
    assert!(!resp.credential_id.trim().is_empty());
    // capability 是 256-bit 随机值（64 位十六进制），且本次响应之外不再可见。
    assert_eq!(resp.capability.len(), 64);
    assert!(TardisFuns::crypto.hex.decode(&resp.capability).is_ok());
    let expires_at = DateTime::parse_from_rfc3339(&resp.expires_at).unwrap().with_timezone(&Utc);
    let delta = (expires_at - Utc::now()).num_seconds();
    assert!(delta > 86_000 && delta <= 86_400, "expires_at should follow the requested ttl, got {delta}s");
    let stored = iam_token_renewal::Entity::find_by_id(&resp.credential_id)
        .one(funs.db().raw_conn())
        .await?
        .ok_or_else(|| tardis::basic::error::TardisError::internal_error("credential row missing", "test-token-renewal-row"))?;
    assert_eq!(stored.run_id, "wfrun-main");
    assert_eq!(stored.secret_key, "Bios-Token");
    assert_eq!(stored.account_id, user_ctx.owner);
    assert_eq!(stored.own_paths, user_ctx.own_paths);
    assert_eq!(stored.app_id.as_deref(), Some(fixture.app_id.as_str()));
    assert!(!stored.revoked);
    assert_eq!(stored.current_run_id.as_deref(), Some("wfrun-main"));
    assert_eq!(stored.current_secret_key.as_deref(), Some("Bios-Token"));
    // 服务端只存 SHA-256 哈希，不存 capability 明文。
    assert_eq!(stored.capability_hash, TardisFuns::crypto.digest.sha256(resp.capability.as_bytes())?);
    assert_ne!(stored.capability_hash, resp.capability);
    let events = rbum_event_helper::get_notify_event_with_ctx(user_ctx).await?.unwrap();
    assert!(events.iter().any(|event| event.table_name == "iam_token_renewal" && event.operate == "c" && event.record_id == resp.credential_id));
    // ttl_sec 超过配置上限时按上限截断。
    let capped = Box::pin(register_renewal(renewal_register_req("wfrun-capped", "Bios-Token", Some(9_999_999_999)), user_ctx.clone())).await?;
    let capped_expires_at = DateTime::parse_from_rfc3339(&capped.expires_at).unwrap().with_timezone(&Utc);
    let capped_delta = (capped_expires_at - Utc::now()).num_seconds();
    assert!(capped_delta > 7 * 24 * 3600 - 600 && capped_delta <= 7 * 24 * 3600, "ttl must be capped at 7 days, got {capped_delta}s");
    // 无效请求直接拒绝。
    for req in [
        renewal_register_req("", "Bios-Token", Some(86400)),
        renewal_register_req("   ", "Bios-Token", None),
        renewal_register_req("wfrun-invalid", "", None),
    ] {
        assert_eq!(
            Box::pin(register_renewal(req, user_ctx.clone())).await.unwrap_err().code,
            "400-token-renewal-invalid-request"
        );
    }
    let personal = Box::pin(register_renewal(renewal_register_req("wfrun-global", "Bios-Token", Some(86400)), fixture.system_ctx.clone())).await?;
    let token = exchange_renewal(&personal.credential_id, &renewal_exchange_req("wfrun-global", &personal.capability), funs).await?;
    assert!(!token.access_token.is_empty());
    Ok(())
}

async fn verify_replacement_revokes_previous_capability(funs: &TardisFunsInst, fixture: &TokenRenewalFixture) -> TardisResult<()> {
    let user_ctx = &fixture.user_ctx;
    let initial = Box::pin(register_renewal(renewal_register_req("wfrun-replace", "Bios-Token", Some(86400)), user_ctx.clone())).await?;
    let replacement = Box::pin(register_renewal(renewal_register_req("wfrun-replace", "Bios-Token", Some(86400)), user_ctx.clone())).await?;
    assert_ne!(initial.credential_id, replacement.credential_id);
    assert_ne!(initial.capability, replacement.capability);
    let replaced_row = iam_token_renewal::Entity::find_by_id(&initial.credential_id).one(funs.db().raw_conn()).await?.unwrap();
    // 替换保留 revoked 记录（审计链路），并清空 current 槽位，旧 capability 立即不可兑换。
    assert!(replaced_row.revoked);
    assert_eq!(replaced_row.revoked_by.as_deref(), Some(user_ctx.owner.as_str()));
    assert!(replaced_row.revoked_time.is_some());
    assert_eq!(replaced_row.replaced_by_credential_id.as_deref(), Some(replacement.credential_id.as_str()));
    assert!(replaced_row.current_run_id.is_none() && replaced_row.current_secret_key.is_none());
    assert_eq!(
        exchange_renewal(&initial.credential_id, &renewal_exchange_req("wfrun-replace", &initial.capability), funs)
            .await
            .unwrap_err()
            .code,
        "403-token-renewal-revoked"
    );
    let current_row = iam_token_renewal::Entity::find_by_id(&replacement.credential_id).one(funs.db().raw_conn()).await?.unwrap();
    assert!(!current_row.revoked);
    assert_eq!(current_row.current_run_id.as_deref(), Some("wfrun-replace"));
    let events = rbum_event_helper::get_notify_event_with_ctx(user_ctx).await?.unwrap();
    assert!(events.iter().any(|event| event.table_name == "iam_token_renewal" && event.operate == "u" && event.record_id == initial.credential_id));
    Ok(())
}

async fn verify_exchange_mints_resolvable_task_token(
    funs: &TardisFunsInst,
    fixture: &TokenRenewalFixture,
    runtime_api_address: std::net::SocketAddr,
    cp_api_address: std::net::SocketAddr,
) -> TardisResult<()> {
    let user_ctx = &fixture.user_ctx;
    // 经 TardisContextExtractor 的注册端点：网关链路把 Bios-Token 解析为上下文转发。
    let (http_code, body) = cp_register_http(cp_api_address, user_ctx, "wfrun-http", "Bios-Token", 86400).await?;
    assert_eq!(http_code, 200, "{body}");
    assert!(body_code(&body).starts_with("200"), "{body}");
    let http_data = renewal_data(&body);
    assert!(http_data.get("credential_id").and_then(Value::as_str).unwrap_or_default().trim().len() > 0);
    assert_eq!(http_data.get("capability").and_then(Value::as_str).unwrap_or_default().len(), 64);
    assert!(!http_data.get("expires_at").and_then(Value::as_str).unwrap_or_default().is_empty());
    let http_credential_id = http_data.get("credential_id").and_then(Value::as_str).unwrap().to_string();
    let http_capability = http_data.get("capability").and_then(Value::as_str).unwrap().to_string();
    let http_row = iam_token_renewal::Entity::find_by_id(&http_credential_id).one(funs.db().raw_conn()).await?.unwrap();
    assert_eq!(http_row.account_id, user_ctx.owner);
    assert_eq!(http_row.capability_hash, TardisFuns::crypto.digest.sha256(http_capability.as_bytes())?);

    // 兑换端点（受信宿主，服务 token，无用户 token）成功铸出新 token。
    let (exchange_code, exchange_body) = renewal_exchange_http(runtime_api_address, &http_credential_id, "wfrun-http", &http_capability, Some("scheduler")).await?;
    assert_eq!(exchange_code, 200, "{exchange_body}");
    assert!(body_code(&exchange_body).starts_with("200"), "{exchange_body}");
    let exchange_data = renewal_data(&exchange_body);
    let access_token = exchange_data.get("access_token").and_then(Value::as_str).unwrap_or_default().to_string();
    assert!(!access_token.trim().is_empty());
    assert!(!exchange_data.get("expires_at").and_then(Value::as_str).unwrap_or_default().is_empty());

    let resp = Box::pin(register_renewal(renewal_register_req("wfrun-exchange", "Bios-Token", Some(86400)), user_ctx.clone())).await?;
    let row_before = iam_token_renewal::Entity::find_by_id(&resp.credential_id).one(funs.db().raw_conn()).await?.unwrap();
    assert!(row_before.last_refreshed_at.is_none());
    let first = exchange_renewal(&resp.credential_id, &renewal_exchange_req("wfrun-exchange", &resp.capability), funs).await?;
    assert!(!first.access_token.trim().is_empty());
    // 兑换响应的 expires_at 是新 token 的到期时间，不是授权的绝对期限。
    let first_expires = DateTime::parse_from_rfc3339(&first.expires_at).unwrap().with_timezone(&Utc);
    let first_delta = (first_expires - Utc::now()).num_seconds();
    assert!(first_delta > 3000 && first_delta <= 3600, "minted token ttl should follow the mint config, got {first_delta}s");
    // 按 TokenTask 上下文取回（照 task grant 测试的取上下文方式）。
    let resolved = IamIdentCacheServ::get_context(
        &IamContextFetchReq {
            token: first.access_token.clone(),
            app_id: Some(fixture.app_id.clone()),
        },
        funs,
    )
    .await?;
    assert_eq!(resolved.owner, user_ctx.owner);
    assert_eq!(resolved.own_paths, user_ctx.own_paths);
    let wrong_app = IamContextFetchReq {
        token: first.access_token.clone(),
        app_id: Some("other-app".into()),
    };
    assert!(IamIdentCacheServ::get_context(&wrong_app, funs).await.is_err());
    assert!(funs.cache().ttl(&format!("iam:renewal-token:{}", first.access_token)).await? > 0);
    let token_info_key = format!("{}{}", funs.conf::<bios_iam::iam_config::IamConfig>().cache_key_token_info_, first.access_token);
    assert_eq!(funs.cache().get(&token_info_key).await?.unwrap(), format!("TokenTask,{}", user_ctx.owner));

    // capability 不轮换、可重复兑换；前一次签发的 token 不被作废。
    let second = exchange_renewal(&resp.credential_id, &renewal_exchange_req("wfrun-exchange", &resp.capability), funs).await?;
    assert_ne!(first.access_token, second.access_token);
    assert!(funs.cache().ttl(&format!("iam:renewal-token:{}", first.access_token)).await? > 0);
    let row_after = iam_token_renewal::Entity::find_by_id(&resp.credential_id).one(funs.db().raw_conn()).await?.unwrap();
    // 绝对期限不因兑换向后延长，仅记录 last_refreshed_at。
    assert_eq!(row_after.expires_at, row_before.expires_at);
    assert!(row_after.last_refreshed_at.is_some());

    // 无应用上下文（租户级 token）注册的授权同样可兑换，运行上下文即租户维度。
    let tenant_resp = Box::pin(register_renewal(renewal_register_req("wfrun-tenant", "Bios-Token", Some(86400)), fixture.tenant_user_ctx.clone())).await?;
    let tenant_row = iam_token_renewal::Entity::find_by_id(&tenant_resp.credential_id).one(funs.db().raw_conn()).await?.unwrap();
    assert_eq!(tenant_row.app_id, None);
    assert_eq!(tenant_row.own_paths, fixture.tenant_user_ctx.own_paths);
    let tenant_exchange = exchange_renewal(&tenant_resp.credential_id, &renewal_exchange_req("wfrun-tenant", &tenant_resp.capability), funs).await?;
    let tenant_resolved = IamIdentCacheServ::get_context(
        &IamContextFetchReq {
            token: tenant_exchange.access_token.clone(),
            app_id: None,
        },
        funs,
    )
    .await?;
    assert_eq!(tenant_resolved.owner, fixture.tenant_user_ctx.owner);
    assert_eq!(tenant_resolved.own_paths, fixture.tenant_user_ctx.own_paths);
    Ok(())
}

async fn verify_principal_revalidation_rejections(funs: &TardisFunsInst, fixture: &TokenRenewalFixture) -> TardisResult<()> {
    let user_ctx = &fixture.user_ctx;
    let app_id = &fixture.app_id;
    let role_id = &fixture.role_id;
    let resp = Box::pin(register_renewal(renewal_register_req("wfrun-reject", "Bios-Token", Some(86400)), user_ctx.clone())).await?;
    let req = renewal_exchange_req("wfrun-reject", &resp.capability);
    // 错误 run_id / 错误 capability → principal mismatch。
    assert_eq!(
        exchange_renewal(&resp.credential_id, &renewal_exchange_req("wfrun-other", &resp.capability), funs).await.unwrap_err().code,
        "409-token-renewal-principal-mismatch"
    );
    assert_eq!(
        exchange_renewal(&resp.credential_id, &renewal_exchange_req("wfrun-reject", "wrong-capability"), funs).await.unwrap_err().code,
        "409-token-renewal-principal-mismatch"
    );
    // 未知 credential → not registered。
    assert_eq!(
        exchange_renewal("no-such-credential", &req, funs).await.unwrap_err().code,
        "404-token-renewal-not-registered"
    );
    assert_eq!(
        revoke_renewal("no-such-credential", &renewal_revoke_req("wfrun-reject", &resp.capability), funs).await.unwrap_err().code,
        "404-token-renewal-not-registered"
    );

    // 账号锁定 → 兑换与注册都被拒绝。
    funs.db()
        .execute_one(
            "UPDATE iam_account SET lock_status = $1 WHERE id = $2",
            vec![IamAccountLockStateKind::ManualLocked.to_int().into(), user_ctx.owner.clone().into()],
        )
        .await?;
    assert_eq!(
        exchange_renewal(&resp.credential_id, &req, funs).await.unwrap_err().code,
        "403-token-renewal-account-invalid"
    );
    assert!(Box::pin(register_renewal(renewal_register_req("wfrun-reject-locked", "Bios-Token", None), user_ctx.clone())).await.is_err());
    funs.db()
        .execute_one(
            "UPDATE iam_account SET lock_status = $1 WHERE id = $2",
            vec![IamAccountLockStateKind::Unlocked.to_int().into(), user_ctx.owner.clone().into()],
        )
        .await?;

    // 应用成员关系被禁用 → 兑换拒绝，注册同样拒绝。
    funs.db()
        .execute_one(
            "UPDATE rbum_rel SET disabled = $1 WHERE tag = $2 AND from_rbum_id = $3 AND to_rbum_item_id = $4",
            vec![
                true.into(),
                IamRelKind::IamAccountApp.to_string().into(),
                user_ctx.owner.clone().into(),
                app_id.clone().into(),
            ],
        )
        .await?;
    assert_eq!(
        exchange_renewal(&resp.credential_id, &req, funs).await.unwrap_err().code,
        "403-token-renewal-membership-invalid"
    );
    assert!(Box::pin(register_renewal(renewal_register_req("wfrun-reject-app", "Bios-Token", None), user_ctx.clone())).await.is_err());
    funs.db()
        .execute_one(
            "UPDATE rbum_rel SET disabled = $1 WHERE tag = $2 AND from_rbum_id = $3 AND to_rbum_item_id = $4",
            vec![
                false.into(),
                IamRelKind::IamAccountApp.to_string().into(),
                user_ctx.owner.clone().into(),
                app_id.clone().into(),
            ],
        )
        .await?;

    // 应用内角色关系被禁用 → 同样拒绝（照 task grant 的角色校验）。
    funs.db()
        .execute_one(
            "UPDATE rbum_rel SET disabled = $1 WHERE tag = $2 AND from_rbum_id = $3 AND to_rbum_item_id = $4",
            vec![
                true.into(),
                IamRelKind::IamAccountRole.to_string().into(),
                user_ctx.owner.clone().into(),
                role_id.clone().into(),
            ],
        )
        .await?;
    assert_eq!(
        exchange_renewal(&resp.credential_id, &req, funs).await.unwrap_err().code,
        "403-token-renewal-membership-invalid"
    );
    funs.db()
        .execute_one(
            "UPDATE rbum_rel SET disabled = $1 WHERE tag = $2 AND from_rbum_id = $3 AND to_rbum_item_id = $4",
            vec![
                false.into(),
                IamRelKind::IamAccountRole.to_string().into(),
                user_ctx.owner.clone().into(),
                role_id.clone().into(),
            ],
        )
        .await?;
    // 恢复后可继续兑换。
    assert!(!exchange_renewal(&resp.credential_id, &req, funs).await?.access_token.is_empty());
    Ok(())
}

async fn verify_absolute_expiry(funs: &TardisFunsInst, fixture: &TokenRenewalFixture) -> TardisResult<()> {
    let user_ctx = &fixture.user_ctx;
    // 授权到期后兑换被拒，错误码归入 expired。
    let resp = Box::pin(register_renewal(renewal_register_req("wfrun-expire", "Bios-Token", Some(1)), user_ctx.clone())).await?;
    tokio::time::sleep(Duration::from_millis(1300)).await;
    assert_eq!(
        exchange_renewal(&resp.credential_id, &renewal_exchange_req("wfrun-expire", &resp.capability), funs)
            .await
            .unwrap_err()
            .code,
        "401-token-renewal-expired"
    );
    Ok(())
}

async fn verify_revocation(funs: &TardisFunsInst, fixture: &TokenRenewalFixture, runtime_api_address: std::net::SocketAddr) -> TardisResult<()> {
    let user_ctx = &fixture.user_ctx;
    let resp = Box::pin(register_renewal(renewal_register_req("wfrun-revoke", "Bios-Token", Some(86400)), user_ctx.clone())).await?;
    // 错误 capability 的吊销请求被拒绝，授权保持可用。
    // BIOS UniformError 把 4xx 映射回 HTTP 200、业务码保留在信封 code（照 task grant 测试的断言约定）。
    let (wrong_code, wrong_body) = renewal_revoke_http(runtime_api_address, &resp.credential_id, "wfrun-revoke", "wrong-capability", Some("scheduler")).await?;
    assert!(wrong_code == 409 || body_code(&wrong_body).starts_with("409-token-renewal-principal-mismatch"), "http_code={wrong_code}, {wrong_body}");
    let still_live = iam_token_renewal::Entity::find_by_id(&resp.credential_id).one(funs.db().raw_conn()).await?.unwrap();
    assert!(!still_live.revoked);
    assert!(!exchange_renewal(&resp.credential_id, &renewal_exchange_req("wfrun-revoke", &resp.capability), funs).await?.access_token.is_empty());
    // 正确 capability 的吊销置 revoked，兑换立即不可用。
    let (revoke_code, revoke_body) = renewal_revoke_http(runtime_api_address, &resp.credential_id, "wfrun-revoke", &resp.capability, Some("scheduler")).await?;
    assert_eq!(revoke_code, 200, "{revoke_body}");
    assert!(body_code(&revoke_body).starts_with("200"), "{revoke_body}");
    let revoked_row = iam_token_renewal::Entity::find_by_id(&resp.credential_id).one(funs.db().raw_conn()).await?.unwrap();
    assert!(revoked_row.revoked);
    assert!(revoked_row.revoked_time.is_some());
    assert!(revoked_row.current_run_id.is_none() && revoked_row.current_secret_key.is_none());
    assert_eq!(
        exchange_renewal(&resp.credential_id, &renewal_exchange_req("wfrun-revoke", &resp.capability), funs).await.unwrap_err().code,
        "403-token-renewal-revoked"
    );
    // 幂等：重复吊销直接成功。
    revoke_renewal(&resp.credential_id, &renewal_revoke_req("wfrun-revoke", &resp.capability), funs).await?;
    // 吊销后同 (run_id, secret_key) 可重新注册拿到全新授权。
    let revived = Box::pin(register_renewal(renewal_register_req("wfrun-revoke", "Bios-Token", Some(86400)), user_ctx.clone())).await?;
    assert_ne!(revived.credential_id, resp.credential_id);
    assert!(!exchange_renewal(&revived.credential_id, &renewal_exchange_req("wfrun-revoke", &revived.capability), funs).await?.access_token.is_empty());
    Ok(())
}
