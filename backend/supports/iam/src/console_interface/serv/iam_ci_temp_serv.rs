use serde::{Deserialize, Serialize};
use tardis::basic::dto::TardisContext;
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::cache::AsyncCommands;
use tardis::url::Url;
use tardis::{TardisFuns, TardisFunsInst};

use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_filer_dto::IamAppFilterReq;
use crate::basic::serv::iam_app_serv::IamAppServ;
use crate::iam_config::IamConfig;

#[derive(Serialize, Deserialize)]
struct TempProductGrant {
    own_paths: String,
    expire_at: i64,
}

pub struct IamCiTempServ;

impl IamCiTempServ {
    pub async fn grant_product(account_id: &str, product_id: &str, tenant_id: &str, expire_at: i64, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let account_id = require_segment("account_id", account_id)?;
        let product_id = require_segment("product_id", product_id)?;
        let tenant_id = require_segment("tenant_id", tenant_id)?;
        if expire_at < 0 {
            return Err(bad_request("expire_at must be a unix second, or 0 for no expiry"));
        }
        let ak_tenant = load_ak_tenant(&ctx.ak, funs).await?;
        let app = IamAppServ::get_item(&product_id, &global_app_filter(), funs, ctx).await?;
        if app.disabled {
            return Err(forbidden("product is disabled"));
        }
        let own_paths = format!("{tenant_id}/{product_id}");
        if app.own_paths != own_paths {
            return Err(forbidden("product does not belong to the tenant"));
        }
        if !ak_tenant.is_empty() && ak_tenant != tenant_id {
            return Err(forbidden("access key cannot grant a product outside its tenant"));
        }
        let grant = TempProductGrant { own_paths, expire_at };
        funs.cache()
            .hset(
                &format!("{}{account_id}", funs.conf::<IamConfig>().cache_key_temp_product_),
                &product_id,
                &TardisFuns::json.obj_to_string(&grant)?,
            )
            .await?;
        Ok(())
    }

    pub async fn revoke_product(account_id: &str, product_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let account_id = require_segment("account_id", account_id)?;
        let product_id = require_segment("product_id", product_id)?;
        let ak_tenant = load_ak_tenant(&ctx.ak, funs).await?;
        if !ak_tenant.is_empty() {
            let product_tenant = match product_own_paths(&product_id, funs, ctx).await? {
                Some(own_paths) => Some(tenant_segment(&own_paths)),
                None => stored_grant_tenant(&account_id, &product_id, funs).await?,
            };
            if let Some(product_tenant) = product_tenant {
                if product_tenant != ak_tenant {
                    return Err(forbidden("access key cannot revoke a product outside its tenant"));
                }
            }
        }
        funs.cache().hdel(&format!("{}{account_id}", funs.conf::<IamConfig>().cache_key_temp_product_), &product_id).await?;
        Ok(())
    }

    /// 全系统接口清单只允许未绑定租户的平台 AK 替换。
    pub async fn require_platform_ak(ak: &str, funs: &TardisFunsInst) -> TardisResult<()> {
        let ak_tenant = load_ak_tenant(ak, funs).await?;
        if !ak_tenant.is_empty() {
            return Err(forbidden("only a platform access key can replace temp apis"));
        }
        Ok(())
    }

    pub async fn replace_apis(apis: &[String], funs: &TardisFunsInst) -> TardisResult<()> {
        let mut members = Vec::with_capacity(apis.len());
        for api in apis {
            members.push(normalize_api_member(api)?);
        }
        let config = funs.conf::<IamConfig>();
        let mut conn = funs.cache().cmd().await?;
        let _: i32 = conn.del(&config.cache_key_temp_api).await?;
        if !members.is_empty() {
            let _: i32 = conn.sadd(&config.cache_key_temp_api, &members).await?;
        }
        funs.cache().incr(&config.cache_key_temp_api_version, 1).await?;
        Ok(())
    }
}

fn global_app_filter() -> IamAppFilterReq {
    IamAppFilterReq {
        basic: RbumBasicFilterReq {
            own_paths: Some(String::new()),
            with_sub_own_paths: true,
            ..Default::default()
        },
        ..Default::default()
    }
}

async fn product_own_paths(product_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<String>> {
    match IamAppServ::get_item(product_id, &global_app_filter(), funs, ctx).await {
        Ok(app) => Ok(Some(app.own_paths)),
        Err(error) if error.code.starts_with("404") => Ok(None),
        Err(error) => Err(error),
    }
}

async fn stored_grant_tenant(account_id: &str, product_id: &str, funs: &TardisFunsInst) -> TardisResult<Option<String>> {
    let Some(raw) = funs.cache().hget(&format!("{}{account_id}", funs.conf::<IamConfig>().cache_key_temp_product_), product_id).await? else {
        return Ok(None);
    };
    let grant = TardisFuns::json.str_to_obj::<TempProductGrant>(&raw)?;
    Ok(Some(tenant_segment(&grant.own_paths)))
}

fn tenant_segment(own_paths: &str) -> String {
    own_paths.split('/').next().unwrap_or("").to_string()
}

/// AK 缓存值是 `sk,tenant_id,app_id`。返回绑定的租户；平台 AK 的租户为空。
async fn load_ak_tenant(ak: &str, funs: &TardisFunsInst) -> TardisResult<String> {
    if ak.is_empty() {
        return Err(forbidden("access key is empty"));
    }
    let Some(raw) = funs.cache().get(&format!("{}{ak}", funs.conf::<IamConfig>().cache_key_aksk_info_)).await? else {
        return Err(forbidden("access key is not registered"));
    };
    let mut parts = raw.splitn(3, ',');
    let (Some(_sk), Some(tenant_id), Some(_app_id)) = (parts.next(), parts.next(), parts.next()) else {
        return Err(forbidden("access key record is invalid"));
    };
    Ok(tenant_id.to_string())
}

fn require_segment(name: &str, value: &str) -> TardisResult<String> {
    let value = value.trim();
    if value.is_empty() || value.contains('/') {
        return Err(bad_request(&format!("{name} must be a single path segment")));
    }
    Ok(value.to_string())
}

fn normalize_api_member(member: &str) -> TardisResult<String> {
    let member = member.trim();
    let Some((uri, action)) = member.split_once("##") else {
        return Err(bad_request(&format!("temp api [{member}] must be {{res_uri}}##{{res_action}}")));
    };
    let action = action.trim();
    if uri.is_empty() || action.is_empty() || action.contains("##") {
        return Err(bad_request(&format!("temp api [{member}] must be {{res_uri}}##{{res_action}}")));
    }
    let url = Url::parse(uri).map_err(|_| bad_request(&format!("temp api uri [{uri}] is not a valid resource uri")))?;
    if url.scheme().contains('*') || url.host_str().unwrap_or("").contains('*') {
        return Err(bad_request(&format!("temp api uri [{uri}] cannot wildcard the scheme or host")));
    }
    Ok(format!("{uri}##{}", action.to_lowercase()))
}

fn bad_request(message: &str) -> TardisError {
    TardisError::bad_request(&format!("[IAM.Temp] {message}"), "400-iam-temp-api-invalid")
}

fn forbidden(message: &str) -> TardisError {
    TardisError::forbidden(&format!("[IAM.Temp] {message}"), "403-iam-temp-forbidden")
}
