use tardis::log::trace;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use crate::dto::auth_kernel_dto::{AuthReq, AuthResp};
use crate::serv::auth_kernel_serv;

pub struct AuthApi;

/// Auth API
#[poem_openapi::OpenApi(prefix_path = "/auth")]
impl AuthApi {
    /// Auth
    #[oai(path = "/", method = "put")]
    async fn auth(&self, mut req: Json<AuthReq>) -> TardisApiResult<AuthResp> {
        let result = auth_kernel_serv::auth(&mut req.0).await?;
        trace!("[Auth] Response auth: {:?}", result);
        TardisResp::ok(result)
    }
}
