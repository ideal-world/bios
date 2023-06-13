use async_trait::async_trait;
use tardis::{
    basic::result::TardisResult,
    log::trace,
    serde_json::Value,
    TardisFunsInst,
};

use crate::basic::serv::iam_cert_oauth2_serv::{IamCertOAuth2Spi, IamCertOAuth2TokenInfo};

pub struct IamCertOAuth2SpiWeChatMp;

#[async_trait]
impl IamCertOAuth2Spi for IamCertOAuth2SpiWeChatMp {
    // https://developers.weixin.qq.com/miniprogram/dev/OpenApiDoc/user-login/code2Session.html
    async fn get_access_token(&self, code: &str, ak: &str, sk: &str, funs: &TardisFunsInst) -> TardisResult<IamCertOAuth2TokenInfo> {
        let result = funs
            .web_client()
            .post_to_obj::<Value>(
                &format!("https://api.weixin.qq.com/sns/jscode2session?appid={ak}&secret={sk}&js_code={code}&grant_type=authorization_code"),
                "",
                None,
            )
            .await?;
        if result.code != 200 {
            return Err(funs.err().not_found(
                "oauth_spi_wechat_mp",
                "get_access_token",
                "oauth get access token error",
                "500-iam-cert-oauth-get-access-token-error",
            ));
        }
        let result = result.body.unwrap_or_default();
        trace!("iam oauth2 spi [wechat_mp] get access token response: {}", result);
        if let Some(err) = result.get("errcode") {
            // 0成功，-1系统繁忙，40029 code无效，45011 访问次数限制（100次/分钟）
            let err = err.as_i64().unwrap_or_default();
            if err != 0 {
                return Err(funs.err().not_found(
                    "oauth_spi_wechat_mp",
                    "get_access_token",
                    &format!("oauth get access token error:[{}]{}", err, result.get("errmsg").unwrap().as_str().unwrap_or("")),
                    "500-iam-cert-oauth-get-access-token-error",
                ));
            }
        }
        let open_id = result
            .get("openid")
            .ok_or(funs.err().not_found(
                "oauth_spi_wechat_mp",
                "get_access_token",
                "oauth get access token error:missing field [openid]",
                "500-iam-cert-oauth-get-access-token-error",
            ))?
            .as_str()
            .unwrap_or("");
        let session_token = result
            .get("session_key")
            .ok_or(funs.err().not_found(
                "oauth_spi_wechat_mp",
                "get_access_token",
                "oauth get access token error:missing field [session_key]",
                "500-iam-cert-oauth-get-access-token-error",
            ))?
            .as_str()
            .unwrap_or("");
        let union_id = result.get("unionid").map(|r| r.as_str().unwrap_or("").to_string());
        Ok(IamCertOAuth2TokenInfo {
            open_id: open_id.to_string(),
            access_token: session_token.to_string(),
            refresh_token: None,
            token_expires_ms: None,
            union_id,
        })
    }

    async fn get_account_name(&self, oauth2_info: IamCertOAuth2TokenInfo, funs: &TardisFunsInst) -> TardisResult<String> {
        let result = funs
            .web_client()
            .post_to_obj::<Value>(
                &format!(
                    "https://api.weixin.qq.com/sns/userinfo?access_token={}&openid={}",
                    oauth2_info.access_token, oauth2_info.open_id,
                ),
                "",
                None,
            )
            .await?;
        if result.code != 200 {
            return Err(funs.err().not_found(
                "oauth_spi_wechat_mp",
                "get_account_info",
                "oauth get account info error",
                "500-iam-cert-oauth-get-account-info-error",
            ));
        }
        let result = result.body.unwrap_or_default();
        trace!("iam oauth2 spi [wechat_mp] get access token response: {}", result);
        if let Some(err) = result.get("errcode") {
            // 0成功，-1系统繁忙，40029 code无效，45011 访问次数限制（100次/分钟）
            let err = err.as_i64().unwrap_or_default();
            if err != 0 {
                return Err(funs.err().not_found(
                    "oauth_spi_wechat_mp",
                    "get_account_info",
                    &format!("oauth get account info error:[{}]{}", err, result.get("errmsg").unwrap().as_str().unwrap_or("")),
                    "500-iam-cert-oauth-get-account-info-error",
                ));
            }
        }
        let name = result
            .get("nick_name")
            .ok_or(funs.err().not_found(
                "oauth_spi_wechat_mp",
                "get_access_token",
                "oauth get access token error:missing field [nick_name]",
                "500-iam-cert-oauth-get-access-token-error",
            ))?
            .as_str()
            .unwrap_or("");
        let _ = result
            .get("mobile")
            .ok_or(funs.err().not_found(
                "oauth_spi_wechat_mp",
                "get_access_token",
                "oauth get access token error:missing field [mobile]",
                "500-iam-cert-oauth-get-access-token-error",
            ))?
            .as_str()
            .unwrap_or("");
        let _ = result
            .get("avatar")
            .ok_or(funs.err().not_found(
                "oauth_spi_wechat_mp",
                "get_access_token",
                "oauth get access token error:missing field [avatar]",
                "500-iam-cert-oauth-get-access-token-error",
            ))?
            .as_str()
            .unwrap_or("");
        let _ = result
            .get("sex")
            .ok_or(funs.err().not_found(
                "oauth_spi_wechat_mp",
                "get_access_token",
                "oauth get access token error:missing field [sex]",
                "500-iam-cert-oauth-get-access-token-error",
            ))?
            .as_str()
            .unwrap_or("");

        Ok(name.into())
    }
}
