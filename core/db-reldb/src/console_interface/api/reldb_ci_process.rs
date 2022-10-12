use std::collections::HashMap;

use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Path;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use crate::basic::dto::reldb_process_dto::{RelDbExecuteReq, RelDbQueryReq};
use crate::reldb_constants;

pub struct RelDbProcessApi;

/// Interface Console Db Porcess API
#[poem_openapi::OpenApi(prefix_path = "/ci/proc", tag = "bios_basic::ApiTag::Interface")]
impl RelDbProcessApi {
    /// Query
    #[oai(path = "/:inst_id/query", method = "post")]
    async fn query(&self, inst_id: Path<String>, query_req: Json<RelDbQueryReq>, ctx: TardisContextExtractor) -> TardisApiResult<Vec<HashMap<String, String>>> {
        let funs = reldb_constants::get_tardis_inst();
        // TODO
        TardisResp::ok(Vec::new())
    }

    /// Execute
    #[oai(path = "/:inst_id/execute", method = "post")]
    async fn execute(&self, inst_id: Path<String>, execute_req: Json<RelDbExecuteReq>, ctx: TardisContextExtractor) -> TardisApiResult<Option<String>> {
        let funs = reldb_constants::get_tardis_inst();
        // TODO
        TardisResp::ok(None)
    }
}
