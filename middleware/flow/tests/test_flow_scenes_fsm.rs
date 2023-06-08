use std::collections::HashMap;

use bios_basic::rbum::rbum_enumeration::{RbumDataTypeKind, RbumWidgetTypeKind};
use bios_basic::test::test_http_client::TestHttpClient;
use bios_mw_flow::dto::flow_inst_dto::{
    FlowInstFindNextTransitionResp, FlowInstFindNextTransitionsReq, FlowInstFindStateAndTransitionsReq, FlowInstFindStateAndTransitionsResp, FlowInstStartReq, FlowInstTransferReq,
    FlowInstTransferResp,
};
use bios_mw_flow::dto::flow_model_dto::{FlowModelAddReq, FlowModelModifyReq};
use bios_mw_flow::dto::flow_state_dto::{FlowStateAddReq, FlowStateSummaryResp, FlowSysStateKind};
use bios_mw_flow::dto::flow_transition_dto::FlowTransitionAddReq;
use bios_mw_flow::dto::flow_var_dto::FlowVarInfo;
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;
use tardis::serde_json::json;
use tardis::web::web_resp::{TardisPage, Void};
use tardis::TardisFuns;

pub async fn test(client: &mut TestHttpClient) -> TardisResult<()> {
    info!("【test_flow_scenes_fsm】");

    let ctx = TardisContext {
        own_paths: "".to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "u001".to_string(),
        ..Default::default()
    };

    client.set_auth(&ctx)?;

    // Add some states
    let state_init_id: String = client
        .post(
            "/cc/state",
            &FlowStateAddReq {
                name: Some(TrimString("初始".to_string())),
                id_prefix: Some(TrimString("init".to_string())),
                tag: Some(vec!["proj_states".to_string()]),
                sys_state: FlowSysStateKind::Start,
                icon: None,
                info: None,
                state_kind: None,
                kind_conf: None,
                template: None,
                rel_state_id: None,
                scope_level: None,
                disabled: None,
            },
        )
        .await;
    let state_confirmed_id: String = client
        .post(
            "/cc/state",
            &FlowStateAddReq {
                name: Some(TrimString("已确认".to_string())),
                id_prefix: Some(TrimString("confirmed".to_string())),
                tag: Some(vec!["proj_states".to_string()]),
                sys_state: FlowSysStateKind::Progress,
                icon: None,
                info: None,
                state_kind: None,
                kind_conf: None,
                template: None,
                rel_state_id: None,
                scope_level: None,
                disabled: None,
            },
        )
        .await;
    let state_rejected_id: String = client
        .post(
            "/cc/state",
            &FlowStateAddReq {
                name: Some(TrimString("已拒绝".to_string())),
                id_prefix: Some(TrimString("rejected".to_string())),
                tag: Some(vec!["proj_states".to_string()]),
                sys_state: FlowSysStateKind::Progress,
                icon: None,
                info: None,
                state_kind: None,
                kind_conf: None,
                template: None,
                rel_state_id: None,
                scope_level: None,
                disabled: None,
            },
        )
        .await;
    let state_assigned_id: String = client
        .post(
            "/cc/state",
            &FlowStateAddReq {
                name: Some(TrimString("已分配".to_string())),
                id_prefix: Some(TrimString("assigned".to_string())),
                tag: Some(vec!["proj_states".to_string()]),
                sys_state: FlowSysStateKind::Progress,
                icon: None,
                info: None,
                state_kind: None,
                kind_conf: None,
                template: None,
                rel_state_id: None,
                scope_level: None,
                disabled: None,
            },
        )
        .await;
    let state_executing_id: String = client
        .post(
            "/cc/state",
            &FlowStateAddReq {
                name: Some(TrimString("执行中".to_string())),
                id_prefix: Some(TrimString("executing".to_string())),
                tag: Some(vec!["proj_states".to_string()]),
                sys_state: FlowSysStateKind::Progress,
                icon: None,
                info: None,
                state_kind: None,
                kind_conf: None,
                template: None,
                rel_state_id: None,
                scope_level: None,
                disabled: None,
            },
        )
        .await;
    let state_finish_id: String = client
        .post(
            "/cc/state",
            &FlowStateAddReq {
                name: Some(TrimString("已完成".to_string())),
                id_prefix: Some(TrimString("finish".to_string())),
                tag: Some(vec!["proj_states".to_string()]),
                sys_state: FlowSysStateKind::Finish,
                icon: None,
                info: None,
                state_kind: None,
                kind_conf: None,
                template: None,
                rel_state_id: None,
                scope_level: None,
                disabled: None,
            },
        )
        .await;
    // Get states list
    let _states: TardisPage<FlowStateSummaryResp> = client.get("/cc/state?tag=proj_states&template=false&enabled=true&page_number=1&page_size=100").await;
    // Add a model
    let model_id: String = client
        .post(
            "/cc/model",
            &FlowModelAddReq {
                name: TrimString("基础流程".to_string()),
                init_state_id: state_init_id.clone(),
                icon: None,
                info: None,
                transitions: None,
                tag: None,
                scope_level: None,
                disabled: None,
            },
        )
        .await;
    // Add some transitions
    let _: Void = client
        .patch(
            &format!("/cc/model/{}", model_id),
            &FlowModelModifyReq {
                add_transitions: Some(vec![
                    FlowTransitionAddReq {
                        name: Some(TrimString("确认任务".to_string())),
                        from_flow_state_id: state_init_id.clone(),
                        to_flow_state_id: state_confirmed_id.clone(),
                        guard_by_spec_role_ids: Some(vec!["admin".to_string()]),
                        transfer_by_auto: None,
                        transfer_by_timer: None,
                        guard_by_creator: None,
                        guard_by_his_operators: None,
                        guard_by_spec_account_ids: None,
                        guard_by_other_conds: None,
                        vars_collect: None,
                        action_by_pre_callback: None,
                        action_by_post_callback: None,
                    },
                    FlowTransitionAddReq {
                        name: Some(TrimString("拒绝任务".to_string())),
                        from_flow_state_id: state_init_id.clone(),
                        to_flow_state_id: state_rejected_id.clone(),
                        guard_by_spec_role_ids: Some(vec!["admin".to_string()]),
                        vars_collect: Some(vec![FlowVarInfo {
                            name: "reason".to_string(),
                            label: "原因".to_string(),
                            data_type: RbumDataTypeKind::String,
                            widget_type: RbumWidgetTypeKind::InputTxt,
                            note: None,
                            sort: None,
                            hide: None,
                            secret: None,
                            show_by_conds: None,
                            widget_columns: None,
                            default_value: None,
                            dyn_default_value: None,
                            options: None,
                            dyn_options: None,
                            required: None,
                            min_length: None,
                            max_length: None,
                            action: None,
                            ext: None,
                            parent_attr_name: None,
                        }]),
                        transfer_by_auto: None,
                        transfer_by_timer: None,
                        guard_by_creator: None,
                        guard_by_his_operators: None,
                        guard_by_spec_account_ids: None,
                        guard_by_other_conds: None,
                        action_by_pre_callback: None,
                        action_by_post_callback: None,
                    },
                    FlowTransitionAddReq {
                        name: Some(TrimString("分配任务".to_string())),
                        from_flow_state_id: state_confirmed_id.clone(),
                        to_flow_state_id: state_assigned_id.clone(),
                        guard_by_spec_role_ids: Some(vec!["mgr".to_string()]),
                        transfer_by_auto: None,
                        transfer_by_timer: None,
                        guard_by_creator: None,
                        guard_by_his_operators: None,
                        guard_by_spec_account_ids: None,
                        guard_by_other_conds: None,
                        vars_collect: None,
                        action_by_pre_callback: None,
                        action_by_post_callback: None,
                    },
                    FlowTransitionAddReq {
                        name: Some(TrimString("执行任务".to_string())),
                        from_flow_state_id: state_assigned_id.clone(),
                        to_flow_state_id: state_executing_id.clone(),
                        guard_by_his_operators: Some(true),
                        transfer_by_auto: None,
                        transfer_by_timer: None,
                        guard_by_creator: None,
                        guard_by_spec_account_ids: None,
                        guard_by_spec_role_ids: None,
                        guard_by_other_conds: None,
                        vars_collect: None,
                        action_by_pre_callback: None,
                        action_by_post_callback: None,
                    },
                    FlowTransitionAddReq {
                        name: Some(TrimString("关闭任务".to_string())),
                        from_flow_state_id: state_executing_id.clone(),
                        to_flow_state_id: state_finish_id.clone(),
                        guard_by_his_operators: Some(true),
                        transfer_by_auto: None,
                        transfer_by_timer: None,
                        guard_by_creator: None,
                        guard_by_spec_account_ids: None,
                        guard_by_spec_role_ids: None,
                        guard_by_other_conds: None,
                        vars_collect: None,
                        action_by_pre_callback: None,
                        action_by_post_callback: None,
                    },
                ]),
                name: None,
                icon: None,
                info: None,
                init_state_id: None,
                modify_transitions: None,
                delete_transitions: None,
                tag: None,
                scope_level: None,
                disabled: None,
            },
        )
        .await;
    // Start a instance
    let inst_id: String = client.post(&format!("/cc/inst?flow_model_id={}", model_id), &FlowInstStartReq { create_vars: None }).await;
    // Get the current status of some tasks
    let names: HashMap<String, String> = client.get(&format!("/cc/state/names?ids={}&ids={}", state_init_id, state_assigned_id)).await;
    assert_eq!(names[&state_init_id], "初始");
    assert_eq!(names[&state_assigned_id], "已分配");
    // Get the state of a task that can be transferable
    let next_transitions: Vec<FlowInstFindNextTransitionResp> = client.put(&format!("/cc/inst/{}/transition/next", inst_id), &FlowInstFindNextTransitionsReq { vars: None }).await;
    assert_eq!(next_transitions.len(), 0);
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
    assert_eq!(next_transitions[0].next_flow_transition_name, "确认任务");
    assert_eq!(next_transitions[1].next_flow_transition_name, "拒绝任务");
    assert_eq!(next_transitions[1].vars_collect.as_ref().unwrap().len(), 1);
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
    assert_eq!(state_and_next_transitions[0].current_flow_state_name, "初始");
    assert_eq!(state_and_next_transitions[0].next_flow_transitions[0].next_flow_transition_name, "确认任务");
    assert_eq!(state_and_next_transitions[0].next_flow_transitions[1].next_flow_transition_name, "拒绝任务");
    assert_eq!(state_and_next_transitions[0].next_flow_transitions[1].vars_collect.as_ref().unwrap().len(), 1);
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
    assert_eq!(transfer.new_flow_state_id, state_rejected_id);
    Ok(())
}
