use bios_basic::process::task_processor::TaskProcessor;
use tardis::serde_json::Value;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Path;
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use crate::iam_config::IamConfig;
use crate::iam_constants::{self, IAM_AVATAR};
#[derive(Clone, Default)]
pub struct IamCcSystemApi;

/// Common Console System API
/// 通用控制台系统API
///
/// Use commas to separate multiple task ids
/// 使用逗号分隔多个任务id
#[poem_openapi::OpenApi(prefix_path = "/cc/system", tag = "bios_basic::ApiTag::Common")]
impl IamCcSystemApi {
    /// Get Async Task Status
    /// 获取异步任务状态
    #[oai(path = "/task/:task_ids", method = "get")]
    async fn task_check_finished(&self, task_ids: Path<String>) -> TardisApiResult<bool> {
        let funs = iam_constants::get_tardis_inst();
        let task_ids = task_ids.0.split(',');
        for task_id in task_ids {
            let task_id = task_id.parse().map_err(|_| funs.err().format_error("system", "task", "task id format error", "406-iam-task-id-format"))?;
            let is_finished = TaskProcessor::check_status(&funs.conf::<IamConfig>().cache_key_async_task_status, task_id, &funs.cache()).await?;
            if !is_finished {
                return TardisResp::ok(false);
            }
        }
        TardisResp::ok(true)
    }

    /// Stop Async Task
    /// 停止异步任务
    #[oai(path = "/task/:task_ids", method = "delete")]
    async fn stop_task(&self, task_ids: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = iam_constants::get_tardis_inst();
        let task_ids = task_ids.0.split(',');
        for task_id in task_ids {
            let task_id = task_id.parse().map_err(|_| funs.err().format_error("system", "task", "task id format error", "406-iam-task-id-format"))?;
            TaskProcessor::stop_task_with_event(
                &funs.conf::<IamConfig>().cache_key_async_task_status,
                task_id,
                &funs.cache(),
                IAM_AVATAR.to_owned(),
                Some(vec![format!("account/{}", ctx.0.owner)]),
            )
            .await?;
        }
        TardisResp::ok(Void {})
    }

    /// Get Task Process Data
    /// 获取任务处理数据
    #[oai(path = "/task/process/:task_id", method = "get")]
    async fn get_task_process_data(&self, task_id: Path<u64>, _ctx: TardisContextExtractor) -> TardisApiResult<Value> {
        let funs = iam_constants::get_tardis_inst();
        let data = TaskProcessor::get_process_data(&funs.conf::<IamConfig>().cache_key_async_task_status, task_id.0, &funs.cache()).await?;
        TardisResp::ok(data)
    }
}
