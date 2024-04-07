//! # 异步任务处理器

use std::{collections::HashMap, future::Future, sync::Arc};

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tardis::{
    basic::{dto::TardisContext, error::TardisError, result::TardisResult},
    cache::cache_client::TardisCacheClient,
    chrono::Local,
    log,
    serde_json::Value,
    tokio::{sync::RwLock, task::JoinHandle},
    web::{ws_client::TardisWSClient, ws_processor::TardisWebsocketReq},
    TardisFuns,
};

lazy_static! {
    static ref TASK_HANDLE: Arc<RwLock<HashMap<u64, JoinHandle<()>>>> = Arc::new(RwLock::new(HashMap::new()));
}
const TASK_PROCESSOR_DATA_EX_SEC: u64 = 60 * 60 * 24;
const TASK_IN_CTX_FLAG: &str = "task_id";

/// 设置任务状态事件标识
pub const EVENT_SET_TASK_STATUS_FLAG: &str = "set_task_status";
/// 设置任务处理数据事件标识
pub const EVENT_SET_TASK_PROCESS_DATA_FLAG: &str = "set_task_process";
/// 执行任务事件标识
pub const EVENT_EXECUTE_TASK_FLAG: &str = "execute_task";

pub struct TaskProcessor;

impl TaskProcessor {
    /// 初始化异步任务状态
    pub async fn init_status(cache_key: &str, task_id: Option<u64>, cache_client: &TardisCacheClient) -> TardisResult<u64> {
        let task_id = task_id.unwrap_or(Local::now().timestamp_nanos_opt().expect("maybe in 23rd century") as u64);
        // u32::MAX * u32::MAX + u32::MAX - 1
        if task_id > 18446744069414584319 {
            return Err(TardisError::bad_request("task id is too large", "400-task-id-too-large"));
        }
        // 使用bitmap存储以减少内存占用
        cache_client.setbit(&format!("{cache_key}:1"), (task_id / u32::MAX as u64) as usize, false).await?;
        cache_client.setbit(&format!("{cache_key}:2"), (task_id % u32::MAX as u64) as usize, false).await?;
        Ok(task_id)
    }

    /// 检查异步任务状态（是否完成）
    pub async fn check_status(cache_key: &str, task_id: u64, cache_client: &TardisCacheClient) -> TardisResult<bool> {
        let result1 = cache_client.getbit(&format!("{cache_key}:1"), (task_id / u32::MAX as u64) as usize).await?;
        let result2 = cache_client.getbit(&format!("{cache_key}:2"), (task_id % u32::MAX as u64) as usize).await?;
        Ok(result1 && result2)
    }

    /// 设置异步任务状态（是否完成）
    pub async fn set_status(cache_key: &str, task_id: u64, status: bool, cache_client: &TardisCacheClient) -> TardisResult<()> {
        cache_client.setbit(&format!("{cache_key}:1"), (task_id / u32::MAX as u64) as usize, status).await?;
        cache_client.setbit(&format!("{cache_key}:2"), (task_id % u32::MAX as u64) as usize, status).await?;
        Ok(())
    }

    /// 设置异步任务状态（是否完成）并发送事件
    pub async fn set_status_with_event(
        cache_key: &str,
        task_id: u64,
        status: bool,
        cache_client: &TardisCacheClient,
        ws_client: Option<TardisWSClient>,
        from_avatar: String,
        to_avatars: Option<Vec<String>>,
    ) -> TardisResult<()> {
        Self::set_status(cache_key, task_id, status, cache_client).await?;
        Self::send_event(
            ws_client,
            TaskWsEventReq {
                task_id,
                data: Value::Bool(status),
                msg: format!("task status: {}", status),
            },
            Some(EVENT_SET_TASK_STATUS_FLAG.to_owned()),
            from_avatar,
            to_avatars,
        )
        .await
    }

    /// 设置异步任务处理数据
    pub async fn set_process_data(cache_key: &str, task_id: u64, data: Value, cache_client: &TardisCacheClient) -> TardisResult<()> {
        cache_client.set_ex(&format!("{cache_key}:{task_id}"), &TardisFuns::json.json_to_string(data)?, TASK_PROCESSOR_DATA_EX_SEC).await?;
        Ok(())
    }

    /// 设置异步任务处理数据并发送事件
    pub async fn set_process_data_with_event(
        cache_key: &str,
        task_id: u64,
        data: Value,
        cache_client: &TardisCacheClient,
        ws_client: Option<TardisWSClient>,
        from_avatar: String,
        to_avatars: Option<Vec<String>>,
    ) -> TardisResult<()> {
        Self::set_process_data(cache_key, task_id, data.clone(), cache_client).await?;
        Self::send_event(
            ws_client,
            TaskWsEventReq {
                task_id,
                data: data.clone(),
                msg: format!("set task process: {}", &TardisFuns::json.json_to_string(data)?),
            },
            Some(EVENT_SET_TASK_PROCESS_DATA_FLAG.to_owned()),
            from_avatar,
            to_avatars,
        )
        .await?;
        Ok(())
    }

    /// 获取异步任务处理数据
    pub async fn get_process_data(cache_key: &str, task_id: u64, cache_client: &TardisCacheClient) -> TardisResult<Value> {
        if let Some(result) = cache_client.get(&format!("{cache_key}:{task_id}")).await? {
            Ok(TardisFuns::json.str_to_obj(&result)?)
        } else {
            Ok(Value::Null)
        }
    }

    /// 执行异步任务
    pub async fn execute_task<P, T>(cache_key: &str, process_fun: P, cache_client: &Arc<TardisCacheClient>) -> TardisResult<u64>
    where
        P: FnOnce(u64) -> T + Send + Sync + 'static,
        T: Future<Output = TardisResult<()>> + Send + 'static,
    {
        Self::do_execute_task_with_ctx(cache_key, process_fun, cache_client, None, "".to_string(), None, None).await
    }

    /// 执行异步任务并发送事件
    pub async fn execute_task_with_ctx<P, T>(
        cache_key: &str,
        process_fun: P,
        cache_client: &Arc<TardisCacheClient>,
        ws_client: Option<TardisWSClient>,
        from_avatar: String,
        to_avatars: Option<Vec<String>>,
        ctx: &TardisContext,
    ) -> TardisResult<u64>
    where
        P: FnOnce(u64) -> T + Send + Sync + 'static,
        T: Future<Output = TardisResult<()>> + Send + 'static,
    {
        Self::do_execute_task_with_ctx(cache_key, process_fun, cache_client, ws_client, from_avatar, to_avatars, Some(ctx)).await
    }

    async fn do_execute_task_with_ctx<P, T>(
        cache_key: &str,
        process_fun: P,
        cache_client: &Arc<TardisCacheClient>,
        ws_client: Option<TardisWSClient>,
        from_avatar: String,
        to_avatars: Option<Vec<String>>,
        ctx: Option<&TardisContext>,
    ) -> TardisResult<u64>
    where
        P: FnOnce(u64) -> T + Send + Sync + 'static,
        T: Future<Output = TardisResult<()>> + Send + 'static,
    {
        let cache_client_clone = cache_client.clone();
        let task_id = TaskProcessor::init_status(cache_key, None, cache_client).await?;
        let cache_key = cache_key.to_string();
        let ws_client_clone = ws_client.clone();
        let from_avatar_clone = from_avatar.clone();
        let to_avatars_clone = to_avatars.clone();
        let handle = tardis::tokio::spawn(async move {
            let result = process_fun(task_id).await;
            match result {
                Ok(_) => match TaskProcessor::set_status_with_event(&cache_key, task_id, true, &cache_client_clone, ws_client, from_avatar, to_avatars).await {
                    Ok(_) => {}
                    Err(e) => log::error!("Asynchronous task [{}] process error:{:?}", task_id, e),
                },
                Err(e) => {
                    log::error!("Asynchronous task [{}] process error:{:?}", task_id, e);
                }
            }
        });
        TASK_HANDLE.write().await.insert(task_id, handle);
        Self::send_event(
            ws_client_clone,
            TaskWsEventReq {
                task_id,
                data: ().into(),
                msg: "execute task start".to_owned(),
            },
            Some(EVENT_EXECUTE_TASK_FLAG.to_owned()),
            from_avatar_clone,
            to_avatars_clone,
        )
        .await?;
        if let Some(ctx) = ctx {
            if let Some(exist_task_ids) = ctx.get_ext(TASK_IN_CTX_FLAG).await? {
                ctx.add_ext(TASK_IN_CTX_FLAG, &format!("{exist_task_ids},{task_id}")).await?;
            } else {
                ctx.add_ext(TASK_IN_CTX_FLAG, &task_id.to_string()).await?;
            }
        }
        Ok(task_id)
    }

    /// 执行异步任务（不带异步函数，仅用于标记任务开始执行）
    pub async fn execute_task_without_fun(
        cache_key: &str,
        task_id: u64,
        cache_client: &Arc<TardisCacheClient>,
        ws_client: Option<TardisWSClient>,
        from_avatar: String,
        to_avatars: Option<Vec<String>>,
    ) -> TardisResult<u64> {
        let task_id = TaskProcessor::init_status(cache_key, Some(task_id), cache_client).await?;
        Self::send_event(
            ws_client,
            TaskWsEventReq {
                task_id,
                data: ().into(),
                msg: "execute task start".to_owned(),
            },
            Some(EVENT_EXECUTE_TASK_FLAG.to_owned()),
            from_avatar,
            to_avatars,
        )
        .await?;
        Ok(task_id)
    }

    /// 停止异步任务
    pub async fn stop_task(cache_key: &str, task_id: u64, cache_client: &TardisCacheClient) -> TardisResult<()> {
        Self::stop_task_with_event(cache_key, task_id, cache_client, None, "".to_string(), None).await
    }

    /// 停止异步任务并发送事件
    pub async fn stop_task_with_event(
        cache_key: &str,
        task_id: u64,
        cache_client: &TardisCacheClient,
        ws_client: Option<TardisWSClient>,
        from_avatar: String,
        to_avatars: Option<Vec<String>>,
    ) -> TardisResult<()> {
        if TaskProcessor::check_status(cache_key, task_id, cache_client).await? {
            TASK_HANDLE.write().await.remove(&task_id);
            return Ok(());
        }
        if TASK_HANDLE.read().await.contains_key(&task_id) {
            match TASK_HANDLE.write().await.remove(&task_id) {
                Some(handle) => {
                    handle.abort();
                }
                None => return Err(TardisError::bad_request("task not found,may task is end", "400-stop-task-error")),
            }
        }
        match TaskProcessor::set_status_with_event(cache_key, task_id, true, cache_client, ws_client, from_avatar, to_avatars).await {
            Ok(_) => {}
            Err(e) => log::error!("Asynchronous task [{}] stop error:{:?}", task_id, e),
        }
        Ok(())
    }

    /// 获取异步任务IDs
    ///
    /// 多个任务使用``,``分隔
    pub async fn get_task_id_with_ctx(ctx: &TardisContext) -> TardisResult<Option<String>> {
        ctx.get_ext(TASK_IN_CTX_FLAG).await
    }

    async fn send_event(ws_client: Option<TardisWSClient>, msg: TaskWsEventReq, event: Option<String>, from_avatar: String, to_avatars: Option<Vec<String>>) -> TardisResult<()> {
        if let Some(ws_client) = ws_client {
            let req = TardisWebsocketReq {
                msg: tardis::serde_json::Value::String(TardisFuns::json.obj_to_string(&msg)?),
                to_avatars,
                from_avatar,
                event,
                ..Default::default()
            };
            ws_client.send_obj(&req).await?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TaskWsEventReq {
    pub task_id: u64,
    pub data: Value,
    pub msg: String,
}
