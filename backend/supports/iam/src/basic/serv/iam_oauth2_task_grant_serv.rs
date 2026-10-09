use crate::basic::domain::iam_oauth2_task_grant;
use crate::basic::dto::iam_filer_dto::{IamAccountFilterReq, IamAppFilterReq, IamRoleFilterReq};
use crate::basic::dto::iam_oauth2_task_grant_dto::{IamOAuth2TaskGrantCreateReq, IamOAuth2TaskGrantExchangeReq, IamOAuth2TaskGrantRef, IamOAuth2TaskGrantToken};
use crate::basic::serv::clients::iam_log_client::{IamLogClient, LogParamTag};
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_app_serv::IamAppServ;
use crate::basic::serv::iam_cert_oauth2_serv::IamCertOAuth2Serv;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_role_serv::IamRoleServ;
use crate::iam_config::{IamBasicConfigApi, IamConfig};
use crate::iam_enumeration::{IamAccountStatusKind, IamRelKind};
use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumItemRelFilterReq};
use bios_basic::rbum::helper::rbum_event_helper;
use bios_basic::rbum::rbum_enumeration::RbumRelFromKind;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use serde::Serialize;
use tardis::basic::{dto::TardisContext, error::TardisError, result::TardisResult};
use tardis::chrono::Utc;
use tardis::db::sea_orm::sea_query::{Expr, OnConflict};
use tardis::db::sea_orm::*;
use tardis::{TardisFuns, TardisFunsInst};

pub struct IamOAuth2TaskGrantServ;

fn authorization_required() -> TardisError {
    TardisError::custom(
        "409-iam-oauth-task-authorization-required",
        "background OAuth authorization is unavailable; authorize the task again",
        "409-iam-oauth-task-authorization-required",
    )
}

fn grant_conflict() -> TardisError {
    TardisError::custom(
        "409-iam-oauth-task-grant-conflict",
        "task grant changed; reload the current authorization and retry",
        "409-iam-oauth-task-grant-conflict",
    )
}

fn takeover_forbidden() -> TardisError {
    TardisError::custom(
        "403-iam-oauth-task-takeover-forbidden",
        "replacing another account's task grant requires application administrator authority",
        "403-iam-oauth-task-takeover-forbidden",
    )
}

fn current_forbidden() -> TardisError {
    TardisError::custom(
        "403-iam-oauth-task-current-forbidden",
        "the current task grant is available only to its owner or an application administrator",
        "403-iam-oauth-task-current-forbidden",
    )
}

fn valid_task_id(task_id: &str) -> bool {
    !task_id.trim().is_empty() && task_id.len() <= 256
}

fn validate_exchange(grant: &iam_oauth2_task_grant::Model, executor: &str, configured_executor: &str, scope: &str, task_id: &str) -> TardisResult<()> {
    if configured_executor.is_empty()
        || executor != configured_executor
        || executor != grant.executor_account_id
        || scope != grant.own_paths
        || task_id != grant.task_id
        || grant.revoked
    {
        return Err(authorization_required());
    }
    Ok(())
}

fn validate_expected_grant(
    expected_grant_id: Option<&str>,
    takeover: bool,
    account_id: &str,
    current: Option<&iam_oauth2_task_grant::Model>,
    is_app_admin: bool,
) -> TardisResult<()> {
    match (current, expected_grant_id) {
        (None, None) => Ok(()),
        (None, Some(_)) | (Some(_), None) => Err(grant_conflict()),
        (Some(current), Some(expected)) if current.id != expected => Err(grant_conflict()),
        (Some(current), Some(_)) if current.account_id != account_id && !(takeover && is_app_admin) => Err(takeover_forbidden()),
        (Some(_), Some(_)) => Ok(()),
    }
}

impl IamOAuth2TaskGrantServ {
    /// 委托只接受应用直接成员关系和应用直接角色关系，不使用应用集合或任务启动时的权限快照。
    async fn require_membership(account_id: &str, scope: &str, funs: &TardisFunsInst) -> TardisResult<String> {
        let parts = scope.split('/').collect::<Vec<_>>();
        if parts.len() != 2 || parts.iter().any(|part| part.is_empty()) || account_id.is_empty() {
            return Err(authorization_required());
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
        .map_err(|error| if error.code.starts_with("404") { authorization_required() } else { error })?;
        if account.status != IamAccountStatusKind::Active || !IamCertServ::is_account_enabled_and_unlocked(account.disabled, &account.lock_status) {
            return Err(authorization_required());
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
            return Err(authorization_required());
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
            return Err(authorization_required());
        }
        Ok(parts[1].to_string())
    }

    async fn find_current_on<C: ConnectionTrait>(task_id: &str, scope: &str, db: &C) -> TardisResult<Option<iam_oauth2_task_grant::Model>> {
        iam_oauth2_task_grant::Entity::find()
            .filter(iam_oauth2_task_grant::Column::CurrentOwnPaths.eq(scope))
            .filter(iam_oauth2_task_grant::Column::CurrentTaskId.eq(task_id))
            .one(db)
            .await
            .map_err(Into::into)
    }

    async fn find_current(task_id: &str, scope: &str, funs: &TardisFunsInst) -> TardisResult<Option<iam_oauth2_task_grant::Model>> {
        let db = funs.db();
        if db.has_tx() {
            Self::find_current_on(task_id, scope, db.raw_tx()?).await
        } else {
            Self::find_current_on(task_id, scope, db.raw_conn()).await
        }
    }

    async fn find_by_id_on<C: ConnectionTrait>(grant_id: &str, db: &C) -> TardisResult<Option<iam_oauth2_task_grant::Model>> {
        iam_oauth2_task_grant::Entity::find_by_id(grant_id).one(db).await.map_err(Into::into)
    }

    async fn find_by_id(grant_id: &str, funs: &TardisFunsInst) -> TardisResult<Option<iam_oauth2_task_grant::Model>> {
        let db = funs.db();
        if db.has_tx() {
            Self::find_by_id_on(grant_id, db.raw_tx()?).await
        } else {
            Self::find_by_id_on(grant_id, db.raw_conn()).await
        }
    }

    async fn has_enabled_app_admin(account_id: &str, scope: &str, funs: &TardisFunsInst) -> TardisResult<bool> {
        let app_ctx = TardisContext {
            owner: account_id.to_string(),
            own_paths: scope.to_string(),
            ..Default::default()
        };
        let role_id = match IamRoleServ::get_embed_sub_role_id(&funs.iam_basic_role_app_admin_id(), funs, &app_ctx).await {
            Ok(role_id) => role_id,
            Err(error) if error.code.starts_with("404") => return Ok(false),
            Err(error) => return Err(error),
        };
        let roles = IamRoleServ::find_items(
            &IamRoleFilterReq {
                basic: RbumBasicFilterReq {
                    ids: Some(vec![role_id]),
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
            &app_ctx,
        )
        .await?;
        Ok(!roles.is_empty())
    }

    async fn prepare_create(req: &IamOAuth2TaskGrantCreateReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<(String, Option<iam_oauth2_task_grant::Model>)> {
        if funs.conf::<IamConfig>().oauth2_task_executor_account_id.is_empty() || !valid_task_id(&req.task_id) {
            return Err(authorization_required());
        }
        let app_id = Self::require_membership(&ctx.owner, &ctx.own_paths, funs).await?;
        let current = Self::find_current(&req.task_id, &ctx.own_paths, funs).await?;
        let is_app_admin = if current.as_ref().is_some_and(|grant| grant.account_id != ctx.owner) && req.takeover {
            Self::has_enabled_app_admin(&ctx.owner, &ctx.own_paths, funs).await?
        } else {
            false
        };
        validate_expected_grant(req.expected_grant_id.as_deref(), req.takeover, &ctx.owner, current.as_ref(), is_app_admin)?;
        Ok((app_id, current))
    }

    /// 在刷新 Provider 凭证前完成成员关系、CAS 和接管权限预检。
    pub async fn validate_create(req: &IamOAuth2TaskGrantCreateReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        Self::prepare_create(req, funs, ctx).await.map(|_| ())
    }

    /// 调用方负责事务；本方法在修改授权槽位前重新检查预检结果。
    pub async fn create(req: &IamOAuth2TaskGrantCreateReq, provider_grant_id: Option<&str>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<IamOAuth2TaskGrantRef> {
        let (app_id, current) = Self::prepare_create(req, funs, ctx).await?;
        let provider_grant_id = if req.requires_external_oauth {
            provider_grant_id.filter(|id| !id.trim().is_empty()).ok_or_else(authorization_required)?.to_string()
        } else if provider_grant_id.is_some() {
            return Err(authorization_required());
        } else {
            String::new()
        };
        let id = TardisFuns::field.nanoid();
        let now = Utc::now().to_rfc3339();

        if let Some(current) = &current {
            let updated = iam_oauth2_task_grant::Entity::update_many()
                .col_expr(iam_oauth2_task_grant::Column::CurrentOwnPaths, Expr::value(Option::<String>::None))
                .col_expr(iam_oauth2_task_grant::Column::CurrentTaskId, Expr::value(Option::<String>::None))
                .col_expr(
                    iam_oauth2_task_grant::Column::RevokedBy,
                    Expr::case(iam_oauth2_task_grant::Column::Revoked.eq(false), Expr::value(Some(ctx.owner.clone())))
                        .finally(Expr::col(iam_oauth2_task_grant::Column::RevokedBy))
                        .into(),
                )
                .col_expr(
                    iam_oauth2_task_grant::Column::RevokedTime,
                    Expr::case(iam_oauth2_task_grant::Column::Revoked.eq(false), Expr::value(Some(now.clone())))
                        .finally(Expr::col(iam_oauth2_task_grant::Column::RevokedTime))
                        .into(),
                )
                .col_expr(iam_oauth2_task_grant::Column::Revoked, Expr::value(true))
                .col_expr(iam_oauth2_task_grant::Column::ReplacedByGrantId, Expr::value(Some(id.clone())))
                .filter(iam_oauth2_task_grant::Column::Id.eq(current.id.clone()))
                .filter(iam_oauth2_task_grant::Column::CurrentOwnPaths.eq(ctx.own_paths.clone()))
                .filter(iam_oauth2_task_grant::Column::CurrentTaskId.eq(req.task_id.clone()))
                .exec(funs.db().raw_tx()?)
                .await?;
            if updated.rows_affected != 1 {
                return Err(grant_conflict());
            }
        }

        let model = iam_oauth2_task_grant::ActiveModel {
            id: Set(id.clone()),
            task_id: Set(req.task_id.clone()),
            account_id: Set(ctx.owner.clone()),
            app_id: Set(app_id),
            provider_grant_id: Set(provider_grant_id),
            executor_account_id: Set(funs.conf::<IamConfig>().oauth2_task_executor_account_id.clone()),
            revoked: Set(false),
            create_time: Set(now),
            own_paths: Set(ctx.own_paths.clone()),
            current_own_paths: Set(Some(ctx.own_paths.clone())),
            current_task_id: Set(Some(req.task_id.clone())),
            revoked_by: Set(None),
            revoked_time: Set(None),
            replaced_by_grant_id: Set(None),
        };
        iam_oauth2_task_grant::Entity::insert(model)
            .on_conflict(OnConflict::columns([iam_oauth2_task_grant::Column::CurrentOwnPaths, iam_oauth2_task_grant::Column::CurrentTaskId]).do_nothing().to_owned())
            .exec_without_returning(funs.db().raw_tx()?)
            .await?;
        if Self::find_by_id(&id, funs).await?.is_none() {
            return Err(grant_conflict());
        }

        let replaced_id = current.map(|grant| grant.id);
        rbum_event_helper::add_notify_event("iam_oauth2_task_grant", "c", &id, ctx).await?;
        if let Some(previous_id) = replaced_id.as_deref() {
            rbum_event_helper::add_notify_event("iam_oauth2_task_grant", "u", previous_id, ctx).await?;
        }
        let description = match replaced_id.as_deref() {
            Some(previous_id) => format!("替换定时任务 OAuth 授权 task_id={} previous_grant_id={} grant_id={}", req.task_id, previous_id, id),
            None => format!("创建定时任务 OAuth 授权 task_id={} grant_id={}", req.task_id, id),
        };
        IamLogClient::add_ctx_task(LogParamTag::Token, Some(id.clone()), description, Some("OAuthTaskGrantCreate".to_string()), ctx).await?;

        Ok(IamOAuth2TaskGrantRef {
            grant_id: id,
            account_id: ctx.owner.clone(),
        })
    }

    /// 仅返回最新授权引用；即使已撤销也保留该槽位供 CAS 恢复使用。
    pub async fn current(task_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<IamOAuth2TaskGrantRef>> {
        if !valid_task_id(task_id) {
            return Err(authorization_required());
        }
        Self::require_membership(&ctx.owner, &ctx.own_paths, funs).await?;
        let Some(current) = Self::find_current(task_id, &ctx.own_paths, funs).await? else {
            return Ok(None);
        };
        if current.account_id != ctx.owner && !Self::has_enabled_app_admin(&ctx.owner, &ctx.own_paths, funs).await? {
            return Err(current_forbidden());
        }
        Ok(Some(IamOAuth2TaskGrantRef {
            grant_id: current.id,
            account_id: current.account_id,
        }))
    }

    pub async fn exchange(req: &IamOAuth2TaskGrantExchangeReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<IamOAuth2TaskGrantToken> {
        let grant = Self::find_by_id(&req.grant_id, funs).await?.ok_or_else(authorization_required)?;
        validate_exchange(&grant, &ctx.owner, &funs.conf::<IamConfig>().oauth2_task_executor_account_id, &ctx.own_paths, &req.task_id)?;
        Self::require_membership(&grant.account_id, &grant.own_paths, funs).await?;
        if grant.provider_grant_id.is_empty() {
            Self::revalidate_grant(&grant, req, funs, ctx).await?;
            return Ok(IamOAuth2TaskGrantToken { access_token: None });
        }
        let token = match IamCertOAuth2Serv::get_validated_grant_token(&grant.provider_grant_id, &grant.account_id, &grant.own_paths, funs).await {
            Ok(token) => token,
            Err(error) if ["401-", "403-", "404-", "409-"].iter().any(|prefix| error.code.starts_with(prefix)) => return Err(authorization_required()),
            Err(error) => return Err(error),
        };
        Self::revalidate_grant(&grant, req, funs, ctx).await?;
        Ok(IamOAuth2TaskGrantToken {
            access_token: Some(token.access_token),
        })
    }

    async fn revalidate_grant(grant: &iam_oauth2_task_grant::Model, req: &IamOAuth2TaskGrantExchangeReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        Self::require_membership(&grant.account_id, &grant.own_paths, funs).await?;
        let current = Self::find_by_id(&grant.id, funs).await?.ok_or_else(authorization_required)?;
        validate_exchange(
            &current,
            &ctx.owner,
            &funs.conf::<IamConfig>().oauth2_task_executor_account_id,
            &ctx.own_paths,
            &req.task_id,
        )
    }

    /// 调用方负责事务；普通撤销保留最新槽位作为下一次 CAS 基线。
    pub async fn revoke(grant_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let grant = Self::find_by_id(grant_id, funs).await?.ok_or_else(authorization_required)?;
        if grant.own_paths != ctx.own_paths || (grant.account_id != ctx.owner && grant.executor_account_id != ctx.owner) {
            return Err(authorization_required());
        }
        if grant.revoked {
            return Ok(());
        }
        let now = Utc::now().to_rfc3339();
        let updated = iam_oauth2_task_grant::Entity::update_many()
            .col_expr(iam_oauth2_task_grant::Column::Revoked, Expr::value(true))
            .col_expr(iam_oauth2_task_grant::Column::RevokedBy, Expr::value(Some(ctx.owner.clone())))
            .col_expr(iam_oauth2_task_grant::Column::RevokedTime, Expr::value(Some(now)))
            .filter(iam_oauth2_task_grant::Column::Id.eq(grant.id.clone()))
            .filter(iam_oauth2_task_grant::Column::Revoked.eq(false))
            .exec(funs.db().raw_tx()?)
            .await?;
        if updated.rows_affected == 0 {
            let current = Self::find_by_id(&grant.id, funs).await?.ok_or_else(authorization_required)?;
            if !current.revoked {
                return Err(authorization_required());
            }
            return Ok(());
        }
        rbum_event_helper::add_notify_event("iam_oauth2_task_grant", "u", &grant.id, ctx).await?;
        IamLogClient::add_ctx_task(
            LogParamTag::Token,
            Some(grant.id.clone()),
            format!("撤销定时任务 OAuth 授权 task_id={} grant_id={}", grant.task_id, grant.id),
            Some("OAuthTaskGrantRevoke".to_string()),
            ctx,
        )
        .await?;
        Ok(())
    }
}

#[cfg(test)]
mod task_grant_tests {
    use super::*;

    fn grant(account_id: &str) -> iam_oauth2_task_grant::Model {
        iam_oauth2_task_grant::Model {
            id: "grant-current".into(),
            task_id: "binding/item".into(),
            account_id: account_id.into(),
            app_id: "app".into(),
            provider_grant_id: String::new(),
            executor_account_id: "scheduler".into(),
            revoked: false,
            create_time: String::new(),
            own_paths: "tenant/app".into(),
            current_own_paths: Some("tenant/app".into()),
            current_task_id: Some("binding/item".into()),
            revoked_by: None,
            revoked_time: None,
            replaced_by_grant_id: None,
        }
    }

    #[test]
    fn first_create_and_replacement_require_the_expected_current_id() {
        assert!(validate_expected_grant(None, false, "alice", None, false).is_ok());
        assert_eq!(
            validate_expected_grant(Some("stale"), false, "alice", None, false).unwrap_err().code,
            "409-iam-oauth-task-grant-conflict"
        );
        let current = grant("alice");
        assert_eq!(
            validate_expected_grant(None, false, "alice", Some(&current), false).unwrap_err().code,
            "409-iam-oauth-task-grant-conflict"
        );
        assert_eq!(
            validate_expected_grant(Some("stale"), false, "alice", Some(&current), false).unwrap_err().code,
            "409-iam-oauth-task-grant-conflict"
        );
        assert!(validate_expected_grant(Some("grant-current"), false, "alice", Some(&current), false).is_ok());
    }

    #[test]
    fn another_owner_requires_explicit_takeover_and_persisted_admin_result() {
        let current = grant("alice");
        assert_eq!(
            validate_expected_grant(Some("grant-current"), false, "bob", Some(&current), true).unwrap_err().code,
            "403-iam-oauth-task-takeover-forbidden"
        );
        assert_eq!(
            validate_expected_grant(Some("grant-current"), true, "bob", Some(&current), false).unwrap_err().code,
            "403-iam-oauth-task-takeover-forbidden"
        );
        assert!(validate_expected_grant(Some("grant-current"), true, "bob", Some(&current), true).is_ok());
    }

    #[test]
    fn exchange_requires_the_configured_executor_and_live_grant() {
        let current = grant("alice");
        assert!(validate_exchange(&current, "scheduler", "scheduler", "tenant/app", "binding/item").is_ok());
        assert!(validate_exchange(&current, "bob", "scheduler", "tenant/app", "binding/item").is_err());
        assert!(validate_exchange(&current, "scheduler", "new-scheduler", "tenant/app", "binding/item").is_err());
        assert!(validate_exchange(&current, "scheduler", "scheduler", "other/app", "binding/item").is_err());
        assert!(validate_exchange(&current, "scheduler", "scheduler", "tenant/app", "another/item").is_err());
        let mut revoked = current;
        revoked.revoked = true;
        assert!(validate_exchange(&revoked, "scheduler", "scheduler", "tenant/app", "binding/item").is_err());
    }
}
