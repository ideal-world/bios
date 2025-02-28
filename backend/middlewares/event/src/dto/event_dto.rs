use std::num::NonZeroU32;

use asteroid_mq::prelude::{TopicCode, TopicConfig, TopicOverflowConfig, TopicOverflowPolicy};
use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumItemFilterFetcher};
use serde::{Deserialize, Serialize};
use tardis::{
    chrono::{DateTime, Utc},
    db::sea_orm::{self, FromQueryResult},
    web::poem_openapi,
};
pub(crate) fn format_code(code: &impl std::fmt::Display) -> String {
    format!("event/topic/{}", code)
}
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct EventTopicConfig {
    pub topic_code: String,
    #[oai(default)]
    pub blocking: bool,
    pub overflow_policy: Option<String>,
    #[oai(default)]
    pub overflow_size: i32,
    #[oai(default)]
    pub check_auth: bool,
    pub max_payload_size: i32,
}

impl EventTopicConfig {
    pub fn into_rbum_req(self) -> EventTopicAddOrModifyReq {
        EventTopicAddOrModifyReq {
            code: format_code(&self.topic_code),
            name: format_code(&self.topic_code),
            blocking: self.blocking,
            topic_code: self.topic_code,
            overflow_policy: self.overflow_policy.unwrap_or("RejectNew".to_string()),
            overflow_size: self.overflow_size.clamp(1, i32::MAX),
            check_auth: self.check_auth,
            max_payload_size: self.max_payload_size.clamp(1024, i32::MAX),
        }
    }
}
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone, FromQueryResult)]
pub struct EventTopicAddOrModifyReq {
    pub code: String,
    pub name: String,
    #[oai(default)]
    pub blocking: bool,
    pub topic_code: String,
    pub overflow_policy: String,
    pub overflow_size: i32,
    #[oai(default)]
    pub check_auth: bool,
    pub max_payload_size: i32,
}

impl EventTopicAddOrModifyReq {
    pub fn into_topic_config(self) -> TopicConfig {
        TopicConfig {
            code: TopicCode::new(self.topic_code),
            blocking: self.blocking,
            overflow_config: Some(TopicOverflowConfig {
                policy: match self.overflow_policy.as_str() {
                    "RejectNew" => TopicOverflowPolicy::RejectNew,
                    "DropOld" => TopicOverflowPolicy::DropOld,
                    _ => TopicOverflowPolicy::default(),
                },
                size: NonZeroU32::new(self.overflow_size.clamp(1, i32::MAX) as u32).expect("clamped"),
            }),
            max_payload_size: self.max_payload_size.clamp(1024, i32::MAX) as u32,
        }
    }
    pub fn from_config(config: TopicConfig) -> Self {
        Self {
            code: format_code(&config.code),
            name: format_code(&config.code),
            blocking: config.blocking,
            topic_code: config.code.to_string(),
            overflow_policy: config.overflow_config.as_ref().map_or("RejectNew".to_string(), |c| match c.policy {
                TopicOverflowPolicy::RejectNew => "RejectNew".to_string(),
                TopicOverflowPolicy::DropOld => "DropOld".to_string(),
            }),
            overflow_size: config.overflow_config.as_ref().map_or(0, |c| c.size.get() as i32),
            check_auth: false,
            max_payload_size: config.max_payload_size as i32,
        }
    }
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone, FromQueryResult)]
pub struct EventTopicInfoResp {
    // #[oai(validator(pattern = r"^[a-z0-9]+$"))]
    pub code: String,
    pub name: String,
    pub blocking: bool,
    pub topic_code: String,
    pub overflow_policy: String,
    pub overflow_size: i32,
    pub check_auth: bool,
    pub max_payload_size: i32
}

impl EventTopicInfoResp {
    pub fn into_topic_config(self) -> TopicConfig {
        TopicConfig {
            code: TopicCode::new(self.topic_code),
            blocking: self.blocking,
            overflow_config: Some(TopicOverflowConfig {
                policy: match self.overflow_policy.as_str() {
                    "RejectNew" => TopicOverflowPolicy::RejectNew,
                    "DropOld" => TopicOverflowPolicy::DropOld,
                    _ => TopicOverflowPolicy::default(),
                },
                size: NonZeroU32::new(self.overflow_size.clamp(1, i32::MAX) as u32).expect("clamped"),
            }),
            max_payload_size: self.max_payload_size as u32
        }
    }
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct EventTopicFilterReq {
    pub basic: RbumBasicFilterReq,
    pub topic_code: Option<String>,
}

impl RbumItemFilterFetcher for EventTopicFilterReq {
    fn basic(&self) -> &RbumBasicFilterReq {
        &self.basic
    }

    fn rel(&self) -> &Option<bios_basic::rbum::dto::rbum_filer_dto::RbumItemRelFilterReq> {
        &None
    }

    fn rel2(&self) -> &Option<bios_basic::rbum::dto::rbum_filer_dto::RbumItemRelFilterReq> {
        &None
    }
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct EventRegisterResp {
    pub node_id: String,
    pub expire_at: DateTime<Utc>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct TopicAuth {
    pub topic: String,
    pub ak: String,
    pub read: bool,
    pub write: bool,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct SetTopicAuth {
    pub topic: String,
    pub read: bool,
    pub write: bool,
}
