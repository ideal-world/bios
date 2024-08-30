use std::num::NonZeroU32;

use asteroid_mq::prelude::{TopicCode, TopicConfig, TopicOverflowConfig, TopicOverflowPolicy};
use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumItemFilterFetcher};
use serde::{Deserialize, Serialize};
use tardis::{
    basic::field::TrimString,
    db::sea_orm::{self, FromQueryResult},
    serde_json::Value,
    web::poem_openapi,
};

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct EventTopicConfig {
    pub code: String,
    pub blocking: bool,
    pub topic_code: String,
    pub overflow_policy: String,
    pub overflow_size: i32,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone, FromQueryResult)]

pub struct EventTopicAddOrModifyReq {
    pub code: String,
    pub name: String,
    pub blocking: bool,
    pub topic_code: String,
    pub overflow_policy: String,
    pub overflow_size: i32,
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
        }
    }
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct EventTopicFilterReq {
    pub basic: RbumBasicFilterReq,
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
pub struct EventListenerRegisterReq {
    // #[oai(validator(pattern = r"^[a-z0-9]+$"))]
    pub topic_code: TrimString,
    pub topic_sk: Option<String>,
    // #[oai(validator(pattern = r"^[a-z0-9-_]+$"))]
    pub events: Option<Vec<TrimString>>,
    pub avatars: Vec<TrimString>,
    pub subscribe_mode: bool,
}
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct EventListenerRegisterResp {
    pub ws_addr: String,
    pub listener_code: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EventListenerInfo {
    pub topic_code: String,
    pub subscribe_mode: bool,
    pub events: Option<Vec<String>>,
    pub avatars: Vec<String>,
    pub mgr: bool,
    pub token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventMessageMgrWrap {
    pub msg: Value,
    pub ori_from_avatar: String,
    pub ori_to_avatars: Option<Vec<String>>,
}
