use std::collections::HashMap;

use bios_sdk_invoke::clients::{
    iam_client::IamClient,
    spi_log_client::{LogItemAddReq, LogItemAddV2Req, SpiLogClient},
};
use serde::Serialize;

use serde_json::Value;
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    chrono::{DateTime, Utc},
    tokio, TardisFuns, TardisFunsInst,
};

use crate::{flow_config::FlowConfig, flow_constants};
pub struct FlowLogClient;

#[derive(Serialize, Default, Debug, Clone)]
pub struct LogParamContent {
    pub subject: Option<String>,
    pub name: Option<String>,
    pub sub_kind: Option<String>,
    pub sub_id: Option<String>,
    pub sub_op: Option<String>,
    pub old_content: Option<String>,
    pub new_content: Option<String>,
    pub detail: Option<String>,
    pub operand: Option<String>,
    pub operand_id: Option<String>,
    pub operand_kind: Option<String>,
    pub operand_name: Option<String>,
    pub flow_message: Option<String>,
    pub flow_result: Option<String>,
    pub flow_referral: Option<String>,
}

#[derive(Serialize, Default, Debug, Clone)]
pub struct LogParamExt {
    pub scene_kind: Option<Vec<String>>,
    pub sys_op: Option<String>,
    pub project_id: Option<String>,
    pub new_log: Option<bool>,
    pub include_detail: Option<bool>,
    pub delete: Option<bool>,
}

pub enum LogParamExtSceneKind {
    ApprovalFlow,
    Dynamic,
    Detail,
}
impl From<LogParamExtSceneKind> for String {
    fn from(val: LogParamExtSceneKind) -> Self {
        match val {
            LogParamExtSceneKind::Dynamic => "dynamic".to_string(),
            LogParamExtSceneKind::ApprovalFlow => "approval_flow".to_string(),
            LogParamExtSceneKind::Detail => "detail".to_string(),
        }
    }
}

pub enum LogParamTag {
    DynamicLog,
    ApprovalFlow,
}

impl From<LogParamTag> for String {
    fn from(val: LogParamTag) -> Self {
        match val {
            LogParamTag::DynamicLog => "dynamic_log".to_string(),
            LogParamTag::ApprovalFlow => "approval_flow".to_string(),
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub enum LogParamOp {
    // 发起
    Start,
    // 审批
    Approval,
    // 审批流转
    ApprovalTransfer,
    // 录入
    Form,
    // 录入流转
    FormTransfer,
    // 结束
    Finish,
}

impl From<LogParamOp> for String {
    fn from(val: LogParamOp) -> Self {
        match val {
            LogParamOp::Start => "FLOW_START".to_string(),
            LogParamOp::Approval => "FLOW_APPROVAL".to_string(),
            LogParamOp::ApprovalTransfer => "FLOW_APPROVAL_TRANSFER".to_string(),
            LogParamOp::Form => "FLOW_FORM".to_string(),
            LogParamOp::FormTransfer => "FLOW_FORM_TRANSFER".to_string(),
            LogParamOp::Finish => "FLOW_FINISH".to_string(),
        }
    }
}

impl FlowLogClient {
    pub async fn add_ctx_task(
        tag: LogParamTag,
        key: Option<String>,
        content: LogParamContent,
        ext: Option<Value>,
        kind: Option<String>,
        op_kind: Option<String>,
        rel_key: Option<String>,
        is_v2: bool,
        ctx: &TardisContext,
        push: bool,
    ) -> TardisResult<()> {
        let ctx_clone = ctx.clone();
        let push_clone = push; // 克隆 push 变量
        ctx.add_async_task(Box::new(move || {
            Box::pin(async move {
                let task_handle = tokio::spawn(async move {
                    let funs = flow_constants::get_tardis_inst();
                    if is_v2 {
                        let _ = Self::addv2_item(
                            tag,
                            key.clone(),
                            content,
                            ext,
                            kind,
                            op_kind,
                            rel_key,
                            &funs,
                            &ctx_clone,
                            push_clone, // 使用克隆的 push 变量
                        )
                        .await;
                    } else {
                        let _ = Self::add_item(
                            tag,
                            key.clone(),
                            content,
                            ext,
                            kind,
                            op_kind,
                            rel_key,
                            &funs,
                            &ctx_clone,
                            push_clone, // 使用克隆的 push 变量
                        )
                        .await;
                    }
                });
                task_handle.await.unwrap();
                Ok(())
            })
        }))
        .await
    }

    pub async fn add_item(
        tag: LogParamTag,
        key: Option<String>,
        content: LogParamContent,
        ext: Option<Value>,
        kind: Option<String>,
        op: Option<String>,
        rel_key: Option<String>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
        _push: bool,
    ) -> TardisResult<()> {
        let ts = tardis::chrono::Utc::now().to_rfc3339();
        // generate log item
        let tag: String = tag.into();
        let own_paths = if ctx.own_paths.len() < 2 { None } else { Some(ctx.own_paths.clone()) };
        let owner = if ctx.owner.len() < 2 { None } else { Some(ctx.owner.clone()) };

        let req = LogItemAddReq {
            id: None,
            tag: tag.to_string(),
            content: TardisFuns::json.obj_to_string(&content).expect("content not a valid json value"),
            kind,
            ext,
            key,
            op,
            rel_key,
            ts: Some(DateTime::parse_from_rfc3339(&ts).unwrap_or_default().with_timezone(&Utc)),
            owner,
            own_paths,
            data_source: None,
        };
        SpiLogClient::add(req, funs, ctx).await?;
        Ok(())
    }

    pub async fn addv2_item(
        tag: LogParamTag,
        key: Option<String>,
        content: LogParamContent,
        ext: Option<Value>,
        kind: Option<String>,
        op: Option<String>,
        rel_key: Option<String>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
        push: bool,
    ) -> TardisResult<()> {
        let ts = tardis::chrono::Utc::now().to_rfc3339();
        // generate log item
        let tag: String = tag.into();
        let own_paths = if ctx.own_paths.len() < 2 { None } else { Some(ctx.own_paths.clone()) };
        let owner = if ctx.owner.len() < 2 { None } else { Some(ctx.owner.clone()) };
        let owner_name = IamClient::new("", funs, ctx, funs.conf::<FlowConfig>().invoke.module_urls.get("iam").expect("missing iam base url"))
            .get_account(&ctx.owner, &ctx.own_paths)
            .await?
            .owner_name;

        let req = LogItemAddV2Req {
            tag: tag.to_string(),
            content: TardisFuns::json.obj_to_json(&content).expect("content not a valid json value"),
            kind,
            ext,
            key,
            op,
            rel_key,
            idempotent_id: None,
            ts: Some(DateTime::parse_from_rfc3339(&ts).unwrap_or_default().with_timezone(&Utc)),
            owner,
            own_paths,
            msg: None,
            owner_name,
            push,
            disable: None,
            data_source: None,
            ignore_push: None,
        };
        SpiLogClient::addv2(req, funs, ctx).await?;
        Ok(())
    }

    pub fn get_flow_kind_text(tag: &str) -> String {
        let flow_tag_map = HashMap::from([
            ("PROJ", "项目"),
            ("MS", "里程碑"),
            ("ITER", "迭代"),
            ("TICKET", "工单"),
            ("REQ", "需求"),
            ("TASK", "任务"),
            ("ISSUE", "缺陷"),
            ("CTS", "转测单"),
            ("TP", "测试计划"),
            ("TS", "测试阶段"),
            ("TC", "用例"),
            ("REVIEW", "评审"),
        ]);
        flow_tag_map.get(tag).map_or("".to_string(), |val| val.to_string())
    }

    pub fn get_junp_kind(tag: &str) -> String {
        let flow_tag_map = HashMap::from([
            ("MS", "idp_feed_ms"),
            ("ITER", "idp_feed_iter"),
            ("REQ", "idp_feed_req"),
            ("TASK", "idp_feed_task"),
            ("ISSUE", "idp_test_issue"),
            ("CTS", "idp_test_cts"),
            ("TP", "idp_test_plan"),
            ("TS", "idp_test_stage"),
            ("FLOW", "flow_approval_edit"),
            ("TC", "idp_test_case"),
            ("REVIEW", "idp_feed_review"),
        ]);
        flow_tag_map.get(tag).map_or("".to_string(), |val| val.to_string())
    }
}
