use tardis::basic::{dto::TardisContext, result::TardisResult};

use crate::serv::clients::{
    log_client::{self, TASK_LOGV2_EXT_KEY, TASK_LOG_EXT_KEY},
    search_client,
};

pub async fn execute_async_task(ctx: &TardisContext) -> TardisResult<()> {
    let mut keys_to_remove = Vec::new();
    for (key, val) in ctx.ext.read().await.iter() {
        if key == TASK_LOG_EXT_KEY {
            log_client::FlowLogClient::execute_async_task(val, ctx).await?;
            keys_to_remove.push(key.clone());
        }
        if key == TASK_LOGV2_EXT_KEY {
            log_client::FlowLogClient::execute_async_v2task(val, ctx).await?;
            keys_to_remove.push(key.clone());
        }
        if key.starts_with("search_") {
            if let Some((_, search_key)) = key.split_once('_') {
                search_client::FlowSearchClient::execute_async_task(search_key, val, ctx).await?;
                keys_to_remove.push(key.clone());
            }
        }
    }
    // 清空已执行的任务，避免重复执行
    for key in keys_to_remove {
        ctx.remove_ext(&key).await?;
    }
    Ok(())
}
