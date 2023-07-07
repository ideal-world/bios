use std::collections::HashMap;

use bios_basic::test::test_http_client::TestHttpClient;
use bios_mw_flow::dto::flow_inst_dto::{
    FlowInstFindNextTransitionResp, FlowInstFindNextTransitionsReq, FlowInstFindStateAndTransitionsReq, FlowInstFindStateAndTransitionsResp, FlowInstStartReq, FlowInstTransferReq,
    FlowInstTransferResp,
};
use bios_mw_flow::dto::flow_model_dto::{
    FlowModelAggResp, FlowModelBindStateReq, FlowModelModifyReq, FlowModelSummaryResp, FlowModelUnbindStateReq, FlowTagKind, FlowTemplateModelResp,
};
use bios_mw_flow::dto::flow_state_dto::FlowStateSummaryResp;
use bios_mw_flow::dto::flow_transition_dto::FlowTransitionModifyReq;

use tardis::basic::dto::TardisContext;

use tardis::basic::result::TardisResult;
use tardis::log::info;
use tardis::serde_json::json;
use tardis::web::poem_openapi::types::Type;
use tardis::web::web_resp::{TardisPage, Void};
use tardis::TardisFuns;

pub async fn test(client: &mut TestHttpClient) -> TardisResult<()> {
    info!("【test_flow_scenes_fsm】");

    let mut ctx = TardisContext {
        own_paths: "".to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "u001".to_string(),
        ..Default::default()
    };

    client.set_auth(&ctx)?;

    // find default model
    let mut models: TardisPage<FlowModelSummaryResp> = client.get("/cc/model/?tag=REQ&page_number=1&page_size=100").await;
    let init_model = models.records.pop().unwrap();
    info!("models: {:?}", init_model);
    assert_eq!(&init_model.name, "默认需求模板");
    assert_eq!(&init_model.owner, "");

    // mock tenant content
    ctx.own_paths = "t1".to_string();
    client.set_auth(&ctx)?;
    // Get states list
    let states: TardisPage<FlowStateSummaryResp> = client.get("/cc/state?tag=REQ&is_global=true&enabled=true&page_number=1&page_size=100").await;
    let init_state_id = states.records[0].id.clone();

    let template_id = "mock_template_id".to_string();
    // 1.Get model based on template id
    let result: HashMap<String, FlowTemplateModelResp> = client.get(&format!("/cc/model/get_models?tag_ids=REQ&temp_id={}", template_id)).await;

    let model_id = result.get("REQ").unwrap().id.clone();
    // 2.modify model
    // Delete and add some transitions
    let _: Void = client
        .post(
            &format!("/cc/model/{}/unbind_state", &model_id),
            &FlowModelUnbindStateReq { state_id: init_state_id.clone() },
        )
        .await;
    let _: Void = client.post(&format!("/cc/model/{}/bind_state", &model_id), &FlowModelBindStateReq { state_id: init_state_id.clone() }).await;
    // get model detail
    let model_agg_old: FlowModelAggResp = client.get(&format!("/cc/model/{}", &model_id)).await;
    // Set initial state
    let _: Void = client
        .patch(
            &format!("/cc/model/{}", model_id),
            &FlowModelModifyReq {
                init_state_id: Some(init_state_id.clone()),
                ..Default::default()
            },
        )
        .await;
    // modify transitions
    let trans_modify = model_agg_old.states.last().unwrap().transitions[0].clone();
    let _: Void = client
        .patch(
            &format!("/cc/model/{}", model_id),
            &FlowModelModifyReq {
                modify_transitions: Some(vec![FlowTransitionModifyReq {
                    id: trans_modify.id.clone().into(),
                    name: Some(format!("{}-modify", &trans_modify.name).into()),
                    from_flow_state_id: None,
                    to_flow_state_id: None,
                    transfer_by_auto: Some(true),
                    transfer_by_timer: None,
                    guard_by_creator: None,
                    guard_by_his_operators: None,
                    guard_by_assigned: None,
                    guard_by_spec_account_ids: None,
                    guard_by_spec_role_ids: None,
                    guard_by_other_conds: None,
                    vars_collect: None,
                    action_by_pre_callback: None,
                    action_by_post_callback: None,
                }]),
                ..Default::default()
            },
        )
        .await;
    let mut model_agg_new: FlowModelAggResp = client.get(&format!("/cc/model/{}", model_id)).await;
    assert!(!model_agg_new.states.first_mut().unwrap().transitions.iter_mut().any(|trans| trans.transfer_by_auto).is_empty());
    info!("model_agg_new: {:?}", model_agg_new);
    // 3.Start a instance
    let inst_id: String = client
        .post(
            "/cc/inst",
            &FlowInstStartReq {
                tag: FlowTagKind::REQ,
                create_vars: None,
                rel_business_obj_id: "".to_string(),
            },
        )
        .await;
    // Get the state of a task that can be transferable
    let next_transitions: Vec<FlowInstFindNextTransitionResp> = client.put(&format!("/cc/inst/{}/transition/next", inst_id), &FlowInstFindNextTransitionsReq { vars: None }).await;
    assert_eq!(next_transitions.len(), 2);
    client.set_auth(&TardisContext {
        own_paths: "".to_string(),
        ak: "".to_string(),
        roles: vec!["admin".to_string()],
        groups: vec![],
        owner: "a001".to_string(),
        ..Default::default()
    })?;
    let next_transitions: Vec<FlowInstFindNextTransitionResp> = client.put(&format!("/cc/inst/{}/transition/next", inst_id), &FlowInstFindNextTransitionsReq { vars: None }).await;
    assert_eq!(next_transitions.len(), 2);
    assert_eq!(next_transitions[0].next_flow_transition_name, "关闭");
    assert_eq!(next_transitions[1].next_flow_transition_name, "开始-modify");
    assert_eq!(next_transitions[1].vars_collect.as_ref().unwrap().len(), 2);
    // Find the state and transfer information of the specified instances in batch
    let state_and_next_transitions: Vec<FlowInstFindStateAndTransitionsResp> = client
        .put(
            "/cc/inst/batch/state_transitions",
            &vec![FlowInstFindStateAndTransitionsReq {
                flow_inst_id: inst_id.clone(),
                vars: None,
            }],
        )
        .await;
    assert_eq!(state_and_next_transitions.len(), 1);
    assert_eq!(state_and_next_transitions[0].current_flow_state_name, "待开始");
    assert_eq!(state_and_next_transitions[0].next_flow_transitions[0].next_flow_transition_name, "关闭");
    assert_eq!(state_and_next_transitions[0].next_flow_transitions[1].next_flow_transition_name, "开始-modify");
    assert_eq!(state_and_next_transitions[0].next_flow_transitions[1].vars_collect.as_ref().unwrap().len(), 2);
    // Transfer task status
    let transfer: FlowInstTransferResp = client
        .put(
            &format!("/cc/inst/{}/transition/transfer", inst_id),
            &FlowInstTransferReq {
                flow_transition_id: state_and_next_transitions[0].next_flow_transitions[1].next_flow_transition_id.clone(),
                vars: Some(TardisFuns::json.json_to_obj(json!({ "reason":"测试关闭" })).unwrap()),
                message: None,
            },
        )
        .await;
    assert_eq!(transfer.new_flow_state_id, state_and_next_transitions[0].next_flow_transitions[1].next_flow_state_id.clone());
    // // Add some transitions
    // let _: Void = client
    //     .patch(
    //         &format!("/cc/model/{}", model_id),
    //         &FlowModelModifyReq {
    //             add_transitions: Some(vec![
    //                 FlowTransitionAddReq {
    //                     name: Some(TrimString("确认任务".to_string())),
    //                     from_flow_state_id: state_init_id.clone(),
    //                     to_flow_state_id: state_confirmed_id.clone(),
    //                     guard_by_spec_role_ids: Some(vec!["admin".to_string()]),
    //                     transfer_by_auto: None,
    //                     transfer_by_timer: None,
    //                     guard_by_creator: None,
    //                     guard_by_his_operators: None,
    //                     guard_by_assigned: None,
    //                     guard_by_spec_account_ids: None,
    //                     guard_by_other_conds: None,
    //                     vars_collect: None,
    //                     action_by_pre_callback: None,
    //                     action_by_post_callback: None,
    //                 },
    //                 FlowTransitionAddReq {
    //                     name: Some(TrimString("拒绝任务".to_string())),
    //                     from_flow_state_id: state_init_id.clone(),
    //                     to_flow_state_id: state_rejected_id.clone(),
    //                     guard_by_spec_role_ids: Some(vec!["admin".to_string()]),
    //                     vars_collect: Some(vec![FlowVarInfo {
    //                         name: "reason".to_string(),
    //                         label: "原因".to_string(),
    //                         data_type: RbumDataTypeKind::String,
    //                         widget_type: RbumWidgetTypeKind::InputTxt,
    //                         note: None,
    //                         sort: None,
    //                         hide: None,
    //                         secret: None,
    //                         show_by_conds: None,
    //                         widget_columns: None,
    //                         default_value: None,
    //                         dyn_default_value: None,
    //                         options: None,
    //                         dyn_options: None,
    //                         required: None,
    //                         min_length: None,
    //                         max_length: None,
    //                         action: None,
    //                         ext: None,
    //                         parent_attr_name: None,
    //                     }]),
    //                     transfer_by_auto: None,
    //                     transfer_by_timer: None,
    //                     guard_by_creator: None,
    //                     guard_by_his_operators: None,
    //                     guard_by_assigned: None,
    //                     guard_by_spec_account_ids: None,
    //                     guard_by_other_conds: None,
    //                     action_by_pre_callback: None,
    //                     action_by_post_callback: None,
    //                 },
    //                 FlowTransitionAddReq {
    //                     name: Some(TrimString("分配任务".to_string())),
    //                     from_flow_state_id: state_confirmed_id.clone(),
    //                     to_flow_state_id: state_assigned_id.clone(),
    //                     guard_by_spec_role_ids: Some(vec!["mgr".to_string()]),
    //                     transfer_by_auto: None,
    //                     transfer_by_timer: None,
    //                     guard_by_creator: None,
    //                     guard_by_his_operators: None,
    //                     guard_by_assigned: None,
    //                     guard_by_spec_account_ids: None,
    //                     guard_by_other_conds: None,
    //                     vars_collect: None,
    //                     action_by_pre_callback: None,
    //                     action_by_post_callback: None,
    //                 },
    //                 FlowTransitionAddReq {
    //                     name: Some(TrimString("执行任务".to_string())),
    //                     from_flow_state_id: state_assigned_id.clone(),
    //                     to_flow_state_id: state_executing_id.clone(),
    //                     guard_by_his_operators: Some(true),
    //                     transfer_by_auto: None,
    //                     transfer_by_timer: None,
    //                     guard_by_creator: None,
    //                     guard_by_assigned: None,
    //                     guard_by_spec_account_ids: None,
    //                     guard_by_spec_role_ids: None,
    //                     guard_by_other_conds: None,
    //                     vars_collect: None,
    //                     action_by_pre_callback: None,
    //                     action_by_post_callback: None,
    //                 },
    //                 FlowTransitionAddReq {
    //                     name: Some(TrimString("关闭任务".to_string())),
    //                     from_flow_state_id: state_executing_id.clone(),
    //                     to_flow_state_id: state_finish_id.clone(),
    //                     guard_by_his_operators: Some(true),
    //                     transfer_by_auto: None,
    //                     transfer_by_timer: None,
    //                     guard_by_creator: None,
    //                     guard_by_assigned: None,
    //                     guard_by_spec_account_ids: None,
    //                     guard_by_spec_role_ids: None,
    //                     guard_by_other_conds: None,
    //                     vars_collect: None,
    //                     action_by_pre_callback: None,
    //                     action_by_post_callback: None,
    //                 },
    //             ]),
    //             name: None,
    //             icon: None,
    //             info: None,
    //             init_state_id: None,
    //             modify_transitions: None,
    //             delete_transitions: None,
    //             tag: None,
    //             scope_level: None,
    //             disabled: None,
    //             template: None,
    //         },
    //     )
    //     .await;

    // let _model_agg: FlowModelAggResp = client.get(&format!("/cc/model/{}", model_id)).await;

    // // Start a instance
    // let inst_id: String = client
    //     .post(
    //         "/cc/inst",
    //         &FlowInstStartReq {
    //             tag: FlowTagKind::Project,
    //             create_vars: None,
    //             rel_business_obj_id:"".to_string(),
    //         },
    //     )
    //     .await;
    // // Get the current status of some tasks
    // let names: HashMap<String, String> = client.get(&format!("/cc/state/names?ids={}&ids={}", state_init_id, state_assigned_id)).await;
    // assert_eq!(names[&state_init_id], "初始");
    // assert_eq!(names[&state_assigned_id], "已分配");

    Ok(())
}
