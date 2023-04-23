use std::collections::HashMap;

use tardis::chrono::{TimeZone, Utc};
use tardis::{
    basic::{dto::TardisContext, error::TardisError, result::TardisResult},
    cache::cache_client::TardisCacheClient,
    log::trace,
    regex::Regex,
    TardisFuns,
};

use crate::dto::auth_kernel_dto::{MixAuthResp, MixRequestBody, ResContainerLeafInfo};
use crate::helper::auth_common_helper;
use crate::{
    auth_config::AuthConfig,
    auth_constants::DOMAIN_CODE,
    dto::auth_kernel_dto::{AuthContext, AuthReq, AuthResp},
};

use super::{auth_crypto_serv, auth_mgr_serv, auth_res_serv};

pub(crate) async fn auth(req: &mut AuthReq, is_mix_req: bool) -> TardisResult<AuthResp> {
    trace!("[Auth] Request auth: {:?}", req);
    let config = TardisFuns::cs_config::<AuthConfig>(DOMAIN_CODE);
    match check(req) {
        Ok(true) => return Ok(AuthResp::ok(None, None, None, config)),
        Err(e) => return Ok(AuthResp::err(e, config)),
        _ => {}
    }
    let cache_client = TardisFuns::cache_by_module_or_default(DOMAIN_CODE);
    match ident(req, config, cache_client).await {
        Ok(ident) => match do_auth(&ident).await {
            Ok(res_container_leaf_info) => match decrypt(&req.headers, &req.body, config, &res_container_leaf_info, is_mix_req).await {
                Ok((body, headers)) => Ok(AuthResp::ok(Some(&ident), body, headers, config)),
                Err(e) => Ok(AuthResp::err(e, config)),
            },
            Err(e) => Ok(AuthResp::err(e, config)),
        },
        Err(e) => Ok(AuthResp::err(e, config)),
    }
}

fn check(req: &mut AuthReq) -> TardisResult<bool> {
    if req.method.to_lowercase() == "options" {
        return Ok(true);
    }
    req.path = req.path.trim().to_string();
    if req.path.starts_with('/') {
        req.path = req.path.strip_prefix('/').unwrap().to_string();
    }
    if req.path.is_empty() {
        return Err(TardisError::bad_request("[Auth] Request is not legal, missing [path]", "400-auth-req-path-not-empty"));
    }

    Ok(false)
}

async fn ident(req: &AuthReq, config: &AuthConfig, cache_client: &TardisCacheClient) -> TardisResult<AuthContext> {
    let rbum_kind = if let Some(rbum_kind) = req.headers.get(&config.head_key_protocol) {
        rbum_kind.to_string()
    } else {
        "iam-res".to_string()
    };
    let app_id = if let Some(app_id) = req.headers.get(&config.head_key_app) {
        app_id.to_string()
    } else {
        "".to_string()
    };
    // package rbum info
    let rbum_uri = format!("{}://{}", rbum_kind, req.path);
    let rbum_action = req.method.to_lowercase();

    if let Some(token) = req.headers.get(&config.head_key_token) {
        let account_id = if let Some(account_info) = cache_client.get(&format!("{}{}", config.cache_key_token_info, token)).await? {
            let account_info = account_info.split(',').collect::<Vec<_>>();
            account_info[1].to_string()
        } else {
            return Err(TardisError::unauthorized(&format!("[Auth] Token [{token}] is not legal"), "401-auth-req-token-not-exist"));
        };
        let mut context = if let Some(context) = cache_client.hget(&format!("{}{}", config.cache_key_account_info, account_id), &app_id).await? {
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
            }
        }
        let own_paths = context.own_paths.split('/').collect::<Vec<_>>();
        Ok(AuthContext {
            rbum_uri,
            rbum_action,
            app_id: if own_paths.len() > 1 { Some(own_paths[1].to_string()) } else { None },
            tenant_id: if context.own_paths.is_empty() { None } else { Some(own_paths[0].to_string()) },
            account_id: Some(context.owner),
            roles: Some(context.roles),
            groups: Some(context.groups),
            own_paths: Some(context.own_paths),
            ak: Some(context.ak),
        })
    } else if let Some(ak_authorization) = req.headers.get(&config.head_key_ak_authorization) {
        let req_date = if let Some(req_date) = req.headers.get(&config.head_key_date_flag) {
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
        let req_head_time = if let Ok(date_time) = Utc.datetime_from_str(req_date, &config.head_date_format) {
            date_time.timestamp_millis()
        } else {
            return Err(TardisError::bad_request("[Auth] bad date format", "401-auth-req-date-incorrect"));
        };
        let now = Utc::now().timestamp_millis();
        if now - req_head_time > config.head_date_interval_millsec as i64 {
            return Err(TardisError::unauthorized(
                "[Auth] The request has already been made or the client's time is incorrect. Please try again.",
                "401-auth-req-date-incorrect",
            ));
        }
        let ak_authorizations = ak_authorization.split(':').collect::<Vec<_>>();
        let ak = ak_authorizations[0];
        let signature = ak_authorizations[1];
        let (cache_sk, cache_tenant_id, cache_appid) = if let Some(ak_info) = cache_client.get(&format!("{}{}", config.cache_key_aksk_info, ak)).await? {
            let ak_vec = ak_info.split(',').collect::<Vec<_>>();
            (ak_vec[0].to_string(), ak_vec[1].to_string(), ak_vec[2].to_string())
        } else {
            return Err(TardisError::unauthorized(&format!("[Auth] Ak [{ak}] is not legal"), "401-auth-req-ak-not-exist"));
        };

        let sorted_req_query = auth_common_helper::sort_hashmap_query(req.query.clone());
        let calc_signature = TardisFuns::crypto
            .base64
            .encode(&TardisFuns::crypto.digest.hmac_sha256(&format!("{}\n{}\n{}\n{}", req.method, req_date, req.path, sorted_req_query).to_lowercase(), &cache_sk)?);

        if calc_signature != signature {
            return Err(TardisError::unauthorized(&format!("Ak [{ak}] authentication failed"), "401-auth-req-authenticate-fail"));
        }

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
        Ok(AuthContext {
            rbum_uri,
            rbum_action,
            app_id: if app_id.is_empty() { None } else { Some(app_id) },
            tenant_id: Some(cache_tenant_id),
            account_id: None,
            roles: None,
            groups: None,
            own_paths: Some(own_paths),
            ak: Some(ak_authorization.to_string()),
        })
    } else {
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

pub async fn do_auth(ctx: &AuthContext) -> TardisResult<Option<ResContainerLeafInfo>> {
    let matched_res = auth_res_serv::match_res(&ctx.rbum_action, &ctx.rbum_uri)?;
    if matched_res.is_empty() {
        // No authentication required
        return Ok(None);
    }
    for matched_res in matched_res {
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
                    if matched_tenants.contains(&format!("#{iam_tenant_id}#")) {
                        return Ok(Some(matched_res));
                    }
                }
            }
        } else {
            return Ok(Some(matched_res));
        }
    }
    Err(TardisError::forbidden("[Auth] Permission denied", "401-auth-req-permission-denied"))
}

pub async fn decrypt(
    headers: &HashMap<String, String>,
    body: &Option<String>,
    config: &AuthConfig,
    res_container_leaf_info: &Option<ResContainerLeafInfo>,
    is_mix_req: bool,
) -> TardisResult<(Option<String>, Option<HashMap<String, String>>)> {
    if is_mix_req {
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
    if headers.contains_key(&config.head_key_crypto) {
        //todo Because the return encryption has not yet been implemented, it has been temporarily modified.todo need_crypto_resp ->true
        let (body, headers) = auth_crypto_serv::decrypt_req(headers, body, true, config.default_resp_crypto, config).await?;
        return Ok((body, headers));
    }
    Ok((None, None))
}

pub(crate) async fn parse_mix_req(req: AuthReq) -> TardisResult<MixAuthResp> {
    let config = TardisFuns::cs_config::<AuthConfig>(DOMAIN_CODE);
    let (body, headers) = auth_crypto_serv::decrypt_req(&req.headers, &req.body, true, true, config).await?;
    let body = body.ok_or_else(|| TardisError::bad_request("[MixReq] decrypt body can't be empty", "401-parse_mix_req-parse-error"))?;

    let mix_body = TardisFuns::json.str_to_obj::<MixRequestBody>(&body)?;
    let mut headers = headers.unwrap_or_default();
    headers.extend(mix_body.headers);
    let auth_resp = auth(
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
    .await?;
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
