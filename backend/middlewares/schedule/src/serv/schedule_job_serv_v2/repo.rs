use std::{future::Future, sync::Arc};

use tardis::{
    basic::{dto::TardisContext, error::TardisError},
    web::web_resp::TardisPage,
    TardisFunsInst,
};

use crate::dto::schedule_job_dto::ScheduleJob;
mod spi_kv;
pub use spi_kv::*;
pub trait Repository: Send + Sync + Clone + 'static {
    fn from_context(funs: impl Into<Arc<TardisFunsInst>>, ctx: impl Into<Arc<TardisContext>>) -> Self;
    fn get_one(&self, code: &str) -> impl Future<Output = Result<Option<ScheduleJob>, TardisError>> + Send;
    fn get_all(&self) -> impl Future<Output = Result<Vec<ScheduleJob>, TardisError>> + Send;
    fn get_paged(&self, page: u32, size: u16) -> impl Future<Output = Result<TardisPage<ScheduleJob>, TardisError>> + Send;
    fn create(&self, req: &ScheduleJob) -> impl Future<Output = Result<(), TardisError>> + Send;
    fn update(&self, req: &ScheduleJob) -> impl Future<Output = Result<(), TardisError>> + Send;
    fn delete(&self, code: &str) -> impl Future<Output = Result<(), TardisError>> + Send;
}
