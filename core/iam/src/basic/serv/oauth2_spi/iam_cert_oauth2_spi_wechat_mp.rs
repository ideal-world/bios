use async_trait::async_trait;
use tardis::{basic::result::TardisResult, log::trace, serde_json::Value, TardisFunsInst};

use crate::basic::serv::iam_cert_oauth2_by_code_serv::{IamCertOAuth2ByCodeSpi, IamCertOAuth2TokenInfo};

pub struct IamCertOAuth2SpiWeChatMp;

#[async_trait]
impl IamCertOAuth2ByCodeSpi for IamCertOAuth2SpiWeChatMp {
    // https://developers.weixin.qq.com/miniprogram/dev/OpenApiDoc/user-login/code2Session.html
    async fn get_access_token(code: &str, ak: &str, sk: &str, funs: &TardisFunsInst) -> TardisResult<IamCertOAuth2TokenInfo> {
        let result = funs
            .web_client()
            .post_to_obj::<Value>(
                &format!(
                    "https://api.weixin.qq.com/sns/jscode2session?appid={}&secret={}&js_code={}&grant_type=authorization_code",
                    ak, sk, code
                ),
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
        let result = result.body.unwrap();
        trace!("iam oauth2 spi [wechat_mp] get access token response: {}", result);
        if let Some(err) = result.get("errcode") {
            // 0成功，-1系统繁忙，40029 code无效，45011 访问次数限制（100次/分钟）
            let err = err.as_str().unwrap();
            if err != "0" {
                return Err(funs.err().not_found(
                    "oauth_spi_wechat_mp",
                    "get_access_token",
                    &format!("oauth get access token error:[{}]{}", err, result.get("errmsg").unwrap().as_str().unwrap()),
                    "500-iam-cert-oauth-get-access-token-error",
                ));
            }
        }
        let open_id = result.get("openid").unwrap().as_str().unwrap();
        let session_token = result.get("session_key").unwrap().as_str().unwrap();
        let union_id = result.get("unionid").map(|r| r.as_str().unwrap().to_string());
        Ok(IamCertOAuth2TokenInfo {
            open_id: open_id.to_string(),
            access_token: session_token.to_string(),
            refresh_token: None,
            token_expires_ms: None,
            union_id,
        })
    }
}
