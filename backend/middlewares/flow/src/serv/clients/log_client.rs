use bios_sdk_invoke::clients::spi_log_client::SpiLogClient;
use serde::Serialize;

use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    tokio, TardisFuns, TardisFunsInst,
};

use crate::flow_constants;
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
    pub async fn add_ctx_task(tag: LogParamTag, key: Option<String>, content: LogParamContent, kind: Option<String>, op_kind: Option<String>, rel_key: Option<String>, ctx: &TardisContext) -> TardisResult<()> {
        let ctx_clone = ctx.clone();
        ctx.add_async_task(Box::new(|| {
            Box::pin(async move {
                let task_handle = tokio::spawn(async move {
                    let funs = flow_constants::get_tardis_inst();
                    Self::add_item(
                        tag,
                        content,
                        kind,
                        key.clone(),
                        op_kind,
                        rel_key,
                        Some(tardis::chrono::Utc::now().to_rfc3339()),
                        &funs,
                        &ctx_clone,
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
        kind: Option<String>,
        key: Option<String>,
        op: Option<String>,
        rel_key: Option<String>,
        ts: Option<String>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        // generate log item
        let tag: String = tag.into();
        let own_paths = if ctx.own_paths.len() < 2 { None } else { Some(ctx.own_paths.clone()) };
        let owner = if ctx.owner.len() < 2 { None } else { Some(ctx.owner.clone()) };
        SpiLogClient::add(
            &tag,
            &TardisFuns::json.obj_to_string(&content).expect("req_msg not a valid json value"),
            None,
            kind,
            key,
            op,
            rel_key,
            ts,
            owner,
            own_paths,
            funs,
            ctx,
        )
        .await?;
        Ok(())
    }
}
