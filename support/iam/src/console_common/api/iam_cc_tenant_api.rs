use tardis::tokio::{self, task};
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Query;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use crate::basic::serv::iam_tenant_serv::IamTenantServ;
use crate::iam_constants;

pub struct IamCcTenantApi;

/// Common Console Tenant API
#[poem_openapi::OpenApi(prefix_path = "/cc/tenant", tag = "bios_basic::ApiTag::Common")]
impl IamCcTenantApi {
    /// Find Tenant Name By Ids
    ///
    /// Return format: ["<id>,<name>"]
    #[oai(path = "/name", method = "get")]
    async fn find_name_by_ids(
        &self,
        // Tenant Ids, multiple ids separated by ,
        ids: Query<String>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<Vec<String>> {
        let funs = iam_constants::get_tardis_inst();
        let ids = ids.0.split(',').map(|s| s.to_string()).collect();
        let result = IamTenantServ::find_name_by_ids(ids, &funs, &ctx.0).await?;
        let task_handle = task::spawn_blocking(move || tokio::runtime::Runtime::new().unwrap().block_on(ctx.0.execute_task()));
        let _ = task_handle.await;
        TardisResp::ok(result)
    }
}
