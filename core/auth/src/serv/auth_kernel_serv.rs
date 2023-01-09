use tardis::{
    basic::{dto::TardisContext, error::TardisError, result::TardisResult},
    cache::cache_client::TardisCacheClient,
    regex::Regex,
    TardisFuns,
};

use crate::{
    auth_config::AuthConfig,
    auth_constants::DOMAIN_CODE,
    dto::auth_dto::{AuthContext, AuthReq, AuthResp},
};

use super::auth_res_serv;

pub(crate) async fn auth(req: &AuthReq) -> TardisResult<AuthResp> {
    let config = TardisFuns::cs_config::<AuthConfig>(DOMAIN_CODE);
    match check(req) {
        Ok(true) => return Ok(AuthResp::ok(None, config)),
        Err(e) => return Ok(AuthResp::err(e, config)),
        _ => {}
    }
    let cache_client = TardisFuns::cache_by_module_or_default(DOMAIN_CODE);
    match ident(req, config, cache_client).await {
        Ok(ident) => match do_auth(&ident) {
            Ok(_) => Ok(AuthResp::ok(Some(&ident), config)),
            Err(e) => Ok(AuthResp::err(e, config)),
        },
        Err(e) => Ok(AuthResp::err(e, config)),
    }
}

fn check(req: &AuthReq) -> TardisResult<bool> {
    if req.method.to_lowercase() == "options" {
        return Ok(true);
    }
    if req.path.is_empty() || !req.path.starts_with('/') {
        return Err(TardisError::bad_request("[Auth]Request is not legal, missing [path]", "400-auth-req-path-not-empty"));
    }
    return Ok(false);
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
    let domain_idx = req.path.strip_prefix('/').unwrap().find('/');
    let (rbum_domain, rbum_item) = if let Some(index) = domain_idx {
        (req.path[1..index].to_string(), req.path[index + 1..].to_string())
    } else {
        (req.path[1..].to_string(), "".to_string())
    };
    // package rbum info
    let rbum_uri = format!("{}://{}{}", rbum_kind, rbum_domain, rbum_item);
    let rbum_action = req.method.to_lowercase();

    if let Some(token) = req.headers.get(&config.head_key_token) {
        let account_id = if let Some(account_info) = cache_client.get(&format!("{}{}", config.cache_key_token_info, token)).await? {
            let account_info = account_info.split(",").collect::<Vec<_>>();
            account_info[1].to_string()
        } else {
            return Err(TardisError::unauthorized(&format!("Token [{}] is not legal", token), "401-auth-req-token-not-exist"));
        };
        let mut context = if let Some(context) = cache_client.hget(&format!("{}{}", config.cache_key_account_info, account_id), &app_id).await? {
            TardisFuns::json.str_to_obj::<TardisContext>(&context)?
        } else {
            return Err(TardisError::unauthorized(
                &format!("Token [{}] with App [{}] is not legal", token, app_id),
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
            iam_app_id: if own_paths.len() > 1 { Some(own_paths[1].to_string()) } else { None },
            iam_tenant_id: if context.own_paths.is_empty() { None } else { Some(own_paths[0].to_string()) },
            iam_account_id: Some(context.owner),
            iam_roles: Some(context.roles),
            iam_groups: Some(context.groups),
            own_paths: Some(context.own_paths),
            ak: Some(context.ak),
        })
    } else {
        // public
        Ok(AuthContext {
            rbum_uri,
            rbum_action,
            iam_app_id: None,
            iam_tenant_id: None,
            iam_account_id: None,
            iam_roles: None,
            iam_groups: None,
            own_paths: None,
            ak: None,
        })
    }
}

pub fn do_auth(ctx: &AuthContext) -> TardisResult<()> {
    let mathced_res = auth_res_serv::match_res(&ctx.rbum_action, &ctx.rbum_uri)?;
    if mathced_res.is_empty() {
        // No authentication required
        return Ok(());
    }
    for mathced_res in mathced_res {
        if let Some(mathed_accounts) = mathced_res.auth.accounts {
            if let Some(req_account_id) = &ctx.iam_account_id {
                if mathed_accounts.contains(&format!("#{}#", req_account_id)) {
                    return Ok(());
                }
            }
        }
        if let Some(mathed_roles) = mathced_res.auth.roles {
            if let Some(iam_roles) = &ctx.iam_roles {
                for iam_role in iam_roles {
                    if mathed_roles.contains(&format!("#{}#", iam_role)) {
                        return Ok(());
                    }
                }
            }
        }
        if let Some(mathed_groups) = mathced_res.auth.groups {
            if let Some(iam_groups) = &ctx.iam_groups {
                for iam_group in iam_groups {
                    if Regex::new(&format!(r"#{}.*#", iam_group))?.is_match(&mathed_groups) {
                        return Ok(());
                    }
                }
            }
        }
        if let Some(mathed_apps) = mathced_res.auth.apps {
            if let Some(iam_app_id) = &ctx.iam_app_id {
                if mathed_apps.contains(&format!("#{}#", iam_app_id)) {
                    return Ok(());
                }
            }
        }
        if let Some(mathed_tenants) = mathced_res.auth.tenants {
            if let Some(iam_tenant_id) = &ctx.iam_tenant_id {
                if mathed_tenants.contains(&format!("#{}#", iam_tenant_id)) {
                    return Ok(());
                }
            }
        }
    }
    return Err(TardisError::unauthorized("Permission denied", "401-auth-req-permission-denied"));
}
