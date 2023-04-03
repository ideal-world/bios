use tardis::{basic::result::TardisResult, serde_json::Value, TardisFuns};

use crate::{auth_config::AuthConfig, auth_constants::DOMAIN_CODE};

use super::auth_res_serv;

pub(crate) fn fetch_cache_res() -> TardisResult<Value> {
    auth_res_serv::get_res_json()
}

pub(crate) async fn add_double_auth(account_id: &str) -> TardisResult<()> {
    let cache_client = TardisFuns::cache_by_module_or_default(DOMAIN_CODE);
    let config = TardisFuns::cs_config::<AuthConfig>(DOMAIN_CODE);
    let double_auth_exp_sec = config.double_auth_exp_sec;
    cache_client.set_ex(&format!("{}{}", config.cache_key_double_auth_info, account_id), "", double_auth_exp_sec as usize).await?;
    Ok(())
}

pub(crate) async fn has_double_auth(account_id: &str) -> TardisResult<bool> {
    let cache_client = TardisFuns::cache_by_module_or_default(DOMAIN_CODE);
    let config = TardisFuns::cs_config::<AuthConfig>(DOMAIN_CODE);
    let result = cache_client.exists(&format!("{}{}", config.cache_key_double_auth_info, account_id)).await?;
    Ok(result)
}
