use std::collections::HashMap;

use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;
use bios_basic::test::test_http_client::TestHttpClient;

use bios_mw_flow::dto::flow_config_dto::FlowConfigModifyReq;
use bios_mw_flow::dto::flow_inst_dto::{
    FlowInstBatchBindReq, FlowInstBatchBindResp, FlowInstBindRelObjReq, FlowInstBindReq, FlowInstDetailResp, FlowInstFindNextTransitionResp, FlowInstFindNextTransitionsReq,
    FlowInstFindStateAndTransitionsReq, FlowInstFindStateAndTransitionsResp, FlowInstStartReq, FlowInstTransferReq, FlowInstTransferResp,
};
use bios_mw_flow::dto::flow_model_dto::{
    FlowModelAddCustomModelItemReq, FlowModelAddCustomModelReq, FlowModelAddCustomModelResp, FlowModelAggResp, FlowModelModifyReq, FlowModelSortStateInfoReq,
    FlowModelSortStatesReq, FlowModelSummaryResp, FlowModelUnbindStateReq, FlowTemplateModelResp, FlowModelFindRelStateResp,
};
use bios_mw_flow::dto::flow_state_dto::FlowStateSummaryResp;
use bios_mw_flow::dto::flow_transition_dto::{
    FlowTransitionActionChangeInfo, FlowTransitionActionChangeKind, FlowTransitionDoubleCheckInfo, FlowTransitionModifyReq, StateChangeCondition, StateChangeConditionItem,
    StateChangeConditionOp,
};

use bios_mw_flow::dto::flow_var_dto::{FlowVarInfo, RbumDataTypeKind, RbumWidgetTypeKind};
use bios_sdk_invoke::clients::spi_kv_client::KvItemSummaryResp;
use tardis::basic::dto::TardisContext;

use tardis::basic::result::TardisResult;
use tardis::log::info;
use tardis::serde_json::json;
use tardis::web::poem_openapi::types::Type;
use tardis::web::web_resp::{TardisPage, Void};
use tardis::TardisFuns;

pub async fn test(flow_client: &mut TestHttpClient, _kv_client: &mut TestHttpClient) -> TardisResult<()> {
    info!("【test_flow_scenes_fsm】");

    let mut ctx = TardisContext {
        own_paths: "".to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "u001".to_string(),
        ..Default::default()
    };

    flow_client.set_auth(&ctx)?;

    // 1. enter platform
    // 1-1. check default model
    let mut models: TardisPage<FlowModelSummaryResp> = flow_client.get("/cc/model/?tag=REQ&page_number=1&page_size=100").await;
    let init_model = models.records.pop().unwrap();
    info!("models: {:?}", init_model);
    assert_eq!(&init_model.name, "待开始-进行中-已完成-已关闭");
    assert_eq!(&init_model.owner, "");
    // 1-2. set config
    let mut modify_configs = vec![];
    let codes = vec!["REQ", "PROJ", "ITER", "TICKET"];
    for code in codes {
        modify_configs.push(FlowConfigModifyReq {
            code: code.to_string(),
            value: "https://localhost:8080/mock/mock/exchange_data".to_string(),
        });
    }
    let _: Void = flow_client.post("/cs/config", &modify_configs).await;
    let configs: Option<TardisPage<KvItemSummaryResp>> = flow_client.get("/cs/config").await;
    info!("configs_new: {:?}", configs);
    // 1-3. mock import data
    ctx.own_paths = "t2/app02".to_string();
    flow_client.set_auth(&ctx)?;
    let template_id = "app01".to_string();
    let mut modify_configs = vec![];
    let tags = vec!["REQ", "PROJ", "ITER", "TICKET", "MOCK"];
    for tag in tags {
        modify_configs.push(FlowModelAddCustomModelItemReq {
            tag: tag.to_string(),
            feature_template_id: None,
        });
    }
    let result: Vec<FlowModelAddCustomModelResp> = flow_client
        .post(
            "/cc/model/add_custom_model",
            &FlowModelAddCustomModelReq {
                proj_template_id: Some(template_id.clone()),
                bind_model_objs: modify_configs,
            },
        )
        .await;
    assert!(result.into_iter().find(|resp| resp.tag == "MOCK").unwrap().model_id.is_none());

    let rel_business_obj_id = TardisFuns::field.nanoid();
    let _: String = flow_client
        .post(
            "/ci/inst/bind",
            &FlowInstBindReq {
                tag: "ITER".to_string(),
                rel_business_obj_id: rel_business_obj_id.clone(),
                create_vars: None,
                current_state_name: Some("进行中".to_string()),
            },
        )
        .await;
    let mut rel_business_objs = vec![];
    for i in 5..8 {
        let rel_business_obj_id_i = format!("{}{}", rel_business_obj_id, i);
        rel_business_objs.push(FlowInstBindRelObjReq {
            rel_business_obj_id: Some(rel_business_obj_id_i),
            current_state_name: Some("进行中".to_string()),
            own_paths: Some("t2/app02".to_string()),
            owner: Some("".to_string()),
        });
    }
    let _: Vec<FlowInstBatchBindResp> = flow_client
        .post(
            "/ci/inst/batch_bind",
            &FlowInstBatchBindReq {
                tag: "ITER".to_string(),
                rel_business_objs,
            },
        )
        .await;
    let mut rel_business_objs = vec![];
    for i in 0..10 {
        let rel_business_obj_id = format!("-c9rgVZOdUH_MbqofO4vc{}", i);
        rel_business_objs.push(FlowInstBindRelObjReq {
            rel_business_obj_id: Some(rel_business_obj_id),
            current_state_name: Some("进行中".to_string()),
            own_paths: Some("t2/app02".to_string()),
            owner: Some("".to_string()),
        });
    }
    let _: Vec<FlowInstBatchBindResp> = flow_client
        .post(
            "/ci/inst/batch_bind",
            &FlowInstBatchBindReq {
                tag: "ITER".to_string(),
                rel_business_objs,
            },
        )
        .await;
    // 2. enter tenant
    ctx.own_paths = "t1".to_string();
    flow_client.set_auth(&ctx)?;
    // 2-1. Get states list
    let req_states: TardisPage<FlowStateSummaryResp> = flow_client.get("/cc/state?tag=REQ&enabled=true&page_number=1&page_size=100").await;
    let init_state_id = req_states.records[0].id.clone();

    let mock_template_id = "mock_template_id".to_string();
    // 2-2. create flow models by template_id
    let result: HashMap<String, FlowTemplateModelResp> = flow_client.get(&format!("/cc/model/get_models?tag_ids=REQ,TICKET,ITER,PROJ&temp_id={}", mock_template_id)).await;
    let req_model_id = result.get("REQ").unwrap().id.clone();
    let req_model_agg: FlowModelAggResp = flow_client.get(&format!("/cc/model/{}", req_model_id)).await;
    let ticket_model_id = result.get("TICKET").unwrap().id.clone();
    let ticket_model_agg: FlowModelAggResp = flow_client.get(&format!("/cc/model/{}", ticket_model_id)).await;
    let iter_model_id = result.get("ITER").unwrap().id.clone();
    let iter_model_agg: FlowModelAggResp = flow_client.get(&format!("/cc/model/{}", iter_model_id)).await;
    let proj_model_id = result.get("PROJ").unwrap().id.clone();
    let proj_model_agg: FlowModelAggResp = flow_client.get(&format!("/cc/model/{}", proj_model_id)).await;
    // 2-3. check find rel states
    let _:Void = flow_client
        .post(
            &format!("/cc/model/{}/unbind_state", &proj_model_id),
            &FlowModelUnbindStateReq {
                state_id: proj_model_agg.states.iter().find(|state| state.name == "已关闭").unwrap().id.clone(),
            },
        )
        .await;
    let result: Vec<FlowModelFindRelStateResp> = flow_client.get(&format!("/cc/model/find_rel_status?tag=PROJ&rel_template_id={}", mock_template_id)).await;
    assert!(result.into_iter().find(|state| state.name == "已关闭").is_none());
    // 3.modify model
    // 3-1. resort state
    let mut sort_states = vec![];
    for (i, state) in req_states.records.iter().enumerate() {
        sort_states.push(FlowModelSortStateInfoReq {
            state_id: state.id.clone(),
            sort: i as i64 + 1,
        });
    }
    let _: Void = flow_client.post(&format!("/cc/model/{}/resort_state", &req_model_id), &FlowModelSortStatesReq { sort_states }).await;
    // 3-2. modify models
    let trans_start = req_model_agg.states.iter().find(|state| state.name == "待开始").unwrap().transitions.iter().find(|trans| trans.name == "开始").unwrap();
    let trans_complate = req_model_agg.states.iter().find(|state| state.name == "进行中").unwrap().transitions.iter().find(|trans| trans.name == "完成").unwrap();
    let trans_close = req_model_agg.states.iter().find(|state| state.name == "进行中").unwrap().transitions.iter().find(|trans| trans.name == "关闭").unwrap();

    let _: Void = flow_client
        .patch(
            &format!("/cc/model/{}", req_model_id),
            &FlowModelModifyReq {
                init_state_id: Some(init_state_id.clone()),
                modify_transitions: Some(vec![
                    FlowTransitionModifyReq {
                        id: trans_start.id.clone().into(),
                        name: Some(format!("{}-modify", &trans_start.name).into()),
                        from_flow_state_id: None,
                        to_flow_state_id: None,
                        transfer_by_auto: Some(true),
                        transfer_by_timer: None,
                        guard_by_creator: None,
                        guard_by_his_operators: None,
                        guard_by_assigned: None,
                        guard_by_spec_account_ids: None,
                        guard_by_spec_role_ids: None,
                        guard_by_spec_org_ids: None,
                        guard_by_other_conds: None,
                        vars_collect: Some(vec![
                            FlowVarInfo {
                                name: "assigned_to".to_string(),
                                label: "负责人".to_string(),
                                data_type: RbumDataTypeKind::STRING,
                                widget_type: RbumWidgetTypeKind::SELECT,
                                required: Some(true),
                                ..Default::default()
                            },
                            FlowVarInfo {
                                name: "start_end".to_string(),
                                label: "计划周期".to_string(),
                                data_type: RbumDataTypeKind::DATETIME,
                                widget_type: RbumWidgetTypeKind::DATETIME,
                                ..Default::default()
                            },
                        ]),
                        action_by_pre_callback: None,
                        action_by_post_callback: None,
                        action_by_post_changes: Some(vec![FlowTransitionActionChangeInfo {
                            kind: FlowTransitionActionChangeKind::State,
                            describe: "".to_string(),
                            obj_tag: Some("TICKET".to_string()),
                            obj_current_state_id: Some(vec![ticket_model_agg.init_state_id.clone()]),
                            change_condition: Some(StateChangeCondition {
                                current: true,
                                conditions: vec![StateChangeConditionItem {
                                    obj_tag: Some("ITER".to_string()),
                                    state_id: vec![iter_model_agg.init_state_id.clone()],
                                    op: StateChangeConditionOp::And,
                                }],
                            }),
                            changed_state_id: ticket_model_agg.states.iter().find(|state| state.name == "处理中").unwrap().id.clone(),
                            current: true,
                            var_name: "".to_string(),
                            changed_val: None,
                        }]),
                        double_check: Some(FlowTransitionDoubleCheckInfo {
                            is_open: true,
                            content: Some("再次确认该操作生效".to_string()),
                        }),
                    },
                    FlowTransitionModifyReq {
                        id: trans_complate.id.clone().into(),
                        name: Some(format!("{}-modify", &trans_complate.name).into()),
                        from_flow_state_id: None,
                        to_flow_state_id: None,
                        transfer_by_auto: Some(true),
                        transfer_by_timer: None,
                        guard_by_creator: None,
                        guard_by_his_operators: None,
                        guard_by_assigned: None,
                        guard_by_spec_account_ids: None,
                        guard_by_spec_role_ids: None,
                        guard_by_spec_org_ids: None,
                        guard_by_other_conds: None,
                        vars_collect: None,
                        action_by_pre_callback: None,
                        action_by_post_callback: None,
                        action_by_post_changes: Some(vec![FlowTransitionActionChangeInfo {
                            kind: FlowTransitionActionChangeKind::Var,
                            describe: "".to_string(),
                            obj_tag: Some("".to_string()),
                            obj_current_state_id: None,
                            change_condition: None,
                            changed_state_id: "".to_string(),
                            current: false,
                            var_name: "id".to_string(),
                            changed_val: Some(json!("xxx".to_string())),
                        }]),
                        double_check: None,
                    },
                    FlowTransitionModifyReq {
                        id: trans_close.id.clone().into(),
                        name: None,
                        from_flow_state_id: None,
                        to_flow_state_id: None,
                        transfer_by_auto: None,
                        transfer_by_timer: None,
                        guard_by_creator: None,
                        guard_by_his_operators: None,
                        guard_by_assigned: Some(true),
                        guard_by_spec_account_ids: None,
                        guard_by_spec_role_ids: None,
                        guard_by_spec_org_ids: None,
                        guard_by_other_conds: None,
                        vars_collect: None,
                        action_by_pre_callback: None,
                        action_by_post_callback: None,
                        action_by_post_changes: None,
                        double_check: None,
                    },
                ]),
                ..Default::default()
            },
        )
        .await;
    // 3-3. check post action endless loop
    let proj_trans = proj_model_agg.states.iter().find(|state| state.name == "进行中").unwrap().transitions.iter().find(|trans| trans.name == "有风险").unwrap();
    let _: Void = flow_client
        .patch(
            &format!("/cc/model/{}", proj_model_id),
            &FlowModelModifyReq {
                modify_transitions: Some(vec![FlowTransitionModifyReq {
                    id: proj_trans.id.clone().into(),
                    name: None,
                    from_flow_state_id: None,
                    to_flow_state_id: None,
                    transfer_by_auto: None,
                    transfer_by_timer: None,
                    guard_by_creator: None,
                    guard_by_his_operators: None,
                    guard_by_assigned: None,
                    guard_by_spec_account_ids: None,
                    guard_by_spec_role_ids: None,
                    guard_by_spec_org_ids: None,
                    guard_by_other_conds: None,
                    vars_collect: None,
                    action_by_pre_callback: None,
                    action_by_post_callback: None,
                    action_by_post_changes: Some(vec![FlowTransitionActionChangeInfo {
                        kind: FlowTransitionActionChangeKind::State,
                        describe: "".to_string(),
                        obj_tag: Some("TICKET".to_string()),
                        obj_current_state_id: None,
                        change_condition: None,
                        changed_state_id: ticket_model_agg.states.iter().find(|state| state.name == "处理中").unwrap().id.clone(),
                        current: true,
                        var_name: "".to_string(),
                        changed_val: None,
                    }]),
                    double_check: None,
                }]),
                ..Default::default()
            },
        )
        .await;
    let ticket_model: FlowModelAggResp = flow_client.get(&format!("/cc/model/{}", &ticket_model_id)).await;
    let ticket_trans = ticket_model.states.iter().find(|state| state.name == "待处理").unwrap().transitions.iter().find(|trans| trans.name == "立即处理").unwrap();
    let endless_loop_error = flow_client
        .patch_resp::<FlowModelModifyReq, Void>(
            &format!("/cc/model/{}", ticket_model_id),
            &FlowModelModifyReq {
                modify_transitions: Some(vec![FlowTransitionModifyReq {
                    id: ticket_trans.id.clone().into(),
                    name: None,
                    from_flow_state_id: None,
                    to_flow_state_id: None,
                    transfer_by_auto: None,
                    transfer_by_timer: None,
                    guard_by_creator: None,
                    guard_by_his_operators: None,
                    guard_by_assigned: None,
                    guard_by_spec_account_ids: None,
                    guard_by_spec_role_ids: None,
                    guard_by_spec_org_ids: None,
                    guard_by_other_conds: None,
                    vars_collect: None,
                    action_by_pre_callback: None,
                    action_by_post_callback: None,
                    action_by_post_changes: Some(vec![FlowTransitionActionChangeInfo {
                        kind: FlowTransitionActionChangeKind::State,
                        describe: "".to_string(),
                        obj_tag: Some("PROJ".to_string()),
                        obj_current_state_id: None,
                        change_condition: None,
                        changed_state_id: proj_model_agg.states.iter().find(|state| state.name == "存在风险").unwrap().id.clone(),
                        current: true,
                        var_name: "".to_string(),
                        changed_val: None,
                    }]),
                    double_check: None,
                }]),
                ..Default::default()
            },
        )
        .await;
    assert_eq!(endless_loop_error.code, "404-flow-flow_model_Serv-after_modify_item");

    let mut model_agg_new: FlowModelAggResp = flow_client.get(&format!("/cc/model/{}", req_model_id)).await;
    assert!(!model_agg_new.states.first_mut().unwrap().transitions.iter_mut().any(|trans| trans.transfer_by_auto).is_empty());
    info!("model_agg_new: {:?}", model_agg_new);
    // 3-4. Share template models
    let share_template_id = "share_template_id".to_string();
    let share_template_models: HashMap<String, FlowTemplateModelResp> = flow_client.get(&format!("/cc/model/get_models?tag_ids=REQ&temp_id={}", share_template_id)).await;
    let req_share_model_id = share_template_models.get("REQ").unwrap().id.clone();
    let _: Void = flow_client
        .patch(
            &format!("/cc/model/{}", req_share_model_id),
            &FlowModelModifyReq {
                scope_level: Some(RbumScopeLevelKind::Root),
                ..Default::default()
            },
        )
        .await;
    ctx.own_paths = "t3/app03".to_string();
    flow_client.set_auth(&ctx)?;
    let mut result: Vec<FlowModelAddCustomModelResp> = flow_client
        .post(
            "/cc/model/add_custom_model",
            &FlowModelAddCustomModelReq {
                proj_template_id: Some(req_share_model_id.clone()),
                bind_model_objs: vec![FlowModelAddCustomModelItemReq {
                    tag: "REQ".to_string(),
                    feature_template_id: None,
                }],
            },
        )
        .await;
    assert!(result.pop().unwrap().model_id.is_some());
    // 4.Start a instance
    // mock tenant content
    ctx.own_paths = "t1/app01".to_string();
    flow_client.set_auth(&ctx)?;
    // create app flow model
    let mut modify_configs = vec![];
    let tags = vec!["REQ", "PROJ", "ITER", "TICKET"];
    for tag in tags {
        modify_configs.push(FlowModelAddCustomModelItemReq {
            tag: tag.to_string(),
            feature_template_id: None,
        });
    }
    let _: Vec<FlowModelAddCustomModelResp> = flow_client
        .post(
            "/cc/model/add_custom_model",
            &FlowModelAddCustomModelReq {
                proj_template_id: Some(mock_template_id.clone()),
                bind_model_objs: modify_configs,
            },
        )
        .await;

    let ticket_inst_rel_id = "mock-ticket-obj-id".to_string();
    let iter_inst_rel_id1 = "mock-iter-obj-id1".to_string();
    let iter_inst_rel_id2 = "mock-iter-obj-id2".to_string();
    let req_inst_id1: String = flow_client
        .post(
            "/cc/inst",
            &FlowInstStartReq {
                tag: "REQ".to_string(),
                create_vars: None,
                rel_business_obj_id: TardisFuns::field.nanoid(),
            },
        )
        .await;
    let req_inst_id2: String = flow_client
        .post(
            "/cc/inst",
            &FlowInstStartReq {
                tag: "REQ".to_string(),
                create_vars: None,
                rel_business_obj_id: TardisFuns::field.nanoid(),
            },
        )
        .await;
    let ticket_inst_id: String = flow_client
        .post(
            "/cc/inst",
            &FlowInstStartReq {
                tag: "TICKET".to_string(),
                create_vars: None,
                rel_business_obj_id: ticket_inst_rel_id.clone(),
            },
        )
        .await;
    let _iter_inst_id: String = flow_client
        .post(
            "/cc/inst",
            &FlowInstStartReq {
                tag: "ITER".to_string(),
                create_vars: None,
                rel_business_obj_id: iter_inst_rel_id1.clone(),
            },
        )
        .await;
    let _iter_inst_id: String = flow_client
        .post(
            "/cc/inst",
            &FlowInstStartReq {
                tag: "ITER".to_string(),
                create_vars: None,
                rel_business_obj_id: iter_inst_rel_id2.clone(),
            },
        )
        .await;
    // Get the state of a task that can be transferable
    let next_transitions: Vec<FlowInstFindNextTransitionResp> =
        flow_client.put(&format!("/cc/inst/{}/transition/next", req_inst_id1), &FlowInstFindNextTransitionsReq { vars: None }).await;
    assert_eq!(next_transitions.len(), 2);
    flow_client.set_auth(&TardisContext {
        own_paths: "t1/app01".to_string(),
        ak: "".to_string(),
        roles: vec!["admin".to_string()],
        groups: vec![],
        owner: "a001".to_string(),
        ..Default::default()
    })?;
    let next_transitions: Vec<FlowInstFindNextTransitionResp> =
        flow_client.put(&format!("/cc/inst/{}/transition/next", req_inst_id1), &FlowInstFindNextTransitionsReq { vars: None }).await;
    assert_eq!(next_transitions.len(), 2);
    assert!(next_transitions.iter().any(|trans| trans.next_flow_transition_name.contains("开始")));
    assert!(next_transitions.iter().any(|trans| trans.next_flow_transition_name.contains("关闭")));
    // Find the state and transfer information of the specified instances in batch
    let state_and_next_transitions: Vec<FlowInstFindStateAndTransitionsResp> = flow_client
        .put(
            "/cc/inst/batch/state_transitions",
            &vec![FlowInstFindStateAndTransitionsReq {
                flow_inst_id: req_inst_id1.clone(),
                vars: None,
            }],
        )
        .await;
    assert_eq!(state_and_next_transitions.len(), 1);
    assert_eq!(state_and_next_transitions[0].current_flow_state_name, "待开始");
    assert!(state_and_next_transitions[0].next_flow_transitions.iter().any(|trans| trans.next_flow_transition_name.contains("开始")));
    assert!(state_and_next_transitions[0].next_flow_transitions.iter().any(|trans| trans.next_flow_transition_name.contains("关闭")));
    // Transfer task status
    let transfer: FlowInstTransferResp = flow_client
        .put(
            &format!("/cc/inst/{}/transition/transfer", req_inst_id1),
            &FlowInstTransferReq {
                flow_transition_id: state_and_next_transitions[0]
                    .next_flow_transitions
                    .iter()
                    .find(|&trans| trans.next_flow_transition_name.contains("关闭"))
                    .unwrap()
                    .next_flow_transition_id
                    .to_string(),
                // vars: Some(TardisFuns::json.json_to_obj(json!({ "reason":"测试关闭" })).unwrap()),
                vars: Some(TardisFuns::json.json_to_obj(json!({})).unwrap()),
                message: None,
            },
        )
        .await;
    assert_eq!(
        transfer.new_flow_state_id,
        state_and_next_transitions[0].next_flow_transitions.iter().find(|&trans| trans.next_flow_transition_name.contains("关闭")).unwrap().next_flow_state_id.clone()
    );
    // check state is used
    let state_unbind_error = flow_client
        .post_resp::<FlowModelUnbindStateReq, Void>(
            &format!("/cc/model/{}/unbind_state", &req_model_id),
            &FlowModelUnbindStateReq {
                state_id: transfer.new_flow_state_id.clone(),
            },
        )
        .await;
    assert_eq!(state_unbind_error.code, "500-flow-flow_model_serv-unbind_state");
    // 6. post action
    // check original instance
    let ticket: FlowInstDetailResp = flow_client.get(&format!("/cc/inst/{}", ticket_inst_id)).await;
    assert_eq!(ticket.current_state_id, ticket_model_agg.init_state_id);
    // transfer trigger post action
    let state_and_next_transitions: Vec<FlowInstFindStateAndTransitionsResp> = flow_client
        .put(
            "/cc/inst/batch/state_transitions",
            &vec![FlowInstFindStateAndTransitionsReq {
                flow_inst_id: req_inst_id2.clone(),
                vars: None,
            }],
        )
        .await;
    let mut vars = HashMap::new();
    vars.insert("assigned_to".to_string(), json!("xxxx_001"));
    let _transfer: FlowInstTransferResp = flow_client
        .put(
            &format!("/cc/inst/{}/transition/transfer", req_inst_id2),
            &FlowInstTransferReq {
                flow_transition_id: state_and_next_transitions[0]
                    .next_flow_transitions
                    .iter()
                    .find(|trans| trans.next_flow_state_name == "进行中")
                    .unwrap()
                    .next_flow_transition_id
                    .clone(),
                vars: Some(vars),
                message: None,
            },
        )
        .await;
    let ticket: FlowInstDetailResp = flow_client.get(&format!("/cc/inst/{}", ticket_inst_id)).await;
    assert_eq!(ticket.current_state_id, ticket_model_agg.states.iter().find(|state| state.name == "处理中").unwrap().id);
    let state_and_next_transitions: Vec<FlowInstFindStateAndTransitionsResp> = flow_client
        .put(
            "/cc/inst/batch/state_transitions",
            &vec![FlowInstFindStateAndTransitionsReq {
                flow_inst_id: req_inst_id2.clone(),
                vars: None,
            }],
        )
        .await;
    assert_eq!(state_and_next_transitions[0].next_flow_transitions.len(), 1);
    let _transfer: FlowInstTransferResp = flow_client
        .put(
            &format!("/cc/inst/{}/transition/transfer", req_inst_id2),
            &FlowInstTransferReq {
                flow_transition_id: state_and_next_transitions[0]
                    .next_flow_transitions
                    .iter()
                    .find(|trans| trans.next_flow_state_name == "已完成")
                    .unwrap()
                    .next_flow_transition_id
                    .clone(),
                vars: None,
                message: None,
            },
        )
        .await;

    Ok(())
}
