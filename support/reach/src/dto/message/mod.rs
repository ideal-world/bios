mod log;
pub use log::*;
mod send_req;
pub use send_req::*;
mod signature;
pub use signature::*;
mod template;
pub use template::*;
use std::collections::HashMap;

use bios_basic::rbum::dto::{
    rbum_filer_dto::RbumItemBasicFilterReq,
    rbum_item_dto::RbumItemAddReq,
    rbum_safe_dto::{RbumSafeDetailResp, RbumSafeSummaryResp},
};
use serde::Serialize;
use tardis::{regex::Regex, web::poem_openapi, db::sea_orm::{self, TryGetable}, chrono::{DateTime, Utc}, basic::result::TardisResult};

use super::*;

lazy_static::lazy_static! {
    static ref EXTRACT_R: Regex = Regex::new(r"(\[^}]+?})").unwrap();
}
const DEFUALT_MAXLEN: usize = 20;
fn content_replace<const MAXLEN: usize>(content: &str, values: &HashMap<String, String>) -> String {
    let mut new_content = content.to_string();
    let matcher = EXTRACT_R.find_iter(content);
    for mat in matcher {
        let key = &content[mat.start() + 1..mat.end() - 1];
        if let Some(value) = values.get(key) {
            let replace_value = if value.len() > MAXLEN {
                format!("{}...", &value[(MAXLEN - 3)..])
            } else {
                value.to_string()
            };
            new_content = new_content.replacen(mat.as_str(), &replace_value, 1);
        }
    }
    new_content
}

// Request

#[derive(Debug, poem_openapi::Object)]
pub struct ReachMessageAddReq {
    #[oai(flatten)]
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
    /// 触达状态
    pub reach_status: ReachStatusKind,
    /// 触达状态
    pub content_replace: String,
}

#[derive(Debug, poem_openapi::Object)]
pub struct ReachMessageModifyReq {
    /// 发件人
    #[oai(validator(max_length = "2000"))]
    from_res: String,
    /// 关联的触达通道
    rel_reach_channel: Option<ReachChannelKind>,
    /// 用户触达接收类型
    receive_kind: ReachReceiveKind,
    /// 接收主体，分号分隔
    #[oai(validator(max_length = "2000"))]
    to_res_ids: String,
    /// 用户触达签名Id
    #[oai(validator(max_length = "255"))]
    rel_reach_msg_signature_id: String,
    /// 用户触达模板Id
    #[oai(validator(max_length = "255"))]
    rel_reach_msg_template_id: String,
    /// 触达状态
    reach_status: ReachStatusKind,
    /// 触达状态
    content_replace: HashMap<String, String>,
}
#[derive(Debug, poem_openapi::Object)]
pub struct ReachMessageFilterReq {
    #[oai(flatten)]
    rbum_item_basic_filter_req: RbumItemBasicFilterReq,
    reach_status: Option<ReachStatusKind>,
}

#[derive(Debug, poem_openapi::Object, Serialize, sea_orm::FromQueryResult)]
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



#[derive(Debug, poem_openapi::Object, Serialize, sea_orm::FromQueryResult)]
pub struct ReachMessageDetailResp {
    pub id: String,
    pub own_paths: String,
    pub owner: String,
    pub owner_name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    #[oai(validator(max_length = "2000"))]
    from_res: String,
    rel_reach_channel: ReachChannelKind,
    receive_kind: ReachReceiveKind,
    #[oai(validator(max_length = "2000"))]
    /// 接收主体，分号分隔
    to_res_ids: String,
    #[oai(validator(max_length = "255"))]
    rel_reach_msg_signature_id: String,
    #[oai(validator(max_length = "255"))]
    rel_reach_msg_template_id: String,
    reach_status: ReachStatusKind,
    content_replace: String,
    template_content: String,
    template_name: String,
}
