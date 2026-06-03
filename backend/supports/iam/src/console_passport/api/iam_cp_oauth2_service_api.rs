use bios_basic::helper::request_helper::try_set_real_ip_from_req_to_ctx;
use tardis::basic::field::TrimString;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Query;
use tardis::web::poem_openapi::payload::{Form, Json};
use tardis::web::poem_openapi::ApiResponse;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use crate::basic::dto::iam_cert_dto::{IamOauth2IntrospectReq, IamOauth2IntrospectResp, IamOauth2TokenResp, IamOauth2UserInfoResp};
use crate::console_passport::dto::iam_cp_cert_dto::{IamCpOAuth2ServiceAuthorizeReq, IamCpOAuth2ServiceAuthorizeResp, IamCpOAuth2ServiceTokenReq};
use crate::console_passport::serv::iam_cp_cert_oauth2_service_serv::IamCpCertOAuth2ServiceServ;
use crate::iam_constants;
use crate::iam_enumeration::OAuth2ResponseType;

/// GET /authorize 的响应：已登录时 302 跳转回接入方，参数错误时返回 400
#[derive(ApiResponse)]
enum IamOAuth2AuthorizeRedirectResp {
    /// 已登录，携带授权码重定向回接入方
    #[oai(status = 302)]
    Found(#[oai(header = "Location")] String),
    /// 参数或客户端校验错误
    #[oai(status = 400)]
    BadRequest(Json<String>),
}

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

    /// OAuth2 Authorization Endpoint (Browser-friendly, GET)
    /// OAuth2授权端点（浏览器友好，GET）
    ///
    /// Designed for browser redirects: when the user is already logged in (a valid
    /// `Tardis-Context` is present), it generates an authorization code and responds
    /// with a 302 redirect to the client's `redirect_uri` carrying `code` and `state`.
    ///
    /// 面向浏览器跳转：当用户已登录（携带有效 `Tardis-Context`）时，生成授权码并以 302
    /// 重定向到接入方 `redirect_uri`，携带 `code` 与 `state`。
    #[oai(path = "/authorize", method = "get")]
    async fn authorize_get(
        &self,
        client_id: Query<String>,
        redirect_uri: Query<String>,
        scope: Query<String>,
        state: Query<Option<String>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> IamOAuth2AuthorizeRedirectResp {
        if let Err(e) = try_set_real_ip_from_req_to_ctx(request, &ctx.0).await {
            return IamOAuth2AuthorizeRedirectResp::BadRequest(Json(e.to_string()));
        }
        let funs = iam_constants::get_tardis_inst();
        let authorize_req = IamCpOAuth2ServiceAuthorizeReq {
            client_id: TrimString::from(client_id.0),
            state: state.0,
            scope: TrimString::from(scope.0),
            redirect_uri: TrimString::from(redirect_uri.0),
            response_type: OAuth2ResponseType::Code,
        };
        match IamCpCertOAuth2ServiceServ::generate_authorization_code(&authorize_req, &funs, &ctx.0).await {
            Ok(resp) => {
                let _ = ctx.0.execute_task().await;
                IamOAuth2AuthorizeRedirectResp::Found(resp.redirect_url)
            }
            Err(e) => IamOAuth2AuthorizeRedirectResp::BadRequest(Json(e.to_string())),
        }
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

    /// OAuth2 UserInfo Endpoint
    /// OAuth2用户信息端点
    ///
    /// Resolve the account from the logged-in request context (`Tardis-Context`)
    /// and return its identity info. The returned `sub` equals the provider-side
    /// account id, which the client should use to bind/lookup its local account.
    ///
    /// 根据登录上下文（`Tardis-Context`）解析当前账号，并返回身份信息。
    /// 返回的 `sub` 等于 Provider 侧账号 ID，接入方应据此绑定/查找本地账号。
    #[oai(path = "/userinfo", method = "get")]
    async fn userinfo(&self, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<IamOauth2UserInfoResp> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let resp = IamCpCertOAuth2ServiceServ::get_userinfo_by_ctx(&funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(resp)
    }

    /// OAuth2 Token Introspection Endpoint
    /// OAuth2令牌内省端点
    ///
    /// Validate whether a token is active and, if so, return its subject.
    ///
    /// 校验令牌是否有效，若有效则返回其主体标识。
    #[oai(path = "/introspect", method = "post")]
    async fn introspect(&self, introspect_req: Form<IamOauth2IntrospectReq>, _request: &Request) -> TardisApiResult<IamOauth2IntrospectResp> {
        let funs = iam_constants::get_tardis_inst();
        let resp = IamCpCertOAuth2ServiceServ::introspect(&introspect_req.0.token, &funs).await?;
        TardisResp::ok(resp)
    }
}
