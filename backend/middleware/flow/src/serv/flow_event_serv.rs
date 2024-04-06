use std::collections::HashMap;

use async_recursion::async_recursion;
use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use serde_json::{json, Value};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    chrono::{SecondsFormat, Utc},
    db::sea_orm::{
        self,
        sea_query::{Expr, Query},
        Set,
    },
    TardisFunsInst,
};

use crate::{
    domain::flow_inst,
    dto::{
        flow_external_dto::{FlowExternalCallbackOp, FlowExternalParams},
        flow_inst_dto::{FlowInstDetailResp, FlowInstTransferReq},
        flow_model_dto::{FlowModelDetailResp, FlowModelFilterReq},
        flow_state_dto::FlowStateFilterReq,
        flow_transition_dto::{
            FlowTransitionActionByStateChangeInfo, FlowTransitionActionByVarChangeInfoChangedKind, FlowTransitionActionChangeAgg, FlowTransitionActionChangeKind,
            FlowTransitionFrontActionInfo, FlowTransitionFrontActionRightValue, StateChangeConditionOp, TagRelKind,
        },
    },
    flow_config::FlowConfig,
    flow_initializer::{default_flow_avatar, ws_flow_client},
};

use super::{
    clients::event_client::FlowEventExt, flow_external_serv::FlowExternalServ, flow_inst_serv::FlowInstServ, flow_model_serv::FlowModelServ, flow_state_serv::FlowStateServ,
};
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use itertools::Itertools;

pub struct FlowEventServ;

impl FlowEventServ {
    #[async_recursion]
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
        let global_ctx = TardisContext {
            own_paths: "".to_string(),
            ..ctx.clone()
        };
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
        // let flow_transitions = flow_model
        //     .transitions()
        //     .into_iter()
        //     .filter(|trans| trans.from_flow_state_id == flow_inst_detail.current_state_id && !trans.action_by_post_changes().is_empty())
        //     .sorted_by_key(|trans| trans.sort)
        //     .collect_vec();
        // if flow_transitions.is_empty() {
        //     return Ok(());
        // }
        // let next_flow_transition =
        //     FlowInstServ::do_find_next_transitions(&flow_inst_detail, &flow_model, Some(flow_transition_id.to_string()), &None, true, funs, ctx).await?.next_flow_transitions.pop();
        let next_flow_transition = flow_model.transitions().into_iter().find(|trans| trans.id == flow_transition_id);
        if next_flow_transition.is_none() {
            return Err(funs.err().not_found("flow_inst", "transfer", "no transferable state", "404-flow-inst-transfer-state-not-found"));
        }
        let next_flow_transition = next_flow_transition.unwrap();
        let prev_flow_state = FlowStateServ::get_item(
            &flow_inst_detail.current_state_id,
            &FlowStateFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            &global_ctx,
        )
        .await?;
        let next_flow_state = FlowStateServ::get_item(
            &next_flow_transition.to_flow_state_id,
            &FlowStateFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            &global_ctx,
        )
        .await?;

        let model_transition = flow_model.transitions();
        let next_transition_detail = model_transition.iter().find(|trans| trans.id == flow_transition_id).unwrap().to_owned();
        if FlowModelServ::check_post_action_ring(next_transition_detail.clone(), (false, vec![]), funs, ctx).await?.0 {
            return Err(funs.err().not_found("flow_inst", "transfer", "this post action exist endless loop", "500-flow-transition-endless-loop"));
        }

        let post_changes = next_transition_detail.action_by_post_changes();
        if post_changes.is_empty() {
            return Ok(());
        }

        for post_change in post_changes {
            let post_change = FlowTransitionActionChangeAgg::from(post_change);
            match post_change.kind {
                FlowTransitionActionChangeKind::Var => {
                    if let Some(mut change_info) = post_change.var_change_info {
                        if change_info.changed_kind.is_some() && change_info.changed_kind.clone().unwrap() == FlowTransitionActionByVarChangeInfoChangedKind::AutoGetOperateTime {
                            change_info.changed_val = Some(json!(Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)));
                            change_info.changed_kind = Some(FlowTransitionActionByVarChangeInfoChangedKind::ChangeContent);
                        }
                        let rel_tag = change_info.obj_tag.unwrap_or_default();
                        if !rel_tag.is_empty() {
                            let mut resp = FlowExternalServ::do_fetch_rel_obj(
                                &flow_model.tag,
                                &flow_inst_detail.id,
                                &flow_inst_detail.rel_business_obj_id,
                                vec![(rel_tag.clone(), change_info.obj_tag_rel_kind.clone())],
                                ctx,
                                funs,
                            )
                            .await?;
                            if !resp.rel_bus_objs.is_empty() {
                                for rel_bus_obj_id in resp.rel_bus_objs.pop().unwrap().rel_bus_obj_ids {
                                    let inst_id = FlowInstServ::get_inst_ids_by_rel_business_obj_id(vec![rel_bus_obj_id.clone()], funs, ctx).await?.pop().unwrap_or_default();
                                    FlowExternalServ::do_modify_field(
                                        &rel_tag,
                                        &next_transition_detail,
                                        &rel_bus_obj_id,
                                        &inst_id,
                                        FlowExternalCallbackOp::PostAction,
                                        next_flow_state.name.clone(),
                                        next_flow_state.sys_state.clone(),
                                        prev_flow_state.name.clone(),
                                        prev_flow_state.sys_state.clone(),
                                        vec![FlowExternalParams {
                                            rel_kind: None,
                                            rel_tag: None,
                                            var_id: None,
                                            var_name: Some(change_info.var_name.clone()),
                                            value: change_info.changed_val.clone(),
                                            changed_kind: change_info.changed_kind.clone(),
                                        }],
                                        ctx,
                                        funs,
                                    )
                                    .await?;
                                    if let Some(ws_client) = ws_flow_client().await {
                                        ws_client
                                            .publish_front_change(inst_id, default_flow_avatar().await.clone(), funs.conf::<FlowConfig>().invoke.spi_app_id.clone(), ctx)
                                            .await?;
                                    } else {
                                        FlowEventServ::do_front_change(&inst_id, ctx, funs).await?;
                                    }
                                }
                            }
                        } else {
                            FlowExternalServ::do_modify_field(
                                &flow_model.tag,
                                &next_transition_detail,
                                &flow_inst_detail.rel_business_obj_id,
                                &flow_inst_detail.id,
                                FlowExternalCallbackOp::PostAction,
                                next_flow_state.name.clone(),
                                next_flow_state.sys_state.clone(),
                                prev_flow_state.name.clone(),
                                prev_flow_state.sys_state.clone(),
                                vec![FlowExternalParams {
                                    rel_kind: None,
                                    rel_tag: None,
                                    var_id: None,
                                    var_name: Some(change_info.var_name.clone()),
                                    value: change_info.changed_val.clone(),
                                    changed_kind: change_info.changed_kind,
                                }],
                                ctx,
                                funs,
                            )
                            .await?;
                            if let Some(ws_client) = ws_flow_client().await {
                                ws_client
                                    .publish_front_change(
                                        flow_inst_detail.id.clone(),
                                        default_flow_avatar().await.clone(),
                                        funs.conf::<FlowConfig>().invoke.spi_app_id.clone(),
                                        ctx,
                                    )
                                    .await?;
                            } else {
                                FlowEventServ::do_front_change(&flow_inst_detail.id, ctx, funs).await?;
                            }
                        }
                    }
                }
                FlowTransitionActionChangeKind::State => {
                    if let Some(change_info) = post_change.state_change_info {
                        let mut resp = FlowExternalServ::do_fetch_rel_obj(
                            &flow_model.tag,
                            &flow_inst_detail.id,
                            &flow_inst_detail.rel_business_obj_id,
                            vec![(change_info.obj_tag.clone(), change_info.obj_tag_rel_kind.clone())],
                            ctx,
                            funs,
                        )
                        .await?;
                        if !resp.rel_bus_objs.is_empty() {
                            let inst_ids = Self::find_inst_ids_by_rel_obj_ids(&flow_model, resp.rel_bus_objs.pop().unwrap().rel_bus_obj_ids, &change_info, funs, ctx).await?;
                            Self::do_modify_state_by_post_action(inst_ids, &change_info, funs, ctx).await?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn find_inst_ids_by_rel_obj_ids(
        flow_model: &FlowModelDetailResp,
        rel_bus_obj_ids: Vec<String>,
        change_info: &FlowTransitionActionByStateChangeInfo,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<String>> {
        let mut result_rel_obj_ids = Self::filter_rel_obj_ids_by_state(&rel_bus_obj_ids, &change_info.obj_current_state_id, funs, ctx).await?;

        if let Some(change_condition) = change_info.change_condition.clone() {
            // Check mismatch rel_obj_ids and filter them
            let mut mismatch_rel_obj_ids = vec![];
            for rel_obj_id in result_rel_obj_ids.iter() {
                if change_condition.current {
                    // collect rel tags
                    let mut rel_tags = vec![];
                    for condition_item in change_condition.conditions.iter() {
                        if condition_item.obj_tag.is_some() && !condition_item.state_id.is_empty() {
                            rel_tags.push((condition_item.obj_tag.clone().unwrap(), condition_item.obj_tag_rel_kind.clone()));
                        }
                    }
                    let inst_id = FlowInstServ::get_inst_ids_by_rel_business_obj_id(vec![rel_obj_id.clone()], funs, ctx).await?.pop().unwrap_or_default();
                    let tag = if change_info.obj_tag_rel_kind == Some(TagRelKind::ParentOrSub) {
                        &flow_model.tag
                    } else {
                        &change_info.obj_tag
                    };
                    let resp = FlowExternalServ::do_fetch_rel_obj(tag, &inst_id, rel_obj_id, rel_tags, ctx, funs).await?;
                    if !resp.rel_bus_objs.is_empty() {
                        for rel_bus_obj in resp.rel_bus_objs {
                            let condition = change_condition
                                .conditions
                                .iter()
                                .find(|condition| condition.obj_tag.is_some() && condition.obj_tag.clone().unwrap() == rel_bus_obj.rel_tag.clone())
                                .unwrap();
                            let rel_obj_ids = Self::filter_rel_obj_ids_by_state(&rel_bus_obj.rel_bus_obj_ids, &Some(condition.state_id.clone()), funs, ctx).await?;
                            match condition.op {
                                StateChangeConditionOp::And => {
                                    if rel_bus_obj.rel_bus_obj_ids.is_empty() || rel_bus_obj.rel_bus_obj_ids.len() != rel_obj_ids.len() {
                                        mismatch_rel_obj_ids.push(rel_obj_id.clone());
                                        continue;
                                    }
                                }
                                StateChangeConditionOp::Or => {
                                    if rel_obj_ids.is_empty() {
                                        mismatch_rel_obj_ids.push(rel_obj_id.clone());
                                        continue;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            result_rel_obj_ids = result_rel_obj_ids.into_iter().filter(|result_rel_obj_id| !mismatch_rel_obj_ids.contains(result_rel_obj_id)).collect_vec();
        }

        let result = FlowInstServ::get_inst_ids_by_rel_business_obj_id(result_rel_obj_ids, funs, ctx).await?;
        Ok(result)
    }

    async fn filter_rel_obj_ids_by_state(
        rel_bus_obj_ids: &[String],
        obj_current_state_id: &Option<Vec<String>>,
        funs: &TardisFunsInst,
        _ctx: &TardisContext,
    ) -> TardisResult<Vec<String>> {
        #[derive(sea_orm::FromQueryResult)]
        pub struct FlowInstRelObjIdsResult {
            pub id: String,
            pub current_state_id: String,
            pub rel_business_obj_id: String,
        }
        let rel_insts = funs
            .db()
            .find_dtos::<FlowInstRelObjIdsResult>(
                Query::select()
                    .columns([flow_inst::Column::Id, flow_inst::Column::CurrentStateId, flow_inst::Column::RelBusinessObjId])
                    .from(flow_inst::Entity)
                    .and_where(Expr::col(flow_inst::Column::RelBusinessObjId).is_in(rel_bus_obj_ids)),
            )
            .await?;
        if rel_bus_obj_ids.len() != rel_insts.len() {
            return Err(funs.err().not_found("flow_inst", "do_post_change", "some flow instances not found", "404-flow-inst-not-found"));
        }
        Ok(rel_insts
            .iter()
            .filter(|inst_result| {
                if let Some(obj_current_state_id) = obj_current_state_id.clone() {
                    if !obj_current_state_id.is_empty() && !obj_current_state_id.contains(&inst_result.current_state_id) {
                        return false;
                    }
                }
                true
            })
            .map(|inst_result| inst_result.rel_business_obj_id.clone())
            .collect_vec())
    }

    async fn do_modify_state_by_post_action(
        rel_inst_ids: Vec<String>,
        change_info: &FlowTransitionActionByStateChangeInfo,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let insts = FlowInstServ::find_detail(rel_inst_ids, funs, ctx).await?;
        for rel_inst in insts {
            // find transition
            let flow_model = FlowModelServ::get_item(
                &rel_inst.rel_flow_model_id,
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
            let transition_resp = FlowInstServ::do_find_next_transitions(&rel_inst, &flow_model, None, &None, false, funs, ctx)
                .await?
                .next_flow_transitions
                .into_iter()
                .filter(|transition_detail| *transition_detail.next_flow_state_id == change_info.changed_state_id)
                .collect_vec()
                .pop();
            if let Some(transition) = transition_resp {
                FlowInstServ::transfer(
                    &rel_inst.id,
                    &FlowInstTransferReq {
                        flow_transition_id: transition.next_flow_transition_id,
                        message: None,
                        vars: None,
                    },
                    true,
                    FlowExternalCallbackOp::PostAction,
                    funs,
                    ctx,
                )
                .await?;
            }
        }
        Ok(())
    }

    pub async fn do_modify_assigned(flow_inst_id: &str, assigned_id: &str, ctx: &TardisContext, funs: &TardisFunsInst) -> TardisResult<()> {
        if funs
            .db()
            .count(
                Query::select()
                    .column((flow_inst::Entity, flow_inst::Column::Id))
                    .from(flow_inst::Entity)
                    .and_where(Expr::col((flow_inst::Entity, flow_inst::Column::Id)).eq(flow_inst_id.to_string()))
                    .and_where(Expr::col((flow_inst::Entity, flow_inst::Column::OwnPaths)).like(format!("{}%", ctx.own_paths))),
            )
            .await?
            == 0
        {
            return Err(funs.err().not_found(
                "flow_inst",
                "modify_assigned",
                &format!("flow instance {} not found", flow_inst_id),
                "404-flow-inst-not-found",
            ));
        }
        let flow_inst = flow_inst::ActiveModel {
            id: Set(flow_inst_id.to_string()),
            current_assigned: Set(Some(assigned_id.to_string())),
            ..Default::default()
        };
        funs.db().update_one(flow_inst, ctx).await?;

        Ok(())
    }
}
