use super::*;
use std::collections::HashMap;

/// 用户触达消息发送请求
#[derive(Debug, Serialize, Deserialize, poem_openapi::Object)]
pub struct ReachMsgSendReq {
    pub scene_code: String,
    pub receives: Vec<ReachMsgReceive>,
    pub rel_item_id: String,
    pub replace: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, poem_openapi::Object)]
pub struct ReachMsgReceive {
    pub receive_group_code: String,
    pub receive_kind: ReachReceiveKind,
    pub receive_ids: Vec<String>,
}
