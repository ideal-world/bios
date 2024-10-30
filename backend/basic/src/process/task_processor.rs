//! Async task processor
//!
//! 异步任务处理器
#[cfg(feature = "with-mq")]
use bios_sdk_invoke::clients::event_client::{
    asteroid_mq::prelude::{EventAttribute, Subject, TopicCode},
    get_topic,
};
use lazy_static::lazy_static;

use std::{collections::HashMap, future::Future, sync::Arc};
use tardis::{
    basic::{dto::TardisContext, error::TardisError, result::TardisResult},
    cache::cache_client::TardisCacheClient,
    chrono::Local,
    log,
    serde_json::Value,
    tokio::{sync::RwLock, task::JoinHandle},
    TardisFuns,
};

lazy_static! {
    static ref TASK_HANDLE: Arc<RwLock<HashMap<u64, JoinHandle<()>>>> = Arc::new(RwLock::new(HashMap::new()));
}
const TASK_PROCESSOR_DATA_EX_SEC: u64 = 60 * 60 * 24;
const TASK_IN_CTX_FLAG: &str = "task_id";
#[cfg(feature = "with-mq")]
const TASK_TOPIC: TopicCode = TopicCode::const_new("task");
/// Set task status event flag
/// 设置任务状态事件标识
pub const EVENT_SET_TASK_STATUS_FLAG: &str = "task/set_status";
/// Set task process data event flag
/// 设置任务处理数据事件标识
pub const EVENT_SET_TASK_PROCESS_DATA_FLAG: &str = "task/set_process";
/// Execute task event flag
/// 执行任务事件标识
pub const EVENT_EXECUTE_TASK_FLAG: &str = "task/execute";

pub struct TaskProcessor;

impl TaskProcessor {
    /// Initialize the asynchronous task status
    ///
    /// 初始化异步任务状态
    pub async fn init_status(cache_key: &str, task_id: Option<u64>, cache_client: &TardisCacheClient) -> TardisResult<u64> {
        let task_id = task_id.unwrap_or(Local::now().timestamp_nanos_opt().expect("maybe in 23rd century") as u64);
        Self::set_status(cache_key, task_id, false, cache_client).await?;
        Ok(task_id)
    }

    /// Set the status of the asynchronous task (whether it is completed)
    ///
    /// 设置异步任务状态（是否完成）
    pub async fn set_status(cache_key: &str, task_id: u64, status: bool, cache_client: &TardisCacheClient) -> TardisResult<()> {
        if task_id <= u32::MAX as u64 {
            cache_client.setbit(&format!("{cache_key}:1"), task_id as usize, status).await?;
        } else if task_id > 18446744069414584319 {
            // u32::MAX * u32::MAX + u32::MAX - 1
            cache_client.setbit(&format!("{cache_key}:2"), (u64::MAX - task_id) as usize, status).await?;
        } else {
            let _: String = cache_client
                .script(
                    r#"
                       redis.call('SETBIT', KEYS[1]..':3', ARGV[1], ARGV[3])
                       redis.call('SETBIT', KEYS[1]..':4', ARGV[2], ARGV[3])
                       return 'OK'
               "#,
                )
                .key(cache_key)
                .arg(&[task_id / u32::MAX as u64, task_id % u32::MAX as u64, if status { 1 } else { 0 }])
                .invoke()
                .await?;
        }
        Ok(())
    }

    /// Check the status of the asynchronous task (whether it is completed)
    ///
    /// 检查异步任务状态（是否完成）
    pub async fn check_status(cache_key: &str, task_id: u64, cache_client: &TardisCacheClient) -> TardisResult<bool> {
        if task_id <= u32::MAX as u64 {
            Ok(cache_client.getbit(&format!("{cache_key}:1"), task_id as usize).await?)
        } else if task_id > 18446744069414584319 {
            // u32::MAX * u32::MAX + u32::MAX - 1
            Ok(cache_client.getbit(&format!("{cache_key}:2"), (u64::MAX - task_id) as usize).await?)
        } else {
            let (r1, r2): (bool, bool) = cache_client
                .script(r#"return {redis.call('GETBIT', KEYS[1]..':3', ARGV[1]),redis.call('GETBIT', KEYS[1]..':4', ARGV[2])}"#)
                .key(cache_key)
                .arg(&[task_id / u32::MAX as u64, task_id % u32::MAX as u64])
                .invoke()
                .await?;
            Ok(r1 && r2)
        }
    }

    /// Set the status of the asynchronous task (whether it is completed) and send an event
    ///
    /// 设置异步任务状态（是否完成）并发送事件
    pub async fn set_status_with_event(
        cache_key: &str,
        task_id: u64,
        status: bool,
        cache_client: &TardisCacheClient,
        _from_avatar: String,
        _to_avatars: Option<Vec<String>>,
    ) -> TardisResult<()> {
        Self::set_status(cache_key, task_id, status, cache_client).await?;
        #[cfg(feature = "with-mq")]
        if let Some(_topic) = get_topic(&TASK_TOPIC) {
            // todo: broadcast event to users
            // topic
            //     .send_event(
            //         TaskSetStatusEventReq {
            //             task_id,
            //             data: status,
            //             msg: format!("task status: {}", status),
            //         }
            //         .json(),
            //     )
            //     .await
            //     .map_err(mq_error)?;
        }
        Ok(())
    }

    /// Set the processing data of the asynchronous task
    ///
    /// 设置异步任务处理数据
    pub async fn set_process_data(cache_key: &str, task_id: u64, data: Value, cache_client: &TardisCacheClient) -> TardisResult<()> {
        cache_client.set_ex(&format!("{cache_key}:{task_id}"), &TardisFuns::json.json_to_string(data)?, TASK_PROCESSOR_DATA_EX_SEC).await?;
        Ok(())
    }

    /// Set the processing data of the asynchronous task and send an event
    ///
    /// 设置异步任务处理数据并发送事件
    pub async fn set_process_data_with_event(
        cache_key: &str,
        task_id: u64,
        data: Value,
        cache_client: &TardisCacheClient,
        _from_avatar: String,
        _to_avatars: Option<Vec<String>>,
    ) -> TardisResult<()> {
        Self::set_process_data(cache_key, task_id, data.clone(), cache_client).await?;
        #[cfg(feature = "with-mq")]
        if let Some(_topic) = get_topic(&TASK_TOPIC) {
            // todo: broadcast event to users
        }
        Ok(())
    }

    /// Fetch the processing data of the asynchronous task
    ///
    /// 获取异步任务处理数据
    pub async fn get_process_data(cache_key: &str, task_id: u64, cache_client: &TardisCacheClient) -> TardisResult<Value> {
        if let Some(result) = cache_client.get(&format!("{cache_key}:{task_id}")).await? {
            Ok(TardisFuns::json.str_to_obj(&result)?)
        } else {
            Ok(Value::Null)
        }
    }

    /// Execute asynchronous task
    ///
    /// 执行异步任务
    pub async fn execute_task<P, T>(cache_key: &str, process_fun: P, cache_client: &Arc<TardisCacheClient>) -> TardisResult<u64>
    where
        P: FnOnce(u64) -> T + Send + Sync + 'static,
        T: Future<Output = TardisResult<()>> + Send + 'static,
    {
        Self::do_execute_task_with_ctx(cache_key, process_fun, cache_client, "".to_string(), None, None).await
    }

    /// Execute asynchronous task and send event
    ///
    /// 执行异步任务并发送事件
    pub async fn execute_task_with_ctx<P, T>(
        cache_key: &str,
        process_fun: P,
        cache_client: &Arc<TardisCacheClient>,
        from_avatar: String,
        to_avatars: Option<Vec<String>>,
        ctx: &TardisContext,
    ) -> TardisResult<u64>
    where
        P: FnOnce(u64) -> T + Send + Sync + 'static,
        T: Future<Output = TardisResult<()>> + Send + 'static,
    {
        Self::do_execute_task_with_ctx(cache_key, process_fun, cache_client, from_avatar, to_avatars, Some(ctx)).await
    }

    async fn do_execute_task_with_ctx<P, T>(
        cache_key: &str,
        process_fun: P,
        cache_client: &Arc<TardisCacheClient>,
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
        let _from_avatar_clone = from_avatar.clone();
        let _to_avatars_clone = to_avatars.clone();
        let handle = tardis::tokio::spawn(async move {
            let result = process_fun(task_id).await;
            match result {
                Ok(_) => match TaskProcessor::set_status_with_event(&cache_key, task_id, true, &cache_client_clone, from_avatar, to_avatars).await {
                    Ok(_) => {}
                    Err(e) => log::error!("Asynchronous task [{}] process error:{:?}", task_id, e),
                },
                Err(e) => {
                    log::error!("Asynchronous task [{}] process error:{:?}", task_id, e);
                }
            }
        });
        TASK_HANDLE.write().await.insert(task_id, handle);
        #[cfg(feature = "with-mq")]
        if let Some(_topic) = get_topic(&TASK_TOPIC) {
            // todo: broadcast event to users
        }
        if let Some(ctx) = ctx {
            if let Some(exist_task_ids) = ctx.get_ext(TASK_IN_CTX_FLAG).await? {
                ctx.add_ext(TASK_IN_CTX_FLAG, &format!("{exist_task_ids},{task_id}")).await?;
            } else {
                ctx.add_ext(TASK_IN_CTX_FLAG, &task_id.to_string()).await?;
            }
        }
        Ok(task_id)
    }

    /// Execute asynchronous task (without asynchronous function, only used to mark the start of the task)
    ///
    /// 执行异步任务（不带异步函数，仅用于标记任务开始执行）
    pub async fn execute_task_without_fun(
        cache_key: &str,
        task_id: u64,
        cache_client: &Arc<TardisCacheClient>,
        _from_avatar: String,
        _to_avatars: Option<Vec<String>>,
    ) -> TardisResult<u64> {
        let task_id = TaskProcessor::init_status(cache_key, Some(task_id), cache_client).await?;
        #[cfg(feature = "with-mq")]
        if let Some(_topic) = get_topic(&TASK_TOPIC) {
            // todo: broadcast event to users
        }
        Ok(task_id)
    }

    /// Stop asynchronous task
    ///
    /// 停止异步任务
    pub async fn stop_task(cache_key: &str, task_id: u64, cache_client: &TardisCacheClient) -> TardisResult<()> {
        Self::stop_task_with_event(cache_key, task_id, cache_client, "".to_string(), None).await
    }

    /// Stop asynchronous task and send event
    ///
    /// 停止异步任务并发送事件
    pub async fn stop_task_with_event(cache_key: &str, task_id: u64, cache_client: &TardisCacheClient, from_avatar: String, to_avatars: Option<Vec<String>>) -> TardisResult<()> {
        if TaskProcessor::check_status(cache_key, task_id, cache_client).await? {
            TASK_HANDLE.write().await.remove(&task_id);
            return Ok(());
        }
        if TASK_HANDLE.read().await.contains_key(&task_id) {
            match TASK_HANDLE.write().await.remove(&task_id) {
                Some(handle) => {
                    handle.abort();
                }
                None => return Err(TardisError::bad_request("task not found,may task is end", "400-task-stop-error")),
            }
        }
        match TaskProcessor::set_status_with_event(cache_key, task_id, true, cache_client, from_avatar, to_avatars).await {
            Ok(_) => {}
            Err(e) => log::error!("Asynchronous task [{}] stop error:{:?}", task_id, e),
        }
        Ok(())
    }

    /// Fetch the asynchronous task id set in the context
    ///
    /// 获取异步任务id集合
    ///
    /// Use ``,`` to separate multiple tasks
    ///
    /// 多个任务使用``,``分隔
    pub async fn get_task_id_with_ctx(ctx: &TardisContext) -> TardisResult<Option<String>> {
        ctx.get_ext(TASK_IN_CTX_FLAG).await
    }
}
#[cfg(feature = "with-mq")]
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
struct TaskSetStatusEventReq {
    pub task_id: u64,
    pub data: bool,
    pub msg: String,
}
#[cfg(feature = "with-mq")]
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
struct TaskSetProcessDataEventReq {
    pub task_id: u64,
    pub data: Value,
    pub msg: String,
}
#[cfg(feature = "with-mq")]
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
struct TaskExecuteEventReq {
    pub task_id: u64,
    pub msg: String,
}
#[cfg(feature = "with-mq")]

impl EventAttribute for TaskSetStatusEventReq {
    const SUBJECT: Subject = Subject::const_new(EVENT_SET_TASK_STATUS_FLAG);
}
#[cfg(feature = "with-mq")]

impl EventAttribute for TaskSetProcessDataEventReq {
    const SUBJECT: Subject = Subject::const_new(EVENT_SET_TASK_PROCESS_DATA_FLAG);
}
#[cfg(feature = "with-mq")]

impl EventAttribute for TaskExecuteEventReq {
    const SUBJECT: Subject = Subject::const_new(EVENT_EXECUTE_TASK_FLAG);
}
