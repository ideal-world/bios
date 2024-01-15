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
    TardisFuns, TardisFunsInst,
};

lazy_static! {
    static ref TASK_HANDLE: Arc<RwLock<HashMap<i64, JoinHandle<()>>>> = Arc::new(RwLock::new(HashMap::new()));
}
const TASK_IN_CTX_FLAG: &str = "task_id";
const NOTIFY_EVENT_IN_CTX_FLAG: &str = "notify";
const TASK_PROCESSOR_DATA_EX: u64 = 60 * 60 * 24;
const EVENT_EXECUTE_TASK_EXTERNAL: &str = "execute_task_external";
const EVENT_TASK_STATUS_EXTERNAL: &str = "task_status_external";
const EVENT_SET_TASK_PROCESS_DATA_EXTERNAL: &str = "set_task_process_data";

pub struct TaskProcessor;

impl TaskProcessor {
    pub async fn init_task(cache_key: &str, cache_client: &TardisCacheClient) -> TardisResult<i64> {
        //todo change to SnowFlake or other distributed ID generator
        let task_id = Local::now().timestamp_nanos_opt().expect("maybe in 23rd centery");
        let max: i64 = u32::MAX.into();
        let task_id_split1: usize = (task_id / max).try_into()?;
        let task_id_split2: usize = (task_id % max).try_into()?;
        cache_client.setbit(&format!("{cache_key}:1"), task_id_split1, false).await?;
        cache_client.setbit(&format!("{cache_key}:2"), task_id_split2, false).await?;
        Ok(task_id)
    }

    pub async fn init_task_external(cache_key: &str, task_id: i64, cache_client: &TardisCacheClient) -> TardisResult<i64> {
        //todo change to SnowFlake or other distributed ID generator
        let max: i64 = u32::MAX.into();
        let task_id_split1: usize = (task_id / max).try_into()?;
        let task_id_split2: usize = (task_id % max).try_into()?;
        cache_client.setbit(&format!("{cache_key}:1"), task_id_split1, false).await?;
        cache_client.setbit(&format!("{cache_key}:2"), task_id_split2, false).await?;
        Ok(task_id)
    }

    pub async fn set_status(
        cache_key: &str,
        task_id: i64,
        status: bool,
        cache_client: &TardisCacheClient,
        ws_client: Option<TardisWSClient>,
        from_avatar: String,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let max: i64 = u32::MAX.into();
        let task_id_split1: usize = (task_id / max).try_into()?;
        let task_id_split2: usize = (task_id % max).try_into()?;
        cache_client.setbit(&format!("{cache_key}:1"), task_id_split1, status).await?;
        cache_client.setbit(&format!("{cache_key}:2"), task_id_split2, status).await?;
        Self::send_event(
            ws_client,
            TaskWsEventReq {
                task_id,
                data: Value::Bool(status),
                msg: format!("task status: {}", status),
            },
            from_avatar,
            Some(EVENT_TASK_STATUS_EXTERNAL.to_owned()),
            ctx,
        )
        .await?;
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

    pub async fn set_task_process_data(
        cache_key: &str,
        task_id: i64,
        data: Value,
        ws_client: Option<TardisWSClient>,
        from_avatar: String,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let cache_client = funs.cache();
        let cache_key = format!("{}:{}", cache_key, task_id);
        cache_client.set_ex(&cache_key, &TardisFuns::json.json_to_string(data.clone())?, TASK_PROCESSOR_DATA_EX).await?;
        Self::send_event(
            ws_client,
            TaskWsEventReq {
                task_id,
                data: data.clone(),
                msg: format!("set task process: {}", &TardisFuns::json.json_to_string(data)?),
            },
            from_avatar,
            Some(EVENT_SET_TASK_PROCESS_DATA_EXTERNAL.to_owned()),
            ctx,
        )
        .await?;
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

    pub async fn execute_task<P, T>(
        cache_key: &str,
        process: P,
        ws_client: Option<TardisWSClient>,
        from_avatar: String,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<i64>
    where
        P: FnOnce(i64) -> T + Send + Sync + 'static,
        T: Future<Output = TardisResult<()>> + Send + 'static,
    {
        let task_id = TaskProcessor::init_task(cache_key, &funs.cache()).await?;
        let cache_client = funs.cache();
        let cache_key = cache_key.to_string();
        let ctx_clone = ctx.clone();
        let handle = tardis::tokio::spawn(async move {
            let result = process(task_id).await;
            match result {
                Ok(_) => match TaskProcessor::set_status(&cache_key, task_id, true, &cache_client, ws_client, from_avatar, &ctx_clone).await {
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

    pub async fn execute_task_external(
        cache_key: &str,
        task_id: i64,
        ws_client: Option<TardisWSClient>,
        from_avatar: String,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<i64> {
        let task_id = TaskProcessor::init_task_external(cache_key, task_id, &funs.cache()).await?;
        Self::send_event(
            ws_client,
            TaskWsEventReq {
                task_id,
                data: ().into(),
                msg: "execute task start".to_owned(),
            },
            from_avatar,
            Some(EVENT_EXECUTE_TASK_EXTERNAL.to_owned()),
            ctx,
        )
        .await?;
        Ok(task_id)
    }

    pub async fn execute_task_with_ctx<P, T>(
        cache_key: &str,
        process: P,
        funs: &TardisFunsInst,
        ws_client: Option<TardisWSClient>,
        from_avatar: String,
        ctx: &TardisContext,
    ) -> TardisResult<i64>
    where
        P: FnOnce(i64) -> T + Send + Sync + 'static,
        T: Future<Output = TardisResult<()>> + Send + 'static,
    {
        let task_id = Self::execute_task(cache_key, process, ws_client.clone(), from_avatar.clone(), funs, &ctx.clone()).await?;
        Self::send_event(
            ws_client,
            TaskWsEventReq {
                task_id,
                data: ().into(),
                msg: "execute task start".to_owned(),
            },
            from_avatar,
            Some(EVENT_EXECUTE_TASK_EXTERNAL.to_owned()),
            ctx,
        )
        .await?;
        if let Some(exist_task_ids) = ctx.get_ext(TASK_IN_CTX_FLAG).await? {
            ctx.add_ext(TASK_IN_CTX_FLAG, &format!("{exist_task_ids},{task_id}")).await?;
        } else {
            ctx.add_ext(TASK_IN_CTX_FLAG, &task_id.to_string()).await?;
        }
        Ok(task_id)
    }

    pub async fn stop_task(cache_key: &str, task_id: i64, ws_client: Option<TardisWSClient>, from_avatar: String, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if TaskProcessor::check_status(cache_key, task_id, &funs.cache()).await? {
            TASK_HANDLE.write().await.remove(&task_id);
        } else {
            match TaskProcessor::do_stop_task(&cache_key, task_id, ws_client, from_avatar, funs, ctx).await {
                Ok(_) => {}
                Err(e) => log::error!("Asynchronous task [{}] process error:{:?}", task_id, e),
            }
        }
        Ok(())
    }

    pub async fn do_stop_task(
        cache_key: &str,
        task_id: i64,
        ws_client: Option<TardisWSClient>,
        from_avatar: String,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        if TASK_HANDLE.read().await.contains_key(&task_id) {
            match TASK_HANDLE.write().await.remove(&task_id) {
                Some(handle) => {
                    handle.abort();
                    match TaskProcessor::set_status(cache_key, task_id, true, &funs.cache(), ws_client, from_avatar, ctx).await {
                        Ok(_) => {}
                        Err(e) => log::error!("Asynchronous task [{}] stop error:{:?}", task_id, e),
                    }
                    Ok(())
                }
                None => Err(TardisError::bad_request("task not found,may task is end", "400-stop-task-error")),
            }
        } else {
            match TaskProcessor::set_status(cache_key, task_id, true, &funs.cache(), ws_client, from_avatar, ctx).await {
                Ok(_) => {}
                Err(e) => log::error!("Asynchronous task [{}] stop error:{:?}", task_id, e),
            }
            Ok(())
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

    pub async fn send_event(ws_client: Option<TardisWSClient>, msg: TaskWsEventReq, from_avatar: String, event: Option<String>, ctx: &TardisContext) -> TardisResult<()> {
        if let Some(ws_client) = ws_client {
            let req = TardisWebsocketReq {
                msg: tardis::serde_json::Value::String(TardisFuns::json.obj_to_string(&msg)?),
                to_avatars: Some(vec![format!("account/{}", ctx.owner)]),
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
    pub task_id: i64,
    pub data: Value,
    pub msg: String,
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
