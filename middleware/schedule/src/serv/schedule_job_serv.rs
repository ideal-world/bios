use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use lazy_static::lazy_static;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::cache::{AsyncCommands, AsyncIter};
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
    pub static ref SCHED: Arc<RwLock<HashMap<String, Uuid>>> = Arc::new(RwLock::new(HashMap::new()));
}

pub(crate) async fn add_or_modify(add_or_modify: ScheduleJobAddOrModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let scheds = SCHED.write().await;
    let log_url = funs.conf::<ScheduleConfig>().log_url.clone();
    let kv_url = funs.conf::<ScheduleConfig>().kv_url.clone();
    let code = add_or_modify.code.0.clone();
    let spi_ctx = TardisContext {
        owner: funs.conf::<ScheduleConfig>().spi_app_id.clone(),
        ..ctx.clone()
    };
    let headers = Some(vec![(
        "Tardis-Context".to_string(),
        TardisFuns::crypto.base64.encode(&TardisFuns::json.obj_to_string(&spi_ctx).unwrap()),
    )]);
    if let Some(_uuid) = scheds.get(&add_or_modify.code.0.clone()) {
        self::delete(&add_or_modify.code.0, funs, ctx).await?;
    }
    funs.web_client()
        .post_obj_to_str(
            &format!("{log_url}/ci/item"),
            &HashMap::from([
                ("tag", "schedule:job"),
                ("content", "add job"),
                ("key", &code),
                ("op", "add"),
                ("ts", &Utc::now().to_rfc3339()),
            ]),
            headers.clone(),
        )
        .await
        .unwrap();
    funs.web_client()
        .put_obj_to_str(
            &format!("{kv_url}/ci/item"),
            &HashMap::from([("key", format!("{KV_KEY_CODE}{code}")), ("value", TardisFuns::json.obj_to_string(&add_or_modify).unwrap())]),
            headers.clone(),
        )
        .await
        .unwrap();
    ScheduleTaskServ::add(&log_url, add_or_modify).await?;
    Ok(())
}

pub(crate) async fn delete(code: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let scheds = SCHED.write().await;
    let log_url = funs.conf::<ScheduleConfig>().log_url.clone();
    let kv_url = funs.conf::<ScheduleConfig>().kv_url.clone();
    let headers = Some(vec![(
        "Tardis-Context".to_string(),
        TardisFuns::crypto.base64.encode(&TardisFuns::json.obj_to_string(&ctx).unwrap()),
    )]);
    TardisFuns::web_client()
        .post_obj_to_str(
            &format!("{log_url}/ci/item"),
            &HashMap::from([
                ("tag", "schedule:job"),
                ("content", "delete job"),
                ("key", code),
                ("op", "d"),
                ("ts", &Utc::now().to_rfc3339()),
            ]),
            headers.clone(),
        )
        .await
        .unwrap();
    if let Some(_uuid) = scheds.get(code) {
        funs.web_client().delete_to_void(&format!("{}/ci/item?key={}", kv_url, format_args!("{KV_KEY_CODE}{code}")), headers.clone()).await.unwrap();
        ScheduleTaskServ::delete(code).await?;
    }
    Ok(())
}

pub(crate) async fn find_job(code: Option<String>, page_number: u32, page_size: u32, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<TardisPage<ScheduleJobInfoResp>> {
    let kv_url = funs.conf::<ScheduleConfig>().kv_url.clone();
    let headers = Some(vec![(
        "Tardis-Context".to_string(),
        TardisFuns::crypto.base64.encode(&TardisFuns::json.obj_to_string(&ctx).unwrap()),
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
        .await
        .unwrap();
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
                cron: record.value.get("cron").unwrap().to_string(),
                callback_url: record.value.get("callback_url").unwrap().to_string(),
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
    let log_url = funs.conf::<ScheduleConfig>().log_url.clone();
    let headers = Some(vec![(
        "Tardis-Context".to_string(),
        TardisFuns::crypto.base64.encode(&TardisFuns::json.obj_to_string(&ctx).unwrap()),
    )]);
    let mut url = format!(
        "{}/ci/item?tag={}&key={}&page_number={}&page_size={}",
        log_url,
        "schedule:task",
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
    let resp = funs.web_client().get::<TardisPage<ScheduleTaskLogFindResp>>(&url, headers).await.unwrap();
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

#[allow(dead_code)]
pub(crate) async fn init(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let cache_client = funs.cache();
    let config = funs.conf::<ScheduleConfig>().clone();
    let log_url = config.log_url.clone();
    let job_resp = self::find_job(None, 1, 9999, funs, ctx).await?;
    let jobs = job_resp.records;
    let mut cache_jobs = TASK.write().await;
    for job in jobs {
        cache_jobs.insert(job.code.clone(), job);
    }
    tardis::tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(config.cache_key_job_changed_timer_sec as u64));
        let log_url = log_url.clone();
        loop {
            {
                let mut cache_cmd = cache_client.cmd().await.unwrap();
                trace!("[Schedule] Fetch changed Job cache");
                let mut res_iter: AsyncIter<String> = cache_cmd.scan_match(&format!("{}*", config.cache_key_job_changed_info)).await.unwrap();
                while let Some(changed_key) = res_iter.next_item().await {
                    if let Some(job_cache) = cache_client.get(&changed_key).await.unwrap() {
                        let job_json: ScheduleJobAddOrModifyReq = TardisFuns::json.str_to_obj(&job_cache).unwrap();
                        ScheduleTaskServ::add(&log_url, job_json).await.unwrap();
                    } else {
                        ScheduleTaskServ::delete(&changed_key).await.unwrap();
                    }
                }
            }
            interval.tick().await;
        }
    });
    Ok(())
}

pub struct ScheduleTaskServ;

impl ScheduleTaskServ {
    pub async fn add(log_url: &str, add_or_modify: ScheduleJobAddOrModifyReq) -> TardisResult<()> {
        let mut scheds = SCHED.write().await;
        if let Some(_uuid) = scheds.get(&add_or_modify.code.0.clone()) {
            Self::delete(&add_or_modify.code.0).await?;
        }
        let callback_url = add_or_modify.callback_url.clone();
        let log_url = log_url.to_string();
        let code = add_or_modify.code.0.clone();
        let ctx = TardisContext {
            own_paths: "".to_string(),
            ak: "".to_string(),
            owner: "".to_string(),
            roles: vec![],
            groups: vec![],
            ..Default::default()
        };
        let headers = Some(vec![(
            "Tardis-Context".to_string(),
            TardisFuns::crypto.base64.encode(&TardisFuns::json.obj_to_string(&ctx).unwrap()),
        )]);
        let mut sched = JobScheduler::new().await.expect("job is not running");
        let job = Job::new_async(add_or_modify.cron.as_str(), move |uuid, _| {
            let callback_url = callback_url.clone();
            let log_url = log_url.clone();
            let code = code.clone();
            let headers = headers.clone();
            Box::pin(async move {
                TardisFuns::web_client()
                    .post_obj_to_str(
                        &format!("{log_url}/ci/item"),
                        &HashMap::from([
                            ("tag", "schedule:tesk"),
                            ("content", ""),
                            ("key", &code),
                            ("op", "exec-start"),
                            ("ts", &Utc::now().to_rfc3339()),
                        ]),
                        headers.clone(),
                    )
                    .await
                    .unwrap();
                let task_msg = TardisFuns::web_client().get_to_str(callback_url.as_str(), headers.clone()).await.unwrap();
                TardisFuns::web_client()
                    .post_obj_to_str(
                        &format!("{log_url}/ci/item"),
                        &HashMap::from([
                            ("tag", "schedule:tesk"),
                            ("content", task_msg.body.unwrap().as_str()),
                            ("key", &code),
                            ("op", "exec-end"),
                            ("ts", &Utc::now().to_rfc3339()),
                        ]),
                        headers,
                    )
                    .await
                    .unwrap();
                info!("Run every seconds UUID :{}", uuid.to_string());
            })
        })
        .unwrap();
        let uuid = sched.add(job).await.expect("job add expect is ok");
        scheds.insert(add_or_modify.code.0.clone(), uuid);
        sched.set_shutdown_handler(Box::new(|| {
            Box::pin(async move {
                info!("Shut down done");
            })
        }));
        match sched.start().await {
            Ok(_) => {
                info!("Start job scheduler");
            }
            Err(e) => {
                error!("Start job scheduler error: {}", e);
            }
        }
        Ok(())
    }

    pub async fn delete(code: &str) -> TardisResult<()> {
        let mut scheds = SCHED.write().await;
        if let Some(uuid) = scheds.get(code) {
            let sched = JobScheduler::new().await.expect("job is not running");
            sched.remove(uuid).await.expect("job is not remove");
            scheds.remove(code);
        }
        Ok(())
    }
}
