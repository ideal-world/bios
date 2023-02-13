use tardis::{basic::result::TardisResult, serde_json::Value};

use super::auth_res_serv;

pub(crate) fn fetch_cache_res() -> TardisResult<Value> {
    auth_res_serv::get_res_json()
}
