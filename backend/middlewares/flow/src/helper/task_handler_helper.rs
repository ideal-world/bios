use tardis::basic::{dto::TardisContext, result::TardisResult};

use crate::serv::clients::{log_client::{self, TASK_LOGV2_EXT_KEY, TASK_LOG_EXT_KEY}, search_client};

pub async fn execute_async_task(ctx: &TardisContext) -> TardisResult<()> {
    for (key, val) in ctx.ext.read().await.iter() {
        if key == TASK_LOG_EXT_KEY {
            log_client::FlowLogClient::execute_async_task(val, ctx).await?;
        }
        if key == TASK_LOGV2_EXT_KEY {
            log_client::FlowLogClient::execute_async_v2task(val, ctx).await?;
        }
        if key.starts_with("search_") {
            if let Some((_, key)) = key.split_once('_') {
                search_client::FlowSearchClient::execute_async_task(key, val, ctx).await?;
            }
        }
    }
    Ok(())
}