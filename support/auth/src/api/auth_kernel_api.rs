use tardis::log::trace;
use tardis::serde_json::Value;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use crate::dto::auth_kernel_dto::{AuthReq, AuthResp, MixRequest, MixAuthResp};
use crate::serv::{auth_kernel_serv, auth_res_serv};

pub struct AuthApi;

/// Auth API
#[poem_openapi::OpenApi(prefix_path = "/auth")]
impl AuthApi {
    /// Auth
    #[oai(path = "/", method = "put")]
    async fn auth(&self, mut req: Json<AuthReq>) -> TardisApiResult<AuthResp> {
        let result = auth_kernel_serv::auth(&mut req.0, false).await?;
        trace!("[Auth] Response auth: {:?}", result);
        TardisResp::ok(result)
    }

    // mix apis
    #[oai(path = "/apis", method = "put")]
    async fn apis(&self, req: Json<AuthReq>) -> TardisApiResult<MixAuthResp> {
        let result = auth_kernel_serv::parse_mix_req(req.0).await?;
        trace!("[Auth] Response apis: {:?}", result);
        TardisResp::ok(result)
    }
    
    /// fetch server config
    #[oai(path = "/apis", method = "get")]
    async fn fetch_server_config(&self) -> TardisApiResult<Value> {
        let result = auth_res_serv::get_apis_json()?;
        TardisResp::ok(result)
    }
}
