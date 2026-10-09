use async_trait::async_trait;
use bios_basic::rbum::helper::rbum_scope_helper::get_path_item;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use std::collections::HashSet;
use tardis::basic::dto::TardisContext;
use tardis::basic::error::TardisError;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::chrono::Utc;
use tardis::{TardisFuns, TardisFunsInst};

use crate::basic::dto::iam_account_dto::IamAccountAggAddReq;
use crate::basic::dto::iam_cert_conf_dto::{IamCertConfOAuth2AddOrModifyReq, IamCertConfOAuth2Resp};
use crate::basic::dto::iam_cert_dto::IamCertOAuth2AddOrModifyReq;
use crate::basic::dto::iam_filer_dto::IamTenantFilterReq;
use crate::basic::serv::iam_cert_user_pwd_serv::IamCertUserPwdServ;
use crate::basic::serv::oauth2_spi::iam_cert_oauth2_spi_github::IamCertOAuth2SpiGithub;
use crate::iam_config::IamBasicConfigApi;
use crate::iam_constants::RBUM_SCOPE_LEVEL_TENANT;
use crate::iam_enumeration::{IamCertExtKind, IamCertOAuth2Supplier};
use bios_basic::rbum::dto::rbum_cert_conf_dto::{RbumCertConfAddReq, RbumCertConfModifyReq};
use bios_basic::rbum::dto::rbum_cert_dto::{RbumCertAddReq, RbumCertModifyReq};
use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumCertConfFilterReq, RbumCertFilterReq};
use bios_basic::rbum::rbum_enumeration::RbumCertStatusKind::Pending;
use bios_basic::rbum::rbum_enumeration::{RbumCertConfStatusKind, RbumCertRelKind, RbumCertStatusKind, RbumScopeLevelKind};
use bios_basic::rbum::serv::rbum_cert_serv::{RbumCertConfServ, RbumCertServ};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use serde::{Deserialize, Serialize};
use tardis::web::poem_openapi;

use super::clients::iam_search_client::IamSearchClient;
use super::iam_account_serv::IamAccountServ;
use super::iam_cert_serv::IamCertServ;
use super::iam_oauth2_provider_grant_serv::{IamOAuth2ProviderGrantServ, ProviderGrantCredential};
use super::iam_tenant_serv::IamTenantServ;
use super::oauth2_spi::iam_cert_oauth2_spi_bios_iam::IamCertOAuth2SpiBiosIam;
use super::oauth2_spi::iam_cert_oauth2_spi_wechat_mp::IamCertOAuth2SpiWeChatMp;

pub struct IamCertOAuth2Serv;

impl IamCertOAuth2Serv {
    pub async fn add_cert_conf(
        cert_supplier: IamCertOAuth2Supplier,
        add_req: &IamCertConfOAuth2AddOrModifyReq,
        rel_iam_item_id: &str,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<String> {
        RbumCertConfServ::add_rbum(
            &mut RbumCertConfAddReq {
                kind: TrimString(IamCertExtKind::OAuth2.to_string()),
                supplier: Some(TrimString(cert_supplier.to_string())),
                name: TrimString(format!("{}{}", IamCertExtKind::OAuth2, cert_supplier)),
                note: None,
                ak_note: None,
                ak_rule: None,
                sk_note: None,
                sk_rule: None,
                ext: Some(TardisFuns::json.obj_to_string(&add_req)?),
                sk_need: Some(false),
                sk_dynamic: Some(false),
                sk_encrypted: Some(false),
                repeatable: None,
                is_basic: Some(false),
                rest_by_kinds: None,
                expire_sec: None,
                sk_lock_cycle_sec: None,
                sk_lock_err_times: None,
                sk_lock_duration_sec: None,
                coexist_num: Some(1),
                conn_uri: None,
                status: RbumCertConfStatusKind::Enabled,
                rel_rbum_domain_id: funs.iam_basic_domain_iam_id(),
                rel_rbum_item_id: (!rel_iam_item_id.is_empty()).then_some(rel_iam_item_id.to_string()),
            },
            funs,
            ctx,
        )
        .await
    }

    pub async fn modify_cert_conf(id: &str, modify_req: &IamCertConfOAuth2AddOrModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        RbumCertConfServ::modify_rbum(
            id,
            &mut RbumCertConfModifyReq {
                name: None,
                note: None,
                ak_note: None,
                ak_rule: None,
                sk_note: None,
                sk_rule: None,
                ext: Some(TardisFuns::json.obj_to_string(&modify_req)?),
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

    pub async fn get_cert_conf(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<IamCertConfOAuth2Resp> {
        RbumCertConfServ::get_rbum(
            id,
            &RbumCertConfFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    own_paths: Some("".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await
        .map(|i: bios_basic::rbum::dto::rbum_cert_conf_dto::RbumCertConfDetailResp| TardisFuns::json.str_to_obj(&i.ext))?
    }

    pub async fn add_or_modify_cert(
        add_or_modify_req: &IamCertOAuth2AddOrModifyReq,
        account_id: &str,
        rel_rbum_cert_conf_id: &str,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let cert_id = RbumCertServ::find_id_rbums(
            &RbumCertFilterReq {
                rel_rbum_cert_conf_ids: Some(vec![rel_rbum_cert_conf_id.to_string()]),
                rel_rbum_id: Some(account_id.to_string()),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        if let Some(cert_id) = cert_id.first() {
            RbumCertServ::modify_rbum(
                cert_id,
                &mut RbumCertModifyReq {
                    ak: Some(add_or_modify_req.open_id.clone()),
                    sk: None,
                    sk_invisible: None,

                    ignore_check_sk: false,
                    ext: None,
                    start_time: None,
                    end_time: None,
                    conn_uri: None,
                    status: None,
                },
                funs,
                ctx,
            )
            .await?;
        } else {
            RbumCertServ::add_rbum(
                &mut RbumCertAddReq {
                    ak: add_or_modify_req.open_id.clone(),
                    sk: None,
                    sk_invisible: None,

                    kind: None,
                    supplier: None,
                    vcode: None,
                    ext: None,
                    start_time: None,
                    end_time: None,
                    conn_uri: None,
                    status: RbumCertStatusKind::Enabled,
                    rel_rbum_cert_conf_id: Some(rel_rbum_cert_conf_id.to_string()),
                    rel_rbum_kind: RbumCertRelKind::Item,
                    rel_rbum_id: account_id.to_string(),
                    is_outside: false,
                    ignore_check_sk: false,
                },
                funs,
                ctx,
            )
            .await?;
        };
        Ok(())
    }

    pub async fn get_cert_rel_account_by_open_id(open_id: &str, rel_rbum_cert_conf_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<String>> {
        let result = RbumCertServ::find_rbums(
            &RbumCertFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    own_paths: Some("".to_string()),
                    ..Default::default()
                },
                rel_rbum_cert_conf_ids: Some(vec![rel_rbum_cert_conf_id.to_string()]),
                ak: Some(open_id.to_string()),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?
        .first()
        .map(|r| r.rel_rbum_id.to_string());
        Ok(result)
    }

    /// 判断指定 OAuth2 外部身份（open_id）是否已绑定到本地账号
    ///
    /// 传入 OAuth2 对应的用户 id（open_id），已绑定返回 `true`，未绑定返回 `false`。
    pub async fn is_open_id_bound(cert_supplier: IamCertOAuth2Supplier, open_id: &str, tenant_id: &str, funs: &TardisFunsInst) -> TardisResult<bool> {
        let cert_conf_id = Self::get_oauth2_cert_conf_id(&cert_supplier, tenant_id, funs).await?;
        let mock_ctx = TardisContext {
            own_paths: tenant_id.to_string(),
            ..Default::default()
        };
        Ok(Self::get_cert_rel_account_by_open_id(open_id, &cert_conf_id, funs, &mock_ctx).await?.is_some())
    }

    pub async fn get_or_add_account(cert_supplier: IamCertOAuth2Supplier, code: &str, tenant_id: &str, funs: &TardisFunsInst) -> TardisResult<(String, String)> {
        let cert_conf_id = Self::get_oauth2_cert_conf_id(&cert_supplier, tenant_id, funs).await?;
        let mut mock_ctx = TardisContext {
            own_paths: "".to_string(),
            ..Default::default()
        };
        let cert_conf = Self::get_cert_conf(&cert_conf_id, funs, &mock_ctx).await?;
        let client = Self::get_access_token_func(cert_supplier.clone());
        let oauth_token_info = client.get_access_token(code, &cert_conf.ak, &cert_conf.sk, cert_conf.base_url.as_deref().unwrap_or_default(), funs).await?;
        if let Some(account_id) = Self::get_cert_rel_account_by_open_id(&oauth_token_info.open_id, &cert_conf_id, funs, &mock_ctx).await? {
            Self::cache_provider_token(&cert_supplier, &account_id, Some(&cert_conf_id), &oauth_token_info, funs).await?;
            return Ok((account_id, oauth_token_info.access_token));
        }
        if !tenant_id.is_empty() && !IamTenantServ::get_item(tenant_id, &IamTenantFilterReq::default(), funs, &mock_ctx).await?.account_self_reg {
            return Err(funs.err().not_found(
                "rbum_cert",
                "get_or_add_account",
                &format!("not found oauth2 cert(openid): {} and self-registration disabled", &oauth_token_info.open_id),
                "401-rbum-cert-valid-error",
            ));
        }
        // Register
        mock_ctx.owner = TardisFuns::field.nanoid();
        let account_id = IamAccountServ::add_account_agg(
            &IamAccountAggAddReq {
                id: Some(TrimString(mock_ctx.owner.clone())),
                name: TrimString(client.get_account_name(oauth_token_info.clone(), funs).await?),
                cert_user_name: IamCertUserPwdServ::rename_ak_if_duplicate(&TardisFuns::field.nanoid_len(8).to_lowercase(), funs, &mock_ctx).await?,
                // FIXME 临时密码
                cert_password: Some(TrimString(format!("{}0Pw$", TardisFuns::field.nanoid_len(6)))),
                cert_phone: None,
                cert_mail: None,
                role_ids: None,
                org_node_ids: None,
                scope_level: Some(RbumScopeLevelKind::Root),
                disabled: None,
                icon: None,
                exts: None,
                status: Some(Pending),
                temporary: None,
                lock_status: None,
                logout_type: None,
                labor_type: None,
                id_card_no: None,
                employee_code: None,
                others_id: None,
            },
            false,
            funs,
            &mock_ctx,
        )
        .await?;
        Self::add_or_modify_cert(
            &IamCertOAuth2AddOrModifyReq {
                open_id: TrimString(oauth_token_info.open_id.to_string()),
            },
            &account_id,
            &cert_conf_id,
            funs,
            &mock_ctx,
        )
        .await?;
        Self::cache_provider_token(&cert_supplier, &account_id, Some(&cert_conf_id), &oauth_token_info, funs).await?;
        IamSearchClient::async_add_or_modify_account_search(&account_id, Box::new(false), "", funs, &mock_ctx).await?;
        mock_ctx.execute_task().await?;
        Ok((account_id, oauth_token_info.access_token))
    }

    /// 将外部 OAuth2 身份（open_id）手动绑定到指定的本地账号
    ///
    /// 用于用户已在本系统登录后，主动把当前本地账号与外部身份提供方账号关联（首次登录绑定两边账号）。
    /// 与 `get_or_add_account` 的区别：不会新建账号，只在传入的 `account_id` 上写入/校验绑定凭证。
    /// 返回绑定的 open_id。若该 open_id 已绑定到其他账号则返回 409，已绑定到当前账号则幂等返回。
    pub async fn bind_cert_account(
        cert_supplier: IamCertOAuth2Supplier,
        code: &str,
        tenant_id: &str,
        account_id: &str,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<String> {
        let cert_conf_id = Self::get_oauth2_cert_conf_id(&cert_supplier, tenant_id, funs).await?;
        let mock_ctx = TardisContext {
            own_paths: tenant_id.to_string(),
            ..Default::default()
        };
        let cert_conf = Self::get_cert_conf(&cert_conf_id, funs, &mock_ctx).await?;
        let client = Self::get_access_token_func(cert_supplier.clone());
        let oauth_token_info = client.get_access_token(code, &cert_conf.ak, &cert_conf.sk, cert_conf.base_url.as_deref().unwrap_or_default(), funs).await?;
        let open_id = Self::bind_open_id(&cert_conf_id, &oauth_token_info.open_id, account_id, funs, ctx, &mock_ctx).await?;
        Self::cache_provider_token(&cert_supplier, account_id, Some(&cert_conf_id), &oauth_token_info, funs).await?;
        Ok(open_id)
    }

    /// 绑定校验与写入的公共逻辑：在租户级范围内校验 open_id 唯一性后写入绑定凭证。
    async fn bind_open_id(cert_conf_id: &str, open_id: &str, account_id: &str, funs: &TardisFunsInst, ctx: &TardisContext, mock_ctx: &TardisContext) -> TardisResult<String> {
        // 校验该外部身份是否已绑定到其他账号（租户级范围），避免一个外部身份绑定到多个本地账号
        if let Some(bound_account_id) = Self::get_cert_rel_account_by_open_id(open_id, cert_conf_id, funs, mock_ctx).await? {
            if bound_account_id != account_id {
                return Err(funs.err().conflict(
                    "rbum_cert",
                    "bind_cert_account",
                    &format!("oauth2 open_id {} has already been bound to another account", open_id),
                    "409-iam-cert-oauth-already-bound",
                ));
            }
            // 已绑定到当前账号，幂等返回
            return Ok(open_id.to_string());
        }
        Self::add_or_modify_cert(
            &IamCertOAuth2AddOrModifyReq {
                open_id: TrimString(open_id.to_string()),
            },
            account_id,
            cert_conf_id,
            funs,
            ctx,
        )
        .await?;
        Ok(open_id.to_string())
    }

    /// 查找 OAuth2 cert_conf_id：优先租户级，找不到时回退到平台级。
    pub(crate) async fn get_oauth2_cert_conf_id(cert_supplier: &IamCertOAuth2Supplier, tenant_id: &str, funs: &TardisFunsInst) -> TardisResult<String> {
        let kind = IamCertExtKind::OAuth2.to_string();
        let supplier_str = cert_supplier.to_string();
        let tenant_opt = get_path_item(RBUM_SCOPE_LEVEL_TENANT.to_int(), tenant_id);
        // 先查租户级
        if let Some(ref tid) = tenant_opt {
            if let Some(conf) = IamCertServ::get_cert_conf_id_and_ext_opt_by_kind_supplier(&kind, &supplier_str, Some(tid.clone()), funs).await? {
                return Ok(conf.id);
            }
        }
        // 回退到平台级
        IamCertServ::get_cert_conf_id_by_kind_supplier(&kind, &supplier_str, None, funs).await
    }

    fn get_access_token_func(supplier: IamCertOAuth2Supplier) -> Box<dyn IamCertOAuth2Spi> {
        match supplier {
            // IamCertOAuth2Supplier::Weibo => {}
            IamCertOAuth2Supplier::Github => Box::new(IamCertOAuth2SpiGithub),
            IamCertOAuth2Supplier::WechatMp => Box::new(IamCertOAuth2SpiWeChatMp),
            IamCertOAuth2Supplier::BiosIam => Box::new(IamCertOAuth2SpiBiosIam),
        }
    }

    /// 将第三方 Provider 的 token（access_token / refresh_token / 过期时间）缓存到 Redis
    ///
    /// 以 `account_id + supplier` 为维度存储；缓存 TTL 取「配置值」与「本地登录 token 默认时长」的较大者，
    /// 确保 Provider token 缓存不会早于本地登录 token 过期，供后续 token 置换（refresh）与查询 Provider 用户信息使用。
    async fn cache_provider_token(
        supplier: &IamCertOAuth2Supplier,
        account_id: &str,
        cert_conf_id: Option<&str>,
        token_info: &IamCertOAuth2TokenInfo,
        funs: &TardisFunsInst,
    ) -> TardisResult<()> {
        if *supplier != IamCertOAuth2Supplier::BiosIam {
            return Self::cache_provider_token_unlocked(supplier, account_id, cert_conf_id, token_info, true, funs).await;
        }
        let lease = Self::acquire_provider_lease(supplier, account_id, funs).await?;
        let result = Self::cache_provider_token_unlocked(supplier, account_id, cert_conf_id, token_info, true, funs).await;
        Self::release_provider_lease(supplier, account_id, &lease, funs).await?;
        result
    }

    async fn cache_provider_token_unlocked(
        supplier: &IamCertOAuth2Supplier,
        account_id: &str,
        cert_conf_id: Option<&str>,
        token_info: &IamCertOAuth2TokenInfo,
        write_cache: bool,
        funs: &TardisFunsInst,
    ) -> TardisResult<()> {
        let mut token_info = token_info.clone();
        if *supplier == IamCertOAuth2Supplier::BiosIam {
            token_info.provider_cert_conf_id = cert_conf_id.map(str::to_owned);
            IamOAuth2ProviderGrantServ::persist_refreshed_token(account_id, &token_info, funs).await?;
            if !write_cache {
                return Ok(());
            }
            return IamOAuth2ProviderGrantServ::cache_token(account_id, &token_info, funs).await;
        }
        let conf = funs.conf::<crate::iam_config::IamConfig>();
        // 与登录 token 默认时长对齐：缓存不得早于登录 token 过期
        let expire_sec = conf.oauth2_provider_token_cache_expire_sec.max(crate::iam_constants::RBUM_CERT_CONF_TOKEN_EXPIRE_SEC as u32);
        funs.cache()
            .set_ex(
                &format!("{}{}:{}", conf.cache_key_oauth2_provider_token_, supplier, account_id),
                &TardisFuns::json.obj_to_string(&token_info)?,
                expire_sec as u64,
            )
            .await?;
        Ok(())
    }

    /// 读取已缓存的第三方 Provider token，不存在或已过期时返回 404
    pub(crate) async fn get_cached_provider_token(supplier: &IamCertOAuth2Supplier, account_id: &str, funs: &TardisFunsInst) -> TardisResult<IamCertOAuth2TokenInfo> {
        if *supplier == IamCertOAuth2Supplier::BiosIam {
            return IamOAuth2ProviderGrantServ::get_cached_token(account_id, funs).await?.ok_or_else(|| {
                funs.err().not_found(
                    "rbum_cert",
                    "get_cached_provider_token",
                    "oauth2 provider token not found or expired",
                    "404-iam-cert-oauth-provider-token-not-found",
                )
            });
        }
        let conf = funs.conf::<crate::iam_config::IamConfig>();
        let cached = funs.cache().get(&format!("{}{}:{}", conf.cache_key_oauth2_provider_token_, supplier, account_id)).await?;
        if let Some(cached) = cached {
            Ok(TardisFuns::json.str_to_obj(&cached)?)
        } else {
            Err(funs.err().not_found(
                "rbum_cert",
                "get_cached_provider_token",
                "oauth2 provider token not found or expired",
                "404-iam-cert-oauth-provider-token-not-found",
            ))
        }
    }

    fn require_refresh_token<'a>(token: &'a IamCertOAuth2TokenInfo, funs: &TardisFunsInst) -> TardisResult<&'a str> {
        token.refresh_token.as_deref().filter(|token| !token.trim().is_empty()).ok_or_else(|| {
            funs.err().conflict(
                "rbum_cert",
                "refresh_provider_token",
                "no refresh_token stored for this account; authorize the provider again",
                "409-iam-cert-oauth-refresh-token-missing",
            )
        })
    }

    fn ensure_bios_provider_conf(token: &IamCertOAuth2TokenInfo, cert_conf_id: &str, funs: &TardisFunsInst) -> TardisResult<()> {
        if token.provider_cert_conf_id.as_deref() == Some(cert_conf_id) {
            return Ok(());
        }
        Err(funs.err().conflict(
            "rbum_cert",
            "get_provider_token",
            "stored BiosIam grant belongs to a different OAuth provider configuration; authorize the provider again",
            "409-iam-cert-oauth-provider-conf-mismatch",
        ))
    }

    fn provider_authorization_required_error() -> TardisError {
        TardisError::custom(
            "409-iam-oauth-provider-authorization-required",
            "external OAuth authorization is unavailable; authorize the provider again",
            "409-iam-oauth-provider-authorization-required",
        )
    }

    fn provider_subject_mismatch_error() -> TardisError {
        TardisError::custom(
            "409-iam-oauth-provider-subject-mismatch",
            "provider token identity changed; authorize the provider again",
            "409-iam-oauth-provider-subject-mismatch",
        )
    }

    fn normalize_provider_grant_error(error: TardisError) -> TardisError {
        if error.code == "409-iam-oauth-provider-subject-mismatch" {
            return error;
        }
        let status = error.code.split_once('-').map_or(error.code.as_str(), |(status, _)| status);
        if matches!(status.parse::<u16>(), Ok(400 | 401 | 403 | 404 | 409)) {
            Self::provider_authorization_required_error()
        } else {
            error
        }
    }

    fn bios_provider_token_expired(token: &IamCertOAuth2TokenInfo) -> bool {
        Self::bios_provider_token_expired_at(token, Utc::now().timestamp_millis())
    }

    fn bios_provider_token_expired_at(token: &IamCertOAuth2TokenInfo, now_ms: i64) -> bool {
        token.expires_at_ms.map(|expires_at_ms| expires_at_ms <= now_ms).unwrap_or(true)
    }

    fn same_provider_token(left: &IamCertOAuth2TokenInfo, right: &IamCertOAuth2TokenInfo) -> bool {
        left.open_id == right.open_id
            && left.access_token == right.access_token
            && left.refresh_token == right.refresh_token
            && left.expires_at_ms == right.expires_at_ms
            && left.scope == right.scope
            && left.provider_cert_conf_id == right.provider_cert_conf_id
    }

    async fn provider_credential_is_bound_to_account(account_id: &str, own_paths: &str, credential: &ProviderGrantCredential, funs: &TardisFunsInst) -> TardisResult<bool> {
        let user_ctx = TardisContext {
            owner: account_id.to_string(),
            own_paths: own_paths.to_string(),
            ..Default::default()
        };
        let conf_result = RbumCertConfServ::get_rbum(
            &credential.cert_conf_id,
            &RbumCertConfFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some(String::new()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                status: Some(RbumCertConfStatusKind::Enabled),
                ..Default::default()
            },
            funs,
            &user_ctx,
        )
        .await;
        if let Err(error) = conf_result {
            if error.code.starts_with("404") {
                return Ok(false);
            }
            return Err(error);
        }
        let certs = RbumCertServ::find_rbums(
            &RbumCertFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some(String::new()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ak: Some(credential.external_subject.clone()),
                rel_rbum_cert_conf_ids: Some(vec![credential.cert_conf_id.clone()]),
                status: Some(RbumCertStatusKind::Enabled),
                ..Default::default()
            },
            None,
            None,
            funs,
            &user_ctx,
        )
        .await?;
        let bound_accounts = certs.iter().map(|cert| cert.rel_rbum_id.as_str()).collect::<std::collections::HashSet<_>>();
        Ok(bound_accounts.len() == 1 && bound_accounts.contains(account_id))
    }

    async fn recover_current_bios_credential_unlocked(
        account_id: &str,
        own_paths: &str,
        cert_conf_id: &str,
        funs: &TardisFunsInst,
    ) -> TardisResult<Option<ProviderGrantCredential>> {
        let candidates = IamOAuth2ProviderGrantServ::load_by_account_config(account_id, cert_conf_id, funs).await?;
        let mut current = None;
        for candidate in candidates {
            if !Self::provider_credential_is_bound_to_account(account_id, own_paths, &candidate, funs).await? {
                continue;
            }
            if current.is_some() {
                return Err(funs.err().conflict(
                    "rbum_cert",
                    "get_provider_token",
                    "multiple current provider identities are bound to this account; authorize the provider again",
                    "409-iam-oauth-provider-identity-ambiguous",
                ));
            }
            current = Some(candidate);
        }
        Ok(current)
    }

    async fn current_bios_provider_token_unlocked(account_id: &str, own_paths: &str, cert_conf_id: &str, funs: &TardisFunsInst) -> TardisResult<IamCertOAuth2TokenInfo> {
        match IamOAuth2ProviderGrantServ::get_cached_token(account_id, funs).await {
            Ok(Some(cached)) => {
                Self::ensure_bios_provider_conf(&cached, cert_conf_id, funs)?;
                let Some(durable) = IamOAuth2ProviderGrantServ::load_by_identity(account_id, cert_conf_id, &cached.open_id, funs).await? else {
                    return Ok(cached);
                };
                if !Self::same_provider_token(&cached, &durable.token) {
                    let _ = IamOAuth2ProviderGrantServ::cache_token(account_id, &durable.token, funs).await;
                }
                Ok(durable.token)
            }
            cached_result => {
                if let Some(durable) = Self::recover_current_bios_credential_unlocked(account_id, own_paths, cert_conf_id, funs).await? {
                    let _ = IamOAuth2ProviderGrantServ::cache_token(account_id, &durable.token, funs).await;
                    return Ok(durable.token);
                }
                match cached_result {
                    Ok(None) => Err(funs.err().not_found(
                        "rbum_cert",
                        "get_cached_provider_token",
                        "oauth2 provider token not found or expired",
                        "404-iam-cert-oauth-provider-token-not-found",
                    )),
                    Err(error) => Err(error),
                    Ok(Some(_)) => unreachable!(),
                }
            }
        }
    }

    fn merge_bios_refresh_scope(cached: &IamCertOAuth2TokenInfo, refreshed: &mut IamCertOAuth2TokenInfo, funs: &TardisFunsInst) -> TardisResult<()> {
        match (cached.scope.as_deref(), refreshed.scope.as_deref()) {
            (Some(previous), Some(current)) => {
                let previous = previous.split_ascii_whitespace().collect::<HashSet<_>>();
                if current.split_ascii_whitespace().any(|scope| !previous.contains("all") && !previous.contains(scope)) {
                    return Err(funs.err().conflict(
                        "rbum_cert",
                        "refresh_provider_token",
                        "provider refresh expanded the granted scope; authorize the provider again",
                        "409-iam-cert-oauth-refresh-scope-expanded",
                    ));
                }
            }
            (Some(previous), None) => refreshed.scope = Some(previous.to_string()),
            (None, Some(_)) => {
                return Err(funs.err().conflict(
                    "rbum_cert",
                    "refresh_provider_token",
                    "stored provider grant has no scope to validate the refresh response; authorize the provider again",
                    "409-iam-cert-oauth-refresh-scope-untrusted",
                ));
            }
            (None, None) => {}
        }
        Ok(())
    }

    /// 获取已缓存的第三方 Provider token，供前端直接调用 Provider API 时使用。
    pub async fn get_provider_token(cert_supplier: IamCertOAuth2Supplier, account_id: &str, tenant_id: &str, funs: &TardisFunsInst) -> TardisResult<IamCertOAuth2TokenInfo> {
        if cert_supplier != IamCertOAuth2Supplier::BiosIam {
            return Self::get_cached_provider_token(&cert_supplier, account_id, funs).await;
        }
        let cert_conf_id = Self::get_oauth2_cert_conf_id(&cert_supplier, tenant_id, funs).await?;
        let lease = IamOAuth2ProviderGrantServ::acquire_lease(account_id, funs).await?;
        let result = tardis::tokio::time::timeout(std::time::Duration::from_secs(40), async {
            let cached = Self::current_bios_provider_token_unlocked(account_id, tenant_id, &cert_conf_id, funs).await?;
            Self::ensure_bios_provider_conf(&cached, &cert_conf_id, funs)?;
            if Self::bios_provider_token_expired(&cached) {
                Self::refresh_provider_token_unlocked(cert_supplier.clone(), account_id, tenant_id, &cached, true, funs).await
            } else {
                Ok(cached)
            }
        })
        .await
        .unwrap_or_else(|_| {
            Err(tardis::basic::error::TardisError::gateway_timeout(
                "OAuth credential refresh timed out",
                "504-iam-oauth-refresh-timeout",
            ))
        });
        IamOAuth2ProviderGrantServ::release_lease(account_id, &lease, funs).await?;
        result
    }

    /// token 置换：使用已缓存的 refresh_token 向 Provider 换取新的 access_token，并更新缓存
    ///
    /// 用于网关/调用方在 Provider access_token 过期后刷新令牌。返回最新的 token 信息。
    pub async fn refresh_provider_token(cert_supplier: IamCertOAuth2Supplier, account_id: &str, tenant_id: &str, funs: &TardisFunsInst) -> TardisResult<IamCertOAuth2TokenInfo> {
        if cert_supplier != IamCertOAuth2Supplier::BiosIam {
            let cached = Self::get_cached_provider_token(&cert_supplier, account_id, funs).await?;
            return Self::refresh_provider_token_unlocked(cert_supplier, account_id, tenant_id, &cached, true, funs).await;
        }
        let cert_conf_id = Self::get_oauth2_cert_conf_id(&cert_supplier, tenant_id, funs).await?;
        let lease = IamOAuth2ProviderGrantServ::acquire_lease(account_id, funs).await?;
        let result = tardis::tokio::time::timeout(std::time::Duration::from_secs(40), async {
            let cached = Self::current_bios_provider_token_unlocked(account_id, tenant_id, &cert_conf_id, funs).await?;
            Self::ensure_bios_provider_conf(&cached, &cert_conf_id, funs)?;
            Self::refresh_provider_token_unlocked(cert_supplier.clone(), account_id, tenant_id, &cached, true, funs).await
        })
        .await
        .unwrap_or_else(|_| {
            Err(tardis::basic::error::TardisError::gateway_timeout(
                "OAuth credential refresh timed out",
                "504-iam-oauth-refresh-timeout",
            ))
        });
        IamOAuth2ProviderGrantServ::release_lease(account_id, &lease, funs).await?;
        result
    }

    pub(crate) async fn acquire_provider_lease(cert_supplier: &IamCertOAuth2Supplier, account_id: &str, funs: &TardisFunsInst) -> TardisResult<String> {
        if *cert_supplier == IamCertOAuth2Supplier::BiosIam {
            return IamOAuth2ProviderGrantServ::acquire_lease(account_id, funs).await;
        }
        let key = format!("iam:oauth2:provider-refresh:{cert_supplier}:{account_id}");
        let lease = TardisFuns::field.nanoid();
        let acquired: bool = funs.cache().script("return redis.call('SET', KEYS[1], ARGV[1], 'NX', 'EX', 120) ~= false").key(&key).arg(&lease).invoke().await?;
        if !acquired {
            return Err(tardis::basic::error::TardisError::custom(
                "503",
                "OAuth credential refresh is in progress; retry later",
                "503-iam-oauth-refresh-in-progress",
            ));
        }
        Ok(lease)
    }

    pub(crate) async fn release_provider_lease(cert_supplier: &IamCertOAuth2Supplier, account_id: &str, lease: &str, funs: &TardisFunsInst) -> TardisResult<()> {
        if *cert_supplier == IamCertOAuth2Supplier::BiosIam {
            return IamOAuth2ProviderGrantServ::release_lease(account_id, lease, funs).await;
        }
        let key = format!("iam:oauth2:provider-refresh:{cert_supplier}:{account_id}");
        let _: i64 = funs.cache().script("if redis.call('GET', KEYS[1]) == ARGV[1] then return redis.call('DEL', KEYS[1]) end return 0").key(&key).arg(&lease).invoke().await?;
        Ok(())
    }

    async fn refresh_provider_token_unlocked(
        cert_supplier: IamCertOAuth2Supplier,
        account_id: &str,
        tenant_id: &str,
        cached: &IamCertOAuth2TokenInfo,
        write_cache: bool,
        funs: &TardisFunsInst,
    ) -> TardisResult<IamCertOAuth2TokenInfo> {
        let cert_conf_id = Self::get_oauth2_cert_conf_id(&cert_supplier, tenant_id, funs).await?;
        let mock_ctx = TardisContext {
            own_paths: tenant_id.to_string(),
            ..Default::default()
        };
        let cert_conf = Self::get_cert_conf(&cert_conf_id, funs, &mock_ctx).await?;
        if cert_supplier == IamCertOAuth2Supplier::BiosIam {
            Self::ensure_bios_provider_conf(&cached, &cert_conf_id, funs)?;
        }
        let refresh_token = Self::require_refresh_token(&cached, funs)?;
        let client = Self::get_access_token_func(cert_supplier.clone());
        let mut new_token = client
            .refresh_access_token(
                &refresh_token,
                &cert_conf.ak,
                &cert_conf.sk,
                cert_conf.base_url.as_deref().unwrap_or_default(),
                cached.scope.as_deref(),
                funs,
            )
            .await?;
        // Provider 刷新响应可能不回传 open_id / refresh_token，回填旧值以保持缓存完整
        if new_token.open_id.is_empty() {
            new_token.open_id = cached.open_id.clone();
        }
        if new_token.refresh_token.is_none() {
            new_token.refresh_token = Some(refresh_token.to_string());
        }
        if cert_supplier == IamCertOAuth2Supplier::BiosIam {
            Self::merge_bios_refresh_scope(&cached, &mut new_token, funs)?;
            new_token.provider_cert_conf_id = Some(cert_conf_id.clone());
        }
        let cache_conf_id = (cert_supplier == IamCertOAuth2Supplier::BiosIam).then_some(cert_conf_id.as_str());
        Self::cache_provider_token_unlocked(&cert_supplier, account_id, cache_conf_id, &new_token, write_cache, funs).await?;
        Ok(new_token)
    }

    pub async fn ensure_provider_grant(account_id: &str, own_paths: &str, funs: &TardisFunsInst) -> TardisResult<String> {
        let supplier = IamCertOAuth2Supplier::BiosIam;
        let cert_conf_id = Self::get_oauth2_cert_conf_id(&supplier, own_paths, funs).await.map_err(Self::normalize_provider_grant_error)?;
        let lease = IamOAuth2ProviderGrantServ::acquire_lease(account_id, funs).await?;
        let result = tardis::tokio::time::timeout(std::time::Duration::from_secs(40), async {
            let mut token = Self::current_bios_provider_token_unlocked(account_id, own_paths, &cert_conf_id, funs).await?;
            if Self::bios_provider_token_expired(&token) {
                token = Self::refresh_provider_token_unlocked(supplier.clone(), account_id, own_paths, &token, true, funs).await?;
            }
            Self::ensure_bios_provider_conf(&token, &cert_conf_id, funs)?;
            if token.open_id.is_empty() || token.refresh_token.as_deref().unwrap_or_default().trim().is_empty() {
                return Err(funs.err().conflict(
                    "rbum_cert",
                    "ensure_provider_grant",
                    "provider credential cannot be used for background authorization; authorize the provider again",
                    "409-iam-oauth-provider-authorization-required",
                ));
            }
            let user_ctx = TardisContext {
                owner: account_id.to_string(),
                own_paths: own_paths.to_string(),
                ..Default::default()
            };
            let cert_conf = Self::get_cert_conf(&cert_conf_id, funs, &user_ctx).await?;
            let subject = IamCertOAuth2SpiBiosIam.validate_access_token(&token.access_token, cert_conf.base_url.as_deref().unwrap_or_default(), funs).await?;
            if subject != token.open_id {
                return Err(Self::provider_subject_mismatch_error());
            }
            IamOAuth2ProviderGrantServ::ensure_provider_grant(account_id, &token, funs).await
        })
        .await
        .unwrap_or_else(|_| {
            Err(tardis::basic::error::TardisError::gateway_timeout(
                "OAuth credential validation timed out",
                "504-iam-oauth-provider-validation-timeout",
            ))
        });
        IamOAuth2ProviderGrantServ::release_lease(account_id, &lease, funs).await?;
        result.map_err(Self::normalize_provider_grant_error)
    }

    pub async fn get_validated_grant_token(provider_grant_id: &str, account_id: &str, own_paths: &str, funs: &TardisFunsInst) -> TardisResult<IamCertOAuth2TokenInfo> {
        let supplier = IamCertOAuth2Supplier::BiosIam;
        let cert_conf_id = Self::get_oauth2_cert_conf_id(&supplier, own_paths, funs).await.map_err(Self::normalize_provider_grant_error)?;
        let lease = IamOAuth2ProviderGrantServ::acquire_lease_with_wait(account_id, funs).await?;
        let result = tardis::tokio::time::timeout(std::time::Duration::from_secs(40), async {
            let provider = IamOAuth2ProviderGrantServ::load_by_id(provider_grant_id, funs).await?;
            if provider.account_id != account_id || provider.cert_conf_id != cert_conf_id {
                return Err(Self::provider_authorization_required_error());
            }
            if !Self::provider_credential_is_bound_to_account(account_id, own_paths, &provider, funs).await? {
                return Err(Self::provider_authorization_required_error());
            }

            let cache_matches_identity = match IamOAuth2ProviderGrantServ::get_cached_token(account_id, funs).await {
                Ok(Some(cached)) => cached.open_id == provider.external_subject && cached.provider_cert_conf_id.as_deref() == Some(cert_conf_id.as_str()),
                Ok(None) | Err(_) => {
                    Self::recover_current_bios_credential_unlocked(account_id, own_paths, &cert_conf_id, funs).await?.is_some_and(|current| current.id == provider.id)
                }
            };
            let mut token = provider.token;
            if Self::bios_provider_token_expired(&token) {
                token = Self::refresh_provider_token_unlocked(supplier.clone(), account_id, own_paths, &token, cache_matches_identity, funs).await?;
            } else if cache_matches_identity {
                if let Some(cached) = IamOAuth2ProviderGrantServ::get_cached_token(account_id, funs).await? {
                    if !Self::same_provider_token(&cached, &token) {
                        let _ = IamOAuth2ProviderGrantServ::cache_token(account_id, &token, funs).await;
                    }
                } else {
                    let _ = IamOAuth2ProviderGrantServ::cache_token(account_id, &token, funs).await;
                }
            }
            Self::ensure_bios_provider_conf(&token, &cert_conf_id, funs)?;
            let user_ctx = TardisContext {
                owner: account_id.to_string(),
                own_paths: own_paths.to_string(),
                ..Default::default()
            };
            let cert_conf = Self::get_cert_conf(&cert_conf_id, funs, &user_ctx).await?;
            let subject = IamCertOAuth2SpiBiosIam.validate_access_token(&token.access_token, cert_conf.base_url.as_deref().unwrap_or_default(), funs).await?;
            if subject != provider.external_subject || token.open_id != provider.external_subject {
                return Err(Self::provider_subject_mismatch_error());
            }
            Ok(token)
        })
        .await
        .unwrap_or_else(|_| {
            Err(tardis::basic::error::TardisError::gateway_timeout(
                "OAuth credential validation timed out",
                "504-iam-oauth-provider-validation-timeout",
            ))
        });
        IamOAuth2ProviderGrantServ::release_lease(account_id, &lease, funs).await?;
        result.map_err(Self::normalize_provider_grant_error)
    }

    /// 通过已缓存的 access_token 代表用户向 Provider 查询用户信息
    ///
    /// 返回 Provider 原始用户信息 JSON，供网关/调用方使用。
    pub async fn get_provider_user_info(cert_supplier: IamCertOAuth2Supplier, account_id: &str, tenant_id: &str, funs: &TardisFunsInst) -> TardisResult<tardis::serde_json::Value> {
        let cert_conf_id = Self::get_oauth2_cert_conf_id(&cert_supplier, tenant_id, funs).await?;
        let mock_ctx = TardisContext {
            own_paths: tenant_id.to_string(),
            ..Default::default()
        };
        let cert_conf = Self::get_cert_conf(&cert_conf_id, funs, &mock_ctx).await?;
        let cached = if cert_supplier == IamCertOAuth2Supplier::BiosIam {
            Self::get_provider_token(cert_supplier.clone(), account_id, tenant_id, funs).await?
        } else {
            Self::get_cached_provider_token(&cert_supplier, account_id, funs).await?
        };
        let client = Self::get_access_token_func(cert_supplier);
        client.get_user_info(&cached.access_token, &cached.open_id, cert_conf.base_url.as_deref().unwrap_or_default(), funs).await
    }

    pub async fn add_or_enable_cert_conf(
        supplier: IamCertOAuth2Supplier,
        add_req: &IamCertConfOAuth2AddOrModifyReq,
        rel_iam_item_id: &str,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<String> {
        let cert_result = RbumCertConfServ::do_find_one_rbum(
            &RbumCertConfFilterReq {
                kind: Some(TrimString(IamCertExtKind::OAuth2.to_string())),
                supplier: Some(supplier.clone().to_string()),
                rel_rbum_item_id: Some(rel_iam_item_id.to_string()),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        let result = if let Some(cert_result) = cert_result {
            IamCertServ::enabled_cert_conf(&cert_result.id, funs, ctx).await?;
            cert_result.id
        } else {
            Self::add_cert_conf(supplier, add_req, rel_iam_item_id, funs, ctx).await?
        };
        Ok(result)
    }
}

#[async_trait]
pub trait IamCertOAuth2Spi: Send + Sync {
    async fn get_access_token(&self, code: &str, ak: &str, sk: &str, base_url: &str, funs: &TardisFunsInst) -> TardisResult<IamCertOAuth2TokenInfo>;
    async fn get_account_name(&self, oauth2_info: IamCertOAuth2TokenInfo, funs: &TardisFunsInst) -> TardisResult<String>;

    /// 使用 refresh_token 向 Provider 置换新的 access_token（token 置换）
    ///
    /// 默认实现返回 409，表示当前 Provider 不支持刷新。
    async fn refresh_access_token(
        &self,
        _refresh_token: &str,
        _ak: &str,
        _sk: &str,
        _base_url: &str,
        _scope: Option<&str>,
        funs: &TardisFunsInst,
    ) -> TardisResult<IamCertOAuth2TokenInfo> {
        Err(funs.err().conflict(
            "rbum_cert",
            "refresh_access_token",
            "current oauth2 supplier does not support refresh_token",
            "409-iam-cert-oauth-refresh-unsupported",
        ))
    }

    /// 使用 access_token 向 Provider 查询用户信息
    ///
    /// 默认实现返回 409，表示当前 Provider 不支持查询用户信息。
    async fn get_user_info(&self, _access_token: &str, _open_id: &str, _base_url: &str, funs: &TardisFunsInst) -> TardisResult<tardis::serde_json::Value> {
        Err(funs.err().conflict(
            "rbum_cert",
            "get_user_info",
            "current oauth2 supplier does not support get_user_info",
            "409-iam-cert-oauth-userinfo-unsupported",
        ))
    }

    async fn validate_access_token(&self, _access_token: &str, _base_url: &str, funs: &TardisFunsInst) -> TardisResult<String> {
        Err(funs.err().conflict(
            "rbum_cert",
            "validate_access_token",
            "current oauth2 supplier does not support token identity validation",
            "409-iam-cert-oauth-token-validation-unsupported",
        ))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, poem_openapi::Object)]
pub struct IamCertOAuth2TokenInfo {
    pub open_id: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_expires_ms: Option<u32>,
    pub expires_at_ms: Option<i64>,
    pub scope: Option<String>,
    /// The OAuth cert configuration that issued a BiosIam grant.
    pub provider_cert_conf_id: Option<String>,
    pub union_id: Option<String>,
}

#[cfg(test)]
mod provider_token_tests {
    use super::*;

    fn test_funs() -> TardisFunsInst {
        TardisFuns::inst(crate::iam_constants::COMPONENT_CODE, None)
    }

    fn token(expires_at_ms: Option<i64>, refresh_token: Option<&str>, scope: Option<&str>, provider_cert_conf_id: Option<&str>) -> IamCertOAuth2TokenInfo {
        IamCertOAuth2TokenInfo {
            open_id: "provider-user".to_string(),
            access_token: "access-token".to_string(),
            refresh_token: refresh_token.map(str::to_owned),
            token_expires_ms: Some(60_000),
            expires_at_ms,
            scope: scope.map(str::to_owned),
            provider_cert_conf_id: provider_cert_conf_id.map(str::to_owned),
            union_id: None,
        }
    }

    #[test]
    fn valid_and_expired_bios_tokens_are_distinguished_by_absolute_expiry() {
        let valid = token(Some(2_000), Some("refresh"), Some("iam.userinfo.read"), Some("conf-a"));
        let expired = token(Some(1_000), Some("refresh"), Some("iam.userinfo.read"), Some("conf-a"));

        assert!(!IamCertOAuth2Serv::bios_provider_token_expired_at(&valid, 1_000));
        assert!(IamCertOAuth2Serv::bios_provider_token_expired_at(&expired, 1_000));
        assert!(IamCertOAuth2Serv::bios_provider_token_expired_at(
            &token(None, Some("refresh"), None, Some("conf-a")),
            1_000
        ));
    }

    #[test]
    fn refresh_requires_a_stored_grant() {
        let funs = test_funs();
        let cached = token(Some(2_000), None, None, Some("conf-a"));
        let result = IamCertOAuth2Serv::require_refresh_token(&cached, &funs);

        assert!(result.is_err());
    }

    #[test]
    fn refresh_scope_must_stay_within_the_existing_grant() {
        let funs = test_funs();
        let cached = token(Some(1_000), Some("old-refresh"), Some("iam.userinfo.read iam.app.read"), Some("conf-a"));
        let mut refreshed = token(Some(3_000), Some("new-refresh"), Some("iam.userinfo.read iam.admin.write"), None);

        assert!(IamCertOAuth2Serv::merge_bios_refresh_scope(&cached, &mut refreshed, &funs).is_err());
    }

    #[test]
    fn refresh_can_narrow_an_all_scope_grant() {
        let funs = test_funs();
        let cached = token(Some(1_000), Some("old-refresh"), Some("all"), Some("conf-a"));
        let mut refreshed = token(Some(3_000), Some("new-refresh"), Some("iam.app.read"), None);

        IamCertOAuth2Serv::merge_bios_refresh_scope(&cached, &mut refreshed, &funs).unwrap();
    }

    #[test]
    fn refresh_cannot_expand_a_specific_scope_grant_to_all() {
        let funs = test_funs();
        let cached = token(Some(1_000), Some("old-refresh"), Some("iam.app.read"), Some("conf-a"));
        let mut refreshed = token(Some(3_000), Some("new-refresh"), Some("all"), None);

        assert!(IamCertOAuth2Serv::merge_bios_refresh_scope(&cached, &mut refreshed, &funs).is_err());
    }

    #[test]
    fn tenant_config_lookup_uses_the_first_context_path_item() {
        assert_eq!(get_path_item(RBUM_SCOPE_LEVEL_TENANT.to_int(), "tenant-a/app-a"), Some("tenant-a".to_string()));
        assert_eq!(get_path_item(RBUM_SCOPE_LEVEL_TENANT.to_int(), "tenant-a/app-a/"), Some("tenant-a".to_string()));
        assert_eq!(get_path_item(RBUM_SCOPE_LEVEL_TENANT.to_int(), "tenant-a"), Some("tenant-a".to_string()));
        assert_eq!(get_path_item(RBUM_SCOPE_LEVEL_TENANT.to_int(), ""), None);
    }

    #[test]
    fn refresh_keeps_scope_when_provider_omits_it_and_accepts_rotation() {
        let funs = test_funs();
        let cached = token(Some(1_000), Some("old-refresh"), Some("iam.userinfo.read"), Some("conf-a"));
        let mut refreshed = token(Some(3_000), Some("new-refresh"), None, None);

        IamCertOAuth2Serv::merge_bios_refresh_scope(&cached, &mut refreshed, &funs).unwrap();
        assert_eq!(refreshed.scope.as_deref(), Some("iam.userinfo.read"));
        assert_eq!(refreshed.refresh_token.as_deref(), Some("new-refresh"));
    }

    #[test]
    fn cached_bios_grant_must_match_the_tenant_resolved_config() {
        let funs = test_funs();
        let cached = token(Some(2_000), Some("refresh"), Some("iam.userinfo.read"), Some("tenant-conf"));

        assert!(IamCertOAuth2Serv::ensure_bios_provider_conf(&cached, "platform-conf", &funs).is_err());
        assert!(IamCertOAuth2Serv::ensure_bios_provider_conf(&cached, "tenant-conf", &funs).is_ok());
    }

    #[test]
    fn provider_facade_normalizes_authorization_errors_and_preserves_outages() {
        let authorization_error = IamCertOAuth2Serv::normalize_provider_grant_error(TardisError::conflict("provider rejected refresh", "legacy-locale"));
        assert_eq!(authorization_error.code, "409-iam-oauth-provider-authorization-required");

        let subject_error = IamCertOAuth2Serv::normalize_provider_grant_error(IamCertOAuth2Serv::provider_subject_mismatch_error());
        assert_eq!(subject_error.code, "409-iam-oauth-provider-subject-mismatch");

        let outage = IamCertOAuth2Serv::normalize_provider_grant_error(TardisError::bad_gateway("provider unavailable", "502-provider-unavailable"));
        assert_eq!(outage.code, "502");

        let timeout = IamCertOAuth2Serv::normalize_provider_grant_error(TardisError::custom("408", "provider timed out", "408-provider-timeout"));
        assert_eq!(timeout.code, "408");

        let rate_limit = IamCertOAuth2Serv::normalize_provider_grant_error(TardisError::custom("429", "provider rate limited", "429-provider-rate-limited"));
        assert_eq!(rate_limit.code, "429");

        let malformed = IamCertOAuth2Serv::normalize_provider_grant_error(TardisError::custom("406", "provider response was malformed", "406-provider-malformed"));
        assert_eq!(malformed.code, "406");
    }
}
