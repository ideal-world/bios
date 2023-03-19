use serde::{Deserialize, Serialize};
use tardis::{
    basic::field::TrimString,
    chrono::{self, DateTime, Utc},
    serde_json::Value,
    web::poem_openapi,
};

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
