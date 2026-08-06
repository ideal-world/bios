use std::time::Duration;

use bios_basic::process::task_processor::TaskProcessor;
use tardis::log::warn;
use tardis::serde_json::Value;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Path;
use tardis::web::poem_openapi::payload::{PlainText, Response};
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};
use tardis::TardisFuns;

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

    /// Health Check
    /// 健康检查（服务、数据库、Redis），输出 Prometheus 文本格式指标供 Prometheus 使用
    /// 全部组件健康时返回 200，任一组件不健康时返回 503（可配合 Blackbox Exporter http_2xx 探针）
    #[oai(path = "/health", method = "get")]
    async fn health(&self) -> Response<PlainText<String>> {
        let funs = iam_constants::get_tardis_inst();

        // service：进程能响应请求即视为存活
        let service_healthy = 1u8;

        // database：实时探测（带超时，避免组件卡死时拖垮探针）
        let database_healthy = match tardis::tokio::time::timeout(Duration::from_secs(3), funs.db().query_all("SELECT 1", vec![])).await {
            Ok(Ok(_)) => 1u8,
            Ok(Err(err)) => {
                warn!("[iam] health check database failed: {err}");
                0u8
            }
            Err(_) => {
                warn!("[iam] health check database timeout");
                0u8
            }
        };

        // redis：实时探测（带超时）
        let redis_healthy = {
            let check = async {
                let cache_key = format!("{IAM_AVATAR}:health:{}", TardisFuns::field.nanoid());
                let mut healthy = 0u8;
                if funs.cache().set_ex(&cache_key, "ok", 5).await.is_ok() {
                    if let Ok(value) = funs.cache().get(&cache_key).await {
                        if value.as_deref() == Some("ok") {
                            healthy = 1u8;
                        }
                    }
                }
                let _ = funs.cache().del(&cache_key).await;
                healthy
            };
            match tardis::tokio::time::timeout(Duration::from_secs(3), check).await {
                Ok(healthy) => healthy,
                Err(_) => {
                    warn!("[iam] health check redis timeout");
                    0u8
                }
            }
        };

        // mq：忽略检查（MQ 在服务启动时即建立连接，启动失败会直接退出进程）

        let healthy = database_healthy == 1 && redis_healthy == 1;

        let body = format!(
            "# HELP iam_up Whether the IAM service process is up\n\
             # TYPE iam_up gauge\n\
             iam_up {service_healthy}\n\
             # HELP iam_database_up Whether the IAM database is reachable\n\
             # TYPE iam_database_up gauge\n\
             iam_database_up {database_healthy}\n\
             # HELP iam_redis_up Whether the IAM redis is reachable\n\
             # TYPE iam_redis_up gauge\n\
             iam_redis_up {redis_healthy}\n"
        );

        let status = if healthy {
            poem::http::StatusCode::OK
        } else {
            poem::http::StatusCode::SERVICE_UNAVAILABLE
        };
        Response::new(PlainText(body)).status(status)
    }
}
