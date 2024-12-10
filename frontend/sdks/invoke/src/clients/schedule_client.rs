use std::collections::HashMap;

use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    serde_json::json,
    TardisFunsInst,
};

use crate::invoke_enumeration::InvokeModuleKind;

use super::base_spi_client::BaseSpiClient;

#[derive(Clone, Debug, Default)]
pub struct ScheduleClient;

pub struct AddOrModifySyncTaskReq {
    pub code: String,
    /// is_enable
    /// 是否开启
    pub enable: bool,
    /// cron
    /// 定时任务表达式
    pub cron: String,
    /// callback_url
    /// 回调地址
    pub callback_url: String,
    /// callback_headers
    /// 回调头
    pub callback_headers: HashMap<String, String>,
    /// callback_method
    /// 回调方法
    pub callback_method: String,
    /// callback_body
    /// 回调体
    pub callback_body: Option<String>,
}

impl ScheduleClient {
    pub async fn add_or_modify_sync_task(req: AddOrModifySyncTaskReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let schedule_url: String = BaseSpiClient::module_url(InvokeModuleKind::Schedule, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;
        if req.enable {
            funs.web_client()
                .put_obj_to_str(
                    &format!("{schedule_url}/ci/schedule/jobs"),
                    &json!({
                      "code": req.code,
                    "cron": vec![req.cron],
                      "callback_url": req.callback_url,
                      "callback_headers": req.callback_headers,
                      "callback_method": req.callback_method,
                      "callback_body": req.callback_body,
                    }),
                    headers.clone(),
                )
                .await?;
        } else {
            Self::delete_sync_task(&req.code, funs, ctx).await?;
        }
        Ok(())
    }

    pub async fn delete_sync_task(code: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let schedule_url: String = BaseSpiClient::module_url(InvokeModuleKind::Schedule, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;
        funs.web_client().delete_to_void(&format!("{schedule_url}/ci/schedule/jobs/{}", code), headers).await?;
        Ok(())
    }
}
