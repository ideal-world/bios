use serde::{Deserialize, Serialize};
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use crate::dto::auth_dto::{AuthReq, AuthResp};
use crate::serv::auth_kernel_serv;

pub struct AuthApi;

/// Auth API
#[poem_openapi::OpenApi(prefix_path = "/auth")]
impl AuthApi {
    /// Auth
    #[oai(path = "/apisix", method = "put")]
    async fn apisix(&self, req: Json<ApisixAuthReq>) -> TardisApiResult<AuthResp> {
        let result = auth_kernel_serv::auth(&req.0.request).await?;
        TardisResp::ok(result)
    }
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
struct ApisixAuthReq {
    pub request: AuthReq,
}
