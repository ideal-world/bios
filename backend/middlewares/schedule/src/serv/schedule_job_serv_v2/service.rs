use std::{collections::HashMap, marker::PhantomData, net::SocketAddr, sync::Arc};

use tardis::{
    basic::{dto::TardisContext, error::TardisError, result::TardisResult},
    chrono::TimeDelta,
    futures::StreamExt,
    log::{debug, error, trace},
    serde_json,
    tokio::sync::RwLock,
    TardisFuns, TardisFunsInst,
};
use tsuki_scheduler::{
    runtime::Tokio,
    schedule::{Cron, ScheduleDynBuilder},
    AsyncSchedulerClient, AsyncSchedulerRunner, Task, TaskUid,
};

use crate::{dto::schedule_job_dto::ScheduleJob, schedule_config::ScheduleConfig, schedule_constants::DOMAIN_CODE};

use super::{
    event::{self, EventComponent},
    repo::Repository,
};
#[derive(Clone)]
pub struct ScheduleJobService<R, E> {
    pub repository: PhantomData<fn(R)>,
    pub event: PhantomData<fn(E)>,
    pub local_cache: Arc<RwLock<HashMap<String, TaskUid>>>,
    pub client: AsyncSchedulerClient<Tokio>,
    pub funs: Arc<TardisFunsInst>,
}

impl<R, E> Default for ScheduleJobService<R, E>
where
    R: Repository,
    E: EventComponent,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<R, E> ScheduleJobService<R, E>
where
    R: Repository,
    E: EventComponent,
{
    pub fn new() -> Self {
        let local_cache = Arc::new(RwLock::new(HashMap::new()));
        let runner = AsyncSchedulerRunner::tokio();
        let client = runner.client();
        let funs = Arc::new(TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None));

        // 运行调度器
        tardis::tokio::spawn(async move { runner.run().await });

        // 监听事件
        {
            let this = Self {
                repository: PhantomData,
                event: PhantomData,
                local_cache: local_cache.clone(),
                client: client.clone(),
                funs: funs.clone(),
            };

            tardis::tokio::spawn(async move {
                let mut es = E::create_event_stream();
                let mut es = std::pin::pin!(es);
                let funs = Arc::new(TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None));
                let ctx = Arc::new(TardisContext::default());
                while let Some(event) = es.next().await {
                    match event {
                        event::ScheduleEvent::JustDelete { code } => {
                            debug!("[Bios.Schedule] event: delete {code} ");
                            this.local_delete_job(&code).await;
                        }
                        event::ScheduleEvent::JustCreate { code } => {
                            debug!("[Bios.Schedule] event: create {code} ");
                            let event_hub = E::from_context(funs.clone(), ctx.clone());
                            let repo = R::from_context(funs.clone(), ctx.clone());
                            let Ok(Some(job)) = repo.get_one(&code).await else { continue };
                            let Ok(task) = this.make_task(&job, event_hub) else { continue };
                            this.local_set_job(&code, task).await;
                        }
                    }
                }
            });
        }
        Self {
            repository: PhantomData,
            event: PhantomData,
            local_cache,
            client,
            funs,
        }
    }

    // 本地删除任务
    async fn local_delete_job(&self, code: &str) {
        let task_uid = self.local_cache.write().await.remove(code);
        if let Some(task_uid) = task_uid {
            self.client.remove_task(task_uid);
        }
    }

    // 本地设置任务
    pub(crate) async fn local_set_job(&self, code: &str, task: Task<Tokio>) {
        self.local_delete_job(code).await;
        let task_uid = TaskUid::uuid();
        self.local_cache.write().await.insert(code.to_string(), task_uid);
        self.client.add_task(task_uid, task);
    }

    /// 生成分布式锁的key
    fn gen_distributed_lock_key(code: &str, config: &ScheduleConfig) -> String {
        format!("{}{}", config.distributed_lock_key_prefix, code)
    }

    /// 创建任务
    pub(crate) fn make_task(&self, job: &ScheduleJob, event: E) -> TardisResult<Task<Tokio>> {
        let schedule_config = self.funs.conf::<ScheduleConfig>();
        let callback_req = job.build_request()?;
        let code = job.code.to_string();
        // 分布式锁的key
        let lock_key = Self::gen_distributed_lock_key(&code, &schedule_config);

        let distributed_lock_expire_sec = schedule_config.distributed_lock_expire_sec;
        let enable_time = job.enable_time;
        let disable_time = job.disable_time;

        // 生成任务
        let mut schedule_builder = job.cron.iter().filter_map(|cron| Cron::local_from_cron_expr(cron).ok()).fold(ScheduleDynBuilder::default(), ScheduleDynBuilder::or);
        if let Some(enable_time) = enable_time {
            schedule_builder = schedule_builder.after(enable_time);
        }
        if let Some(disable_time) = disable_time {
            schedule_builder = schedule_builder.before(disable_time);
        }
        // 一个节点下一分钟内只能执行一次
        schedule_builder = schedule_builder.throttling(TimeDelta::minutes(1));
        let task = Task::tokio(schedule_builder, move || {
            let callback_req = callback_req.try_clone().expect("body should be a string");
            let code = code.clone();
            let lock_key = lock_key.clone();
            let event = event.clone();
            async move {
                let cache_client = TardisFuns::cache();
                // about set and setnx, see:
                // 1. https://redis.io/commands/set/
                // 2. https://redis.io/commands/setnx/
                // At Redis version 2.6.12, setnx command is regarded as deprecated. see: https://redis.io/commands/setnx/
                // "executing" could be any string now, it's just a placeholder
                match cache_client.set_nx(&lock_key, "executing").await {
                    Ok(true) => {
                        // safety: it's ok to unwrap in this closure, scheduler will restart this job when after panic
                        let Ok(()) = cache_client.expire(&lock_key, distributed_lock_expire_sec as i64).await else {
                            return;
                        };
                        trace!("executing schedule task {code}");
                        // 1. write log exec start
                        event.notify_execute_start(&code);
                        // 2. request webhook
                        match TardisFuns::web_client().raw().execute(callback_req).await {
                            Ok(resp) => {
                                let status_code = resp.status();
                                let remote_addr = resp.remote_addr().as_ref().map(SocketAddr::to_string);
                                let response_header: HashMap<String, String> = resp
                                    .headers()
                                    .into_iter()
                                    .filter_map(|(k, v)| {
                                        let v = v.to_str().ok()?.to_string();
                                        Some((k.to_string(), v))
                                    })
                                    .collect();
                                let ext = serde_json::json! {
                                    {
                                        "remote_addr": remote_addr,
                                        "status_code": status_code.to_string(),
                                        "headers": response_header
                                    }
                                };
                                let content = resp.text().await.unwrap_or_default();
                                // 3.1. write log exec end
                                event.notify_execute_end(&code, content, ext);
                            }
                            Err(e) => {
                                event.notify_execute_end(&code, e.to_string(), serde_json::Value::Null);
                            }
                        }
                        trace!("executed schedule task {code}");
                    }
                    Ok(false) => {
                        trace!("schedule task {} is executed by other nodes, skip", code);
                    }
                    Err(e) => {
                        error!("cannot set lock to schedule task {code}, error: {e}");
                    }
                }
            }
        });

        Ok(task)
    }

    pub async fn set_job(&self, job: ScheduleJob, repo: R, event: E) -> Result<(), TardisError> {
        let code = job.code.to_string();
        // 如果存在，先删除
        self.local_delete_job(&code).await;

        // 生成任务
        let task = self.make_task(&job, event.clone())?;

        // 写入仓库
        repo.create(&job).await?;

        // 写入调度器
        self.local_set_job(&code, task).await;

        // 通知创建成功
        event.notify_create(&code);
        Ok(())
    }

    pub async fn delete_job(&self, code: &str, repo: R, event: E) -> Result<(), TardisError> {
        // 从仓库删除
        repo.delete(code).await?;

        // 删除调度器
        self.local_delete_job(code).await;

        // 通知删除成功
        event.notify_delete(code);
        Ok(())
    }
}
