use bios_basic::process::task_processor::TaskProcessor;
use tardis::serde_json::Value;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::{Path, Query};
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use crate::iam_constants;
use crate::iam_initializer::{default_iam_avatar, default_iam_send_avatar, ws_iam_client, ws_iam_send_client};
#[derive(Clone, Default)]
pub struct IamCiSystemApi;

/// Interface Console System API
///
/// Use commas to separate multiple task ids
#[poem_openapi::OpenApi(prefix_path = "/ci/system", tag = "bios_basic::ApiTag::Interface")]
impl IamCiSystemApi {
    #[oai(path = "/task/check/:task_ids", method = "get")]
    async fn task_check_finished(&self, cache_key: Query<String>, task_ids: Path<String>) -> TardisApiResult<bool> {
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

    #[oai(path = "/task/execute", method = "put")]
    async fn execute_task_external(&self, cache_key: Query<String>, task_id: Query<i64>, ctx: TardisContextExtractor) -> TardisApiResult<i64> {
        let funs = iam_constants::get_tardis_inst();
        let task_id = TaskProcessor::execute_task_external(
            &cache_key.0,
            task_id.0,
            ws_iam_send_client().await.clone(),
            default_iam_send_avatar().await.clone(),
            &funs,
            &ctx.0,
        )
        .await?;
        TardisResp::ok(task_id)
    }

    #[oai(path = "/task/execute/stop/:task_ids", method = "delete")]
    async fn stop_task_external(&self, cache_key: Query<String>, task_ids: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = iam_constants::get_tardis_inst();
        let task_ids = task_ids.0.split(',');
        for task_id in task_ids {
            let task_id = task_id.parse().map_err(|_| funs.err().format_error("system", "task", "task id format error", "406-iam-task-id-format"))?;
            TaskProcessor::stop_task(
                &cache_key.0,
                task_id,
                ws_iam_send_client().await.clone(),
                default_iam_send_avatar().await.clone(),
                &funs,
                &ctx.0,
            )
            .await?;
        }
        TardisResp::ok(Void {})
    }

    #[oai(path = "/task/process/:task_id", method = "put")]
    async fn set_task_process_data(&self, cache_key: Query<String>, task_id: Path<i64>, data: Json<Value>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = iam_constants::get_tardis_inst();
        TaskProcessor::set_task_process_data(
            &cache_key.0,
            task_id.0,
            data.0,
            ws_iam_send_client().await.clone(),
            default_iam_send_avatar().await.clone(),
            &funs,
            &ctx.0,
        )
        .await?;
        TardisResp::ok(Void {})
    }

    #[oai(path = "/task/process/:task_id", method = "get")]
    async fn get_task_process_data(&self, cache_key: Query<String>, task_id: Path<i64>, ctx: TardisContextExtractor) -> TardisApiResult<Value> {
        let funs = iam_constants::get_tardis_inst();
        let data = TaskProcessor::get_task_process_data(&cache_key.0, task_id.0, &funs).await?;
        TardisResp::ok(data)
    }
}
