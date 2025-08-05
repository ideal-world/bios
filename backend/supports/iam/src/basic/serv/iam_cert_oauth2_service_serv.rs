use bios_basic::rbum::{
    dto::{
        rbum_cert_conf_dto::{RbumCertConfAddReq, RbumCertConfModifyReq},
        rbum_filer_dto::{RbumBasicFilterReq, RbumCertConfFilterReq},
    },
    rbum_enumeration::RbumCertConfStatusKind,
    serv::{rbum_cert_serv::RbumCertConfServ, rbum_crud_serv::RbumCrudOperation as _},
};
use serde::{Deserialize, Serialize};
use tardis::{
    basic::{dto::TardisContext, field::TrimString, result::TardisResult},
    chrono::Utc,
    TardisFuns, TardisFunsInst,
};

use crate::{
    basic::{
        dto::{
            iam_cert_conf_dto::{IamCertConfOAuth2ServiceAddOrModifyReq, IamCertConfOAuth2ServiceExt, IamCertConfOAuth2ServiceResp},
            iam_cert_dto::{IamCertOAuth2ServiceCodeAddReq, IamCertOAuth2ServiceCodeVerifyReq, IamCertOAuth2ServiceRefreshTokenReq, IamOauth2TokenResp},
        },
        serv::iam_key_cache_serv::IamIdentCacheServ,
    },
    iam_config::{IamBasicConfigApi as _, IamConfig},
    iam_enumeration::{IamCertExtKind, IamCertTokenKind, OAuth2ResponseType, Oauth2GrantType, Oauth2TokenType},
};

const REDIS_CODE_KEY: &str = "iam:oauth2:service:code:";
const REDIS_REFRESH_TOKEN_KEY: &str = "iam:oauth2:service:refresh_token:";

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
                    redirect_uri: add_req.redirect_uri.to_string(),
                    scope: vec![],
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
                conn_uri: Some(add_req.redirect_uri.to_string()),
                status: RbumCertConfStatusKind::Enabled,
                rel_rbum_domain_id: funs.iam_basic_domain_iam_id(),
                rel_rbum_item_id: add_req.rel_rbum_item_id.clone(),
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
            redirect_uri: ext.redirect_uri,
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
                redirect_uri: ext.redirect_uri.to_string(),
            });
        }

        Ok(result)
    }

    /// 删除OAuth2服务证书配置
    pub async fn delete_cert_conf(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<u64> {
        RbumCertConfServ::delete_rbum(id, funs, ctx).await
    }

    /// 改进的生成授权码方法 - 使用配置中的有效期
    pub async fn generate_code(add_req: &IamCertOAuth2ServiceCodeAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        // 1. 验证响应类型 - 目前只支持code模式
        if add_req.response_type != OAuth2ResponseType::Code {
            return Err(funs.err().bad_request("oauth2", "generate_code", "unsupported_response_type", "400-oauth2-unsupported-response-type"));
        }

        let code = TardisFuns::field.nanoid();

        // 2. 获取客户端配置
        let conf = RbumCertConfServ::get_rbum(
            &add_req.client_id,
            &RbumCertConfFilterReq {
                basic: RbumBasicFilterReq {
                    ignore_scope: true,
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                kind: Some(TrimString(IamCertExtKind::OAuth2Service.to_string())),
                supplier: None,
                status: Some(RbumCertConfStatusKind::Enabled),
                rel_rbum_domain_id: Some(funs.iam_basic_domain_iam_id()),
                rel_rbum_item_id: None,
            },
            funs,
            ctx,
        )
        .await
        .map_err(|_| funs.err().unauthorized("oauth2", "generate_code", &format!("client not found: {}", add_req.client_id), "401-oauth2-invalid-client"))?;

        let ext = TardisFuns::json.str_to_obj::<IamCertConfOAuth2ServiceExt>(&conf.ext)?;

        // 3. 验证重定向URI
        if add_req.redirect_uri.to_string() != ext.redirect_uri {
            return Err(funs.err().bad_request("oauth2", "generate_code", "invalid_redirect_uri", "400-oauth2-invalid-redirect-uri"));
        }

        // 4. 构建授权码信息
        let code_info = IamCertOAuth2ServiceCode {
            ctx: ctx.clone(),
            client_id: add_req.client_id.to_string(),
            redirect_uri: add_req.redirect_uri.to_string(),
            scope: add_req.scope.to_string(),
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
        let global_ctx = TardisContext::default();

        // 2. 获取客户端配置并验证client_secret
        let conf = RbumCertConfServ::get_rbum(
            &req.client_id,
            &RbumCertConfFilterReq {
                basic: RbumBasicFilterReq {
                    ignore_scope: true,
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                kind: Some(TrimString(IamCertExtKind::OAuth2Service.to_string())),
                supplier: None,
                status: Some(RbumCertConfStatusKind::Enabled),
                rel_rbum_domain_id: Some(funs.iam_basic_domain_iam_id()),
                rel_rbum_item_id: None,
            },
            funs,
            &global_ctx,
        )
        .await
        .map_err(|_| funs.err().unauthorized("oauth2", "verify_code", "invalid_client", "401-oauth2-invalid-client"))?;

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
        IamIdentCacheServ::add_token(&access_token, &IamCertTokenKind::TokenOauth2, &code_info.ctx.owner, None, access_token_expire_sec, 1, funs).await?;

        // 8. 存储刷新令牌
        let refresh_token_info = IamOAuth2RefreshTokenInfo {
            user_id: code_info.ctx.owner.clone(),
            client_id: req.client_id.clone(),
            scope: code_info.scope.clone(),
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
            scope: Some(code_info.scope),
        })
    }

    /// 刷新令牌方法
    pub async fn refresh_token(req: &IamCertOAuth2ServiceRefreshTokenReq, funs: &TardisFunsInst) -> TardisResult<IamOauth2TokenResp> {
        // 1. 验证grant_type
        if req.grant_type != Oauth2GrantType::RefreshToken {
            return Err(funs.err().bad_request("oauth2", "refresh_token", "unsupported_grant_type", "400-oauth2-unsupported-grant-type"));
        }

        // 2. 获取刷新令牌信息
        let refresh_token_data = funs.cache().get(&format!("iam:oauth2:refresh_token:{}", req.refresh_token)).await?;
        let refresh_token_info: IamOAuth2RefreshTokenInfo = match refresh_token_data {
            Some(data) => TardisFuns::json.str_to_obj(&data)?,
            None => return Err(funs.err().unauthorized("oauth2", "refresh_token", "invalid_refresh_token", "401-oauth2-invalid-refresh-token")),
        };

        // 3. 验证客户端和刷新令牌
        if refresh_token_info.client_id != req.client_id {
            return Err(funs.err().unauthorized("oauth2", "refresh_token", "invalid_client", "401-oauth2-invalid-client"));
        }

        if Utc::now().timestamp() > refresh_token_info.expires_at {
            return Err(funs.err().unauthorized("oauth2", "refresh_token", "refresh_token_expired", "401-oauth2-refresh-token-expired"));
        }

        // 4. 生成新的访问令牌
        let new_access_token = TardisFuns::crypto.key.generate_token()?;
        let iam_config = funs.conf::<IamConfig>();
        let access_token_expire_sec = iam_config.oauth2_access_token_default_expire_sec;

        // 5. 存储新的访问令牌
        IamIdentCacheServ::add_token(
            &new_access_token,
            &IamCertTokenKind::TokenOauth2,
            &refresh_token_info.user_id,
            None,
            access_token_expire_sec as i64,
            1,
            funs,
        )
        .await?;

        Ok(IamOauth2TokenResp {
            access_token: new_access_token,
            token_type: Oauth2TokenType::Bearer,
            expires_in: access_token_expire_sec as i64,
            refresh_token: Some(req.refresh_token.clone()), // 保持相同的刷新令牌
            scope: Some(refresh_token_info.scope),
        })
    }

    /// 保持向后兼容的简化方法
    pub async fn verify_code(add_req: &IamCertOAuth2ServiceCodeVerifyReq, funs: &TardisFunsInst) -> TardisResult<String> {
        let token_resp = Self::verify_code_and_generate_token(add_req, funs).await?;
        Ok(token_resp.access_token)
    }
}
