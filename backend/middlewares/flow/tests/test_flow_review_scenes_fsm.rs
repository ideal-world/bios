use std::collections::HashMap;

use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;
use bios_basic::test::test_http_client::TestHttpClient;

use bios_iam::basic::dto::iam_account_dto::IamAccountAggAddReq;
use bios_iam::basic::dto::iam_app_dto::IamAppAggAddReq;
use bios_iam::basic::dto::iam_tenant_dto::IamTenantAggAddReq;
use bios_iam::iam_constants::RBUM_SCOPE_LEVEL_TENANT;
use bios_iam::iam_test_helper::BIOSWebTestClient;
use bios_mw_flow::dto::flow_cond_dto::{BasicQueryCondInfo, BasicQueryOpKind};
use bios_mw_flow::dto::flow_config_dto::FlowConfigModifyReq;

use bios_mw_flow::dto::flow_inst_dto::{
    FlowInstDetailResp, FlowInstFindStateAndTransitionsReq, FlowInstFindStateAndTransitionsResp, FlowInstOperateReq, FlowInstRelChildObj, FlowInstStartReq, FlowInstTransferReq,
    FlowInstTransferResp,
};
use bios_mw_flow::dto::flow_model_dto::{
    FlowModelAddReq, FlowModelAggResp, FlowModelAssociativeOperationKind, FlowModelBindNewStateReq, FlowModelBindStateReq, FlowModelCopyOrReferenceCiReq,
    FlowModelCopyOrReferenceReq, FlowModelDetailResp, FlowModelKind, FlowModelModifyReq, FlowModelStatus, FlowModelSummaryResp,
};
use bios_mw_flow::dto::flow_model_version_dto::{
    FlowModelVersionAddReq, FlowModelVersionBindState, FlowModelVersionDetailResp, FlowModelVersionModifyReq, FlowModelVersionModifyState, FlowModelVesionState,
};
use bios_mw_flow::dto::flow_state_dto::{
    FLowStateIdAndName, FLowStateKindConf, FlowStateAddReq, FlowStateAggResp, FlowStateApproval, FlowStateCountersignConf, FlowStateCountersignKind, FlowStateForm, FlowStateKind,
    FlowStateModifyReq, FlowStateOperatorKind, FlowStateRelModelExt, FlowStateRelModelModifyReq, FlowStateSummaryResp, FlowStatusAutoStrategyKind, FlowStatusMultiApprovalKind,
    FlowSysStateKind,
};

use bios_mw_flow::dto::flow_transition_dto::{FlowTransitionAddReq, FlowTransitionModifyReq};
use bios_sdk_invoke::clients::spi_kv_client::KvItemSummaryResp;
use bios_sdk_invoke::dto::search_item_dto::SearchItemAddReq;
use bios_spi_search::dto::search_item_dto::{SearchItemQueryReq, SearchItemSearchCtxReq, SearchItemSearchPageReq, SearchItemSearchReq, SearchItemSearchResp};
use serde_json::json;
use tardis::basic::dto::TardisContext;

use std::time::Duration;
use tardis::basic::result::TardisResult;
use tardis::log::{debug, info};
use tardis::tokio::time::sleep;
use tardis::web::web_resp::{TardisPage, TardisResp, Void};
use tardis::TardisFuns;

pub async fn test(
    flow_client: &mut TestHttpClient,
    search_client: &mut TestHttpClient,
    kv_client: &mut TestHttpClient,
    iam_client: &mut BIOSWebTestClient,
    sysadmin_name: String,
    sysadmin_password: String,
) -> TardisResult<()> {
    info!("【test_flow_scenes_fsm】");
    let mut ctx = TardisContext {
        own_paths: "".to_string(),
        ak: "u001".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "u001".to_string(),
        ..Default::default()
    };

    flow_client.set_auth(&ctx)?;
    kv_client.set_auth(&ctx)?;
    search_client.set_auth(&ctx)?;
    let iam_data = load_iam_data(search_client, iam_client, sysadmin_name, sysadmin_password).await?;
    let t1_data = &iam_data[0];
    // 1. enter platform
    // 1-1. check default model
    let mut models: TardisPage<FlowModelSummaryResp> = flow_client.get("/cc/model?tag=REVIEW&page_number=1&page_size=100").await;
    let init_review_model = models.records.pop().unwrap();
    assert_eq!(&init_review_model.name, "评审通用审批流");
    assert_eq!(&init_review_model.owner, "");
    let mut models: TardisPage<FlowModelSummaryResp> = flow_client.get("/cc/model?tag=REQ&page_number=1&page_size=100").await;
    let init_req_model = models.records.pop().unwrap();
    assert_eq!(&init_req_model.name, "待开始-进行中-已完成-已关闭");
    assert_eq!(&init_req_model.owner, "");
    // 1-2. set config
    let mut modify_configs = vec![];
    let codes = vec!["REQ", "PROJ", "ITER", "TICKET", "REVIEW"];
    for code in codes {
        modify_configs.push(FlowConfigModifyReq {
            code: code.to_string(),
            value: "http://127.0.0.1:8080/mock/mock/exchange_data".to_string(),
        });
    }
    let _: Void = flow_client.post("/cs/config", &modify_configs).await;
    let configs: Option<TardisPage<KvItemSummaryResp>> = flow_client.get("/cs/config").await;
    info!("configs_new: {:?}", configs);
    // 1-4. Get states list
    let req_states: TardisPage<FlowStateSummaryResp> = flow_client.get("/cc/state?tag=REQ&enabled=true&page_number=1&page_size=100").await;
    let init_state_id = req_states.records[0].id.clone(); // 待开始
    let processing_state_id = req_states.records[1].id.clone(); // 进行中
    let finish_state_id = req_states.records[2].id.clone(); // 已完成
    let closed_state_id = req_states.records[3].id.clone(); // 已关闭
                                                            // 2-1 进入项目1
    let t1_tenant_id = t1_data.tenant_id.clone();
    let t1_app_id = t1_data.app_ids.first().cloned().unwrap_or_default();
    let t1_account_a_id = t1_data.accounts[0].clone();
    let t1_account_b_id = t1_data.accounts[1].clone();
    ctx.owner = t1_account_a_id.clone();
    ctx.own_paths = format!("{}/{}", t1_tenant_id, t1_app_id).to_string();
    flow_client.set_auth(&ctx)?;
    kv_client.set_auth(&ctx)?;
    search_client.set_auth(&ctx)?;
    // 1-3 初始化需求业务数据
    let req_a_obj_id = TardisFuns::field.nanoid();
    let _: Void = search_client
        .put(
            "/ci/item",
            &json!({
                "tag":"idp_project",
                "kind": "idp_feed_req",
                "key": req_a_obj_id,
                "title": "需求A",
                "content": "需求A",
                "owner":ctx.owner,
                "own_paths":ctx.own_paths,
                "create_time":"2022-09-26T23:23:59.000Z",
                "update_time": "2022-09-27T01:20:20.000Z",
                "visit_keys":{"apps":[],"tenants":[t1_tenant_id],"roles":[]}
            }),
        )
        .await;
    let req_a_inst_id: String = flow_client
        .post(
            "/ci/inst",
            &FlowInstStartReq {
                tag: "REQ".to_string(),
                rel_business_obj_id: req_a_obj_id.clone(),
                ..Default::default()
            },
        )
        .await;
    let req_b_obj_id = TardisFuns::field.nanoid();
    let _: Void = search_client
        .put(
            "/ci/item",
            &json!({
                "tag":"idp_project",
                "kind": "idp_feed_req",
                "key": req_b_obj_id,
                "title": "需求B",
                "content": "需求B",
                "owner":ctx.owner,
                "own_paths":ctx.own_paths,
                "create_time":"2022-09-26T23:23:59.000Z",
                "update_time": "2022-09-27T01:20:20.000Z",
                "visit_keys":{"apps":[],"tenants":[t1_tenant_id],"roles":[]}
            }),
        )
        .await;
    let req_b_inst_id: String = flow_client
        .post(
            "/ci/inst",
            &FlowInstStartReq {
                tag: "REQ".to_string(),
                rel_business_obj_id: req_b_obj_id.clone(),
                ..Default::default()
            },
        )
        .await;
    // 初始化评审规则
    let req_label = json!({
        "origin_status": init_state_id,
        "pass_status": processing_state_id,
        "unpass_status": init_state_id,
    })
    .to_string();
    let _: Void = kv_client
        .put(
            "/ci/item",
            &json!({
                "key": format!("__tag__:{}:{}:_:review_config", t1_tenant_id, t1_app_id),
                "value": json!(vec![
                    json!({
                        "code": "REQ",
                        "label": req_label
                    })
                ])
            }),
        )
        .await;
    // 初始化评审审批流
    let review_approval_flow: FlowModelAggResp = flow_client
        .post(
            "/cc/model",
            &FlowModelAddReq {
                id: None,
                kind: FlowModelKind::AsModel,
                status: FlowModelStatus::Enabled,
                rel_transition_ids: Some(vec!["__REVIEW__".to_string()]),
                add_version: None,
                current_version_id: None,
                name: "评审审批流".into(),
                info: Some("xxx".to_string()),
                rel_template_ids: None,
                template: false,
                main: false,
                tag: Some("REVIEW".to_string()),
                scope_level: None,
                icon: None,
                rel_model_id: None,
                disabled: None,
                front_conds: None,
                data_source: None,
                default: None,
            },
        )
        .await;
    let review_approval_flow_version: FlowModelVersionDetailResp = flow_client.get(&format!("/cc/model_version/{}", review_approval_flow.edit_version_id)).await;
    let start_review_state_id = review_approval_flow_version.states()[0].id.clone();
    let approval_review_state_a_id = TardisFuns::field.nanoid();
    let approval_review_state_b_id = TardisFuns::field.nanoid();
    let finish_review_state_id = review_approval_flow_version.states()[1].id.clone();
    let start_review_transition_id = review_approval_flow_version.states()[0].transitions[0].id.clone();
    let _: Void = flow_client
        .patch(
            &format!("/cc/model_version/{}", review_approval_flow.edit_version_id),
            &FlowModelVersionModifyReq {
                bind_states: Some(vec![FlowModelVersionBindState {
                    bind_new_state: Some(FlowModelBindNewStateReq {
                        new_state: FlowStateAddReq {
                            id: Some(approval_review_state_a_id.clone().into()),
                            name: Some("审批节点1".into()),
                            sys_state: FlowSysStateKind::Progress,
                            state_kind: Some(FlowStateKind::Approval),
                            tags: Some(vec![review_approval_flow.tag.clone()]),
                            main: Some(false),
                            kind_conf: Some(FLowStateKindConf {
                                form: None,
                                approval: Some(FlowStateApproval {
                                    countersign_conf: FlowStateCountersignConf {
                                        kind: FlowStateCountersignKind::All,
                                        ..Default::default()
                                    },
                                    revoke: true,
                                    multi_approval_kind: FlowStatusMultiApprovalKind::Orsign,
                                    pass_btn_name: "通过".to_string(),
                                    back_btn_name: "退回".to_string(),
                                    overrule_btn_name: "不通过".to_string(),
                                    guard_by_creator: true,
                                    guard_by_his_operators: false,
                                    guard_by_assigned: true,
                                    auto_transfer_when_empty_kind: None,
                                    referral: true,
                                    ..Default::default()
                                }),
                            }),
                            ..Default::default()
                        },
                        ext: FlowStateRelModelExt { sort: 0, show_btns: None, ..Default::default() },
                    }),
                    is_init: false,
                    add_transitions: Some(vec![FlowTransitionAddReq {
                        name: Some("提交".into()),
                        from_flow_state_id: approval_review_state_a_id.clone(),
                        to_flow_state_id: finish_review_state_id.clone(),
                        ..Default::default()
                    }]),
                    ..Default::default()
                }]),
                modify_states: Some(vec![FlowModelVersionModifyState {
                    id: Some(start_review_state_id.clone()),
                    modify_transitions: Some(vec![FlowTransitionModifyReq {
                        id: start_review_transition_id.clone().into(),
                        to_flow_state_id: Some(approval_review_state_a_id.clone()),
                        ..Default::default()
                    }]),
                    ..Default::default()
                }]),
                ..Default::default()
            },
        )
        .await;
    let review_approval_flow_version: FlowModelVersionDetailResp = flow_client.get(&format!("/cc/model_version/{}", review_approval_flow.edit_version_id)).await;
    let review_approve_state_a_transition_id =
        review_approval_flow_version.states().clone().into_iter().find(|state| state.id == approval_review_state_a_id).unwrap().transitions.first().cloned().unwrap().id;
    let _: Void = flow_client
        .patch(
            &format!("/cc/model_version/{}", review_approval_flow.edit_version_id),
            &FlowModelVersionModifyReq {
                bind_states: Some(vec![FlowModelVersionBindState {
                    bind_new_state: Some(FlowModelBindNewStateReq {
                        new_state: FlowStateAddReq {
                            id: Some(approval_review_state_b_id.clone().into()),
                            name: Some("审批节点2".into()),
                            sys_state: FlowSysStateKind::Progress,
                            state_kind: Some(FlowStateKind::Approval),
                            tags: Some(vec![review_approval_flow.tag.clone()]),
                            main: Some(false),
                            kind_conf: Some(FLowStateKindConf {
                                form: None,
                                approval: Some(FlowStateApproval {
                                    countersign_conf: FlowStateCountersignConf {
                                        kind: FlowStateCountersignKind::All,
                                        ..Default::default()
                                    },
                                    revoke: true,
                                    multi_approval_kind: FlowStatusMultiApprovalKind::Orsign,
                                    pass_btn_name: "通过".to_string(),
                                    back_btn_name: "退回".to_string(),
                                    overrule_btn_name: "不通过".to_string(),
                                    guard_by_creator: false,
                                    guard_by_his_operators: false,
                                    guard_by_assigned: true,
                                    auto_transfer_when_empty_kind: None,
                                    referral: true,
                                    ..Default::default()
                                }),
                            }),
                            ..Default::default()
                        },
                        ext: FlowStateRelModelExt { sort: 1, show_btns: None, ..Default::default() },
                    }),
                    is_init: false,
                    add_transitions: Some(vec![FlowTransitionAddReq {
                        name: Some("提交".into()),
                        from_flow_state_id: approval_review_state_b_id.clone(),
                        to_flow_state_id: finish_review_state_id.clone(),
                        ..Default::default()
                    }]),
                    ..Default::default()
                }]),
                modify_states: Some(vec![FlowModelVersionModifyState {
                    id: Some(approval_review_state_a_id.clone()),
                    modify_transitions: Some(vec![FlowTransitionModifyReq {
                        id: review_approve_state_a_transition_id.clone().into(),
                        to_flow_state_id: Some(approval_review_state_b_id.clone()),
                        ..Default::default()
                    }]),
                    ..Default::default()
                }]),
                ..Default::default()
            },
        )
        .await;
    let review_approval_version: FlowModelVersionDetailResp = flow_client.get(&format!("/cc/model_version/{}", review_approval_flow.edit_version_id)).await;
    // 启用评审审批流
    let _: Void = flow_client
        .patch(
            &format!("/cc/model_version/{}", review_approval_flow.edit_version_id),
            &FlowModelVersionModifyReq {
                status: Some(FlowModelVesionState::Enabled),
                ..Default::default()
            },
        )
        .await;
    // 新建评审
    let review_obj_id = TardisFuns::field.nanoid();
    let review_inst_id: String = flow_client
        .post(
            "/ci/inst",
            &FlowInstStartReq {
                tag: "REVIEW".to_string(),
                rel_business_obj_id: review_obj_id.clone(),
                rel_transition_id: Some("__REVIEW__".to_string()),
                rel_child_objs: Some(vec![
                    FlowInstRelChildObj {
                        tag: "REQ".to_string(),
                        obj_id: req_a_obj_id.to_string(),
                    },
                    FlowInstRelChildObj {
                        tag: "REQ".to_string(),
                        obj_id: req_b_obj_id.to_string(),
                    },
                ]),
                operator_map: Some(HashMap::from([
                    (approval_review_state_a_id.to_string(), vec![t1_account_a_id.clone()]),
                    (approval_review_state_b_id.to_string(), vec![t1_account_b_id.clone()]),
                ])),
                ..Default::default()
            },
        )
        .await;
    // 发起评审
    let state_and_next_transitions: Vec<FlowInstFindStateAndTransitionsResp> = flow_client
        .put(
            "/cc/inst/batch/state_transitions",
            &vec![FlowInstFindStateAndTransitionsReq {
                flow_inst_id: review_inst_id.clone(),
                vars: None,
            }],
        )
        .await;
    let review_end_transition_id =
        state_and_next_transitions[0].next_flow_transitions.iter().find(|tran| tran.next_flow_transition_name == *"结束评审").unwrap().next_flow_transition_id.clone();
    let resp: FlowInstTransferResp = flow_client
        .put(
            &format!("/cc/inst/{}/transition/transfer", review_inst_id),
            &FlowInstTransferReq {
                flow_transition_id: review_end_transition_id,
                message: None,
                vars: None,
            },
        )
        .await;
    // 开始评审
    let _: Void = flow_client
        .post(
            &format!("/ci/inst/{}/batch_operate", review_inst_id),
            &HashMap::from([
                (
                    req_a_obj_id.to_string(),
                    FlowInstOperateReq {
                        operate: FlowStateOperatorKind::Pass,
                        vars: None,
                        all_vars: None,
                        output_message: None,
                        operator: None,
                        log_text: None,
                    },
                ),
                (
                    req_b_obj_id.to_string(),
                    FlowInstOperateReq {
                        operate: FlowStateOperatorKind::Overrule,
                        vars: None,
                        all_vars: None,
                        output_message: None,
                        operator: None,
                        log_text: None,
                    },
                ),
            ]),
        )
        .await;
    ctx.owner = t1_account_b_id.clone();
    ctx.own_paths = format!("{}/{}", t1_tenant_id, t1_app_id).to_string();
    flow_client.set_auth(&ctx)?;
    kv_client.set_auth(&ctx)?;
    search_client.set_auth(&ctx)?;
    let _: Void = flow_client
        .post(
            &format!("/ci/inst/{}/batch_operate", review_inst_id),
            &HashMap::from([(
                req_a_obj_id.to_string(),
                FlowInstOperateReq {
                    operate: FlowStateOperatorKind::Pass,
                    vars: None,
                    all_vars: None,
                    output_message: None,
                    operator: None,
                    log_text: None,
                },
            )]),
        )
        .await;
    // 评审自动结束
    Ok(())
}

struct IamData {
    tenant_id: String,
    accounts: Vec<String>,
    app_ids: Vec<String>,
}

async fn load_iam_data(search_client: &mut TestHttpClient, iam_client: &mut BIOSWebTestClient, sysadmin_name: String, sysadmin_password: String) -> TardisResult<Vec<IamData>> {
    // 1. create iam rbum data
    iam_client.login(&sysadmin_name, &sysadmin_password, None, None, None, true).await?;

    // Add Tenant
    let t1_tenant_id: String = iam_client
        .post(
            "/cs/tenant",
            &IamTenantAggAddReq {
                name: "t1".into(),
                icon: None,
                contact_phone: None,
                note: None,
                admin_name: "测试管理员1".into(),
                admin_username: "admin1".into(),
                admin_password: Some("123456".into()),
                admin_phone: None,
                admin_mail: None,
                audit_username: "audit1".into(),
                audit_name: "审计管理员1".into(),
                audit_password: None,
                audit_phone: None,
                audit_mail: None,
                disabled: None,
                account_self_reg: None,
                cert_conf_by_oauth2: None,
                cert_conf_by_ldap: None,
            },
        )
        .await;
    sleep(Duration::from_secs(1)).await;
    // Add Account
    iam_client.login("admin1", "123456", Some(t1_tenant_id.clone()), None, None, true).await?;
    let t1_account_id1: String = iam_client
        .post(
            "/ct/account",
            &IamAccountAggAddReq {
                id: None,
                name: "u001".into(),
                cert_user_name: "user_dp1".into(),
                cert_password: Some("123456".into()),
                cert_phone: None,
                cert_mail: Some("devopsxxx1@xx.com".into()),
                role_ids: None,
                org_node_ids: None,
                scope_level: Some(RBUM_SCOPE_LEVEL_TENANT),
                disabled: None,
                icon: None,
                exts: HashMap::from([("ext1_idx".to_string(), "00002".to_string())]),
                status: None,
                temporary: None,
                lock_status: None,
                logout_type: None,
                labor_type: None,
            },
        )
        .await;
    let _: Void = search_client
        .put(
            "/ci/item",
            &json!({
                "tag":"iam_account",
                "kind": "iam_account",
                "key": t1_account_id1,
                "title": "u001",
                "content": format!("u001,{:?}", vec!["user_dp1", "devopsxxx1@xx.com"],),
                "owner":"u001",
                "own_paths":t1_tenant_id,
                "create_time":"2022-09-26T23:23:59.000Z",
                "update_time": "2022-09-27T01:20:20.000Z",
                "visit_keys":{"apps":[],"tenants":[t1_tenant_id],"roles":[]}
            }),
        )
        .await;
    let t1_account_id2: String = iam_client
        .post(
            "/ct/account",
            &IamAccountAggAddReq {
                id: Some("u002_2".into()),
                name: "u002_2".into(),
                cert_user_name: "user_dp2_2".into(),
                cert_password: Some("123456".into()),
                cert_phone: None,
                cert_mail: Some("devopsxxx22@xx.com".into()),
                role_ids: None,
                org_node_ids: None,
                scope_level: Some(RBUM_SCOPE_LEVEL_TENANT),
                disabled: None,
                icon: None,
                exts: HashMap::new(),
                status: None,
                temporary: None,
                lock_status: None,
                logout_type: None,
                labor_type: None,
            },
        )
        .await;
    let _: Void = search_client
        .put(
            "/ci/item",
            &json!({
                "tag":"iam_account",
                "kind": "iam_account",
                "key": t1_account_id2,
                "title": "u002_2",
                "content": format!("u002_2,{:?}", vec!["user_dp2_2", "devopsxxx22@xx.com"],),
                "owner":"u002_2",
                "own_paths":t1_tenant_id,
                "create_time":"2022-09-26T23:23:59.000Z",
                "update_time": "2022-09-27T01:20:20.000Z",
                "visit_keys":{"apps":[],"tenants":[t1_tenant_id],"roles":[]}
            }),
        )
        .await;
    // Add App
    iam_client.login("admin1", "123456", Some(t1_tenant_id.clone()), None, None, true).await?;
    let app1_id: String = iam_client
        .post(
            "/ct/app",
            &IamAppAggAddReq {
                app_name: "app01".into(),
                app_description: None,
                app_icon: None,
                app_sort: None,
                app_contact_phone: None,
                admin_ids: Some(vec![t1_account_id1.clone(), t1_account_id2.clone()]),
                disabled: None,
                set_cate_id: None,
                kind: None,
                sync_apps_group: None,
            },
        )
        .await;
    let mut result = vec![];
    let t1_data = IamData {
        tenant_id: t1_tenant_id,
        accounts: vec![t1_account_id1, t1_account_id2],
        app_ids: vec![app1_id],
    };
    result.push(t1_data);
    Ok(result)
}
