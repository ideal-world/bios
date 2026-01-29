use std::collections::HashMap;

use bios_basic::rbum::helper::rbum_scope_helper;
use bios_sdk_invoke::clients::{
    reach_client::{ReachClient, ReachMsgSendReq},
    spi_kv_client::SpiKvClient,
};
use bios_sdk_invoke::dto::reach_item_dto::ReachTriggerInstanceConfigSummaryResp;
use itertools::Itertools;
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    TardisFunsInst,
};

use crate::serv::{clients::kv_client::FlowKvClient, flow_inst_serv::FlowInstServ};

const REACH_APPROVE_FINISH_TAG: &str = "flow_approve_finish";
const REACH_APPROVE_START_TAG: &str = "flow_approve_start";
const REACH_REVIEW_REMIND_TAG: &str = "flow_review_remind";
const REACH_REVIEW_START_TAG: &str = "flow_review_start";

pub struct FlowReachClient;

impl FlowReachClient {
    async fn send_message(req: &ReachMsgSendReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        ReachClient::send_message(req, funs, ctx).await
    }

    async fn batch_send_message(reqs: &Vec<ReachMsgSendReq>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        ReachClient::batch_send_message(reqs, funs, ctx).await
    }

    pub async fn send_approve_start_message(inst_id: &str, ctx: &TardisContext, funs: &TardisFunsInst) -> TardisResult<()> {
        let inst = FlowInstServ::get(inst_id, funs, ctx).await?;
        let rel_item_id = ctx.own_paths.split(":").nth(2).unwrap_or_default();
        let trigger_instance_config = Self::find_trigger_instance_config(rel_item_id, "SMS", Some(REACH_APPROVE_START_TAG), funs, ctx).await?;
        if let Some(trigger_instance_config) = trigger_instance_config {
            let mut reqs = Vec::new();
            for config in trigger_instance_config {
                let mut replace = HashMap::new();
                let username = FlowKvClient::get_account_name(&inst.create_ctx.owner, funs, ctx).await?;
                let product_name = FlowKvClient::get_product_name(rel_item_id, funs, ctx).await?;
                let create_vars = inst.create_vars.clone().unwrap_or_default();
                let kind = create_vars.get("on_line").map(|v| if v.to_string() == "true" { "线上" } else { "线下" }).unwrap_or_default();
                replace.insert("username".to_string(), username);
                replace.insert("productName".to_string(), product_name);
                replace.insert("feedName".to_string(), create_vars.get("name").map(|v| v.to_string()).unwrap_or_default());
                replace.insert("time".to_string(), create_vars.get("review_start_time").map(|v| v.to_string()).unwrap_or_default());
                replace.insert("kind".to_string(), kind.to_string());

                if config.receive_group_code == "CREATOR" { // 创建人接收组
                    // let mut req = ReachMsgSendReq {
                    //     scene_code: config.scene_code,
                    //     receives: vec![ReachMsgReceive {
                    //         receive_group_code: config.receive_group_code,
                    //         receive_kind: ReachReceiveKind::Account,
                    //         receive_ids: config.receive_ids,
                    //     }],
                    //     rel_item_id: rel_item_id.to_string(),
                    //     replace,
                    // };
                    // reqs.push(req);
                }
                if config.receive_group_code == "INITIATOR" { // 发起人接收组
                    
                }
                if config.receive_group_code == "REVIEW_MEMBER" { // 发起人接收组
                    
                }
            }
            Self::batch_send_message(&reqs, funs, ctx).await?;
        }
        Ok(())
    }

    /// 根据类型获取所有用户触达触发实例配置数据
    pub async fn find_trigger_instance_config(
        rel_item_id: &str,
        channel: &str,
        scene_code: Option<&str>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Option<Vec<ReachTriggerInstanceConfigSummaryResp>>> {
        ReachClient::find_trigger_instance_config(rel_item_id, channel, scene_code, funs, ctx).await
    }
}
