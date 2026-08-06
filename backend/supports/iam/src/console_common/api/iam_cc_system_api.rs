use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

use bios_basic::process::task_processor::TaskProcessor;
use lazy_static::lazy_static;
use tardis::log::warn;
use tardis::serde_json::Value;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Path;
use tardis::web::poem_openapi::payload::{PlainText, Response};
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};
use tardis::TardisFuns;

use crate::basic::dto::iam_health_dto::IamHealthComponentResp;
use crate::iam_config::IamConfig;
use crate::iam_constants::{self, IAM_AVATAR};

lazy_static! {
    /// MQ 健康状态缓存，由后台任务低频探测更新，避免高频调用向 MQ 发送消息污染队列
    static ref MQ_HEALTH: Mutex<IamHealthComponentResp> = Mutex::new(IamHealthComponentResp { healthy: false, detail: None });
}

/// 启动 MQ 健康检查后台任务：每 60 秒探测一次 MQ 并缓存结果
pub fn init_mq_health_check() {
    tardis::tokio::spawn(async move {
        let funs = iam_constants::get_tardis_inst();
        let mut interval = tardis::tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            let mq_topic = format!("{IAM_AVATAR}::health");
            let mq_payload = format!("{IAM_AVATAR}-health-check");
            let mq = match funs.mq().publish(&mq_topic, mq_payload, &HashMap::new()).await {
                Ok(_) => IamHealthComponentResp { healthy: true, detail: None },
                Err(err) => IamHealthComponentResp {
                    healthy: false,
                    detail: Some(err.to_string()),
                },
            };
            *MQ_HEALTH.lock().unwrap() = mq;
        }
    });
}

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
    /// 健康检查（服务、数据库、Redis、MQ），输出 Prometheus 文本格式指标供 Prometheus 使用
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

        // mq：读取后台低频探测缓存（每 60s 更新一次），避免每次探测都向 MQ 发送消息污染队列
        let mq_healthy = if MQ_HEALTH.lock().unwrap().healthy { 1u8 } else { 0u8 };

        let healthy = database_healthy == 1 && redis_healthy == 1 && mq_healthy == 1;

        let body = format!(
            "# HELP iam_up Whether the IAM service process is up\n\
             # TYPE iam_up gauge\n\
             iam_up {service_healthy}\n\
             # HELP iam_database_up Whether the IAM database is reachable\n\
             # TYPE iam_database_up gauge\n\
             iam_database_up {database_healthy}\n\
             # HELP iam_redis_up Whether the IAM redis is reachable\n\
             # TYPE iam_redis_up gauge\n\
             iam_redis_up {redis_healthy}\n\
             # HELP iam_mq_up Whether the IAM MQ is reachable\n\
             # TYPE iam_mq_up gauge\n\
             iam_mq_up {mq_healthy}\n"
        );

        let status = if healthy {
            poem::http::StatusCode::OK
        } else {
            poem::http::StatusCode::SERVICE_UNAVAILABLE
        };
        Response::new(PlainText(body)).status(status)
    }
}
