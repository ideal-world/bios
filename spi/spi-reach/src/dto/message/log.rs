use bios_basic::rbum::{
    dto::{rbum_item_dto::{RbumItemAddReq, RbumItemKernelAddReq}, rbum_filer_dto::RbumItemBasicFilterReq, rbum_safe_dto::RbumSafeSummaryResp},
    rbum_enumeration::RbumScopeLevelKind,
};

use tardis::{
    basic::field::TrimString,
    chrono::{DateTime, Utc},
    web::poem_openapi,
};

#[derive(Debug, poem_openapi::Object)]
pub struct ReachMsgLogAddReq {
    #[oai(flatten)]
    pub rbum_add_req: RbumItemAddReq,
    pub rel_account_id: String,
    pub dnd_time: [NaiveTime; 2],
    pub dnd_strategy: ReachDndStrategyKind,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub failure: bool,
    pub fail_message: String,
    pub rel_reach_message_id: String,
}
use tardis::chrono::NaiveTime;

use super::{ReachStatusKind, ReachDndStrategyKind};
#[derive(Debug, poem_openapi::Object)]
pub struct ReachMsgLogFilterReq {
    #[oai(flatten)]
    pub rbum_item_basic_filter_req: RbumItemBasicFilterReq,
    pub reach_status: Option<ReachStatusKind>,
}

#[derive(Debug, poem_openapi::Object)]
pub struct ReachMsgLogModifyReq {
    /// 关联接收人Id
    pub rel_account_id: String,
    /// 免扰时间，ISO 8601 time without timezone.
    pub dnd_time: [NaiveTime; 2],
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

#[derive(Debug, poem_openapi::Object)]
pub struct ReachMsgLogSummaryResp {
    #[oai(flatten)]
    pub rbum_safe_summary_resp: RbumSafeSummaryResp,
    /// 关联接收人Id
    pub rel_account_id: String,
    /// 免扰时间，ISO 8601 time without timezone.
    pub dnd_time: [NaiveTime; 2],
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

#[derive(Debug, poem_openapi::Object)]
pub struct ReachMsgLogDetailResp {
    #[oai(flatten)]
    pub rbum_safe_detail_resp: RbumSafeSummaryResp,
    /// 关联接收人Id
    pub rel_account_id: String,
    /// 免扰时间，ISO 8601 time without timezone.
    pub dnd_time: [NaiveTime; 2],
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