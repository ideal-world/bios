use bios_basic::process::task_processor::TaskProcessor;
use bios_basic::rbum::helper::rbum_scope_helper::check_without_owner_and_unsafe_fill_ctx;
use tardis::serde_json::Value;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::{Path, Query};
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use crate::iam_constants::{self, IAM_AVATAR};
#[derive(Clone, Default)]
pub struct IamCiSystemApi;

/// Interface Console System API
/// 接口控制台系统API
///
/// Use commas to separate multiple task ids
/// 使用逗号分隔多个任务id
#[poem_openapi::OpenApi(prefix_path = "/ci/system", tag = "bios_basic::ApiTag::Interface")]
impl IamCiSystemApi {
    /// Check if the task is finished
    /// 检查任务是否完成
    #[oai(path = "/task/check/:task_ids", method = "get")]
    async fn check_finished(&self, cache_key: Query<String>, task_ids: Path<String>) -> TardisApiResult<bool> {
        let funs = iam_constants::get_tardis_inst();
        let task_ids = task_ids.0.split(',');
        for task_id in task_ids {
            let task_id = task_id.parse().map_err(|_| funs.err().format_error("system", "task", "task id format error", "406-iam-task-id-format"))?;
            let is_finished = TaskProcessor::check_status(&cache_key.0, task_id, &funs.cache()).await?;
            if !is_finished {
                return TardisResp::ok(false);
            }
        }
        TardisResp::ok(true)
    }

    /// Execute Task
    /// 执行任务
    #[oai(path = "/task/execute", method = "put")]
    async fn execute_task(&self, cache_key: Query<String>, task_id: Query<u64>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<u64> {
        let funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        let task_id = TaskProcessor::execute_task_without_fun(
            &cache_key.0,
            task_id.0,
            &funs.cache(),
            IAM_AVATAR.to_owned(),
            Some(vec![format!("account/{}", ctx.0.owner)]),
        )
        .await?;
        TardisResp::ok(task_id)
    }

    /// Stop Task
    /// 停止任务
    #[oai(path = "/task/execute/stop/:task_ids", method = "delete")]
    async fn stop_task(&self, cache_key: Query<String>, task_ids: Path<String>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        let task_ids = task_ids.0.split(',');
        for task_id in task_ids {
            let task_id = task_id.parse().map_err(|_| funs.err().format_error("system", "task", "task id format error", "406-iam-task-id-format"))?;
            TaskProcessor::stop_task_with_event(&cache_key.0, task_id, &funs.cache(), IAM_AVATAR.to_owned(), Some(vec![format!("account/{}", ctx.0.owner)])).await?;
        }
        TardisResp::ok(Void {})
    }

    /// Set Task Process Data
    /// 设置任务处理数据
    #[oai(path = "/task/process/:task_id", method = "put")]
    async fn set_task_process_data(
        &self,
        cache_key: Query<String>,
        task_id: Path<u64>,
        data: Json<Value>,
        mut ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Void> {
        let funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        TaskProcessor::set_process_data_with_event(
            &cache_key.0,
            task_id.0,
            data.0,
            &funs.cache(),
            IAM_AVATAR.to_owned(),
            Some(vec![format!("account/{}", ctx.0.owner)]),
        )
        .await?;
        TardisResp::ok(Void {})
    }

    /// Get Task Process Data
    /// 获取任务处理数据
    #[oai(path = "/task/process/:task_id", method = "get")]
    async fn get_task_process_data(&self, cache_key: Query<String>, task_id: Path<u64>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Value> {
        let funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        let data = TaskProcessor::get_process_data(&cache_key.0, task_id.0, &funs.cache()).await?;
        TardisResp::ok(data)
    }
}
