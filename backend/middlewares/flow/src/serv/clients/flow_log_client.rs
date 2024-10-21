use bios_sdk_invoke::clients::{
    iam_client::IamClient,
    spi_log_client::{LogItemAddReq, SpiLogClient},
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
    pub subject: String,
    pub name: String,
    pub sub_kind: String,
}

pub enum LogParamTag {
    DynamicLog,
}

impl From<LogParamTag> for String {
    fn from(val: LogParamTag) -> Self {
        match val {
            LogParamTag::DynamicLog => "dynamic_log".to_string(),
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
        ctx: &TardisContext,
        push: bool,
    ) -> TardisResult<()> {
        let ctx_clone = ctx.clone();
        let push_clone = push; // 克隆 push 变量
        ctx.add_async_task(Box::new(move || {
            Box::pin(async move {
                let task_handle = tokio::spawn(async move {
                    let funs = flow_constants::get_tardis_inst();
                    Self::add_item(
                        tag,
                        content,
                        ext,
                        kind,
                        key.clone(),
                        op_kind,
                        rel_key,
                        Some(tardis::chrono::Utc::now().to_rfc3339()),
                        &funs,
                        &ctx_clone,
                        push_clone, // 使用克隆的 push 变量
                    )
                    .await
                    .unwrap();
                });
                task_handle.await.unwrap();
                Ok(())
            })
        }))
        .await
    }

    pub async fn add_item(
        tag: LogParamTag,
        content: LogParamContent,
        ext: Option<Value>,
        kind: Option<String>,
        key: Option<String>,
        op: Option<String>,
        rel_key: Option<String>,
        ts: Option<String>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
        push: bool,
    ) -> TardisResult<()> {
        // generate log item
        let tag: String = tag.into();
        let own_paths = if ctx.own_paths.len() < 2 { None } else { Some(ctx.own_paths.clone()) };
        let owner = if ctx.owner.len() < 2 { None } else { Some(ctx.owner.clone()) };
        let owner_name = IamClient::new("", funs, &ctx, funs.conf::<FlowConfig>().invoke.module_urls.get("iam").expect("missing iam base url"))
            .get_account(&ctx.owner, &ctx.own_paths)
            .await?
            .owner_name;

        let req = LogItemAddReq {
            tag: tag.to_string(),
            content: TardisFuns::json.obj_to_json(&content).expect("req_msg not a valid json value"),
            kind,
            ext,
            key,
            op,
            rel_key,
            idempotent_id: None,
            ts: ts.map(|ts| DateTime::parse_from_rfc3339(&ts).unwrap_or_default().with_timezone(&Utc)),
            owner,
            own_paths,
            msg: None,
            owner_name,
            push: push,
        };
        SpiLogClient::add(req, funs, ctx).await?;
        Ok(())
    }
}
