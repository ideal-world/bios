use std::default::Default;
use std::str::FromStr;

use bios_basic::helper::request_helper::{add_ip, get_real_ip_from_ctx};
use bios_basic::process::task_processor::TaskProcessor;
use bios_basic::rbum::rbum_config::RbumConfigApi;
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tardis::basic::dto::TardisContext;
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::chrono::Utc;
use tardis::{log, TardisFuns, TardisFunsInst};

use std::collections::HashMap;

use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::serv::rbum_item_serv::{RbumItemCrudOperation, RbumItemServ};

use crate::basic::dto::iam_account_dto::IamAccountInfoResp;
use crate::basic::dto::iam_cert_dto::IamContextFetchReq;
use crate::basic::dto::iam_filer_dto::{IamAccountFilterReq, IamAppFilterReq};
use crate::basic::serv::clients::iam_log_client::{IamLogClient, LogParamTag};
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_app_serv::IamAppServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_rel_serv::IamRelServ;
use crate::iam_config::IamConfig;
use crate::iam_constants::{self, IAM_AVATAR, IAM_OAUTH2_TOKEN_META_CACHE_KEY_PREFIX};
use crate::iam_enumeration::{IamCertTokenKind, IamRelKind};
#[derive(Serialize)]
struct IamCacheExtraRoleInfoValue {
    own_paths: String,
    role_id: String,
}

pub struct IamIdentCacheServ;

impl IamIdentCacheServ {
    pub async fn add_token(
        token: &str,
        token_kind: &IamCertTokenKind,
        rel_iam_item_id: &str,
        tenant_id: Option<&str>,
        renewal_expire_sec: Option<i64>,
        expire_sec: i64,
        coexist_num: i16,
        funs: &TardisFunsInst,
    ) -> TardisResult<()> {
        // 值为 `{token_kind},{account_id},{expire},{tenant_id}`，后两段允许为空。
        // expire 为空表示不续期；tenant_id 段存在且为空表示绑定到平台上下文。
        let token_value = iam_constants::build_token_info(&token_kind.to_string(), rel_iam_item_id, renewal_expire_sec, tenant_id);
        log::trace!("add token for account: {}", rel_iam_item_id);
        if expire_sec > 0 {
            funs.cache()
                .set_ex(
                    format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, token).as_str(),
                    token_value.as_str(),
                    expire_sec as u64,
                )
                .await?;
        } else {
            funs.cache().set(format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, token).as_str(), token_value.as_str()).await?;
        }
        // 短期任务凭据不属于交互登录会话；避免会话数量限制和过期 hash 堆积。
        if *token_kind == IamCertTokenKind::TokenTask {
            return Ok(());
        }
        funs.cache()
            .hset(
                format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, rel_iam_item_id).as_str(),
                token,
                &format!("{},{}", token_kind, Utc::now().timestamp_nanos_opt().expect("maybe in 23rd centery")),
            )
            .await?;
        // Remove old tokens
        if coexist_num != 0 {
            let old_tokens = funs.cache().hgetall(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, rel_iam_item_id).as_str()).await?;
            let old_tokens = old_tokens
                .into_iter()
                .map(|(k, v)| {
                    (
                        k,
                        v.split(',').next().unwrap_or("").to_string(),
                        i64::from_str(v.split(',').nth(1).unwrap_or("")).unwrap_or(0),
                    )
                })
                .filter(|(_, kind, _)| kind == token_kind.to_string().as_str())
                .sorted_by(|(_, _, t1), (_, _, t2)| t2.cmp(t1))
                .skip(coexist_num as usize)
                .map(|(token, _, _)| token)
                .collect::<Vec<String>>();
            for old_token in old_tokens {
                Self::delete_token_by_token(&old_token, None, funs).await?;
            }
        }
        Ok(())
    }

    /// 将 token 的租户绑定重新指向目标租户（切换租户时只影响当前 token 所属会话）。
    ///
    /// 直接改写 `token:info` 值的第 4 段，保持原 TTL 与 expire 不变。
    pub async fn rebind_token_tenant(token: &str, tenant_id: &str, funs: &TardisFunsInst) -> TardisResult<()> {
        let token_key = format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, token);
        let Some(token_info) = funs.cache().get(&token_key).await? else {
            // token 已失效，无需重绑
            return Ok(());
        };
        let parts = iam_constants::parse_token_info(&token_info);
        let new_value = iam_constants::build_token_info(parts.token_kind, parts.account_id, parts.renewal_expire_sec, Some(tenant_id));
        let ttl = funs.cache().ttl(&token_key).await?;
        // Redis TTL: 大于 0 表示剩余有效期；-1（无过期）/ -2（不存在）转成 usize 后为极大值
        const MAX_REASONABLE_TTL_SEC: usize = 60 * 60 * 24 * 365 * 100;
        if ttl > 0 && ttl < MAX_REASONABLE_TTL_SEC {
            funs.cache().set_ex(&token_key, &new_value, ttl as u64).await?;
        } else {
            funs.cache().set(&token_key, &new_value).await?;
        }
        Ok(())
    }

    pub async fn delete_token_by_token(token: &str, ip: Option<String>, funs: &TardisFunsInst) -> TardisResult<()> {
        log::trace!("delete token: token={}", token);
        if let Some(token_info) = funs.cache().get(format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, token).as_str()).await? {
            let token_parts = iam_constants::parse_token_info(&token_info);
            let iam_item_id = token_parts.account_id;
            let token_tenant = token_parts.tenant_id;
            funs.cache().del(format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, token).as_str()).await?;
            funs.cache().del(format!("{}{}", IAM_OAUTH2_TOKEN_META_CACHE_KEY_PREFIX, token).as_str()).await?;
            Self::delete_double_auth(iam_item_id, funs).await?;
            funs.cache().hdel(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, iam_item_id).as_str(), token).await?;

            let mut mock_ctx = TardisContext::default();
            let account_context = match token_tenant {
                Some(tenant_id) => Self::get_account_context_with_tenant(iam_item_id, tenant_id, "", funs).await,
                None => Self::get_account_context(iam_item_id, "", funs).await,
            };
            if let Ok(account_context) = account_context {
                mock_ctx = account_context;
            } else {
                mock_ctx.owner = iam_item_id.to_string();
                let own_paths = RbumItemServ::get_rbum(
                    iam_item_id,
                    &RbumBasicFilterReq {
                        ignore_scope: true,
                        with_sub_own_paths: true,
                        own_paths: Some("".to_string()),
                        ..Default::default()
                    },
                    funs,
                    &mock_ctx,
                )
                .await?
                .own_paths;
                mock_ctx.own_paths = own_paths;
            }
            add_ip(ip, &mock_ctx).await?;

            let _ = IamLogClient::add_ctx_task(
                LogParamTag::IamAccount,
                Some(iam_item_id.to_string()),
                "下线账号".to_string(),
                Some("OfflineAccount".to_string()),
                &mock_ctx,
            )
            .await;
            let _ = IamLogClient::add_ctx_task(
                LogParamTag::SecurityVisit,
                Some(iam_item_id.to_string()),
                "退出".to_string(),
                Some("Quit".to_string()),
                &mock_ctx,
            )
            .await;

            mock_ctx.execute_task().await?;
        } else {
            funs.cache().del(format!("{}{}", IAM_OAUTH2_TOKEN_META_CACHE_KEY_PREFIX, token).as_str()).await?;
        }
        Ok(())
    }

    pub async fn delete_tokens_and_contexts_by_tenant_or_app(tenant_or_app_id: &str, is_app: bool, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let tenant_or_app_id = tenant_or_app_id.to_string();
        let own_paths = if is_app {
            IamAppServ::peek_item(
                &tenant_or_app_id,
                &IamAppFilterReq {
                    basic: RbumBasicFilterReq {
                        with_sub_own_paths: true,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?
            .own_paths
        } else {
            tenant_or_app_id.clone()
        };
        let ctx_clone = ctx.clone();
        let target_tenant_id = if is_app {
            own_paths.split('/').next().unwrap_or("").to_string()
        } else {
            own_paths.clone()
        };
        TaskProcessor::execute_task_with_ctx(
            &funs.conf::<IamConfig>().cache_key_async_task_status,
            move |_task_id| async move {
                let funs = iam_constants::get_tardis_inst();
                let filter = IamAccountFilterReq {
                    basic: RbumBasicFilterReq {
                        own_paths: Some(own_paths),
                        with_sub_own_paths: true,
                        ..Default::default()
                    },
                    ..Default::default()
                };
                let mut count = IamAccountServ::count_items(&filter, &funs, &ctx_clone).await.unwrap_or_default() as isize;
                let mut page_number = 1;
                while count > 0 {
                    let mut ids = Vec::new();
                    if let Ok(page) = IamAccountServ::paginate_id_items(&filter, page_number, 100, None, None, &funs, &ctx_clone).await {
                        ids = page.records;
                    }
                    for id in ids {
                        Self::delete_tokens_by_tenant(&id, &target_tenant_id, get_real_ip_from_ctx(&ctx_clone).await?, &funs).await?;
                    }
                    page_number += 1;
                    count -= 100;
                }
                if is_app {
                    let mut count = IamRelServ::count_to_rels(&IamRelKind::IamAccountApp, &tenant_or_app_id, &funs, &ctx_clone).await.unwrap_or_default() as isize;
                    let mut page_number = 1;
                    while count > 0 {
                        let mut ids = Vec::new();
                        if let Ok(page) = IamRelServ::paginate_to_id_rels(&IamRelKind::IamAccountApp, &tenant_or_app_id, page_number, 100, None, None, &funs, &ctx_clone).await {
                            ids = page.records;
                        }
                        for id in ids {
                            Self::delete_tokens_by_tenant(&id, &target_tenant_id, get_real_ip_from_ctx(&ctx_clone).await?, &funs).await?;
                        }
                        page_number += 1;
                        count -= 100;
                    }
                }
                Ok(())
            },
            &funs.cache(),
            IAM_AVATAR.to_owned(),
            Some(vec![format!("account/{}", ctx.owner)]),
            ctx,
        )
        .await?;
        Ok(())
    }

    pub async fn refresh_tokens_and_contexts_by_tenant_or_app(tenant_or_app_id: &str, is_app: bool, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let tenant_or_app_id = tenant_or_app_id.to_string();
        let own_paths = if is_app {
            IamAppServ::peek_item(
                &tenant_or_app_id,
                &IamAppFilterReq {
                    basic: RbumBasicFilterReq {
                        with_sub_own_paths: true,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?
            .own_paths
        } else {
            tenant_or_app_id.clone()
        };
        let ctx_clone = ctx.clone();
        let target_tenant_id = if is_app {
            own_paths.split('/').next().unwrap_or("").to_string()
        } else {
            own_paths.clone()
        };
        TaskProcessor::execute_task_with_ctx(
            &funs.conf::<IamConfig>().cache_key_async_task_status,
            move |_task_id| async move {
                let funs = iam_constants::get_tardis_inst();
                let filter = IamAccountFilterReq {
                    basic: RbumBasicFilterReq {
                        own_paths: Some(own_paths),
                        with_sub_own_paths: true,
                        ..Default::default()
                    },
                    ..Default::default()
                };
                let mut count = IamAccountServ::count_items(&filter, &funs, &ctx_clone).await.unwrap_or_default() as isize;
                let mut page_number = 1;
                while count > 0 {
                    let mut ids = Vec::new();
                    if let Ok(page) = IamAccountServ::paginate_id_items(&filter, page_number, 100, None, None, &funs, &ctx_clone).await {
                        ids = page.records;
                    }
                    for id in ids {
                        Self::refresh_account_info_by_account_id(&id, &target_tenant_id, &funs).await?;
                    }
                    page_number += 1;
                    count -= 100;
                }
                if is_app {
                    let mut count = IamRelServ::count_to_rels(&IamRelKind::IamAccountApp, &tenant_or_app_id, &funs, &ctx_clone).await.unwrap_or_default() as isize;
                    let mut page_number = 1;
                    while count > 0 {
                        let mut ids = Vec::new();
                        if let Ok(page) = IamRelServ::paginate_to_id_rels(&IamRelKind::IamAccountApp, &tenant_or_app_id, page_number, 100, None, None, &funs, &ctx_clone).await {
                            ids = page.records;
                        }
                        for id in ids {
                            Self::refresh_account_info_by_account_id(&id, &target_tenant_id, &funs).await?;
                        }
                        page_number += 1;
                        count -= 100;
                    }
                }
                Ok(())
            },
            &funs.cache(),
            IAM_AVATAR.to_owned(),
            Some(vec![format!("account/{}", ctx.owner)]),
            ctx,
        )
        .await?;
        Ok(())
    }

    pub async fn delete_tokens_and_contexts_by_account_id(account_id: &str, ip: Option<String>, funs: &TardisFunsInst) -> TardisResult<()> {
        log::trace!("delete tokens and contexts: account_id={}", account_id);
        let tokens = funs.cache().hgetall(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, account_id).as_str()).await?;
        for (token, _) in tokens.iter() {
            funs.cache().del(format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, token).as_str()).await?;
            funs.cache().del(format!("{}{}", IAM_OAUTH2_TOKEN_META_CACHE_KEY_PREFIX, token).as_str()).await?;
        }
        funs.cache().del(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, account_id).as_str()).await?;
        funs.cache().del(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_info_, account_id).as_str()).await?;
        // 临时产品授权没有下次登录的重建来源，不在下线时删除，只能由 CI 回收。
        // 临时产品授权不在下线时删除。账号上下文下次登录会重建，临时产品记录没有重建来源，只能由 CI 回收。

        let mock_ctx = TardisContext { ..Default::default() };
        add_ip(ip, &mock_ctx).await?;
        let _ = IamLogClient::add_ctx_task(
            LogParamTag::IamAccount,
            Some(account_id.to_string()),
            "下线账号".to_string(),
            Some("OfflineAccount".to_string()),
            &mock_ctx,
        )
        .await;

        mock_ctx.execute_task().await?;

        Ok(())
    }

    /// 只清理绑定到指定租户的会话，避免同账号其他租户的会话被误下线。
    async fn delete_tokens_by_tenant(account_id: &str, tenant_id: &str, ip: Option<String>, funs: &TardisFunsInst) -> TardisResult<()> {
        let tokens = funs.cache().hgetall(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, account_id).as_str()).await?;
        for (token, _) in tokens.iter() {
            let bound_tenant = funs
                .cache()
                .get(format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, token).as_str())
                .await?
                .map(|token_info| iam_constants::parse_token_info(&token_info).tenant_id.map(str::to_string))
                .unwrap_or_default();
            let matched = match bound_tenant {
                Some(bound_tenant) => bound_tenant == tenant_id,
                // 旧 token 没有租户绑定，沿用旧的账号级上下文判断
                None => Self::get_account_context(account_id, "", funs).await.map(|context| context.own_paths == tenant_id).unwrap_or(false),
            };
            if matched {
                Self::delete_token_by_token(token, ip.clone(), funs).await?;
            }
        }
        // token 已失效，同时清掉该租户的上下文缓存，避免下次登录前仍能读到旧权限
        Self::delete_contexts_by_tenant(account_id, tenant_id, funs).await?;
        Ok(())
    }

    /// 删除账号在指定租户下的上下文分片，其他租户的分片保持不变。
    async fn delete_contexts_by_tenant(account_id: &str, tenant_id: &str, funs: &TardisFunsInst) -> TardisResult<()> {
        let cache_key = format!("{}{}", funs.conf::<IamConfig>().cache_key_account_info_, account_id);
        if !funs.cache().exists(&cache_key).await? {
            return Ok(());
        }
        funs.cache().hdel(&cache_key, &iam_constants::account_tenant_context_field(tenant_id)).await?;
        let app_field_prefix = format!("app:{tenant_id}:");
        for field in funs.cache().hkeys(&cache_key).await? {
            if let Some(app_id) = field.strip_prefix(&app_field_prefix) {
                funs.cache().hdel(&cache_key, &field).await?;
                // 同步清理过渡期的旧应用字段
                funs.cache().hdel(&cache_key, app_id).await?;
            }
        }
        // 旧租户字段确实属于该租户时才清理，避免误删其他租户的兼容数据
        if let Some(legacy_context) = funs.cache().hget(&cache_key, "").await? {
            if TardisFuns::json.str_to_obj::<TardisContext>(&legacy_context).map(|context| context.own_paths == tenant_id).unwrap_or(false) {
                funs.cache().hdel(&cache_key, "").await?;
            }
        }
        // is_global 是纯缓存属性，随上下文一起清除，避免保留判断依据
        funs.cache().hdel(&cache_key, "is_global").await?;
        Ok(())
    }

    pub async fn exist_token_by_account_id(account_id: &str, funs: &TardisFunsInst) -> TardisResult<bool> {
        log::trace!("exist tokens: account_id={}", account_id);
        let tokens = funs.cache().hgetall(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, account_id).as_str()).await?;
        for (token, _) in tokens.iter() {
            if funs.cache().exists(format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, token).as_str()).await? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub async fn refresh_account_info_by_account_id(account_id: &str, tenant_id: &str, funs: &TardisFunsInst) -> TardisResult<()> {
        log::trace!("refresh account info: account_id={}, tenant_id={}", account_id, tenant_id);
        let cache_key = format!("{}{}", funs.conf::<IamConfig>().cache_key_account_info_, account_id);
        let tenant_field = iam_constants::account_tenant_context_field(tenant_id);
        let mut should_refresh = funs.cache().hget(&cache_key, &tenant_field).await?.is_some();
        // 迁移期：旧版本只写了账号级字段，若其租户正好一致则顺带升级为租户字段
        if !should_refresh {
            should_refresh = funs
                .cache()
                .hget(&cache_key, "")
                .await?
                .and_then(|legacy| TardisFuns::json.str_to_obj::<TardisContext>(&legacy).ok())
                .map(|legacy| legacy.own_paths == tenant_id)
                .unwrap_or(false);
        }
        if !should_refresh {
            return Ok(());
        }
        let mock_ctx = TardisContext {
            own_paths: tenant_id.to_string(),
            ..Default::default()
        };
        let _ = IamCertServ::package_tardis_account_context_and_resp(account_id, tenant_id, "".to_string(), None, funs, &mock_ctx).await;
        Ok(())
    }

    pub async fn delete_lock_by_account_id(account_id: &str, funs: &TardisFunsInst) -> TardisResult<()> {
        log::trace!("delete lock: account_id={}", account_id);
        funs.cache().del(&format!("{}{}", funs.rbum_conf_cache_key_cert_locked_(), &account_id)).await?;
        Ok(())
    }

    pub async fn add_contexts(account_info: &IamAccountInfoResp, tenant_id: &str, funs: &TardisFunsInst) -> TardisResult<()> {
        log::trace!("add contexts: account_id={:?}, tenant_id={}", account_info, tenant_id);
        let cache_key = format!("{}{}", funs.conf::<IamConfig>().cache_key_account_info_, account_info.account_id);
        let tenant_context = TardisFuns::json.obj_to_string(&TardisContext {
            own_paths: tenant_id.to_string(),
            owner: account_info.account_id.to_string(),
            roles: account_info.roles.keys().map(|id| id.to_string()).collect(),
            groups: account_info.groups.keys().map(|id| id.to_string()).collect(),
            ..Default::default()
        })?;
        // 租户维度分片，token 通过绑定关系精确读取自己所属租户的上下文
        funs.cache().hset(&cache_key, &iam_constants::account_tenant_context_field(tenant_id), &tenant_context).await?;
        // 过渡期保留旧字段，兼容滚动发布期间的旧实例和旧 token
        funs.cache().hset(&cache_key, "", &tenant_context).await?;

        let mut current_app_fields = Vec::with_capacity(account_info.apps.len());
        for account_app_info in &account_info.apps {
            let app_context = TardisFuns::json.obj_to_string(&TardisContext {
                own_paths: if tenant_id.is_empty() {
                    account_app_info.app_own_paths.clone()
                } else {
                    format!("{}/{}", tenant_id, account_app_info.app_id)
                },
                owner: account_info.account_id.to_string(),
                roles: account_app_info.roles.keys().map(|id| id.to_string()).collect(),
                groups: account_app_info.groups.keys().map(|id| id.to_string()).collect(),
                ..Default::default()
            })?;
            let app_field = iam_constants::account_app_context_field(tenant_id, &account_app_info.app_id);
            funs.cache().hset(&cache_key, &app_field, &app_context).await?;
            // 旧字段只用于兼容旧 token
            funs.cache().hset(&cache_key, &account_app_info.app_id, &app_context).await?;
            current_app_fields.push(app_field);
        }

        // 清理本租户下已失效的应用上下文，避免取消应用授权后仍命中旧权限
        let app_field_prefix = format!("app:{tenant_id}:");
        for field in funs.cache().hkeys(&cache_key).await? {
            if field.starts_with(&app_field_prefix) && !current_app_fields.contains(&field) {
                funs.cache().hdel(&cache_key, &field).await?;
            }
        }
        Ok(())
    }

    pub async fn get_account_context(account_id: &str, field: &str, funs: &TardisFunsInst) -> TardisResult<TardisContext> {
        Self::get_account_context_inner(account_id, None, field, funs).await
    }

    /// 按 (account_id, tenant_id) 读取账号上下文。应用上下文由 `field` 指定应用 ID，传空串表示租户上下文。
    pub async fn get_account_context_with_tenant(account_id: &str, tenant_id: &str, field: &str, funs: &TardisFunsInst) -> TardisResult<TardisContext> {
        Self::get_account_context_inner(account_id, Some(tenant_id), field, funs).await
    }

    async fn get_account_context_inner(account_id: &str, tenant_id: Option<&str>, field: &str, funs: &TardisFunsInst) -> TardisResult<TardisContext> {
        let cache_key = format!("{}{}", funs.conf::<IamConfig>().cache_key_account_info_, account_id);
        let context_field = match tenant_id {
            Some(tenant_id) if field.is_empty() => iam_constants::account_tenant_context_field(tenant_id),
            Some(tenant_id) => iam_constants::account_app_context_field(tenant_id, field),
            None => field.to_string(),
        };
        let mut context = if let Some(context) = funs.cache().hget(&cache_key, &context_field).await? {
            TardisFuns::json.str_to_obj::<TardisContext>(&context)?
        } else {
            return Err(funs.err().not_found("get_account_context", "get", "not found context", "404-iam-cache-context-not-exist"));
        };
        if !field.is_empty() {
            let tenant_field = match tenant_id {
                Some(tenant_id) => iam_constants::account_tenant_context_field(tenant_id),
                None => "".to_string(),
            };
            if let Some(tenant_context) = funs.cache().hget(&cache_key, &tenant_field).await? {
                let tenant_context = TardisFuns::json.str_to_obj::<TardisContext>(&tenant_context)?;
                if !tenant_context.roles.is_empty() {
                    context.roles.extend(tenant_context.roles);
                }
                if !tenant_context.groups.is_empty() {
                    context.groups.extend(tenant_context.groups);
                }
            }
        }
        Ok(context)
    }

    pub async fn get_context(fetch_req: &IamContextFetchReq, funs: &TardisFunsInst) -> TardisResult<TardisContext> {
        if let Some(token_info) = funs.cache().get(format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, &fetch_req.token).as_str()).await? {
            let token_parts = iam_constants::parse_token_info(&token_info);
            let account_id = token_parts.account_id;
            if token_parts.token_kind == "TokenTask" {
                let encoded = funs
                    .cache()
                    .get(&format!("iam:task-token:{}", fetch_req.token))
                    .await?
                    .ok_or_else(|| TardisError::unauthorized("task token expired", "401-iam-task-token-expired"))?;
                let ctx: TardisContext = TardisFuns::json.str_to_obj(&encoded)?;
                if ctx.owner != account_id || fetch_req.app_id.as_deref() != ctx.own_paths.split('/').nth(1) {
                    return Err(TardisError::forbidden("task token application mismatch", "403-iam-task-token-scope"));
                }
                return Ok(ctx);
            }
            let app_id = fetch_req.app_id.as_deref().unwrap_or("");
            let context_field = match token_parts.tenant_id {
                Some(tenant_id) if app_id.is_empty() => iam_constants::account_tenant_context_field(tenant_id),
                Some(tenant_id) => iam_constants::account_app_context_field(tenant_id, app_id),
                None => app_id.to_string(),
            };
            if let Some(context) = funs.cache().hget(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_info_, account_id).as_str(), &context_field).await? {
                return TardisFuns::json.str_to_obj(&context);
            }
        }
        Err(funs.err().not_found("iam_cache_context", "get", "not found context", "404-iam-cache-context-not-exist"))
    }

    pub async fn add_aksk(ak: &str, sk: &str, tenant_id: &str, app_id: Option<String>, expire_sec: i64, funs: &TardisFunsInst) -> TardisResult<()> {
        log::trace!("add aksk: ak={},sk={}", ak, sk);
        if expire_sec > 0 {
            funs.cache()
                .set_ex(
                    format!("{}{}", funs.conf::<IamConfig>().cache_key_aksk_info_, ak).as_str(),
                    format!("{},{},{}", sk, tenant_id, app_id.unwrap_or_default()).as_str(),
                    expire_sec as u64,
                )
                .await?;
        } else {
            funs.cache()
                .set(
                    format!("{}{}", funs.conf::<IamConfig>().cache_key_aksk_info_, ak).as_str(),
                    format!("{},{},{}", sk, tenant_id, app_id.unwrap_or_default()).as_str(),
                )
                .await?;
        }
        Ok(())
    }

    pub async fn add_or_modify_gateway_rule_info(ak: &str, rule_name: &str, match_method: Option<&str>, value: &str, funs: &TardisFunsInst) -> TardisResult<()> {
        log::trace!(
            "add gateway_rule_info: ak={},rule_name={},match_method={},value={}",
            ak,
            rule_name,
            match_method.unwrap_or("*"),
            value
        );
        let match_path = &funs.conf::<IamConfig>().gateway_openapi_path;
        funs.cache()
            .set(
                format!(
                    "{}{}:{}:{}:{}",
                    funs.conf::<IamConfig>().cache_key_gateway_rule_info_,
                    rule_name,
                    match_method.unwrap_or("*"),
                    match_path,
                    ak
                )
                .as_str(),
                value,
            )
            .await?;
        Ok(())
    }

    // 设置开放平台api调用频率
    pub async fn set_open_api_call_frequency(spec_id: &str, match_method: Option<&str>, value: &str, funs: &TardisFunsInst) -> TardisResult<()> {
        log::trace!("add gateway_rule_info: spec_id={},match_method={},value={}", spec_id, match_method.unwrap_or("*"), value);
        let match_path = &funs.conf::<IamConfig>().gateway_openapi_path;
        funs.cache()
            .set(
                format!(
                    "{}{}:{}:{}:{}",
                    funs.conf::<IamConfig>().cache_key_gateway_rule_info_,
                    funs.conf::<IamConfig>().openapi_plugin_limit,
                    match_method.unwrap_or("*"),
                    match_path,
                    spec_id
                )
                .as_str(),
                value,
            )
            .await?;
        Ok(())
    }

    // 删除开放平台api调用频率
    pub async fn delete_open_api_call_frequency(spec_id: &str, match_method: Option<&str>, funs: &TardisFunsInst) -> TardisResult<()> {
        log::trace!("add gateway_rule_info: spec_id={},match_method={},", spec_id, match_method.unwrap_or("*"));
        let match_path = &funs.conf::<IamConfig>().gateway_openapi_path;
        funs.cache()
            .del(
                format!(
                    "{}{}:{}:{}:{}",
                    funs.conf::<IamConfig>().cache_key_gateway_rule_info_,
                    funs.conf::<IamConfig>().openapi_plugin_limit,
                    match_method.unwrap_or("*"),
                    match_path,
                    spec_id
                )
                .as_str(),
            )
            .await?;
        Ok(())
    }

    // 设置开放API扩展头信息
    pub async fn set_open_api_extand_header(ak: &str, match_method: Option<&str>, value: HashMap<String, String>, funs: &TardisFunsInst) -> TardisResult<()> {
        log::trace!("set_open_api_extand_header: ak={},match_method={},value={:?}", ak, match_method.unwrap_or("*"), value);
        let match_path = &funs.conf::<IamConfig>().gateway_openapi_path;
        let key = format!(
            "{}{}:{}:{}:{}",
            funs.conf::<IamConfig>().cache_key_gateway_rule_info_,
            funs.conf::<IamConfig>().openapi_plugin_extand_header,
            match_method.unwrap_or("*"),
            match_path,
            ak
        );
        let mut new_header: HashMap<String, String> = HashMap::new();
        if let Some(original_header) = funs.cache().get(key.as_str()).await? {
            new_header.extend(tardis::TardisFuns::json.str_to_obj::<HashMap<String, String>>(&original_header)?);
        }
        new_header.extend(value);
        funs.cache().set(key.as_str(), &tardis::TardisFuns::json.obj_to_string(&new_header)?).await?;
        Ok(())
    }

    pub async fn get_open_api_extand_header(ak: &str, match_method: Option<&str>, funs: &TardisFunsInst) -> TardisResult<Option<HashMap<String, String>>> {
        let match_path = &funs.conf::<IamConfig>().gateway_openapi_path;
        let result = funs
            .cache()
            .get(&format!(
                "{}{}:{}:{}:{}",
                funs.conf::<IamConfig>().cache_key_gateway_rule_info_,
                funs.conf::<IamConfig>().openapi_plugin_extand_header,
                match_method.unwrap_or("*"),
                match_path,
                ak
            ))
            .await?
            .map(|s| tardis::TardisFuns::json.str_to_obj::<HashMap<String, String>>(&s).unwrap_or_default());
        Ok(result)
    }

    pub async fn add_or_modify_bind_api_res(res_id: &str, match_method: Option<&str>, bind_api_res: Vec<String>, funs: &TardisFunsInst) -> TardisResult<()> {
        log::trace!("add bind_api_res: spec_id={},bind_api_res={:?}", res_id, bind_api_res,);
        let match_path = &funs.conf::<IamConfig>().gateway_openapi_path;
        let cache_key = format!(
            "{}{}:res:{}:{}:{}",
            funs.conf::<IamConfig>().cache_key_gateway_rule_info_,
            funs.conf::<IamConfig>().openapi_plugin_allow_api_res,
            match_method.unwrap_or("*"),
            match_path,
            res_id,
        );
        funs.cache().del(cache_key.as_str()).await?;

        funs.cache().set(cache_key.as_str(), &tardis::TardisFuns::json.obj_to_string(&bind_api_res)?).await?;
        Ok(())
    }

    pub async fn get_bind_api_res(res_id: &str, match_method: Option<&str>, funs: &TardisFunsInst) -> TardisResult<Option<Vec<String>>> {
        let match_path = &funs.conf::<IamConfig>().gateway_openapi_path;
        let result = funs
            .cache()
            .get(&format!(
                "{}{}:res:{}:{}:{}",
                funs.conf::<IamConfig>().cache_key_gateway_rule_info_,
                funs.conf::<IamConfig>().openapi_plugin_allow_api_res,
                match_method.unwrap_or("*"),
                match_path,
                res_id,
            ))
            .await?;
        Ok(result.map(|s| tardis::TardisFuns::json.str_to_obj::<Vec<String>>(&s).unwrap_or_default()))
    }

    pub async fn add_or_modify_ak_bind_api_res(ak: &str, match_method: Option<&str>, bind_api_res: Vec<String>, funs: &TardisFunsInst) -> TardisResult<()> {
        log::trace!("add bind_api_res: spec_id={},bind_api_res={:?}", ak, bind_api_res,);
        let match_path = &funs.conf::<IamConfig>().gateway_openapi_path;
        let cache_key = format!(
            "{}{}:{}:{}:{}",
            funs.conf::<IamConfig>().cache_key_gateway_rule_info_,
            funs.conf::<IamConfig>().openapi_plugin_allow_api_res,
            match_method.unwrap_or("*"),
            match_path,
            ak,
        );
        funs.cache().del(cache_key.as_str()).await?;

        funs.cache().set(cache_key.as_str(), &tardis::TardisFuns::json.obj_to_string(&bind_api_res)?).await?;
        Ok(())
    }

    pub async fn get_ak_bind_api_res(res_id: &str, match_method: Option<&str>, funs: &TardisFunsInst) -> TardisResult<Option<Vec<String>>> {
        let match_path = &funs.conf::<IamConfig>().gateway_openapi_path;
        let result = funs
            .cache()
            .get(&format!(
                "{}{}:{}:{}:{}",
                funs.conf::<IamConfig>().cache_key_gateway_rule_info_,
                funs.conf::<IamConfig>().openapi_plugin_allow_api_res,
                match_method.unwrap_or("*"),
                match_path,
                res_id,
            ))
            .await?;
        Ok(result.map(|s| tardis::TardisFuns::json.str_to_obj::<Vec<String>>(&s).unwrap_or_default()))
    }

    pub async fn get_gateway_cumulative_count(ak: &str, match_method: Option<&str>, funs: &TardisFunsInst) -> TardisResult<Option<String>> {
        let match_path = &funs.conf::<IamConfig>().gateway_openapi_path;
        let result = funs
            .cache()
            .get(&format!(
                "{}{}:{}:{}:{}:cumulative-count",
                funs.conf::<IamConfig>().cache_key_gateway_rule_info_,
                funs.conf::<IamConfig>().openapi_plugin_count,
                match_method.unwrap_or("*"),
                match_path,
                ak
            ))
            .await?;
        Ok(result)
    }

    pub async fn get_gateway_rule_info(ak: &str, rule_name: &str, match_method: Option<&str>, funs: &TardisFunsInst) -> TardisResult<Option<String>> {
        let match_path = &funs.conf::<IamConfig>().gateway_openapi_path;
        let result = funs
            .cache()
            .get(
                format!(
                    "{}{}:{}:{}:{}",
                    funs.conf::<IamConfig>().cache_key_gateway_rule_info_,
                    rule_name,
                    match_method.unwrap_or("*"),
                    match_path,
                    ak
                )
                .as_str(),
            )
            .await?;
        Ok(result)
    }

    pub async fn delete_aksk(ak: &str, match_method: Option<&str>, funs: &TardisFunsInst) -> TardisResult<()> {
        log::trace!("delete aksk: ak={}", ak);

        funs.cache().del(format!("{}{}", funs.conf::<IamConfig>().cache_key_aksk_info_, ak).as_str()).await?;

        let match_path = &funs.conf::<IamConfig>().gateway_openapi_path;
        funs.cache()
            .del(
                format!(
                    "{}{}:{}:{}:{}",
                    funs.conf::<IamConfig>().cache_key_gateway_rule_info_,
                    funs.conf::<IamConfig>().openapi_plugin_state,
                    match_method.unwrap_or("*"),
                    match_path,
                    ak
                )
                .as_str(),
            )
            .await?;
        funs.cache()
            .del(
                format!(
                    "{}{}:{}:{}:{}",
                    funs.conf::<IamConfig>().cache_key_gateway_rule_info_,
                    funs.conf::<IamConfig>().openapi_plugin_time_range,
                    match_method.unwrap_or("*"),
                    match_path,
                    ak
                )
                .as_str(),
            )
            .await?;
        funs.cache()
            .del(
                format!(
                    "{}{}:{}:{}:{}",
                    funs.conf::<IamConfig>().cache_key_gateway_rule_info_,
                    funs.conf::<IamConfig>().openapi_plugin_count,
                    match_method.unwrap_or("*"),
                    match_path,
                    ak
                )
                .as_str(),
            )
            .await?;
        funs.cache()
            .del(
                format!(
                    "{}{}:{}:{}:{}",
                    funs.conf::<IamConfig>().cache_key_gateway_rule_info_,
                    funs.conf::<IamConfig>().openapi_plugin_dynamic_route,
                    match_method.unwrap_or("*"),
                    match_path,
                    ak
                )
                .as_str(),
            )
            .await?;
        funs.cache()
            .del(
                format!(
                    "{}{}:{}:{}:{}",
                    funs.conf::<IamConfig>().cache_key_gateway_rule_info_,
                    funs.conf::<IamConfig>().openapi_plugin_state,
                    match_method.unwrap_or("*"),
                    match_path,
                    ak
                )
                .as_str(),
            )
            .await?;
        funs.cache()
            .del(
                format!(
                    "{}{}:{}:{}:{}",
                    funs.conf::<IamConfig>().cache_key_gateway_rule_info_,
                    funs.conf::<IamConfig>().openapi_plugin_extand_header,
                    match_method.unwrap_or("*"),
                    match_path,
                    ak
                )
                .as_str(),
            )
            .await?;
        funs.cache()
            .del(
                format!(
                    "{}{}:{}:{}:{}:cumulative-count",
                    funs.conf::<IamConfig>().cache_key_gateway_rule_info_,
                    funs.conf::<IamConfig>().openapi_plugin_count,
                    match_method.unwrap_or("*"),
                    match_path,
                    ak
                )
                .as_str(),
            )
            .await?;

        Ok(())
    }

    pub async fn add_double_auth(account_id: &str, funs: &TardisFunsInst) -> TardisResult<()> {
        log::trace!("add double auth: account_id={}", account_id);

        funs.cache()
            .set_ex(
                format!("{}{}", funs.conf::<IamConfig>().cache_key_double_auth_info, account_id).as_str(),
                "",
                funs.conf::<IamConfig>().cache_key_double_auth_expire_sec as u64,
            )
            .await?;

        Ok(())
    }
    pub async fn delete_double_auth(account_id: &str, funs: &TardisFunsInst) -> TardisResult<()> {
        log::trace!("delete double auth: account_id={}", account_id);
        if (funs.cache().get(format!("{}{}", funs.conf::<IamConfig>().cache_key_double_auth_info, account_id).as_str()).await?).is_some() {
            funs.cache().del(format!("{}{}", funs.conf::<IamConfig>().cache_key_double_auth_info, account_id).as_str()).await?;
        }
        Ok(())
    }
    pub async fn add_extra_role_info(extra_role_id: &str, app_id: &str, own_paths: &str, app_extend_role_id: &str, funs: &TardisFunsInst) -> TardisResult<()> {
        log::trace!(
            "add extra role info: extra_role_id={}, app_id={}, own_paths={}, app_extend_role_id={}",
            extra_role_id,
            app_id,
            own_paths,
            app_extend_role_id
        );
        let value = TardisFuns::json.obj_to_string(&IamCacheExtraRoleInfoValue {
            own_paths: own_paths.to_string(),
            role_id: app_extend_role_id.to_string(),
        })?;
        funs.cache()
            .hset(
                format!("{}{}", funs.conf::<IamConfig>().cache_key_extra_role_info_, extra_role_id).as_str(),
                app_id,
                value.as_str(),
            )
            .await?;
        Ok(())
    }
    pub async fn delete_extra_role_info(extra_role_id: &str, app_id: &str, funs: &TardisFunsInst) -> TardisResult<()> {
        log::trace!("delete extra role info: extra_role_id={}, app_id={}", extra_role_id, app_id);
        funs.cache().hdel(format!("{}{}", funs.conf::<IamConfig>().cache_key_extra_role_info_, extra_role_id).as_str(), app_id).await?;
        Ok(())
    }
}

pub struct IamResCacheServ;

impl IamResCacheServ {
    pub async fn add_res(
        item_code: &str,
        action: &str,
        crypto_req: bool,
        crypto_resp: bool,
        double_auth: bool,
        only_aksk: bool,
        need_login: bool,
        funs: &TardisFunsInst,
    ) -> TardisResult<()> {
        let uri_mixed = Self::package_uri_mixed(item_code, action);
        log::trace!("add res: uri_mixed={}", uri_mixed);
        let add_res_dto = IamCacheResRelAddOrModifyDto {
            need_crypto_req: crypto_req,
            need_crypto_resp: crypto_resp,
            need_double_auth: double_auth,
            need_only_aksk: only_aksk,
            need_login,
            ..Default::default()
        };
        funs.cache().hset(&funs.conf::<IamConfig>().cache_key_res_info, &uri_mixed, &TardisFuns::json.obj_to_string(&add_res_dto)?).await?;
        Self::add_change_trigger(&uri_mixed, funs).await
    }

    pub async fn delete_res(item_code: &str, action: &str, funs: &TardisFunsInst) -> TardisResult<()> {
        let uri_mixed = Self::package_uri_mixed(item_code, action);
        log::trace!("delete res: uri_mixed={}", uri_mixed);
        funs.cache().hdel(&funs.conf::<IamConfig>().cache_key_res_info, &uri_mixed).await?;
        Self::add_change_trigger(&uri_mixed, funs).await
    }

    // add anonymous access permissions
    pub async fn add_anonymous_res_rel(item_code: &str, action: &str, st: Option<i64>, et: Option<i64>, funs: &TardisFunsInst) -> TardisResult<()> {
        let res_auth = IamCacheResAuth {
            tenants: "#*#".to_string(),
            st,
            et,
            ..Default::default()
        };
        let mut res_dto = IamCacheResRelAddOrModifyDto {
            auth: Some(res_auth),
            need_crypto_req: false,
            need_crypto_resp: false,
            need_double_auth: false,
            need_only_aksk: false,
            need_login: false,
        };
        let uri_mixed = Self::package_uri_mixed(item_code, action);
        let rels = funs.cache().hget(&funs.conf::<IamConfig>().cache_key_res_info, &uri_mixed).await?;
        if let Some(rels) = rels {
            let old_res_dto = TardisFuns::json.str_to_obj::<IamCacheResRelAddOrModifyDto>(&rels)?;
            res_dto.need_crypto_req = old_res_dto.need_crypto_req;
            res_dto.need_crypto_resp = old_res_dto.need_crypto_resp;
            res_dto.need_double_auth = old_res_dto.need_double_auth;
            res_dto.need_only_aksk = old_res_dto.need_only_aksk;
            res_dto.need_login = old_res_dto.need_login;
        }
        funs.cache().hset(&funs.conf::<IamConfig>().cache_key_res_info, &uri_mixed, &TardisFuns::json.obj_to_string(&res_dto)?).await?;
        Self::add_change_trigger(&uri_mixed, funs).await
    }

    pub async fn add_or_modify_res_rel(item_code: &str, action: &str, add_or_modify_req: &IamCacheResRelAddOrModifyReq, funs: &TardisFunsInst) -> TardisResult<()> {
        if add_or_modify_req.st.is_some() || add_or_modify_req.et.is_some() {
            // TODO supports time range
            return Err(funs.err().conflict("iam_cache_res", "add_or_modify", "st and et must be none", "409-iam-cache-res-date-not-none"));
        }
        let mut res_auth = IamCacheResAuth {
            accounts: format!("#{}#", add_or_modify_req.accounts.join("#")),
            roles: format!("#{}#", add_or_modify_req.roles.join("#")),
            groups: format!("#{}#", add_or_modify_req.groups.join("#")),
            apps: format!("#{}#", add_or_modify_req.apps.join("#")),
            tenants: format!("#{}#", add_or_modify_req.tenants.join("#")),
            aks: format!("#{}#", add_or_modify_req.aks.join("#")),
            ..Default::default()
        };
        let mut res_dto = IamCacheResRelAddOrModifyDto {
            auth: None,
            need_crypto_req: false,
            need_crypto_resp: false,
            need_double_auth: false,
            need_only_aksk: false,
            need_login: false,
        };
        let uri_mixed = Self::package_uri_mixed(item_code, action);
        log::trace!("add or modify res rel: uri_mixed={}", uri_mixed);
        let rels = funs.cache().hget(&funs.conf::<IamConfig>().cache_key_res_info, &uri_mixed).await?;
        if let Some(rels) = rels {
            let old_res_dto = TardisFuns::json.str_to_obj::<IamCacheResRelAddOrModifyDto>(&rels)?;
            if let Some(old_auth) = old_res_dto.auth {
                res_auth.accounts = format!("{}{}", res_auth.accounts, old_auth.accounts);
                res_auth.roles = format!("{}{}", res_auth.roles, old_auth.roles);
                res_auth.groups = format!("{}{}", res_auth.groups, old_auth.groups);
                res_auth.apps = format!("{}{}", res_auth.apps, old_auth.apps);
                res_auth.tenants = format!("{}{}", res_auth.tenants, old_auth.tenants);
                res_auth.aks = format!("{}{}", res_auth.aks, old_auth.aks);
            }

            if let Some(need_crypto_req) = add_or_modify_req.need_crypto_req {
                res_dto.need_crypto_req = need_crypto_req
            } else {
                res_dto.need_crypto_req = old_res_dto.need_crypto_req
            }
            if let Some(need_crypto_resp) = add_or_modify_req.need_crypto_resp {
                res_dto.need_crypto_resp = need_crypto_resp
            } else {
                res_dto.need_crypto_resp = old_res_dto.need_crypto_resp
            }
            if let Some(need_double_auth) = add_or_modify_req.need_double_auth {
                res_dto.need_double_auth = need_double_auth
            } else {
                res_dto.need_double_auth = old_res_dto.need_double_auth
            }
            if let Some(need_only_aksk) = add_or_modify_req.need_only_aksk {
                res_dto.need_only_aksk = need_only_aksk
            } else {
                res_dto.need_only_aksk = old_res_dto.need_only_aksk
            }
            if let Some(need_login) = add_or_modify_req.need_login {
                res_dto.need_login = need_login
            } else {
                res_dto.need_login = old_res_dto.need_login
            }
        }
        res_auth.accounts = res_auth.accounts.replace("##", "#");
        res_auth.roles = res_auth.roles.replace("##", "#");
        res_auth.groups = res_auth.groups.replace("##", "#");
        res_auth.apps = res_auth.apps.replace("##", "#");
        res_auth.tenants = res_auth.tenants.replace("##", "#");
        res_auth.aks = res_auth.aks.replace("##", "#");

        if (res_auth.accounts == "#" || res_auth.accounts == "##")
            && (res_auth.roles == "#" || res_auth.roles == "##")
            && (res_auth.groups == "#" || res_auth.groups == "##")
            && (res_auth.apps == "#" || res_auth.apps == "##")
            && (res_auth.tenants == "#" || res_auth.tenants == "##")
            && (res_auth.aks == "#" || res_auth.aks == "##")
        {
            res_dto.auth = None;
        } else {
            res_dto.auth = Some(res_auth);
        }

        funs.cache().hset(&funs.conf::<IamConfig>().cache_key_res_info, &uri_mixed, &TardisFuns::json.obj_to_string(&res_dto)?).await?;
        Self::add_change_trigger(&uri_mixed, funs).await
    }

    pub async fn refresh_res_rel(item_code: &str, action: &str, add_or_modify_req: IamCacheResRelAddOrModifyReq, funs: &TardisFunsInst) -> TardisResult<()> {
        let uri_mixed = Self::package_uri_mixed(item_code, action);
        let res_auth = IamCacheResAuth {
            accounts: format!("#{}#", add_or_modify_req.accounts.join("#")),
            roles: format!("#{}#", add_or_modify_req.roles.join("#")),
            groups: format!("#{}#", add_or_modify_req.groups.join("#")),
            apps: format!("#{}#", add_or_modify_req.apps.join("#")),
            tenants: format!("#{}#", add_or_modify_req.tenants.join("#")),
            aks: format!("#{}#", add_or_modify_req.aks.join("#")),
            ..Default::default()
        };
        let mut res_dto = IamCacheResRelAddOrModifyDto {
            auth: None,
            need_crypto_req: false,
            need_crypto_resp: false,
            need_double_auth: false,
            need_only_aksk: false,
            need_login: false,
        };
        if let Some(need_crypto_req) = add_or_modify_req.need_crypto_req {
            res_dto.need_crypto_req = need_crypto_req
        }
        if let Some(need_crypto_resp) = add_or_modify_req.need_crypto_resp {
            res_dto.need_crypto_resp = need_crypto_resp
        }
        if let Some(need_double_auth) = add_or_modify_req.need_double_auth {
            res_dto.need_double_auth = need_double_auth
        }
        if let Some(need_only_aksk) = add_or_modify_req.need_only_aksk {
            res_dto.need_only_aksk = need_only_aksk
        }
        if let Some(need_login) = add_or_modify_req.need_login {
            res_dto.need_login = need_login
        }
        if (res_auth.accounts == "#" || res_auth.accounts == "##")
            && (res_auth.roles == "#" || res_auth.roles == "##")
            && (res_auth.groups == "#" || res_auth.groups == "##")
            && (res_auth.apps == "#" || res_auth.apps == "##")
            && (res_auth.tenants == "#" || res_auth.tenants == "##")
            && (res_auth.aks == "#" || res_auth.aks == "##")
        {
            res_dto.auth = None;
        } else {
            res_dto.auth = Some(res_auth);
        }
        log::trace!("refresh res rel: uri_mixed={}", uri_mixed);
        funs.cache().hset(&funs.conf::<IamConfig>().cache_key_res_info, &uri_mixed, &TardisFuns::json.obj_to_string(&res_dto)?).await?;
        Self::add_change_trigger(&uri_mixed, funs).await
    }

    pub async fn delete_res_rel(item_code: &str, action: &str, delete_req: &IamCacheResRelDeleteReq, funs: &TardisFunsInst) -> TardisResult<()> {
        let uri_mixed = Self::package_uri_mixed(item_code, action);
        log::trace!("delete res rel: uri_mixed={}", uri_mixed);
        let rels = funs.cache().hget(&funs.conf::<IamConfig>().cache_key_res_info, &uri_mixed).await?;
        if let Some(rels) = rels {
            let mut res_dto = TardisFuns::json.str_to_obj::<IamCacheResRelAddOrModifyDto>(&rels)?;
            if let Some(res_auth) = res_dto.auth {
                let mut auth = res_auth;
                for account in &delete_req.accounts {
                    while auth.accounts.contains(&format!("#{account}#")) {
                        auth.accounts = auth.accounts.replacen(&format!("#{account}#"), "#", 1);
                    }
                }
                for role in &delete_req.roles {
                    while auth.roles.contains(&format!("#{role}#")) {
                        auth.roles = auth.roles.replacen(&format!("#{role}#"), "#", 1);
                    }
                }
                for group in &delete_req.groups {
                    while auth.groups.contains(&format!("#{group}#")) {
                        auth.groups = auth.groups.replacen(&format!("#{group}#"), "#", 1);
                    }
                }
                for app in &delete_req.apps {
                    while auth.apps.contains(&format!("#{app}#")) {
                        auth.apps = auth.apps.replacen(&format!("#{app}#"), "#", 1);
                    }
                }
                for tenant in &delete_req.tenants {
                    while auth.tenants.contains(&format!("#{tenant}#")) {
                        auth.tenants = auth.tenants.replacen(&format!("#{tenant}#"), "#", 1);
                    }
                }
                for ak in &delete_req.aks {
                    while auth.aks.contains(&format!("#{ak}#")) {
                        auth.aks = auth.aks.replacen(&format!("#{ak}#"), "#", 1);
                    }
                }
                if (auth.accounts == "#" || auth.accounts == "##")
                    && (auth.roles == "#" || auth.roles == "##")
                    && (auth.groups == "#" || auth.groups == "##")
                    && (auth.apps == "#" || auth.apps == "##")
                    && (auth.tenants == "#" || auth.tenants == "##")
                {
                    res_dto.auth = None;
                } else {
                    res_dto.auth = Some(auth);
                }
            }
            funs.cache().hset(&funs.conf::<IamConfig>().cache_key_res_info, &uri_mixed, &TardisFuns::json.obj_to_string(&res_dto)?).await?;
            return Self::add_change_trigger(&uri_mixed, funs).await;
        }
        // Err(funs.err().not_found("iam_cache_res", "delete", "not found res rel", "404-iam-cache-res-rel-not-exist"))
        Ok(())
    }

    async fn add_change_trigger(uri: &str, funs: &TardisFunsInst) -> TardisResult<()> {
        funs.cache()
            .set_ex(
                &format!("{}{}", funs.conf::<IamConfig>().cache_key_res_changed_info_, uri),
                "",
                funs.conf::<IamConfig>().cache_key_res_changed_expire_sec as u64,
            )
            .await?;
        Ok(())
    }

    pub fn package_uri_mixed(item_code: &str, action: &str) -> String {
        let domain_idx = item_code.find('/').unwrap_or(item_code.len());
        let domain = &item_code[0..domain_idx];
        let path_and_query = if domain_idx != item_code.len() { &item_code[domain_idx + 1..] } else { "" };
        format!(
            "{}://{}/{}##{}",
            iam_constants::RBUM_KIND_CODE_IAM_RES.to_lowercase(),
            domain.to_lowercase(),
            path_and_query,
            action
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
struct IamCacheResRelAddOrModifyDto {
    pub auth: Option<IamCacheResAuth>,
    pub need_crypto_req: bool,
    pub need_crypto_resp: bool,
    pub need_double_auth: bool,
    pub need_only_aksk: bool,
    pub need_login: bool,
}
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
struct IamCacheResAuth {
    pub accounts: String,
    pub roles: String,
    pub groups: String,
    pub apps: String,
    pub tenants: String,
    pub aks: String,
    pub st: Option<i64>,
    pub et: Option<i64>,
}

pub struct IamCacheResRelAddOrModifyReq {
    pub st: Option<i64>,
    pub et: Option<i64>,
    pub accounts: Vec<String>,
    pub roles: Vec<String>,
    pub groups: Vec<String>,
    pub apps: Vec<String>,
    pub tenants: Vec<String>,
    pub aks: Vec<String>,
    pub need_crypto_req: Option<bool>,
    pub need_crypto_resp: Option<bool>,
    pub need_double_auth: Option<bool>,
    pub need_only_aksk: Option<bool>,
    pub need_login: Option<bool>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct IamCacheResRelDeleteReq {
    pub accounts: Vec<String>,
    pub roles: Vec<String>,
    pub groups: Vec<String>,
    pub apps: Vec<String>,
    pub tenants: Vec<String>,
    pub aks: Vec<String>,
}
