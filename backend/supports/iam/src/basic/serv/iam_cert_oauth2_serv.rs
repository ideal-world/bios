use async_trait::async_trait;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::{TardisFuns, TardisFunsInst};

use crate::basic::dto::iam_account_dto::IamAccountAggAddReq;
use crate::basic::dto::iam_cert_conf_dto::{IamCertConfOAuth2AddOrModifyReq, IamCertConfOAuth2Resp};
use crate::basic::dto::iam_cert_dto::IamCertOAuth2AddOrModifyReq;
use crate::basic::dto::iam_filer_dto::IamTenantFilterReq;
use crate::basic::serv::iam_cert_user_pwd_serv::IamCertUserPwdServ;
use crate::basic::serv::oauth2_spi::iam_cert_oauth2_spi_github::IamCertOAuth2SpiGithub;
use crate::iam_config::IamBasicConfigApi;
use crate::iam_enumeration::{IamCertExtKind, IamCertOAuth2Supplier};
use bios_basic::rbum::dto::rbum_cert_conf_dto::{RbumCertConfAddReq, RbumCertConfModifyReq};
use bios_basic::rbum::dto::rbum_cert_dto::{RbumCertAddReq, RbumCertModifyReq};
use bios_basic::rbum::dto::rbum_filer_dto::{RbumCertConfFilterReq, RbumCertFilterReq};
use bios_basic::rbum::rbum_enumeration::RbumCertStatusKind::Pending;
use bios_basic::rbum::rbum_enumeration::{RbumCertConfStatusKind, RbumCertRelKind, RbumCertStatusKind, RbumScopeLevelKind};
use bios_basic::rbum::serv::rbum_cert_serv::{RbumCertConfServ, RbumCertServ};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use serde::{Deserialize, Serialize};
use tardis::web::poem_openapi;

use super::clients::iam_search_client::IamSearchClient;
use super::iam_account_serv::IamAccountServ;
use super::iam_cert_serv::IamCertServ;
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
                rel_rbum_item_id: Some(rel_iam_item_id.to_string()),
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
        RbumCertConfServ::get_rbum(id, &RbumCertConfFilterReq::default(), funs, ctx)
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

    pub async fn get_or_add_account(cert_supplier: IamCertOAuth2Supplier, code: &str, tenant_id: &str, funs: &TardisFunsInst) -> TardisResult<(String, String)> {
        let cert_conf_id = Self::get_oauth2_cert_conf_id(&cert_supplier, tenant_id, funs).await?;
        let mut mock_ctx = TardisContext {
            own_paths: tenant_id.to_string(),
            ..Default::default()
        };
        let cert_conf = Self::get_cert_conf(&cert_conf_id, funs, &mock_ctx).await?;
        let client = Self::get_access_token_func(cert_supplier.clone());
        let oauth_token_info = client.get_access_token(code, &cert_conf.ak, &cert_conf.sk, cert_conf.base_url.as_deref().unwrap_or_default(), funs).await?;
        if let Some(account_id) = Self::get_cert_rel_account_by_open_id(&oauth_token_info.open_id, &cert_conf_id, funs, &mock_ctx).await? {
            Self::cache_provider_token(&cert_supplier, &account_id, &oauth_token_info, funs).await?;
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
        Self::cache_provider_token(&cert_supplier, &account_id, &oauth_token_info, funs).await?;
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
        Self::cache_provider_token(&cert_supplier, account_id, &oauth_token_info, funs).await?;
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
    async fn get_oauth2_cert_conf_id(cert_supplier: &IamCertOAuth2Supplier, tenant_id: &str, funs: &TardisFunsInst) -> TardisResult<String> {
        let kind = IamCertExtKind::OAuth2.to_string();
        let supplier_str = cert_supplier.to_string();
        let tenant_opt = if tenant_id.is_empty() { None } else { Some(tenant_id.to_string()) };
        // 先查租户级
        if let Some(ref tid) = tenant_opt {
            if let Ok(id) = IamCertServ::get_cert_conf_id_by_kind_supplier(&kind, &supplier_str, Some(tid.clone()), funs).await {
                return Ok(id);
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
    async fn cache_provider_token(supplier: &IamCertOAuth2Supplier, account_id: &str, token_info: &IamCertOAuth2TokenInfo, funs: &TardisFunsInst) -> TardisResult<()> {
        let conf = funs.conf::<crate::iam_config::IamConfig>();
        // 与登录 token 默认时长对齐：缓存不得早于登录 token 过期
        let expire_sec = conf.oauth2_provider_token_cache_expire_sec.max(crate::iam_constants::RBUM_CERT_CONF_TOKEN_EXPIRE_SEC as u32);
        funs.cache()
            .set_ex(
                &format!("{}{}:{}", conf.cache_key_oauth2_provider_token_, supplier, account_id),
                &TardisFuns::json.obj_to_string(token_info)?,
                expire_sec as u64,
            )
            .await?;
        Ok(())
    }

    /// 读取已缓存的第三方 Provider token，不存在或已过期时返回 404
    async fn get_cached_provider_token(supplier: &IamCertOAuth2Supplier, account_id: &str, funs: &TardisFunsInst) -> TardisResult<IamCertOAuth2TokenInfo> {
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

    /// 获取已缓存的第三方 Provider token，供前端直接调用 Provider API 时使用。
    pub async fn get_provider_token(cert_supplier: IamCertOAuth2Supplier, account_id: &str, _tenant_id: &str, funs: &TardisFunsInst) -> TardisResult<IamCertOAuth2TokenInfo> {
        Self::get_cached_provider_token(&cert_supplier, account_id, funs).await
    }

    /// token 置换：使用已缓存的 refresh_token 向 Provider 换取新的 access_token，并更新缓存
    ///
    /// 用于网关/调用方在 Provider access_token 过期后刷新令牌。返回最新的 token 信息。
    pub async fn refresh_provider_token(cert_supplier: IamCertOAuth2Supplier, account_id: &str, tenant_id: &str, funs: &TardisFunsInst) -> TardisResult<IamCertOAuth2TokenInfo> {
        let cert_conf_id = Self::get_oauth2_cert_conf_id(&cert_supplier, tenant_id, funs).await?;
        let mock_ctx = TardisContext {
            own_paths: tenant_id.to_string(),
            ..Default::default()
        };
        let cert_conf = Self::get_cert_conf(&cert_conf_id, funs, &mock_ctx).await?;
        let cached = Self::get_cached_provider_token(&cert_supplier, account_id, funs).await?;
        let refresh_token = cached.refresh_token.clone().ok_or_else(|| {
            funs.err().conflict(
                "rbum_cert",
                "refresh_provider_token",
                "no refresh_token stored for this account",
                "409-iam-cert-oauth-refresh-token-missing",
            )
        })?;
        let client = Self::get_access_token_func(cert_supplier.clone());
        let mut new_token = client.refresh_access_token(&refresh_token, &cert_conf.ak, &cert_conf.sk, cert_conf.base_url.as_deref().unwrap_or_default(), funs).await?;
        // Provider 刷新响应可能不回传 open_id / refresh_token，回填旧值以保持缓存完整
        if new_token.open_id.is_empty() {
            new_token.open_id = cached.open_id.clone();
        }
        if new_token.refresh_token.is_none() {
            new_token.refresh_token = Some(refresh_token);
        }
        Self::cache_provider_token(&cert_supplier, account_id, &new_token, funs).await?;
        Ok(new_token)
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
        let cached = Self::get_cached_provider_token(&cert_supplier, account_id, funs).await?;
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
                rel_rbum_item_id: Some(rel_iam_item_id.into()),
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
    async fn refresh_access_token(&self, _refresh_token: &str, _ak: &str, _sk: &str, _base_url: &str, funs: &TardisFunsInst) -> TardisResult<IamCertOAuth2TokenInfo> {
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
}

#[derive(Clone, Debug, Serialize, Deserialize, poem_openapi::Object)]
pub struct IamCertOAuth2TokenInfo {
    pub open_id: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_expires_ms: Option<u32>,
    pub union_id: Option<String>,
}
