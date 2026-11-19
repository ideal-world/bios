use crate::basic::domain::iam_token_renewal;
use crate::basic::dto::iam_filer_dto::{IamAccountFilterReq, IamAppFilterReq, IamRoleFilterReq};
use crate::basic::dto::iam_token_renewal_dto::{
    IamTokenRenewalExchangeReq, IamTokenRenewalExchangeResp, IamTokenRenewalRegisterReq, IamTokenRenewalRegisterResp, IamTokenRenewalRevokeReq,
};
use crate::basic::serv::clients::iam_log_client::{IamLogClient, LogParamTag};
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_app_serv::IamAppServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_key_cache_serv::IamIdentCacheServ;
use crate::basic::serv::iam_role_serv::IamRoleServ;
use crate::iam_config::IamConfig;
use crate::iam_enumeration::{IamAccountStatusKind, IamCertTokenKind, IamRelKind};
use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumItemRelFilterReq};
use bios_basic::rbum::helper::rbum_event_helper;
use bios_basic::rbum::rbum_enumeration::RbumRelFromKind;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use tardis::basic::{dto::TardisContext, error::TardisError, result::TardisResult};
use tardis::chrono::{DateTime, Duration, Utc};
use tardis::db::sea_orm::sea_query::{Expr, OnConflict};
use tardis::db::sea_orm::*;
use tardis::{TardisFuns, TardisFunsInst};

pub struct IamTokenRenewalServ;

fn not_registered() -> TardisError {
    TardisError::custom(
        "404-token-renewal-not-registered",
        "the token renewal authorization is not registered",
        "404-token-renewal-not-registered",
    )
}

fn renewal_revoked() -> TardisError {
    TardisError::custom(
        "403-token-renewal-revoked",
        "the token renewal authorization has been revoked",
        "403-token-renewal-revoked",
    )
}

fn renewal_expired() -> TardisError {
    TardisError::custom(
        "401-token-renewal-expired",
        "the token renewal authorization has expired",
        "401-token-renewal-expired",
    )
}

fn principal_mismatch() -> TardisError {
    TardisError::custom(
        "409-token-renewal-principal-mismatch",
        "the run or capability does not match the registered principal",
        "409-token-renewal-principal-mismatch",
    )
}

fn account_invalid() -> TardisError {
    TardisError::custom(
        "403-token-renewal-account-invalid",
        "the account of the token renewal authorization is no longer valid",
        "403-token-renewal-account-invalid",
    )
}

fn membership_invalid() -> TardisError {
    TardisError::custom(
        "403-token-renewal-membership-invalid",
        "the app membership of the token renewal authorization is no longer valid",
        "403-token-renewal-membership-invalid",
    )
}

fn exchange_limit() -> TardisError {
    TardisError::custom(
        "429-token-renewal-exchange-limit",
        "the token renewal exchange limit has been exceeded",
        "429-token-renewal-exchange-limit",
    )
}

fn renewal_conflict() -> TardisError {
    TardisError::custom(
        "409-token-renewal-conflict",
        "the token renewal authorization changed; register again",
        "409-token-renewal-conflict",
    )
}

fn invalid_request() -> TardisError {
    TardisError::custom(
        "400-token-renewal-invalid-request",
        "the token renewal request is not valid",
        "400-token-renewal-invalid-request",
    )
}

fn valid_binding_key(key: &str) -> bool {
    !key.trim().is_empty() && key.len() <= 256
}

fn renewal_record_expired(expires_at: &str) -> TardisResult<bool> {
    let expires_at = DateTime::parse_from_rfc3339(expires_at)
        .map_err(|error| TardisError::internal_error(&format!("[Iam] parse token renewal expires_at error: {error}"), "500-token-renewal-expires-at-invalid"))?
        .with_timezone(&Utc);
    Ok(Utc::now() > expires_at)
}

impl IamTokenRenewalServ {
    /// 注册运行级续期授权（契约 §5.1）；调用方负责事务。
    ///
    /// 同一 `(run_id, secret_key)` 幂等替换：照 task grant 的替换语义把旧记录置 revoked 并保留
    /// 审计链路，而不是物理删除——同时清空 current 槽位，保证旧 capability 在替换提交后立即不可兑换。
    /// 不提供管理员接管语义；仅同一主体同一 scope 可以替换。
    pub async fn register(req: &IamTokenRenewalRegisterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<IamTokenRenewalRegisterResp> {
        if !valid_binding_key(&req.run_id) || !valid_binding_key(&req.secret_key) {
            return Err(invalid_request());
        }
        let app_id = Self::require_valid_principal(&ctx.owner, &ctx.own_paths, funs).await?;
        let config = funs.conf::<IamConfig>();
        let cap_sec = config.token_renewal_ttl_cap_sec as u64;
        if cap_sec == 0 || config.token_renewal_mint_ttl_sec == 0 {
            return Err(invalid_request());
        }
        // 授权时长取申请值与配置上限的较小者；缺省按上限处理。
        let ttl_sec = req.ttl_sec.unwrap_or(cap_sec).clamp(1, cap_sec);
        let now = Utc::now();
        let expires_at = (now + Duration::seconds(ttl_sec as i64)).to_rfc3339();
        let id = TardisFuns::field.nanoid();
        // 256-bit 随机 capability；服务端只存 SHA-256 哈希，明文仅本次响应可见一次。
        let capability = TardisFuns::crypto.key.rand_64_hex();
        let capability_hash = TardisFuns::crypto.digest.sha256(capability.as_bytes())?;

        let current = Self::find_current(&req.run_id, &req.secret_key, funs).await?;
        if let Some(current) = &current {
            // 用户认证注册不能借可猜测的 run/key 撤销或接管其他主体的授权。
            if current.account_id != ctx.owner || current.own_paths != ctx.own_paths {
                return Err(principal_mismatch());
            }
            let updated = iam_token_renewal::Entity::update_many()
                .col_expr(iam_token_renewal::Column::CurrentRunId, Expr::value(Option::<String>::None))
                .col_expr(iam_token_renewal::Column::CurrentSecretKey, Expr::value(Option::<String>::None))
                .col_expr(
                    iam_token_renewal::Column::RevokedBy,
                    Expr::case(iam_token_renewal::Column::Revoked.eq(false), Expr::value(Some(ctx.owner.clone())))
                        .finally(Expr::col(iam_token_renewal::Column::RevokedBy))
                        .into(),
                )
                .col_expr(
                    iam_token_renewal::Column::RevokedTime,
                    Expr::case(iam_token_renewal::Column::Revoked.eq(false), Expr::value(Some(now.to_rfc3339())))
                        .finally(Expr::col(iam_token_renewal::Column::RevokedTime))
                        .into(),
                )
                .col_expr(iam_token_renewal::Column::Revoked, Expr::value(true))
                .col_expr(iam_token_renewal::Column::ReplacedByCredentialId, Expr::value(Some(id.clone())))
                .filter(iam_token_renewal::Column::Id.eq(current.id.clone()))
                .filter(iam_token_renewal::Column::CurrentRunId.eq(req.run_id.clone()))
                .filter(iam_token_renewal::Column::CurrentSecretKey.eq(req.secret_key.clone()))
                .exec(funs.db().raw_tx()?)
                .await?;
            if updated.rows_affected != 1 {
                return Err(renewal_conflict());
            }
        }

        let model = iam_token_renewal::ActiveModel {
            id: Set(id.clone()),
            run_id: Set(req.run_id.clone()),
            secret_key: Set(req.secret_key.clone()),
            account_id: Set(ctx.owner.clone()),
            own_paths: Set(ctx.own_paths.clone()),
            app_id: Set(app_id),
            capability_hash: Set(capability_hash),
            expires_at: Set(expires_at.clone()),
            last_refreshed_at: Set(None),
            revoked: Set(false),
            revoked_by: Set(None),
            revoked_time: Set(None),
            create_time: Set(now.to_rfc3339()),
            current_run_id: Set(Some(req.run_id.clone())),
            current_secret_key: Set(Some(req.secret_key.clone())),
            replaced_by_credential_id: Set(None),
        };
        iam_token_renewal::Entity::insert(model)
            .on_conflict(
                OnConflict::columns([iam_token_renewal::Column::CurrentRunId, iam_token_renewal::Column::CurrentSecretKey])
                    .do_nothing()
                    .to_owned(),
            )
            .exec_without_returning(funs.db().raw_tx()?)
            .await?;
        if Self::find_by_id(&id, funs).await?.is_none() {
            return Err(renewal_conflict());
        }

        let replaced_id = current.map(|record| record.id);
        rbum_event_helper::add_notify_event("iam_token_renewal", "c", &id, ctx).await?;
        if let Some(previous_id) = replaced_id.as_deref() {
            rbum_event_helper::add_notify_event("iam_token_renewal", "u", previous_id, ctx).await?;
        }
        let description = match replaced_id.as_deref() {
            Some(previous_id) => format!(
                "替换工作流凭证续期授权 run_id={} secret_key={} previous_credential_id={} credential_id={}",
                req.run_id, req.secret_key, previous_id, id
            ),
            None => format!("登记工作流凭证续期授权 run_id={} secret_key={} credential_id={}", req.run_id, req.secret_key, id),
        };
        // 审计走现有 iam log；描述只含声明键名，绝不包含 capability 明文。
        IamLogClient::add_ctx_task(LogParamTag::Token, Some(id.clone()), description, Some("TokenRenewalRegister".to_string()), ctx).await?;

        Ok(IamTokenRenewalRegisterResp {
            credential_id: id,
            capability,
            expires_at,
        })
    }

    /// 兑换运行级续期授权（契约 §5.2）；服务 token 守卫在 API 层完成，本方法不依赖用户上下文。
    ///
    /// 校验顺序：记录存在 → 未吊销 → 未过期 → `run_id` 与 capability 哈希匹配 → 账号/应用成员关系
    /// 仍成立 → 限流 → 签发新 token。capability 不轮换、可重复兑换；授权的绝对 `expires_at`
    /// 不因兑换向后延长，每次兑换只更新 `last_refreshed_at`。
    pub async fn exchange(credential_id: &str, req: &IamTokenRenewalExchangeReq, _funs: &TardisFunsInst) -> TardisResult<IamTokenRenewalExchangeResp> {
        let mut funs = crate::iam_constants::get_tardis_inst();
        funs.begin().await?;
        iam_token_renewal::Entity::find_by_id(credential_id).lock_exclusive().one(funs.db().raw_tx()?).await?;
        match Self::exchange_locked(credential_id, req, &funs).await {
            Ok(resp) => { funs.commit().await?; Ok(resp) }
            Err(error) => { funs.rollback().await?; Err(error) }
        }
    }

    async fn exchange_locked(credential_id: &str, req: &IamTokenRenewalExchangeReq, funs: &TardisFunsInst) -> TardisResult<IamTokenRenewalExchangeResp> {
        let record = Self::find_by_id(credential_id, funs).await?.ok_or_else(not_registered)?;
        if record.revoked {
            return Err(renewal_revoked());
        }
        if renewal_record_expired(&record.expires_at)? {
            return Err(renewal_expired());
        }
        Self::validate_binding(&record, &req.run_id, &req.capability)?;
        // 只续期，不提权：兑换时重新校验账号状态与应用成员关系，拒绝即为兑换失败。
        Self::require_valid_principal(&record.account_id, &record.own_paths, funs).await?;
        Self::check_exchange_limit(&record.id, funs).await?;
        let runtime_ctx = Self::build_runtime_context(&record, funs).await?;
        let bios_token = TardisFuns::crypto.key.generate_token()?;
        let config = funs.conf::<IamConfig>();
        let remaining = (DateTime::parse_from_rfc3339(&record.expires_at).map_err(|_| invalid_request())?.with_timezone(&Utc) - Utc::now()).num_seconds();
        let mint_ttl = (config.token_renewal_mint_ttl_sec as i64).min(remaining);
        if mint_ttl <= 0 { return Err(renewal_expired()); }
        // 照 task grant runtime：任务 token 独立保存应用上下文，不修改用户当前登录的租户，也不续期；
        // 续期签发的 token 使用独立缓存前缀 iam:renewal-token:，解析路径统一在
        // IamIdentCacheServ::get_context 与 auth_kernel_serv::get_token_context 内回退兼容。
        funs.cache()
            .set_ex(&format!("iam:renewal-token:{bios_token}"), &TardisFuns::json.obj_to_string(&runtime_ctx)?, mint_ttl as u64)
            .await?;
        IamIdentCacheServ::add_token(&bios_token, &IamCertTokenKind::TokenTask, &record.account_id, None, mint_ttl, 0, funs).await?;
        let now = Utc::now();
        iam_token_renewal::Entity::update_many()
            .col_expr(iam_token_renewal::Column::LastRefreshedAt, Expr::value(Some(now.to_rfc3339())))
            .filter(iam_token_renewal::Column::Id.eq(record.id.clone()))
            .filter(iam_token_renewal::Column::Revoked.eq(false))
            .exec(funs.db().raw_tx()?)
            .await?;
        Ok(IamTokenRenewalExchangeResp {
            access_token: bios_token,
            expires_at: (now + Duration::seconds(mint_ttl)).to_rfc3339(),
        })
    }

    /// 吊销运行级续期授权（契约 §5.3）；校验 `run_id + capability` 后置 revoked。
    ///
    /// 照替换语义保留 revoked 记录并清空 current 槽位：既保留审计链路，又允许同
    /// `(run_id, secret_key)` 在重新注册时拿到全新授权。宿主服务吊销无用户主体，
    /// `revoked_by` 留空，仅记录 `revoked_time`。
    pub async fn revoke(credential_id: &str, req: &IamTokenRenewalRevokeReq, _funs: &TardisFunsInst) -> TardisResult<()> {
        let mut funs = crate::iam_constants::get_tardis_inst();
        funs.begin().await?;
        iam_token_renewal::Entity::find_by_id(credential_id).lock_exclusive().one(funs.db().raw_tx()?).await?;
        match Self::revoke_locked(credential_id, req, &funs).await {
            Ok(()) => { funs.commit().await?; Ok(()) }
            Err(error) => { funs.rollback().await?; Err(error) }
        }
    }

    async fn revoke_locked(credential_id: &str, req: &IamTokenRenewalRevokeReq, funs: &TardisFunsInst) -> TardisResult<()> {
        let record = Self::find_by_id(credential_id, funs).await?.ok_or_else(not_registered)?;
        Self::validate_binding(&record, &req.run_id, &req.capability)?;
        if record.revoked {
            // 幂等：已吊销直接成功，照 task grant revoke 的幂等语义。
            return Ok(());
        }
        let updated = iam_token_renewal::Entity::update_many()
            .col_expr(iam_token_renewal::Column::CurrentRunId, Expr::value(Option::<String>::None))
            .col_expr(iam_token_renewal::Column::CurrentSecretKey, Expr::value(Option::<String>::None))
            .col_expr(iam_token_renewal::Column::Revoked, Expr::value(true))
            .col_expr(iam_token_renewal::Column::RevokedTime, Expr::value(Some(Utc::now().to_rfc3339())))
            .filter(iam_token_renewal::Column::Id.eq(record.id.clone()))
            .filter(iam_token_renewal::Column::Revoked.eq(false))
            .exec(funs.db().raw_tx()?)
            .await?;
        if updated.rows_affected == 0 {
            let current = Self::find_by_id(&record.id, funs).await?.ok_or_else(not_registered)?;
            if !current.revoked {
                return Err(renewal_conflict());
            }
            return Ok(());
        }
        Ok(())
    }

    /// 绑定校验：`run_id` 与 capability 的 SHA-256 哈希必须同时匹配注册主体。
    fn validate_binding(record: &iam_token_renewal::Model, run_id: &str, capability: &str) -> TardisResult<()> {
        let capability_hash = TardisFuns::crypto.digest.sha256(capability.as_bytes())?;
        if record.run_id != run_id || record.capability_hash != capability_hash {
            return Err(principal_mismatch());
        }
        Ok(())
    }

    /// 账号/应用校验路径照抄 IamOAuth2TaskGrantServ::require_membership：账号存在且 Active、
    /// 未禁用未锁定；带应用上下文（`tenant/app`）时要求应用直接成员关系且至少一个应用内角色。
    /// 续期授权允许无应用上下文（仅 `tenant`），此时只校验账号并返回 `None`。
    async fn require_valid_principal(account_id: &str, scope: &str, funs: &TardisFunsInst) -> TardisResult<Option<String>> {
        let parts = scope.split('/').collect::<Vec<_>>();
        if account_id.is_empty() || (!scope.is_empty() && parts.iter().any(|part| part.is_empty())) {
            return Err(invalid_request());
        }
        if parts.len() > 2 {
            return Err(invalid_request());
        }
        let tenant_ctx = TardisContext {
            owner: account_id.to_string(),
            own_paths: parts[0].to_string(),
            ..Default::default()
        };
        let account = IamAccountServ::get_item(
            account_id,
            &IamAccountFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some(String::new()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            &tenant_ctx,
        )
        .await
        .map_err(|error| if error.code.starts_with("404") { account_invalid() } else { error })?;
        if account.status != IamAccountStatusKind::Active || !IamCertServ::is_account_enabled_and_unlocked(account.disabled, &account.lock_status) {
            return Err(account_invalid());
        }
        if parts.len() == 1 {
            return Ok(None);
        }

        let apps = IamAppServ::find_items(
            &IamAppFilterReq {
                basic: RbumBasicFilterReq {
                    ids: Some(vec![parts[1].to_string()]),
                    enabled: Some(true),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                rel: Some(RbumItemRelFilterReq {
                    rel_by_from: false,
                    optional: false,
                    tag: Some(IamRelKind::IamAccountApp.to_string()),
                    from_rbum_kind: Some(RbumRelFromKind::Item),
                    rel_item_id: Some(account_id.to_string()),
                    own_paths: Some(scope.to_string()),
                    disabled: Some(false),
                    ..Default::default()
                }),
                ..Default::default()
            },
            None,
            None,
            funs,
            &tenant_ctx,
        )
        .await?;
        if apps.len() != 1 {
            return Err(membership_invalid());
        }

        let roles = IamRoleServ::find_items(
            &IamRoleFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some(scope.to_string()),
                    enabled: Some(true),
                    ..Default::default()
                },
                rel: Some(RbumItemRelFilterReq {
                    rel_by_from: false,
                    optional: false,
                    tag: Some(IamRelKind::IamAccountRole.to_string()),
                    from_rbum_kind: Some(RbumRelFromKind::Item),
                    rel_item_id: Some(account_id.to_string()),
                    own_paths: Some(scope.to_string()),
                    disabled: Some(false),
                    ..Default::default()
                }),
                ..Default::default()
            },
            None,
            None,
            funs,
            &tenant_ctx,
        )
        .await?;
        if roles.is_empty() {
            return Err(membership_invalid());
        }
        Ok(Some(parts[1].to_string()))
    }

    /// 照 task grant runtime 重建运行上下文；应用上下文独立于用户当前登录租户。
    async fn build_runtime_context(record: &iam_token_renewal::Model, funs: &TardisFunsInst) -> TardisResult<TardisContext> {
        let record_ctx = TardisContext {
            owner: record.account_id.clone(),
            own_paths: record.own_paths.clone(),
            ..Default::default()
        };
        let details = IamAccountServ::get_account_detail_aggs(
            &record.account_id,
            &IamAccountFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some(String::new()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            false,
            true,
            false,
            funs,
            &record_ctx,
        )
        .await?;
        let runtime_ctx = match &record.app_id {
            // 注册时带应用上下文：照 task grant 只接受应用直接成员关系下的角色/群组。
            Some(app_id) => {
                let app = details
                    .apps
                    .into_iter()
                    .find(|app| app.app_id == *app_id && app.app_own_paths == record.own_paths)
                    .ok_or_else(account_invalid)?;
                TardisContext {
                    owner: record.account_id.clone(),
                    own_paths: record.own_paths.clone(),
                    roles: app.roles.into_keys().collect(),
                    groups: app.groups.into_keys().collect(),
                    ..Default::default()
                }
            }
            // 注册时无应用上下文：使用租户维度角色/群组，等价于租户级登录 token 的上下文。
            None => TardisContext {
                owner: record.account_id.clone(),
                own_paths: record.own_paths.clone(),
                roles: details.roles.into_keys().collect(),
                groups: details.groups.into_keys().collect(),
                ..Default::default()
            },
        };
        Ok(runtime_ctx)
    }

    /// 单授权兑换计数：Redis 固定一小时窗口（自该授权首次成功兑换起算），超限归入拒绝错误码；
    /// 限流拒绝不吊销授权，capability 仍然有效，窗口过后可继续兑换。
    async fn check_exchange_limit(credential_id: &str, funs: &TardisFunsInst) -> TardisResult<()> {
        let limit = funs.conf::<IamConfig>().token_renewal_exchange_hourly_limit;
        if limit == 0 {
            return Ok(());
        }
        let key = format!("iam:renewal-exchange-count:{credential_id}");
        let count = funs.cache().incr(&key, 1).await?;
        if count == 1 {
            funs.cache().expire(&key, 3600).await?;
        }
        if count > limit as isize {
            return Err(exchange_limit());
        }
        Ok(())
    }

    async fn find_current_on<C: ConnectionTrait>(run_id: &str, secret_key: &str, db: &C) -> TardisResult<Option<iam_token_renewal::Model>> {
        iam_token_renewal::Entity::find()
            .filter(iam_token_renewal::Column::CurrentRunId.eq(run_id))
            .filter(iam_token_renewal::Column::CurrentSecretKey.eq(secret_key))
            .one(db)
            .await
            .map_err(Into::into)
    }

    async fn find_current(run_id: &str, secret_key: &str, funs: &TardisFunsInst) -> TardisResult<Option<iam_token_renewal::Model>> {
        let db = funs.db();
        if db.has_tx() {
            Self::find_current_on(run_id, secret_key, db.raw_tx()?).await
        } else {
            Self::find_current_on(run_id, secret_key, db.raw_conn()).await
        }
    }

    async fn find_by_id_on<C: ConnectionTrait>(credential_id: &str, db: &C) -> TardisResult<Option<iam_token_renewal::Model>> {
        iam_token_renewal::Entity::find_by_id(credential_id).one(db).await.map_err(Into::into)
    }

    async fn find_by_id(credential_id: &str, funs: &TardisFunsInst) -> TardisResult<Option<iam_token_renewal::Model>> {
        let db = funs.db();
        if db.has_tx() {
            Self::find_by_id_on(credential_id, db.raw_tx()?).await
        } else {
            Self::find_by_id_on(credential_id, db.raw_conn()).await
        }
    }
}

#[cfg(test)]
mod token_renewal_tests {
    use super::*;

    fn record(run_id: &str, capability: &str) -> iam_token_renewal::Model {
        iam_token_renewal::Model {
            id: "credential-current".into(),
            run_id: run_id.into(),
            secret_key: "Bios-Token".into(),
            account_id: "alice".into(),
            own_paths: "tenant/app".into(),
            app_id: Some("app".into()),
            capability_hash: TardisFuns::crypto.digest.sha256(capability.as_bytes()).unwrap(),
            expires_at: Utc::now().to_rfc3339(),
            last_refreshed_at: None,
            revoked: false,
            revoked_by: None,
            revoked_time: None,
            create_time: String::new(),
            current_run_id: Some(run_id.into()),
            current_secret_key: Some("Bios-Token".into()),
            replaced_by_credential_id: None,
        }
    }

    #[test]
    fn exchange_requires_both_the_run_and_the_capability_hash_to_match() {
        let current = record("run-1", "capability-1");
        assert!(IamTokenRenewalServ::validate_binding(&current, "run-1", "capability-1").is_ok());
        assert_eq!(
            IamTokenRenewalServ::validate_binding(&current, "run-2", "capability-1").unwrap_err().code,
            "409-token-renewal-principal-mismatch"
        );
        assert_eq!(
            IamTokenRenewalServ::validate_binding(&current, "run-1", "capability-2").unwrap_err().code,
            "409-token-renewal-principal-mismatch"
        );
        assert_eq!(
            IamTokenRenewalServ::validate_binding(&current, "run-1", "").unwrap_err().code,
            "409-token-renewal-principal-mismatch"
        );
    }

    #[test]
    fn expiry_is_decided_by_the_absolute_deadline_only() {
        let past = (Utc::now() - Duration::seconds(1)).to_rfc3339();
        let future = (Utc::now() + Duration::seconds(3600)).to_rfc3339();
        assert!(renewal_record_expired(&past).unwrap());
        assert!(!renewal_record_expired(&future).unwrap());
    }

    #[test]
    fn binding_keys_must_be_short_and_non_empty() {
        assert!(valid_binding_key("wfrun_01M7PK6K6D2X3BNVT66DH7N8FN"));
        assert!(!valid_binding_key(""));
        assert!(!valid_binding_key("   "));
        assert!(!valid_binding_key(&"r".repeat(257)));
    }
}
