use std::collections::HashMap;

use bios_basic::rbum::{dto::rbum_item_dto::RbumItemAddReq, serv::rbum_crud_serv::RbumCrudOperation};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    db::sea_orm::{sea_query::Query, ColumnTrait, Iterable},
    TardisFunsInst,
};

use crate::{domain, dto::*, reach_consts::*, serv::*};

pub async fn message_send(send_req: ReachMsgSendReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    tardis::tracing::debug!("[BIOS.Reach] input: {:?}", send_req);
    let err = |msg: &str| funs.err().not_found("reach", "event_listener", msg, "");

    if send_req.receives.is_empty() {
        return Err(err("receiver is empty"));
    }
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
        rel_reach_trigger_scene_id: Some(scene.id.clone()),
        ..Default::default()
    };

    // retrieve all related global configs, group by channel
    let global_configs = ReachTriggerGlobalConfigService::find_detail_rbums(filter, None, None, funs, ctx).await?.into_iter().fold(HashMap::new(), |mut map, item| {
        map.entry(item.rel_reach_channel).or_insert(item);
        map
    });
    if global_configs.is_empty() {
        return Err(err("global_configs is empty"));
    }

    // retrieve all instance configs, group by group_code
    let filter = &ReachTriggerInstanceConfigFilterReq {
        rel_reach_trigger_scene_id: Some(scene.id),
        rel_item_id: Some(send_req.rel_item_id),
        ..Default::default()
    };
    let instances = ReachTriggerInstanceConfigService::find_detail_rbums(filter, None, None, funs, ctx).await?;
    if instances.is_empty() {
        return Ok(());
    }
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

    let other_receive_collect = receive_group_code.into_iter().fold(HashMap::new(), |mut other_receive_collect, (group_code, receives)| {
        if let Some(instance_list) = instance_group_code.get_mut(&group_code) {
            if !instance_list.is_empty() {
                for i in instance_list {
                    for r in &receives {
                        other_receive_collect.entry((r.receive_kind, i.rel_reach_channel)).or_insert(Vec::new()).extend(r.receive_ids.clone())
                    }
                }
            }
        }
        other_receive_collect
    });
    // let (other_receive_collect, other_group_code) = receive_group_code.into_iter().fold(
    //     (HashMap::new(), HashSet::new()),
    //     |(mut other_receive_collect, mut other_group_code), (group_code, receives)| {
    //         if let Some(instance_list) = instance_group_code.get_mut(&group_code) {
    //             if !instance_list.is_empty() {
    //                 for r in receives {
    //                     other_receive_collect.entry(r.receive_kind).or_insert(Vec::new()).extend(r.receive_ids)
    //                 }
    //                 for i in instance_list {
    //                     other_group_code.insert(i.rel_reach_channel);
    //                 }
    //             }
    //         }
    //         (other_receive_collect, other_group_code)
    //     },
    // );

    for (_kind, gc) in global_configs {
        for ((receive_kind, rel_reach_channel), to_res_ids) in &other_receive_collect {
            if rel_reach_channel == &gc.rel_reach_channel && !gc.rel_reach_msg_signature_id.is_empty() && !gc.rel_reach_msg_template_id.is_empty() {
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
                        content_replace: tardis::serde_json::to_string(&send_req.replace).expect("convert from string:string map shouldn't fail"),
                    },
                    funs,
                    ctx,
                )
                .await?;
            }
        }
    }

    Ok(())
}
