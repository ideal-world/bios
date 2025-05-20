use std::collections::HashMap;

use bios_basic::rbum::{dto::rbum_filer_dto::RbumBasicFilterReq, helper::rbum_scope_helper, rbum_enumeration::RbumScopeLevelKind, serv::rbum_item_serv::RbumItemCrudOperation};
use bios_sdk_invoke::clients::spi_log_client::{LogItemFindReq, LogItemFindResp};
use serde_json::Value;
use tardis::{
    basic::{dto::TardisContext, field::TrimString, result::TardisResult}, TardisFuns, TardisFunsInst
};

use crate::{dto::{
    flow_inst_dto::{FlowInstDetailResp, FlowInstOperateReq, FlowInstStartReq, FlowInstStateKind},
    flow_model_dto::{FlowModelDetailResp, FlowModelRelTransitionKind},
    flow_state_dto::{FlowStateDetailResp, FlowStateFilterReq, FlowStateKind, FlowStateOperatorKind},
}, flow_constants};

use super::{
    clients::{
        kv_client::FlowKvClient,
        log_client::{FlowLogClient, LogParamContent, LogParamExt, LogParamExtSceneKind, LogParamOp, LogParamTag},
    },
    flow_state_serv::FlowStateServ,
};

pub struct FlowLogServ;

impl FlowLogServ {
    pub async fn add_start_log_async_task(
        start_req: &FlowInstStartReq,
        flow_inst_detail: &FlowInstDetailResp,
        create_vars: &HashMap<String, Value>,
        _funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let ctx_cp = ctx.clone();
        let start_req_cp = start_req.clone();
        let curr_inst_cp = flow_inst_detail.clone();
        let curr_inst_id = flow_inst_detail.id.clone();
        let create_vars_cp = create_vars.clone();
        let funs_cp = flow_constants::get_tardis_inst();
        ctx.add_sync_task(Box::new(
            || {
                Box::pin(async move {
                    let task_handle = tardis::tokio::spawn(async move {
                        let _ = Self::add_start_log(
                            &start_req_cp,
                            &curr_inst_cp,
                            &create_vars_cp,
                            &funs_cp,
                            &ctx_cp,
                        ).await;
                    });
                    match task_handle.await {
                        Ok(_) => {}
                        Err(e) => tardis::log::error!("Flow Instance {} add_start_log error:{:?}", curr_inst_id, e),
                    }
                    tardis::tokio::time::sleep(tardis::tokio::time::Duration::from_millis(1000)).await;
                    Ok(())
                })
            }
        )).await?;
        Ok(())
    }

    // 添加审批流发起日志
    async fn add_start_log(
        start_req: &FlowInstStartReq,
        flow_inst_detail: &FlowInstDetailResp,
        create_vars: &HashMap<String, Value>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        if flow_inst_detail.rel_inst_id.as_ref().is_some_and(|id| !id.is_empty()) {
            return Ok(());
        }
        let artifacts = flow_inst_detail.artifacts.clone().unwrap_or_default();
        let rel_transition = FlowModelRelTransitionKind::from(flow_inst_detail.rel_transition.clone().unwrap_or_default());
        let operand = rel_transition.log_text();
        let mut log_ext = LogParamExt {
            scene_kind: Some(vec![String::from(LogParamExtSceneKind::ApprovalFlow)]),
            new_log: Some(true),
            project_id: rbum_scope_helper::get_path_item(RbumScopeLevelKind::L2.to_int(), &ctx.own_paths),
            ..Default::default()
        };
        let mut log_content = LogParamContent {
            subject: Some(FlowLogClient::get_flow_kind_text(&start_req.tag)),
            name: Some(create_vars.get("name").map_or("".to_string(), |val| val.as_str().unwrap_or("").to_string())),
            sub_id: Some(start_req.rel_business_obj_id.clone()),
            sub_kind: Some(FlowLogClient::get_junp_kind(&start_req.tag)),
            operand: Some(operand),
            operand_name: Some(flow_inst_detail.code.clone()),
            operand_id: Some(flow_inst_detail.id.clone()),
            operand_kind: Some(FlowLogClient::get_junp_kind("FLOW")),
            ..Default::default()
        };
        if !artifacts.his_operators.as_ref().unwrap_or(&vec![]).contains(&ctx.owner) && !artifacts.curr_operators.as_ref().unwrap_or(&vec![]).contains(&ctx.owner) {
            log_content.operand_id = None;
            log_content.operand_kind = None;
        }
        if start_req.vars.is_none() || start_req.vars.clone().unwrap_or_default().is_empty() {
            log_ext.include_detail = Some(false);
            log_content.old_content = None;
            log_content.new_content = None;
        } else {
            let new_content = start_req.vars.clone().unwrap_or_default().get("content").map(|content| content.as_str().unwrap_or("").to_string()).unwrap_or_default();
            let old_content = create_vars.get("content").map_or("".to_string(), |val| val.as_str().unwrap_or("").to_string());
            if new_content.is_empty() {
                log_content.old_content = None;
                log_content.new_content = None;
            } else {
                log_content.old_content = Some(old_content);
                log_content.new_content = Some(new_content);
            }

            log_content.detail = start_req.log_text.clone();
            log_ext.include_detail = Some(true);
        }
        FlowLogClient::addv2_item(
            LogParamTag::ApprovalFlow,
            Some(flow_inst_detail.id.clone()),
            log_content,
            Some(TardisFuns::json.obj_to_json(&log_ext).expect("ext not a valid json value")),
            Some("dynamic_log_approval_flow".to_string()),
            Some(LogParamOp::Start.into()),
            rbum_scope_helper::get_path_item(RbumScopeLevelKind::L1.to_int(), &ctx.own_paths),
            funs,
            ctx,
            false,
        )
        .await?;
        Ok(())
    }

    pub async fn add_start_dynamic_log_async_task(
        start_req: &FlowInstStartReq,
        flow_inst_detail: &FlowInstDetailResp,
        create_vars: &HashMap<String, Value>,
        _funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let ctx_cp = ctx.clone();
        let start_req_cp = start_req.clone();
        let curr_inst_cp = flow_inst_detail.clone();
        let curr_inst_id = flow_inst_detail.id.clone();
        let create_vars_cp = create_vars.clone();
        let funs_cp = flow_constants::get_tardis_inst();
        ctx.add_sync_task(Box::new(
            || {
                Box::pin(async move {
                    let task_handle = tardis::tokio::spawn(async move {
                        let _ = Self::add_start_dynamic_log(
                            &start_req_cp,
                            &curr_inst_cp,
                            &create_vars_cp,
                            &funs_cp,
                            &ctx_cp,
                        ).await;
                    });
                    match task_handle.await {
                        Ok(_) => {}
                        Err(e) => tardis::log::error!("Flow Instance {} add_start_dynamic_log error:{:?}", curr_inst_id, e),
                    }
                    tardis::tokio::time::sleep(tardis::tokio::time::Duration::from_millis(1000)).await;
                    Ok(())
                })
            }
        )).await?;
        Ok(())
    }

    // 添加审批流发起动态日志
    async fn add_start_dynamic_log(
        start_req: &FlowInstStartReq,
        flow_inst_detail: &FlowInstDetailResp,
        create_vars: &HashMap<String, Value>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        if flow_inst_detail.rel_inst_id.as_ref().is_some_and(|id| !id.is_empty()) {
            return Ok(());
        }
        let artifacts = flow_inst_detail.artifacts.clone().unwrap_or_default();
        let rel_transition = FlowModelRelTransitionKind::from(flow_inst_detail.rel_transition.clone().unwrap_or_default());
        let operand = rel_transition.log_text();
        let mut log_ext = LogParamExt {
            scene_kind: Some(vec![String::from(LogParamExtSceneKind::Dynamic)]),
            new_log: Some(true),
            project_id: rbum_scope_helper::get_path_item(RbumScopeLevelKind::L2.to_int(), &ctx.own_paths),
            ..Default::default()
        };
        let mut log_content = LogParamContent {
            subject: Some(FlowLogClient::get_flow_kind_text(&start_req.tag)),
            name: Some(create_vars.get("name").map_or("".to_string(), |val| val.as_str().unwrap_or("").to_string())),
            sub_id: Some(start_req.rel_business_obj_id.clone()),
            sub_kind: Some(FlowLogClient::get_junp_kind(&start_req.tag)),
            operand: Some(operand),
            operand_name: Some(flow_inst_detail.code.clone()),
            operand_id: Some(flow_inst_detail.id.clone()),
            operand_kind: Some(FlowLogClient::get_junp_kind("FLOW")),
            ..Default::default()
        };
        if !artifacts.his_operators.as_ref().unwrap_or(&vec![]).contains(&ctx.owner) && !artifacts.curr_operators.as_ref().unwrap_or(&vec![]).contains(&ctx.owner) {
            log_content.sub_id = None;
            log_content.sub_kind = None;
            log_content.operand_id = None;
            log_content.operand_kind = None;
        }
        if start_req.vars.is_none() || start_req.vars.clone().unwrap_or_default().is_empty() {
            log_ext.include_detail = Some(false);
            log_content.old_content = None;
            log_content.new_content = None;
        } else {
            let new_content = start_req.vars.clone().unwrap_or_default().get("content").map(|content| content.as_str().unwrap_or("").to_string()).unwrap_or_default();
            let old_content = create_vars.get("content").map_or("".to_string(), |val| val.as_str().unwrap_or("").to_string());
            if new_content.is_empty() {
                log_content.old_content = None;
                log_content.new_content = None;
            } else {
                log_content.old_content = Some(old_content);
                log_content.new_content = Some(new_content);
            }
            log_content.detail = start_req.log_text.clone();
            log_ext.include_detail = Some(true);
        }
        FlowLogClient::add_item(
            LogParamTag::DynamicLog,
            Some(flow_inst_detail.id.clone()),
            log_content,
            Some(TardisFuns::json.obj_to_json(&log_ext).expect("ext not a valid json value")),
            Some("dynamic_log_approval_flow".to_string()),
            Some(LogParamOp::Start.into()),
            rbum_scope_helper::get_path_item(RbumScopeLevelKind::L1.to_int(), &ctx.own_paths),
            funs,
            ctx,
            false,
        )
        .await?;
        Ok(())
    }

    pub async fn add_start_business_log_async_task(
        start_req: &FlowInstStartReq,
        flow_inst_detail: &FlowInstDetailResp,
        create_vars: &HashMap<String, Value>,
        _funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let ctx_cp = ctx.clone();
        let start_req_cp = start_req.clone();
        let curr_inst_cp = flow_inst_detail.clone();
        let curr_inst_id = flow_inst_detail.id.clone();
        let create_vars_cp = create_vars.clone();
        let funs_cp = flow_constants::get_tardis_inst();
        ctx.add_async_task(Box::new(
            || {
                Box::pin(async move {
                    let task_handle = tardis::tokio::spawn(async move {
                        let _ = Self::add_start_business_log(
                            &start_req_cp,
                            &curr_inst_cp,
                            &create_vars_cp,
                            &funs_cp,
                            &ctx_cp,
                        ).await;
                    });
                    match task_handle.await {
                        Ok(_) => {}
                        Err(e) => tardis::log::error!("Flow Instance {} add_start_business_log error:{:?}", curr_inst_id, e),
                    }
                    tardis::tokio::time::sleep(tardis::tokio::time::Duration::from_millis(1000)).await;
                    Ok(())
                })
            }
        )).await?;
        Ok(())
    }

    // 添加审批流发起业务日志
    async fn add_start_business_log(
        _start_req: &FlowInstStartReq,
        flow_inst_detail: &FlowInstDetailResp,
        _create_vars: &HashMap<String, Value>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        if flow_inst_detail.rel_inst_id.as_ref().is_some_and(|id| !id.is_empty()) {
            return Ok(());
        }
        let artifacts = flow_inst_detail.artifacts.clone().unwrap_or_default();
        let rel_transition = FlowModelRelTransitionKind::from(flow_inst_detail.rel_transition.clone().unwrap_or_default());
        let subject = rel_transition.log_text();
        let log_ext = LogParamExt {
            scene_kind: Some(vec![String::from(LogParamExtSceneKind::Detail)]),
            new_log: Some(true),
            project_id: rbum_scope_helper::get_path_item(RbumScopeLevelKind::L2.to_int(), &ctx.own_paths),
            ..Default::default()
        };
        let mut log_content = LogParamContent {
            subject: Some(subject),
            name: Some(format!("编号{}", flow_inst_detail.code)),
            sub_id: Some(flow_inst_detail.id.clone()),
            sub_kind: Some(FlowLogClient::get_junp_kind(&flow_inst_detail.tag)),
            ..Default::default()
        };
        if !artifacts.his_operators.as_ref().unwrap_or(&vec![]).contains(&ctx.owner) && !artifacts.curr_operators.as_ref().unwrap_or(&vec![]).contains(&ctx.owner) {
            log_content.sub_id = None;
            log_content.sub_kind = None;
        }
        // if start_req.create_vars.is_none() {
        //     log_ext.include_detail = Some(false);
        //     log_content.old_content = "".to_string();
        //     log_content.new_content = "".to_string();
        // } else {
        //     log_content.old_content = create_vars.get("content").map_or("".to_string(), |val| val.as_str().unwrap_or("").to_string());
        //     log_content.new_content = start_req.create_vars.clone().unwrap_or_default().get("content").map(|content| content.as_str().unwrap_or("").to_string()).unwrap_or_default();
        //     log_content.detail = start_req.log_text.clone();
        //     log_ext.include_detail = Some(true);
        // }
        FlowLogClient::add_item(
            LogParamTag::DynamicLog,
            Some(flow_inst_detail.rel_business_obj_id.clone()),
            log_content,
            Some(TardisFuns::json.obj_to_json(&log_ext).expect("ext not a valid json value")),
            Some("dynamic_log_approval_flow".to_string()),
            Some(LogParamOp::Start.into()),
            rbum_scope_helper::get_path_item(RbumScopeLevelKind::L1.to_int(), &ctx.own_paths),
            funs,
            ctx,
            false,
        )
        .await?;
        Ok(())
    }

    pub async fn add_operate_log_async_task(
        operate_req: &FlowInstOperateReq,
        flow_inst_detail: &FlowInstDetailResp,
        op_kind: LogParamOp,
        _funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let ctx_cp = ctx.clone();
        let operate_req_cp = operate_req.clone();
        let curr_inst_cp = flow_inst_detail.clone();
        let curr_inst_id = flow_inst_detail.id.clone();
        let op_kind_cp = op_kind.clone();
        let funs_cp = flow_constants::get_tardis_inst();
        ctx.add_sync_task(Box::new(
            || {
                Box::pin(async move {
                    let task_handle = tardis::tokio::spawn(async move {
                        let _ = Self::add_operate_log(
                            &operate_req_cp,
                            &curr_inst_cp,
                            op_kind_cp,
                            &funs_cp,
                            &ctx_cp,
                        ).await;
                    });
                    match task_handle.await {
                        Ok(_) => {}
                        Err(e) => tardis::log::error!("Flow Instance {} add_operate_log error:{:?}", curr_inst_id, e),
                    }
                    tardis::tokio::time::sleep(tardis::tokio::time::Duration::from_millis(1000)).await;
                    Ok(())
                })
            }
        )).await?;
        Ok(())
    }

    // 添加审批流操作日志
    async fn add_operate_log(
        operate_req: &FlowInstOperateReq,
        flow_inst_detail: &FlowInstDetailResp,
        op_kind: LogParamOp,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        if flow_inst_detail.rel_inst_id.as_ref().is_some_and(|id| !id.is_empty()) {
            return Ok(());
        }
        let current_state = FlowStateServ::get_item(
            &flow_inst_detail.current_state_id,
            &FlowStateFilterReq {
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
        let subject_text = match current_state.state_kind {
            FlowStateKind::Approval => "审批节点".to_string(),
            FlowStateKind::Form => "录入节点".to_string(),
            _ => "".to_string(),
        };
        let mut log_ext = LogParamExt {
            scene_kind: Some(vec![String::from(LogParamExtSceneKind::ApprovalFlow)]),
            new_log: Some(true),
            project_id: rbum_scope_helper::get_path_item(RbumScopeLevelKind::L2.to_int(), &ctx.own_paths),
            ..Default::default()
        };
        let mut log_content = LogParamContent {
            subject: Some(subject_text),
            name: Some(current_state.name),
            sub_id: Some(flow_inst_detail.rel_business_obj_id.clone()),
            sub_kind: Some(FlowLogClient::get_junp_kind(&flow_inst_detail.tag)),
            flow_message: operate_req.output_message.clone(),
            flow_result: Some(operate_req.operate.to_string().to_uppercase()),
            old_content: None,
            new_content: None,
            ..Default::default()
        };
        if operate_req.operate == FlowStateOperatorKind::Referral {
            log_content.flow_referral = Some(FlowKvClient::get_account_name(&operate_req.operator.clone().unwrap_or_default(), funs, ctx).await?);
        }
        if operate_req.vars.is_none() || operate_req.vars.clone().unwrap_or_default().is_empty() {
            log_ext.include_detail = Some(false);
        } else {
            let new_content = operate_req.vars.clone().unwrap_or_default().get("content").map(|content| content.as_str().unwrap_or("").to_string()).unwrap_or_default();
            let old_content = flow_inst_detail.create_vars.clone().unwrap_or_default().get("content").map_or("".to_string(), |val| val.as_str().unwrap_or("").to_string());
            if new_content.is_empty() {
                log_content.old_content = None;
                log_content.new_content = None;
            } else {
                log_content.old_content = Some(old_content);
                log_content.new_content = Some(new_content);
            }
            log_content.detail = operate_req.log_text.clone();
            log_ext.include_detail = Some(true);
        }
        if operate_req.output_message.is_some() {
            log_ext.include_detail = Some(true);
        }
        FlowLogClient::addv2_item(
            LogParamTag::ApprovalFlow,
            Some(flow_inst_detail.id.clone()),
            log_content,
            Some(TardisFuns::json.obj_to_json(&log_ext).expect("ext not a valid json value")),
            Some("dynamic_log_approval_flow".to_string()),
            Some(op_kind.into()),
            rbum_scope_helper::get_path_item(RbumScopeLevelKind::L1.to_int(), &ctx.own_paths),
            funs,
            ctx,
            false,
        )
        .await?;
        Ok(())
    }

    pub async fn add_operate_dynamic_log_async_task(
        operate_req: &FlowInstOperateReq,
        flow_inst_detail: &FlowInstDetailResp,
        op_kind: LogParamOp,
        _funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let ctx_cp = ctx.clone();
        let operate_req_cp = operate_req.clone();
        let curr_inst_cp = flow_inst_detail.clone();
        let curr_inst_id = flow_inst_detail.id.clone();
        let op_kind_cp = op_kind.clone();
        let funs_cp = flow_constants::get_tardis_inst();
        ctx.add_sync_task(Box::new(
            || {
                Box::pin(async move {
                    let task_handle = tardis::tokio::spawn(async move {
                        let _ = Self::add_operate_dynamic_log(
                            &operate_req_cp,
                            &curr_inst_cp,
                            op_kind_cp,
                            &funs_cp,
                            &ctx_cp,
                        ).await;
                    });
                    match task_handle.await {
                        Ok(_) => {}
                        Err(e) => tardis::log::error!("Flow Instance {} add_operate_dynamic_log error:{:?}", curr_inst_id, e),
                    }
                    tardis::tokio::time::sleep(tardis::tokio::time::Duration::from_millis(1000)).await;
                    Ok(())
                })
            }
        )).await?;
        Ok(())
    }

    // 添加审批流操作动态日志
    async fn add_operate_dynamic_log(
        operate_req: &FlowInstOperateReq,
        flow_inst_detail: &FlowInstDetailResp,
        op_kind: LogParamOp,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        if flow_inst_detail.rel_inst_id.as_ref().is_some_and(|id| !id.is_empty()) {
            return Ok(());
        }
        let artifacts = flow_inst_detail.artifacts.clone().unwrap_or_default();
        let rel_transition = FlowModelRelTransitionKind::from(flow_inst_detail.rel_transition.clone().unwrap_or_default());
        let subject_text = rel_transition.log_text();
        let current_state = FlowStateServ::get_item(
            &flow_inst_detail.current_state_id,
            &FlowStateFilterReq {
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
        let operand = match current_state.state_kind {
            FlowStateKind::Approval => "审批节点".to_string(),
            FlowStateKind::Form => "录入节点".to_string(),
            _ => "".to_string(),
        };
        let mut log_ext = LogParamExt {
            scene_kind: Some(vec![String::from(LogParamExtSceneKind::Dynamic)]),
            new_log: Some(true),
            project_id: rbum_scope_helper::get_path_item(RbumScopeLevelKind::L2.to_int(), &ctx.own_paths),
            ..Default::default()
        };
        let mut log_content = LogParamContent {
            subject: Some(subject_text),
            name: Some(flow_inst_detail.code.clone()),
            sub_id: Some(flow_inst_detail.id.clone()),
            sub_kind: Some(FlowLogClient::get_junp_kind(&flow_inst_detail.tag)),
            operand: Some(operand),
            operand_name: Some(current_state.name),
            operand_id: Some(flow_inst_detail.rel_business_obj_id.clone()),
            operand_kind: Some(FlowLogClient::get_junp_kind(&flow_inst_detail.tag)),
            flow_message: operate_req.output_message.clone(),
            flow_result: Some(operate_req.operate.to_string().to_uppercase()),
            old_content: None,
            new_content: None,
            ..Default::default()
        };
        if !artifacts.his_operators.as_ref().unwrap_or(&vec![]).contains(&ctx.owner) && !artifacts.curr_operators.as_ref().unwrap_or(&vec![]).contains(&ctx.owner) {
            log_content.operand_id = None;
        }
        if operate_req.operate == FlowStateOperatorKind::Referral {
            log_content.flow_referral = Some(FlowKvClient::get_account_name(&operate_req.operator.clone().unwrap_or_default(), funs, ctx).await?);
        }
        if operate_req.vars.is_none() || operate_req.vars.clone().unwrap_or_default().is_empty() {
            log_ext.include_detail = Some(false);
        } else {
            let new_content = operate_req.vars.clone().unwrap_or_default().get("content").map(|content| content.as_str().unwrap_or("").to_string()).unwrap_or_default();
            let old_content = flow_inst_detail.create_vars.clone().unwrap_or_default().get("content").map_or("".to_string(), |val| val.as_str().unwrap_or("").to_string());
            if new_content.is_empty() {
                log_content.old_content = None;
                log_content.new_content = None;
            } else {
                log_content.old_content = Some(old_content);
                log_content.new_content = Some(new_content);
            }
            log_content.detail = operate_req.log_text.clone();
            log_ext.include_detail = Some(true);
        }
        if operate_req.output_message.is_some() {
            log_ext.include_detail = Some(true);
        }
        FlowLogClient::add_item(
            LogParamTag::DynamicLog,
            Some(flow_inst_detail.id.clone()),
            log_content,
            Some(TardisFuns::json.obj_to_json(&log_ext).expect("ext not a valid json value")),
            Some("dynamic_log_approval_flow".to_string()),
            Some(op_kind.into()),
            rbum_scope_helper::get_path_item(RbumScopeLevelKind::L1.to_int(), &ctx.own_paths),
            funs,
            ctx,
            false,
        )
        .await?;
        Ok(())
    }

    pub async fn add_finish_log_async_task(
        flow_inst_detail: &FlowInstDetailResp,
        msg: Option<String>,
        _funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let ctx_cp = ctx.clone();
        let msg_cp = msg.clone();
        let curr_inst_cp = flow_inst_detail.clone();
        let curr_inst_id = flow_inst_detail.id.clone();
        let funs_cp = flow_constants::get_tardis_inst();
        ctx.add_sync_task(Box::new(
            || {
                Box::pin(async move {
                    let task_handle = tardis::tokio::spawn(async move {
                        let _ = Self::add_finish_log(
                            &curr_inst_cp,
                            msg_cp,
                            &funs_cp,
                            &ctx_cp,
                        ).await;
                    });
                    match task_handle.await {
                        Ok(_) => {}
                        Err(e) => tardis::log::error!("Flow Instance {} add_finish_log error:{:?}", curr_inst_id, e),
                    }
                    tardis::tokio::time::sleep(tardis::tokio::time::Duration::from_millis(1000)).await;
                    Ok(())
                })
            }
        )).await?;
        Ok(())
    }

    async fn add_finish_log(flow_inst_detail: &FlowInstDetailResp, msg: Option<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if flow_inst_detail.rel_inst_id.as_ref().is_some_and(|id| !id.is_empty()) {
            return Ok(());
        }
        let artifacts = flow_inst_detail.artifacts.clone().unwrap_or_default();
        let rel_transition = FlowModelRelTransitionKind::from(flow_inst_detail.rel_transition.clone().unwrap_or_default());
        let subject_text = rel_transition.log_text();
        let log_ext = LogParamExt {
            scene_kind: Some(vec![String::from(LogParamExtSceneKind::ApprovalFlow)]),
            new_log: Some(true),
            project_id: rbum_scope_helper::get_path_item(RbumScopeLevelKind::L2.to_int(), &ctx.own_paths),
            ..Default::default()
        };
        let mut log_content = LogParamContent {
            subject: Some(subject_text),
            name: Some(flow_inst_detail.code.clone()),
            sub_id: Some(flow_inst_detail.id.clone()),
            sub_kind: Some(FlowLogClient::get_junp_kind(&flow_inst_detail.tag)),
            flow_message: msg,
            old_content: None,
            new_content: None,
            ..Default::default()
        };

        if !artifacts.his_operators.as_ref().unwrap_or(&vec![]).contains(&ctx.owner) && !artifacts.curr_operators.as_ref().unwrap_or(&vec![]).contains(&ctx.owner) {
            log_content.sub_id = None;
        }
        FlowLogClient::addv2_item(
            LogParamTag::ApprovalFlow,
            Some(flow_inst_detail.id.clone()),
            log_content,
            Some(TardisFuns::json.obj_to_json(&log_ext).expect("ext not a valid json value")),
            Some("dynamic_log_approval_flow".to_string()),
            Some(LogParamOp::Finish.into()),
            rbum_scope_helper::get_path_item(RbumScopeLevelKind::L1.to_int(), &ctx.own_paths),
            funs,
            ctx,
            false,
        )
        .await?;
        Ok(())
    }

    pub async fn add_finish_business_log_async_task(
        flow_inst_detail: &FlowInstDetailResp,
        msg: Option<String>,
        _funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let ctx_cp = ctx.clone();
        let msg_cp = msg.clone();
        let curr_inst_cp = flow_inst_detail.clone();
        let curr_inst_id = flow_inst_detail.id.clone();
        let funs_cp = flow_constants::get_tardis_inst();
        ctx.add_sync_task(Box::new(
            || {
                Box::pin(async move {
                    let task_handle = tardis::tokio::spawn(async move {
                        let _ = Self::add_finish_business_log(
                            &curr_inst_cp,
                            msg_cp,
                            &funs_cp,
                            &ctx_cp,
                        ).await;
                    });
                    match task_handle.await {
                        Ok(_) => {}
                        Err(e) => tardis::log::error!("Flow Instance {} add_finish_log error:{:?}", curr_inst_id, e),
                    }
                    tardis::tokio::time::sleep(tardis::tokio::time::Duration::from_millis(1000)).await;
                    Ok(())
                })
            }
        )).await?;
        Ok(())
    }

    async fn add_finish_business_log(flow_inst_detail: &FlowInstDetailResp, msg: Option<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let mock_ctx = TardisContext {
            owner: flow_inst_detail.create_ctx.owner.clone(), // 以发起人的身份记录
            ..ctx.clone()
        };
        let artifacts = flow_inst_detail.artifacts.clone().unwrap_or_default();
        let state_text = match artifacts.state.unwrap_or_default() {
            FlowInstStateKind::Pass => { "审批通过".to_string() },
            FlowInstStateKind::Overrule => { "审批拒绝".to_string() },
            _ => { "审批拒绝".to_string() },
        };
        let rel_transition = FlowModelRelTransitionKind::from(flow_inst_detail.rel_transition.clone().unwrap_or_default());
        let log_ext = LogParamExt {
            new_log: Some(true),
            project_id: rbum_scope_helper::get_path_item(RbumScopeLevelKind::L2.to_int(), &ctx.own_paths),
            include_detail: Some(false),
            scene_kind: Some(vec![String::from(LogParamExtSceneKind::Detail)]),
            delete: Some(false),
            sys_op: Some(state_text),
        };
        let log_content = LogParamContent {
            jump: Some(false),
            subject: Some(FlowLogClient::get_flow_kind_text(&flow_inst_detail.tag)),
            name: Some(flow_inst_detail.create_vars.clone().unwrap_or_default().get("name").map_or("".to_string(), |val| val.as_str().unwrap_or("").to_string())),
            sub_id: Some(flow_inst_detail.id.clone()),
            sub_kind: Some(FlowLogClient::get_junp_kind(&flow_inst_detail.tag)),
            flow_message: msg,
            ..Default::default()
        };

        let op = match rel_transition {
            FlowModelRelTransitionKind::Edit => LogParamOp::Update,
            FlowModelRelTransitionKind::Related => LogParamOp::FeedRel,
            FlowModelRelTransitionKind::Review => LogParamOp::Review,
            FlowModelRelTransitionKind::Delete => LogParamOp::Delete,
            FlowModelRelTransitionKind::Transfer(_) => LogParamOp::Update,
        };

        FlowLogClient::add_item(
            LogParamTag::DynamicLog,
            Some(flow_inst_detail.rel_business_obj_id.clone()),
            log_content,
            Some(TardisFuns::json.obj_to_json(&log_ext).expect("ext not a valid json value")),
            Some("dynamic_log_project_manager".to_string()),
            Some(op.into()),
            rbum_scope_helper::get_path_item(RbumScopeLevelKind::L1.to_int(), &ctx.own_paths),
            funs,
            &mock_ctx,
            false,
        )
        .await?;
        Ok(())
    }

    pub async fn add_model_delete_state_log_async_task(
        flow_model: &FlowModelDetailResp, 
        original_state: &FlowStateDetailResp, 
        target_state: &FlowStateDetailResp,
        _funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let ctx_cp = ctx.clone();
        let flow_model_cp = flow_model.clone();
        let original_state_cp = original_state.clone();
        let target_state_cp = target_state.clone();
        let flow_model_id = flow_model.id.clone();
        let funs_cp = flow_constants::get_tardis_inst();
        ctx.add_sync_task(Box::new(
            || {
                Box::pin(async move {
                    let task_handle = tardis::tokio::spawn(async move {
                        let _ = Self::add_model_delete_state_log(
                            &flow_model_cp,
                            &original_state_cp,
                            &target_state_cp,
                            &funs_cp,
                            &ctx_cp,
                        ).await;
                    });
                    match task_handle.await {
                        Ok(_) => {}
                        Err(e) => tardis::log::error!("Flow model {} add_model_delete_state_log error:{:?}", flow_model_id, e),
                    }
                    tardis::tokio::time::sleep(tardis::tokio::time::Duration::from_millis(1000)).await;
                    Ok(())
                })
            }
        )).await?;
        Ok(())
    }

    async fn add_model_delete_state_log(flow_model: &FlowModelDetailResp, original_state: &FlowStateDetailResp, target_state: &FlowStateDetailResp, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let log_content = LogParamContent {
            subject: Some("状态".to_string()),
            sub_id: Some(original_state.id.clone()),
            name: Some(original_state.name.clone()),
            sub_kind: Some(original_state.sys_state.to_string()),
            operand: Some("状态".to_string()),
            operand_id: Some(target_state.id.clone()),
            operand_name: Some(target_state.name.clone()),
            operand_kind: Some(target_state.sys_state.to_string()),
            ..Default::default()
        };
        FlowLogClient::addv2_item(
            LogParamTag::FlowModel,
            Some(flow_model.current_version_id.clone()),
            log_content,
            None,
            Some("dynamic_log_flow_model".to_string()),
            Some(LogParamOp::Delete.into()),
            rbum_scope_helper::get_path_item(RbumScopeLevelKind::L1.to_int(), &ctx.own_paths),
            funs,
            ctx,
            false,
        )
        .await?;
        Ok(())
    }

    pub async fn find_model_delete_state_log(flow_model: &FlowModelDetailResp, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<Vec<LogItemFindResp>>> {
        let find_req = LogItemFindReq {
            tag: LogParamTag::FlowModel.into(),
            kinds: Some(vec![TrimString("dynamic_log_flow_model")]),
            ops: Some(vec![LogParamOp::Delete.into()]),
            keys: Some(vec![TrimString(flow_model.current_version_id.clone())]),
            page_number: 1,
            page_size: 9999,
            ..Default::default()
        };

        Ok(FlowLogClient::findv2(find_req, funs, ctx).await?.map(|p| p.records))
    }
}
