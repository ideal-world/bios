use bios_basic::helper::request_helper::try_set_real_ip_from_req_to_ctx;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use crate::basic::dto::iam_cert_dto::IamOauth2TokenResp;
use crate::console_passport::dto::iam_cp_cert_dto::{IamCpOAuth2ServiceAuthorizeReq, IamCpOAuth2ServiceAuthorizeResp, IamCpOAuth2ServiceTokenReq};
use crate::console_passport::serv::iam_cp_cert_oauth2_service_serv::IamCpCertOAuth2ServiceServ;
use crate::iam_constants;

#[derive(Clone, Default)]
pub struct IamCpOAuth2ServiceApi;

/// Passport Console OAuth2 Service API
/// 通行证控制台OAuth2服务API
#[poem_openapi::OpenApi(prefix_path = "/cp/oauth2", tag = "bios_basic::ApiTag::Passport")]
impl IamCpOAuth2ServiceApi {
    /// OAuth2 Authorization Endpoint
    /// OAuth2授权端点
    ///
    /// This endpoint generates an authorization code that can be exchanged for an access token.
    /// It follows the OAuth2 authorization code flow.
    ///
    /// 此端点生成可以交换访问令牌的授权码。
    /// 它遵循OAuth2授权码流程。
    #[oai(path = "/authorize", method = "post")]
    async fn authorize(
        &self,
        authorize_req: Json<IamCpOAuth2ServiceAuthorizeReq>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<IamCpOAuth2ServiceAuthorizeResp> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let resp = IamCpCertOAuth2ServiceServ::generate_authorization_code(&authorize_req.0, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(resp)
    }

    /// OAuth2 Token Endpoint
    /// OAuth2令牌端点
    ///
    /// This endpoint exchanges authorization codes or refresh tokens for access tokens.
    /// Supports both authorization_code and refresh_token grant types.
    ///
    /// 此端点将授权码或刷新令牌交换为访问令牌。
    /// 支持authorization_code和refresh_token两种授权类型。
    #[oai(path = "/token", method = "post")]
    async fn token(&self, token_req: Json<IamCpOAuth2ServiceTokenReq>, request: &Request) -> TardisApiResult<IamOauth2TokenResp> {
        let funs = iam_constants::get_tardis_inst();
        let resp = IamCpCertOAuth2ServiceServ::exchange_token(&token_req.0, &funs).await?;
        TardisResp::ok(resp)
    }
}
