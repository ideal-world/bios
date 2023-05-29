use bios_basic::process::task_processor::TaskProcessor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Path;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use crate::iam_config::IamConfig;
use crate::iam_constants;

pub struct IamCcSystemApi;

/// Common Console System API
///
/// Use commas to separate multiple task ids
#[poem_openapi::OpenApi(prefix_path = "/cc/system", tag = "bios_basic::ApiTag::Common")]
impl IamCcSystemApi {
    /// Get Async Task Status
    #[oai(path = "/task/:task_ids", method = "get")]
    async fn task_check_finished(&self, task_ids: Path<String>) -> TardisApiResult<bool> {
        let funs = iam_constants::get_tardis_inst();
        let task_ids = task_ids.0.split(',');
        for task_id in task_ids {
            let task_id = task_id.parse().map_err(|_| funs.err().format_error("system", "task", "task id format error", "406-iam-task-id-format"))?;
            let is_finished = TaskProcessor::check_status(&funs.conf::<IamConfig>().cache_key_async_task_status, task_id, funs.cache()).await?;
            if !is_finished {
                return TardisResp::ok(false);
            }
        }
        TardisResp::ok(true)
    }
}
