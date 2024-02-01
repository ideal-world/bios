use std::iter;

use serde::{Deserialize, Serialize};
use tardis::{
    basic::result::TardisResult,
    web::{
        poem_openapi::{self, Object},
        web_resp::{TardisResp, Void},
    },
    TardisFunsInst,
};

#[derive(Clone)]
pub struct EventClient<'a> {
    pub funs: &'a TardisFunsInst,
    pub base_url: &'a str,
}

impl<'a> EventClient<'a> {
    pub fn new(url: &'a str, funs: &'a TardisFunsInst) -> Self {
        Self { base_url: url, funs }
    }

    pub async fn register(&self, req: &EventListenerRegisterReq) -> TardisResult<EventListenerRegisterResp> {
        let url = format!("{}/listener", self.base_url.trim_end_matches('/'));

        let resp = self.funs.web_client().post::<EventListenerRegisterReq, TardisResp<EventListenerRegisterResp>>(&url, req, iter::empty()).await?;
        if let Some(resp) = resp.body {
            if let Some(data) = resp.data {
                return Ok(data);
            } else {
                return Err(self.funs.err().internal_error("event", "register", &resp.msg, ""));
            }
        }
        return Err(self.funs.err().internal_error("event", "register", "failed to register event listener", ""));
    }

    pub async fn remove(&self, listener_code: &str, token: &str) -> TardisResult<()> {
        let url = format!("{}/listener/{}?token={}", self.base_url.trim_end_matches('/'), listener_code, token);
        let resp = self.funs.web_client().delete::<TardisResp<Void>>(&url, iter::empty()).await?;
        if let Some(resp) = resp.body {
            if resp.data.is_some() {
                return Ok(());
            } else {
                return Err(self.funs.err().internal_error("event", "register", &resp.msg, ""));
            }
        }
        return Err(self.funs.err().internal_error("event", "register", "failed to register event listener", ""));
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventListenerRegisterReq {
    // #[oai(validator(pattern = r"^[a-z0-9]+$"))]
    pub topic_code: String,
    pub topic_sk: Option<String>,
    // #[oai(validator(pattern = r"^[a-z0-9-_]+$"))]
    pub events: Option<Vec<String>>,
    pub avatars: Vec<String>,
    pub subscribe_mode: bool,
}

#[derive(Serialize, Deserialize, Debug, Object)]
pub struct EventListenerRegisterResp {
    pub ws_addr: String,
    pub listener_code: String,
}

// GLOBAL EVENT BUS
pub const TOPIC_EVENT_BUS: &str = "event_bus";

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(default)]
pub struct EventTopicConfig {
    pub topic_code: String,
    pub topic_sk: Option<String>,
    pub events: Option<Vec<String>>,
    pub avatars: Vec<String>,
    pub subscribe_mode: bool,
    pub base_url: String,
    pub in_event: bool,
}

impl From<EventTopicConfig> for EventListenerRegisterReq {
    fn from(val: EventTopicConfig) -> Self {
        EventListenerRegisterReq {
            topic_code: val.topic_code,
            topic_sk: val.topic_sk,
            events: val.events,
            avatars: val.avatars,
            subscribe_mode: val.subscribe_mode,
        }
    }
}
