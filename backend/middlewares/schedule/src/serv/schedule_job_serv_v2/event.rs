use std::sync::Arc;
mod spi_log;
mod schedule_event;
pub use spi_log::*;
use tardis::{basic::dto::TardisContext, futures::Stream, serde_json::Value, TardisFunsInst};

pub trait EventComponent: Send + Sync + Clone + 'static {
    fn from_context(funs: impl Into<Arc<TardisFunsInst>>, ctx: impl Into<Arc<TardisContext>>) -> Self;
    fn notify_create(&self, code: &str);
    fn notify_update(&self, code: &str) {
        self.notify_create(code);
    }
    fn notify_delete(&self, code: &str);
    fn notify_execute_start(&self, code: &str);
    fn notify_execute_end(&self, code: &str, message: String, ext: Value);
    fn create_event_stream() -> impl Stream<Item = ScheduleEvent> + Send;
}

pub enum ScheduleEvent {
    JustDelete { code: String },
    JustCreate { code: String },
}
