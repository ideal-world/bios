use bios_basic::rbum::dto::{rbum_filer_dto::RbumItemBasicFilterReq, rbum_item_dto::RbumItemAddReq};

use serde::{Serialize, Deserialize};
use tardis::{
    chrono::{DateTime, Utc},
    db::sea_orm,
    web::poem_openapi,
};

#[derive(Debug, poem_openapi::Object, Serialize, Deserialize)]
pub struct ReachMsgLogAddReq {
    #[oai(flatten)]
    #[serde(flatten)]
    pub rbum_add_req: RbumItemAddReq,
    pub rel_account_id: String,
    pub dnd_time: String,
    pub dnd_strategy: ReachDndStrategyKind,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub failure: bool,
    pub fail_message: String,
    pub rel_reach_message_id: String,
}

use super::ReachDndStrategyKind;
#[derive(Debug, poem_openapi::Object, Default, Serialize, Deserialize)]
pub struct ReachMsgLogFilterReq {
    #[oai(flatten)]
    #[serde(flatten)]
    pub base_filter: RbumItemBasicFilterReq,
    pub rel_reach_message_id: Option<String>,
}

#[derive(Debug, poem_openapi::Object, Serialize, Deserialize)]
pub struct ReachMsgLogModifyReq {
    /// 关联接收人Id
    pub rel_account_id: String,
    /// 免扰时间，ISO 8601 time without timezone.
    pub dnd_time: String,
    /// 免扰策略
    pub dnd_strategy: ReachDndStrategyKind,
    /// 开始时间
    pub start_time: DateTime<Utc>,
    /// 结束时间
    pub end_time: DateTime<Utc>,
    /// 是否失败
    pub failure: bool,
    /// 失败原因
    pub fail_message: String,
}

#[derive(Debug, poem_openapi::Object, Serialize, Deserialize, sea_orm::FromQueryResult)]
pub struct ReachMsgLogSummaryResp {
    pub id: String,
    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    /// 关联接收人Id
    pub rel_account_id: String,
    /// 免扰时间，ISO 8601 time without timezone.
    pub dnd_time: String,
    /// 免扰策略
    pub dnd_strategy: ReachDndStrategyKind,
    /// 开始时间
    pub start_time: DateTime<Utc>,
    /// 结束时间
    pub end_time: DateTime<Utc>,
    /// 是否失败
    pub failure: bool,
    /// 失败原因
    pub fail_message: String,
    /// 用户触达消息Id
    pub rel_reach_message_id: String,
}

#[derive(Debug, poem_openapi::Object, Serialize, Deserialize, sea_orm::FromQueryResult)]
pub struct ReachMsgLogDetailResp {
    pub id: String,
    pub own_paths: String,
    pub owner: String,
    pub owner_name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    /// 关联接收人Id
    pub rel_account_id: String,
    /// 免扰时间，ISO 8601 time without timezone.
    pub dnd_time: String,
    /// 免扰策略
    pub dnd_strategy: ReachDndStrategyKind,
    /// 开始时间
    pub start_time: DateTime<Utc>,
    /// 结束时间
    pub end_time: DateTime<Utc>,
    /// 是否失败
    pub failure: bool,
    /// 失败原因
    pub fail_message: String,
    /// 用户触达消息Id
    pub rel_reach_message_id: String,
}
