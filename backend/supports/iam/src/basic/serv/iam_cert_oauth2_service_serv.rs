use std::collections::HashSet;

use bios_basic::rbum::{
    dto::{
        rbum_cert_conf_dto::{RbumCertConfAddReq, RbumCertConfModifyReq, RbumCertConfSummaryResp},
        rbum_filer_dto::{RbumBasicFilterReq, RbumCertConfFilterReq, RbumCertFilterReq, RbumItemRelFilterReq},
    },
    rbum_enumeration::{RbumCertConfStatusKind, RbumRelFromKind},
    serv::{rbum_cert_serv::RbumCertConfServ, rbum_crud_serv::RbumCrudOperation as _, rbum_item_serv::RbumItemCrudOperation as _},
};
use serde::{Deserialize, Serialize};
use tardis::{
    basic::{dto::TardisContext, field::TrimString, result::TardisResult},
    chrono::Utc,
    web::web_resp::TardisPage,
    TardisFuns, TardisFunsInst,
};

use crate::{
    basic::{
        dto::{
            iam_app_dto::IamAppSummaryResp,
            iam_cert_conf_dto::{IamCertConfOAuth2ServiceAddOrModifyReq, IamCertConfOAuth2ServiceExt, IamCertConfOAuth2ServiceResp, IamCertConfOAuth2ServiceScopeModifyReq},
            iam_cert_dto::{
                IamCertOAuth2ServiceCodeAddReq, IamCertOAuth2ServiceCodeVerifyReq, IamCertOAuth2ServiceRefreshTokenReq, IamOauth2AppResp, IamOauth2IntrospectResp,
                IamOauth2RoleMemberResp, IamOauth2TokenMeta, IamOauth2TokenResp, IamOauth2UserInfoResp,
            },
            iam_filer_dto::{IamAccountFilterReq, IamAppFilterReq, IamRoleFilterReq},
        },
        serv::{iam_account_serv::IamAccountServ, iam_app_serv::IamAppServ, iam_cert_serv::IamCertServ, iam_key_cache_serv::IamIdentCacheServ, iam_role_serv::IamRoleServ},
    },
    iam_config::{IamBasicConfigApi as _, IamConfig},
    iam_enumeration::{IamCertExtKind, IamCertKernelKind, IamCertTokenKind, IamRelKind, IamRoleKind, OAuth2ResponseType, Oauth2GrantType, Oauth2TokenType},
};

/// userinfo / introspect 返回的身份提供方标识
const OAUTH2_PROVIDER: &str = "bios-iam";

const REDIS_CODE_KEY: &str = "iam:oauth2:service:code:";
const REDIS_REFRESH_TOKEN_KEY: &str = "iam:oauth2:service:refresh_token:";
const OAUTH2_ALL_SCOPE: &str = "all";
const OAUTH2_FIXED_SCOPE_CODES: [&str; 3] = ["iam.userinfo.read", "iam.app.read", "iam.app.role_member.read"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OAuth2ScopeValidationError {
    Empty,
    Unsupported,
    NotAllowed,
}

fn normalize_oauth2_allowed_scopes(scopes: &[String]) -> Result<Vec<String>, OAuth2ScopeValidationError> {
    let mut normalized = HashSet::new();
    let mut has_all = false;
    for scope in scopes {
        if scope == OAUTH2_ALL_SCOPE {
            has_all = true;
            continue;
        }
        if !OAUTH2_FIXED_SCOPE_CODES.contains(&scope.as_str()) {
            return Err(OAuth2ScopeValidationError::Unsupported);
        }
        normalized.insert(scope.as_str());
    }

    if has_all {
        return Ok(vec![OAUTH2_ALL_SCOPE.to_string()]);
    }

    Ok(OAUTH2_FIXED_SCOPE_CODES.iter().filter(|scope| normalized.contains(**scope)).map(|scope| scope.to_string()).collect())
}

fn validate_oauth2_scopes(requested: &str, allowed: &[String]) -> Result<Vec<String>, OAuth2ScopeValidationError> {
    let requested = requested.split_ascii_whitespace().collect::<HashSet<_>>();
    if requested.is_empty() {
        return Err(OAuth2ScopeValidationError::Empty);
    }

    let allowed = normalize_oauth2_allowed_scopes(allowed)?;
    let allowed_all = allowed.iter().any(|scope| scope == OAUTH2_ALL_SCOPE);
    if requested.iter().any(|scope| *scope != OAUTH2_ALL_SCOPE && !OAUTH2_FIXED_SCOPE_CODES.contains(scope)) {
        return Err(OAuth2ScopeValidationError::Unsupported);
    }
    if requested.contains(OAUTH2_ALL_SCOPE) {
        if allowed_all {
            return Ok(vec![OAUTH2_ALL_SCOPE.to_string()]);
        }
        return Err(OAuth2ScopeValidationError::NotAllowed);
    }
    for scope in &requested {
        if !allowed_all && !allowed.iter().any(|allowed_scope| allowed_scope == scope) {
            return Err(OAuth2ScopeValidationError::NotAllowed);
        }
    }

    Ok(OAUTH2_FIXED_SCOPE_CODES.iter().filter(|scope| requested.contains(**scope)).map(|scope| scope.to_string()).collect())
}

fn narrow_oauth2_scopes(granted: &[String], requested: Option<&str>) -> Result<Vec<String>, OAuth2ScopeValidationError> {
    let granted = normalize_oauth2_allowed_scopes(granted)?;
    if granted.is_empty() {
        return Err(OAuth2ScopeValidationError::Empty);
    }

    match requested {
        Some(requested) => validate_oauth2_scopes(requested, &granted),
        None => Ok(granted),
    }
}

fn intersect_oauth2_scopes(granted: &[String], allowed: &[String]) -> Result<Vec<String>, OAuth2ScopeValidationError> {
    let granted = normalize_oauth2_allowed_scopes(granted)?;
    let allowed = normalize_oauth2_allowed_scopes(allowed)?;
    let granted_all = granted.iter().any(|scope| scope == OAUTH2_ALL_SCOPE);
    let allowed_all = allowed.iter().any(|scope| scope == OAUTH2_ALL_SCOPE);
    if granted_all {
        return if allowed_all {
            Ok(vec![OAUTH2_ALL_SCOPE.to_string()])
        } else if allowed.is_empty() {
            Err(OAuth2ScopeValidationError::NotAllowed)
        } else {
            Ok(allowed)
        };
    }
    if allowed_all {
        return if granted.is_empty() { Err(OAuth2ScopeValidationError::NotAllowed) } else { Ok(granted) };
    }
    let scopes = OAUTH2_FIXED_SCOPE_CODES
        .iter()
        .filter(|scope| granted.iter().any(|granted_scope| granted_scope == **scope) && allowed.iter().any(|allowed_scope| allowed_scope == **scope))
        .map(|scope| scope.to_string())
        .collect::<Vec<_>>();
    if scopes.is_empty() {
        return Err(OAuth2ScopeValidationError::NotAllowed);
    }
    Ok(scopes)
}

fn scope_validation_error(funs: &TardisFunsInst, operation: &str, error: OAuth2ScopeValidationError) -> tardis::basic::error::TardisError {
    let (message, code) = match error {
        OAuth2ScopeValidationError::Empty => ("scope is required", "400-oauth2-scope-required"),
        OAuth2ScopeValidationError::Unsupported => ("unsupported OAuth2 scope", "400-oauth2-unsupported-scope"),
        OAuth2ScopeValidationError::NotAllowed => ("scope is not allowed for this client", "400-oauth2-scope-not-allowed"),
    };
    funs.err().bad_request("oauth2", operation, message, code)
}

pub struct IamCertOAuth2ServiceServ;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IamCertOAuth2ServiceCode {
    pub ctx: TardisContext,
    pub client_id: String,
    pub redirect_uri: String,
    pub scope: String,
    pub state: Option<String>,
    pub created_at: i64,
    pub used: bool,
}

// 刷新令牌信息结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IamOAuth2RefreshTokenInfo {
    pub user_id: String,
    pub client_id: String,
    pub scope: String,
    pub expires_at: i64,
}

impl IamCertOAuth2ServiceServ {
    pub async fn add_cert_conf(add_req: &IamCertConfOAuth2ServiceAddOrModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let scopes = normalize_oauth2_allowed_scopes(&add_req.scope).map_err(|error| scope_validation_error(funs, "add_client", error))?;
        let client_id = TardisFuns::crypto.key.generate_ak()?;
        let client_secret = TardisFuns::crypto.key.generate_sk(&client_id)?;
        RbumCertConfServ::add_rbum(
            &mut RbumCertConfAddReq {
                kind: TrimString(IamCertExtKind::OAuth2Service.to_string()),
                supplier: Some(TrimString(client_id.clone())),
                name: add_req.name.clone(),
                note: None,
                ak_note: None,
                ak_rule: None,
                sk_note: None,
                sk_rule: None,
                ext: Some(TardisFuns::json.obj_to_string(&IamCertConfOAuth2ServiceExt {
                    client_id,
                    client_secret,
                    redirect_uris: add_req.redirect_uris.clone(),
                    scope: scopes,
                })?),
                sk_need: Some(false),
                sk_dynamic: Some(false),
                sk_encrypted: Some(false),
                repeatable: None,
                is_basic: Some(false),
                rest_by_kinds: None,
                expire_sec: Some(add_req.access_token_expire_sec.unwrap_or(60 * 60 * 24 * 7)),
                sk_lock_cycle_sec: None,
                sk_lock_err_times: None,
                sk_lock_duration_sec: None,
                coexist_num: Some(1),
                conn_uri: add_req.redirect_uris.first().cloned(),
                status: RbumCertConfStatusKind::Enabled,
                rel_rbum_domain_id: funs.iam_basic_domain_iam_id(),
                rel_rbum_item_id: add_req.rel_rbum_item_id.clone(),
            },
            funs,
            ctx,
        )
        .await
    }

    /// 只更新允许的 scope，不更换客户端凭证或回调地址。
    pub async fn modify_allowed_scopes(id: &str, modify_req: &IamCertConfOAuth2ServiceScopeModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let scopes = normalize_oauth2_allowed_scopes(&modify_req.scope).map_err(|error| scope_validation_error(funs, "modify_client_scopes", error))?;
        let cert_conf = RbumCertConfServ::get_rbum(
            id,
            &RbumCertConfFilterReq {
                basic: RbumBasicFilterReq::default(),
                kind: Some(TrimString(IamCertExtKind::OAuth2Service.to_string())),
                supplier: None,
                status: Some(RbumCertConfStatusKind::Enabled),
                rel_rbum_domain_id: Some(funs.iam_basic_domain_iam_id()),
                rel_rbum_item_id: None,
            },
            funs,
            ctx,
        )
        .await?;
        let mut ext = TardisFuns::json.str_to_obj::<IamCertConfOAuth2ServiceExt>(&cert_conf.ext)?;
        ext.scope = scopes;
        RbumCertConfServ::modify_rbum(
            id,
            &mut RbumCertConfModifyReq {
                name: None,
                note: None,
                ak_note: None,
                ak_rule: None,
                sk_note: None,
                sk_rule: None,
                ext: Some(TardisFuns::json.obj_to_string(&ext)?),
                sk_need: None,
                sk_encrypted: None,
                repeatable: None,
                is_basic: None,
                rest_by_kinds: None,
                expire_sec: None,
                sk_lock_cycle_sec: None,
                sk_lock_err_times: None,
                sk_lock_duration_sec: None,
                coexist_num: None,
                conn_uri: None,
                status: None,
            },
            funs,
            ctx,
        )
        .await
    }

    /// 获取OAuth2服务证书配置
    pub async fn get_cert_conf(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<IamCertConfOAuth2ServiceResp> {
        let cert_conf = RbumCertConfServ::get_rbum(
            id,
            &RbumCertConfFilterReq {
                basic: RbumBasicFilterReq::default(),
                kind: Some(TrimString(IamCertExtKind::OAuth2Service.to_string())),
                supplier: None,
                status: Some(RbumCertConfStatusKind::Enabled),
                rel_rbum_domain_id: Some(funs.iam_basic_domain_iam_id()),
                rel_rbum_item_id: None,
            },
            funs,
            ctx,
        )
        .await?;

        let ext = TardisFuns::json.str_to_obj::<IamCertConfOAuth2ServiceExt>(&cert_conf.ext)?;

        Ok(IamCertConfOAuth2ServiceResp {
            id: cert_conf.id,
            name: cert_conf.name,
            client_id: ext.client_id,
            client_secret: ext.client_secret,
            access_token_expire_sec: cert_conf.expire_sec,
            redirect_uris: ext.redirect_uris,
            scope: normalize_oauth2_allowed_scopes(&ext.scope).map_err(|error| scope_validation_error(funs, "get_client", error))?,
        })
    }

    /// 列出OAuth2服务证书配置
    pub async fn find_cert_confs(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<IamCertConfOAuth2ServiceResp>> {
        let cert_confs = RbumCertConfServ::find_rbums(
            &RbumCertConfFilterReq {
                basic: RbumBasicFilterReq::default(),
                kind: Some(TrimString(IamCertExtKind::OAuth2Service.to_string())),
                supplier: None,
                status: Some(RbumCertConfStatusKind::Enabled),
                rel_rbum_domain_id: Some(funs.iam_basic_domain_iam_id()),
                rel_rbum_item_id: None,
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;

        let mut result = Vec::new();
        for cert_conf in cert_confs {
            let ext = TardisFuns::json.str_to_obj::<IamCertConfOAuth2ServiceExt>(&cert_conf.ext)?;
            result.push(IamCertConfOAuth2ServiceResp {
                id: cert_conf.id.clone(),
                name: cert_conf.name,
                client_id: ext.client_id,
                client_secret: ext.client_secret,
                access_token_expire_sec: cert_conf.expire_sec,
                redirect_uris: ext.redirect_uris.clone(),
                scope: normalize_oauth2_allowed_scopes(&ext.scope).map_err(|error| scope_validation_error(funs, "list_clients", error))?,
            });
        }

        Ok(result)
    }

    /// 删除OAuth2服务证书配置
    pub async fn delete_cert_conf(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<u64> {
        RbumCertConfServ::delete_rbum(id, funs, ctx).await
    }

    async fn get_cert_conf_by_client_id(client_id: &str, funs: &TardisFunsInst) -> TardisResult<RbumCertConfSummaryResp> {
        let global_ctx = TardisContext::default();

        let mut conf = RbumCertConfServ::find_rbums(
            &RbumCertConfFilterReq {
                basic: RbumBasicFilterReq {
                    ignore_scope: true,
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                kind: Some(TrimString(IamCertExtKind::OAuth2Service.to_string())),
                supplier: Some(client_id.to_string()),
                status: Some(RbumCertConfStatusKind::Enabled),
                rel_rbum_domain_id: Some(funs.iam_basic_domain_iam_id()),
                rel_rbum_item_id: None,
            },
            None,
            None,
            funs,
            &global_ctx,
        )
        .await?;

        if conf.is_empty() {
            return Err(funs.err().unauthorized("oauth2", "generate_code", &format!("client not found: {}", client_id), "401-oauth2-invalid-client"));
        } else if conf.len() > 1 {
            return Err(funs.err().bad_request("oauth2", "generate_code", "multiple_clients_found", "400-oauth2-multiple-clients-found"));
        }

        Ok(conf.remove(0))
    }

    /// 改进的生成授权码方法 - 使用配置中的有效期
    pub async fn generate_code(add_req: &IamCertOAuth2ServiceCodeAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        // 1. 验证响应类型 - 目前只支持code模式
        if add_req.response_type != OAuth2ResponseType::Code {
            return Err(funs.err().bad_request("oauth2", "generate_code", "unsupported_response_type", "400-oauth2-unsupported-response-type"));
        }

        let code = TardisFuns::field.nanoid();

        // 2. 获取客户端配置
        let conf = Self::get_cert_conf_by_client_id(&add_req.client_id, funs).await?;

        let ext = TardisFuns::json.str_to_obj::<IamCertConfOAuth2ServiceExt>(&conf.ext)?;

        let requested_scope = add_req.scope.to_string();
        let scopes = validate_oauth2_scopes(&requested_scope, &ext.scope).map_err(|error| scope_validation_error(funs, "generate_code", error))?;

        // 3. 验证重定向URI（必须在已注册列表中）
        if !ext.redirect_uris.iter().any(|uri| *uri == add_req.redirect_uri.to_string()) {
            return Err(funs.err().bad_request("oauth2", "generate_code", "invalid_redirect_uri", "400-oauth2-invalid-redirect-uri"));
        }

        // 4. 构建授权码信息
        let code_info = IamCertOAuth2ServiceCode {
            ctx: ctx.clone(),
            client_id: add_req.client_id.to_string(),
            redirect_uri: add_req.redirect_uri.to_string(),
            scope: scopes.join(" "),
            state: add_req.state.clone(),
            created_at: Utc::now().timestamp(),
            used: false,
        };

        // 5. 存储到Redis - 使用配置中的有效期
        let iam_config = funs.conf::<IamConfig>();
        let expire_sec = iam_config.oauth2_auth_code_expire_sec as u64;
        funs.cache().set_ex(&format!("{}{}", REDIS_CODE_KEY, code), &TardisFuns::json.obj_to_string(&code_info)?, expire_sec).await?;

        Ok(code)
    }

    /// 改进的验证授权码并生成令牌方法
    pub async fn verify_code_and_generate_token(req: &IamCertOAuth2ServiceCodeVerifyReq, funs: &TardisFunsInst) -> TardisResult<IamOauth2TokenResp> {
        // 1. 验证grant_type
        if req.grant_type != Oauth2GrantType::AuthorizationCode {
            return Err(funs.err().bad_request("oauth2", "verify_code", "unsupported_grant_type", "400-oauth2-unsupported-grant-type"));
        }

        // 2. 获取客户端配置并验证client_secret
        let conf = Self::get_cert_conf_by_client_id(&req.client_id, funs).await?;

        let ext = TardisFuns::json.str_to_obj::<IamCertConfOAuth2ServiceExt>(&conf.ext)?;

        // 验证客户端密钥
        if req.client_secret != ext.client_secret {
            return Err(funs.err().unauthorized("oauth2", "verify_code", "invalid_client", "401-oauth2-invalid-client"));
        }

        // 3. 获取并验证授权码
        let code_data = funs.cache().get(&format!("{}{}", REDIS_CODE_KEY, req.code)).await?;
        let code_info: IamCertOAuth2ServiceCode = match code_data {
            Some(data) => TardisFuns::json.str_to_obj(&data)?,
            None => return Err(funs.err().unauthorized("oauth2", "verify_code", "invalid_or_expired_code", "401-oauth2-invalid-code")),
        };

        // 4. 验证授权码状态和参数
        if code_info.used {
            return Err(funs.err().unauthorized("oauth2", "verify_code", "code_already_used", "401-oauth2-code-used"));
        }

        if code_info.client_id != req.client_id {
            return Err(funs.err().unauthorized("oauth2", "verify_code", "invalid_client", "401-oauth2-invalid-client"));
        }

        if let Some(redirect_uri) = &req.redirect_uri {
            if code_info.redirect_uri != *redirect_uri {
                return Err(funs.err().unauthorized("oauth2", "verify_code", "invalid_redirect_uri", "401-oauth2-invalid-redirect-uri"));
            }
        }

        let scopes = validate_oauth2_scopes(&code_info.scope, &ext.scope).map_err(|error| scope_validation_error(funs, "verify_code", error))?;

        // 5. 标记授权码为已使用
        let mut used_code_info = code_info.clone();
        used_code_info.used = true;
        funs.cache()
            .set_ex(
                &format!("{}{}", REDIS_CODE_KEY, req.code),
                &TardisFuns::json.obj_to_string(&used_code_info)?,
                60, // 保留1分钟用于防重放攻击检测
            )
            .await?;

        // 6. 生成访问令牌和刷新令牌
        let access_token = TardisFuns::crypto.key.generate_token()?;
        let refresh_token = TardisFuns::crypto.key.generate_token()?;

        let iam_config = funs.conf::<IamConfig>();
        let access_token_expire_sec = conf.expire_sec;

        // 7. 存储访问令牌（复用现有的令牌缓存系统）
        Self::add_oauth2_token(
            &access_token,
            &req.client_id,
            &scopes,
            &code_info.ctx.owner,
            access_token_expire_sec,
            conf.coexist_num,
            funs,
        )
        .await?;

        // 8. 存储刷新令牌
        let refresh_token_info = IamOAuth2RefreshTokenInfo {
            user_id: code_info.ctx.owner.clone(),
            client_id: req.client_id.clone(),
            scope: scopes.join(" "),
            expires_at: Utc::now().timestamp() + iam_config.oauth2_refresh_token_expire_sec as i64,
        };
        funs.cache()
            .set_ex(
                &format!("{}{}", REDIS_REFRESH_TOKEN_KEY, refresh_token),
                &TardisFuns::json.obj_to_string(&refresh_token_info)?,
                iam_config.oauth2_refresh_token_expire_sec as u64,
            )
            .await?;

        Ok(IamOauth2TokenResp {
            access_token,
            token_type: Oauth2TokenType::Bearer,
            expires_in: access_token_expire_sec as i64,
            refresh_token: Some(refresh_token),
            scope: Some(scopes.join(" ")),
        })
    }

    /// 刷新令牌方法
    pub async fn refresh_token(req: &IamCertOAuth2ServiceRefreshTokenReq, funs: &TardisFunsInst) -> TardisResult<IamOauth2TokenResp> {
        // 1. 验证grant_type
        if req.grant_type != Oauth2GrantType::RefreshToken {
            return Err(funs.err().bad_request("oauth2", "refresh_token", "unsupported_grant_type", "400-oauth2-unsupported-grant-type"));
        }

        // 2. 获取刷新令牌信息
        let refresh_token_data = funs.cache().get(&format!("{}{}", REDIS_REFRESH_TOKEN_KEY, req.refresh_token)).await?;
        let refresh_token_info: IamOAuth2RefreshTokenInfo = match refresh_token_data {
            Some(data) => TardisFuns::json.str_to_obj(&data)?,
            None => return Err(funs.err().unauthorized("oauth2", "refresh_token", "invalid_refresh_token", "401-oauth2-invalid-refresh-token")),
        };

        // 3. 验证客户端和刷新令牌
        if refresh_token_info.client_id != req.client_id {
            return Err(funs.err().unauthorized("oauth2", "refresh_token", "invalid_client", "401-oauth2-invalid-client"));
        }

        let now = Utc::now().timestamp();
        if now >= refresh_token_info.expires_at {
            return Err(funs.err().unauthorized("oauth2", "refresh_token", "refresh_token_expired", "401-oauth2-refresh-token-expired"));
        }

        // 4. 获取凭证配置
        let conf = Self::get_cert_conf_by_client_id(&refresh_token_info.client_id, funs).await?;
        let ext = TardisFuns::json.str_to_obj::<IamCertConfOAuth2ServiceExt>(&conf.ext)?;
        if req.client_secret.as_deref() != Some(ext.client_secret.as_str()) {
            return Err(funs.err().unauthorized("oauth2", "refresh_token", "invalid_client", "401-oauth2-invalid-client"));
        }
        let granted = refresh_token_info.scope.split_ascii_whitespace().map(str::to_string).collect::<Vec<_>>();
        let current_grant = intersect_oauth2_scopes(&granted, &ext.scope).map_err(|error| scope_validation_error(funs, "refresh_token", error))?;
        let scopes = narrow_oauth2_scopes(&current_grant, req.scope.as_deref()).map_err(|error| scope_validation_error(funs, "refresh_token", error))?;
        let mut narrowed_refresh_token_info = refresh_token_info.clone();
        narrowed_refresh_token_info.scope = scopes.join(" ");
        funs.cache()
            .set_ex(
                &format!("{}{}", REDIS_REFRESH_TOKEN_KEY, req.refresh_token),
                &TardisFuns::json.obj_to_string(&narrowed_refresh_token_info)?,
                (refresh_token_info.expires_at - now) as u64,
            )
            .await?;
        let access_token_expire_sec = conf.expire_sec;

        // 5. 生成新的访问令牌
        let new_access_token = TardisFuns::crypto.key.generate_token()?;

        // 6. 存储新的访问令牌
        Self::add_oauth2_token(
            &new_access_token,
            &refresh_token_info.client_id,
            &scopes,
            &refresh_token_info.user_id,
            access_token_expire_sec,
            conf.coexist_num,
            funs,
        )
        .await?;

        Ok(IamOauth2TokenResp {
            access_token: new_access_token,
            token_type: Oauth2TokenType::Bearer,
            expires_in: access_token_expire_sec as i64,
            refresh_token: Some(req.refresh_token.clone()), // 保持相同的刷新令牌
            scope: Some(scopes.join(" ")),
        })
    }

    async fn add_oauth2_token(
        access_token: &str,
        client_id: &str,
        scopes: &[String],
        account_id: &str,
        expire_sec: i64,
        coexist_num: i16,
        funs: &TardisFunsInst,
    ) -> TardisResult<()> {
        let key = format!("{}{}", crate::iam_constants::IAM_OAUTH2_TOKEN_META_CACHE_KEY_PREFIX, access_token);
        let meta = IamOauth2TokenMeta {
            version: 1,
            client_id: client_id.to_string(),
            scopes: scopes.to_vec(),
        };
        let value = TardisFuns::json.obj_to_string(&meta)?;
        if expire_sec > 0 {
            funs.cache().set_ex(&key, &value, expire_sec as u64).await?;
        } else {
            funs.cache().set(&key, &value).await?;
        }

        if let Err(error) = IamIdentCacheServ::add_token(access_token, &IamCertTokenKind::TokenOauth2, account_id, None, expire_sec, coexist_num, funs).await {
            let _ = funs.cache().del(&key).await;
            return Err(error);
        }
        Ok(())
    }

    /// 保持向后兼容的简化方法
    pub async fn verify_code(add_req: &IamCertOAuth2ServiceCodeVerifyReq, funs: &TardisFunsInst) -> TardisResult<String> {
        let token_resp = Self::verify_code_and_generate_token(add_req, funs).await?;
        Ok(token_resp.access_token)
    }

    /// 解析访问令牌并返回对应的账号 ID
    ///
    /// 仅接受 OAuth2 类型（`TokenOauth2`）的令牌，避免普通登录令牌被用于换取用户信息。
    async fn resolve_account_id_by_access_token(access_token: &str, funs: &TardisFunsInst) -> TardisResult<String> {
        let token_info = funs
            .cache()
            .get(format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, access_token).as_str())
            .await?
            .ok_or_else(|| funs.err().unauthorized("oauth2", "userinfo", "invalid_or_expired_token", "401-oauth2-invalid-token"))?;
        let mut parts = token_info.split(',');
        let token_kind = parts.next().unwrap_or_default();
        let account_id = parts.next().unwrap_or_default();
        if token_kind != IamCertTokenKind::TokenOauth2.to_string() || account_id.is_empty() {
            return Err(funs.err().unauthorized("oauth2", "userinfo", "invalid_token_kind", "401-oauth2-invalid-token"));
        }
        Ok(account_id.to_string())
    }

    /// 根据访问令牌返回用户信息（Provider 侧 userinfo 端点的服务实现）
    pub async fn get_userinfo(access_token: &str, funs: &TardisFunsInst) -> TardisResult<IamOauth2UserInfoResp> {
        let account_id = Self::resolve_account_id_by_access_token(access_token, funs).await?;
        Self::build_userinfo_by_account_id(&account_id, None, funs).await
    }

    /// 查询当前上下文账号可见的应用，包含直接关联应用和应用集合授权应用。
    pub async fn find_apps(ctx: &TardisContext, funs: &TardisFunsInst) -> TardisResult<Vec<IamOauth2AppResp>> {
        Ok(Self::find_visible_app_summaries(ctx, funs)
            .await?
            .into_iter()
            .map(|app| IamOauth2AppResp {
                id: app.id,
                name: app.name,
                icon: app.icon,
                kind: app.kind,
                description: app.description,
            })
            .collect())
    }

    /// 查询指定应用的指定角色成员；调用方只能查询当前账号可见的应用。
    pub async fn find_role_members(
        app_id: &str,
        role_code: &str,
        page_number: u32,
        page_size: u32,
        ctx: &TardisContext,
        funs: &TardisFunsInst,
    ) -> TardisResult<TardisPage<IamOauth2RoleMemberResp>> {
        let role_code = role_code.trim();
        if role_code.is_empty() {
            return Err(funs.err().bad_request("oauth2", "role_members", "role_code is required", "400-oauth2-role-code-required"));
        }

        let visible_app = Self::find_visible_app_summaries(ctx, funs)
            .await?
            .into_iter()
            .find(|app| app.id == app_id)
            .ok_or_else(|| funs.err().not_found("oauth2", "role_members", "app is not found", "404-oauth2-app-not-found"))?;

        // The app path was returned by the visibility query above. It may belong
        // to another tenant when the account has platform-level Apps access.
        let app_ctx = TardisContext {
            own_paths: visible_app.own_paths.clone(),
            ..ctx.clone()
        };
        let global_ctx = TardisContext {
            own_paths: "".to_string(),
            ..app_ctx.clone()
        };

        let empty_page = || TardisPage {
            page_size: page_size as u64,
            page_number: page_number as u64,
            total_size: 0,
            records: Vec::new(),
        };

        let role = IamRoleServ::find_one_item(
            &IamRoleFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some(app_ctx.own_paths.clone()),
                    with_sub_own_paths: false,
                    ignore_scope: true,
                    enabled: Some(true),
                    codes: Some(vec![format!("{}:{}", app_id, role_code)]),
                    ..Default::default()
                },
                kind: Some(IamRoleKind::App),
                ..Default::default()
            },
            funs,
            &global_ctx,
        )
        .await?;
        let role = match role {
            Some(role) => Some(role),
            None => {
                IamRoleServ::find_one_item(
                    &IamRoleFilterReq {
                        basic: RbumBasicFilterReq {
                            own_paths: Some(app_ctx.own_paths.clone()),
                            with_sub_own_paths: false,
                            ignore_scope: true,
                            enabled: Some(true),
                            codes: Some(vec![role_code.to_string()]),
                            ..Default::default()
                        },
                        kind: Some(IamRoleKind::App),
                        ..Default::default()
                    },
                    funs,
                    &global_ctx,
                )
                .await?
            }
        };
        let Some(role) = role else {
            return Ok(empty_page());
        };
        let role_id = role.id;
        // Filter disabled accounts before pagination so the page metadata and
        // page windows describe the data actually exposed by this endpoint.
        let accounts = IamAccountServ::paginate_items(
            &IamAccountFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ignore_scope: true,
                    enabled: Some(true),
                    ..Default::default()
                },
                rel: Some(RbumItemRelFilterReq {
                    rel_by_from: true,
                    optional: false,
                    tag: Some(IamRelKind::IamAccountRole.to_string()),
                    from_rbum_kind: Some(RbumRelFromKind::Item),
                    rel_item_id: Some(role_id),
                    own_paths: Some(app_ctx.own_paths.clone()),
                    disabled: Some(false),
                    ..Default::default()
                }),
                ..Default::default()
            },
            page_number,
            page_size,
            Some(false),
            None,
            funs,
            &global_ctx,
        )
        .await?;
        let TardisPage {
            page_size,
            page_number,
            total_size,
            records: account_records,
        } = accounts;
        let records = account_records
            .into_iter()
            .map(|account| IamOauth2RoleMemberResp {
                id: account.id,
                name: account.name,
                avatar: account.icon,
            })
            .collect();

        Ok(TardisPage {
            page_size,
            page_number,
            total_size,
            records,
        })
    }

    async fn find_visible_app_summaries(ctx: &TardisContext, funs: &TardisFunsInst) -> TardisResult<Vec<IamAppSummaryResp>> {
        let query_ctx = IamCertServ::use_sys_or_tenant_ctx_unsafe(ctx.clone())?;
        let direct_apps = IamAppServ::find_items(
            &IamAppFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    enabled: Some(true),
                    ..Default::default()
                },
                rel: Some(RbumItemRelFilterReq {
                    rel_by_from: false,
                    optional: false,
                    tag: Some(IamRelKind::IamAccountApp.to_string()),
                    from_rbum_kind: Some(RbumRelFromKind::Item),
                    rel_item_id: Some(query_ctx.owner.clone()),
                    disabled: Some(false),
                    ..Default::default()
                }),
                ..Default::default()
            },
            None,
            None,
            funs,
            &query_ctx,
        )
        .await?;

        let mut visible_app_ids = direct_apps.iter().map(|app| app.id.clone()).collect::<HashSet<_>>();
        let extra_apps = IamAccountServ::get_account_apps_from_all_sets(&query_ctx.owner, &visible_app_ids, funs, &query_ctx).await?;
        let extra_app_ids = extra_apps.into_iter().map(|app| app.app_id).filter(|app_id| visible_app_ids.insert(app_id.clone())).collect::<Vec<_>>();

        if extra_app_ids.is_empty() {
            return Ok(direct_apps);
        }

        let global_ctx = TardisContext {
            own_paths: "".to_string(),
            ..query_ctx
        };
        let extra_apps = IamAppServ::find_items(
            &IamAppFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ignore_scope: true,
                    enabled: Some(true),
                    ids: Some(extra_app_ids),
                    ..Default::default()
                },
                ..Default::default()
            },
            None,
            None,
            funs,
            &global_ctx,
        )
        .await?;

        let mut apps = direct_apps;
        apps.extend(extra_apps);
        Ok(apps)
    }

    /// 根据账号 ID 构建用户信息（供基于登录上下文 `TardisContext` 的 userinfo 端点复用）
    ///
    /// `tenant_id_override` 用于指定返回的 `tenant_id`：基于登录上下文时传入 `ctx.own_paths`，
    /// 以返回当前请求所处的租户；为 `None` 时回退到账号自身所属租户。
    pub async fn build_userinfo_by_account_id(account_id: &str, tenant_id_override: Option<&str>, funs: &TardisFunsInst) -> TardisResult<IamOauth2UserInfoResp> {
        let account_id = account_id.to_string();
        // 以全局视角读取账号信息，避免账号上下文未缓存时无法定位租户
        let mock_ctx = TardisContext {
            own_paths: "".to_string(),
            owner: account_id.clone(),
            ..Default::default()
        };
        let account = IamAccountServ::get_item(
            &account_id,
            &IamAccountFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ignore_scope: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            &mock_ctx,
        )
        .await?;
        let certs = IamCertServ::find_certs(
            &RbumCertFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some(account.own_paths.clone()),
                    with_sub_own_paths: true,
                    ignore_scope: true,
                    ..Default::default()
                },
                rel_rbum_id: Some(account_id.clone()),
                ..Default::default()
            },
            None,
            None,
            funs,
            &mock_ctx,
        )
        .await
        .unwrap_or_default();
        let mail = certs.iter().find(|c| c.kind == IamCertKernelKind::MailVCode.to_string()).map(|c| c.ak.clone());
        let phone = certs.iter().find(|c| c.kind == IamCertKernelKind::PhoneVCode.to_string()).map(|c| c.ak.clone());
        Ok(IamOauth2UserInfoResp {
            provider: OAUTH2_PROVIDER.to_string(),
            sub: account.id,
            tenant_id: tenant_id_override.map(|t| t.to_string()).unwrap_or(account.own_paths),
            name: account.name,
            mail,
            phone,
            employee_no: if account.employee_code.is_empty() { None } else { Some(account.employee_code) },
            id_card_no: if account.id_card_no.is_empty() { None } else { Some(account.id_card_no) },
            disabled: account.disabled,
        })
    }

    /// 令牌内省（Provider 侧 introspect 端点的服务实现）
    pub async fn introspect(token: &str, funs: &TardisFunsInst) -> TardisResult<IamOauth2IntrospectResp> {
        match Self::resolve_account_id_by_access_token(token, funs).await {
            Ok(account_id) => Ok(IamOauth2IntrospectResp {
                active: true,
                provider: Some(OAUTH2_PROVIDER.to_string()),
                sub: Some(account_id),
            }),
            Err(_) => Ok(IamOauth2IntrospectResp {
                active: false,
                provider: None,
                sub: None,
            }),
        }
    }
}

#[cfg(test)]
mod oauth2_scope_tests {
    use super::{intersect_oauth2_scopes, narrow_oauth2_scopes, normalize_oauth2_allowed_scopes, validate_oauth2_scopes, OAUTH2_FIXED_SCOPE_CODES};
    use crate::basic::dto::{
        iam_cert_conf_dto::{IamCertConfOAuth2ServiceAddOrModifyReq, IamCertConfOAuth2ServiceExt},
        iam_cert_dto::IamOauth2TokenMeta,
    };
    use crate::iam_constants::IAM_OAUTH2_TOKEN_META_CACHE_KEY_PREFIX;
    use tardis::TardisFuns;

    fn scopes(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn requested_scopes_are_deduplicated_and_canonicalized() {
        let allowed = scopes(&["iam.app.role_member.read", "iam.app.read", "iam.userinfo.read", "iam.app.read"]);

        assert_eq!(
            validate_oauth2_scopes("iam.app.read iam.userinfo.read iam.app.role_member.read iam.app.read", &allowed).unwrap(),
            scopes(&["iam.userinfo.read", "iam.app.read", "iam.app.role_member.read"])
        );
    }

    #[test]
    fn empty_or_unknown_scope_is_rejected() {
        assert!(validate_oauth2_scopes("iam.userinfo.read", &[]).is_err());
        assert!(normalize_oauth2_allowed_scopes(&scopes(&["iam.scope.unknown"])).is_err());
        assert!(validate_oauth2_scopes("iam.scope.unknown", &scopes(&["all"])).is_err());
    }

    #[test]
    fn all_scope_is_a_formal_client_wildcard_and_mixed_scopes_canonicalize_to_all() {
        assert_eq!(normalize_oauth2_allowed_scopes(&scopes(&["all", "iam.app.read"])).unwrap(), scopes(&["all"]));
        assert_eq!(validate_oauth2_scopes("all", &scopes(&["all"])).unwrap(), scopes(&["all"]));
        assert_eq!(validate_oauth2_scopes("iam.app.read", &scopes(&["all"])).unwrap(), scopes(&["iam.app.read"]));
        assert_eq!(validate_oauth2_scopes("all iam.app.read", &scopes(&["all"])).unwrap(), scopes(&["all"]));
        assert!(validate_oauth2_scopes("all", &scopes(&["iam.app.read"])).is_err());
        assert!(validate_oauth2_scopes("all iam.app.read", &scopes(&["iam.app.read"])).is_err());
        assert!(validate_oauth2_scopes("all iam.scope.unknown", &scopes(&["all"])).is_err());
    }

    #[test]
    fn requested_scope_must_be_allowed_by_client() {
        let allowed = scopes(&["iam.userinfo.read"]);

        assert!(validate_oauth2_scopes("iam.app.read", &allowed).is_err());
        assert!(validate_oauth2_scopes("iam.userinfo.read iam.app.read", &allowed).is_err());
    }

    #[test]
    fn refresh_keeps_or_reduces_granted_scope_only() {
        let granted = scopes(&["iam.userinfo.read", "iam.app.read", "iam.app.role_member.read"]);

        assert_eq!(narrow_oauth2_scopes(&granted, None).unwrap(), granted);
        assert_eq!(narrow_oauth2_scopes(&granted, Some("iam.app.read")).unwrap(), scopes(&["iam.app.read"]));
        assert_eq!(
            intersect_oauth2_scopes(&granted, &scopes(&["iam.userinfo.read", "iam.app.role_member.read"])).unwrap(),
            scopes(&["iam.userinfo.read", "iam.app.role_member.read"])
        );
        assert!(!intersect_oauth2_scopes(&granted, &scopes(&["iam.userinfo.read"])).unwrap().contains(&"iam.app.role_member.read".to_string()));
        assert!(narrow_oauth2_scopes(&granted, Some("all")).is_err());
        assert!(narrow_oauth2_scopes(&granted, Some("iam.app.read iam.userinfo.read iam.app.role_member.read")).is_ok());
        assert!(narrow_oauth2_scopes(&scopes(&["iam.userinfo.read"]), Some("iam.userinfo.read iam.app.read")).is_err());

        assert_eq!(intersect_oauth2_scopes(&scopes(&["all"]), &scopes(&["iam.app.read"])).unwrap(), scopes(&["iam.app.read"]));
        assert_eq!(intersect_oauth2_scopes(&scopes(&["iam.app.read"]), &scopes(&["all"])).unwrap(), scopes(&["iam.app.read"]));
        assert_eq!(intersect_oauth2_scopes(&scopes(&["all"]), &scopes(&["all"])).unwrap(), scopes(&["all"]));
        assert_eq!(narrow_oauth2_scopes(&scopes(&["all"]), Some("iam.app.read")).unwrap(), scopes(&["iam.app.read"]));
        assert_eq!(narrow_oauth2_scopes(&scopes(&["all"]), Some("all iam.app.read")).unwrap(), scopes(&["all"]));
        assert!(narrow_oauth2_scopes(&scopes(&["iam.app.read"]), Some("all")).is_err());
    }

    #[test]
    fn oauth2_token_metadata_matches_gateway_redis_contract() {
        let meta = IamOauth2TokenMeta {
            version: 1,
            client_id: "test-client".to_string(),
            scopes: scopes(&["iam.userinfo.read", "iam.app.read"]),
        };
        let encoded = TardisFuns::json.obj_to_string(&meta).unwrap();
        let decoded = TardisFuns::json.str_to_obj::<IamOauth2TokenMeta>(&encoded).unwrap();

        assert_eq!(decoded.version, 1);
        assert_eq!(decoded.client_id, "test-client");
        assert_eq!(decoded.scopes, meta.scopes);
        assert_eq!(IAM_OAUTH2_TOKEN_META_CACHE_KEY_PREFIX, "iam:cache:token:oauth2:meta:");
        assert_eq!(OAUTH2_FIXED_SCOPE_CODES, ["iam.userinfo.read", "iam.app.read", "iam.app.role_member.read"]);
    }

    #[test]
    fn old_client_requests_and_configs_default_to_no_scopes() {
        let add_req = TardisFuns::json.str_to_obj::<IamCertConfOAuth2ServiceAddOrModifyReq>(r#"{"name":"Hub","redirect_uris":["https://hub.example/callback"]}"#).unwrap();
        let existing_ext = TardisFuns::json
            .str_to_obj::<IamCertConfOAuth2ServiceExt>(r#"{"client_id":"client","client_secret":"secret","redirect_uris":["https://hub.example/callback"]}"#)
            .unwrap();

        assert!(add_req.scope.is_empty());
        assert!(existing_ext.scope.is_empty());
        assert!(validate_oauth2_scopes("iam.userinfo.read", &existing_ext.scope).is_err());
    }
}
