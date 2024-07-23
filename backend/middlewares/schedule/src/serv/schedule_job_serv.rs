use std::vec;

use bios_sdk_invoke::clients::base_spi_client::BaseSpiClient;
use bios_sdk_invoke::clients::spi_log_client::{LogItemFindReq, SpiLogClient};
use bios_sdk_invoke::invoke_enumeration::InvokeModuleKind;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::chrono::{self, Utc};

use tardis::web::web_resp::{TardisPage, TardisResp};
use tardis::TardisFunsInst;

use crate::dto::schedule_job_dto::{ScheduleJob, ScheduleJobInfoResp, ScheduleJobKvSummaryResp, ScheduleTaskInfoResp};
use crate::schedule_constants::KV_KEY_CODE;

pub(crate) async fn find_job(code: Option<String>, page_number: u32, page_size: u16, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<TardisPage<ScheduleJobInfoResp>> {
    let kv_url = BaseSpiClient::module_url(InvokeModuleKind::Kv, funs).await?;
    let headers = BaseSpiClient::headers(None, funs, ctx).await?;
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
    let body = BaseSpiClient::package_resp(resp)?;
    let Some(pages) = body else {
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
                let job = ScheduleJob::parse_from_json(&record.value);
                ScheduleJobInfoResp {
                    code: record.key.replace(KV_KEY_CODE, ""),
                    cron: job.cron,
                    callback_url: job.callback_url,
                    callback_headers: job.callback_headers,
                    callback_method: job.callback_method,
                    callback_body: job.callback_body,
                    create_time: Some(record.create_time),
                    update_time: Some(record.update_time),
                    enable_time: job.enable_time,
                    disable_time: job.disable_time,
                }
            })
            .collect(),
    })
}

pub(crate) async fn find_task(
    job_code: &str,
    ts_start: Option<chrono::DateTime<Utc>>,
    ts_end: Option<chrono::DateTime<Utc>>,
    page_number: u32,
    page_size: u16,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
) -> TardisResult<TardisPage<ScheduleTaskInfoResp>> {
    let resp = SpiLogClient::find(
        LogItemFindReq {
            tag: "schedule_task".to_string(),
            keys: Some(vec![job_code.into()]),
            page_number,
            page_size: page_size * 2,
            ts_start,
            ts_end,
            ..Default::default()
        },
        funs,
        ctx,
    )
    .await?;
    let Some(page) = resp else {
        return Err(funs.err().conflict("find_job", "find", "task response missing body", ""));
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