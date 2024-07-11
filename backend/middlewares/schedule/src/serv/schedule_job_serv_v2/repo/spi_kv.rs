use std::{ops::Deref, sync::Arc};

use bios_sdk_invoke::{
    clients::{
        base_spi_client::BaseSpiClient,
        spi_kv_client::{KvItemDetailResp, SpiKvClient},
    },
    invoke_enumeration::InvokeModuleKind,
};
use tardis::{
    basic::{dto::TardisContext, error::TardisError},
    web::web_resp::{TardisPage, TardisResp},
    TardisFunsInst,
};

use crate::{dto::schedule_job_dto::ScheduleJob, schedule_constants::KV_KEY_CODE};
#[derive(Clone)]
pub struct SpiKv {
    funs: Arc<TardisFunsInst>,
    ctx: Arc<TardisContext>,
    _client: SpiKvClient,
}

impl super::Repository for SpiKv {
    fn from_context(funs: impl Into<Arc<TardisFunsInst>>, ctx: impl Into<Arc<TardisContext>>) -> Self {
        Self {
            funs: funs.into(),
            ctx: ctx.into(),
            _client: SpiKvClient,
        }
    }
    async fn get_one(&self, code: &str) -> Result<Option<ScheduleJob>, TardisError> {
        let kv_url = BaseSpiClient::module_url(InvokeModuleKind::Kv, &self.funs).await?;
        let headers = BaseSpiClient::headers(None, &self.funs, &self.ctx).await?;
        let resp =
            self.funs.web_client().get::<TardisResp<Option<KvItemDetailResp>>>(&format!("{}/ci/item?key={}", kv_url, format_args!("{}{}", KV_KEY_CODE, code)), headers).await?;
        let body = BaseSpiClient::package_resp(resp)?;
        Ok(body.flatten().map(|resp| ScheduleJob::parse_from_json(&resp.value)))
    }

    async fn get_all(&self) -> Result<Vec<ScheduleJob>, TardisError> {
        let paged = self.get_paged(1, 9999).await?;
        Ok(paged.records)
    }

    async fn get_paged(&self, page: u32, size: u16) -> Result<TardisPage<ScheduleJob>, TardisError> {
        let resp = SpiKvClient::match_items_by_key_prefix(KV_KEY_CODE.to_string(), None, page, size, &self.funs, &self.ctx).await?;
        let Some(pages) = resp else {
            return Err(self.funs.err().conflict("find_job", "find", "get Job Kv failed", ""));
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
                    ScheduleJob {
                        code: record.key.replace(KV_KEY_CODE, "").into(),
                        cron: job.cron,
                        callback_url: job.callback_url,
                        callback_headers: job.callback_headers,
                        callback_method: job.callback_method,
                        callback_body: job.callback_body,
                        enable_time: job.enable_time,
                        disable_time: job.disable_time,
                    }
                })
                .collect(),
        })
    }

    async fn create(&self, req: &ScheduleJob) -> Result<(), TardisError> {
        self.update(req).await
    }

    async fn update(&self, req: &ScheduleJob) -> Result<(), TardisError> {
        let code = req.code.deref();
        SpiKvClient::add_or_modify_item(&format!("{KV_KEY_CODE}{code}"), &req, None, None, &self.funs, &self.ctx).await?;
        Ok(())
    }

    async fn delete(&self, code: &str) -> Result<(), TardisError> {
        SpiKvClient::delete_item(&format!("{KV_KEY_CODE}{code}"), &self.funs, &self.ctx).await?;
        Ok(())
    }
}
