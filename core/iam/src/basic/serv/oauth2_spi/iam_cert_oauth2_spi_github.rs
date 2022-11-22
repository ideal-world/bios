use crate::basic::serv::iam_cert_oauth2_serv::{IamCertOAuth2Spi, IamCertOAuth2TokenInfo};
use async_trait::async_trait;
use ldap3::log::trace;
use tardis::basic::result::TardisResult;
use tardis::serde_json::Value;
use tardis::{TardisFuns, TardisFunsInst};

pub struct IamCertOAuth2SpiGithub;
const OAUTH2_GITHUB_USER_INFO_CACHE_KEY: &str = "OAUTH2_GITHUB_USER_INFO_CACHE_KEY:";
#[async_trait]
impl IamCertOAuth2Spi for IamCertOAuth2SpiGithub {
    async fn get_access_token(&self, code: &str, ak: &str, sk: &str, funs: &TardisFunsInst) -> TardisResult<IamCertOAuth2TokenInfo> {
        //https://docs.github.com/cn/developers/apps/building-oauth-apps/authorizing-oauth-apps
        let headers = vec![("Accept".to_string(), "application/json".to_string())];
        let result = funs
            .web_client()
            .post_to_obj::<Value>(
                &format!("https://github.com/login/oauth/access_token?client_id={}&client_secret={}&code={}", ak, sk, code),
                "",
                Some(headers),
            )
            .await?;
        if result.code != 200 {
            return Err(funs.err().not_found(
                "oauth_spi_github",
                "get_access_token",
                "oauth get access token error",
                "500-iam-cert-oauth-get-access-token-error",
            ));
        }
        let result = result.body.unwrap();
        trace!("iam oauth2 spi [Github] get access token response: {}", result);
        if let Some(access_token) = result.get("access_token") {
            let access_token = access_token.as_str().unwrap();
            let headers = vec![
                ("Authorization".to_string(), format!("Bearer {}", access_token)),
                ("Accept".to_string(), "application/json".to_string()),
                ("User-Agent".to_string(), "BIOS".to_string()),
            ];
            //get user info
            let result = funs.web_client().get_to_str("https://api.github.com/user", Some(headers)).await?;
            trace!("iam oauth2 spi [Github] get user info response: {:?}", result);
            if result.code != 200 {
                return Err(funs.err().not_found(
                    "oauth_spi_github",
                    "get_access_token",
                    "oauth get user info error",
                    "500-iam-cert-oauth-get-user-info-error",
                ));
            }
            let user_info = result.body.unwrap();
            let user_info = TardisFuns::json.str_to_obj::<Value>(&user_info)?;
            funs.cache().set_ex(&format!("{}{}", OAUTH2_GITHUB_USER_INFO_CACHE_KEY, access_token), &user_info.to_string(), 5).await?;
            if let Some(id) = user_info.get("id") {
                Ok(IamCertOAuth2TokenInfo {
                    open_id: id.to_string(),
                    access_token: access_token.to_string(),
                    refresh_token: None,
                    token_expires_ms: None,
                    union_id: None,
                })
            } else {
                Err(funs.err().not_found(
                    "oauth_spi_github",
                    "get_access_token",
                    "oauth get user info error",
                    "500-iam-cert-oauth-get-user-info-error",
                ))
            }
        } else {
            let mut v_error = "";
            if let Some(error) = result.get("error") {
                v_error = error.as_str().unwrap();
            }
            Err(funs.err().not_found(
                "oauth_spi_github",
                "get_access_token",
                &format!("oauth get access token error:{}", v_error),
                "500-iam-cert-oauth-get-access-token-error",
            ))
        }
    }

    async fn get_account_name(&self, oauth2_info: IamCertOAuth2TokenInfo, funs: &TardisFunsInst) -> TardisResult<String> {
        let user_info = funs.cache().get(&format!("{}{}", OAUTH2_GITHUB_USER_INFO_CACHE_KEY, oauth2_info.access_token.clone())).await?;
        if let Some(user_info) = user_info {
            let result = TardisFuns::json.str_to_obj::<Value>(&user_info)?;
            Ok(result.get("name").unwrap_or(&Value::Null).as_str().unwrap_or("").to_string())
        } else {
            Err(funs.err().not_found(
                "oauth_spi_github",
                "get_account_name",
                "oauth get account name error",
                "500-iam-cert-oauth-get-account-name-error",
            ))
        }
    }
}
