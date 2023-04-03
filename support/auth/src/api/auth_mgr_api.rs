use tardis::serde_json::Value;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use crate::dto::auth_dto::MgrDoubleAuthReq;
use crate::serv::auth_mgr_serv;

pub struct MgrApi;

/// Management API
#[poem_openapi::OpenApi(prefix_path = "/mgr")]
impl MgrApi {
    /// Add Double Auth
    #[oai(path = "/auth/double", method = "post")]
    async fn add_double_auth(&self, add_req: Json<MgrDoubleAuthReq>) -> TardisApiResult<Void> {
        auth_mgr_serv::add_double_auth(&add_req.0.account_id).await?;
        TardisResp::ok(Void {})
    }

    /// TODO Add Res
    /// TODO Remove Res

    /// Fetch Cached Resources
    #[oai(path = "/cache/res", method = "get")]
    async fn fetch_cache_res(&self) -> TardisApiResult<Value> {
        let result = auth_mgr_serv::fetch_cache_res()?;
        TardisResp::ok(result)
    }
}
