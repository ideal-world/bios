use std::collections::{HashMap, HashSet};

use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumItemBasicFilterReq};
use bios_basic::rbum::{dto::rbum_item_dto::RbumItemAddReq, serv::rbum_crud_serv::RbumCrudOperation};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    db::sea_orm::{sea_query::Query, ColumnTrait, Iterable},
    TardisFunsInst,
};

use crate::{domain, dto::*, reach_constants::*, serv::*};

pub async fn message_send(send_req: ReachMsgSendReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    tardis::tracing::debug!("[BIOS.Reach] input: {:?}", send_req);
    let err = |msg: &str| funs.err().not_found("reach", "event_listener", msg, "");

    if send_req.replace.values().any(|v| v.is_none()) {
        tardis::log::warn!("[BIOS.Reach] replace contains None values, skipping message send ({:?})", send_req.replace);
        tardis::tracing::warn!("[BIOS.Reach] replace contains None values, skipping message send");
        return Ok(());
    }

    // if send_req.receives.is_empty() {
    //     return Err(err("receiver is empty"));
    // }
    // trigger scene
    let scene = funs
        .db()
        .get_dto::<domain::trigger_scene::Model>(
            Query::select()
                .columns(domain::trigger_scene::Column::iter())
                .from(domain::trigger_scene::Entity)
                .and_where(domain::trigger_scene::Column::Code.eq(&send_req.scene_code)),
        )
        .await?
        .ok_or_else(|| funs.err().not_found("reach", "event_listener", "cannot find scene", ""))?;
    let filter = &ReachTriggerGlobalConfigFilterReq {
        base_filter: RbumItemBasicFilterReq {
            basic: RbumBasicFilterReq {
                with_sub_own_paths: true,
                own_paths: Some(String::default()),
                ..Default::default()
            },
            ..Default::default()
        },
        rel_reach_trigger_scene_id: Some(scene.id.clone()),
        ..Default::default()
    };

    // retrieve all related global configs, group by channel
    let mut global_configs = ReachTriggerGlobalConfigService::find_detail_rbums(filter, None, None, funs, ctx).await?.into_iter().fold(HashMap::new(), |mut map, item| {
        map.entry(item.rel_reach_channel).or_insert(item);
        map
    });
    if global_configs.is_empty() {
        return Err(err("global_configs is empty"));
    }

    // retrieve all instance configs, group by group_code
    let filter = &ReachTriggerInstanceConfigFilterReq {
        base_filter: RbumItemBasicFilterReq {
            basic: RbumBasicFilterReq {
                with_sub_own_paths: true,
                own_paths: Some(String::default()),
                ..Default::default()
            },
            ..Default::default()
        },
        rel_reach_trigger_scene_id: Some(scene.id),
        rel_item_id: Some(send_req.rel_item_id.clone()),
        ..Default::default()
    };
    let instances = ReachTriggerInstanceConfigService::find_detail_rbums(filter, None, None, funs, ctx).await?;
    if instances.is_empty() {
        return Ok(());
    }
    if let Some(webhook_global_config) = global_configs.remove(&ReachChannelKind::WebHook) {
        let webhook_instances = instances.iter().filter(|inst| inst.rel_reach_channel == ReachChannelKind::WebHook).cloned().collect();
        send_webhook_message(send_req.clone(), webhook_instances, webhook_global_config.clone(), funs, ctx).await?;
    }
    let non_webhook_instances = instances.into_iter().filter(|inst| inst.rel_reach_channel != ReachChannelKind::WebHook).collect();
    send_non_webhook_message(send_req, non_webhook_instances, global_configs.clone(), funs, ctx).await?;
    Ok(())
}

async fn send_non_webhook_message(send_req: ReachMsgSendReq, instances: Vec<ReachTriggerInstanceConfigDetailResp>, global_configs: HashMap<ReachChannelKind, ReachTriggerGlobalConfigDetailResp>,
    funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let replace = send_req.replace.iter().filter_map(|(k, v)| v.as_ref().map(|v| (k.clone(), v.clone()))).collect::<HashMap<String, String>>();
    let receive_group_code = send_req.receives.into_iter().fold(HashMap::<String, Vec<_>>::new(), |mut map, item| {
        map.entry(item.receive_group_code.clone()).or_default().push(item);
        map
    });
    let mut instance_group_code =
        instances.into_iter().filter(|inst| receive_group_code.contains_key(&inst.receive_group_code.clone())).fold(HashMap::<String, Vec<_>>::new(), |mut map, item| {
            map.entry(item.receive_group_code.clone()).or_default().push(item);
            map
        });

    if instance_group_code.is_empty() {
        return Ok(());
    }

    let other_receive_collect = receive_group_code.into_iter().fold(
        HashMap::<(ReachReceiveKind, ReachChannelKind), Vec<_>>::new(),
        |mut other_receive_collect: HashMap<(ReachReceiveKind, ReachChannelKind), Vec<_>>, (group_code, receives)| {
            if let Some(instance_list) = instance_group_code.get_mut(&group_code) {
                if !instance_list.is_empty() {
                    for i in instance_list {
                        for r in &receives {
                            other_receive_collect.entry((r.receive_kind, i.rel_reach_channel)).or_default().extend(r.receive_ids.clone())
                        }
                    }
                }
            }
            other_receive_collect
        },
    );
    // 记录发送的receive_kind to_res_ids rel_reach_channel
    let mut send_records: Vec<(ReachReceiveKind, String, ReachChannelKind)> = vec![];
    let global_ctx = TardisContext {
        own_paths: String::default(),
        ..ctx.clone()
    };
    for (_kind, gc) in global_configs {
        for ((receive_kind, rel_reach_channel), to_res_ids) in &other_receive_collect {
            if rel_reach_channel == &gc.rel_reach_channel && !gc.rel_reach_msg_signature_id.is_empty() && !gc.rel_reach_msg_template_id.is_empty() {
                if send_records.contains(&(*receive_kind, to_res_ids.join(";"), gc.rel_reach_channel)) {
                    continue;
                }
                ReachMessageServ::add_rbum(
                    &mut ReachMessageAddReq {
                        rbum_item_add_req: RbumItemAddReq {
                            id: Default::default(),
                            code: Default::default(),
                            name: "".into(),
                            rel_rbum_kind_id: RBUM_KIND_CODE_REACH_MESSAGE.into(),
                            rel_rbum_domain_id: DOMAIN_CODE.into(),
                            scope_level: Default::default(),
                            disabled: Default::default(),
                        },
                        from_res: Default::default(),
                        rel_reach_channel: gc.rel_reach_channel,
                        receive_kind: *receive_kind,
                        to_res_ids: to_res_ids.join(";"),
                        rel_reach_msg_signature_id: gc.rel_reach_msg_signature_id.clone(),
                        rel_reach_msg_template_id: gc.rel_reach_msg_template_id.clone(),
                        reach_status: ReachStatusKind::Pending,
                        content_replace: tardis::serde_json::to_string(&replace).expect("convert from string:string map shouldn't fail"),
                    },
                    funs,
                    &global_ctx,
                )
                .await?;
                send_records.push((*receive_kind, to_res_ids.join(";"), gc.rel_reach_channel));
            }
        }
    }

    Ok(())
}

async fn send_webhook_message(send_req: ReachMsgSendReq, instances: Vec<ReachTriggerInstanceConfigDetailResp>, global_config: ReachTriggerGlobalConfigDetailResp, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let global_ctx = TardisContext {
        own_paths: String::default(),
        ..ctx.clone()
    };
    if instances.iter().any(|i| i.rel_reach_channel == global_config.rel_reach_channel) && !global_config.rel_reach_msg_signature_id.is_empty() && !global_config.rel_reach_msg_template_id.is_empty() {
        ReachMessageServ::add_rbum(
            &mut ReachMessageAddReq {
                rbum_item_add_req: RbumItemAddReq {
                    id: Default::default(),
                    code: Default::default(),
                    name: "".into(),
                    rel_rbum_kind_id: RBUM_KIND_CODE_REACH_MESSAGE.into(),
                    rel_rbum_domain_id: DOMAIN_CODE.into(),
                    scope_level: Default::default(),
                    disabled: Default::default(),
                },
                from_res: Default::default(),
                rel_reach_channel: global_config.rel_reach_channel,
                receive_kind: ReachReceiveKind::Account,
                to_res_ids: "".to_string(),
                rel_reach_msg_signature_id: global_config.rel_reach_msg_signature_id.clone(),
                rel_reach_msg_template_id: global_config.rel_reach_msg_template_id.clone(),
                reach_status: ReachStatusKind::Pending,
                content_replace: tardis::serde_json::to_string(&send_req.replace).expect("convert from string:string map shouldn't fail"),
            },
            funs,
            &global_ctx,
        )
        .await?;
    }
    Ok(())
}

/// 去重批量发送请求
/// 如果body中的rel_item_id、replace相同，并且receives的receive_ids中存在相同的数据，则去掉该数据
/// 若去掉后该数组为空，则去掉这个receive。若去掉之后receives为空，则删除这个send_req
pub(crate) fn deduplicate_send_requests(body: &mut Vec<ReachMsgSendReq>) {
    let mut i = 0;
    while i < body.len() {
        let mut j = i + 1;
        while j < body.len() {
            // 检查rel_item_id和replace是否相同
            if body[i].rel_item_id == body[j].rel_item_id && body[i].replace == body[j].replace {
                // 去重receive_ids
                let mut receive_ids_to_remove = HashSet::new();
                
                // 收集body[i]中所有的receive_ids
                for receive in &body[i].receives {
                    for receive_id in &receive.receive_ids {
                        receive_ids_to_remove.insert(receive_id.clone());
                    }
                }
                
                // 从body[j]中移除重复的receive_ids
                for receive in &mut body[j].receives {
                    receive.receive_ids.retain(|id| !receive_ids_to_remove.contains(id));
                }
                
                // 移除空的receives
                body[j].receives.retain(|receive| !receive.receive_ids.is_empty());
            }
            j += 1;
        }
        i += 1;
    }
    
    // 移除空的send_req
    body.retain(|send_req| !send_req.receives.is_empty());
}
