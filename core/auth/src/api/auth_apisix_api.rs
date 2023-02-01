use serde::{Deserialize, Serialize};
use tardis::log::debug;
use tardis::serde_json::json;
use tardis::web::poem::http::{Extensions, HeaderMap, StatusCode, Version};
use tardis::web::poem::{IntoResponse, Response};
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Path;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::poem_openapi::registry::{MetaResponses, Registry};
use tardis::web::poem_openapi::ApiResponse;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use crate::dto::auth_dto::{AuthReq, AuthResp};
use crate::serv::auth_kernel_serv;

pub struct AuthApi;

/// Auth API
#[poem_openapi::OpenApi(prefix_path = "/auth")]
impl AuthApi {
    // Auth
    // #[oai(path = "/apisix", method = "put")]
    // async fn apisix(&self, mut req: Json<ApisixAuthReq>) -> TardisApiResult<AuthResp> {
    //      let result = auth_kernel_serv::auth(&mut req.0.request).await?;
    //     TardisResp::ok(result)
    // }
}

pub struct MockOPAApi;
/// fake OPA API
/// POST /v1/data/<policy>
#[poem_openapi::OpenApi(prefix_path = "/v1")]
impl MockOPAApi {
    /// Auth endpoint
    #[oai(path = "/data/:policy", method = "post")]
    async fn apisix(&self, mut req: Json<ApisixAuthReq>, policy: Path<String>) -> TardisApiResult<AuthResp> {
        debug!("policy:{}", policy.0);
        let result = auth_kernel_serv::auth(&mut req.0.input.request).await?;
        TardisResp::ok(result)
    }
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
struct ApisixAuthReq {
    pub input: ApisixAuthInputReq,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
struct ApisixAuthInputReq {
    pub request: AuthReq,
}
