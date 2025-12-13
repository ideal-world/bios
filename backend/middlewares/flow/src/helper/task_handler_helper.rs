use tardis::basic::{dto::TardisContext, result::TardisResult};

use crate::serv::clients::{
    log_client::{self, TASK_LOGV2_EXT_KEY, TASK_LOG_EXT_KEY},
    search_client,
};

pub async fn execute_async_task(ctx: &TardisContext) -> TardisResult<()> {
    // 先原子性地读取并移除任务，避免并发重复执行
    // 使用单个写锁来保证原子性
    let mut ext_write = ctx.ext.write().await;
    
    // 收集需要执行的任务
    let mut log_task: Option<String> = None;
    let mut logv2_task: Option<String> = None;
    let mut search_tasks: Vec<(String, String)> = Vec::new();
    
    // 读取并立即移除任务，保证原子性
    if let Some(val) = ext_write.remove(TASK_LOG_EXT_KEY) {
        log_task = Some(val);
    }
    if let Some(val) = ext_write.remove(TASK_LOGV2_EXT_KEY) {
        logv2_task = Some(val);
    }
    
    // 处理 search_ 开头的任务
    let search_keys: Vec<String> = ext_write
        .keys()
        .filter(|k| k.starts_with("search_"))
        .cloned()
        .collect();
    for key in search_keys {
        if let Some((_, search_key)) = key.split_once('_') {
            if let Some(val) = ext_write.remove(&key) {
                search_tasks.push((search_key.to_string(), val));
            }
        }
    }
    
    // 释放写锁
    drop(ext_write);
    
    // 执行所有已移除的任务
    if let Some(task_val) = log_task {
        log_client::FlowLogClient::execute_async_task(&task_val, ctx).await?;
    }
    if let Some(task_val) = logv2_task {
        log_client::FlowLogClient::execute_async_v2task(&task_val, ctx).await?;
    }
    for (search_key, val) in search_tasks {
        search_client::FlowSearchClient::execute_async_task(&search_key, &val, ctx).await?;
    }
    
    Ok(())
}
