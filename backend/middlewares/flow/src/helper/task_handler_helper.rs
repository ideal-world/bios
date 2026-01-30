use std::{collections::HashMap, str::FromStr};
use tardis::basic::{dto::TardisContext, result::TardisResult};
use tardis::TardisFuns;

use crate::{
    dto::flow_inst_dto::ModifyObjSearchExtReq,
    flow_constants,
    serv::clients::{
        log_client::{self, TASK_LOGV2_EXT_KEY, TASK_LOG_EXT_KEY},
        search_client::{FlowSearchClient, FlowSearchTaskKind},
    },
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
    
    // 分离不同类型的任务
    let mut instance_ids: Vec<String> = Vec::new();
    let mut business_obj_items: HashMap<String, ModifyObjSearchExtReq> = HashMap::new();
    let mut review_instance_items: HashMap<String, ModifyObjSearchExtReq> = HashMap::new();
    let mut other_tasks: Vec<(String, String)> = Vec::new();
    
    for (search_key, val) in search_tasks {
        let (kind, id) = search_key.split_once('_').unwrap_or_default();
        match FlowSearchTaskKind::from_str(kind) {
            Ok(FlowSearchTaskKind::AddInstance) | Ok(FlowSearchTaskKind::ModifyInstance) => {
                instance_ids.push(id.to_string());
            }
            Ok(FlowSearchTaskKind::ModifyBusinessObj) => {
                // 解析 ModifyObjSearchExtReq，如果同一个 id 有多个任务需要合并（参考 add_search_task 中的处理）
                let modify_req = match TardisFuns::json.str_to_obj::<ModifyObjSearchExtReq>(&val) {
                    Ok(r) => r,
                    Err(_) => continue, // 如果解析失败，跳过这个任务
                };
                let mut req = business_obj_items
                    .remove(id)
                    .unwrap_or_default();
                req.tag = modify_req.tag;
                if modify_req.status.is_some() {
                    req.status = modify_req.status;
                }
                if modify_req.rel_state.is_some() {
                    req.rel_state = modify_req.rel_state;
                }
                if modify_req.rel_transition_state_name.is_some() {
                    req.rel_transition_state_name = modify_req.rel_transition_state_name;
                }
                if modify_req.current_state_color.is_some() {
                    req.current_state_color = modify_req.current_state_color;
                }
                business_obj_items.insert(id.to_string(), req);
            }
            Ok(FlowSearchTaskKind::ModifyReviewInstance) => {
                // 解析 ModifyObjSearchExtReq，如果同一个 id 有多个任务需要合并（参考 add_search_task 中的处理）
                let modify_req = match TardisFuns::json.str_to_obj::<ModifyObjSearchExtReq>(&val) {
                    Ok(r) => r,
                    Err(_) => continue, // 如果解析失败，跳过这个任务
                };
                let mut req = business_obj_items
                    .remove(id)
                    .unwrap_or_default();
                req.tag = modify_req.tag;
                if modify_req.status.is_some() {
                    req.status = modify_req.status;
                }
                if modify_req.rel_state.is_some() {
                    req.rel_state = modify_req.rel_state;
                }
                if modify_req.rel_transition_state_name.is_some() {
                    req.rel_transition_state_name = modify_req.rel_transition_state_name;
                }
                if modify_req.current_state_color.is_some() {
                    req.current_state_color = modify_req.current_state_color;
                }
                review_instance_items.insert(id.to_string(), req);
            }
            _ => {
                other_tasks.push((search_key, val));
            }
        }
    }
    
    // 批量处理实例任务
    if !instance_ids.is_empty() {
        let funs = flow_constants::get_tardis_inst();
        FlowSearchClient::batch_add_or_modify_instance_search(&instance_ids, &funs, ctx).await?;
    }
    
    // 批量处理业务对象任务
    if !business_obj_items.is_empty() {
        let funs = flow_constants::get_tardis_inst();
        FlowSearchClient::batch_modify_business_obj_search_ext(&business_obj_items, &funs, ctx).await?;
    }
    
    // 单独处理其他任务
    for (search_key, val) in other_tasks {
        FlowSearchClient::execute_async_task(&search_key, &val, ctx).await?;
    }

    // 批量处理实例任务
    if !instance_ids.is_empty() {
        let funs = flow_constants::get_tardis_inst();
        FlowSearchClient::batch_modify_review_obj_search_ext(&review_instance_items, &funs, ctx).await?;
    }
    
    Ok(())
}
