use std::{sync::Arc, task::ready, time::Duration};

use bios_sdk_invoke::clients::spi_log_client::{LogItemAddReq, LogItemFindReq, SpiLogClient};
use tardis::{
    basic::dto::TardisContext,
    chrono::Utc,
    futures::Stream,
    log::{self as tracing, debug, error, instrument},
    tokio, TardisFuns, TardisFunsInst,
};

use crate::schedule_constants::*;

use super::EventComponent;

#[derive(Clone)]
pub struct SpiLog {
    funs: Arc<TardisFunsInst>,
    ctx: Arc<TardisContext>,
    _client: SpiLogClient,
}

impl EventComponent for SpiLog {
    fn from_context(funs: impl Into<Arc<TardisFunsInst>>, ctx: impl Into<Arc<TardisContext>>) -> Self {
        Self {
            funs: funs.into(),
            ctx: ctx.into(),
            _client: SpiLogClient,
        }
    }
    #[instrument(skip(self))]
    fn notify_create(&self, code: &str) {
        let funs = self.funs.clone();
        let ctx = self.ctx.clone();
        let code = code.to_string();
        let _handle = tokio::spawn(async move {
            let result = SpiLogClient::add(
                LogItemAddReq {
                    tag: JOB_TAG.to_string(),
                    content: "add job".into(),
                    key: Some(code.to_string()),
                    op: Some(OP_ADD.to_string()),
                    ts: Some(Utc::now()),
                    ..Default::default()
                },
                &funs,
                &ctx,
            )
            .await;
            if let Err(e) = result {
                error!("notify_create error: {:?}", e);
            }
        });
    }

    #[instrument(skip(self))]
    fn notify_delete(&self, code: &str) {
        let funs = self.funs.clone();
        let ctx = self.ctx.clone();
        let code = code.to_string();
        let _handle = tokio::spawn(async move {
            let result = SpiLogClient::add(
                LogItemAddReq {
                    tag: JOB_TAG.to_string(),
                    content: "delete job".into(),
                    key: Some(code.to_string()),
                    op: Some(OP_DELETE.to_string()),
                    ts: Some(Utc::now()),
                    ..Default::default()
                },
                &funs,
                &ctx,
            )
            .await;
            if let Err(e) = result {
                error!("notify_create error: {:?}", e);
            }
        });
    }

    #[instrument(skip(self))]
    fn notify_execute_start(&self, code: &str) {
        let funs = self.funs.clone();
        let ctx = self.ctx.clone();
        let code = code.to_string();
        let _handle = tokio::spawn(async move {
            let result = SpiLogClient::add(
                LogItemAddReq {
                    tag: TASK_TAG.to_string(),
                    content: "start request".into(),
                    key: Some(code.to_string()),
                    op: Some(OP_EXECUTE_START.to_string()),
                    ts: Some(Utc::now()),
                    ..Default::default()
                },
                &funs,
                &ctx,
            )
            .await;
            if let Err(e) = result {
                error!("notify_create error: {:?}", e);
            }
        });
    }

    #[instrument(skip(self))]
    fn notify_execute_end(&self, code: &str, message: String, ext: tardis::serde_json::Value) {
        let funs = self.funs.clone();
        let ctx = self.ctx.clone();
        let code = code.to_string();
        let _handle = tokio::spawn(async move {
            let result = SpiLogClient::add(
                LogItemAddReq {
                    tag: TASK_TAG.to_string(),
                    content: tardis::serde_json::Value::Null,
                    ext: Some(ext),
                    key: Some(code.to_string()),
                    op: Some(OP_EXECUTE_END.to_string()),
                    ts: Some(Utc::now()),
                    msg: Some(message),
                    ..Default::default()
                },
                &funs,
                &ctx,
            )
            .await;

            if let Err(e) = result {
                error!("notify_create error: {:?}", e);
            }
        });
    }

    fn create_event_stream() -> impl Stream<Item = super::ScheduleEvent> {
        let funs = TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None);
        let ctx = TardisContext::default();
        LogScanBasedEventStream::new(funs.into(), ctx.into(), Duration::from_secs(30))
    }
}

pub struct LogScanBasedEventStream {
    scan_handle: tokio::task::JoinHandle<()>,
    event_rx: tokio::sync::mpsc::Receiver<super::ScheduleEvent>,
    _client: SpiLogClient,
}

impl LogScanBasedEventStream {
    pub fn new(funs: Arc<TardisFunsInst>, ctx: Arc<TardisContext>, scan_period: Duration) -> Self {
        let (event_tx, event_rx) = tokio::sync::mpsc::channel(100);
        let scan_handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(scan_period);
            let mut prev_scan = Utc::now();
            loop {
                let this_scan = Utc::now();
                let find_req = LogItemFindReq {
                    tag: JOB_TAG.to_string(),
                    ops: Some(vec![OP_ADD.to_string(), OP_DELETE.to_string()]),
                    ts_start: Some(prev_scan),
                    ts_end: Some(this_scan),
                    page_number: 1,
                    page_size: u16::MAX,
                    ..Default::default()
                };
                let result = SpiLogClient::find(find_req, &funs, &ctx).await;
                if let Ok(Some(page)) = result {
                    debug!("scan log page: {:?}", page);
                    for item in page.records {
                        let event = match item.op.as_str() {
                            OP_ADD => super::ScheduleEvent::JustCreate { code: item.key.clone() },
                            OP_DELETE => super::ScheduleEvent::JustDelete { code: item.key.clone() },
                            _ => continue,
                        };
                        if let Err(e) = event_tx.send(event).await {
                            error!("send event error: {:?}", e);
                            break;
                        }
                    }
                    prev_scan = this_scan;
                }
                interval.tick().await;
            }
        });
        Self {
            scan_handle,
            event_rx,
            _client: SpiLogClient,
        }
    }
}

impl Stream for LogScanBasedEventStream {
    type Item = super::ScheduleEvent;

    fn poll_next(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
        let mut this = std::pin::pin!(self);
        let message = ready!(this.event_rx.poll_recv(cx));
        match message {
            Some(message) => std::task::Poll::Ready(Some(message)),
            None => {
                if !this.scan_handle.is_finished() {
                    this.scan_handle.abort();
                }
                std::task::Poll::Ready(None)
            }
        }
    }
}
