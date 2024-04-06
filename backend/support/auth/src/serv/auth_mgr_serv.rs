use tardis::{basic::result::TardisResult, TardisFuns};

use crate::{auth_config::AuthConfig, auth_constants::DOMAIN_CODE};

pub(crate) async fn has_double_auth(account_id: &str) -> TardisResult<bool> {
    let cache_client = TardisFuns::cache_by_module_or_default(DOMAIN_CODE);
    let config = TardisFuns::cs_config::<AuthConfig>(DOMAIN_CODE);
    let result = cache_client.exists(&format!("{}{}", config.cache_key_double_auth_info, account_id)).await?;
    Ok(result)
}
