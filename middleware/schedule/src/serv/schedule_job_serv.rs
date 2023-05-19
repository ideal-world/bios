use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use lazy_static::lazy_static;
use tardis::basic::dto::TardisContext;
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::cache::AsyncCommands;
use tardis::chrono::{self, Utc};
use tardis::db::sea_orm::prelude::Uuid;
use tardis::log::{error, info, trace};
use tardis::tokio::sync::RwLock;
use tardis::tokio::time;
use tardis::web::web_resp::TardisPage;
use tardis::{TardisFuns, TardisFunsInst};
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::dto::schedule_job_dto::{ScheduleJobAddOrModifyReq, ScheduleJobInfoResp, ScheduleJobKvSummaryResp, ScheduleTaskInfoResp, ScheduleTaskLogFindResp};
use crate::schedule_config::ScheduleConfig;
use crate::schedule_constants::KV_KEY_CODE;

lazy_static! {
    pub static ref TASK: Arc<RwLock<HashMap<String, ScheduleJobInfoResp>>> = Arc::new(RwLock::new(HashMap::new()));
}

/// global service instance
static mut MAYBE_GLOBAL_SERV: Option<OwnedScheduleTaskServ> = None;

/// get service instance without checking if it's initialized
/// # Safety
/// if called before init, this function will panic
unsafe fn service() -> OwnedScheduleTaskServ {
    MAYBE_GLOBAL_SERV.as_ref().cloned().expect("tring to get scheduler before it's initialized")
}

pub(crate) async fn add_or_modify(add_or_modify: ScheduleJobAddOrModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let log_url = &funs.conf::<ScheduleConfig>().log_url;
    let kv_url = &funs.conf::<ScheduleConfig>().kv_url;
    let code = &add_or_modify.code.0;
    let spi_ctx = TardisContext {
        owner: funs.conf::<ScheduleConfig>().spi_app_id.clone(),
        ..ctx.clone()
    };
    let headers = Some(vec![(
        "Tardis-Context".to_string(),
        TardisFuns::crypto.base64.encode(&TardisFuns::json.obj_to_string(&spi_ctx).unwrap()),
    )]);
    // if exist delete it first
    {
        if let Some(_uuid) = unsafe { service() }.code_uuid.write().await.get(code) {
            self::delete(code, funs, ctx).await?;
        }
    }
    // log this operation
    funs.web_client()
        .post_obj_to_str(
            &format!("{log_url}/ci/item"),
            &HashMap::from([
                ("tag", "schedule_job"),
                ("content", "add job"),
                ("key", code),
                ("op", "add"),
                ("ts", &Utc::now().to_rfc3339()),
            ]),
            headers.clone(),
        )
        .await?;
    // put schedual-task to kv cache
    funs.web_client()
        .put_obj_to_str(
            &format!("{kv_url}/ci/item"),
            &HashMap::from([("key", format!("{KV_KEY_CODE}{code}")), ("value", TardisFuns::json.obj_to_string(&add_or_modify)?)]),
            headers.clone(),
        )
        .await?;
    // add schedual-task to scheduler
    ScheduleTaskServ::add(log_url, add_or_modify, funs.conf::<ScheduleConfig>()).await?;
    Ok(())
}

pub(crate) async fn delete(code: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let log_url = &funs.conf::<ScheduleConfig>().log_url;
    let kv_url = &funs.conf::<ScheduleConfig>().kv_url;
    let headers = Some(vec![(
        "Tardis-Context".to_string(),
        TardisFuns::crypto.base64.encode(&TardisFuns::json.obj_to_string(&ctx)?),
    )]);
    // log this operation
    TardisFuns::web_client()
        .post_obj_to_str(
            &format!("{log_url}/ci/item"),
            &HashMap::from([
                ("tag", "schedule_job"),
                ("content", "delete job"),
                ("key", code),
                ("op", "delete"),
                ("ts", &Utc::now().to_rfc3339()),
            ]),
            headers.clone(),
        )
        .await?;
    {
        if let Some(_uuid) = unsafe { service() }.code_uuid.read().await.get(code) {
            // delete schedual-task from kv cache first
            funs.web_client()
                .delete_to_void(
                    &format!("{kv_url}/ci/item?key={key}", kv_url = kv_url, key = format_args!("{KV_KEY_CODE}{code}")),
                    headers.clone(),
                )
                .await?;
            // delete schedual-task from scheduler
            ScheduleTaskServ::delete(code).await?;
        }
    }
    Ok(())
}

pub(crate) async fn find_job(code: Option<String>, page_number: u32, page_size: u32, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<TardisPage<ScheduleJobInfoResp>> {
    let kv_url = &funs.conf::<ScheduleConfig>().kv_url;
    let headers = Some(vec![(
        "Tardis-Context".to_string(),
        TardisFuns::crypto.base64.encode(&TardisFuns::json.obj_to_string(&ctx)?),
    )]);
    let resp = funs
        .web_client()
        .get::<TardisPage<ScheduleJobKvSummaryResp>>(
            &format!(
                "{}/ci/item/match?key_prefix={}&page_number={}&page_size={}",
                kv_url,
                format_args!("{}{}", KV_KEY_CODE, code.unwrap_or("".to_string())),
                page_number,
                page_size
            ),
            headers,
        )
        .await?;
    if resp.code != 200 {
        return Err(funs.err().conflict("find_job", "find", "job is anomaly", ""));
    }
    let page = resp.body.unwrap();
    Ok(TardisPage {
        page_size: page.page_size,
        page_number: page.page_number,
        total_size: page.total_size,
        records: page
            .records
            .into_iter()
            .map(|record| ScheduleJobInfoResp {
                code: record.key.replace(KV_KEY_CODE, ""),
                cron: record.value.get("cron").map(ToString::to_string).unwrap_or_default(),
                callback_url: record.value.get("callback_url").map(ToString::to_string).unwrap_or_default(),
                create_time: Some(record.create_time),
                update_time: Some(record.update_time),
            })
            .collect(),
    })
}

pub(crate) async fn find_task(
    job_code: &str,
    ts_start: Option<chrono::DateTime<Utc>>,
    ts_end: Option<chrono::DateTime<Utc>>,
    page_number: u32,
    page_size: u32,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
) -> TardisResult<TardisPage<ScheduleTaskInfoResp>> {
    let log_url = &funs.conf::<ScheduleConfig>().log_url;
    let headers = Some(vec![(
        "Tardis-Context".to_string(),
        TardisFuns::crypto.base64.encode(&TardisFuns::json.obj_to_string(&ctx)?),
    )]);
    let mut url = format!(
        "{}/ci/item?tag={}&key={}&page_number={}&page_size={}",
        log_url,
        "schedule_task",
        job_code,
        page_number,
        (page_size * 2)
    );
    if let Some(ts_start) = ts_start {
        url += &format!("&ts_start={}", ts_start.to_rfc3339());
    }
    if let Some(ts_end) = ts_end {
        url += &format!("&ts_end={}", ts_end.to_rfc3339());
    }
    let resp = funs.web_client().get::<TardisPage<ScheduleTaskLogFindResp>>(&url, headers).await?;
    if resp.code != 200 {
        return Err(funs.err().conflict("find_job", "find", "job is anomaly", ""));
    }
    let page = resp.body.unwrap();
    let mut records = vec![];
    let mut log_iter = page.records.into_iter();
    while let Some(start_log) = log_iter.next() {
        let mut task = ScheduleTaskInfoResp {
            start: Some(start_log.ts),
            end: None,
            err_msg: None,
        };
        if let Some(end_log) = log_iter.next() {
            task.end = Some(end_log.ts);
            task.err_msg = Some(end_log.content);
        }
        records.push(task)
    }

    Ok(TardisPage {
        page_number: page.page_number,
        page_size: page.page_size / 2,
        total_size: page.total_size / 2,
        records,
    })
}

pub(crate) async fn init(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let service_instance = OwnedScheduleTaskServ::init(funs, ctx).await?;
    unsafe { MAYBE_GLOBAL_SERV.replace(service_instance) };
    Ok(())
}

pub struct ScheduleTaskServ;

impl ScheduleTaskServ {
    /// add schedule task
    pub async fn add(log_url: &str, add_or_modify: ScheduleJobAddOrModifyReq, config: &ScheduleConfig) -> TardisResult<()> {
        unsafe { MAYBE_GLOBAL_SERV.as_ref().expect("Schedule task serv not yet initialized") }.add(log_url, add_or_modify, config).await
    }

    pub async fn delete(code: &str) -> TardisResult<()> {
        unsafe { MAYBE_GLOBAL_SERV.as_ref().expect("Schedule task serv not yet initialized") }.delete(code).await
    }
}

#[derive(Clone)]
pub struct OwnedScheduleTaskServ {
    pub code_uuid: Arc<RwLock<HashMap<String, Uuid>>>,
    pub scheduler: Arc<JobScheduler>,
}

impl OwnedScheduleTaskServ {
    pub async fn init(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Self> {
        let cache_client = funs.cache();
        let config = funs.conf::<ScheduleConfig>().clone();
        let log_url = config.log_url.clone();
        if let Ok(job_resp) = self::find_job(None, 1, 9999, funs, ctx).await {
            let jobs = job_resp.records;
            {
                let mut cache_jobs = TASK.write().await;
                for job in jobs {
                    cache_jobs.insert(job.code.clone(), job);
                }
            }
        } else {
            tardis::log::debug!("encounter an error while init schedule middleware: fail to find job");
        }
        let mut scheduler = JobScheduler::new().await.expect("fail to create job scheduler for schedule mw");
        scheduler.set_shutdown_handler(Box::new(|| {
            Box::pin(async move {
                info!("mw-schedule: global scheduler shutted down");
            })
        }));
        scheduler.init().await.expect("fail to init job scheduler for schedule mw");
        scheduler.start().await.expect("fail to start job scheduler for schedule mw");
        tardis::tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(config.cache_key_job_changed_timer_sec as u64));
            let log_url = log_url.clone();
            loop {
                let mut conn = cache_client.cmd().await;
                let mut res_iter = {
                    match conn {
                        Ok(ref mut cache_cmd) => match cache_cmd.scan_match::<_, String>(&format!("{}*", config.cache_key_job_changed_info)).await {
                            Ok(res_iter) => res_iter,
                            Err(e) => {
                                error!("fail to scan match in redis: {e}");
                                break;
                            }
                        },
                        Err(e) => {
                            error!("fail to get redis connection: {e}");
                            break;
                        }
                    }
                };
                trace!("[Schedule] Fetch changed Job cache");
                {
                    while let Some(changed_key) = res_iter.next_item().await {
                        if let Ok(Some(job_cache)) = cache_client.get(&changed_key).await {
                            // safety: since we create job_cache ourselves, it's ok to unwrap
                            let job_json: ScheduleJobAddOrModifyReq = TardisFuns::json.str_to_obj(&job_cache).unwrap();
                            ScheduleTaskServ::add(&log_url, job_json, &config).await.map_err(|e| error!("fail to add schedule task: {e}")).unwrap_or_default();
                        } else {
                            ScheduleTaskServ::delete(&changed_key).await.map_err(|e| error!("fail to delete schedule task: {e}")).unwrap_or_default();
                        }
                    }
                }
                interval.tick().await;
            }
        });
        Ok(Self {
            code_uuid: Arc::new(RwLock::new(HashMap::new())),
            scheduler: Arc::new(scheduler),
        })
    }

    /// genetate distributed lock key for a certain task
    fn gen_distributed_lock_key(code: &str, config: &ScheduleConfig) -> String {
        format!("{}{}", config.distributed_lock_key_prefix, code)
    }
    /// add schedule task
    pub async fn add(&self, log_url: &str, add_or_modify: ScheduleJobAddOrModifyReq, config: &ScheduleConfig) -> TardisResult<()> {
        {
            if let Some(_uuid) = self.code_uuid.read().await.get(&add_or_modify.code.0) {
                self.delete(&add_or_modify.code.0).await?;
            }
        }
        let callback_url = add_or_modify.callback_url.clone();
        let log_url = log_url.to_string();
        let code = add_or_modify.code.0.clone();
        let lock_key = OwnedScheduleTaskServ::gen_distributed_lock_key(&code, config);
        let distributed_lock_expire_sec = config.distributed_lock_expire_sec;
        let ctx = TardisContext {
            own_paths: "".to_string(),
            ak: "".to_string(),
            owner: config.spi_app_id.clone(),
            roles: vec![],
            groups: vec![],
            ..Default::default()
        };
        let headers = Some(vec![(
            "Tardis-Context".to_string(),
            TardisFuns::crypto.base64.encode(&TardisFuns::json.obj_to_string(&ctx)?),
        )]);
        // startup cron scheduler
        let job = Job::new_async(add_or_modify.cron.as_str(), move |_uuid, _scheduler| {
            let callback_url = callback_url.clone();
            let log_url = log_url.clone();
            let code = code.clone();
            let headers = headers.clone();
            let lock_key = lock_key.clone();
            Box::pin(async move {
                let cache_client = TardisFuns::cache();
                // about set and setnx, see:
                // 1. https://redis.io/commands/set/
                // 2. https://redis.io/commands/setnx/
                // At Redis version 2.6.12, setnx command is regarded as deprecated. see: https://redis.io/commands/setnx/
                // "executing" could be any string now, it's just a placeholder
                match cache_client.set_nx(&lock_key, "executing").await {
                    Ok(true) => {
                        // safety: it's ok to unwrap in this closure, scheduler will restart this job when after panic
                        cache_client.expire(&lock_key, distributed_lock_expire_sec as usize).await.unwrap();
                        trace!("executing schedule task {code}");
                        // 1. write log exec start
                        TardisFuns::web_client()
                            .post_obj_to_str(
                                &format!("{log_url}/ci/item"),
                                &HashMap::from([
                                    ("tag", "schedule_task"),
                                    ("content", format!("schedule task {} exec start", code).as_str()),
                                    ("key", &code),
                                    ("op", "exec-start"),
                                    ("ts", &Utc::now().to_rfc3339()),
                                ]),
                                headers.clone(),
                            )
                            .await
                            .unwrap();
                        // 2. request webhook
                        let task_msg = TardisFuns::web_client().get_to_str(callback_url.as_str(), headers.clone()).await.unwrap();
                        // 3. write log exec end
                        TardisFuns::web_client()
                            .post_obj_to_str(
                                &format!("{log_url}/ci/item"),
                                &HashMap::from([
                                    ("tag", "schedule_task"),
                                    ("content", task_msg.body.unwrap().as_str()),
                                    ("key", &code),
                                    ("op", "exec-end"),
                                    ("ts", &Utc::now().to_rfc3339()),
                                ]),
                                headers,
                            )
                            .await
                            .unwrap();
                        trace!("executed schedule task {code}");
                    }
                    Ok(false) => {
                        trace!("schedule task {} is executed by other nodes, skip", code);
                    }
                    Err(e) => {
                        error!("cannot set lock to schedule task {code}, error: {e}");
                    }
                }
            })
        })
        .map_err(|err| {
            let msg = format!("fail to create job: {}", err);
            TardisError::internal_error(&msg, "500-middleware-schedual-create-task-failed")
        })?;
        let uuid = self.scheduler.add(job).await.map_err(|err| {
            let msg = format!("fail to add job: {}", err);
            TardisError::internal_error(&msg, "500-middleware-schedual-create-task-failed")
        })?;
        {
            self.code_uuid.write().await.insert(add_or_modify.code.0.clone(), uuid);
        }
        Ok(())
    }

    pub async fn delete(&self, code: &str) -> TardisResult<()> {
        let mut scheds = self.code_uuid.write().await;
        if let Some(uuid) = scheds.get(code) {
            self.scheduler.remove(uuid).await.map_err(|err| {
                let msg = format!("fail to add job: {}", err);
                TardisError::internal_error(&msg, "500-middleware-schedual-create-task-failed")
            })?;
            scheds.remove(code);
        }
        Ok(())
    }
}
