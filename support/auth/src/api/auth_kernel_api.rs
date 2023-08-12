use tardis::basic::dto::TardisContext;
use tardis::log::trace;
use tardis::serde_json::Value;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Query;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp};
use tardis::TardisFuns;

use crate::auth_config::AuthConfig;
use crate::auth_constants::DOMAIN_CODE;
use crate::dto::auth_kernel_dto::{AuthReq, AuthResp, MixAuthResp, SignWebHookReq};
use crate::serv::clients::spi_log_client::{LogParamContent, SpiLogClient};
use crate::serv::{auth_kernel_serv, auth_res_serv};

#[derive(Clone)]
pub struct AuthApi;

/// Auth API 身份验证API
#[poem_openapi::OpenApi(prefix_path = "/auth")]
impl AuthApi {
    /// Auth 身份验证
    #[oai(path = "/", method = "put")]
    async fn auth(&self, req: Json<AuthReq>, request: &Request) -> TardisApiResult<AuthResp> {
        let result = AuthResp::from_result(auth_kernel_serv::auth(&mut req.0.clone(), false).await?);
        trace!("[Auth] Response auth: {:?}", result);
        let _ = SpiLogClient::add_item(
            LogParamContent {
                op: "auth".to_string(),
                ext: None,
                addr: request.remote_addr().to_string(),
                auth_req: Some(req.0),
                ..Default::default()
            },
            None,
            None,
            &TardisContext::default(),
        )
        .await;
        TardisResp::ok(result)
    }

    // mix apis 解析混合api
    #[oai(path = "/apis", method = "post")]
    async fn apis(&self, req: Json<AuthReq>, request: &Request) -> TardisApiResult<MixAuthResp> {
        let result = auth_kernel_serv::parse_mix_req(req.0.clone()).await?;
        trace!("[Auth] Response apis: {:?}", result);
        let _ = SpiLogClient::add_item(
            LogParamContent {
                op: "mix-apis".to_string(),
                ext: None,
                addr: request.remote_addr().to_string(),
                auth_req: Some(req.0),
                ..Default::default()
            },
            None,
            None,
            &TardisContext::default(),
        )
        .await;
        TardisResp::ok(result)
    }

    /// fetch server config 获取服务器配置
    #[oai(path = "/apis", method = "get")]
    async fn fetch_server_config(&self, request: &Request) -> TardisApiResult<Value> {
        let result = auth_res_serv::get_apis_json()?;
        let _ = SpiLogClient::add_item(
            LogParamContent {
                op: "fetch_apis_config".to_string(),
                ext: None,
                addr: request.remote_addr().to_string(),
                ..Default::default()
            },
            None,
            None,
            &TardisContext::default(),
        )
        .await;
        TardisResp::ok(result)
    }

    /// get token context  获取token上下文
    #[oai(path = "/token", method = "get")]
    async fn get_token_context(&self, token: Query<String>, app_id: Query<Option<String>>) -> TardisApiResult<String> {
        let config = TardisFuns::cs_config::<AuthConfig>(DOMAIN_CODE);
        let cache_client = TardisFuns::cache_by_module_or_default(DOMAIN_CODE);
        let result = auth_kernel_serv::get_token_context(&token.0, &app_id.0.unwrap_or("".to_string()), config, cache_client).await?;
        TardisResp::ok(TardisFuns::crypto.base64.encode(&TardisFuns::json.obj_to_string(&result)?))
    }

    /// sign webhook 签名webhook
    #[oai(path = "/sign/webhook", method = "put")]
    async fn sign_webhook(&self, req: Json<SignWebHookReq>) -> TardisApiResult<String> {
        let result = auth_kernel_serv::sign_webhook_ak(&req.0).await?;
        TardisResp::ok(result)
    }
}
