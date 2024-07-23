use std::{collections::HashMap, str::FromStr};

use bios_sdk_invoke::clients::spi_kv_client::KvItemDetailResp;
use serde::{Deserialize, Serialize};
use tardis::{
    basic::{error::TardisError, field::TrimString, result::TardisResult},
    chrono::{self, DateTime, Utc},
    db::sea_orm,
    serde_json::{self, Value},
    url::Url,
    web::{
        poem_openapi,
        reqwest::{header, Method},
    },
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
pub struct ScheduleJob {
    #[oai(validator(min_length = "2"))]
    pub code: TrimString,
    pub cron: Vec<String>,
    #[oai(validator(min_length = "2"))]
    pub callback_url: String,
    #[oai(default)]
    #[serde(default)]
    pub callback_headers: HashMap<String, String>,
    #[oai(default)]
    #[serde(default)]
    pub callback_method: String,
    #[oai(default)]
    #[serde(default)]
    pub callback_body: Option<String>,
    #[oai(default)]
    #[serde(default)]
    pub enable_time: Option<DateTime<Utc>>,
    #[oai(default)]
    #[serde(default)]
    pub disable_time: Option<DateTime<Utc>>,
}

impl Default for ScheduleJob {
    fn default() -> Self {
        Self {
            code: Default::default(),
            cron: Default::default(),
            callback_url: Default::default(),
            callback_headers: Default::default(),
            callback_method: "GET".to_string(),
            callback_body: Default::default(),
            enable_time: Default::default(),
            disable_time: Default::default(),
        }
    }
}

impl ScheduleJob {
    pub fn parse_time_from_json_value(value: &Value) -> Option<DateTime<Utc>> {
        match value {
            Value::String(s) => Some(chrono::DateTime::parse_from_rfc3339(s).or_else(|_| chrono::DateTime::parse_from_rfc2822(s)).ok()?.to_utc()),
            _ => None,
        }
    }
    // for the compatibility with the old version, we need to parse this manually
    pub fn parse_from_json(value: &serde_json::Value) -> Self {
        let code = value.get("code").and_then(|v| v.as_str()).unwrap_or_default();
        let cron = value
            .get("cron")
            .map(|v| match v {
                serde_json::Value::Array(arr) => arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect(),
                serde_json::Value::String(s) => vec![s.to_string()],
                _ => vec![],
            })
            .unwrap_or_default();
        let callback_url = value.get("callback_url").and_then(|v| v.as_str()).unwrap_or_default();
        let callback_headers = value
            .get("callback_headers")
            .map(|v| match v {
                serde_json::Value::Object(obj) => obj.iter().map(|(k, v)| (k.to_string(), v.as_str().unwrap_or_default().to_string())).collect(),
                _ => HashMap::new(),
            })
            .unwrap_or_default();
        let callback_method = value.get("callback_method").and_then(|v| v.as_str()).unwrap_or("GET");
        let callback_body = value.get("callback_body").and_then(|v| v.as_str()).map(|s| s.to_string());
        let enable_time = value.get("enable_time").and_then(ScheduleJob::parse_time_from_json_value);
        let disable_time = value.get("disable_time").and_then(ScheduleJob::parse_time_from_json_value);
        Self {
            code: code.into(),
            cron,
            callback_url: callback_url.into(),
            callback_headers,
            callback_method: callback_method.into(),
            callback_body,
            enable_time,
            disable_time,
        }
    }
    pub fn build_request(&self) -> TardisResult<tardis::web::reqwest::Request> {
        let method = Method::from_bytes(self.callback_method.as_bytes()).unwrap_or(Method::GET);
        let url = Url::parse(&self.callback_url)?;
        let mut request = tardis::web::reqwest::Request::new(method, url);
        if let Some(body) = &self.callback_body {
            request.body_mut().replace(tardis::web::reqwest::Body::from(body.to_string()));
        }
        request.headers_mut().extend(self.callback_headers.iter().filter_map(|(k, v)| Some((header::HeaderName::from_str(k).ok()?, header::HeaderValue::from_str(v).ok()?))));
        Ok(request)
    }
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
    pub cron: Vec<String>,
    pub callback_url: String,
    pub callback_headers: HashMap<String, String>,
    pub callback_method: String,
    pub callback_body: Option<String>,
    pub enable_time: Option<DateTime<Utc>>,
    pub disable_time: Option<DateTime<Utc>>,
    pub create_time: Option<chrono::DateTime<Utc>>,
    pub update_time: Option<chrono::DateTime<Utc>>,
}

#[derive(poem_openapi::Object, Deserialize, Debug, Serialize)]
pub(crate) struct KvScheduleJobItemDetailResp {
    pub key: String,
    pub value: ScheduleJob,
    pub info: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

impl TryFrom<KvItemSummaryResp> for KvScheduleJobItemDetailResp {
    type Error = TardisError;

    fn try_from(resp: KvItemSummaryResp) -> Result<Self, Self::Error> {
        let value = ScheduleJob::parse_from_json(&resp.value);
        Ok(Self {
            key: resp.key.trim_start_matches(KV_KEY_CODE).to_string(),
            value,
            info: resp.info,
            create_time: resp.create_time,
            update_time: resp.update_time,
        })
    }
}
impl TryFrom<KvItemDetailResp> for KvScheduleJobItemDetailResp {
    type Error = TardisError;

    fn try_from(resp: KvItemDetailResp) -> Result<Self, Self::Error> {
        let value = ScheduleJob::parse_from_json(&resp.value);
        Ok(Self {
            key: resp.key.trim_start_matches(KV_KEY_CODE).to_string(),
            value,
            info: resp.info,
            create_time: resp.create_time,
            update_time: resp.update_time,
        })
    }
}

impl ScheduleJobInfoResp {
    pub fn create_add_or_mod_req(&self) -> ScheduleJob {
        ScheduleJob {
            code: self.code.clone().into(),
            cron: self.cron.clone(),
            callback_url: self.callback_url.clone(),
            callback_headers: self.callback_headers.clone(),
            callback_method: self.callback_method.clone(),
            callback_body: self.callback_body.clone(),
            enable_time: self.enable_time,
            disable_time: self.disable_time,
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
