use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use std::vec;

use tardis::basic::dto::TardisContext;
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::cache::AsyncCommands;
use tardis::chrono::{self, Utc};
use tardis::db::sea_orm::prelude::Uuid;
use tardis::log::{error, info, trace, warn};
use tardis::tokio::sync::RwLock;
use tardis::tokio::time;
use tardis::web::web_resp::{TardisPage, TardisResp};
use tardis::{TardisFuns, TardisFunsInst};
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::dto::schedule_job_dto::{
    KvItemSummaryResp, KvScheduleJobItemDetailResp, ScheduleJobAddOrModifyReq, ScheduleJobInfoResp, ScheduleJobKvSummaryResp, ScheduleTaskInfoResp, ScheduleTaskLogFindResp,
};
use crate::schedule_config::ScheduleConfig;
use crate::schedule_constants::{DOMAIN_CODE, KV_KEY_CODE};

/// global service instance
static GLOBAL_SERV: OnceLock<Arc<OwnedScheduleTaskServ>> = OnceLock::new();

/// get service instance without checking if it's initialized
/// # Safety
/// if called before init, this function will panic
fn service() -> Arc<OwnedScheduleTaskServ> {
    GLOBAL_SERV.get().expect("trying to get scheduler before it's initialized").clone()
}

// still not good, should manage to merge it with `OwnedScheduleTaskServ::add`
// same as `delete`
pub(crate) async fn add_or_modify(add_or_modify: ScheduleJobAddOrModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let log_url = &funs.conf::<ScheduleConfig>().log_url;
    let kv_url = &funs.conf::<ScheduleConfig>().kv_url;
    let code = add_or_modify.code.as_str();
    let spi_ctx = TardisContext {
        owner: funs.conf::<ScheduleConfig>().spi_app_id.clone(),
        ..ctx.clone()
    };
    let headers = vec![("Tardis-Context".to_string(), TardisFuns::crypto.base64.encode(TardisFuns::json.obj_to_string(&spi_ctx)?))];
    // if exist delete it first
    if service().code_uuid.write().await.get(code).is_some() {
        delete(code, funs, ctx).await?;
    }
    // 1. log add operation
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
    let config = funs.conf::<ScheduleConfig>();
    // 2. sync to kv
    funs.web_client()
        .put_obj_to_str(
            &format!("{kv_url}/ci/item"),
            &HashMap::from([("key", format!("{KV_KEY_CODE}{code}")), ("value", TardisFuns::json.obj_to_string(&add_or_modify)?)]),
            headers.clone(),
        )
        .await?;
    // 3. notify cache
    let mut conn = funs.cache().cmd().await?;
    let cache_key_job_changed_info = &config.cache_key_job_changed_info;
    conn.set_ex(&format!("{cache_key_job_changed_info}{code}"), "update", config.cache_key_job_changed_timer_sec as u64).await?;
    // 4. do add at local scheduler
    ScheduleTaskServ::add(add_or_modify, &config).await?;
    Ok(())
}

pub(crate) async fn delete(code: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let log_url = &funs.conf::<ScheduleConfig>().log_url;
    let kv_url = &funs.conf::<ScheduleConfig>().kv_url;
    let spi_ctx = TardisContext {
        owner: funs.conf::<ScheduleConfig>().spi_app_id.clone(),
        ..ctx.clone()
    };
    let headers = vec![("Tardis-Context".to_string(), TardisFuns::crypto.base64.encode(TardisFuns::json.obj_to_string(&spi_ctx)?))];
    // 1. log this operation
    // ws_client()
    //     .await
    //     .publish_add_log(
    //         &LogItemAddReq {
    //             tag: "schedule_job".to_string(),
    //             content: "delete job".to_string(),
    //             key: Some(code.into()),
    //             op: Some("delete".to_string()),
    //             ts: Some(Utc::now()),
    //             ..Default::default()
    //         },
    //         default_avatar().await.clone(),
    //         funs.conf::<ScheduleConfig>().spi_app_id.clone(),
    //         ctx,
    //     )
    //     .await?;
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
    // 2. sync to kv
    TardisFuns::web_client().delete_to_void(&format!("{kv_url}/ci/item?key={KV_KEY_CODE}{code}"), headers.clone()).await?;
    // 3. notify cache
    let config = funs.conf::<ScheduleConfig>();
    let mut conn = funs.cache().cmd().await?;
    let cache_key_job_changed_info = &config.cache_key_job_changed_info;
    conn.set_ex(&format!("{cache_key_job_changed_info}{code}"), "delete", config.cache_key_job_changed_timer_sec as u64).await?;
    // 4. do delete at local scheduler
    if service().code_uuid.read().await.get(code).is_some() {
        // delete schedual-task from kv cache first
        let mut conn = funs.cache().cmd().await?;
        let config = funs.conf::<ScheduleConfig>();
        let cache_key_job_changed_info = &config.cache_key_job_changed_info;
        conn.del(&format!("{cache_key_job_changed_info}{code}")).await?;
        // delete schedual-task from scheduler
        ScheduleTaskServ::delete(code).await?;
    }
    Ok(())
}

pub(crate) async fn find_job(code: Option<String>, page_number: u32, page_size: u32, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<TardisPage<ScheduleJobInfoResp>> {
    let kv_url = &funs.conf::<ScheduleConfig>().kv_url;
    let spi_ctx = TardisContext {
        owner: funs.conf::<ScheduleConfig>().spi_app_id.clone(),
        ..ctx.clone()
    };
    let headers = [("Tardis-Context".to_string(), TardisFuns::crypto.base64.encode(TardisFuns::json.obj_to_string(&spi_ctx)?))];
    let resp = funs
        .web_client()
        .get::<TardisResp<TardisPage<ScheduleJobKvSummaryResp>>>(
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
    let Some(body) = resp.body else {
        return Err(funs.err().conflict("find_job", "find", "get Job Kv response missing body", ""));
    };
    let Some(pages) = body.data else {
        return Err(funs.err().conflict("find_job", "find", "get Job Kv failed", ""));
    };
    Ok(TardisPage {
        page_size: pages.page_size,
        page_number: pages.page_number,
        total_size: pages.total_size,
        records: pages
            .records
            .into_iter()
            .map(|record| {
                let job = record.value.as_str().map(|json_str| TardisFuns::json.str_to_obj::<ScheduleJobAddOrModifyReq>(json_str));
                match job {
                    Some(Ok(job)) => ScheduleJobInfoResp {
                        code: record.key.replace(KV_KEY_CODE, ""),
                        cron: job.cron,
                        callback_url: job.callback_url,
                        create_time: Some(record.create_time),
                        update_time: Some(record.update_time),
                        enable_time: job.enable_time,
                        disable_time: job.disable_time,
                    },
                    _ => ScheduleJobInfoResp {
                        code: record.key.replace(KV_KEY_CODE, ""),
                        cron: "".to_string(),
                        callback_url: "".to_string(),
                        create_time: Some(record.create_time),
                        update_time: Some(record.update_time),
                        enable_time: None,
                        disable_time: None,
                    },
                }
            })
            .collect(),
    })
}

pub(crate) async fn find_one_job(code: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<KvScheduleJobItemDetailResp>> {
    let kv_url = &funs.conf::<ScheduleConfig>().kv_url;
    let spi_ctx = TardisContext {
        owner: funs.conf::<ScheduleConfig>().spi_app_id.clone(),
        ..ctx.clone()
    };
    let headers = [("Tardis-Context".to_string(), TardisFuns::crypto.base64.encode(TardisFuns::json.obj_to_string(&spi_ctx)?))];
    let resp = funs.web_client().get::<TardisResp<Option<KvItemSummaryResp>>>(&format!("{}/ci/item?key={}", kv_url, format_args!("{}{}", KV_KEY_CODE, code)), headers).await?;

    let Some(body) = resp.body else {
        return Err(funs.err().internal_error("find_job", "find", "kv response missing body", ""));
    };
    let msg = &body.msg;
    if &body.code != "200" {
        return Err(funs.err().internal_error("find_job", "find", &format!("fail to get kv resp: {}", msg), ""));
    }
    body.data.flatten().map(KvScheduleJobItemDetailResp::try_from).transpose()
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
    let spi_ctx = TardisContext {
        owner: funs.conf::<ScheduleConfig>().spi_app_id.clone(),
        ..ctx.clone()
    };
    let headers = [("Tardis-Context".to_string(), TardisFuns::crypto.base64.encode(TardisFuns::json.obj_to_string(&spi_ctx)?))];
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
    let resp = funs.web_client().get::<TardisResp<TardisPage<ScheduleTaskLogFindResp>>>(&url, headers).await?;
    if resp.code != 200 {
        return Err(funs.err().conflict("find_job", "find", &resp.body.map(|x| x.msg).unwrap_or_default(), ""));
    }
    let Some(body) = resp.body else {
        return Err(funs.err().conflict("find_job", "find", "task response missing body", ""));
    };
    let Some(page) = body.data else {
        return Err(funs.err().conflict("find_job", "find", &body.msg, ""));
    };
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
    GLOBAL_SERV.get_or_init(|| service_instance);
    Ok(())
}

pub struct ScheduleTaskServ;

impl ScheduleTaskServ {
    /// add schedule task
    pub async fn add(add_or_modify: ScheduleJobAddOrModifyReq, config: &ScheduleConfig) -> TardisResult<()> {
        service().add(add_or_modify, config).await
    }

    pub async fn delete(code: &str) -> TardisResult<()> {
        service().delete(code).await
    }
}

#[derive(Clone)]
pub struct OwnedScheduleTaskServ {
    #[allow(clippy::type_complexity)]
    pub code_uuid: Arc<RwLock<HashMap<String, (Uuid, String, String)>>>,
    pub scheduler: Arc<JobScheduler>,
}

impl OwnedScheduleTaskServ {
    pub async fn init(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Arc<Self>> {
        let cache_client = funs.cache();
        let mut scheduler = JobScheduler::new().await.expect("fail to create job scheduler for schedule mw");
        scheduler.set_shutdown_handler(Box::new(|| {
            Box::pin(async move {
                info!("mw-schedule: global scheduler shutted down");
            })
        }));
        scheduler.init().await.expect("fail to init job scheduler for schedule mw");
        scheduler.start().await.expect("fail to start job scheduler for schedule mw");
        let code_uuid_cache_raw = Arc::new(RwLock::new(HashMap::<String, (Uuid, String, String)>::new()));
        let serv_raw = Arc::new(Self {
            code_uuid: code_uuid_cache_raw,
            scheduler: Arc::new(scheduler),
        });
        let serv = serv_raw.clone();
        let sync_db_ctx = ctx.clone();
        // sync from db task
        tardis::tokio::spawn(async move {
            let funs = TardisFuns::inst(DOMAIN_CODE, None);
            // every 5 seconds, query if webserver is started
            let mut interval = time::interval(Duration::from_secs(5));
            let config = funs.conf::<ScheduleConfig>().clone();
            let mut retry_time = 0;
            let max_retry_time = 5;
            loop {
                if TardisFuns::web_server().is_running().await {
                    if let Ok(job_resp) = self::find_job(None, 1, 9999, &funs, &sync_db_ctx).await {
                        let jobs = job_resp.records;
                        for job in jobs {
                            serv.add(job.create_add_or_mod_req(), &config).await.map_err(|e| error!("fail to delete schedule task: {e}")).unwrap_or_default();
                        }
                        info!("synced all jobs from kv");
                        break;
                    } else {
                        warn!("encounter an error while init schedule middleware: fail to find job {retry_time}/{max_retry_time}");
                        retry_time += 1;
                        if retry_time >= max_retry_time {
                            error!("fail to sync jobs from kv, schedule running without history jobs");
                            break;
                        }
                    }
                }
                interval.tick().await;
            }
        });
        let ctx = ctx.clone();
        let serv = serv_raw.clone();
        // sync from cache task
        tardis::tokio::spawn(async move {
            let funs = TardisFuns::inst(DOMAIN_CODE, None);
            let config = funs.conf::<ScheduleConfig>();
            let mut interval = time::interval(Duration::from_secs(config.cache_key_job_changed_timer_sec as u64));
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
                    // collect configs from remote cache
                    while let Some(remote_job_code) = res_iter.next_item().await {
                        {
                            let code = remote_job_code.trim_start_matches(&config.cache_key_job_changed_info);
                            let funs = TardisFuns::inst(DOMAIN_CODE, None);
                            match self::find_one_job(code, &funs, &ctx).await {
                                Ok(Some(resp)) => {
                                    // if we have this job code in local cache, update or add it
                                    serv.add(resp.value, &config).await.map_err(|e| error!("fail to delete schedule task: {e}")).unwrap_or_default();
                                }
                                Ok(None) => {
                                    // if we don't have this job code in local cache, remove it
                                    serv.delete(&remote_job_code).await.map_err(|e| error!("fail to delete schedule task: {e}")).unwrap_or_default();
                                }
                                Err(e) => {
                                    error!("fail to fetch error from spi-kv: {e}")
                                }
                            }
                        }
                    }
                }
                interval.tick().await;
            }
        });
        Ok(serv_raw)
    }

    /// genetate distributed lock key for a certain task
    fn gen_distributed_lock_key(code: &str, config: &ScheduleConfig) -> String {
        format!("{}{}", config.distributed_lock_key_prefix, code)
    }

    /// add schedule task
    pub async fn add(&self, job_config: ScheduleJobAddOrModifyReq, config: &ScheduleConfig) -> TardisResult<()> {
        let has_job = { self.code_uuid.read().await.get(job_config.code.as_str()).is_some() };
        if has_job {
            self.delete(&job_config.code).await?;
        }
        let callback_url = job_config.callback_url.clone();
        let log_url = config.log_url.clone();
        let code = job_config.code.to_string();
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
        let headers = [("Tardis-Context".to_string(), TardisFuns::crypto.base64.encode(TardisFuns::json.obj_to_string(&ctx)?))];

        let enable_time = job_config.enable_time;
        let disable_time = job_config.disable_time;
        // startup cron scheduler
        let job = Job::new_async(job_config.cron.as_str(), move |_uuid, _scheduler| {
            let callback_url = callback_url.clone();
            let log_url = log_url.clone();
            let code = code.clone();
            let headers = headers.clone();
            let lock_key = lock_key.clone();
            if let Some(enable_time) = enable_time {
                if enable_time > Utc::now() {
                    return Box::pin(async move {
                        trace!("schedule task {code} is not enabled yet, skip", code = code);
                    });
                }
            }
            if let Some(disable_time) = disable_time {
                if disable_time < Utc::now() {
                    return Box::pin(async move {
                        trace!("schedule task {code} is disabled, skip", code = code);
                    });
                }
            }
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
                        let Ok(()) = cache_client.expire(&lock_key, distributed_lock_expire_sec as i64).await else {
                            return;
                        };
                        trace!("executing schedule task {code}");
                        // 1. write log exec start
                        let Ok(_) = TardisFuns::web_client()
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
                        else {
                            return;
                        };
                        // 2. request webhook
                        let Ok(task_msg) = TardisFuns::web_client().get_to_str(callback_url.as_str(), headers.clone()).await else {
                            return;
                        };
                        // 3. write log exec end
                        let Ok(_) = TardisFuns::web_client()
                            .post_obj_to_str(
                                &format!("{log_url}/ci/item"),
                                &HashMap::from([
                                    ("tag", "schedule_task"),
                                    ("content", task_msg.body.unwrap_or_default().as_str()),
                                    ("key", &code),
                                    ("op", "exec-end"),
                                    ("ts", &Utc::now().to_rfc3339()),
                                ]),
                                headers,
                            )
                            .await
                        else {
                            return;
                        };
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
            self.code_uuid.write().await.insert(job_config.code.to_string(), (uuid, job_config.cron.clone(), job_config.callback_url.clone()));
        }
        Ok(())
    }

    pub async fn delete(&self, code: &str) -> TardisResult<()> {
        let mut uuid_cache = self.code_uuid.write().await;
        if let Some((uuid, _, _)) = uuid_cache.get(code) {
            self.scheduler.remove(uuid).await.map_err(|err| {
                let msg = format!("fail to add job: {}", err);
                TardisError::internal_error(&msg, "500-middleware-schedual-create-task-failed")
            })?;
            uuid_cache.remove(code);
        }
        Ok(())
    }
}
