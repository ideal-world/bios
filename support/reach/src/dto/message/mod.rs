mod log;
pub use log::*;
mod send_req;
pub use send_req::*;
mod signature;
pub use signature::*;
mod template;
pub use template::*;

use bios_basic::rbum::dto::{rbum_filer_dto::RbumItemBasicFilterReq, rbum_item_dto::RbumItemAddReq};
use serde::{Deserialize, Serialize};
use tardis::{
    chrono::{DateTime, Utc},
    db::sea_orm,
    web::poem_openapi,
};

use super::*;

// Request

#[derive(Debug, poem_openapi::Object, Deserialize, Serialize)]
pub struct ReachMessageAddReq {
    #[oai(flatten)]
    #[serde(flatten)]
    pub rbum_item_add_req: RbumItemAddReq,
    /// 发件人
    #[oai(validator(max_length = "2000"))]
    pub from_res: String,
    /// 关联的触达通道
    pub rel_reach_channel: ReachChannelKind,
    /// 用户触达接收类型
    pub receive_kind: ReachReceiveKind,
    #[oai(validator(max_length = "2000"))]
    /// 接收主体，分号分隔
    pub to_res_ids: String,
    #[oai(validator(max_length = "255"))]
    /// 用户触达签名Id
    pub rel_reach_msg_signature_id: String,
    #[oai(validator(max_length = "255"))]
    /// 用户触达模板Id
    pub rel_reach_msg_template_id: String,
    #[serde(default)]
    #[oai(default)]
    /// 触达状态
    pub reach_status: ReachStatusKind,
    /// 触达状态
    pub content_replace: String,
}

#[derive(Debug, poem_openapi::Object, Default, Deserialize, Serialize)]
pub struct ReachMessageModifyReq {
    /// 发件人
    #[oai(validator(max_length = "2000"))]
    pub from_res: Option<String>,
    /// 关联的触达通道
    pub rel_reach_channel: Option<ReachChannelKind>,
    /// 用户触达接收类型
    pub receive_kind: Option<ReachReceiveKind>,
    /// 接收主体，分号分隔
    #[oai(validator(max_length = "2000"))]
    pub to_res_ids: Option<String>,
    /// 用户触达签名Id
    #[oai(validator(max_length = "255"))]
    pub rel_reach_msg_signature_id: Option<String>,
    /// 用户触达模板Id
    #[oai(validator(max_length = "255"))]
    pub rel_reach_msg_template_id: Option<String>,
    /// 触达状态
    pub reach_status: Option<ReachStatusKind>,
    /// 触达状态
    pub content_replace: Option<String>,
}
#[derive(Debug, poem_openapi::Object, Default, Serialize, Deserialize)]
pub struct ReachMessageFilterReq {
    #[oai(flatten)]
    #[serde(flatten)]
    pub rbum_item_basic_filter_req: RbumItemBasicFilterReq,
    pub reach_status: Option<ReachStatusKind>,
}

#[derive(Debug, poem_openapi::Object, Serialize, Deserialize, sea_orm::FromQueryResult)]
pub struct ReachMessageSummaryResp {
    pub id: String,
    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    // pub rbum_safe_summary_resp: RbumSafeSummaryResp,
    #[oai(validator(max_length = "2000"))]
    pub from_res: String,
    pub rel_reach_channel: ReachChannelKind,
    pub receive_kind: ReachReceiveKind,
    #[oai(validator(max_length = "2000"))]
    pub to_res_ids: String,
    #[oai(validator(max_length = "255"))]
    pub rel_reach_msg_signature_id: String,
    #[oai(validator(max_length = "255"))]
    pub rel_reach_msg_template_id: String,
    pub reach_status: ReachStatusKind,
    pub content_replace: String,
    pub template_content: String,
    pub template_name: String,
}

#[derive(Debug, poem_openapi::Object, Serialize, Deserialize, sea_orm::FromQueryResult)]
pub struct ReachMessageDetailResp {
    pub id: String,
    pub own_paths: String,
    pub owner: String,
    pub owner_name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    #[oai(validator(max_length = "2000"))]
    pub from_res: String,
    pub rel_reach_channel: ReachChannelKind,
    pub receive_kind: ReachReceiveKind,
    #[oai(validator(max_length = "2000"))]
    /// 接收主体，分号分隔
    pub to_res_ids: String,
    #[oai(validator(max_length = "255"))]
    pub rel_reach_msg_signature_id: String,
    #[oai(validator(max_length = "255"))]
    pub rel_reach_msg_template_id: String,
    pub reach_status: ReachStatusKind,
    pub content_replace: String,
    pub template_content: String,
    pub template_name: String,
}
