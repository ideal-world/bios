use crate::basic::dto::iam_cert_dto::{IamOauth2TokenResp, IamOauth2UserInfoResp};
use crate::basic::serv::iam_cert_oauth2_serv::{IamCertOAuth2Spi, IamCertOAuth2TokenInfo};
use async_trait::async_trait;
use tardis::basic::result::TardisResult;
use tardis::chrono::Utc;
use tardis::log::trace;
use tardis::serde_json::{json, Value};
use tardis::web::web_resp::TardisResp;
use tardis::{TardisFuns, TardisFunsInst};

/// 对接另一套 bios IAM 作为 OAuth2 身份提供方的客户端实现
///
/// `ak` = client_id，`sk` = client_secret，`base_url` = 提供方基础地址（如 `https://bios.example.com`）。
/// 流程：用授权码换取 access_token（`POST {base_url}/cp/oauth2/token`），再用 access_token
/// 拉取用户信息（`GET {base_url}/cp/oauth2/userinfo`），并以 userinfo.sub 作为本地绑定的 open_id。
pub struct IamCertOAuth2SpiBiosIam;

const OAUTH2_BIOS_IAM_USER_INFO_CACHE_KEY: &str = "OAUTH2_BIOS_IAM_USER_INFO_CACHE_KEY:";

#[async_trait]
impl IamCertOAuth2Spi for IamCertOAuth2SpiBiosIam {
    async fn get_access_token(&self, code: &str, ak: &str, sk: &str, base_url: &str, funs: &TardisFunsInst) -> TardisResult<IamCertOAuth2TokenInfo> {
        if base_url.is_empty() {
            return Err(funs.err().bad_request(
                "oauth_spi_bios_iam",
                "get_access_token",
                "missing base_url for bios oauth2 supplier",
                "400-iam-cert-oauth-conf-error",
            ));
        }
        let base = base_url.trim_end_matches('/');

        // 1. 用授权码换取 access_token
        // grant_type 使用字面量 "authorization_code"，与提供方 poem_openapi 反序列化一致
        let token_body = json!({
            "grant_type": "authorization_code",
            "code": code,
            "client_id": ak,
            "client_secret": sk,
        })
        .to_string();
        let headers = vec![("Content-Type".to_string(), "application/json".to_string())];
        let token_result = funs.web_client().post_to_obj::<TardisResp<IamOauth2TokenResp>>(&format!("{base}/cp/oauth2/token"), token_body, headers).await?;
        if token_result.code != 200 {
            return Err(funs.err().not_found(
                "oauth_spi_bios_iam",
                "get_access_token",
                "bios oauth get access token error",
                "500-iam-cert-oauth-get-access-token-error",
            ));
        }
        let token = token_result.body.and_then(|b| b.data).ok_or_else(|| {
            funs.err().not_found(
                "oauth_spi_bios_iam",
                "get_access_token",
                "bios oauth get access token error: empty body",
                "500-iam-cert-oauth-get-access-token-error",
            )
        })?;
        let access_token = token.access_token.clone();

        // 2. 用 access_token 拉取用户信息（复用 get_user_info）
        let userinfo_value = self.get_user_info(&access_token, "", base, funs).await?;
        trace!("iam oauth2 spi [BiosIam] get user info response: {}", userinfo_value);
        let userinfo = TardisFuns::json.json_to_obj::<TardisResp<IamOauth2UserInfoResp>>(userinfo_value)?.data.ok_or_else(|| {
            funs.err().not_found(
                "oauth_spi_bios_iam",
                "get_access_token",
                "bios oauth get user info error: empty body",
                "500-iam-cert-oauth-get-user-info-error",
            )
        })?;

        // 缓存用户信息，供 get_account_name 使用
        funs.cache()
            .set_ex(
                &format!("{OAUTH2_BIOS_IAM_USER_INFO_CACHE_KEY}{access_token}"),
                &TardisFuns::json.obj_to_string(&userinfo)?,
                5,
            )
            .await?;

        token_info(token, userinfo.sub)
    }

    async fn get_account_name(&self, oauth2_info: IamCertOAuth2TokenInfo, funs: &TardisFunsInst) -> TardisResult<String> {
        let user_info = funs.cache().get(&format!("{}{}", OAUTH2_BIOS_IAM_USER_INFO_CACHE_KEY, oauth2_info.access_token)).await?;
        if let Some(user_info) = user_info {
            let result = TardisFuns::json.str_to_obj::<IamOauth2UserInfoResp>(&user_info)?;
            Ok(result.name)
        } else {
            Err(funs.err().not_found(
                "oauth_spi_bios_iam",
                "get_account_name",
                "bios oauth get account name error",
                "500-iam-cert-oauth-get-account-name-error",
            ))
        }
    }

    /// 使用 refresh_token 向 bios IAM Provider 置换新的 access_token
    ///
    /// 调用 Provider 的 `POST {base_url}/cp/oauth2/token`，grant_type 为 `refresh_token`。
    async fn refresh_access_token(
        &self,
        refresh_token: &str,
        ak: &str,
        sk: &str,
        base_url: &str,
        scope: Option<&str>,
        funs: &TardisFunsInst,
    ) -> TardisResult<IamCertOAuth2TokenInfo> {
        if base_url.is_empty() {
            return Err(funs.err().bad_request(
                "oauth_spi_bios_iam",
                "refresh_access_token",
                "missing base_url for bios oauth2 supplier",
                "400-iam-cert-oauth-conf-error",
            ));
        }
        let base = base_url.trim_end_matches('/');
        let mut token_body = json!({
            "grant_type": "refresh_token",
            "refresh_token": refresh_token,
            "client_id": ak,
            "client_secret": sk,
        });
        if let Some(scope) = scope.filter(|scope| !scope.trim().is_empty()) {
            token_body["scope"] = json!(scope);
        }
        let headers = vec![("Content-Type".to_string(), "application/json".to_string())];
        let token_result = funs.web_client().post_to_obj::<TardisResp<IamOauth2TokenResp>>(&format!("{base}/cp/oauth2/token"), token_body.to_string(), headers).await?;
        let envelope_code = token_result.body.as_ref().map(|body| body.code.as_str()).unwrap_or_default();
        if token_result.code != 200 || !envelope_code.starts_with("200") {
            return Err(provider_response_error(token_result.code, envelope_code));
        }
        let token = token_result.body.and_then(|b| b.data).ok_or_else(|| provider_response_error(502, ""))?;
        token_info(token, String::new())
    }

    /// 使用 access_token 向 bios IAM Provider 查询用户信息
    ///
    /// 调用 Provider 的 `GET {base_url}/cp/oauth2/userinfo`，返回原始用户信息 JSON。
    async fn get_user_info(&self, access_token: &str, _open_id: &str, base_url: &str, funs: &TardisFunsInst) -> TardisResult<Value> {
        if base_url.is_empty() {
            return Err(funs.err().bad_request(
                "oauth_spi_bios_iam",
                "get_user_info",
                "missing base_url for bios oauth2 supplier",
                "400-iam-cert-oauth-conf-error",
            ));
        }
        let base = base_url.trim_end_matches('/');
        let headers = vec![("Bios-Token".to_string(), format!("{access_token}"))];
        let userinfo_result = funs.web_client().get_to_str(&format!("{base}/cp/oauth2/userinfo"), headers).await?;
        if userinfo_result.code != 200 {
            return Err(provider_response_error(userinfo_result.code, ""));
        }
        let userinfo_body = userinfo_result.body.unwrap_or_default();
        let value = TardisFuns::json.str_to_obj::<Value>(&userinfo_body)?;
        let code = value.get("code").and_then(Value::as_str).unwrap_or_default();
        if !code.starts_with("200") {
            return Err(provider_response_error(200, code));
        }
        Ok(value)
    }

    async fn validate_access_token(&self, access_token: &str, base_url: &str, funs: &TardisFunsInst) -> TardisResult<String> {
        let userinfo = self.get_user_info(access_token, "", base_url, funs).await?;
        normalized_subject(&userinfo, funs)
    }
}

fn provider_response_error(http_status: u16, envelope_code: &str) -> tardis::basic::error::TardisError {
    use tardis::basic::error::TardisError;
    if [401, 403].contains(&http_status) || ["401", "403"].iter().any(|prefix| envelope_code.starts_with(prefix)) {
        TardisError::conflict(
            "external OAuth authorization was rejected; authorize again",
            "409-iam-oauth-provider-authorization-required",
        )
    } else {
        TardisError::bad_gateway("external OAuth provider is temporarily unavailable", "502-iam-oauth-provider-unavailable")
    }
}

fn normalized_subject(userinfo: &Value, funs: &TardisFunsInst) -> TardisResult<String> {
    userinfo
        .pointer("/data/sub")
        .and_then(Value::as_str)
        .filter(|subject| !subject.trim().is_empty())
        .map(str::to_owned)
        .ok_or_else(|| {
            funs.err().conflict(
                "oauth_spi_bios_iam",
                "validate_access_token",
                "provider response does not contain a valid subject",
                "409-iam-oauth-provider-subject-missing",
            )
        })
}

fn token_info(token: IamOauth2TokenResp, open_id: String) -> TardisResult<IamCertOAuth2TokenInfo> {
    let expires_in_sec = token.expires_in.max(0);
    let expires_in_ms = expires_in_sec.checked_mul(1_000);
    let expires_at_ms = expires_in_ms.map(|expires_in_ms| Utc::now().timestamp_millis().saturating_add(expires_in_ms));
    Ok(IamCertOAuth2TokenInfo {
        open_id,
        access_token: token.access_token,
        refresh_token: token.refresh_token,
        token_expires_ms: expires_in_ms.and_then(|expires_in_ms| u32::try_from(expires_in_ms).ok()),
        expires_at_ms,
        scope: token.scope,
        provider_cert_conf_id: None,
        union_id: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::iam_enumeration::Oauth2TokenType;

    fn test_funs() -> TardisFunsInst {
        TardisFuns::inst(crate::iam_constants::COMPONENT_CODE, None)
    }

    #[test]
    fn provider_outages_do_not_become_revoked_authorizations() {
        assert_eq!(provider_response_error(401, "").code, "409");
        assert_eq!(provider_response_error(200, "403-oauth2-revoked").code, "409");
        assert_eq!(provider_response_error(503, "").code, "502");
        assert_eq!(provider_response_error(200, "500-oauth2-error").code, "502");
    }

    #[test]
    fn mock_provider_token_response_keeps_refresh_expiry_and_scope() {
        let token = token_info(
            IamOauth2TokenResp {
                access_token: "access-token".to_string(),
                token_type: Oauth2TokenType::Bearer,
                expires_in: 90,
                refresh_token: Some("refresh-token".to_string()),
                scope: Some("iam.userinfo.read iam.app.read".to_string()),
            },
            "provider-user".to_string(),
        )
        .unwrap();

        assert_eq!(token.refresh_token.as_deref(), Some("refresh-token"));
        assert_eq!(token.token_expires_ms, Some(90_000));
        assert_eq!(token.scope.as_deref(), Some("iam.userinfo.read iam.app.read"));
        assert!(token.expires_at_ms.unwrap() > Utc::now().timestamp_millis());
    }

    #[test]
    fn provider_subject_normalization_rejects_missing_or_empty_subjects() {
        let funs = test_funs();
        assert_eq!(normalized_subject(&json!({"data": {"sub": "account-42"}}), &funs).unwrap(), "account-42");
        assert!(normalized_subject(&json!({"data": {"sub": " "}}), &funs).is_err());
        assert!(normalized_subject(&json!({"data": {}}), &funs).is_err());
    }
}
