use tardis::basic::dto::TardisContext;
use tardis::log::trace;
use tardis::serde_json::Value;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use crate::dto::auth_kernel_dto::{AuthReq, AuthResp, MixAuthResp, SignWebHookReq};
use crate::serv::clients::spi_log_client::{LogParamContent, SpiLogClient};
use crate::serv::{auth_kernel_serv, auth_res_serv};

pub struct AuthApi;

/// Auth API
#[poem_openapi::OpenApi(prefix_path = "/auth")]
impl AuthApi {
    /// Auth
    #[oai(path = "/", method = "put")]
    async fn auth(&self, req: Json<AuthReq>, request: &Request) -> TardisApiResult<AuthResp> {
        let result = auth_kernel_serv::auth(&mut req.0.clone(), false).await?;
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

    // mix apis
    #[oai(path = "/apis", method = "put")]
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

    /// fetch server config
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

    /// sign webhook
    #[oai(path = "/sign/webhook", method = "put")]
    async fn sign_webhook(&self, req: Json<SignWebHookReq>) -> TardisApiResult<String> {
        let result = auth_kernel_serv::sign_webhook_ak(&req.0).await?;
        TardisResp::ok(result)
    }
}
