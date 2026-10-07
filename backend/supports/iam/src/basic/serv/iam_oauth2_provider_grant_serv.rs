use crate::basic::domain::iam_oauth2_provider_grant;
use crate::basic::serv::iam_cert_oauth2_serv::IamCertOAuth2TokenInfo;
use crate::iam_config::IamConfig;
use std::time::Duration;
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::chrono::Utc;
use tardis::crypto::crypto_aead::algorithm::Aes256Gcm;
use tardis::db::sea_orm::{sea_query::OnConflict, ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
use tardis::{TardisFuns, TardisFunsInst};

pub(crate) struct ProviderGrantCredential {
    pub id: String,
    pub account_id: String,
    pub cert_conf_id: String,
    pub external_subject: String,
    pub token: IamCertOAuth2TokenInfo,
}

pub struct IamOAuth2ProviderGrantServ;

impl IamOAuth2ProviderGrantServ {
    fn authorization_required() -> TardisError {
        TardisError::custom(
            "409-iam-oauth-provider-authorization-required",
            "external OAuth authorization is unavailable; authorize the provider again",
            "409-iam-oauth-provider-authorization-required",
        )
    }

    fn encryption_key(funs: &TardisFunsInst) -> TardisResult<Vec<u8>> {
        let key = TardisFuns::crypto
            .hex
            .decode(&funs.conf::<IamConfig>().oauth2_provider_grant_key)
            .map_err(|_| TardisError::internal_error("OAuth provider grant encryption key is invalid", "500-iam-oauth-provider-key-invalid"))?;
        if key.len() != 32 {
            return Err(TardisError::internal_error(
                "OAuth provider grant encryption key must contain 32 bytes",
                "500-iam-oauth-provider-key-invalid",
            ));
        }
        Ok(key)
    }

    fn grant_id(account_id: &str, cert_conf_id: &str, external_subject: &str) -> TardisResult<String> {
        let identity = TardisFuns::json.obj_to_string(&(account_id, cert_conf_id, external_subject))?;
        TardisFuns::crypto.digest.sha256(identity)
    }

    fn seal_token(key: &[u8], grant_id: &str, plaintext: &[u8]) -> TardisResult<String> {
        if key.len() != 32 {
            return Err(Self::authorization_required());
        }
        let nonce = TardisFuns::crypto.aead.random_nonce::<Aes256Gcm>();
        let (ciphertext, _) = TardisFuns::crypto.aead.encrypt::<Aes256Gcm>(key, grant_id.as_bytes(), &nonce, plaintext)?;
        Ok(format!("{}:{}", TardisFuns::crypto.hex.encode(nonce), TardisFuns::crypto.hex.encode(ciphertext)))
    }

    fn open_token(key: &[u8], grant_id: &str, sealed: &str) -> TardisResult<Vec<u8>> {
        if key.len() != 32 {
            return Err(Self::authorization_required());
        }
        let (nonce, ciphertext) = sealed.split_once(':').ok_or_else(Self::authorization_required)?;
        let nonce = TardisFuns::crypto.hex.decode(nonce).map_err(|_| Self::authorization_required())?;
        let ciphertext = TardisFuns::crypto.hex.decode(ciphertext).map_err(|_| Self::authorization_required())?;
        if nonce.len() != 12 {
            return Err(Self::authorization_required());
        }
        TardisFuns::crypto.aead.decrypt::<Aes256Gcm>(key, grant_id.as_bytes(), nonce, ciphertext).map_err(|_| Self::authorization_required())
    }

    fn validate_token_identity(token: &IamCertOAuth2TokenInfo, cert_conf_id: &str, subject: &str) -> TardisResult<()> {
        if subject.is_empty() || token.open_id != subject || token.provider_cert_conf_id.as_deref() != Some(cert_conf_id) {
            return Err(Self::authorization_required());
        }
        Ok(())
    }

    fn from_model(model: iam_oauth2_provider_grant::Model, funs: &TardisFunsInst) -> TardisResult<ProviderGrantCredential> {
        let expected_id = Self::grant_id(&model.account_id, &model.cert_conf_id, &model.external_subject)?;
        if expected_id != model.id {
            return Err(Self::authorization_required());
        }
        let plaintext = Self::open_token(&Self::encryption_key(funs)?, &model.id, &model.encrypted_token)?;
        let serialized = String::from_utf8(plaintext).map_err(|_| Self::authorization_required())?;
        let token: IamCertOAuth2TokenInfo = TardisFuns::json.str_to_obj(&serialized).map_err(|_| Self::authorization_required())?;
        Self::validate_token_identity(&token, &model.cert_conf_id, &model.external_subject)?;
        Ok(ProviderGrantCredential {
            id: model.id,
            account_id: model.account_id,
            cert_conf_id: model.cert_conf_id,
            external_subject: model.external_subject,
            token,
        })
    }

    pub(crate) async fn ensure_provider_grant(account_id: &str, token: &IamCertOAuth2TokenInfo, funs: &TardisFunsInst) -> TardisResult<String> {
        let cert_conf_id = token.provider_cert_conf_id.as_deref().filter(|value| !value.is_empty()).ok_or_else(Self::authorization_required)?;
        Self::validate_token_identity(token, cert_conf_id, &token.open_id)?;
        let id = Self::grant_id(account_id, cert_conf_id, &token.open_id)?;
        let sealed = Self::seal_token(&Self::encryption_key(funs)?, &id, &TardisFuns::json.obj_to_string(token)?.into_bytes())?;
        let model = iam_oauth2_provider_grant::ActiveModel {
            id: Set(id.clone()),
            account_id: Set(account_id.to_string()),
            cert_conf_id: Set(cert_conf_id.to_string()),
            external_subject: Set(token.open_id.clone()),
            encrypted_token: Set(sealed),
            update_time: Set(Utc::now().to_rfc3339()),
        };
        iam_oauth2_provider_grant::Entity::insert(model)
            .on_conflict(
                OnConflict::column(iam_oauth2_provider_grant::Column::Id)
                    .update_columns([iam_oauth2_provider_grant::Column::EncryptedToken, iam_oauth2_provider_grant::Column::UpdateTime])
                    .to_owned(),
            )
            .exec_without_returning(funs.db().raw_conn())
            .await?;
        Ok(id)
    }

    pub(crate) async fn load_by_id(id: &str, funs: &TardisFunsInst) -> TardisResult<ProviderGrantCredential> {
        let model = iam_oauth2_provider_grant::Entity::find_by_id(id).one(funs.db().raw_conn()).await?.ok_or_else(Self::authorization_required)?;
        Self::from_model(model, funs)
    }

    pub(crate) async fn load_by_identity(account_id: &str, cert_conf_id: &str, external_subject: &str, funs: &TardisFunsInst) -> TardisResult<Option<ProviderGrantCredential>> {
        let id = Self::grant_id(account_id, cert_conf_id, external_subject)?;
        let model = iam_oauth2_provider_grant::Entity::find_by_id(id).one(funs.db().raw_conn()).await?;
        model.map(|model| Self::from_model(model, funs)).transpose()
    }

    pub(crate) async fn load_by_account_config(account_id: &str, cert_conf_id: &str, funs: &TardisFunsInst) -> TardisResult<Vec<ProviderGrantCredential>> {
        let models = iam_oauth2_provider_grant::Entity::find()
            .filter(iam_oauth2_provider_grant::Column::AccountId.eq(account_id))
            .filter(iam_oauth2_provider_grant::Column::CertConfId.eq(cert_conf_id))
            .all(funs.db().raw_conn())
            .await?;
        models.into_iter().map(|model| Self::from_model(model, funs)).collect()
    }

    /// 在调用方的 Tardis 事务之外持久化刷新令牌轮换结果。
    /// 调用方在单条更新提交后才释放 Provider 租约。
    pub(crate) async fn persist_refreshed_token(account_id: &str, token: &IamCertOAuth2TokenInfo, funs: &TardisFunsInst) -> TardisResult<()> {
        let Some(cert_conf_id) = token.provider_cert_conf_id.as_deref().filter(|value| !value.is_empty()) else {
            return Ok(());
        };
        if token.open_id.is_empty() {
            return Err(Self::authorization_required());
        }
        let Some(current) = Self::load_by_identity(account_id, cert_conf_id, &token.open_id, funs).await? else {
            return Ok(());
        };
        let sealed = Self::seal_token(&Self::encryption_key(funs)?, &current.id, &TardisFuns::json.obj_to_string(token)?.into_bytes())?;
        let model = iam_oauth2_provider_grant::ActiveModel {
            id: Set(current.id),
            encrypted_token: Set(sealed),
            update_time: Set(Utc::now().to_rfc3339()),
            ..Default::default()
        };
        model.update(funs.db().raw_conn()).await?;
        Ok(())
    }

    pub(crate) fn cache_key(account_id: &str, funs: &TardisFunsInst) -> String {
        format!("{}BiosIam:{account_id}", funs.conf::<IamConfig>().cache_key_oauth2_provider_token_)
    }

    pub(crate) async fn get_cached_token(account_id: &str, funs: &TardisFunsInst) -> TardisResult<Option<IamCertOAuth2TokenInfo>> {
        funs.cache().get(&Self::cache_key(account_id, funs)).await?.map(|serialized| TardisFuns::json.str_to_obj(&serialized)).transpose()
    }

    pub(crate) async fn cache_token(account_id: &str, token: &IamCertOAuth2TokenInfo, funs: &TardisFunsInst) -> TardisResult<()> {
        let expire_sec = funs.conf::<IamConfig>().oauth2_provider_token_cache_expire_sec.max(crate::iam_constants::RBUM_CERT_CONF_TOKEN_EXPIRE_SEC as u32);
        funs.cache().set_ex(&Self::cache_key(account_id, funs), &TardisFuns::json.obj_to_string(token)?, expire_sec as u64).await?;
        Ok(())
    }

    pub(crate) async fn acquire_lease(account_id: &str, funs: &TardisFunsInst) -> TardisResult<String> {
        Self::try_acquire_lease(account_id, funs).await?.ok_or_else(Self::lease_in_progress_error)
    }

    pub(crate) async fn acquire_lease_with_wait(account_id: &str, funs: &TardisFunsInst) -> TardisResult<String> {
        let result = tardis::tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                if let Some(lease) = Self::try_acquire_lease(account_id, funs).await? {
                    return Ok(lease);
                }
                tardis::tokio::time::sleep(Duration::from_millis(50)).await;
            }
        })
        .await;
        result.unwrap_or_else(|_| Err(Self::lease_in_progress_error()))
    }

    async fn try_acquire_lease(account_id: &str, funs: &TardisFunsInst) -> TardisResult<Option<String>> {
        let key = format!("iam:oauth2:provider-refresh:BiosIam:{account_id}");
        let lease = TardisFuns::field.nanoid();
        let acquired: bool = funs.cache().script("return redis.call('SET', KEYS[1], ARGV[1], 'NX', 'EX', 120) ~= false").key(&key).arg(&lease).invoke().await?;
        Ok(acquired.then_some(lease))
    }

    fn lease_in_progress_error() -> TardisError {
        TardisError::custom("503", "OAuth credential refresh is in progress; retry later", "503-iam-oauth-refresh-in-progress")
    }

    pub(crate) async fn release_lease(account_id: &str, lease: &str, funs: &TardisFunsInst) -> TardisResult<()> {
        let key = format!("iam:oauth2:provider-refresh:BiosIam:{account_id}");
        let _: i64 = funs.cache().script("if redis.call('GET', KEYS[1]) == ARGV[1] then return redis.call('DEL', KEYS[1]) end return 0").key(&key).arg(lease).invoke().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_identity_is_scoped_to_account_configuration_and_subject() {
        let account_a = IamOAuth2ProviderGrantServ::grant_id("alice", "conf-a", "subject").unwrap();
        assert_ne!(account_a, IamOAuth2ProviderGrantServ::grant_id("bob", "conf-a", "subject").unwrap());
        assert_ne!(account_a, IamOAuth2ProviderGrantServ::grant_id("alice", "conf-b", "subject").unwrap());
        assert_ne!(account_a, IamOAuth2ProviderGrantServ::grant_id("alice", "conf-a", "other-subject").unwrap());
    }

    #[test]
    fn encrypted_provider_credential_is_bound_to_its_grant_id() {
        let key = vec![7; 32];
        let sealed = IamOAuth2ProviderGrantServ::seal_token(&key, "grant-a", b"rotated-refresh-token").unwrap();
        assert!(!sealed.contains("rotated-refresh-token"));
        assert_eq!(IamOAuth2ProviderGrantServ::open_token(&key, "grant-a", &sealed).unwrap(), b"rotated-refresh-token");
        assert!(IamOAuth2ProviderGrantServ::open_token(&key, "grant-b", &sealed).is_err());
    }

    #[test]
    fn missing_provider_authorization_has_a_stable_error_code() {
        assert_eq!(IamOAuth2ProviderGrantServ::authorization_required().code, "409-iam-oauth-provider-authorization-required");
    }
}
