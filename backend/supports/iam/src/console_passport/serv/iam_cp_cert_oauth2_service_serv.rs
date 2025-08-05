use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::TardisFunsInst;

use crate::basic::dto::iam_cert_dto::{IamCertOAuth2ServiceCodeAddReq, IamCertOAuth2ServiceCodeVerifyReq, IamCertOAuth2ServiceRefreshTokenReq, IamOauth2TokenResp};
use crate::basic::serv::iam_cert_oauth2_service_serv::IamCertOAuth2ServiceServ;
use crate::console_passport::dto::iam_cp_cert_dto::{IamCpOAuth2ServiceAuthorizeReq, IamCpOAuth2ServiceAuthorizeResp, IamCpOAuth2ServiceTokenReq};
use crate::iam_enumeration::Oauth2GrantType;

pub struct IamCpCertOAuth2ServiceServ;

impl IamCpCertOAuth2ServiceServ {
    /// Generate OAuth2 authorization code
    /// 生成OAuth2授权码
    pub async fn generate_authorization_code(req: &IamCpOAuth2ServiceAuthorizeReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<IamCpOAuth2ServiceAuthorizeResp> {
        let code_req = IamCertOAuth2ServiceCodeAddReq {
            client_id: req.client_id.clone(),
            state: req.state.clone(),
            scope: req.scope.clone(),
            redirect_uri: req.redirect_uri.clone(),
            response_type: req.response_type.clone(),
        };

        let code = IamCertOAuth2ServiceServ::generate_code(&code_req, funs, ctx).await?;

        // Build redirect URL with authorization code
        let mut redirect_url = req.redirect_uri.to_string();
        redirect_url.push_str(&format!("?code={}", code));

        if let Some(state) = &req.state {
            redirect_url.push_str(&format!("&state={}", state));
        }

        Ok(IamCpOAuth2ServiceAuthorizeResp { redirect_url })
    }

    /// Exchange authorization code or refresh token for access token
    /// 交换授权码或刷新令牌获取访问令牌
    pub async fn exchange_token(req: &IamCpOAuth2ServiceTokenReq, funs: &TardisFunsInst) -> TardisResult<IamOauth2TokenResp> {
        match req.grant_type {
            Oauth2GrantType::AuthorizationCode => {
                let code = req
                    .code
                    .as_ref()
                    .ok_or_else(|| funs.err().bad_request("oauth2", "exchange_token", "code is required for authorization_code grant", "400-oauth2-code-required"))?;

                let verify_req = IamCertOAuth2ServiceCodeVerifyReq {
                    grant_type: req.grant_type.clone(),
                    code: code.clone(),
                    client_id: req.client_id.clone(),
                    client_secret: req.client_secret.clone(),
                    redirect_uri: req.redirect_uri.clone(),
                };

                IamCertOAuth2ServiceServ::verify_code_and_generate_token(&verify_req, funs).await
            }
            Oauth2GrantType::RefreshToken => {
                let refresh_token = req.refresh_token.as_ref().ok_or_else(|| {
                    funs.err().bad_request(
                        "oauth2",
                        "exchange_token",
                        "refresh_token is required for refresh_token grant",
                        "400-oauth2-refresh-token-required",
                    )
                })?;

                let refresh_req = IamCertOAuth2ServiceRefreshTokenReq {
                    grant_type: req.grant_type.clone(),
                    refresh_token: refresh_token.clone(),
                    client_id: req.client_id.clone(),
                    client_secret: Some(req.client_secret.clone()),
                    scope: req.scope.clone(),
                };

                IamCertOAuth2ServiceServ::refresh_token(&refresh_req, funs).await
            }
            _ => Err(funs.err().bad_request("oauth2", "exchange_token", "unsupported grant type", "400-oauth2-unsupported-grant-type")),
        }
    }
}
