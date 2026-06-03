use crate::basic::dto::iam_cert_dto::{IamOauth2TokenResp, IamOauth2UserInfoResp};
use crate::basic::serv::iam_cert_oauth2_serv::{IamCertOAuth2Spi, IamCertOAuth2TokenInfo};
use async_trait::async_trait;
use tardis::basic::result::TardisResult;
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
        let token_result = funs.web_client().post_to_obj::<TardisResp<IamOauth2TokenResp>>(&format!("{base}/cp/oauth2/token"), token_body, None).await?;
        if token_result.code != 200 {
            return Err(funs.err().not_found(
                "oauth_spi_bios_iam",
                "get_access_token",
                "bios oauth get access token error",
                "500-iam-cert-oauth-get-access-token-error",
            ));
        }
        let access_token = token_result.body.and_then(|b| b.data).map(|d| d.access_token).ok_or_else(|| {
            funs.err().not_found(
                "oauth_spi_bios_iam",
                "get_access_token",
                "bios oauth get access token error: empty body",
                "500-iam-cert-oauth-get-access-token-error",
            )
        })?;

        // 2. 用 access_token 拉取用户信息
        let headers = vec![("Authorization".to_string(), format!("Bearer {access_token}"))];
        let userinfo_result = funs.web_client().get_to_str(&format!("{base}/cp/oauth2/userinfo"), headers).await?;
        if userinfo_result.code != 200 {
            return Err(funs.err().not_found(
                "oauth_spi_bios_iam",
                "get_access_token",
                "bios oauth get user info error",
                "500-iam-cert-oauth-get-user-info-error",
            ));
        }
        let userinfo_body = userinfo_result.body.unwrap_or_default();
        trace!("iam oauth2 spi [BiosIam] get user info response: {}", userinfo_body);
        let userinfo = TardisFuns::json.str_to_obj::<TardisResp<IamOauth2UserInfoResp>>(&userinfo_body)?.data.ok_or_else(|| {
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

        Ok(IamCertOAuth2TokenInfo {
            open_id: userinfo.sub,
            access_token,
            refresh_token: None,
            token_expires_ms: None,
            union_id: None,
        })
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
    async fn refresh_access_token(&self, refresh_token: &str, ak: &str, sk: &str, base_url: &str, funs: &TardisFunsInst) -> TardisResult<IamCertOAuth2TokenInfo> {
        if base_url.is_empty() {
            return Err(funs.err().bad_request(
                "oauth_spi_bios_iam",
                "refresh_access_token",
                "missing base_url for bios oauth2 supplier",
                "400-iam-cert-oauth-conf-error",
            ));
        }
        let base = base_url.trim_end_matches('/');
        let token_body = json!({
            "grant_type": "refresh_token",
            "refresh_token": refresh_token,
            "client_id": ak,
            "client_secret": sk,
        })
        .to_string();
        let token_result = funs.web_client().post_to_obj::<TardisResp<IamOauth2TokenResp>>(&format!("{base}/cp/oauth2/token"), token_body, None).await?;
        if token_result.code != 200 {
            return Err(funs.err().not_found(
                "oauth_spi_bios_iam",
                "refresh_access_token",
                "bios oauth refresh access token error",
                "500-iam-cert-oauth-refresh-access-token-error",
            ));
        }
        let token = token_result.body.and_then(|b| b.data).ok_or_else(|| {
            funs.err().not_found(
                "oauth_spi_bios_iam",
                "refresh_access_token",
                "bios oauth refresh access token error: empty body",
                "500-iam-cert-oauth-refresh-access-token-error",
            )
        })?;
        Ok(IamCertOAuth2TokenInfo {
            open_id: String::new(),
            access_token: token.access_token,
            refresh_token: token.refresh_token,
            token_expires_ms: Some(token.expires_in.max(0) as u32),
            union_id: None,
        })
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
        let headers = vec![("Authorization".to_string(), format!("Bearer {access_token}"))];
        let userinfo_result = funs.web_client().get_to_str(&format!("{base}/cp/oauth2/userinfo"), headers).await?;
        if userinfo_result.code != 200 {
            return Err(funs.err().not_found(
                "oauth_spi_bios_iam",
                "get_user_info",
                "bios oauth get user info error",
                "500-iam-cert-oauth-get-user-info-error",
            ));
        }
        let userinfo_body = userinfo_result.body.unwrap_or_default();
        TardisFuns::json.str_to_obj::<Value>(&userinfo_body)
    }
}
