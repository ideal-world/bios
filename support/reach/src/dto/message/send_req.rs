use super::*;
use std::collections::HashMap;
use tardis::basic::dto::TardisContext;

/// 用户触达消息发送请求
#[derive(Debug, Serialize, Deserialize)]
pub struct ReachMsgSendReq {
    pub scene_code: String,
    pub receives: Vec<ReachMsgReceive>,
    pub rel_item_id: String,
    pub replace: HashMap<String, String>,
    pub own_paths: String,
}

impl ReachMsgSendReq {
    pub fn get_ctx(&self) -> TardisContext {
        TardisContext {
            own_paths: self.own_paths.clone(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReachMsgReceive {
    pub receive_group_code: String,
    pub receive_kind: ReachReceiveKind,
    pub receive_ids: Vec<String>,
}
