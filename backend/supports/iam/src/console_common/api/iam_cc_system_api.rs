use std::collections::HashMap;
use std::time::Duration;

use bios_basic::process::task_processor::TaskProcessor;
use lapin::{Connection, ConnectionProperties};
use tardis::db::sea_orm::DatabaseBackend;
use tardis::log::warn;
use tardis::serde_json::json;
use tardis::serde_json::Value;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Path;
use tardis::web::poem_openapi::payload::{Json, Response};
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};
use tardis::TardisFuns;

use crate::basic::dto::iam_health_dto::{IamHealthComponentResp, IamHealthResp};
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
    /// 健康检查（服务、数据库、Redis），返回 Spring Boot Actuator 风格 JSON
    /// 全部组件健康时返回 200，任一组件不健康时返回 503（可配合 Blackbox Exporter http_2xx 探针）
    #[oai(path = "/health", method = "get")]
    async fn health(&self) -> Response<Json<IamHealthResp>> {
        let funs = iam_constants::get_tardis_inst();

        let mut healthy = true;
        let mut components = HashMap::new();

        // ping / liveness / readiness：进程能响应请求即视为存活
        components.insert("ping".to_string(), IamHealthComponentResp { status: "UP".to_string(), details: None });
        components.insert("livenessState".to_string(), IamHealthComponentResp { status: "UP".to_string(), details: None });
        components.insert("readinessState".to_string(), IamHealthComponentResp { status: "UP".to_string(), details: None });

        // database：实时探测（带超时，避免组件卡死时拖垮探针）
        let database_name = match TardisFuns::reldb().backend() {
            DatabaseBackend::Postgres => "PostgreSQL",
            DatabaseBackend::MySql => "MySQL",
            DatabaseBackend::Sqlite => "SQLite",
        };
        let database = match tardis::tokio::time::timeout(Duration::from_secs(3), funs.db().query_all("SELECT 1", vec![])).await {
            Ok(Ok(_)) => IamHealthComponentResp {
                status: "UP".to_string(),
                details: Some(json!({
                    "database": database_name,
                    "validationQuery": "SELECT 1",
                })),
            },
            Ok(Err(err)) => {
                warn!("[iam] health check database failed: {err}");
                healthy = false;
                IamHealthComponentResp { status: "DOWN".to_string(), details: Some(json!({ "error": err.to_string() })) }
            }
            Err(_) => {
                warn!("[iam] health check database timeout");
                healthy = false;
                IamHealthComponentResp { status: "DOWN".to_string(), details: Some(json!({ "error": "timeout" })) }
            }
        };
        components.insert("db".to_string(), database);

        // redis：实时探测（带超时）
        let redis = {
            let check = async {
                let cache_key = format!("{IAM_AVATAR}:health:{}", TardisFuns::field.nanoid());
                let mut ping_ok = false;
                if funs.cache().set_ex(&cache_key, "ok", 5).await.is_ok() {
                    if let Ok(value) = funs.cache().get(&cache_key).await {
                        if value.as_deref() == Some("ok") {
                            ping_ok = true;
                        }
                    }
                }
                let _ = funs.cache().del(&cache_key).await;
                ping_ok
            };
            match tardis::tokio::time::timeout(Duration::from_secs(3), check).await {
                Ok(true) => IamHealthComponentResp { status: "UP".to_string(), details: None },
                Ok(false) => {
                    healthy = false;
                    IamHealthComponentResp { status: "DOWN".to_string(), details: Some(json!({ "error": "ping failed" })) }
                }
                Err(_) => {
                    warn!("[iam] health check redis timeout");
                    healthy = false;
                    IamHealthComponentResp { status: "DOWN".to_string(), details: Some(json!({ "error": "timeout" })) }
                }
            }
        };
        components.insert("redis".to_string(), redis);

        // rabbit：参考 Spring Boot Actuator 的 RabbitHealthIndicator，
        // 通过建立 AMQP 连接验证 broker 可达（不发消息、不消费）；未配置 MQ 时跳过该组件
        let fw_config = TardisFuns::fw_config();
        if let Some(mq_url) = fw_config.mq.as_ref().map(|mq| mq.default.url.as_str().to_string()) {
            let rabbit = match tardis::tokio::time::timeout(Duration::from_secs(3), Connection::connect(&mq_url, ConnectionProperties::default())).await {
                Ok(Ok(conn)) => {
                    let _ = conn.close(200, "health check").await;
                    IamHealthComponentResp { status: "UP".to_string(), details: None }
                }
                Ok(Err(err)) => {
                    warn!("[iam] health check rabbit failed: {err}");
                    healthy = false;
                    IamHealthComponentResp { status: "DOWN".to_string(), details: Some(json!({ "error": err.to_string() })) }
                }
                Err(_) => {
                    warn!("[iam] health check rabbit timeout");
                    healthy = false;
                    IamHealthComponentResp { status: "DOWN".to_string(), details: Some(json!({ "error": "timeout" })) }
                }
            };
            components.insert("rabbit".to_string(), rabbit);
        }

        let resp = IamHealthResp {
            status: if healthy { "UP".to_string() } else { "DOWN".to_string() },
            groups: vec!["liveness".to_string(), "readiness".to_string()],
            components,
        };
        let status = if healthy {
            poem::http::StatusCode::OK
        } else {
            poem::http::StatusCode::SERVICE_UNAVAILABLE
        };
        Response::new(Json(resp)).status(status)
    }
}
