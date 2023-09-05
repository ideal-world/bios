use std::collections::HashMap;

use bios_basic::test::test_http_client::TestHttpClient;

use bios_mw_flow::dto::flow_config_dto::FlowConfigModifyReq;
use bios_mw_flow::dto::flow_inst_dto::{
    FlowInstBatchBindReq, FlowInstBatchBindResp, FlowInstBindRelObjReq, FlowInstBindReq, FlowInstDetailResp, FlowInstFindNextTransitionResp, FlowInstFindNextTransitionsReq,
    FlowInstFindStateAndTransitionsReq, FlowInstFindStateAndTransitionsResp, FlowInstStartReq, FlowInstTransferReq, FlowInstTransferResp,
};
use bios_mw_flow::dto::flow_model_dto::{
    FlowModelAddCustomModelItemReq, FlowModelAddCustomModelReq, FlowModelAddCustomModelResp, FlowModelAggResp, FlowModelBindStateReq, FlowModelModifyReq,
    FlowModelSortStateInfoReq, FlowModelSortStatesReq, FlowModelSummaryResp, FlowModelUnbindStateReq, FlowTemplateModelResp,
};
use bios_mw_flow::dto::flow_state_dto::{FlowStateNameResp, FlowStateSummaryResp};
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

    // find default model
    let mut models: TardisPage<FlowModelSummaryResp> = flow_client.get("/cc/model/?tag=REQ&page_number=1&page_size=100").await;
    let init_model = models.records.pop().unwrap();
    info!("models: {:?}", init_model);
    assert_eq!(&init_model.name, "默认需求模板");
    assert_eq!(&init_model.owner, "");

    // mock tenant content
    ctx.own_paths = "t1".to_string();
    flow_client.set_auth(&ctx)?;
    // Get states list
    let states: TardisPage<FlowStateSummaryResp> = flow_client.get("/cc/state?tag=REQ&is_global=true&enabled=true&page_number=1&page_size=100").await;
    let init_state_id = states.records[0].id.clone();

    let template_id = "mock_template_id".to_string();
    // 1. set config
    let mut modify_configs = vec![];
    let codes = vec!["REQ", "MS", "PROJ", "ITER", "TICKET"];
    for code in codes {
        modify_configs.push(FlowConfigModifyReq {
            code: code.to_string(),
            value: "https://localhost:8080/mock/mock/exchange_data".to_string(),
        });
    }
    let _: Void = flow_client.post("/cs/config", &modify_configs).await;
    let configs: Option<TardisPage<KvItemSummaryResp>> = flow_client.get("/cs/config").await;
    info!("configs_new: {:?}", configs);
    // 2.Get model based on template id
    let result: HashMap<String, FlowTemplateModelResp> = flow_client.get(&format!("/cc/model/get_models?tag_ids=REQ&temp_id={}", template_id)).await;
    let req_model_id = result.get("REQ").unwrap().id.clone();

    let result: HashMap<String, FlowTemplateModelResp> = flow_client.get(&format!("/cc/model/get_models?tag_ids=TICKET&temp_id={}", template_id)).await;
    let ticket_model_id = result.get("TICKET").unwrap().id.clone();
    let ticket_model_agg: FlowModelAggResp = flow_client.get(&format!("/cc/model/{}", ticket_model_id)).await;

    let result: HashMap<String, FlowTemplateModelResp> = flow_client.get(&format!("/cc/model/get_models?tag_ids=ITER&temp_id={}", template_id)).await;
    let iter_model_id = result.get("ITER").unwrap().id.clone();
    let iter_model_agg: FlowModelAggResp = flow_client.get(&format!("/cc/model/{}", iter_model_id)).await;

    let result: HashMap<String, FlowTemplateModelResp> = flow_client.get(&format!("/cc/model/get_models?tag_ids=PROJ&temp_id={}", template_id)).await;
    let proj_model_id = result.get("PROJ").unwrap().id.clone();
    let proj_model_agg: FlowModelAggResp = flow_client.get(&format!("/cc/model/{}", proj_model_id)).await;

    // 3.modify model
    // Delete and add some transitions
    let _: Void = flow_client
        .post(
            &format!("/cc/model/{}/unbind_state", &req_model_id),
            &FlowModelUnbindStateReq { state_id: init_state_id.clone() },
        )
        .await;
    let _: Void = flow_client
        .post(
            &format!("/cc/model/{}/bind_state", &req_model_id),
            &FlowModelBindStateReq {
                state_id: init_state_id.clone(),
                sort: 10,
            },
        )
        .await;
    // resort state
    let mut sort_states = vec![];
    for (i, state) in states.records.iter().enumerate() {
        sort_states.push(FlowModelSortStateInfoReq {
            state_id: state.id.clone(),
            sort: i as i64 + 1,
        });
    }
    let _: Void = flow_client.post(&format!("/cc/model/{}/resort_state", &req_model_id), &FlowModelSortStatesReq { sort_states }).await;
    // get model detail
    let model_agg_old: FlowModelAggResp = flow_client.get(&format!("/cc/model/{}", &req_model_id)).await;
    // Set initial state
    let _: Void = flow_client
        .patch(
            &format!("/cc/model/{}", req_model_id),
            &FlowModelModifyReq {
                init_state_id: Some(init_state_id.clone()),
                ..Default::default()
            },
        )
        .await;
    // modify transitions
    let state_modify_trans = model_agg_old.states.iter().find(|state| state.name == "待开始").unwrap().transitions.iter().find(|trans| trans.name == "开始").unwrap();
    let _: Void = flow_client
        .patch(
            &format!("/cc/model/{}", req_model_id),
            &FlowModelModifyReq {
                modify_transitions: Some(vec![FlowTransitionModifyReq {
                    id: state_modify_trans.id.clone().into(),
                    name: Some(format!("{}-modify", &state_modify_trans.name).into()),
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
                }]),
                ..Default::default()
            },
        )
        .await;
    let field_modify_trans = model_agg_old.states.iter().find(|state| state.name == "进行中").unwrap().transitions.iter().find(|trans| trans.name == "完成").unwrap();
    let _: Void = flow_client
        .patch(
            &format!("/cc/model/{}", req_model_id),
            &FlowModelModifyReq {
                modify_transitions: Some(vec![FlowTransitionModifyReq {
                    id: field_modify_trans.id.clone().into(),
                    name: Some(format!("{}-modify", &field_modify_trans.name).into()),
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
                }]),
                ..Default::default()
            },
        )
        .await;
    let mut model_agg_new: FlowModelAggResp = flow_client.get(&format!("/cc/model/{}", req_model_id)).await;
    assert!(!model_agg_new.states.first_mut().unwrap().transitions.iter_mut().any(|trans| trans.transfer_by_auto).is_empty());
    info!("model_agg_new: {:?}", model_agg_new);
    // 4.Start a instance
    // mock tenant content
    ctx.own_paths = "t1/app01".to_string();
    flow_client.set_auth(&ctx)?;
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
        own_paths: "".to_string(),
        ak: "".to_string(),
        roles: vec!["admin".to_string()],
        groups: vec![],
        owner: "a001".to_string(),
        ..Default::default()
    })?;
    let next_transitions: Vec<FlowInstFindNextTransitionResp> =
        flow_client.put(&format!("/cc/inst/{}/transition/next", req_inst_id1), &FlowInstFindNextTransitionsReq { vars: None }).await;
    assert_eq!(next_transitions.len(), 2);
    assert!(next_transitions.iter().any(|trans| trans.next_flow_transition_name.contains("开始") && trans.vars_collect.as_ref().unwrap().len() == 2));
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
    assert!(state_and_next_transitions[0]
        .next_flow_transitions
        .iter()
        .any(|trans| trans.next_flow_transition_name.contains("开始") && trans.vars_collect.as_ref().unwrap().len() == 2));
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
    // 5. check post action endless loop
    ctx.own_paths = "t1".to_string();
    flow_client.set_auth(&ctx)?;
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
    // 7. bind app custom model
    ctx.own_paths = "t1/app01".to_string();
    flow_client.set_auth(&ctx)?;
    let mut modify_configs = vec![];
    let tags = vec!["REQ", "MS", "PROJ", "ITER", "TICKET", "MOCK"];
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

    // {"tag":"ISSUE","rel_business_objs":[{"rel_business_obj_id":"-c9rgVZOdUH_MbqofO4vc","current_state_name":"已解决","own_paths":"bzeUPv/JXYtZ0"}
    let _: String = flow_client
        .post(
            "/ci/inst/bind",
            &FlowInstBindReq {
                tag: "ISSUE".to_string(),
                rel_business_obj_id: "-c9rgVZOdUH_MbqofO4vc".to_string(),
                create_vars: None,
                current_state_name: Some("已解决".to_string()),
            },
        )
        .await;
    let mut rel_business_objs = vec![];
    for i in 5..8 {
        let rel_business_obj_id = format!("-c9rgVZOdUH_MbqofO4vc{}", i);
        rel_business_objs.push(FlowInstBindRelObjReq {
            rel_business_obj_id: Some(rel_business_obj_id),
            current_state_name: Some("已解决".to_string()),
            own_paths: Some("bzeUPv/JXYtZ0".to_string()),
            owner: Some("".to_string()),
        });
    }
    let _: Vec<FlowInstBatchBindResp> = flow_client
        .post(
            "/ci/inst/batch_bind",
            &FlowInstBatchBindReq {
                tag: "ISSUE".to_string(),
                rel_business_objs,
            },
        )
        .await;
    let mut rel_business_objs = vec![];
    for i in 0..10 {
        let rel_business_obj_id = format!("-c9rgVZOdUH_MbqofO4vc{}", i);
        rel_business_objs.push(FlowInstBindRelObjReq {
            rel_business_obj_id: Some(rel_business_obj_id),
            current_state_name: Some("已解决".to_string()),
            own_paths: Some("bzeUPv/JXYtZ0".to_string()),
            owner: Some("".to_string()),
        });
    }
    let _: Vec<FlowInstBatchBindResp> = flow_client
        .post(
            "/ci/inst/batch_bind",
            &FlowInstBatchBindReq {
                tag: "ISSUE".to_string(),
                rel_business_objs,
            },
        )
        .await;
    // let states: Vec<FlowStateNameResp> = flow_client.get("/cc/state/names?tag=REQ&app_ids=app01").await;

    Ok(())
}
