use std::collections::HashMap;

use bios_basic::rbum::{dto::rbum_filer_dto::RbumBasicFilterReq, helper::rbum_scope_helper, rbum_enumeration::RbumScopeLevelKind, serv::rbum_item_serv::RbumItemCrudOperation};
use serde_json::Value;
use tardis::{basic::{dto::TardisContext, result::TardisResult}, TardisFuns, TardisFunsInst};

use crate::dto::{flow_inst_dto::{FlowInstDetailResp, FlowInstOperateReq, FlowInstStartReq}, flow_model_dto::{FlowModelDetailResp, FlowModelFilterReq}, flow_model_version_dto::FlowModelVersionFilterReq, flow_state_dto::{FlowStateFilterReq, FlowStateKind, FlowStateOperatorKind}};

use super::{clients::{kv_client::FlowKvClient, log_client::{FlowLogClient, LogParamContent, LogParamExt, LogParamExtSceneKind, LogParamOp, LogParamTag}}, flow_model_serv::FlowModelServ, flow_model_version_serv::FlowModelVersionServ, flow_state_serv::FlowStateServ};

pub struct FlowLogServ;

impl FlowLogServ{
    // 添加审批流发起日志
    pub async fn add_start_log(
        start_req: &FlowInstStartReq,
        flow_inst_detail: &FlowInstDetailResp,
        create_vars: &HashMap<String, Value>,
        flow_model: &FlowModelDetailResp,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let rel_transition = flow_model.rel_transition().unwrap_or_default();
        let operand = match rel_transition.id.as_str() {
            "__EDIT__" => "编辑审批".to_string(),
            "__DELETE__" => "删除审批".to_string(),
            _ => format!("{}({})", rel_transition.name, rel_transition.from_flow_state_name).to_string(),
        };
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
        if start_req.create_vars.is_none() {
            log_ext.include_detail = Some(false);
            log_content.old_content = "".to_string();
            log_content.new_content = "".to_string();
        } else {
            log_content.old_content = create_vars.get("content").map_or("".to_string(), |val| val.as_str().unwrap_or("").to_string());
            log_content.new_content = start_req.create_vars.clone().unwrap_or_default().get("content").map(|content| content.as_str().unwrap_or("").to_string()).unwrap_or_default();
            log_content.detail = start_req.log_text.clone();
            log_ext.include_detail = Some(true);
        }
        FlowLogClient::add_ctx_task(
            LogParamTag::ApprovalFlow,
            Some(flow_inst_detail.id.clone()),
            log_content,
            Some(TardisFuns::json.obj_to_json(&log_ext).expect("ext not a valid json value")),
            Some("dynamic_log_approval_flow".to_string()),
            Some(LogParamOp::Start.into()),
            rbum_scope_helper::get_path_item(RbumScopeLevelKind::L1.to_int(), &ctx.own_paths),
            true,
            ctx,
            false,
        )
        .await?;
        Ok(())
    }

    // 添加审批流发起动态日志
    pub async fn add_start_dynamic_log(
        start_req: &FlowInstStartReq,
        flow_inst_detail: &FlowInstDetailResp,
        create_vars: &HashMap<String, Value>,
        flow_model: &FlowModelDetailResp,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let rel_transition = flow_model.rel_transition().unwrap_or_default();
        let operand = match rel_transition.id.as_str() {
            "__EDIT__" => "编辑审批".to_string(),
            "__DELETE__" => "删除审批".to_string(),
            _ => format!("{}({})", rel_transition.name, rel_transition.from_flow_state_name).to_string(),
        };
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
        if start_req.create_vars.is_none() {
            log_ext.include_detail = Some(false);
            log_content.old_content = "".to_string();
            log_content.new_content = "".to_string();
        } else {
            log_content.old_content = create_vars.get("content").map_or("".to_string(), |val| val.as_str().unwrap_or("").to_string());
            log_content.new_content = start_req.create_vars.clone().unwrap_or_default().get("content").map(|content| content.as_str().unwrap_or("").to_string()).unwrap_or_default();
            log_content.detail = start_req.log_text.clone();
            log_ext.include_detail = Some(true);
        }
        FlowLogClient::add_ctx_task(
            LogParamTag::DynamicLog,
            Some(flow_inst_detail.id.clone()),
            log_content,
            Some(TardisFuns::json.obj_to_json(&log_ext).expect("ext not a valid json value")),
            Some("dynamic_log_approval_flow".to_string()),
            Some(LogParamOp::Start.into()),
            rbum_scope_helper::get_path_item(RbumScopeLevelKind::L1.to_int(), &ctx.own_paths),
            false,
            ctx,
            false,
        )
        .await?;
        Ok(())
    }

    // 添加审批流发起业务日志
    pub async fn add_start_business_log(
        start_req: &FlowInstStartReq,
        flow_inst_detail: &FlowInstDetailResp,
        create_vars: &HashMap<String, Value>,
        flow_model: &FlowModelDetailResp,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let rel_transition = flow_model.rel_transition().unwrap_or_default();
        let subject = match rel_transition.id.as_str() {
            "__EDIT__" => "编辑审批".to_string(),
            "__DELETE__" => "删除审批".to_string(),
            _ => format!("{}({})", rel_transition.name, rel_transition.from_flow_state_name).to_string(),
        };
        let mut log_ext = LogParamExt {
            scene_kind: Some(vec![String::from(LogParamExtSceneKind::Detail)]),
            new_log: Some(true),
            project_id: rbum_scope_helper::get_path_item(RbumScopeLevelKind::L2.to_int(), &ctx.own_paths),
            ..Default::default()
        };
        let mut log_content = LogParamContent {
            subject: Some(subject),
            name: Some(format!("编号{}", flow_inst_detail.code)),
            sub_id: Some(flow_inst_detail.id.clone()),
            sub_kind: Some(FlowLogClient::get_junp_kind("FLOW")),
            ..Default::default()
        };
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
        FlowLogClient::add_ctx_task(
            LogParamTag::DynamicLog,
            Some(flow_inst_detail.rel_business_obj_id.clone()),
            log_content,
            Some(TardisFuns::json.obj_to_json(&log_ext).expect("ext not a valid json value")),
            Some("dynamic_log_approval_flow".to_string()),
            Some(LogParamOp::Start.into()),
            rbum_scope_helper::get_path_item(RbumScopeLevelKind::L1.to_int(), &ctx.own_paths),
            false,
            ctx,
            false,
        )
        .await?;
        Ok(())
    }

    // 添加审批流操作日志
    pub async fn add_operate_log(
        operate_req: &FlowInstOperateReq,
        flow_inst_detail: &FlowInstDetailResp,
        op_kind: LogParamOp,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
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
            old_content: "".to_string(),
            new_content: "".to_string(),
            ..Default::default()
        };
        if operate_req.operate == FlowStateOperatorKind::Referral {
            log_content.flow_referral = Some(FlowKvClient::get_account_name(&operate_req.operator.clone().unwrap_or_default(), funs, ctx).await?);
        }
        if operate_req.vars.is_none() {
            log_ext.include_detail = Some(false);
        } else {
            log_content.old_content =
                flow_inst_detail.create_vars.clone().unwrap_or_default().get("content").map_or("".to_string(), |val| val.as_str().unwrap_or("").to_string());
            log_content.new_content = operate_req.vars.clone().unwrap_or_default().get("content").map(|content| content.as_str().unwrap_or("").to_string()).unwrap_or_default();
            log_content.detail = operate_req.log_text.clone();
            log_ext.include_detail = Some(true);
        }
        if operate_req.output_message.is_some() {
            log_ext.include_detail = Some(true);
        }
        FlowLogClient::add_ctx_task(
            LogParamTag::ApprovalFlow,
            Some(flow_inst_detail.id.clone()),
            log_content,
            Some(TardisFuns::json.obj_to_json(&log_ext).expect("ext not a valid json value")),
            Some("dynamic_log_approval_flow".to_string()),
            Some(op_kind.into()),
            rbum_scope_helper::get_path_item(RbumScopeLevelKind::L1.to_int(), &ctx.own_paths),
            true,
            ctx,
            false,
        )
        .await?;
        Ok(())
    }

    // 添加审批流操作动态日志
    pub async fn add_operate_dynamic_log(
        operate_req: &FlowInstOperateReq,
        flow_inst_detail: &FlowInstDetailResp,
        op_kind: LogParamOp,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let flow_model_version = FlowModelVersionServ::get_item(
            &flow_inst_detail.rel_flow_version_id,
            &FlowModelVersionFilterReq {
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
        let flow_model = FlowModelServ::get_item(
            &flow_model_version.rel_model_id,
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
        let rel_transition = flow_model.rel_transition().unwrap_or_default();
        let subject_text = match rel_transition.id.as_str() {
            "__EDIT__" => "编辑审批".to_string(),
            "__DELETE__" => "删除审批".to_string(),
            _ => format!("{}({})", rel_transition.name, rel_transition.from_flow_state_name).to_string(),
        };
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
            sub_kind: Some(FlowLogClient::get_junp_kind("FLOW")),
            operand: Some(operand),
            operand_name: Some(current_state.name),
            operand_id: Some(flow_inst_detail.rel_business_obj_id.clone()),
            operand_kind: Some(FlowLogClient::get_junp_kind(&flow_inst_detail.tag)),
            flow_message: operate_req.output_message.clone(),
            flow_result: Some(operate_req.operate.to_string().to_uppercase()),
            old_content: "".to_string(),
            new_content: "".to_string(),
            ..Default::default()
        };
        if operate_req.operate == FlowStateOperatorKind::Referral {
            log_content.flow_referral = Some(FlowKvClient::get_account_name(&operate_req.operator.clone().unwrap_or_default(), funs, ctx).await?);
        }
        if operate_req.vars.is_none() {
            log_ext.include_detail = Some(false);
        } else {
            log_content.old_content =
                flow_inst_detail.create_vars.clone().unwrap_or_default().get("content").map_or("".to_string(), |val| val.as_str().unwrap_or("").to_string());
            log_content.new_content = operate_req.vars.clone().unwrap_or_default().get("content").map(|content| content.as_str().unwrap_or("").to_string()).unwrap_or_default();
            log_content.detail = operate_req.log_text.clone();
            log_ext.include_detail = Some(true);
        }
        if operate_req.output_message.is_some() {
            log_ext.include_detail = Some(true);
        }
        FlowLogClient::add_ctx_task(
            LogParamTag::DynamicLog,
            Some(flow_inst_detail.id.clone()),
            log_content,
            Some(TardisFuns::json.obj_to_json(&log_ext).expect("ext not a valid json value")),
            Some("dynamic_log_approval_flow".to_string()),
            Some(op_kind.into()),
            rbum_scope_helper::get_path_item(RbumScopeLevelKind::L1.to_int(), &ctx.own_paths),
            false,
            ctx,
            false,
        )
        .await?;
        Ok(())
    }

    pub async fn add_finish_log(flow_inst_detail: &FlowInstDetailResp, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let flow_model_version = FlowModelVersionServ::get_item(
            &flow_inst_detail.rel_flow_version_id,
            &FlowModelVersionFilterReq {
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
        let flow_model = FlowModelServ::get_item(
            &flow_model_version.rel_model_id,
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
        let rel_transition = flow_model.rel_transition().unwrap_or_default();
        let subject_text = match rel_transition.id.as_str() {
            "__EDIT__" => "编辑审批".to_string(),
            "__DELETE__" => "删除审批".to_string(),
            _ => format!("{}({})", rel_transition.name, rel_transition.from_flow_state_name).to_string(),
        };
        let log_ext = LogParamExt {
            scene_kind: Some(vec![String::from(LogParamExtSceneKind::ApprovalFlow)]),
            new_log: Some(true),
            project_id: rbum_scope_helper::get_path_item(RbumScopeLevelKind::L2.to_int(), &ctx.own_paths),
            ..Default::default()
        };
        let log_content = LogParamContent {
            subject: Some(subject_text),
            name: Some(flow_inst_detail.code.clone()),
            sub_id: Some(flow_inst_detail.id.clone()),
            sub_kind: Some(FlowLogClient::get_junp_kind("FLOW")),
            old_content: "".to_string(),
            new_content: "".to_string(),
            ..Default::default()
        };
        FlowLogClient::add_ctx_task(
            LogParamTag::ApprovalFlow,
            Some(flow_inst_detail.id.clone()),
            log_content,
            Some(TardisFuns::json.obj_to_json(&log_ext).expect("ext not a valid json value")),
            Some("dynamic_log_approval_flow".to_string()),
            Some(LogParamOp::Finish.into()),
            rbum_scope_helper::get_path_item(RbumScopeLevelKind::L1.to_int(), &ctx.own_paths),
            true,
            ctx,
            false,
        )
        .await?;
        Ok(())
    }
}