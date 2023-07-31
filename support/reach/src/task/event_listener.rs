use std::collections::{HashMap, HashSet};

use bios_basic::rbum::{dto::rbum_item_dto::RbumItemAddReq, serv::rbum_crud_serv::RbumCrudOperation};
use tardis::{
    basic::result::TardisResult,
    db::sea_orm::{self, sea_query::Query, ColumnTrait},
    log, TardisFuns,
};

use crate::{consts::*, domain::*, dto::*, serv::*};

#[derive(Debug, Default, Clone)]
pub struct EventListener {}

impl EventListener {
    pub async fn run(&self) -> TardisResult<()> {
        let funs = get_tardis_inst();
        funs.mq()
            .response(MQ_REACH_TOPIC_MESSAGE, |(_, msg)| async move {
                log::debug!("Receive message : {msg}");
                let funs = get_tardis_inst();
                let err = |msg: &str| funs.err().not_found("reach", "event_listener", msg, "");
                let send_req: ReachMsgSendReq = TardisFuns::json.str_to_obj(&msg)?;
                if send_req.receives.is_empty() {
                    return Err(err("receiver is empty"));
                }
                let ctx = send_req.get_ctx();
                // find scene
                #[derive(sea_orm::FromQueryResult)]
                struct TriggerScene {
                    id: String,
                }
                let scene = funs
                    .db()
                    .get_dto::<TriggerScene>(Query::select().and_where(trigger_scene::Column::Code.eq(&send_req.scene_code)).limit(1))
                    .await?
                    .ok_or_else(|| funs.err().not_found("reach", "event_listener", "cannot find scene", ""))?;
                let filter = &ReachTriggerGlobalConfigFilterReq {
                    rel_reach_trigger_scene_id: Some(scene.id.clone()),
                    ..Default::default()
                };
                let global_configs =
                    ReachTriggerGlobalConfigService::find_detail_rbums(filter, None, None, &funs, &ctx).await?.into_iter().fold(HashMap::new(), |mut map, item| {
                        map.entry(item.rel_reach_channel).or_insert(item);
                        map
                    });
                if global_configs.is_empty() {
                    return Err(err("global_configs is empty"));
                }
                let filter = &ReachTriggerInstanceConfigFilterReq {
                    rel_reach_trigger_scene_id: Some(scene.id),
                    rel_item_id: Some(send_req.rel_item_id),
                    ..Default::default()
                };
                let instances = ReachTriggerInstanceConfigService::find_detail_rbums(filter, None, None, &funs, &ctx).await?;
                if instances.is_empty() {
                    return Ok(());
                }
                // group is nightly
                let receive_group_code = send_req.receives.into_iter().fold(HashMap::new(), |mut map, item| {
                    map.entry(item.receive_group_code.clone()).or_insert(Vec::new()).push(item);
                    map
                });

                let mut instance_group_code =
                    instances.into_iter().filter(|inst| receive_group_code.contains_key(&inst.receive_group_code.clone())).fold(HashMap::new(), |mut map, item| {
                        map.entry(item.receive_group_code.clone()).or_insert(Vec::new()).push(item);
                        map
                    });

                if instance_group_code.is_empty() {
                    return Ok(());
                }
                let (other_receive_collect, other_group_code) = receive_group_code.into_iter().fold(
                    (HashMap::new(), HashSet::new()),
                    |(mut other_receive_collect, mut other_group_code), (group_code, receives)| {
                        if let Some(instance_list) = instance_group_code.get_mut(&group_code) {
                            if !instance_list.is_empty() {
                                for r in receives {
                                    other_receive_collect.entry(r.receive_kind).or_insert(Vec::new()).extend(r.receive_ids)
                                }
                                for i in instance_list {
                                    other_group_code.insert(i.rel_reach_channel);
                                }
                            }
                        }
                        (other_receive_collect, other_group_code)
                    },
                );
                for (_kind, gc) in global_configs {
                    for (receive_kind, to_res_ids) in &other_receive_collect {
                        if other_group_code.contains(&gc.rel_reach_channel) {
                            ReachMessageServ::add_rbum(
                                &mut ReachMessageAddReq {
                                    rbum_item_add_req: RbumItemAddReq {
                                        id: Default::default(),
                                        code: Default::default(),
                                        name: "".into(),
                                        rel_rbum_kind_id: Default::default(),
                                        rel_rbum_domain_id: Default::default(),
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
                                &funs,
                                &ctx,
                            )
                            .await?;
                        }
                    }
                }
                TardisResult::Ok(())
            })
            .await?;
        Ok(())
    }
}
