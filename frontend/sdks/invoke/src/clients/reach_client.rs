use std::collections::HashMap;

use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    web::web_resp::TardisResp,
    TardisFunsInst,
};

use crate::dto::reach_item_dto::ReachTriggerInstanceConfigSummaryResp;
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
    pub async fn batch_send_message(reqs: &Vec<ReachMsgSendReq>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let reach_url: String = BaseSpiClient::module_url(InvokeModuleKind::Reach, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;
        funs.web_client().put_obj_to_str(&format!("{reach_url}/ci/message/batch/send"), &reqs, headers.clone()).await?;
        Ok(())
    }
    pub async fn general_send(to: &str, template_id: &str, replacement: &HashMap<String, String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let reach_url: String = BaseSpiClient::module_url(InvokeModuleKind::Reach, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;
        funs.web_client().put_obj_to_str(&format!("{reach_url}/cc/msg/general/{to}/template/{template_id}"), &replacement, headers.clone()).await?;
        Ok(())
    }

    pub async fn send_vcode(to: &str, vcode: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let reach_url: String = BaseSpiClient::module_url(InvokeModuleKind::Reach, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;
        funs.web_client().put_str_to_str(&format!("{reach_url}/cc/msg/vcode/{to}/{vcode}"), "", headers.clone()).await?;
        Ok(())
    }

    pub async fn send_pwd(to: &str, pwd: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let reach_url: String = BaseSpiClient::module_url(InvokeModuleKind::Reach, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;
        funs.web_client().put_str_to_str(&format!("{reach_url}/cc/msg/pwd/{to}/{pwd}"), "", headers.clone()).await?;
        Ok(())
    }

    /// Find all user reach trigger instance config data
    /// 根据类型获取所有用户触达触发实例配置数据
    pub async fn find_trigger_instance_config(
        rel_item_id: &str,
        channel: &str,
        scene_code: Option<&str>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Option<Vec<ReachTriggerInstanceConfigSummaryResp>>> {
        let reach_url: String = BaseSpiClient::module_url(InvokeModuleKind::Reach, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;
        let mut url = format!("{reach_url}/ci/trigger/instance/config?rel_item_id={rel_item_id}&channel={channel}");
        if let Some(scene_code) = scene_code {
            url = format!("{url}&scene_code={scene_code}");
        }
        let resp = funs.web_client().get::<TardisResp<Vec<ReachTriggerInstanceConfigSummaryResp>>>(&url, headers).await?;
        BaseSpiClient::package_resp(resp)
    }
}
