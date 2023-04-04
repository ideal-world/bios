use tardis::serde_json::Value;
use tardis::web::poem_openapi;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use crate::serv::auth_mgr_serv;

pub struct MgrApi;

/// Management API
#[poem_openapi::OpenApi(prefix_path = "/mgr")]
impl MgrApi {
    /// Fetch Cached Resources
    #[oai(path = "/cache/res", method = "get")]
    async fn fetch_cache_res(&self) -> TardisApiResult<Value> {
        let result = auth_mgr_serv::fetch_cache_res()?;
        TardisResp::ok(result)
    }
}
