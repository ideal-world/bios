use std::collections::HashMap;

use bios_basic::rbum::{dto::rbum_filer_dto::RbumBasicFilterReq, serv::rbum_item_serv::RbumItemCrudOperation};
use bios_sdk_invoke::clients::spi_log_client::LogItemFindResp;
use itertools::Itertools;
use serde_json::json;
use tardis::{
    basic::{dto::TardisContext, field::TrimString, result::TardisResult}, db::sea_orm::Set, futures::future::join_all, TardisFuns, TardisFunsInst
};

use crate::{domain::flow_inst, dto::{flow_inst_dto::FlowInstFilterReq, flow_model_dto::{FlowModelBindStateReq, FlowModelDetailResp, FlowModelFilterReq, FlowModelModifyReq}, flow_model_version_dto::{FlowModelVersionBindState, FlowModelVersionModifyReq, FlowModelVersionModifyState}, flow_state_dto::{FlowStateAddReq, FlowStateFilterReq, FlowStateRelModelModifyReq}, flow_sub_deploy_dto::{FlowSubDeployOneExportAggResp, FlowSubDeployOneImportReq, FlowSubDeployTowExportAggResp, FlowSubDeployTowImportReq}, flow_transition_dto::{FlowTransitionAddReq, FlowTransitionModifyReq}}};

use super::{clients::log_client::LogParamContent, flow_inst_serv::FlowInstServ, flow_log_serv::FlowLogServ, flow_model_serv::FlowModelServ, flow_state_serv::FlowStateServ, flow_transition_serv::FlowTransitionServ};

pub struct FlowSubDeployServ;

impl FlowSubDeployServ {
    pub(crate) async fn one_deploy_export(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<FlowSubDeployOneExportAggResp> {
        let mut states = HashMap::new();
        let mut main_models = HashMap::new();
        let mut delete_logs = HashMap::new();
        for main_model in FlowModelServ::find_detail_items(&FlowModelFilterReq {
            basic: RbumBasicFilterReq {
                own_paths: Some("".to_string()),
                with_sub_own_paths: true,
                ..Default::default()
            },
            main: Some(true),
            data_source: Some(id.to_string()),
            ..Default::default()
        }, Some(true), None, funs, ctx).await? {
            if main_models.contains_key(&main_model.tag) {
                continue;
            }
            let model_states = FlowStateServ::find_detail_items(&FlowStateFilterReq {
                basic: RbumBasicFilterReq {
                    ids: Some(main_model.states().into_iter().map(|state| state.id).collect_vec()),
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            }, None, None, funs, ctx).await?;
            for model_state in model_states {
                if !states.contains_key(&model_state.id) {
                    states.insert(model_state.id.clone(), model_state);
                }
            }

            delete_logs.insert(main_model.id.clone(), FlowLogServ::find_model_delete_state_log(&main_model, funs, ctx).await?);
            main_models.insert(main_model.tag.clone(), main_model);
        }
        let mut models = main_models.values().cloned().collect_vec();
        for approve_model in FlowModelServ::find_detail_items(&FlowModelFilterReq {
            basic: RbumBasicFilterReq {
                own_paths: Some("".to_string()),
                with_sub_own_paths: true,
                ..Default::default()
            },
            main: Some(false),
            data_source: Some(id.to_string()),
            ..Default::default()
        }, Some(true), None, funs, ctx).await? {
            let model_states = FlowStateServ::find_detail_items(&FlowStateFilterReq {
                basic: RbumBasicFilterReq {
                    ids: Some(approve_model.states().into_iter().map(|state| state.id).collect_vec()),
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            }, None, None, funs, ctx).await?;
            for model_state in model_states {
                if !states.contains_key(&model_state.id) {
                    states.insert(model_state.id.clone(), model_state);
                }
            }

            models.push(approve_model);
        }
        Ok(FlowSubDeployOneExportAggResp {
            states: states.values().cloned().collect_vec(),
            models,
            delete_logs,
        })
    }

    pub(crate) async fn sub_deploy_import(import_req: FlowSubDeployTowImportReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        for import_state in import_req.states {
            let mock_ctx = TardisContext {
                own_paths: import_state.own_paths.clone(),
                owner: import_state.owner.clone(),
                ..Default::default()
            };
            if FlowStateServ::get_item(&import_state.id, &FlowStateFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            }, funs, &mock_ctx).await.is_ok() {
                continue;
            }
            let mut add_req: FlowStateAddReq = import_state.clone().into();
            add_req.id = Some(TrimString::from(import_state.id.clone()));
            FlowStateServ::add_item(&mut add_req, funs, &mock_ctx).await?;
        }
        for new_model in import_req.models {
            let mock_ctx = TardisContext {
                own_paths: new_model.own_paths.clone(),
                owner: new_model.owner.clone(),
                ..Default::default()
            };
            if let Ok(original_model) = FlowModelServ::get_item(&new_model.id, &FlowModelFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            }, funs, &mock_ctx).await {
                let original_model_states = original_model.states();
                let new_model_states = new_model.states();
                // update bind states
                let bind_states = new_model_states.iter().filter(|new_state| !original_model_states.iter().any(|original_state| original_state.id == new_state.id)).collect_vec();
                for bind_state in bind_states.iter() {
                    FlowModelServ::modify_model(
                        &original_model.id,
                        &mut FlowModelModifyReq {
                            modify_version: Some(FlowModelVersionModifyReq {
                                bind_states: Some(vec![FlowModelVersionBindState {
                                    exist_state: Some(FlowModelBindStateReq {
                                        state_id: bind_state.id.clone(),
                                        ext: bind_state.ext.clone()
                                    }),
                                    ..Default::default()
                                }]),
                                ..Default::default()
                            }),
                            ..Default::default()
                        },
                        funs,
                        &mock_ctx,
                    )
                    .await?;
                }
                for bind_state in bind_states {
                    let add_transitions = bind_state.transitions.clone().into_iter().map(|transition| {
                        let transition_id = transition.id.clone();
                        let mut add_req = FlowTransitionAddReq::from(transition);
                        add_req.id = Some(transition_id);
                        add_req
                    }).collect_vec();
                    FlowTransitionServ::add_transitions(&original_model.current_version_id, &bind_state.id, &add_transitions, funs, ctx).await?;
                }
                // update unbind states
                let unbind_states = original_model_states.iter().filter(|original_state| !new_model_states.iter().any(|new_state| new_state.id == original_state.id)).collect_vec();
                for unbind_state in unbind_states {
                    FlowModelServ::modify_model(
                        &original_model.id,
                        &mut FlowModelModifyReq {
                            modify_version: Some(FlowModelVersionModifyReq {
                                unbind_states: Some(vec![unbind_state.id.clone()]),
                                ..Default::default()
                            }),
                            ..Default::default()
                        },
                        funs,
                        &mock_ctx,
                    )
                    .await?;
                }

                // modify exists states
                let exist_states = new_model_states.iter().filter(|new_state| original_model_states.iter().any(|original_state| new_state.id == original_state.id)).collect_vec();
                let modify_states_req = exist_states.iter().map(|exist_state| {
                    FlowModelVersionModifyState {
                        id: Some(exist_state.id.clone()),
                        modify_rel: Some(FlowStateRelModelModifyReq {
                            id: exist_state.id.clone(),
                            sort: Some(exist_state.ext.sort),
                            show_btns: exist_state.ext.show_btns.clone(),
                        }),
                        ..Default::default()
                    }
                }).collect_vec();
                FlowModelServ::modify_model(
                    &original_model.id,
                    &mut FlowModelModifyReq {
                        modify_version: Some(FlowModelVersionModifyReq {
                            modify_states: Some(modify_states_req),
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                    funs,
                    &mock_ctx,
                )
                .await?;
                // add or modify transitions
                let original_transitions = FlowModelServ::get_item(&new_model.id, &FlowModelFilterReq {
                    basic: RbumBasicFilterReq {
                        own_paths: Some("".to_string()),
                        with_sub_own_paths: true,
                        ..Default::default()
                    },
                    ..Default::default()
                }, funs, &mock_ctx).await?.transitions();
                let mut modify_transitions_req = vec![];
                for new_transition in new_model.transitions() {
                    if original_transitions.iter().any(|transition| transition.id == new_transition.id) {
                        let modify_transition = FlowTransitionModifyReq {
                            id: TrimString(new_transition.id.clone()),
                            name: Some(TrimString(new_transition.name.clone())),
                            from_flow_state_id: Some(new_transition.from_flow_state_id.clone()),
                            to_flow_state_id: Some(new_transition.to_flow_state_id.clone()),
                            transfer_by_auto: Some(new_transition.transfer_by_auto),
                            transfer_by_timer: Some(new_transition.transfer_by_timer.clone()),
                            guard_by_creator: Some(new_transition.guard_by_creator),
                            guard_by_his_operators: Some(new_transition.guard_by_his_operators),
                            guard_by_assigned: Some(new_transition.guard_by_assigned),
                            guard_by_spec_account_ids: Some(new_transition.guard_by_spec_account_ids.clone()),
                            guard_by_spec_role_ids: Some(new_transition.guard_by_spec_role_ids.clone()),
                            guard_by_spec_org_ids: Some(new_transition.guard_by_spec_org_ids.clone()),
                            guard_by_other_conds: new_transition.guard_by_other_conds(),
                            double_check: new_transition.double_check(),
                            vars_collect: new_transition.vars_collect(),
                            is_notify: Some(new_transition.is_notify),
                            action_by_pre_callback: Some(new_transition.action_by_pre_callback.clone()),
                            action_by_post_callback: Some(new_transition.action_by_post_callback.clone()),
                            action_by_post_changes: Some(new_transition.action_by_post_changes()),
                            action_by_post_var_changes: None,
                            action_by_post_state_changes: None,
                            action_by_front_changes: Some(new_transition.action_by_front_changes()),
                            sort: Some(new_transition.sort),
                        };
                        modify_transitions_req.push(modify_transition);
                    } else {
                        let transition_id = new_transition.id.clone();
                        let mut add_req = FlowTransitionAddReq::from(new_transition);
                        add_req.id = Some(transition_id);
                        FlowTransitionServ::add_transitions(&original_model.current_version_id, &add_req.from_flow_state_id.clone(), &[add_req], funs, ctx).await?;
                    }
                }
                FlowTransitionServ::modify_transitions(&original_model.current_version_id, &modify_transitions_req, funs, ctx).await?;
                FlowModelServ::modify_model(&original_model.id, &mut FlowModelModifyReq {
                    name: Some(TrimString(original_model.name.clone())),
                    front_conds: original_model.front_conds(),
                    ..Default::default()
                }, funs, &mock_ctx).await?;

                // update instances state
                if let Some(delete_logs) = import_req.delete_logs.get(&original_model.id).cloned() {
                    Self::modify_inst_state(&original_model, delete_logs, funs, &mock_ctx).await?;
                }
            } else {
                let mut add_req = new_model.create_add_req();
                FlowModelServ::add_item(&mut add_req, funs, &mock_ctx).await?;
            }
        }
        Ok(())
    }

    async fn modify_inst_state(flow_model: &FlowModelDetailResp, delete_logs: Option<Vec<LogItemFindResp>>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let global_ctx = TardisContext {
            own_paths: "".to_string(),
            ..ctx.clone()
        };
        let model_update_time = flow_model.update_time;
        let mut modify_state_map = HashMap::new();
        // init map
        for state in flow_model.states() {
            modify_state_map.insert(state.id.clone(), vec![state.id.clone()]);
        }
        // complete map
        if let Some(delete_logs) = delete_logs {
            for delete_log in delete_logs.into_iter().filter(|log| log.ts >= model_update_time) {
                let log_content = TardisFuns::json.json_to_obj::<LogParamContent>(delete_log.content.clone())?;
                let orginal_state = log_content.sub_id.clone().unwrap_or_default();
                let new_state = log_content.operand_id.clone().unwrap_or_default();
                let orginal_modify_states = modify_state_map.get(&orginal_state).cloned().unwrap_or_default();
                let state_map = modify_state_map.entry(new_state.clone()).or_insert(vec![]);
                for orginal_modify_state in orginal_modify_states {
                    state_map.push(orginal_modify_state);
                }
                modify_state_map.remove(&orginal_state);
            }
        }
        // update inst state
        for (new_state_id, orginal_states) in modify_state_map {
            for orginal_state_id in orginal_states {
                if orginal_state_id == new_state_id {
                    continue;
                }
                FlowInstServ::async_unsafe_modify_state(&FlowInstFilterReq {
                    flow_version_id: Some(flow_model.current_version_id.clone()),
                    current_state_id: Some(orginal_state_id.clone()),
                    with_sub: Some(true),
                    ..Default::default()
                }, &new_state_id, funs, &global_ctx).await?;
            }
        }

        Ok(())
    }

    pub(crate) async fn sub_deploy_export(
        // _start_time: String,
        // _end_time: String,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<FlowSubDeployTowExportAggResp> {
        let insts = FlowInstServ::find_detail_items(&FlowInstFilterReq {
            with_sub: Some(true),
            // update_time_start: Some(start_time),
            // update_time_end: Some(end_time),
            ..Default::default()
        }, funs, ctx).await?;
        Ok(FlowSubDeployTowExportAggResp {
            insts, 
        })
    }

    pub(crate) async fn one_deploy_import(import_req: FlowSubDeployOneImportReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if let Some(insts) = import_req.insts {
            let max_size = insts.len();
            let mut page = 0;
            let page_size = 100;
            loop {
                let current_insts = &insts[((page * page_size).min(max_size))..(((page+1) * page_size).min(max_size))];
                if current_insts.is_empty() {
                    break;
                }
                tardis::tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                join_all(
                    current_insts
                        .iter()
                        .map(|inst| async {
                            let flow_inst: flow_inst::ActiveModel = flow_inst::ActiveModel {
                                id: Set(inst.id.clone()),
                                code: Set(Some(inst.code.clone())),
                                tag: Set(Some(inst.tag.clone())),
                                rel_flow_version_id: Set(inst.rel_flow_version_id.clone()),
                                rel_business_obj_id: Set(inst.rel_business_obj_id.clone()),
                                rel_transition_id: Set(inst.rel_transition_id.clone()),
                    
                                current_state_id: Set(inst.current_state_id.clone()),
                    
                                create_vars: Set(inst.create_vars.clone().map(|vars| TardisFuns::json.obj_to_json(&vars).unwrap_or(json!({})))),
                                current_vars: Set(inst.current_vars.clone().map(|vars| TardisFuns::json.obj_to_json(&vars).unwrap_or(json!({})))),
                    
                                create_ctx: Set(inst.create_ctx.clone()),
        
                                finish_ctx: Set(inst.finish_ctx.clone()),
                                finish_time: Set(inst.finish_time),
                                finish_abort: Set(inst.finish_abort),
                                output_message: Set(inst.output_message.clone()),
                                
                                transitions: Set(inst.transitions.clone()),
                                artifacts: Set(inst.artifacts.clone()),
                                comments: Set(inst.comments.clone()),
        
                                own_paths: Set(inst.own_paths.clone()),
                                main: Set(inst.main),
                                rel_inst_id: Set(inst.rel_inst_id.clone()),
                                data_source: Set(inst.data_source.clone()),
        
                                create_time: Set(inst.create_time),
                                update_time: Set(inst.update_time),
                            };
                            match FlowInstServ::get(&inst.id, funs, ctx).await {
                                Ok(_) => {
                                    funs.db().update_one(flow_inst, ctx).await
                                },
                                Err(_e) => {
                                    funs.db().insert_one(flow_inst, ctx).await.map(|_| ())
                                }
                            }
                        })
                        .collect_vec(),
                )
                .await
                .into_iter()
                .collect::<TardisResult<Vec<_>>>()?;
                page += 1;
            }
        }
        Ok(())
    }
    
}
