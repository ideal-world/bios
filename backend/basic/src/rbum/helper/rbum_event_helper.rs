use std::collections::HashMap;
use std::future::Future;

use serde::{Deserialize, Serialize};
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::chrono::Utc;
use tardis::{TardisFuns, TardisFunsInst};

use crate::rbum::rbum_config::RbumConfigApi;

const NOTIFY_EVENT_IN_CTX_FLAG: &str = "notify";

pub async fn try_notifies(event_messages: Vec<NotifyEventMessage>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    for event_message in event_messages {
        self::try_notify(&event_message.table_name, &event_message.operate, &event_message.record_id, funs, ctx).await?;
    }
    Ok(())
}

pub async fn try_notify<'a>(table_name: &str, operate: &str, record_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
    #[cfg(feature = "with-mq")]
    {
        if funs.rbum_conf_match_event(table_name, operate) {
            funs.mq()
                .publish(
                    &funs.rbum_conf_mq_topic_event(),
                    tardis::TardisFuns::json.obj_to_string(&RbumEventMessage {
                        table_name: table_name.to_string(),
                        operate: operate.to_string(),
                        operator: ctx.owner.clone(),
                        record_id: record_id.to_string(),
                        ts: Utc::now().timestamp_millis(),
                    })?,
                    &HashMap::new(),
                )
                .await?;
        }
        Ok(true)
    }
    #[cfg(not(feature = "with-mq"))]
    {
        Ok(false)
    }
}

pub async fn receive<F, T>(fun: F, funs: &TardisFunsInst) -> TardisResult<bool>
where
    F: Fn((HashMap<String, String>, String)) -> T + Send + Sync + 'static,
    T: Future<Output = TardisResult<()>> + Send + 'static,
{
    #[cfg(feature = "with-mq")]
    {
        funs.mq().subscribe(&funs.rbum_conf_mq_topic_event(), fun).await?;
        Ok(true)
    }
    #[cfg(not(feature = "with-mq"))]
    {
        Ok(false)
    }
}

pub fn parse_message(message: String) -> TardisResult<RbumEventMessage> {
    tardis::TardisFuns::json.str_to_obj::<RbumEventMessage>(&message)
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RbumEventMessage {
    pub table_name: String,
    pub operate: String,
    pub operator: String,
    pub record_id: String,
    pub ts: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NotifyEventMessage {
    pub table_name: String,
    pub operate: String,
    pub record_id: String,
}
