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
    FlowInstDetailResp, FlowInstFindStateAndTransitionsReq, FlowInstFindStateAndTransitionsResp, FlowInstOperateReq, FlowInstRelChildObj, FlowInstStartReq,
};
use bios_mw_flow::dto::flow_model_dto::{
    FlowModelAddReq, FlowModelAggResp, FlowModelAssociativeOperationKind, FlowModelBindNewStateReq, FlowModelBindStateReq, FlowModelCopyOrReferenceCiReq,
    FlowModelCopyOrReferenceReq, FlowModelDetailResp, FlowModelKind, FlowModelModifyReq, FlowModelStatus, FlowModelSummaryResp, FlowModelUnbindStateReq,
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
    search_client.set_auth(&ctx)?;
    let iam_data = load_iam_data(search_client, iam_client, sysadmin_name, sysadmin_password).await?;
    let t1_data = &iam_data[0];
    let t2_data = &iam_data[1];
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
            value: "http://127.0.0.1:8080/mock/mock/exchange_data".to_string(),
        });
    }
    let _: Void = flow_client.post("/cs/config", &modify_configs).await;
    let configs: Option<TardisPage<KvItemSummaryResp>> = flow_client.get("/cs/config").await;
    info!("configs_new: {:?}", configs);
    // 2. enter tenant
    ctx.owner = t1_data.accounts.first().cloned().unwrap();
    ctx.own_paths = t1_data.tenant_id.clone();
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
                id: None,
                kind: FlowModelKind::AsTemplate,
                status: FlowModelStatus::Enabled,
                rel_transition_ids: None,
                add_version: Some(FlowModelVersionAddReq {
                    id: None,
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
                front_conds: None,
                data_source: None,
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
                id: None,
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
                front_conds: None,
                data_source: None,
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
                id: None,
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
                front_conds: None,
                data_source: None,
            },
        )
        .await;
    let req_model_uninit_template_id = req_model_uninit_template_aggs.id.clone();
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
                adv_by_or: None,
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
    ctx.owner = t2_data.accounts.first().cloned().unwrap();
    ctx.own_paths = t2_data.tenant_id.clone();
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
                adv_by_or: None,
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
                adv_by_or: None,
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
    ctx.owner = t1_data.accounts.first().cloned().unwrap();
    ctx.own_paths = t1_data.tenant_id.clone();
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
    ctx.owner = t2_data.accounts.first().cloned().unwrap();
    ctx.own_paths = t2_data.tenant_id.clone();
    flow_client.set_auth(&ctx)?;
    search_client.set_auth(&ctx)?;
    let req_models: Vec<FlowModelSummaryResp> = flow_client.get("/cc/model/find_by_rel_template_id?tag=REQ&template=true").await;
    assert_eq!(req_models.len(), 3);
    assert!(req_models.iter().any(|model| model.id == req_default_model_template_id));
    assert!(req_models.iter().all(|model| model.id != req_model_template_id));
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
                update_states: None,
            },
        )
        .await;
    let bind_tempalte_model = result.get(&req_model_template_id).unwrap();
    assert_eq!(result.keys().len(), 1);
    assert!(result.contains_key(&req_model_template_id));
    assert_ne!(bind_tempalte_model.id, req_model_template_id);
    assert_eq!(bind_tempalte_model.tag, "REQ".to_string());
    assert_eq!(bind_tempalte_model.rel_model_id, req_model_template_id);
    let model_templates: HashMap<String, FlowModelSummaryResp> = flow_client
        .put(
            &format!("/cc/model/find_rel_models?tag_ids=REQ,PROJ,ITER,TICKET&is_shared=false&temp_id={}", project_template_id1),
            &json!(""),
        )
        .await;
    assert_eq!(model_templates.keys().len(), 1);
    assert_eq!(model_templates.get("REQ").unwrap().id, bind_tempalte_model.id);
    // modify model template ,sync bind model
    ctx.owner = t1_data.accounts.first().cloned().unwrap();
    ctx.own_paths = t1_data.tenant_id.clone();
    flow_client.set_auth(&ctx)?;
    search_client.set_auth(&ctx)?;
    let _: Void = flow_client
        .patch(
            &format!("/cc/model/{}", req_model_template_id.clone()),
            &FlowModelModifyReq {
                modify_version: Some(FlowModelVersionModifyReq {
                    unbind_states: Some(vec![FlowModelUnbindStateReq {
                        state_id: finish_state_id.clone(),
                        new_state_id: None,
                    }]),
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .await;
    sleep(Duration::from_millis(1000)).await;
    ctx.owner = t2_data.accounts.first().cloned().unwrap();
    ctx.own_paths = t2_data.tenant_id.clone();
    flow_client.set_auth(&ctx)?;
    search_client.set_auth(&ctx)?;
    let model_templates: HashMap<String, FlowModelSummaryResp> = flow_client
        .put(
            &format!("/cc/model/find_rel_models?tag_ids=REQ,PROJ,ITER,TICKET&is_shared=false&temp_id={}", project_template_id1),
            &json!(""),
        )
        .await;
    let bind_tempalte_model = model_templates.get("REQ").unwrap();
    let states = TardisFuns::json.json_to_obj::<Vec<FLowStateIdAndName>>(bind_tempalte_model.states.clone())?;
    assert!(states.iter().all(|state| state.id != finish_state_id));
    ctx.owner = t1_data.accounts.first().cloned().unwrap();
    ctx.own_paths = t1_data.tenant_id.clone();
    flow_client.set_auth(&ctx)?;
    search_client.set_auth(&ctx)?;
    let _: Void = flow_client
        .patch(
            &format!("/cc/model/{}", req_model_template_id.clone()),
            &FlowModelModifyReq {
                modify_version: Some(FlowModelVersionModifyReq {
                    bind_states: Some(vec![FlowModelVersionBindState {
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
                    }]),
                    modify_states: Some(vec![FlowModelVersionModifyState {
                        id: Some(processing_state_id.clone()),
                        add_transitions: Some(vec![FlowTransitionAddReq {
                            from_flow_state_id: processing_state_id.clone(),
                            to_flow_state_id: finish_state_id.clone(),
                            name: Some("完成111".into()),
                            ..Default::default()
                        }]),
                        ..Default::default()
                    }]),
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .await;
    sleep(Duration::from_millis(1000)).await;
    ctx.owner = t2_data.accounts.first().cloned().unwrap();
    ctx.own_paths = t2_data.tenant_id.clone();
    flow_client.set_auth(&ctx)?;
    search_client.set_auth(&ctx)?;
    let model_detail: FlowModelAggResp = flow_client.get(&format!("/cc/model/{}", bind_tempalte_model.id.clone())).await;
    let finish_tran = model_detail
        .states
        .iter()
        .find(|state| state.id == processing_state_id)
        .unwrap()
        .transitions
        .iter()
        .find(|tran| tran.from_flow_state_id == processing_state_id && tran.to_flow_state_id == finish_state_id)
        .unwrap();
    assert!(model_detail.states.iter().any(|state| state.id == finish_state_id));
    assert_eq!(finish_tran.name, "完成111".to_string());
    ctx.owner = t1_data.accounts.first().cloned().unwrap();
    ctx.own_paths = t1_data.tenant_id.clone();
    flow_client.set_auth(&ctx)?;
    search_client.set_auth(&ctx)?;
    let parent_model: FlowModelAggResp = flow_client.get(&format!("/cc/model/{}", req_model_template_id.clone())).await;
    let finish_tran = parent_model
        .states
        .iter()
        .find(|state| state.id == processing_state_id)
        .unwrap()
        .transitions
        .iter()
        .find(|tran| tran.from_flow_state_id == processing_state_id && tran.to_flow_state_id == finish_state_id)
        .unwrap();
    let _: Void = flow_client
        .patch(
            &format!("/cc/model/{}", req_model_template_id.clone()),
            &FlowModelModifyReq {
                modify_version: Some(FlowModelVersionModifyReq {
                    modify_states: Some(vec![FlowModelVersionModifyState {
                        id: Some(processing_state_id.clone()),
                        modify_transitions: Some(vec![FlowTransitionModifyReq {
                            id: finish_tran.id.clone().into(),
                            name: Some("完成-modify".into()),
                            ..Default::default()
                        }]),
                        ..Default::default()
                    }]),
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .await;
    sleep(Duration::from_millis(1000)).await;
    ctx.owner = t2_data.accounts.first().cloned().unwrap();
    ctx.own_paths = t2_data.tenant_id.clone();
    flow_client.set_auth(&ctx)?;
    search_client.set_auth(&ctx)?;
    let model_detail: FlowModelAggResp = flow_client.get(&format!("/cc/model/{}", bind_tempalte_model.id.clone())).await;
    let finish_tran = model_detail
        .states
        .iter()
        .find(|state| state.id == processing_state_id)
        .unwrap()
        .transitions
        .iter()
        .find(|tran| tran.from_flow_state_id == processing_state_id && tran.to_flow_state_id == finish_state_id)
        .unwrap();
    assert!(model_detail.states.iter().any(|state| state.id == finish_state_id));
    assert_eq!(finish_tran.name, "完成-modify".to_string());

    // enter app
    ctx.owner = t2_data.accounts.first().cloned().unwrap();
    ctx.own_paths = format!("{}/{}", t2_data.tenant_id, t2_data.app_ids.first().cloned().unwrap_or_default()).to_string();
    flow_client.set_auth(&ctx)?;
    search_client.set_auth(&ctx)?;
    let result: HashMap<String, String> = flow_client
        .post(
            "/ci/model/copy_or_reference_model",
            &FlowModelCopyOrReferenceCiReq {
                rel_template_id: Some(project_template_id1.to_string()),
                op: FlowModelAssociativeOperationKind::Reference,
                update_states: None,
            },
        )
        .await;
    assert_eq!(bind_tempalte_model.id, result.get(&bind_tempalte_model.id).unwrap().clone());

    let models: HashMap<String, FlowModelSummaryResp> = flow_client.put("/cc/model/find_rel_models?tag_ids=REQ,PROJ,ITER,TICKET&is_shared=false", &json!("")).await;
    sleep(Duration::from_millis(1000)).await;
    let rel_business_obj_id = TardisFuns::field.nanoid();
    let req_inst_id1: String = flow_client
        .post(
            "/cc/inst",
            &FlowInstStartReq {
                tag: "REQ".to_string(),
                create_vars: None,
                rel_business_obj_id: rel_business_obj_id.clone(),
                transition_id: None,
                vars: None,
                check_vars: None,
                log_text: None,
                rel_transition_id: None,
                rel_child_objs: None,
                operator_map: None,
                ..Default::default()
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
    // 切换模板为复制
    let mut update_states = HashMap::new();
    let req_update_states = HashMap::from([(init_state_id.clone(), processing_state_id.clone())]);
    update_states.insert("REQ".to_string(), req_update_states);
    let result: HashMap<String, String> = flow_client
        .post(
            "/ci/model/copy_or_reference_model",
            &FlowModelCopyOrReferenceCiReq {
                rel_template_id: Some(project_template_id1.to_string()),
                op: FlowModelAssociativeOperationKind::Copy,
                update_states: Some(update_states),
            },
        )
        .await;
    assert_ne!(bind_tempalte_model.id, result.get(&bind_tempalte_model.id).unwrap().clone());
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
    assert_eq!(state_and_next_transitions[0].current_flow_state_name, "进行中");
    // 新建空审批流
    let req_delete_flow: FlowModelAggResp = flow_client
        .post(
            "/cc/model",
            &FlowModelAddReq {
                id: None,
                kind: FlowModelKind::AsModel,
                status: FlowModelStatus::Enabled,
                rel_transition_ids: Some(vec!["__DELETE__".to_string()]),
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
                front_conds: None,
                data_source: None,
            },
        )
        .await;
    let req_delete_flow_version: FlowModelVersionDetailResp = flow_client.get(&format!("/cc/model_version/{}", req_delete_flow.edit_version_id)).await;
    let _: Void = flow_client
        .patch(
            &format!("/cc/model_version/{}", req_delete_flow_version.id),
            &FlowModelVersionModifyReq {
                status: Some(FlowModelVesionState::Enabled),
                ..Default::default()
            },
        )
        .await;
    // 新建审批流
    let req_approval_flow: FlowModelAggResp = flow_client
        .post(
            "/cc/model",
            &FlowModelAddReq {
                id: None,
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
                front_conds: Some(vec![vec![BasicQueryCondInfo {
                    field: "name".to_string(),
                    op: BasicQueryOpKind::Like,
                    op_text: None,
                    value: json!("1111"),
                }]]),
                data_source: None,
            },
        )
        .await;
    let req_approval_flow_version: FlowModelVersionDetailResp = flow_client.get(&format!("/cc/model_version/{}", req_approval_flow.edit_version_id)).await;
    let start_state_id = req_approval_flow_version.states()[0].id.clone();
    let form_autoskip_state_id = TardisFuns::field.nanoid();
    let form_state_id = TardisFuns::field.nanoid();
    let approval_state_id = TardisFuns::field.nanoid();
    let finish_state_id = req_approval_flow_version.states()[1].id.clone();
    let start_transition_id = req_approval_flow_version.states()[0].transitions[0].id.clone();
    let _: Void = flow_client
        .patch(
            &format!("/cc/model_version/{}", req_approval_flow.edit_version_id),
            &FlowModelVersionModifyReq {
                bind_states: Some(vec![FlowModelVersionBindState {
                    bind_new_state: Some(FlowModelBindNewStateReq {
                        new_state: FlowStateAddReq {
                            id: Some(form_autoskip_state_id.clone().into()),
                            name: Some("录入节点".into()),
                            sys_state: FlowSysStateKind::Progress,
                            state_kind: Some(FlowStateKind::Form),
                            tags: Some(vec![req_approval_flow.tag.clone()]),
                            main: Some(false),
                            kind_conf: Some(FLowStateKindConf {
                                form: Some(FlowStateForm {
                                    submit_btn_name: "提交".to_string(),
                                    auto_transfer_when_empty_kind: Some(FlowStatusAutoStrategyKind::Autoskip),
                                    ..Default::default()
                                }),
                                approval: None,
                            }),
                            ..Default::default()
                        },
                        ext: FlowStateRelModelExt { sort: 1, show_btns: None },
                    }),
                    add_transitions: Some(vec![FlowTransitionAddReq {
                        name: Some("提交".into()),
                        from_flow_state_id: form_autoskip_state_id.clone(),
                        to_flow_state_id: finish_state_id.clone(),
                        ..Default::default()
                    }]),
                    ..Default::default()
                }]),
                modify_states: Some(vec![
                    FlowModelVersionModifyState {
                        id: Some(start_state_id.clone()),
                        modify_transitions: Some(vec![FlowTransitionModifyReq {
                            id: start_transition_id.clone().into(),
                            to_flow_state_id: Some(form_autoskip_state_id.clone()),
                            ..Default::default()
                        }]),
                        ..Default::default()
                    },
                    FlowModelVersionModifyState {
                        id: Some(finish_state_id.clone()),
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
    let form_autoskip_transition_id = &req_approval_flow_version.states().into_iter().find(|state| state.id == form_autoskip_state_id).unwrap().transitions[0].id;
    let _: Void = flow_client
        .patch(
            &format!("/cc/model_version/{}", req_approval_flow.edit_version_id),
            &FlowModelVersionModifyReq {
                bind_states: Some(vec![FlowModelVersionBindState {
                    bind_new_state: Some(FlowModelBindNewStateReq {
                        new_state: FlowStateAddReq {
                            id: Some(form_state_id.clone().into()),
                            name: Some("录入节点".into()),
                            sys_state: FlowSysStateKind::Progress,
                            state_kind: Some(FlowStateKind::Form),
                            tags: Some(vec![req_approval_flow.tag.clone()]),
                            main: Some(false),
                            kind_conf: Some(FLowStateKindConf {
                                form: Some(FlowStateForm {
                                    guard_by_creator: true,
                                    guard_by_assigned: true,
                                    submit_btn_name: "提交".to_string(),
                                    auto_transfer_when_empty_kind: Some(FlowStatusAutoStrategyKind::Autoskip),
                                    referral: true,
                                    ..Default::default()
                                }),
                                approval: None,
                            }),
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
                modify_states: Some(vec![FlowModelVersionModifyState {
                    id: Some(form_autoskip_state_id.clone()),
                    modify_transitions: Some(vec![FlowTransitionModifyReq {
                        id: form_autoskip_transition_id.to_string().into(),
                        to_flow_state_id: Some(form_state_id.clone()),
                        ..Default::default()
                    }]),
                    ..Default::default()
                }]),
                ..Default::default()
            },
        )
        .await;
    let req_approval_flow_version: FlowModelVersionDetailResp = flow_client.get(&format!("/cc/model_version/{}", req_approval_flow.edit_version_id)).await;
    let form_transition_id = &req_approval_flow_version.states().into_iter().find(|state| state.id == form_state_id).unwrap().transitions[0].id;
    let _: Void = flow_client
        .patch(
            &format!("/cc/model_version/{}", req_approval_flow.edit_version_id),
            &FlowModelVersionModifyReq {
                bind_states: Some(vec![FlowModelVersionBindState {
                    bind_new_state: Some(FlowModelBindNewStateReq {
                        new_state: FlowStateAddReq {
                            id: Some(approval_state_id.clone().into()),
                            name: Some("审批节点".into()),
                            sys_state: FlowSysStateKind::Progress,
                            state_kind: Some(FlowStateKind::Approval),
                            tags: Some(vec![req_approval_flow.tag.clone()]),
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
                                    guard_by_assigned: false,
                                    auto_transfer_when_empty_kind: Some(FlowStatusAutoStrategyKind::Autoskip),
                                    referral: true,
                                    ..Default::default()
                                }),
                            }),
                            ..Default::default()
                        },
                        ext: FlowStateRelModelExt { sort: 1, show_btns: None },
                    }),
                    is_init: false,
                    add_transitions: Some(vec![FlowTransitionAddReq {
                        name: Some("提交".into()),
                        from_flow_state_id: approval_state_id.clone(),
                        to_flow_state_id: finish_state_id.clone(),
                        ..Default::default()
                    }]),
                    ..Default::default()
                }]),
                modify_states: Some(vec![FlowModelVersionModifyState {
                    id: Some(form_state_id.clone()),
                    modify_transitions: Some(vec![FlowTransitionModifyReq {
                        id: form_transition_id.to_string().into(),
                        to_flow_state_id: Some(approval_state_id.clone()),
                        ..Default::default()
                    }]),
                    ..Default::default()
                }]),
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
            },
        )
        .await;
    let review_approval_flow_version: FlowModelVersionDetailResp = flow_client.get(&format!("/cc/model_version/{}", review_approval_flow.edit_version_id)).await;
    let start_review_state_id = review_approval_flow_version.states()[0].id.clone();
    let approval_review_state_id = TardisFuns::field.nanoid();
    let finish_review_state_id = review_approval_flow_version.states()[1].id.clone();
    let start_review_transition_id = review_approval_flow_version.states()[0].transitions[0].id.clone();
    let _: Void = flow_client
        .patch(
            &format!("/cc/model_version/{}", review_approval_flow.edit_version_id),
            &FlowModelVersionModifyReq {
                bind_states: Some(vec![FlowModelVersionBindState {
                    bind_new_state: Some(FlowModelBindNewStateReq {
                        new_state: FlowStateAddReq {
                            id: Some(approval_review_state_id.clone().into()),
                            name: Some("审批节点".into()),
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
                        ext: FlowStateRelModelExt { sort: 1, show_btns: None },
                    }),
                    is_init: false,
                    add_transitions: Some(vec![FlowTransitionAddReq {
                        name: Some("提交".into()),
                        from_flow_state_id: approval_review_state_id.clone(),
                        to_flow_state_id: finish_review_state_id.clone(),
                        ..Default::default()
                    }]),
                    ..Default::default()
                }]),
                modify_states: Some(vec![FlowModelVersionModifyState {
                    id: Some(start_review_state_id.clone()),
                    modify_transitions: Some(vec![FlowTransitionModifyReq {
                        id: start_review_transition_id.clone().into(),
                        to_flow_state_id: Some(approval_review_state_id.clone()),
                        ..Default::default()
                    }]),
                    ..Default::default()
                }]),
                ..Default::default()
            },
        )
        .await;
    let review_approval_version: FlowModelVersionDetailResp = flow_client.get(&format!("/cc/model_version/{}", review_approval_flow.edit_version_id)).await;
    info!("review_approval_version: {:?}", TardisFuns::json.obj_to_json(&review_approval_version).unwrap().to_string());
    let _: Void = flow_client
        .patch(
            &format!("/cc/model_version/{}", review_approval_flow.edit_version_id),
            &FlowModelVersionModifyReq {
                status: Some(FlowModelVesionState::Enabled),
                ..Default::default()
            },
        )
        .await;
    // 尝试启动空配置的实例
    let resp_error: TardisResp<String> = flow_client
        .post_resp(
            "/cc/inst",
            &FlowInstStartReq {
                tag: "REQ".to_string(),
                create_vars: None,
                rel_business_obj_id: rel_business_obj_id.clone(),
                transition_id: Some("__DELETE__".to_string()),
                vars: None,
                check_vars: None,
                log_text: None,
                rel_transition_id: None,
                rel_child_objs: None,
                operator_map: None,
                ..Default::default()
            },
        )
        .await;
    assert_eq!(resp_error.code, *"500-flow-flow_inst-start_secondary_flow");
    // 尝试启动不符合条件的审批流
    let empty_inst_id: String = flow_client
        .post(
            "/cc/inst/try",
            &FlowInstStartReq {
                tag: "REQ".to_string(),
                create_vars: None,
                rel_business_obj_id: rel_business_obj_id.clone(),
                transition_id: Some("__EDIT__".to_string()),
                vars: None,
                check_vars: Some(HashMap::from([("name".to_string(), json!("xxx111"))])),
                log_text: None,
                rel_transition_id: None,
                rel_child_objs: None,
                operator_map: None,
                ..Default::default()
            },
        )
        .await;
    assert_eq!(empty_inst_id, "".to_string());
    // 尝试启动审批流实例
    let req_inst_id2: String = flow_client
        .post(
            "/cc/inst/try",
            &FlowInstStartReq {
                tag: "REQ".to_string(),
                create_vars: None,
                rel_business_obj_id: rel_business_obj_id.clone(),
                transition_id: Some("__EDIT__".to_string()),
                vars: None,
                check_vars: Some(HashMap::from([("name".to_string(), json!("xxx1111"))])),
                log_text: None,
                rel_transition_id: None,
                rel_child_objs: None,
                operator_map: None,
                ..Default::default()
            },
        )
        .await;
    sleep(Duration::from_millis(5000)).await;
    let req_inst2: FlowInstDetailResp = flow_client.get(&format!("/cc/inst/{}", req_inst_id2)).await;
    assert_eq!(req_inst2.current_state_id, form_state_id);
    assert_eq!(req_inst2.rel_flow_version_id, req_approval_flow.edit_version_id);
    assert_eq!(req_inst2.current_state_conf.clone().unwrap().operators.len(), 2);
    assert!(req_inst2.current_state_conf.unwrap().operators.contains_key(&FlowStateOperatorKind::Referral));
    // 操作转审
    let operator = &t2_data.accounts[1];
    let _: Void = flow_client
        .post(
            &format!("/cc/inst/{}/operate", req_inst_id2),
            &FlowInstOperateReq {
                operate: FlowStateOperatorKind::Referral,
                operator: Some(operator.clone()),
                vars: None,
                all_vars: None,
                output_message: None,
                log_text: None,
            },
        )
        .await;
    let req_inst2: FlowInstDetailResp = flow_client.get(&format!("/cc/inst/{}", req_inst_id2)).await;
    assert_eq!(req_inst2.current_state_id, form_state_id);
    assert_eq!(req_inst2.current_state_conf.clone().unwrap().operators.len(), 0);
    // 切换转审操作用户
    ctx.owner = operator.clone();
    flow_client.set_auth(&ctx)?;
    search_client.set_auth(&ctx)?;
    let req_inst2: FlowInstDetailResp = flow_client.get(&format!("/cc/inst/{}", req_inst_id2)).await;
    assert_eq!(req_inst2.current_state_conf.clone().unwrap().operators.len(), 2);
    assert!(req_inst2.current_state_conf.clone().unwrap().operators.contains_key(&FlowStateOperatorKind::Referral));
    assert!(req_inst2.current_state_conf.clone().unwrap().operators.contains_key(&FlowStateOperatorKind::Submit));
    let _: Void = flow_client
        .post(
            &format!("/cc/inst/{}/operate", req_inst_id2),
            &FlowInstOperateReq {
                operate: FlowStateOperatorKind::Submit,
                operator: None,
                vars: None,
                all_vars: None,
                output_message: None,
                log_text: None,
            },
        )
        .await;
    let req_inst2: FlowInstDetailResp = flow_client.get(&format!("/cc/inst/{}", req_inst_id2)).await;
    assert_eq!(req_inst2.current_state_conf.clone().unwrap().operators.len(), 1);
    assert!(req_inst2.current_state_conf.clone().unwrap().operators.contains_key(&FlowStateOperatorKind::Revoke));
    // 切回创建人操作
    ctx.owner = t2_data.accounts[0].clone();
    flow_client.set_auth(&ctx)?;
    search_client.set_auth(&ctx)?;
    let req_inst2: FlowInstDetailResp = flow_client.get(&format!("/cc/inst/{}", req_inst_id2)).await;
    assert_eq!(req_inst2.current_state_conf.clone().unwrap().operators.len(), 4);
    assert!(req_inst2.current_state_conf.clone().unwrap().operators.contains_key(&FlowStateOperatorKind::Referral));
    assert!(req_inst2.current_state_conf.clone().unwrap().operators.contains_key(&FlowStateOperatorKind::Back));
    assert!(req_inst2.current_state_conf.clone().unwrap().operators.contains_key(&FlowStateOperatorKind::Pass));
    assert!(req_inst2.current_state_conf.clone().unwrap().operators.contains_key(&FlowStateOperatorKind::Overrule));
    let _: Void = flow_client
        .post(
            &format!("/cc/inst/{}/operate", req_inst_id2),
            &FlowInstOperateReq {
                operate: FlowStateOperatorKind::Pass,
                operator: None,
                vars: None,
                all_vars: None,
                output_message: None,
                log_text: None,
            },
        )
        .await;
    let req_inst2: FlowInstDetailResp = flow_client.get(&format!("/cc/inst/{}", req_inst_id2)).await;
    assert_eq!(req_inst2.current_state_id, finish_state_id);
    assert!(req_inst2.current_state_conf.is_none());
    // 创建携带子工作流的实例
    let review_obj_id = TardisFuns::field.nanoid();
    let child_obj_id = TardisFuns::field.nanoid();
    let resp: TardisResp<String> = flow_client
        .post_resp(
            "/cc/inst",
            &FlowInstStartReq {
                tag: "REVIEW".to_string(),
                create_vars: None,
                rel_business_obj_id: review_obj_id.clone(),
                transition_id: None,
                vars: None,
                check_vars: None,
                log_text: None,
                rel_transition_id: Some("__REVIEW__".to_string()),
                rel_child_objs: Some(vec![FlowInstRelChildObj {
                    tag: "REQ".to_string(),
                    obj_id: child_obj_id.clone(),
                }]),
                operator_map: Some(HashMap::from([(approval_review_state_id.clone(), vec![t1_data.accounts[0].clone()])])),
                ..Default::default()
            },
        )
        .await;
    assert_eq!(resp.code, "200".to_string());
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
    let t2_tenant_id: String = iam_client
        .post(
            "/cs/tenant",
            &IamTenantAggAddReq {
                name: "t2".into(),
                icon: None,
                contact_phone: None,
                note: None,
                admin_name: "测试管理员2".into(),
                admin_username: "admin2".into(),
                admin_password: Some("123456".into()),
                admin_phone: None,
                admin_mail: None,
                audit_username: "audit2".into(),
                audit_name: "审计管理员2".into(),
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
    let t1_account_id: String = iam_client
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
                "key": t1_account_id,
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
    iam_client.login("admin2", "123456", Some(t2_tenant_id.clone()), None, None, true).await?;
    let t2_account_id1: String = iam_client
        .post(
            "/ct/account",
            &IamAccountAggAddReq {
                id: None,
                name: "u002_1".into(),
                cert_user_name: "user_dp2_1".into(),
                cert_password: Some("123456".into()),
                cert_phone: None,
                cert_mail: Some("devopsxxx21@xx.com".into()),
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
                "key": t2_account_id1,
                "title": "u002_1",
                "content": format!("u002_1,{:?}", vec!["user_dp2_1", "devopsxxx21@xx.com"],),
                "owner":"u002_1",
                "own_paths":t2_tenant_id,
                "create_time":"2022-09-26T23:23:59.000Z",
                "update_time": "2022-09-27T01:20:20.000Z",
                "visit_keys":{"apps":[],"tenants":[t2_tenant_id],"roles":[]}
            }),
        )
        .await;
    let t2_account_id2: String = iam_client
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
                "key": t2_account_id2,
                "title": "u002_2",
                "content": format!("u002_2,{:?}", vec!["user_dp2_2", "devopsxxx22@xx.com"],),
                "owner":"u002_2",
                "own_paths":t2_tenant_id,
                "create_time":"2022-09-26T23:23:59.000Z",
                "update_time": "2022-09-27T01:20:20.000Z",
                "visit_keys":{"apps":[],"tenants":[t2_tenant_id],"roles":[]}
            }),
        )
        .await;
    let t2_account_id3: String = iam_client
        .post(
            "/ct/account",
            &IamAccountAggAddReq {
                id: None,
                name: "u002_3".into(),
                cert_user_name: "user_dp2_3".into(),
                cert_password: Some("123456".into()),
                cert_phone: None,
                cert_mail: Some("devopsxxx23@xx.com".into()),
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
                "key": t2_account_id3,
                "title": "u002_3",
                "content": format!("u002_3,{:?}", vec!["user_dp2_3", "devopsxxx23@xx.com"],),
                "owner":"u002_3",
                "own_paths":t2_tenant_id,
                "create_time":"2022-09-26T23:23:59.000Z",
                "update_time": "2022-09-27T01:20:20.000Z",
                "visit_keys":{"apps":[],"tenants":[t2_tenant_id],"roles":[]}
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
                admin_ids: Some(vec![t1_account_id.clone()]),
                disabled: None,
                set_cate_id: None,
                kind: None,
                sync_apps_group: None,
            },
        )
        .await;
    iam_client.login("admin2", "123456", Some(t2_tenant_id.clone()), None, None, true).await?;
    let app2_id: String = iam_client
        .post(
            "/ct/app",
            &IamAppAggAddReq {
                app_name: "app02".into(),
                app_description: None,
                app_icon: None,
                app_sort: None,
                app_contact_phone: None,
                admin_ids: Some(vec![t2_account_id1.clone(), t2_account_id2.clone(), t2_account_id3.clone()]),
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
        accounts: vec![t1_account_id],
        app_ids: vec![app1_id],
    };
    let t2_data = IamData {
        tenant_id: t2_tenant_id,
        accounts: vec![t2_account_id1, t2_account_id2, t2_account_id3],
        app_ids: vec![app2_id],
    };
    result.push(t1_data);
    result.push(t2_data);
    Ok(result)
}
