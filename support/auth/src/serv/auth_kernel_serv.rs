use std::collections::HashMap;

use tardis::chrono::{NaiveDateTime, Utc};
use tardis::{
    basic::{dto::TardisContext, error::TardisError, result::TardisResult},
    cache::cache_client::TardisCacheClient,
    log::trace,
    regex::Regex,
    TardisFuns,
};

use super::{auth_crypto_serv, auth_mgr_serv, auth_res_serv};
#[cfg(feature = "web-server")]
use crate::dto::auth_kernel_dto::{AuthResp, MixAuthResp, MixRequestBody};
use crate::dto::auth_kernel_dto::{AuthResult, ResContainerLeafInfo, SignWebHookReq};
use crate::helper::auth_common_helper;
use crate::{
    auth_config::AuthConfig,
    auth_constants::DOMAIN_CODE,
    dto::auth_kernel_dto::{AuthContext, AuthReq},
};

pub async fn auth(req: &mut AuthReq, is_mix_req: bool) -> TardisResult<AuthResult> {
    trace!("[Auth] Request auth: {:?}", req);
    let config = TardisFuns::cs_config::<AuthConfig>(DOMAIN_CODE);
    match check(req) {
        Ok(true) => return Ok(AuthResult::ok(None, None, None, &config)),
        Err(e) => return Ok(AuthResult::err(e, &config)),
        _ => {}
    }
    let cache_client = TardisFuns::cache_by_module_or_default(DOMAIN_CODE);
    match ident(req, &config, &cache_client).await {
        Ok(ident) => match do_auth(&ident).await {
            Ok(res_container_leaf_info) => match decrypt(req, &config, &res_container_leaf_info, is_mix_req).await {
                Ok((body, headers)) => Ok(AuthResult::ok(Some(&ident), body, headers, &config)),
                Err(e) => Ok(AuthResult::err(e, &config)),
            },
            Err(e) => Ok(AuthResult::err(e, &config)),
        },
        Err(e) => Ok(AuthResult::err(e, &config)),
    }
}

fn check(req: &mut AuthReq) -> TardisResult<bool> {
    if req.method.to_lowercase() == "options" {
        return Ok(true);
    }
    req.path = req.path.trim().to_string();
    if req.path.starts_with('/') {
        req.path = req.path.strip_prefix('/').map_or(req.path.clone(), |s| s.to_string());
    }
    if req.path.is_empty() {
        return Err(TardisError::bad_request("[Auth] Request is not legal, missing [path]", "400-auth-req-path-not-empty"));
    }

    Ok(false)
}

async fn ident(req: &mut AuthReq, config: &AuthConfig, cache_client: &TardisCacheClient) -> TardisResult<AuthContext> {
    // Do not allow external header information to be used internally
    req.headers.remove(&config.head_key_auth_ident);

    let rbum_kind = if let Some(rbum_kind) = req.headers.get(&config.head_key_protocol).or_else(|| req.headers.get(&config.head_key_protocol.to_lowercase())) {
        rbum_kind.to_string()
    } else {
        "iam-res".to_string()
    };
    let app_id = if let Some(app_id) = req
        .headers
        .get(&config.head_key_app)
        .or_else(|| req.headers.get(&config.head_key_app.to_lowercase()))
        .or_else(|| req.query.get(&config.head_key_app))
        .or_else(|| req.query.get(&config.head_key_app.to_lowercase()))
    {
        app_id.to_string()
    } else {
        "".to_string()
    };
    // package rbum info
    let rbum_uri = format!("{}://{}", rbum_kind, req.path);
    let rbum_action = req.method.to_lowercase();

    if let Some(token) = req
        .headers
        .get(&config.head_key_token)
        .or_else(|| req.headers.get(&config.head_key_token.to_lowercase()))
        .or_else(|| req.query.get(&config.head_key_token))
        .or_else(|| req.query.get(&config.head_key_token.to_lowercase()))
    {
        let context = self::get_token_context(token, &app_id, config, cache_client).await?;
        let own_paths_split = context.own_paths.split('/').collect::<Vec<_>>();
        let tenant_id = if context.own_paths.is_empty() { None } else { Some(own_paths_split[0].to_string()) };
        let app_id = if own_paths_split.len() > 1 { Some(own_paths_split[1].to_string()) } else { None };
        let mut roles = context.roles.clone();
        for role in context.roles.clone() {
            if role.contains(':') {
                let extend_role = role.split(':').collect::<Vec<_>>()[0];
                roles.push(extend_role.to_string());
            }
        }
        Ok(AuthContext {
            rbum_uri,
            rbum_action,
            app_id,
            tenant_id,
            account_id: Some(context.owner),
            roles: Some(roles),
            groups: Some(context.groups),
            own_paths: Some(context.own_paths),
            ak: Some(context.ak),
        })
    } else if let Some(ak_authorization) = get_ak_key(req, config) {
        let (req_date, ak, signature) = self::parsing_base_ak(&ak_authorization, req, config, false).await?;
        let (cache_sk, cache_tenant_id, cache_appid) = self::get_cache_ak(&ak, config, cache_client).await?;
        self::check_ak_signature(&ak, &cache_sk, &signature, &req_date, req).await?;

        let mut own_paths = cache_tenant_id.clone();
        if !app_id.is_empty() {
            if app_id != cache_appid {
                return Err(TardisError::unauthorized(
                    &format!("Ak [{ak}]  with App [{app_id}] is not legal"),
                    "401-auth-req-ak-or-app-not-exist",
                ));
            }
            own_paths = format!("{cache_tenant_id}/{app_id}")
        }

        req.headers.insert(config.head_key_auth_ident.clone(), ak_authorization.clone());

        Ok(AuthContext {
            rbum_uri,
            rbum_action,
            app_id: if app_id.is_empty() { None } else { Some(app_id) },
            tenant_id: Some(cache_tenant_id),
            account_id: None,
            roles: None,
            groups: None,
            own_paths: Some(own_paths),
            ak: Some(ak.to_string()),
        })
    } else if let Some(ak_authorization) = get_webhook_ak_key(req, config) {
        let (req_date, ak, signature) = self::parsing_base_ak(&ak_authorization, req, config, true).await?;
        let (cache_sk, cache_tenant_id, cache_appid) = self::get_cache_ak(&ak, config, cache_client).await?;
        let owner = if let Some(owner) = req.query.get(&config.query_owner).or_else(|| req.query.get(&config.query_owner.to_lowercase())) {
            owner
        } else {
            return Err(TardisError::unauthorized(
                &format!("[Auth] Request is not legal, missing query [{}]", config.query_owner),
                "401-auth-req-ak-not-exist",
            ));
        };
        let own_paths = if let Some(own_paths) = req.query.get(&config.query_own_paths).or_else(|| req.query.get(&config.query_own_paths.to_lowercase())) {
            own_paths
        } else {
            return Err(TardisError::unauthorized(
                &format!("[Auth] Request is not legal, missing query [{}]", config.query_own_paths),
                "401-auth-req-ak-not-exist",
            ));
        };
        self::check_webhook_ak_signature(owner, own_paths, &ak, &cache_sk, &signature, &req_date, req, config).await?;
        let mut cache_own_paths = cache_tenant_id.clone();
        if !cache_appid.is_empty() {
            if app_id != cache_appid {
                return Err(TardisError::unauthorized(
                    &format!("Ak [{ak}]  with App [{app_id}] is not legal"),
                    "401-auth-req-ak-or-app-not-exist",
                ));
            }
            cache_own_paths = format!("{cache_tenant_id}/{app_id}")
        }
        if own_paths.contains(&cache_own_paths) {
            let context = self::get_account_context(&ak_authorization, owner, &app_id, config, cache_client).await?;
            let mut roles = context.roles.clone();
            for role in roles.clone() {
                if role.contains(':') {
                    let extend_role = role.split(':').collect::<Vec<_>>()[0];
                    roles.push(extend_role.to_string());
                }
            }
            let own_paths_split = own_paths.split('/').collect::<Vec<_>>();
            let tenant_id = if own_paths.is_empty() { None } else { Some(own_paths_split[0].to_string()) };
            let app_id = if own_paths_split.len() > 1 { Some(own_paths_split[1].to_string()) } else { None };
            Ok(AuthContext {
                rbum_uri,
                rbum_action,
                app_id,
                tenant_id,
                account_id: Some(owner.to_string()),
                roles: Some(roles),
                groups: Some(context.groups),
                own_paths: Some(own_paths.to_string()),
                ak: Some(ak.to_string()),
            })
        } else {
            Err(TardisError::forbidden(
                &format!("[Auth] Request is not legal from head [{}]", config.query_own_paths),
                "403-auth-req-permission-denied",
            ))
        }
    } else {
        let matched_res = auth_res_serv::match_res(&rbum_action, &rbum_uri)?;
        for res in matched_res {
            if res.auth.is_none() && res.need_login {
                return Err(TardisError::unauthorized(
                    &format!("[Auth] need is not legal from head [{}]", config.head_key_token),
                    "401-auth-req-token-not-exist",
                ));
            }
        }
        // public
        Ok(AuthContext {
            rbum_uri,
            rbum_action,
            app_id: None,
            tenant_id: None,
            account_id: None,
            roles: None,
            groups: None,
            own_paths: None,
            ak: None,
        })
    }
}

pub async fn get_token_context(token: &str, app_id: &str, config: &AuthConfig, cache_client: &TardisCacheClient) -> TardisResult<TardisContext> {
    let account_id = if let Some(token_value) = cache_client.get(&format!("{}{}", config.cache_key_token_info, token)).await? {
        trace!("Token info: {}", token_value);
        let account_info: Vec<&str> = token_value.split(',').collect::<Vec<_>>();
        if account_info.len() > 2 {
            cache_client
                .set_ex(
                    &format!("{}{}", config.cache_key_token_info, token),
                    &token_value,
                    account_info[2].parse().map_err(|e| TardisError::internal_error(&format!("[Auth] account_info ex_sec parse error {}", e), ""))?,
                )
                .await?;
        }
        account_info[1].to_string()
    } else {
        return Err(TardisError::unauthorized(&format!("[Auth] Token [{token}] is not legal"), "401-auth-req-token-not-exist"));
    };
    let context = self::get_account_context(token, &account_id, app_id, config, cache_client).await?;
    Ok(context)
}

async fn get_account_context(token: &str, account_id: &str, app_id: &str, config: &AuthConfig, cache_client: &TardisCacheClient) -> TardisResult<TardisContext> {
    let mut context: TardisContext = if let Some(context) = cache_client.hget(&format!("{}{}", config.cache_key_account_info, account_id), app_id).await? {
        TardisFuns::json.str_to_obj::<TardisContext>(&context)?
    } else {
        return Err(TardisError::unauthorized(
            &format!("[Auth] Token [{token}] with App [{app_id}] is not legal"),
            "401-auth-req-token-or-app-not-exist",
        ));
    };
    if !app_id.is_empty() {
        if let Some(tenant_context) = cache_client.hget(&format!("{}{}", config.cache_key_account_info, account_id), "").await? {
            let tenant_context = TardisFuns::json.str_to_obj::<TardisContext>(&tenant_context)?;
            if !tenant_context.roles.is_empty() {
                context.roles.extend(tenant_context.roles);
            }
            if !tenant_context.groups.is_empty() {
                context.groups.extend(tenant_context.groups);
            }
            if !context.own_paths.contains(&tenant_context.own_paths) {
                return Err(TardisError::unauthorized(
                    &format!("[Auth] Token [{token}] with App [{app_id}] is not legal"),
                    "401-auth-req-token-or-app-not-exist",
                ));
            }
        }
    }
    Ok(context)
}

async fn parsing_base_ak(ak_authorization: &str, req: &AuthReq, config: &AuthConfig, is_webhook: bool) -> TardisResult<(String, String, String)> {
    let req_date = if let Some(req_date) = req
        .headers
        .get(&config.head_key_date_flag)
        .or_else(|| req.headers.get(&config.head_key_date_flag.to_lowercase()))
        .or_else(|| req.query.get(&config.head_key_date_flag))
        .or_else(|| req.query.get(&config.head_key_date_flag.to_lowercase()))
    {
        req_date
    } else {
        return Err(TardisError::unauthorized(
            &format!("[Auth] Request is not legal, missing header [{}]", config.head_key_date_flag),
            "401-auth-req-ak-not-exist",
        ));
    };
    if !ak_authorization.contains(':') {
        return Err(TardisError::unauthorized(
            &format!("[Auth] Ak-Authorization [{ak_authorization}] is not legal",),
            "401-auth-req-ak-not-exist",
        ));
    }
    let req_head_time = if let Ok(date_time) = NaiveDateTime::parse_from_str(req_date, &config.head_date_format) {
        date_time.and_utc().timestamp_millis()
    } else {
        return Err(TardisError::bad_request("[Auth] bad date format", "401-auth-req-date-incorrect"));
    };
    let now = Utc::now().timestamp_millis();
    if is_webhook {
        if req_head_time > now {
            return Err(TardisError::unauthorized(
                "[Auth] The webhook interface time has expired. Procedure.",
                "401-auth-req-date-incorrect",
            ));
        }
    } else if now - req_head_time > config.head_date_interval_ms as i64 {
        return Err(TardisError::unauthorized(
            "[Auth] The request has already been made or the client's time is incorrect. Please try again.",
            "401-auth-req-date-incorrect",
        ));
    }
    let ak_authorizations = ak_authorization.split(':').collect::<Vec<_>>();
    let ak = ak_authorizations[0];
    let signature = ak_authorizations[1];
    Ok((req_date.to_string(), ak.to_string(), signature.to_string()))
}

async fn get_cache_ak(ak: &str, config: &AuthConfig, cache_client: &TardisCacheClient) -> TardisResult<(String, String, String)> {
    let (cache_sk, cache_tenant_id, cache_appid) = if let Some(ak_info) = cache_client.get(&format!("{}{}", config.cache_key_aksk_info, ak)).await? {
        let ak_vec = ak_info.split(',').collect::<Vec<_>>();
        (ak_vec[0].to_string(), ak_vec[1].to_string(), ak_vec[2].to_string())
    } else {
        return Err(TardisError::unauthorized(&format!("[Auth] Ak [{ak}] is not legal"), "401-auth-req-ak-not-exist"));
    };
    Ok((cache_sk, cache_tenant_id, cache_appid))
}

async fn check_ak_signature(ak: &str, cache_sk: &str, signature: &str, req_date: &str, req: &AuthReq) -> TardisResult<()> {
    let sorted_req_query = auth_common_helper::sort_hashmap_query(req.query.clone());
    let calc_signature = TardisFuns::crypto
        .base64
        .encode(TardisFuns::crypto.digest.hmac_sha256(format!("{}\n{}\n{}\n{}", req.method, req_date, req.path, sorted_req_query).to_lowercase(), cache_sk)?);
    if calc_signature != signature {
        return Err(TardisError::unauthorized(&format!("Ak [{ak}] authentication failed"), "401-auth-req-authenticate-fail"));
    }
    Ok(())
}

async fn check_webhook_ak_signature(
    owner: &str,
    own_paths: &str,
    ak: &str,
    cache_sk: &str,
    signature: &str,
    req_date: &str,
    req: &AuthReq,
    config: &AuthConfig,
) -> TardisResult<()> {
    let mut query = req.query.clone();
    query.remove(&config.head_key_ak_authorization);
    let sorted_req_query = auth_common_helper::sort_hashmap_query(query);
    let calc_signature = TardisFuns::crypto.base64.encode(TardisFuns::crypto.digest.hmac_sha256(
        format!("{}\n{}\n{}\n{}\n{}\n{}", owner, own_paths, req.method, req_date, req.path, sorted_req_query).to_lowercase(),
        cache_sk,
    )?);
    if calc_signature != signature {
        return Err(TardisError::unauthorized(&format!("Ak [{ak}] authentication failed"), "401-auth-req-authenticate-fail"));
    }
    Ok(())
}

fn get_ak_key(req: &AuthReq, config: &AuthConfig) -> Option<String> {
    let lowercase_key = config.head_key_ak_authorization.to_lowercase();
    req.headers.get(&config.head_key_ak_authorization).or_else(|| req.headers.get(&lowercase_key)).cloned()
}

fn get_webhook_ak_key(req: &AuthReq, config: &AuthConfig) -> Option<String> {
    let lowercase_key = config.head_key_ak_authorization.to_lowercase();
    req.query.get(&config.head_key_ak_authorization).or_else(|| req.query.get(&lowercase_key)).cloned()
}

pub async fn sign_webhook_ak(sign_req: &SignWebHookReq) -> TardisResult<String> {
    let config = TardisFuns::cs_config::<AuthConfig>(DOMAIN_CODE);
    let cache_client = TardisFuns::cache_by_module_or_default(DOMAIN_CODE);
    let (cache_sk, cache_tenant_id, cache_appid) = self::get_cache_ak(&sign_req.ak, &config, &cache_client).await?;
    let mut cache_own_paths = cache_tenant_id.clone();
    if !cache_appid.is_empty() {
        cache_own_paths = format!("{cache_tenant_id}/{cache_appid}")
    }
    if sign_req.own_paths.contains(&cache_own_paths) {
        let sorted_req_query = auth_common_helper::sort_hashmap_query(sign_req.query.clone());
        let calc_signature = TardisFuns::crypto.base64.encode(
            TardisFuns::crypto.digest.hmac_sha256(
                format!(
                    "{}\n{}\n{}\n{}\n{}\n{}",
                    sign_req.onwer, sign_req.own_paths, sign_req.method, sign_req.req_date, sign_req.path, sorted_req_query
                )
                .to_lowercase(),
                cache_sk,
            )?,
        );
        return Ok(calc_signature);
    }
    Err(TardisError::forbidden(
        "[Auth] Signing the webhook permission denied. ",
        "500-auth-sign-webhook-permission-denied",
    ))
}

pub async fn do_auth(ctx: &AuthContext) -> TardisResult<Option<ResContainerLeafInfo>> {
    let matched_res = auth_res_serv::match_res(&ctx.rbum_action, &ctx.rbum_uri)?;
    if matched_res.is_empty() {
        // No authentication required
        return Ok(None);
    }
    let matched_res = matched_res[0].clone();
    // for matched_res in matched_res {
    // Determine if the most precisely matched resource requires double authentication
    if matched_res.need_double_auth {
        if let Some(req_account_id) = &ctx.account_id {
            if !auth_mgr_serv::has_double_auth(req_account_id).await? {
                return Err(TardisError::forbidden("[Auth] Secondary confirmation is required", "401-auth-req-need-double-auth"));
            }
        } else {
            return Err(TardisError::forbidden("[Auth] Secondary confirmation is required", "401-auth-req-need-double-auth"));
        }
    }
    // Check auth
    if let Some(auth) = &matched_res.auth {
        // let now = Utc::now().timestamp();
        // if let (Some(st), Some(et)) = (auth.st, auth.et) {
        //     if now > et || now < st {
        //         // expired,need delete auth
        //         auth_res_serv::delete_auth(&matched_res.action, &matched_res.uri).await?;
        //         continue;
        //     }
        // }
        if let Some(matched_accounts) = &auth.accounts {
            if let Some(req_account_id) = &ctx.account_id {
                if matched_accounts.contains(&format!("#{req_account_id}#")) {
                    return Ok(Some(matched_res));
                }
            }
        }
        if let Some(matched_roles) = &auth.roles {
            if let Some(iam_roles) = &ctx.roles {
                for iam_role in iam_roles {
                    if matched_roles.contains(&format!("#{iam_role}#")) {
                        return Ok(Some(matched_res));
                    }
                }
            }
        }
        if let Some(matched_groups) = &auth.groups {
            if let Some(iam_groups) = &ctx.groups {
                for iam_group in iam_groups {
                    if Regex::new(&format!(r"#{iam_group}.*#"))?.is_match(matched_groups) {
                        return Ok(Some(matched_res));
                    }
                }
            }
        }
        if let Some(matched_apps) = &auth.apps {
            if let Some(iam_app_id) = &ctx.app_id {
                if matched_apps.contains(&format!("#{iam_app_id}#")) {
                    return Ok(Some(matched_res));
                }
            }
        }
        if let Some(matched_tenants) = &auth.tenants {
            if let Some(iam_tenant_id) = &ctx.tenant_id {
                if matched_tenants.contains(&format!("#{iam_tenant_id}#")) || matched_tenants.contains(&"#*#".to_string()) {
                    return Ok(Some(matched_res));
                }
            }
        }
        if let Some(matched_aks) = &auth.ak {
            if let Some(iam_ak) = &ctx.ak {
                if matched_aks.contains(&format!("#{iam_ak}#")) || matched_aks.contains(&"#*#".to_string()) {
                    return Ok(Some(matched_res));
                }
            }
        }
    } else {
        return Ok(Some(matched_res));
    }
    // }
    if ctx.ak.is_some() {
        //have token,not not have permission
        Err(TardisError::forbidden("[Auth] Permission denied", "403-auth-req-permission-denied"))
    } else {
        //not token
        Err(TardisError::unauthorized("[Auth] Permission denied", "401-auth-req-unauthorized"))
    }
}

pub async fn decrypt(
    req: &AuthReq,
    config: &AuthConfig,
    res_container_leaf_info: &Option<ResContainerLeafInfo>,
    is_mix_req: bool,
) -> TardisResult<(Option<String>, Option<HashMap<String, String>>)> {
    let headers = &req.headers;
    let body = &req.body;
    let mut is_skip = false;
    for exclude_path in TardisFuns::cs_config::<AuthConfig>(DOMAIN_CODE).exclude_encrypt_decrypt_path.clone() {
        if req.path.starts_with(&exclude_path) {
            is_skip = true;
        }
    }
    if is_mix_req || is_skip {
        return Ok((body.clone(), Some(headers.clone())));
    }
    if let Some(res_container_leaf_info) = res_container_leaf_info {
        // The interface configuration specifies that the encryption must be done
        if res_container_leaf_info.need_crypto_req || res_container_leaf_info.need_crypto_resp {
            let (body, headers) = auth_crypto_serv::decrypt_req(headers, body, res_container_leaf_info.need_crypto_req, res_container_leaf_info.need_crypto_resp, config).await?;
            return Ok((body, headers));
        }
    }
    // Or, the interface configuration does not require encryption, but the request comes with encrypted headers. (Content consultation mechanism)
    if headers.contains_key(&config.head_key_crypto) || headers.contains_key(&config.head_key_crypto.to_lowercase()) {
        //todo Because the return encryption has not yet been implemented, it has been temporarily modified.todo need_crypto_resp ->true
        let (body, headers) = auth_crypto_serv::decrypt_req(headers, body, true, config.default_resp_crypto, config).await?;
        return Ok((body, headers));
    }
    Ok((None, None))
}

#[cfg(feature = "web-server")]
pub(crate) async fn parse_mix_req(req: AuthReq) -> TardisResult<MixAuthResp> {
    let config = TardisFuns::cs_config::<AuthConfig>(DOMAIN_CODE);
    let (body, headers) = auth_crypto_serv::decrypt_req(&req.headers, &req.body, true, true, &config).await?;
    let body = body.ok_or_else(|| TardisError::bad_request("[MixReq] decrypt body can't be empty", "401-parse_mix_req-parse-error"))?;

    let mix_body = TardisFuns::json.str_to_obj::<MixRequestBody>(&body)?;
    let mut headers = headers.unwrap_or_default();
    headers.extend(mix_body.headers);
    let auth_resp = AuthResp::from(
        auth(
            &mut AuthReq {
                scheme: req.scheme,
                path: req.path,
                query: req.query,
                method: mix_body.method.clone(),
                host: "".to_string(),
                port: 80,
                headers,
                body: Some(mix_body.body),
            },
            true,
        )
        .await?,
    );
    let url = if let Some(0) = mix_body.uri.find('/') {
        mix_body.uri
    } else {
        format!("/{}", mix_body.uri)
    };
    Ok(MixAuthResp {
        url,
        method: mix_body.method,
        allow: auth_resp.allow,
        status_code: auth_resp.status_code,
        reason: auth_resp.reason,
        headers: auth_resp.headers,
        body: auth_resp.body,
    })
}
