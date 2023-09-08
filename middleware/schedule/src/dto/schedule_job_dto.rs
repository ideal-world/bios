use serde::{Deserialize, Serialize};
use tardis::{
    basic::{error::TardisError, field::TrimString},
    chrono::{self, DateTime, Utc},
    db::sea_orm,
    serde_json::Value,
    web::poem_openapi,
};

use crate::schedule_constants::KV_KEY_CODE;

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, sea_orm::FromQueryResult)]
pub(crate) struct KvItemSummaryResp {
    #[oai(validator(min_length = "2"))]
    pub key: String,
    pub value: Value,
    pub info: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Clone, Debug)]
pub struct ScheduleJobAddOrModifyReq {
    #[oai(validator(min_length = "2"))]
    pub code: TrimString,
    #[oai(validator(min_length = "2"))]
    pub cron: String,
    #[oai(validator(min_length = "2"))]
    pub callback_url: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Clone, Debug)]
pub struct ScheduleJobKvSummaryResp {
    pub key: String,
    pub value: Value,
    pub info: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Default)]
pub struct ScheduleJobInfoResp {
    pub code: String,
    pub cron: String,
    pub callback_url: String,
    pub create_time: Option<chrono::DateTime<Utc>>,
    pub update_time: Option<chrono::DateTime<Utc>>,
}

#[derive(poem_openapi::Object, Deserialize, Debug, Serialize)]
pub(crate) struct KvSchedualJobItemDetailResp {
    pub key: String,
    pub value: ScheduleJobAddOrModifyReq,
    pub info: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

impl TryFrom<KvItemSummaryResp> for KvSchedualJobItemDetailResp {
    type Error = TardisError;

    fn try_from(resp: KvItemSummaryResp) -> Result<Self, Self::Error> {
        let Some(s) = &resp.value.as_str() else {
            return Err(TardisError::internal_error("value are expected to be a string", "schedule-409-bad-schedule-job"));
        };
        let req: ScheduleJobAddOrModifyReq =
            tardis::serde_json::from_str(s).map_err(|e| TardisError::internal_error(&format!("can't parse schedule job json body: {e}"), "schedule-409-bad-schedule-job"))?;
        Ok(Self {
            key: resp.key.trim_start_matches(KV_KEY_CODE).to_string(),
            value: req,
            info: resp.info,
            create_time: resp.create_time,
            update_time: resp.update_time,
        })
    }
}

impl ScheduleJobInfoResp {
    pub fn create_add_or_mod_req(&self) -> ScheduleJobAddOrModifyReq {
        ScheduleJobAddOrModifyReq {
            code: self.code.clone().into(),
            cron: self.cron.clone(),
            callback_url: self.callback_url.clone(),
        }
    }
}
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct ScheduleTaskLogFindResp {
    #[oai(validator(min_length = "2"))]
    pub content: String,
    pub key: String,
    pub op: String,
    pub rel_key: String,
    pub ts: DateTime<Utc>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Default)]
pub struct ScheduleTaskInfoResp {
    pub start: Option<chrono::DateTime<Utc>>,
    pub end: Option<chrono::DateTime<Utc>>,
    pub err_msg: Option<String>,
}
