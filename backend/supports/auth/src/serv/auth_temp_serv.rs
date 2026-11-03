use std::sync::RwLock;

use lazy_static::lazy_static;
use serde::Deserialize;
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    cache::AsyncCommands,
    chrono::Utc,
    log::warn,
    TardisFuns,
};

use crate::{
    auth_config::AuthConfig,
    auth_constants::DOMAIN_CODE,
    dto::auth_kernel_dto::{AuthContext, ResContainerNode},
};

use super::auth_res_serv;

#[derive(Deserialize)]
struct TempProductGrant {
    own_paths: String,
    expire_at: i64,
}

struct TempApiCache {
    version: Option<String>,
    root: ResContainerNode,
}

lazy_static! {
    static ref TEMP_API_CACHE: RwLock<TempApiCache> = RwLock::new(TempApiCache {
        version: None,
        root: ResContainerNode::new(),
    });
}

/// 版本号变化时，用 Redis 中的临时 API Set 重建独立匹配树。
/// 请求路径只读这棵内存树。
pub async fn sync_temp_api_cache() -> TardisResult<()> {
    let config = TardisFuns::cs_config::<AuthConfig>(DOMAIN_CODE);
    let cache_client = TardisFuns::cache_by_module_or_default(DOMAIN_CODE);
    let version = cache_client.get(&config.cache_key_temp_api_version).await?.unwrap_or_default();
    {
        let cache = TEMP_API_CACHE.read()?;
        if cache.version.as_deref() == Some(version.as_str()) {
            return Ok(());
        }
    }
    let mut conn = cache_client.cmd().await?;
    let members: Vec<String> = conn.smembers(&config.cache_key_temp_api).await?;
    let mut root = ResContainerNode::new();
    for member in members {
        let Some((uri, action)) = split_api_member(&member) else {
            warn!("[Auth] skip invalid temp api member [{member}]");
            continue;
        };
        if let Err(error) = auth_res_serv::insert_res_pattern(&mut root, &action, &uri) {
            warn!("[Auth] skip temp api member [{member}] : {error}");
        }
    }
    let mut cache = TEMP_API_CACHE.write()?;
    cache.version = Some(version);
    cache.root = root;
    Ok(())
}

/// 账号在正式上下文里没有该应用时，用未过期的临时产品记录造一个最小上下文。
/// 读失败按没有授权处理，身份解析继续返回原来的 401。
pub async fn minimal_context(account_id: &str, app_id: &str) -> Option<TardisContext> {
    if app_id.is_empty() {
        return None;
    }
    let grant = match load_grant(account_id, app_id).await {
        Ok(Some(grant)) => grant,
        Ok(None) => return None,
        Err(error) => {
            warn!("[Auth] temp product grant read failed: {error}");
            return None;
        }
    };
    if grant_expired(&grant) {
        return None;
    }
    Some(TardisContext {
        own_paths: grant.own_paths,
        owner: account_id.to_string(),
        roles: vec![],
        groups: vec![],
        ..Default::default()
    })
}

/// 正式授权未命中后的临时放行。Redis 或记录异常按拒绝处理。
pub async fn allows(ctx: &AuthContext) -> bool {
    let (Some(account_id), Some(app_id)) = (ctx.account_id.as_deref(), ctx.app_id.as_deref()) else {
        return false;
    };
    if app_id.is_empty() {
        return false;
    }
    let grant = match load_grant(account_id, app_id).await {
        Ok(Some(grant)) => grant,
        Ok(None) => return false,
        Err(error) => {
            warn!("[Auth] temp product grant read failed: {error}");
            return false;
        }
    };
    if grant_expired(&grant) || ctx.own_paths.as_deref() != Some(grant.own_paths.as_str()) {
        return false;
    }
    let cache = match TEMP_API_CACHE.read() {
        Ok(cache) => cache,
        Err(error) => {
            warn!("[Auth] temp api cache read failed: {error}");
            return false;
        }
    };
    auth_res_serv::matches_res_pattern(&cache.root, &ctx.rbum_action, &ctx.rbum_uri).unwrap_or(false)
}

async fn load_grant(account_id: &str, app_id: &str) -> TardisResult<Option<TempProductGrant>> {
    let config = TardisFuns::cs_config::<AuthConfig>(DOMAIN_CODE);
    let cache_client = TardisFuns::cache_by_module_or_default(DOMAIN_CODE);
    let Some(raw) = cache_client.hget(&format!("{}{account_id}", config.cache_key_temp_product), app_id).await? else {
        return Ok(None);
    };
    match TardisFuns::json.str_to_obj::<TempProductGrant>(&raw) {
        Ok(grant) => Ok(Some(grant)),
        Err(error) => {
            warn!("[Auth] temp product grant [{account_id}/{app_id}] is not json: {error}");
            Ok(None)
        }
    }
}

fn grant_expired(grant: &TempProductGrant) -> bool {
    grant.expire_at > 0 && Utc::now().timestamp() >= grant.expire_at
}

fn split_api_member(member: &str) -> Option<(String, String)> {
    let (uri, action) = member.split_once("##")?;
    if uri.is_empty() || action.is_empty() || action.contains("##") {
        return None;
    }
    Some((uri.to_string(), action.to_string()))
}
