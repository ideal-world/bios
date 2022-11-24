use std::future::Future;

use serde::{Deserialize, Serialize};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    cache::cache_client::TardisCacheClient,
    chrono::Local,
    log, TardisFuns, TardisFunsInst,
};

use crate::rbum::helper::rbum_event_helper;

const TASK_IN_CTX_FLAG: &str = "task_id";
const NOTIFY_EVENT_IN_CTX_FLAG: &str = "notify";

pub struct TaskProcessor;

impl TaskProcessor {
    pub async fn init_task(cache_key: &str, cache_client: &TardisCacheClient) -> TardisResult<i64> {
        let task_id = Local::now().timestamp_nanos();
        let max: i64 = u32::MAX.into();
        let task_id_split1: usize = (task_id / max).try_into()?;
        let task_id_split2: usize = (task_id % max).try_into()?;
        cache_client.setbit(&format!("{}:1", cache_key), task_id_split1, false).await?;
        cache_client.setbit(&format!("{}:2", cache_key), task_id_split2, false).await?;
        Ok(task_id)
    }

    pub async fn set_status(cache_key: &str, task_id: i64, status: bool, cache_client: &TardisCacheClient) -> TardisResult<()> {
        let max: i64 = u32::MAX.into();
        let task_id_split1: usize = (task_id / max).try_into()?;
        let task_id_split2: usize = (task_id % max).try_into()?;
        cache_client.setbit(&format!("{}:1", cache_key), task_id_split1, status).await?;
        cache_client.setbit(&format!("{}:2", cache_key), task_id_split2, status).await?;
        Ok(())
    }

    pub async fn check_status(cache_key: &str, task_id: i64, cache_client: &TardisCacheClient) -> TardisResult<bool> {
        let max: i64 = u32::MAX.into();
        let task_id_split1: usize = (task_id / max).try_into()?;
        let task_id_split2: usize = (task_id % max).try_into()?;
        let result1 = cache_client.getbit(&format!("{}:1", cache_key), task_id_split1).await?;
        let result2 = cache_client.getbit(&format!("{}:2", cache_key), task_id_split2).await?;
        Ok(result1 && result2)
    }

    pub async fn execute_task<P, T>(cache_key: &str, process: P, funs: &TardisFunsInst) -> TardisResult<i64>
    where
        P: FnOnce() -> T + Send + Sync + 'static,
        T: Future<Output = TardisResult<()>> + Send + 'static,
    {
        let task_id = TaskProcessor::init_task(cache_key, funs.cache()).await?;
        let cache_client = funs.cache();
        let cache_key = cache_key.to_string();
        tardis::tokio::spawn(async move {
            let result = process().await;
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
        Ok(task_id)
    }

    pub async fn execute_task_with_ctx<P, T>(cache_key: &str, process: P, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()>
    where
        P: FnOnce() -> T + Send + Sync + 'static,
        T: Future<Output = TardisResult<()>> + Send + 'static,
    {
        let task_id = Self::execute_task(cache_key, process, funs).await?;
        if let Some(exist_task_ids) = ctx.get_ext(TASK_IN_CTX_FLAG)? {
            ctx.add_ext(TASK_IN_CTX_FLAG, &format!("{},{}", exist_task_ids, task_id))
        } else {
            ctx.add_ext(TASK_IN_CTX_FLAG, &task_id.to_string())
        }
    }

    pub fn add_notify_event(table_name: &str, operate: &str, record_id: &str, ctx: &TardisContext) -> TardisResult<()> {
        ctx.add_ext(
            &format!("{}{}", NOTIFY_EVENT_IN_CTX_FLAG, TardisFuns::field.nanoid()),
            &tardis::TardisFuns::json.obj_to_string(&NotifyEventMessage {
                table_name: table_name.to_string(),
                operate: operate.to_string(),
                record_id: record_id.to_string(),
            })?,
        )
    }

    pub fn get_task_id_with_ctx(ctx: &TardisContext) -> TardisResult<Option<String>> {
        ctx.get_ext(TASK_IN_CTX_FLAG)
    }

    pub fn get_notify_event_with_ctx(ctx: &TardisContext) -> TardisResult<Option<Vec<NotifyEventMessage>>> {
        let notify_events = ctx.ext.read()?;
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
