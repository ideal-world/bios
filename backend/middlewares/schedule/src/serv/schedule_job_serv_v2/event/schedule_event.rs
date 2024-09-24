use std::sync::Arc;

use bios_sdk_invoke::clients::{
    spi_log_client::LogItemAddReq,
};
use serde::{Deserialize, Serialize};
use tardis::{
    basic::{dto::TardisContext, error::TardisError, result::TardisResult},
    chrono::Utc,
    futures::Stream,
    log::error,
    tokio, TardisFunsInst,
};

use crate::schedule_constants::*;

use super::{EventComponent, ScheduleEvent};
#[derive(Clone)]
pub struct ScheduleEventCenter {
    funs: Arc<TardisFunsInst>,
    ctx: Arc<TardisContext>,
}

const SCHEDULE_AVATAR: &str = "schedule";
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AddTaskEvent {
    pub code: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeleteTaskEvent {
    pub code: String,
}
impl Event for AddTaskEvent {
    const CODE: &'static str = "schedule/add_task";
}
impl Event for DeleteTaskEvent {
    const CODE: &'static str = "schedule/delete_task";
}

impl EventComponent for ScheduleEventCenter {
    fn from_context(funs: impl Into<std::sync::Arc<tardis::TardisFunsInst>>, ctx: impl Into<std::sync::Arc<tardis::basic::dto::TardisContext>>) -> Self {
        Self {
            funs: funs.into(),
            ctx: ctx.into(),
        }
    }

    fn notify_create(&self, code: &str) {
        if let Some(ec) = BiosEventCenter::pub_sub() {
            let code = code.to_owned();
            tokio::spawn(async move {
                let _ = ec.publish(AddTaskEvent { code }.with_source(SCHEDULE_AVATAR)).await;
            });
        }
    }

    fn notify_delete(&self, code: &str) {
        if let Some(ec) = BiosEventCenter::pub_sub() {
            let code = code.to_owned();
            tokio::spawn(async move {
                let _ = ec.publish(DeleteTaskEvent { code }.with_source(SCHEDULE_AVATAR)).await;
            });
        }
    }

    fn notify_execute_start(&self, code: &str) {
        if let Some(ec) = BiosEventCenter::worker_queue() {
            let funs = self.funs.clone();
            let ctx = self.ctx.clone();
            let code = code.to_string();
            let _handle = tokio::spawn(async move {
                let result = ec
                    .publish(
                        LogItemAddReq {
                            tag: TASK_TAG.to_string(),
                            content: "start request".into(),
                            key: Some(code.to_string()),
                            op: Some(OP_EXECUTE_START.to_string()),
                            ts: Some(Utc::now()),
                            ..Default::default()
                        }
                        .inject_context(&funs, &ctx),
                    )
                    .await;
                if let Err(e) = result {
                    error!("notify_create error: {:?}", e);
                }
            });
        }
    }

    fn notify_execute_end(&self, code: &str, message: String, ext: tardis::serde_json::Value) {
        if let Some(ec) = BiosEventCenter::worker_queue() {
            let funs = self.funs.clone();
            let ctx = self.ctx.clone();
            let code = code.to_string();
            let _handle = tokio::spawn(async move {
                let result = ec
                    .publish(
                        LogItemAddReq {
                            tag: TASK_TAG.to_string(),
                            content: tardis::serde_json::Value::Null,
                            ext: Some(ext),
                            key: Some(code.to_string()),
                            op: Some(OP_EXECUTE_END.to_string()),
                            ts: Some(Utc::now()),
                            msg: Some(message),
                            ..Default::default()
                        }
                        .inject_context(&funs, &ctx),
                    )
                    .await;
                if let Err(e) = result {
                    error!("notify_create error: {:?}", e);
                }
            });
        }
    }

    fn create_event_stream() -> impl tardis::futures::Stream<Item = super::ScheduleEvent> + Send {
        EventCenterBasedEventStream::new()
    }
}

pub struct EventCenterBasedEventStream {
    event_rx: tokio::sync::mpsc::Receiver<super::ScheduleEvent>,
}

impl EventCenterBasedEventStream {
    pub fn new() -> Self {
        let (event_tx, event_rx) = tokio::sync::mpsc::channel(100);
        if let Some(ec) = BiosEventCenter::pub_sub() {
            {
                let event_tx = event_tx.clone();
                ec.subscribe(move |AddTaskEvent { code }: AddTaskEvent| {
                    let event_tx = event_tx.clone();
                    async move {
                        event_tx.send(ScheduleEvent::JustCreate { code }).await.map_err(|_| TardisError::internal_error("fail to send out event", ""))?;
                        TardisResult::Ok(())
                    }
                });
            }
            {
                let event_tx = event_tx.clone();
                ec.subscribe(move |DeleteTaskEvent { code }: DeleteTaskEvent| {
                    let event_tx = event_tx.clone();
                    async move {
                        event_tx.send(ScheduleEvent::JustCreate { code }).await.map_err(|_| TardisError::internal_error("fail to send out event", ""))?;
                        TardisResult::Ok(())
                    }
                });
            }
        }
        EventCenterBasedEventStream { event_rx }
    }
}

impl Stream for EventCenterBasedEventStream {
    type Item = super::ScheduleEvent;

    fn poll_next(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
        let mut this = std::pin::pin!(self);
        this.event_rx.poll_recv(cx)
    }
}
