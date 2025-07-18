use std::collections::HashMap;
use tardis::{chrono::Duration, web::poem_openapi};

use bios_sdk_invoke::clients::{
    iam_client::IamClient,
    spi_log_client::{LogItemAddReq, LogItemAddV2Req, LogItemFindReq, LogItemFindResp, SpiLogClient},
};
use serde::{Deserialize, Serialize};

use serde_json::Value;
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    chrono::{DateTime, Utc},
    tokio,
    web::web_resp::TardisPage,
    TardisFuns, TardisFunsInst,
};

use crate::{flow_config::FlowConfig, flow_constants};
pub struct FlowLogClient;

pub const TASK_LOG_EXT_KEY: &str = "log_add_task";
pub const TASK_LOGV2_EXT_KEY: &str = "log_addv2_task";

#[derive(Deserialize, Serialize, Default, Debug, Clone, poem_openapi::Object)]
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
    pub jump: Option<bool>,
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
    FlowModel,
}

impl From<LogParamTag> for String {
    fn from(val: LogParamTag) -> Self {
        match val {
            LogParamTag::DynamicLog => "dynamic_log".to_string(),
            LogParamTag::ApprovalFlow => "approval_flow".to_string(),
            LogParamTag::FlowModel => "flow_model".to_string(),
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
    // 编辑
    Update,
    // 关联
    FeedRel,
    // 评审
    Review,
    // 删除
    Delete,
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
            LogParamOp::Update => "UPDATE".to_string(),
            LogParamOp::FeedRel => "FEED_REL".to_string(),
            LogParamOp::Review => "REVIEW".to_string(),
            LogParamOp::Delete => "DELETE".to_string(),
        }
    }
}

impl FlowLogClient {
    pub async fn batch_add_task(req: LogItemAddReq, _funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let task_key = TASK_LOG_EXT_KEY.to_string();
        let add_log_tasks = if let Some(val) = ctx.ext.read().await.get(&task_key) {
            let mut original_val = TardisFuns::json.str_to_obj::<Vec<LogItemAddReq>>(val)?;
            original_val.push(req);
            original_val
        } else {
            vec![req]
        };
        let new_val = TardisFuns::json.obj_to_string(&add_log_tasks)?;
        ctx.remove_ext(&task_key).await?;
        ctx.add_ext(&task_key, &new_val).await?;
        Ok(())
    }

    pub async fn execute_async_task(task_val: &str, ctx: &TardisContext) -> TardisResult<()> {
        let funs = flow_constants::get_tardis_inst();
        let mut add_log_tasks = TardisFuns::json.str_to_obj::<Vec<LogItemAddReq>>(task_val)?;
        let mut ts = tardis::chrono::Utc::now();
        for task in add_log_tasks.iter_mut() {
            ts += Duration::milliseconds(10);
            task.ts = Some(ts);
        }
        SpiLogClient::batch_add(add_log_tasks, &funs, ctx).await?;
        Ok(())
    }

    pub async fn batch_add_v2task(req: LogItemAddV2Req, _funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let task_key = TASK_LOGV2_EXT_KEY.to_string();
        let add_log_tasks = if let Some(val) = ctx.ext.read().await.get(&task_key) {
            let mut original_val = TardisFuns::json.str_to_obj::<Vec<LogItemAddV2Req>>(val)?;
            original_val.push(req);
            original_val
        } else {
            vec![req]
        };
        let new_val = TardisFuns::json.obj_to_string(&add_log_tasks)?;
        ctx.remove_ext(&task_key).await?;
        ctx.add_ext(&task_key, &new_val).await?;
        Ok(())
    }

    pub async fn execute_async_v2task(task_val: &str, ctx: &TardisContext) -> TardisResult<()> {
        let funs = flow_constants::get_tardis_inst();
        let mut add_log_tasks = TardisFuns::json.str_to_obj::<Vec<LogItemAddV2Req>>(task_val)?;
        let mut ts = tardis::chrono::Utc::now();
        for task in add_log_tasks.iter_mut() {
            ts += Duration::milliseconds(10);
            task.ts = Some(ts);
        }
        SpiLogClient::batch_addv2(add_log_tasks, &funs, ctx).await?;
        Ok(())
    }

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
                            None,
                            rel_key,
                            false,
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
                            None,
                            rel_key,
                            false,
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
        ts: Option<DateTime<Utc>>,
        rel_key: Option<String>,
        is_async: bool,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
        _push: bool,
    ) -> TardisResult<()> {
        let ts = if let Some(ts) = ts { ts } else { tardis::chrono::Utc::now() };
        // generate log item
        let tag: String = tag.into();
        let own_paths = if ctx.own_paths.len() < 2 { None } else { Some(ctx.own_paths.clone()) };
        let owner = if ctx.owner.len() < 2 { None } else { Some(ctx.owner.clone()) };

        let req = LogItemAddReq {
            id: None,
            tag,
            content: TardisFuns::json.obj_to_string(&content).expect("content not a valid json value"),
            kind,
            ext,
            key,
            op,
            rel_key,
            ts: Some(ts),
            owner,
            own_paths,
            data_source: None,
        };
        if is_async {
            Self::batch_add_task(req, funs, ctx).await?;
        } else {
            SpiLogClient::add(req, funs, ctx).await?;
        }
        Ok(())
    }

    pub async fn addv2_item(
        tag: LogParamTag,
        key: Option<String>,
        content: LogParamContent,
        ext: Option<Value>,
        kind: Option<String>,
        op: Option<String>,
        ts: Option<DateTime<Utc>>,
        rel_key: Option<String>,
        is_async: bool,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
        push: bool,
    ) -> TardisResult<()> {
        let ts = if let Some(ts) = ts { ts } else { tardis::chrono::Utc::now() };
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
            ts: Some(ts),
            owner,
            own_paths,
            msg: None,
            owner_name,
            push: Some(push),
            disable: None,
            data_source: None,
            ignore_push: None,
        };
        if is_async {
            Self::batch_add_v2task(req, funs, ctx).await?;
        } else {
            SpiLogClient::addv2(req, funs, ctx).await?;
        }
        Ok(())
    }

    pub async fn findv2(find_req: LogItemFindReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<TardisPage<LogItemFindResp>>> {
        SpiLogClient::findv2(find_req, funs, ctx).await
    }

    pub fn get_flow_kind_text(tag: &str) -> String {
        let flow_tag_map = HashMap::from([
            ("PRODUCT", "产品"),
            ("PROJECT_MS", "产品里程碑"),
            ("PROJ", "合同"),
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
            ("PROJECT_MS", "idp_feed_project_ms"),
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
