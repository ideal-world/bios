use std::collections::HashMap;

use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use serde_json::Value;
use tardis::{basic::{dto::TardisContext, result::TardisResult}, TardisFunsInst, chrono::{Utc, SecondsFormat}};

use crate::dto::{flow_model_dto::FlowModelFilterReq, flow_inst_dto::{FlowInstTransferReq, FlowInstDetailResp}, flow_external_dto::FlowExternalCallbackOp, flow_transition_dto::{FlowTransitionFrontActionInfo, FlowTransitionFrontActionRightValue}};

use super::{flow_model_serv::FlowModelServ, flow_inst_serv::FlowInstServ};
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use itertools::Itertools;

pub struct FlowEventServ;

impl FlowEventServ {
    pub async fn do_front_change(flow_inst_id: &str, ctx: &TardisContext, funs: &TardisFunsInst) -> TardisResult<()> {
        let flow_inst_detail = FlowInstServ::get(flow_inst_id, funs, ctx).await?;
        let flow_model = FlowModelServ::get_item(
            &flow_inst_detail.rel_flow_model_id,
            &FlowModelFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    own_paths: Some("".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        let flow_transitions = flow_model
            .transitions()
            .into_iter()
            .filter(|trans| trans.from_flow_state_id == flow_inst_detail.current_state_id && !trans.action_by_front_changes().is_empty())
            .sorted_by_key(|trans| trans.sort)
            .collect_vec();
        if flow_transitions.is_empty() {
            return Ok(());
        }
        for flow_transition in flow_transitions {
            if Self::check_front_conditions(&flow_inst_detail, flow_transition.action_by_front_changes())? {
                FlowInstServ::transfer(
                    &flow_inst_detail.id,
                    &FlowInstTransferReq {
                        flow_transition_id: flow_transition.id.clone(),
                        message: None,
                        vars: None,
                    },
                    true,
                    FlowExternalCallbackOp::ConditionalTrigger,
                    funs,
                    ctx,
                )
                .await?;
                break;
            }
        }

        Ok(())
    }

    fn check_front_conditions(flow_inst_detail: &FlowInstDetailResp, conditions: Vec<FlowTransitionFrontActionInfo>) -> TardisResult<bool> {
        if flow_inst_detail.current_vars.is_none() {
            return Ok(false);
        }
        let current_vars = flow_inst_detail.current_vars.clone().unwrap();
        for condition in conditions {
            if !Self::do_check_front_condition(&current_vars, &condition)? {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn do_check_front_condition(current_vars: &HashMap<String, Value>, condition: &FlowTransitionFrontActionInfo) -> TardisResult<bool> {
        match condition.right_value {
            FlowTransitionFrontActionRightValue::ChangeContent => {
                if let Some(left_value) = current_vars.get(&condition.left_value) {
                    Ok(condition.relevance_relation.check_conform(
                        left_value.as_str().unwrap_or(left_value.to_string().as_str()).to_string(),
                        condition
                            .change_content
                            .clone()
                            .unwrap_or_default()
                            .as_str()
                            .unwrap_or(condition.change_content.clone().unwrap_or_default().to_string().as_str())
                            .to_string(),
                    ))
                } else {
                    Ok(false)
                }
            }
            FlowTransitionFrontActionRightValue::SelectField => {
                if let (Some(left_value), Some(right_value)) = (
                    current_vars.get(&condition.left_value),
                    current_vars.get(&condition.select_field.clone().unwrap_or_default()),
                ) {
                    Ok(condition.relevance_relation.check_conform(
                        left_value.as_str().unwrap_or(left_value.to_string().as_str()).to_string(),
                        right_value.as_str().unwrap_or(left_value.to_string().as_str()).to_string(),
                    ))
                } else {
                    Ok(false)
                }
            }
            FlowTransitionFrontActionRightValue::RealTime => {
                if let Some(left_value) = current_vars.get(&condition.left_value) {
                    Ok(condition.relevance_relation.check_conform(
                        left_value.as_str().unwrap_or(left_value.to_string().as_str()).to_string(),
                        Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
                    ))
                } else {
                    Ok(false)
                }
            }
        }
    }

    pub async fn do_post_change(flow_inst_id: &str, flow_transition_id: &str, ctx: &TardisContext, funs: &TardisFunsInst) -> TardisResult<()> {
        let flow_inst_detail = FlowInstServ::get(flow_inst_id, funs, ctx).await?;
        let flow_model = FlowModelServ::get_item(
            &flow_inst_detail.rel_flow_model_id,
            &FlowModelFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    own_paths: Some("".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        let flow_transitions = flow_model
            .transitions()
            .into_iter()
            .filter(|trans| trans.from_flow_state_id == flow_inst_detail.current_state_id && !trans.action_by_front_changes().is_empty())
            .sorted_by_key(|trans| trans.sort)
            .collect_vec();
        if flow_transitions.is_empty() {
            return Ok(());
        }
        let next_flow_transition = FlowInstServ::do_find_next_transitions(
            &flow_inst_detail,
            &flow_model,
            Some(flow_transition_id.to_string()),
            &None,
            true,
            funs,
            ctx,
        )
        .await?
        .next_flow_transitions
        .pop();
        if next_flow_transition.is_none() {
            return Err(funs.err().not_found("flow_inst", "transfer", "no transferable state", "404-flow-inst-transfer-state-not-found"));
        }
        let next_flow_transition = next_flow_transition.unwrap();
        
        let model_transition = flow_model.transitions();
        let next_transition_detail = model_transition.iter().find(|trans| trans.id == flow_transition_id).unwrap().to_owned();
        if FlowModelServ::check_post_action_ring(next_transition_detail.clone(), (false, vec![]), funs, ctx).await?.0 {
            return Err(funs.err().not_found("flow_inst", "transfer", "this post action exist endless loop", "500-flow-transition-endless-loop"));
        }

        let post_changes = model_transition.into_iter().find(|model_transition| model_transition.id == next_flow_transition.next_flow_transition_id).unwrap_or_default().action_by_post_changes();
        Ok(())
    }
}