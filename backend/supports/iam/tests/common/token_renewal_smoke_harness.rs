//! Optional real HTTP smoke harness. Used only by the ignored lifecycle test when explicitly
//! requested. The IAM endpoints and token/cache operations below are production implementations;
//! this is a small localhost gateway adapter, not an auth mock.
use std::collections::HashSet;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bios_auth::{auth_config::AuthConfig, serv::auth_kernel_serv::get_token_context};
use bios_iam::basic::serv::iam_oauth2_task_grant_serv::validate_task_service_token;
use bios_iam::console_passport::api::iam_cp_token_renewal_api::{IamCpTokenRenewalApi, IamTokenRenewalRuntimeApi};
use bios_iam::console_passport::dto::iam_cp_cert_dto::IamCpUserPwdLoginReq;
use bios_iam::console_passport::serv::iam_cp_cert_user_pwd_serv::IamCpCertUserPwdServ;
use bios_iam::iam_constants;
use tardis::basic::{dto::TardisContext, error::TardisError, field::TrimString, result::TardisResult};
use tardis::serde_json::{json, Value};
use tardis::tokio;
use tardis::web::poem::{get, post, http::{HeaderValue, StatusCode}, listener::TcpAcceptor, Endpoint, EndpointExt, IntoResponse, Middleware, Request, Response, Route, Server};
use tardis::web::poem_openapi::OpenApiService;
use tardis::web::uniform_error_mw::UniformError;
use tardis::TardisFuns;

struct SmokeState {
    owner: String,
    tenant_id: String,
    app_id: String,
    login_username: String,
    login_password: String,
    service_token: String,
    stop: tokio::sync::Notify,
    issued_tokens: Mutex<HashSet<String>>,
}

fn safe_error(code: &str, message: &str, status: StatusCode) -> Response {
    Response::builder().status(status).header("Content-Type", "application/json")
        .body(json!({"code":code,"msg":message,"data":null}).to_string())
}

fn safe_ok(data: Value) -> Response {
    Response::builder().status(StatusCode::OK).header("Content-Type", "application/json")
        .body(json!({"code":"200","msg":"","data":data}).to_string())
}

async fn authenticate(req: &Request) -> TardisResult<TardisContext> {
    let token = req.headers().get("Bios-Token").and_then(|v| v.to_str().ok()).unwrap_or_default();
    let app = req.headers().get("Bios-App").and_then(|v| v.to_str().ok()).unwrap_or_default();
    let funs = iam_constants::get_tardis_inst();
    // Same parser as the BIOS gateway. In particular, deleting token-info really invalidates both
    // original login tokens and renewal tokens; no magic token values or simulated expiration.
    Box::pin(get_token_context(token, app, &AuthConfig::default(), &funs.cache())).await
}

#[derive(Clone)]
struct TokenGateway;
struct TokenGatewayEndpoint<E>(E);
impl<E: Endpoint> Middleware<E> for TokenGateway {
    type Output = TokenGatewayEndpoint<E>;
    fn transform(&self, ep: E) -> Self::Output { TokenGatewayEndpoint(ep) }
}
impl<E: Endpoint> Endpoint for TokenGatewayEndpoint<E> {
    type Output = Response;
    async fn call(&self, mut req: Request) -> tardis::web::poem::Result<Response> {
        if req.uri().path().starts_with("/iam/cp/token-renewals") {
            // Never trust caller-supplied gateway context. Resolve actual Bios-Token then inject
            // the context consumed by the production TardisContextExtractor.
            let context_header_name = TardisFuns::fw_config().web_server().context_conf.context_header_name.clone();
            req.headers_mut().remove(&context_header_name);
            let ctx = match authenticate(&req).await {
                Ok(ctx) => ctx,
                Err(_) => return Ok(safe_error("401-gateway-cert-error", "authentication rejected", StatusCode::UNAUTHORIZED)),
            };
            let context = TardisFuns::crypto.base64.encode(TardisFuns::json.obj_to_string(&ctx).map_err(tardis::web::poem::Error::from)?);
            req.headers_mut().insert(
                context_header_name.parse::<tardis::web::poem::http::HeaderName>().expect("context header name"),
                HeaderValue::from_str(&context).expect("encoded context"),
            );
        }
        Ok(self.0.call(req).await?.into_response())
    }
}

struct ProtectedEndpoint { state: Arc<SmokeState>, permission_denied: bool }
impl Endpoint for ProtectedEndpoint {
    type Output = Response;
    async fn call(&self, req: Request) -> tardis::web::poem::Result<Response> {
        let ctx = match authenticate(&req).await {
            Ok(ctx) if ctx.owner == self.state.owner && ctx.own_paths == format!("{}/{}", self.state.tenant_id, self.state.app_id) => ctx,
            _ => return Ok(safe_error("401-gateway-cert-error", "authentication rejected", StatusCode::UNAUTHORIZED)),
        };
        if self.permission_denied {
            // A genuine authenticated business refusal with a non-gateway code: never refresh.
            return Ok(safe_error("401-smoke-business-policy", "business permission denied", StatusCode::UNAUTHORIZED));
        }
        let token = req.headers().get("Bios-Token").and_then(|v| v.to_str().ok()).unwrap_or_default();
        let fingerprint = TardisFuns::crypto.digest.sha256(token.as_bytes()).map_err(tardis::web::poem::Error::from)?;
        Ok(safe_ok(json!({"owner":ctx.owner,"ownPaths":ctx.own_paths,"tokenFingerprint":fingerprint})))
    }
}

#[derive(Clone, Copy)]
enum ControlAction { IssueToken, InvalidateToken, Stop }
struct ControlEndpoint { state: Arc<SmokeState>, action: ControlAction }
impl Endpoint for ControlEndpoint {
    type Output = Response;
    async fn call(&self, mut req: Request) -> tardis::web::poem::Result<Response> {
        let supplied = req.headers().get("X-Bios-Task-Service-Token").and_then(|v| v.to_str().ok());
        if validate_task_service_token(&self.state.service_token, supplied).is_err() {
            return Ok(safe_error("401-iam-oauth-task-service-required", "service authentication required", StatusCode::UNAUTHORIZED));
        }
        let funs = iam_constants::get_tardis_inst();
        match self.action {
            ControlAction::IssueToken => {
                // Production login-token mint path including cert/token cache and real account
                // validation. The test-only trusted control endpoint replaces password submission.
                let token = issue_fixture_token(&self.state).await.map_err(tardis::web::poem::Error::from)?;
                Ok(safe_ok(json!({"token":token})))
            }
            ControlAction::InvalidateToken => {
                let body: Value = req.take_body().into_json().await?;
                let Some(token) = body.get("token").and_then(Value::as_str).filter(|token| !token.is_empty() && token.len() <= 512) else {
                    return Ok(safe_error("400-smoke-invalid-request", "token is required", StatusCode::BAD_REQUEST));
                };
                let key = format!("{}{}", funs.conf::<bios_iam::iam_config::IamConfig>().cache_key_token_info_, token);
                let cache = funs.cache();
                let info = match cache.get(&key).await {
                    Ok(info) => info,
                    Err(_) => return Ok(safe_error("500-smoke-cache", "cache lookup failed", StatusCode::INTERNAL_SERVER_ERROR)),
                };
                if let Some(info) = info {
                    if info.split(',').nth(1) != Some(self.state.owner.as_str()) {
                        return Ok(safe_error("409-smoke-principal-mismatch", "token principal does not match fixture", StatusCode::CONFLICT));
                    }
                } else if !self.state.issued_tokens.lock().expect("issued tokens lock").contains(token) {
                    return Ok(safe_error("404-smoke-token-not-found", "token is not active in fixture", StatusCode::NOT_FOUND));
                }
                // Actual Redis deletion; do not mutate other users or unrelated keys.
                for key in [key, format!("iam:task-token:{token}"), format!("iam:renewal-token:{token}")] {
                    if cache.del(&key).await.is_err() {
                        return Ok(safe_error("500-smoke-cache", "cache invalidation failed", StatusCode::INTERNAL_SERVER_ERROR));
                    }
                }
                Ok(safe_ok(json!({"invalidated":true})))
            }
            ControlAction::Stop => {
                self.state.stop.notify_one();
                Ok(safe_ok(json!({"stopping":true})))
            }
        }
    }
}

async fn issue_fixture_token(state: &SmokeState) -> TardisResult<String> {
    let funs = iam_constants::get_tardis_inst();
    // 真实用户密码登录：校验 userpwd cert、账号状态并发行登录 token/缓存上下文，不做任何伪造。
    let account = Box::pin(IamCpCertUserPwdServ::login_by_user_pwd(
        &IamCpUserPwdLoginReq {
            ak: TrimString::from(state.login_username.clone()),
            sk: TrimString::from(state.login_password.clone()),
            tenant_id: Some(state.tenant_id.clone()),
            flag: None,
        },
        None,
        &funs,
    ))
    .await?;
    // 用受保护接口实际使用的同一解析器校验发行结果。
    let ctx = Box::pin(get_token_context(&account.token, &state.app_id, &AuthConfig::default(), &funs.cache())).await?;
    if ctx.owner != state.owner || ctx.own_paths != format!("{}/{}", state.tenant_id, state.app_id) {
        return Err(TardisError::internal_error("fixture token principal mismatch", "500-smoke-token-context"));
    }
    state.issued_tokens.lock().expect("issued tokens lock").insert(account.token.clone());
    Ok(account.token)
}

pub async fn serve_if_requested(fixture: &crate::TokenRenewalFixture) -> TardisResult<()> {
    let Ok(path) = std::env::var("BIOS_WF_RENEWAL_SMOKE_CREDENTIALS") else { return Ok(()) };
    let path = std::path::PathBuf::from(path);
    if !path.is_absolute() {
        return Err(TardisError::bad_request("smoke credentials path must be absolute", "400-smoke-path"));
    }
    let service_token = TardisFuns::crypto.key.rand_64_hex();
    std::env::set_var("BIOS_OAUTH2_TASK_SERVICE_TOKEN", &service_token);
    let state = Arc::new(SmokeState {
        owner: fixture.user_ctx.owner.clone(),
        tenant_id: fixture.tenant_user_ctx.own_paths.clone(),
        app_id: fixture.app_id.clone(),
        login_username: fixture.login_username.clone(),
        login_password: fixture.login_password.clone(),
        service_token,
        stop: tokio::sync::Notify::new(),
        issued_tokens: Mutex::new(HashSet::new()),
    });
    let bios_token = issue_fixture_token(&state).await?;
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await.expect("smoke localhost listener");
    let address = listener.local_addr().expect("smoke local address");
    let acceptor = TcpAcceptor::from_tokio(listener).expect("smoke acceptor");
    let api = Route::new()
        .nest("/iam", OpenApiService::new((IamCpTokenRenewalApi, IamTokenRenewalRuntimeApi), "Real IAM renewal smoke", "1").with(UniformError))
        .at("/protected", get(ProtectedEndpoint { state: state.clone(), permission_denied: false }))
        .at("/permission-denied", get(ProtectedEndpoint { state: state.clone(), permission_denied: true }))
        .at("/control/issue-token", post(ControlEndpoint { state: state.clone(), action: ControlAction::IssueToken }))
        .at("/control/invalidate-token", post(ControlEndpoint { state: state.clone(), action: ControlAction::InvalidateToken }))
        .at("/control/stop", post(ControlEndpoint { state: state.clone(), action: ControlAction::Stop }))
        .with(TokenGateway);
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    let task = tokio::spawn(async move {
        Server::new_with_acceptor(acceptor).run_with_graceful_shutdown(api, async move { let _ = shutdown_rx.await; }, Some(Duration::from_secs(1))).await.expect("real IAM smoke server");
    });
    let origin = format!("http://{address}");
    let credentials = json!({
        "iamBaseUrl":format!("{origin}/iam"),"protectedUrl":format!("{origin}/protected"),
        "permissionDeniedUrl":format!("{origin}/permission-denied"),"controlBaseUrl":format!("{origin}/control"),
        "issueTokenUrl":format!("{origin}/control/issue-token"),"stopUrl":format!("{origin}/control/stop"),
        "biosToken":bios_token,"biosApp":state.app_id,"owner":state.owner,"ownPaths":fixture.user_ctx.own_paths,
        "serviceToken":state.service_token,
    });
    // No overwrite/symlink following. File contains all secrets, mode 0600 from creation onward.
    let mut file = std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&path)
        .map_err(|_| TardisError::internal_error("cannot create private smoke credentials file", "500-smoke-credentials-file"))?;
    file.write_all(TardisFuns::json.obj_to_string(&credentials)?.as_bytes())
        .map_err(|_| TardisError::internal_error("cannot write smoke credentials file", "500-smoke-credentials-write"))?;
    file.sync_all().map_err(|_| TardisError::internal_error("cannot sync smoke credentials file", "500-smoke-credentials-sync"))?;
    eprintln!("[BIOS.Smoke] real HTTP fixture ready; credentials: {}", path.display());
    let stop_path = path.with_extension("json.stop");
    let keepalive_sec = std::env::var("BIOS_WF_RENEWAL_SMOKE_KEEPALIVE_SEC").ok().and_then(|v| v.parse::<u64>().ok()).unwrap_or(3600).clamp(30, 7200);
    let mut interval = tokio::time::interval(Duration::from_millis(250));
    let deadline = tokio::time::sleep(Duration::from_secs(keepalive_sec));
    tokio::pin!(deadline);
    loop {
        tokio::select! {
            _ = state.stop.notified() => break,
            _ = &mut deadline => break,
            _ = interval.tick() => if stop_path.exists() { break; },
        }
    }
    let _ = shutdown_tx.send(());
    task.await.expect("smoke server join");
    eprintln!("[BIOS.Smoke] real HTTP fixture stopped; only owned testcontainers will be dropped");
    Ok(())
}
