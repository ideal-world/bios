use std::collections::HashMap;

use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;
use bios_basic::test::test_http_client::TestHttpClient;

use bios_mw_flow::dto::flow_config_dto::FlowConfigModifyReq;

use bios_mw_flow::dto::flow_inst_dto::{FlowInstDetailResp, FlowInstStartReq};
use bios_mw_flow::dto::flow_model_dto::{
    FlowModelAddReq, FlowModelAggResp, FlowModelAssociativeOperationKind, FlowModelBindStateReq, FlowModelCopyOrReferenceCiReq, FlowModelCopyOrReferenceReq, FlowModelModifyReq,
    FlowModelSummaryResp,
};
use bios_mw_flow::dto::flow_state_dto::{FlowStateRelModelExt, FlowStateSummaryResp};

use bios_mw_flow::dto::flow_transition_dto::{FlowTransitionAddReq, FlowTransitionModifyReq};
use bios_sdk_invoke::clients::spi_kv_client::KvItemSummaryResp;
use serde_json::json;
use tardis::basic::dto::TardisContext;

use std::time::Duration;
use tardis::basic::result::TardisResult;
use tardis::log::info;
use tardis::web::web_resp::{TardisPage, Void};
use tardis::TardisFuns;

pub async fn test(flow_client: &mut TestHttpClient) -> TardisResult<()> {
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
                name: "测试创建模板1".into(),
                info: Some("xxx".to_string()),
                init_state_id: "".to_string(),
                rel_template_ids: Some(vec![req_template_id1.to_string(), req_template_id2.to_string()]),
                template: true,
                tag: Some("REQ".to_string()),
                scope_level: Some(RbumScopeLevelKind::Private),
                icon: None,
                transitions: None,
                states: None,
                rel_model_id: None,
                disabled: None,
            },
        )
        .await;
    let req_model_template_id = req_model_template_aggs.id.clone();
    // 2-3 config new flow template
    let _: Void = flow_client
        .patch(
            &format!("/cc/model/{}", req_model_template_id.clone()),
            &FlowModelModifyReq {
                init_state_id: Some(init_state_id.to_string()),
                bind_states: Some(vec![
                    FlowModelBindStateReq {
                        state_id: init_state_id.clone(),
                        ext: FlowStateRelModelExt { sort: 1, show_btns: None },
                    },
                    FlowModelBindStateReq {
                        state_id: processing_state_id.clone(),
                        ext: FlowStateRelModelExt { sort: 2, show_btns: None },
                    },
                    FlowModelBindStateReq {
                        state_id: finish_state_id.clone(),
                        ext: FlowStateRelModelExt { sort: 3, show_btns: None },
                    },
                    FlowModelBindStateReq {
                        state_id: closed_state_id.clone(),
                        ext: FlowStateRelModelExt { sort: 4, show_btns: None },
                    },
                ]),
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
                    FlowTransitionAddReq {
                        from_flow_state_id: closed_state_id.clone(),
                        to_flow_state_id: init_state_id.clone(),
                        name: Some("激活".into()),
                        ..Default::default()
                    },
                ]),
                ..Default::default()
            },
        )
        .await;
    let req_model_template: FlowModelAggResp = flow_client.get(&format!("/cc/model/{}", &req_model_template_id)).await;
    // 2-2. flow template bind project template
    let mut rel_model_ids = HashMap::new();
    rel_model_ids.insert("REQ".to_string(), req_model_template_id.clone());
    let result: HashMap<String, FlowModelAggResp> = flow_client
        .post(
            "/ct/model/copy_or_reference_model",
            &FlowModelCopyOrReferenceReq {
                rel_model_ids,
                rel_template_id: Some(project_template_id1.to_string()),
                op: FlowModelAssociativeOperationKind::Reference,
            },
        )
        .await;
    info!("result: {:?}", result);
    let project_req_model_template_id = result.get(&req_model_template_id).unwrap().id.clone();
    assert_ne!(req_model_template_id, project_req_model_template_id);
    let project_result: HashMap<String, FlowModelSummaryResp> = flow_client
        .put(
            &format!("/cc/model/find_rel_models?tag_ids=REQ&is_shared=false&temp_id={}", project_template_id1),
            &json!(""),
        )
        .await;
    assert_eq!(project_req_model_template_id, project_result.get("REQ").unwrap().id.clone());
    // 2-3. modify flow temoplate
    let _: Void = flow_client
        .patch(
            &format!("/cc/model/{}", req_model_template_id.clone()),
            &FlowModelModifyReq {
                modify_transitions: Some(vec![FlowTransitionModifyReq {
                    id: req_model_template.states.iter().find(|state| state.id == init_state_id.clone()).unwrap().transitions[0].id.clone().into(),
                    name: Some("111".into()),
                    ..Default::default()
                }]),
                ..Default::default()
            },
        )
        .await;
    tardis::tokio::time::sleep(Duration::from_millis(100)).await;
    let project_req_model_template: FlowModelAggResp = flow_client.get(&format!("/cc/model/{}", &project_req_model_template_id)).await;
    assert!(project_req_model_template.states.iter().find(|state| state.id == init_state_id.clone()).unwrap().transitions.iter().any(|tran| tran.name == "111"));
    // copy models by project template id
    let result: HashMap<String, FlowModelSummaryResp> = flow_client
        .post(
            &format!("/ct/model/copy_models_by_template_id/{}/{}", project_template_id1, project_template_id2),
            &json!(""),
        )
        .await;
    assert!(result.contains_key(&req_model_template_id));
    // 3. enter project
    ctx.own_paths = "t1/app01".to_string();
    flow_client.set_auth(&ctx)?;
    // 3-1 create project
    let result: HashMap<String, String> = flow_client
        .post(
            "/ci/model/copy_or_reference_model",
            &FlowModelCopyOrReferenceCiReq {
                rel_template_id: Some(project_template_id1.to_string()),
                op: FlowModelAssociativeOperationKind::Reference,
            },
        )
        .await;
    info!("result: {:?}", result);
    // 3-2 Start a instance
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
    info!("req_inst_id1: {:?}", req_inst_id1);
    let req_inst1: FlowInstDetailResp = flow_client.get(&format!("/cc/inst/{}", req_inst_id1)).await;
    info!("req_inst1: {:?}", req_inst1);
    assert_eq!(req_inst1.rel_flow_model_id, project_req_model_template_id);

    let result: HashMap<String, FlowModelSummaryResp> = flow_client.put("/cc/model/find_rel_models?tag_ids=REQ&is_shared=false", &json!("")).await;
    let req_model_id = result.get("REQ").unwrap().id.clone();
    assert_eq!(project_req_model_template_id, req_model_id);
    // 3-3 copy project template to app
    let result: HashMap<String, FlowModelAggResp> = flow_client
        .post(
            "/ca/model/copy_or_reference_model",
            &FlowModelCopyOrReferenceReq {
                rel_model_ids: HashMap::from([("REQ".to_string(), req_model_id.clone())]),
                rel_template_id: None,
                op: FlowModelAssociativeOperationKind::Copy,
            },
        )
        .await;
    let app_req_model_id = result.get(&req_model_id).unwrap().id.clone();
    assert_ne!(req_model_id.clone(), app_req_model_id.clone());
    let result: HashMap<String, FlowModelSummaryResp> = flow_client.put("/cc/model/find_rel_models?tag_ids=REQ&is_shared=false", &json!("")).await;
    let req_model_id = result.get("REQ").unwrap().id.clone();
    assert_eq!(app_req_model_id, req_model_id);
    let req_model_aggs: FlowModelAggResp = flow_client.get(&format!("/cc/model/{}", req_model_id)).await;
    assert_eq!(req_model_aggs.rel_model_id, "".to_string());
    // 3-4 exit app and return tenant
    ctx.own_paths = "t1".to_string();
    flow_client.set_auth(&ctx)?;
    let req_models: Vec<FlowModelSummaryResp> = flow_client.get(&format!("/cc/model/find_by_rel_template_id?tag=REQ&template=true&rel_template_id={}", req_template_id1)).await;
    assert!(!req_models.into_iter().any(|model| model.id == app_req_model_id.clone()));
    // 4 return app
    ctx.own_paths = "t1/app01".to_string();
    flow_client.set_auth(&ctx)?;
    // 4-1 find models by project template id
    let project_result: HashMap<String, FlowModelSummaryResp> = flow_client
        .put(
            &format!("/cc/model/find_rel_models?tag_ids=REQ&is_shared=false&temp_id={}", project_template_id1),
            &json!(""),
        )
        .await;
    assert_eq!(project_req_model_template_id, project_result.get("REQ").unwrap().id.clone());
    // 4-2 rebind project template
    let result: HashMap<String, FlowModelAggResp> = flow_client
        .post(
            "/ca/model/copy_or_reference_model",
            &FlowModelCopyOrReferenceReq {
                rel_model_ids: HashMap::from([("REQ".to_string(), project_req_model_template_id.clone())]),
                rel_template_id: Some(project_template_id1.to_string()),
                op: FlowModelAssociativeOperationKind::Reference,
            },
        )
        .await;
    assert_eq!(project_req_model_template_id, result.get(&project_req_model_template_id).unwrap().id.clone());
    Ok(())
}
