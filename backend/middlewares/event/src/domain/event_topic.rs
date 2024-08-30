use std::num::NonZeroU32;
use tardis::db::sea_orm;
use tardis::db::sea_orm::prelude::*;
use tardis::db::sea_orm::*;

use asteroid_mq::prelude::{TopicCode, TopicConfig, TopicOverflowConfig, TopicOverflowPolicy};
use tardis::{TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "mq_topic")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: String,
    pub blocking: bool,
    pub topic_code: String,
    pub overflow_policy: String,
    pub overflow_size: i32,
    #[fill_ctx]
    pub own_paths: String,
}

impl Model {
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
