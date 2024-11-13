use std::collections::HashMap;

use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;
use bios_basic::test::test_http_client::TestHttpClient;

use bios_mw_flow::dto::flow_config_dto::FlowConfigModifyReq;

use bios_mw_flow::dto::flow_inst_dto::{FlowInstDetailResp, FlowInstFindStateAndTransitionsReq, FlowInstFindStateAndTransitionsResp, FlowInstStartReq};
use bios_mw_flow::dto::flow_model_dto::{
    FlowModelAddReq, FlowModelAggResp, FlowModelAssociativeOperationKind, FlowModelBindNewStateReq, FlowModelBindStateReq, FlowModelCopyOrReferenceCiReq,
    FlowModelCopyOrReferenceReq, FlowModelKind, FlowModelModifyReq, FlowModelStatus, FlowModelSummaryResp,
};
use bios_mw_flow::dto::flow_model_version_dto::{
    FlowModelVersionAddReq, FlowModelVersionBindState, FlowModelVersionDetailResp, FlowModelVersionModifyReq, FlowModelVersionModifyState, FlowModelVesionState,
};
use bios_mw_flow::dto::flow_state_dto::{
    FlowStateAddReq, FlowStateKind, FlowStateModifyReq, FlowStateRelModelExt, FlowStateRelModelModifyReq, FlowStateSummaryResp, FlowSysStateKind,
};

use bios_mw_flow::dto::flow_transition_dto::{FlowTransitionAddReq, FlowTransitionModifyReq};
use bios_sdk_invoke::clients::spi_kv_client::KvItemSummaryResp;
use bios_spi_search::dto::search_item_dto::{
    SearchItemQueryReq, SearchItemSearchCtxReq, SearchItemSearchPageReq, SearchItemSearchQScopeKind, SearchItemSearchReq, SearchItemSearchResp,
};
use serde_json::json;
use tardis::basic::dto::TardisContext;

use std::time::Duration;
use tardis::basic::result::TardisResult;
use tardis::log::{debug, info};
use tardis::tokio::time::sleep;
use tardis::web::web_resp::{TardisPage, Void};
use tardis::TardisFuns;

pub async fn test(flow_client: &mut TestHttpClient, search_client: &mut TestHttpClient) -> TardisResult<()> {
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
    search_client.set_auth(&ctx)?;

    // 1. enter platform
    // 1-1. check default model
    let mut models: TardisPage<FlowModelSummaryResp> = flow_client.get("/cc/model?tag=REQ&page_number=1&page_size=100").await;
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
            value: "https://127.0.0.1:8080/mock/mock/exchange_data".to_string(),
        });
    }
    let _: Void = flow_client.post("/cs/config", &modify_configs).await;
    let configs: Option<TardisPage<KvItemSummaryResp>> = flow_client.get("/cs/config").await;
    info!("configs_new: {:?}", configs);
    // 2. enter tenant
    ctx.owner = "u001".to_string();
    ctx.own_paths = "t1".to_string();
    flow_client.set_auth(&ctx)?;
    search_client.set_auth(&ctx)?;
    // 2-1. Get states list
    let req_states: TardisPage<FlowStateSummaryResp> = flow_client.get("/cc/state?tag=REQ&enabled=true&page_number=1&page_size=100").await;
    let init_state_id = req_states.records[0].id.clone(); // 待开始
    let processing_state_id = req_states.records[1].id.clone(); // 进行中
    let finish_state_id = req_states.records[2].id.clone(); // 已完成
    let closed_state_id = req_states.records[3].id.clone(); // 已关闭
                                                            // 2-2. creat flow template
    let req_template_id1 = "template_req_1";
    let req_template_id2 = "template_req_2";
    let project_template_id1 = "template_project_1";
    let project_template_id2 = "template_project_2";
    let req_model_template_aggs: FlowModelAggResp = flow_client
        .post(
            "/cc/model",
            &FlowModelAddReq {
                kind: FlowModelKind::AsTemplate,
                status: FlowModelStatus::Enabled,
                rel_transition_ids: None,
                add_version: Some(FlowModelVersionAddReq {
                    name: "测试需求模板1".into(),
                    rel_model_id: None,
                    bind_states: None,
                    status: FlowModelVesionState::Enabled,
                    scope_level: Some(RbumScopeLevelKind::Private),
                    disabled: None,
                }),
                current_version_id: None,
                name: "测试需求模板1".into(),
                info: Some("xxx".to_string()),
                rel_template_ids: Some(vec![req_template_id1.to_string(), req_template_id2.to_string()]),
                template: true,
                main: true,
                tag: Some("REQ".to_string()),
                scope_level: Some(RbumScopeLevelKind::Private),
                icon: None,
                rel_model_id: None,
                disabled: None,
            },
        )
        .await;
    let req_model_template_id = req_model_template_aggs.id.clone();
    let _: Void = flow_client
        .patch(
            &format!("/cc/model/{}", req_model_template_id.clone()),
            &FlowModelModifyReq {
                modify_version: Some(FlowModelVersionModifyReq {
                    bind_states: Some(vec![
                        FlowModelVersionBindState {
                            exist_state: Some(FlowModelBindStateReq {
                                state_id: init_state_id.clone(),
                                ext: FlowStateRelModelExt { sort: 1, show_btns: None },
                            }),
                            add_transitions: Some(vec![
                                FlowTransitionAddReq {
                                    from_flow_state_id: init_state_id.clone(),
                                    to_flow_state_id: processing_state_id.clone(),
                                    name: Some("开始".into()),
                                    ..Default::default()
                                },
                                FlowTransitionAddReq {
                                    from_flow_state_id: init_state_id.clone(),
                                    to_flow_state_id: closed_state_id.clone(),
                                    name: Some("关闭".into()),
                                    ..Default::default()
                                },
                            ]),
                            is_init: true,
                            ..Default::default()
                        },
                        FlowModelVersionBindState {
                            exist_state: Some(FlowModelBindStateReq {
                                state_id: processing_state_id.clone(),
                                ext: FlowStateRelModelExt { sort: 2, show_btns: None },
                            }),
                            add_transitions: Some(vec![
                                FlowTransitionAddReq {
                                    from_flow_state_id: processing_state_id.clone(),
                                    to_flow_state_id: finish_state_id.clone(),
                                    name: Some("完成".into()),
                                    ..Default::default()
                                },
                                FlowTransitionAddReq {
                                    from_flow_state_id: processing_state_id.clone(),
                                    to_flow_state_id: closed_state_id.clone(),
                                    name: Some("关闭".into()),
                                    ..Default::default()
                                },
                            ]),
                            ..Default::default()
                        },
                        FlowModelVersionBindState {
                            exist_state: Some(FlowModelBindStateReq {
                                state_id: finish_state_id.clone(),
                                ext: FlowStateRelModelExt { sort: 3, show_btns: None },
                            }),
                            add_transitions: Some(vec![
                                FlowTransitionAddReq {
                                    from_flow_state_id: finish_state_id.clone(),
                                    to_flow_state_id: processing_state_id.clone(),
                                    name: Some("重新处理".into()),
                                    ..Default::default()
                                },
                                FlowTransitionAddReq {
                                    from_flow_state_id: finish_state_id.clone(),
                                    to_flow_state_id: closed_state_id.clone(),
                                    name: Some("关闭".into()),
                                    ..Default::default()
                                },
                            ]),
                            ..Default::default()
                        },
                        FlowModelVersionBindState {
                            exist_state: Some(FlowModelBindStateReq {
                                state_id: closed_state_id.clone(),
                                ext: FlowStateRelModelExt { sort: 4, show_btns: None },
                            }),
                            add_transitions: Some(vec![FlowTransitionAddReq {
                                from_flow_state_id: closed_state_id.clone(),
                                to_flow_state_id: init_state_id.clone(),
                                name: Some("激活".into()),
                                ..Default::default()
                            }]),
                            ..Default::default()
                        },
                    ]),
                    init_state_id: Some(init_state_id.to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .await;
    let req_default_model_template_aggs: FlowModelAggResp = flow_client
        .post(
            "/cc/model",
            &FlowModelAddReq {
                name: "测试需求默认模板1".into(),
                info: Some("xxx".to_string()),
                kind: FlowModelKind::AsTemplate,
                status: FlowModelStatus::Enabled,
                rel_transition_ids: None,
                add_version: None,
                current_version_id: None,
                rel_template_ids: None,
                template: true,
                main: true,
                tag: Some("REQ".to_string()),
                scope_level: Some(RbumScopeLevelKind::Private),
                icon: None,
                rel_model_id: None,
                disabled: None,
            },
        )
        .await;
    let req_default_model_template_id = req_default_model_template_aggs.id.clone();
    let _: Void = flow_client
        .patch(
            &format!("/cc/model/{}", req_default_model_template_id.clone()),
            &FlowModelModifyReq {
                modify_version: Some(FlowModelVersionModifyReq {
                    init_state_id: Some(init_state_id.to_string()),
                    bind_states: Some(vec![FlowModelVersionBindState {
                        exist_state: Some(FlowModelBindStateReq {
                            state_id: init_state_id.clone(),
                            ext: FlowStateRelModelExt { sort: 1, show_btns: None },
                        }),
                        is_init: true,
                        ..Default::default()
                    }]),
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .await;
    let req_model_uninit_template_aggs: FlowModelAggResp = flow_client
        .post(
            "/cc/model",
            &FlowModelAddReq {
                name: "测试需求未初始化模板1".into(),
                info: Some("xxx".to_string()),
                rel_template_ids: Some(vec![req_template_id1.to_string(), req_template_id2.to_string()]),
                template: true,
                main: true,
                tag: Some("REQ".to_string()),
                scope_level: Some(RbumScopeLevelKind::Private),
                icon: None,
                rel_model_id: None,
                disabled: None,
                kind: FlowModelKind::AsTemplate,
                status: FlowModelStatus::Enabled,
                rel_transition_ids: None,
                add_version: None,
                current_version_id: None,
            },
        )
        .await;
    let req_model_uninit_template_id = req_model_uninit_template_aggs.id.clone();
    sleep(Duration::from_millis(1000)).await;
    let model_templates: TardisPage<SearchItemSearchResp> = search_client
        .put(
            "/ci/item/search",
            &SearchItemSearchReq {
                tag: "flow_model".to_string(),
                ctx: SearchItemSearchCtxReq {
                    tenants: Some(vec![ctx.own_paths.clone()]),
                    ..Default::default()
                },
                query: SearchItemQueryReq { ..Default::default() },
                adv_query: None,
                sort: None,
                page: SearchItemSearchPageReq {
                    number: 1,
                    size: 20,
                    fetch_total: true,
                },
            },
        )
        .await;
    // assert_eq!(model_templates.total_size, 3);
    // assert!(model_templates.records.iter().any(|record| record.key == req_default_model_template_id));
    // assert!(model_templates.records.iter().any(|record| record.key == req_model_uninit_template_id));
    // assert!(model_templates.records.iter().any(|record| record.key == req_model_template_id));
    // template bind model
    let mut rel_model_ids = HashMap::new();
    rel_model_ids.insert("REQ".to_string(), req_model_template_id.clone());
    let result: HashMap<String, FlowModelAggResp> = flow_client
        .post(
            "/ct/model/copy_or_reference_model",
            &FlowModelCopyOrReferenceReq {
                rel_model_ids,
                rel_template_id: Some(project_template_id1.to_string()),
                op: FlowModelAssociativeOperationKind::ReferenceOrCopy,
            },
        )
        .await;
    info!("result: {:?}", result);
    let _result: Void = flow_client
        .patch(
            &format!("/cc/model/{}", req_default_model_template_id),
            &FlowModelModifyReq {
                name: Some("测试需求默认模板11".into()),
                tag: Some("REQ".to_string()),
                scope_level: Some(RbumScopeLevelKind::Private),
                ..Default::default()
            },
        )
        .await;
    sleep(Duration::from_millis(500)).await;
    let model_templates: TardisPage<SearchItemSearchResp> = search_client
        .put(
            "/ci/item/search",
            &SearchItemSearchReq {
                tag: "flow_model".to_string(),
                ctx: SearchItemSearchCtxReq {
                    tenants: Some(vec![ctx.own_paths.clone()]),
                    ..Default::default()
                },
                query: SearchItemQueryReq { ..Default::default() },
                adv_query: None,
                sort: None,
                page: SearchItemSearchPageReq {
                    number: 1,
                    size: 20,
                    fetch_total: true,
                },
            },
        )
        .await;
    assert_eq!(
        model_templates.records.iter().find(|record| record.key == req_default_model_template_id).unwrap().title,
        "测试需求默认模板11".to_string()
    );
    // creat share model template
    let _: Void = flow_client
        .patch(
            &format!("/cc/model/{}", req_default_model_template_id.clone()),
            &FlowModelModifyReq {
                scope_level: Some(RbumScopeLevelKind::Root),
                ..Default::default()
            },
        )
        .await;
    // checkout tenant
    ctx.owner = "u001".to_string();
    ctx.own_paths = "t2".to_string();
    flow_client.set_auth(&ctx)?;
    search_client.set_auth(&ctx)?;
    sleep(Duration::from_millis(1000)).await;
    let model_templates: TardisPage<SearchItemSearchResp> = search_client
        .put(
            "/ci/item/search",
            &SearchItemSearchReq {
                tag: "flow_model".to_string(),
                ctx: SearchItemSearchCtxReq {
                    tenants: Some(vec![ctx.own_paths.clone(), "".to_string()]),
                    ..Default::default()
                },
                query: SearchItemQueryReq { ..Default::default() },
                adv_query: None,
                sort: None,
                page: SearchItemSearchPageReq {
                    number: 1,
                    size: 20,
                    fetch_total: true,
                },
            },
        )
        .await;
    assert_eq!(model_templates.total_size, 1);
    assert_eq!(model_templates.records[0].key, req_default_model_template_id);
    let copy_template_model: FlowModelAggResp = flow_client.patch(&format!("/cc/model/copy/{}", req_default_model_template_id.clone()), &json!({})).await;
    sleep(Duration::from_millis(1000)).await;
    let model_templates: TardisPage<SearchItemSearchResp> = search_client
        .put(
            "/ci/item/search",
            &SearchItemSearchReq {
                tag: "flow_model".to_string(),
                ctx: SearchItemSearchCtxReq {
                    tenants: Some(vec![ctx.own_paths.clone(), "".to_string()]),
                    ..Default::default()
                },
                query: SearchItemQueryReq { ..Default::default() },
                adv_query: None,
                sort: None,
                page: SearchItemSearchPageReq {
                    number: 1,
                    size: 20,
                    fetch_total: true,
                },
            },
        )
        .await;
    assert_eq!(model_templates.total_size, 2);
    assert!(model_templates.records.iter().any(|record| record.key == req_default_model_template_id));
    assert!(model_templates.records.iter().any(|record| record.key == copy_template_model.id));
    // project template bind flow model
    ctx.owner = "u001".to_string();
    ctx.own_paths = "t1".to_string();
    flow_client.set_auth(&ctx)?;
    search_client.set_auth(&ctx)?;
    //
    let req_models: Vec<FlowModelSummaryResp> = flow_client.get(&format!("/cc/model/find_by_rel_template_id?tag=REQ&template=true&rel_template_id={}", req_template_id1)).await;
    assert_eq!(req_models.len(), 4);
    assert!(req_models.iter().any(|model| model.id == req_default_model_template_id));
    assert!(req_models.iter().any(|model| model.id == req_model_template_id));
    assert!(req_models.iter().all(|model| model.id != req_model_uninit_template_id));

    let req_models: Vec<FlowModelSummaryResp> = flow_client.get("/cc/model/find_by_rel_template_id?tag=REQ&template=true").await;
    assert_eq!(req_models.len(), 3);
    assert!(req_models.iter().any(|model| model.id == req_default_model_template_id));
    assert!(req_models.iter().all(|model| model.id != req_model_template_id));
    ctx.owner = "u001".to_string();
    ctx.own_paths = "t2".to_string();
    flow_client.set_auth(&ctx)?;
    search_client.set_auth(&ctx)?;
    let req_models: Vec<FlowModelSummaryResp> = flow_client.get("/cc/model/find_by_rel_template_id?tag=REQ&template=true").await;
    assert_eq!(req_models.len(), 3);
    assert!(req_models.iter().any(|model| model.id == req_default_model_template_id));
    assert!(req_models.iter().all(|model| model.id != req_model_template_id));
    // enter app
    ctx.owner = "u001".to_string();
    ctx.own_paths = "t1/app01".to_string();
    flow_client.set_auth(&ctx)?;
    search_client.set_auth(&ctx)?;
    let result: HashMap<String, String> = flow_client
        .post(
            "/ci/model/copy_or_reference_model",
            &FlowModelCopyOrReferenceCiReq {
                rel_template_id: Some(project_template_id1.to_string()),
                op: FlowModelAssociativeOperationKind::Copy,
                update_states: None,
            },
        )
        .await;
    info!("result: {:?}", result);
    let models: HashMap<String, FlowModelSummaryResp> = flow_client.put("/cc/model/find_rel_models?tag_ids=REQ,PROJ,ITER,TICKET&is_shared=false", &json!("")).await;
    info!("models: {:?}", models);
    sleep(Duration::from_millis(1000)).await;
    let req_inst_id1: String = flow_client
        .post(
            "/cc/inst",
            &FlowInstStartReq {
                tag: "REQ".to_string(),
                create_vars: None,
                rel_business_obj_id: TardisFuns::field.nanoid(),
                transition_id: None,
            },
        )
        .await;
    info!("req_inst_id1: {:?}", req_inst_id1);
    let req_inst1: FlowInstDetailResp = flow_client.get(&format!("/cc/inst/{}", req_inst_id1)).await;
    info!("req_inst1: {:?}", req_inst1);
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

    // 新建审批流
    let req_approval_flow: FlowModelAggResp = flow_client
        .post(
            "/cc/model",
            &FlowModelAddReq {
                kind: FlowModelKind::AsModel,
                status: FlowModelStatus::Enabled,
                rel_transition_ids: Some(vec!["__EDIT__".to_string()]),
                add_version: None,
                current_version_id: None,
                name: "编辑需求审批流".into(),
                info: Some("xxx".to_string()),
                rel_template_ids: None,
                template: false,
                main: false,
                tag: Some("REQ".to_string()),
                scope_level: None,
                icon: None,
                rel_model_id: None,
                disabled: None,
            },
        )
        .await;
    let req_approval_flow_version: FlowModelVersionDetailResp = flow_client.get(&format!("/cc/model_version/{}", req_approval_flow.edit_version_id)).await;
    let start_state_id = req_approval_flow_version.states()[0].id.clone();
    let form_state_id = TardisFuns::field.nanoid();
    let finish_state_id = req_approval_flow_version.states()[1].id.clone();
    let start_transition_id = req_approval_flow_version.states()[0].transitions[0].id.clone();
    let _: Void = flow_client
        .patch(
            &format!("/cc/model_version/{}", req_approval_flow.edit_version_id),
            &FlowModelVersionModifyReq {
                bind_states: Some(vec![FlowModelVersionBindState {
                    bind_new_state: Some(FlowModelBindNewStateReq {
                        new_state: FlowStateAddReq {
                            id: Some(form_state_id.clone().into()),
                            name: Some("录入".into()),
                            sys_state: FlowSysStateKind::Progress,
                            state_kind: Some(FlowStateKind::Form),
                            tags: Some(vec![req_approval_flow.tag.clone()]),
                            ..Default::default()
                        },
                        ext: FlowStateRelModelExt { sort: 1, show_btns: None },
                    }),
                    add_transitions: Some(vec![FlowTransitionAddReq {
                        name: Some("提交".into()),
                        from_flow_state_id: form_state_id.clone(),
                        to_flow_state_id: finish_state_id.clone(),
                        ..Default::default()
                    }]),
                    ..Default::default()
                }]),
                modify_states: Some(vec![
                    FlowModelVersionModifyState {
                        id: start_state_id.clone(),
                        modify_transitions: Some(vec![FlowTransitionModifyReq {
                            id: start_transition_id.into(),
                            to_flow_state_id: Some(form_state_id.clone()),
                            ..Default::default()
                        }]),
                        ..Default::default()
                    },
                    FlowModelVersionModifyState {
                        id: finish_state_id.clone(),
                        modify_rel: Some(FlowStateRelModelModifyReq {
                            id: finish_state_id.clone(),
                            sort: Some(2),
                            show_btns: None,
                        }),
                        ..Default::default()
                    },
                ]),
                ..Default::default()
            },
        )
        .await;
    let req_approval_flow_version: FlowModelVersionDetailResp = flow_client.get(&format!("/cc/model_version/{}", req_approval_flow.edit_version_id)).await;
    info!(
        "req_approval_flow_version: {:?}",
        TardisFuns::json.obj_to_json(&req_approval_flow_version).unwrap().to_string()
    );
    let _: Void = flow_client
        .patch(
            &format!("/cc/model_version/{}", req_approval_flow.edit_version_id),
            &FlowModelVersionModifyReq {
                status: Some(FlowModelVesionState::Enabled),
                ..Default::default()
            },
        )
        .await;
    let _versions: TardisPage<FlowModelVersionDetailResp> = flow_client.get(&format!("/cc/model_version?rel_model_id={}&page_number=1&page_size=100", req_approval_flow.id)).await;
    let state_and_next_transitions: Vec<FlowInstFindStateAndTransitionsResp> = flow_client
        .put(
            "/cc/inst/batch/state_transitions",
            &vec![FlowInstFindStateAndTransitionsReq {
                flow_inst_id: req_inst_id1.clone(),
                vars: None,
            }],
        )
        .await;
    info!(
        "state_and_next_transitions: {:?}",
        TardisFuns::json.obj_to_json(&state_and_next_transitions).unwrap().to_string()
    );
    Ok(())
}
