use serde::Serialize;
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::cache::AsyncCommands;
use tardis::url::Url;
use tardis::{TardisFuns, TardisFunsInst};

use crate::iam_config::IamConfig;

#[derive(Serialize)]
struct TempProductGrant {
    own_paths: String,
    expire_at: i64,
}

pub struct IamCiTempServ;

impl IamCiTempServ {
    pub async fn grant_product(account_id: &str, product_id: &str, tenant_id: &str, expire_at: i64, funs: &TardisFunsInst) -> TardisResult<()> {
        let account_id = require_segment("account_id", account_id)?;
        let product_id = require_segment("product_id", product_id)?;
        let tenant_id = require_segment("tenant_id", tenant_id)?;
        if expire_at < 0 {
            return Err(bad_request("expire_at must be a unix second, or 0 for no expiry"));
        }
        let grant = TempProductGrant {
            own_paths: format!("{tenant_id}/{product_id}"),
            expire_at,
        };
        funs.cache()
            .hset(
                &format!("{}{account_id}", funs.conf::<IamConfig>().cache_key_temp_product_),
                &product_id,
                &TardisFuns::json.obj_to_string(&grant)?,
            )
            .await?;
        Ok(())
    }

    pub async fn revoke_product(account_id: &str, product_id: &str, funs: &TardisFunsInst) -> TardisResult<()> {
        let account_id = require_segment("account_id", account_id)?;
        let product_id = require_segment("product_id", product_id)?;
        funs.cache().hdel(&format!("{}{account_id}", funs.conf::<IamConfig>().cache_key_temp_product_), &product_id).await?;
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
    Url::parse(uri).map_err(|_| bad_request(&format!("temp api uri [{uri}] is not a valid resource uri")))?;
    Ok(format!("{uri}##{}", action.to_lowercase()))
}

fn bad_request(message: &str) -> TardisError {
    TardisError::bad_request(&format!("[IAM.Temp] {message}"), "400-iam-temp-api-invalid")
}
