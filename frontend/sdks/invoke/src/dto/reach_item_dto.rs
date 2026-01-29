use serde::{Deserialize, Serialize};
use tardis::{
    chrono::{DateTime, Utc},
    web::poem_openapi,
};

/// 触达通道类型
#[derive(Debug, poem_openapi::Enum, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[oai(rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReachChannelKind {
    #[default]
    Sms,
    Email,
    Inbox,
    Wechat,
    DingTalk,
    Push,
    WebHook,
}

/// 用户触达触发实例配置摘要响应（与后端 ReachTriggerInstanceConfigSummaryResp 一致）
#[derive(Debug, poem_openapi::Object, Serialize, Deserialize, Clone)]
pub struct ReachTriggerInstanceConfigSummaryResp {
    pub id: String,
    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    /// 关联的触发场景id
    pub rel_reach_trigger_scene_id: String,
    /// 关联的触达通道
    pub rel_reach_channel: ReachChannelKind,
    /// 关联资源项id
    pub rel_item_id: String,
    /// 接收组编码
    pub receive_group_code: String,
    /// 接收组名称
    pub receive_group_name: String,
}
