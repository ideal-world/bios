use std::{sync::Arc, time::Duration};

use service::ScheduleJobService;
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    log::{error, info, warn},
    tardis_static, TardisFuns, TardisFunsInst,
};

use crate::{dto::schedule_job_dto::ScheduleJob, schedule_constants::DOMAIN_CODE};
use event::{EventComponent, SpiLog};
use repo::{Repository, SpiKv};

pub mod event;
pub mod repo;
pub mod service;

tardis_static! {
    service: ScheduleJobService<SpiKv, SpiLog>;
}

pub async fn add_or_modify(add_or_modify: ScheduleJob, funs: TardisFunsInst, ctx: TardisContext) -> TardisResult<()> {
    let funs = Arc::new(funs);
    let ctx = Arc::new(ctx);
    let repo = repo::SpiKv::from_context(funs.clone(), ctx.clone());
    let event = event::SpiLog::from_context(funs, ctx);
    service().set_job(add_or_modify, repo, event).await
}

pub async fn delete(code: &str, funs: TardisFunsInst, ctx: TardisContext) -> TardisResult<()> {
    let funs = Arc::new(funs);
    let ctx = Arc::new(ctx);
    let repo = repo::SpiKv::from_context(funs.clone(), ctx.clone());
    let event = event::SpiLog::from_context(funs, ctx);
    service().delete_job(code, repo, event).await
}


pub(crate) fn init() {
    tardis::tokio::spawn(async move {
        // 这里初始化服务
        let service = service();

        let funs = Arc::new(TardisFuns::inst(DOMAIN_CODE, None));

        let mut interval = tardis::tokio::time::interval(Duration::from_secs(5));
        let mut retry_time = 0;
        let max_retry_time = 5;
        let repo = SpiKv::from_context(funs.clone(), TardisContext::default());
        let spi_log = SpiLog::from_context(funs, Arc::new(TardisContext::default()));
        // 等待webserver启动
        loop {
            if !TardisFuns::web_server().is_running().await {
                tardis::tokio::task::yield_now().await;
            } else {
                break;
            }
        }
        // 五秒钟轮询一次
        loop {
            // 从仓库同步所有任务
            if let Ok(jobs) = repo.get_all().await {
                for job in jobs {
                    if let Ok(task) = service.make_task(&job, spi_log.clone()) {
                        service.local_set_job(&job.code, task).await;
                    } else {
                        error!("fail to create task for job {job:?}");
                    }
                }
                info!("synced all jobs from kv");
                break;
            } else {
                warn!("encounter an error while init schedule middlewares: fail to find job {retry_time}/{max_retry_time}");
                retry_time += 1;
                if retry_time >= max_retry_time {
                    error!("fail to sync jobs from kv, schedule running without history jobs");
                    break;
                }
            }
            interval.tick().await;
        }
    });
}
