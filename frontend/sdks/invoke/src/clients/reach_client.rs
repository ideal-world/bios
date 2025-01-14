use std::collections::HashMap;

use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    serde_json::json,
    TardisFunsInst,
};

use crate::invoke_enumeration::InvokeModuleKind;

use super::base_spi_client::BaseSpiClient;
use serde::Serialize;
#[derive(Clone, Debug, Default)]
pub struct ReachClient;

#[derive(Debug, Serialize, Clone)]
pub struct ReachMsgSendReq {
    pub scene_code: String,
    pub receives: Vec<ReachMsgReceive>,
    pub rel_item_id: String,
    pub replace: HashMap<String, String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ReachMsgReceive {
    pub receive_group_code: String,
    pub receive_kind: String,
    pub receive_ids: Vec<String>,
}
impl ReachClient {
    pub async fn send_message(req: &ReachMsgSendReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let reach_url: String = BaseSpiClient::module_url(InvokeModuleKind::Reach, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;
        funs.web_client().put_obj_to_str(&format!("{reach_url}/ci/message/send"), &req, headers.clone()).await?;
        Ok(())
    }
    pub async fn general_send(
        to: &str,
        template_id: &str,
        replacement: &HashMap<String, String>,
        funs: &TardisFunsInst, 
        ctx: &TardisContext
    ) -> TardisResult<()> {
        let reach_url: String = BaseSpiClient::module_url(InvokeModuleKind::Reach, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;
        funs.web_client().put_obj_to_str(&format!("{reach_url}/cc/msg/general/{to}/template/{template_id}"), &replacement, headers.clone()).await?;
        Ok(())
    }
}
