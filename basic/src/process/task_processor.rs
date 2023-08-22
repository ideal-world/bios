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
    TardisFuns, TardisFunsInst,
};

use crate::rbum::rbum_config::RbumConfigApi;

lazy_static! {
    static ref TASK_HANDLE: Arc<RwLock<HashMap<i64, JoinHandle<()>>>> = Arc::new(RwLock::new(HashMap::new()));
}
const TASK_IN_CTX_FLAG: &str = "task_id";
const NOTIFY_EVENT_IN_CTX_FLAG: &str = "notify";
const TASK_PROCESSOR_DATA_EX: usize = 60 * 60 * 24;
pub struct TaskProcessor;

impl TaskProcessor {
    pub async fn subscribe_task(funs: &TardisFunsInst) -> TardisResult<()> {
        let component_code = funs.module_code().to_string();
        //todo Use node id to differentiate nodes
        funs.mq()
            .subscribe(&funs.rbum_conf_task_mq_topic_event(), move |(_, msg)| {
                let component_code_clone = component_code.clone();
                async move {
                    let funs = TardisFuns::inst_with_db_conn(component_code_clone, None);
                    let task_msg = TardisFuns::json.str_to_obj::<TaskEventMessage>(&msg)?;
                    match task_msg.operate {
                        TaskOperate::Stop => {
                            Self::do_stop_task(&task_msg.cache_key, task_msg.task_id, &funs).await?;
                        }
                        _ => {}
                    }
                    Ok(())
                }
            })
            .await?;
        Ok(())
    }

    pub async fn init_task(cache_key: &str, cache_client: &TardisCacheClient) -> TardisResult<i64> {
        //todo change to SnowFlake or other distributed ID generator
        let task_id = Local::now().timestamp_nanos();
        let max: i64 = u32::MAX.into();
        let task_id_split1: usize = (task_id / max).try_into()?;
        let task_id_split2: usize = (task_id % max).try_into()?;
        cache_client.setbit(&format!("{cache_key}:1"), task_id_split1, false).await?;
        cache_client.setbit(&format!("{cache_key}:2"), task_id_split2, false).await?;
        Ok(task_id)
    }

    pub async fn set_status(cache_key: &str, task_id: i64, status: bool, cache_client: &TardisCacheClient) -> TardisResult<()> {
        let max: i64 = u32::MAX.into();
        let task_id_split1: usize = (task_id / max).try_into()?;
        let task_id_split2: usize = (task_id % max).try_into()?;
        cache_client.setbit(&format!("{cache_key}:1"), task_id_split1, status).await?;
        cache_client.setbit(&format!("{cache_key}:2"), task_id_split2, status).await?;
        Ok(())
    }

    pub async fn check_status(cache_key: &str, task_id: i64, cache_client: &TardisCacheClient) -> TardisResult<bool> {
        let max: i64 = u32::MAX.into();
        let task_id_split1: usize = (task_id / max).try_into()?;
        let task_id_split2: usize = (task_id % max).try_into()?;
        let result1 = cache_client.getbit(&format!("{cache_key}:1"), task_id_split1).await?;
        let result2 = cache_client.getbit(&format!("{cache_key}:2"), task_id_split2).await?;
        Ok(result1 && result2)
    }

    pub async fn set_task_process_data(cache_key: &str, task_id: i64, data: Value, funs: &TardisFunsInst) -> TardisResult<()> {
        let cache_client = funs.cache();
        let cache_key = format!("{}:{}", cache_key, task_id);
        cache_client.set_ex(&cache_key, &TardisFuns::json.json_to_string(data)?, TASK_PROCESSOR_DATA_EX).await?;
        Ok(())
    }

    pub async fn get_task_process_data(cache_key: &str, task_id: i64, funs: &TardisFunsInst) -> TardisResult<Value> {
        let cache_client = funs.cache();
        let cache_key = format!("{}:{}", cache_key, task_id);
        let result = cache_client.get(&cache_key).await?;
        if let Some(result) = result {
            Ok(TardisFuns::json.str_to_obj(&result)?)
        } else {
            Ok(Value::Null)
        }
    }

    pub async fn execute_task<P, T>(cache_key: &str, process: P, funs: &TardisFunsInst) -> TardisResult<i64>
    where
        P: FnOnce(i64) -> T + Send + Sync + 'static,
        T: Future<Output = TardisResult<()>> + Send + 'static,
    {
        let task_id = TaskProcessor::init_task(cache_key, funs.cache()).await?;
        let cache_client = funs.cache();
        let cache_key = cache_key.to_string();
        let handle = tardis::tokio::spawn(async move {
            let result = process(task_id).await;
            match result {
                Ok(_) => match TaskProcessor::set_status(&cache_key, task_id, true, cache_client).await {
                    Ok(_) => {}
                    Err(e) => log::error!("Asynchronous task [{}] process error:{:?}", task_id, e),
                },
                Err(e) => {
                    log::error!("Asynchronous task [{}] process error:{:?}", task_id, e);
                }
            }
        });
        TASK_HANDLE.write().await.insert(task_id, handle);
        Ok(task_id)
    }

    pub async fn execute_task_external(cache_key: &str, funs: &TardisFunsInst) -> TardisResult<i64> {
        let task_id = TaskProcessor::init_task(cache_key, funs.cache()).await?;
        Ok(task_id)
    }

    pub async fn execute_task_with_ctx<P, T>(cache_key: &str, process: P, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<i64>
    where
        P: FnOnce(i64) -> T + Send + Sync + 'static,
        T: Future<Output = TardisResult<()>> + Send + 'static,
    {
        let task_id = Self::execute_task(cache_key, process, funs).await?;
        if let Some(exist_task_ids) = ctx.get_ext(TASK_IN_CTX_FLAG).await? {
            ctx.add_ext(TASK_IN_CTX_FLAG, &format!("{exist_task_ids},{task_id}")).await?;
        } else {
            ctx.add_ext(TASK_IN_CTX_FLAG, &task_id.to_string()).await?;
        }
        Ok(task_id)
    }

    pub async fn stop_task(cache_key: &str, task_id: i64, funs: &TardisFunsInst) -> TardisResult<()> {
        if TaskProcessor::check_status(cache_key, task_id, funs.cache()).await? {
            TASK_HANDLE.write().await.remove(&task_id);
        } else {
            funs.mq()
                .publish(
                    &funs.rbum_conf_task_mq_topic_event(),
                    TardisFuns::json.obj_to_string(&TaskEventMessage {
                        task_id,
                        operate: TaskOperate::Stop,
                        cache_key: cache_key.to_string(),
                    })?,
                    &HashMap::new(),
                )
                .await?;
        }
        Ok(())
    }

    pub async fn stop_task_external(cache_key: &str, task_id: i64, funs: &TardisFunsInst) -> TardisResult<()> {
        match TaskProcessor::set_status(cache_key, task_id, true, funs.cache()).await {
            Ok(_) => {}
            Err(e) => log::error!("Asynchronous task [{}] stop error:{:?}", task_id, e),
        }
        Ok(())
    }

    pub async fn do_stop_task(cache_key: &str, task_id: i64, funs: &TardisFunsInst) -> TardisResult<()> {
        match TASK_HANDLE.write().await.remove(&task_id) {
            Some(handle) => {
                handle.abort();
                match TaskProcessor::set_status(cache_key, task_id, true, funs.cache()).await {
                    Ok(_) => {}
                    Err(e) => log::error!("Asynchronous task [{}] stop error:{:?}", task_id, e),
                }
                Ok(())
            }
            None => Err(TardisError::bad_request("task not found,may task is end", "400-stop-task-error")),
        }
    }

    pub async fn add_notify_event(table_name: &str, operate: &str, record_id: &str, ctx: &TardisContext) -> TardisResult<()> {
        ctx.add_ext(
            &format!("{}{}", NOTIFY_EVENT_IN_CTX_FLAG, TardisFuns::field.nanoid()),
            &tardis::TardisFuns::json.obj_to_string(&NotifyEventMessage {
                table_name: table_name.to_string(),
                operate: operate.to_string(),
                record_id: record_id.to_string(),
            })?,
        )
        .await
    }

    pub async fn get_task_id_with_ctx(ctx: &TardisContext) -> TardisResult<Option<String>> {
        ctx.get_ext(TASK_IN_CTX_FLAG).await
    }

    pub async fn get_notify_event_with_ctx(ctx: &TardisContext) -> TardisResult<Option<Vec<NotifyEventMessage>>> {
        let notify_events = ctx.ext.read().await;
        let notify_events = notify_events
            .iter()
            .filter(|(k, _)| k.starts_with(NOTIFY_EVENT_IN_CTX_FLAG))
            .map(|(_, v)| TardisFuns::json.str_to_obj::<NotifyEventMessage>(v).unwrap())
            .collect::<Vec<_>>();
        if notify_events.is_empty() {
            Ok(None)
        } else {
            Ok(Some(notify_events))
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NotifyEventMessage {
    pub table_name: String,
    pub operate: String,
    pub record_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TaskEventMessage {
    pub cache_key: String,
    pub task_id: i64,
    pub operate: TaskOperate,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum TaskOperate {
    Start,
    Stop,
}
