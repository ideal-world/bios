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
use bios_spi_search::dto::search_item_dto::{SearchItemQueryReq, SearchItemSearchCtxReq, SearchItemSearchPageReq, SearchItemSearchQScopeKind, SearchItemSearchReq, SearchItemSearchResp};
use serde_json::json;
use tardis::basic::dto::TardisContext;

use std::time::Duration;
use tardis::basic::result::TardisResult;
use tardis::log::{debug, info};
use tardis::web::web_resp::{TardisPage, Void};
use tardis::TardisFuns;
use tardis::tokio::time::sleep;

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
                name: "测试需求模板1".into(),
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
    let req_default_model_template_aggs: FlowModelAggResp = flow_client
        .post(
            "/cc/model",
            &FlowModelAddReq {
                name: "测试需求默认模板1".into(),
                info: Some("xxx".to_string()),
                init_state_id: "".to_string(),
                rel_template_ids: None,
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
    let req_default_model_template_id = req_default_model_template_aggs.id.clone();
    let _: Void = flow_client
        .patch(
            &format!("/cc/model/{}", req_default_model_template_id.clone()),
            &FlowModelModifyReq {
                init_state_id: Some(init_state_id.to_string()),
                bind_states: Some(vec![
                    FlowModelBindStateReq {
                        state_id: init_state_id.clone(),
                        ext: FlowStateRelModelExt { sort: 1, show_btns: None },
                    },
                ]),
                add_transitions: None,
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
    let req_model_uninit_template_id = req_model_uninit_template_aggs.id.clone();
    sleep(Duration::from_millis(500)).await;
    let model_templates: TardisPage<SearchItemSearchResp> = search_client.put("/ci/item/search", &SearchItemSearchReq {
        tag: "flow_model".to_string(),
        ctx: SearchItemSearchCtxReq {
            tenants: Some(vec![ctx.own_paths.clone()]),
            ..Default::default()
        },
        query: SearchItemQueryReq {
            ..Default::default()
        },
        adv_query: None,
        sort: None,
        page: SearchItemSearchPageReq {
            number: 1,
            size: 20,
            fetch_total: true,
        }
    }).await;
    assert_eq!(model_templates.total_size, 3);
    assert!(model_templates.records.iter().any(|record| record.key == req_default_model_template_id));
    assert!(model_templates.records.iter().any(|record| record.key == req_model_uninit_template_id));
    assert!(model_templates.records.iter().any(|record| record.key == req_model_template_id));
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
    let model_templates: TardisPage<SearchItemSearchResp> = search_client.put("/ci/item/search", &SearchItemSearchReq {
        tag: "flow_model".to_string(),
        ctx: SearchItemSearchCtxReq {
            tenants: Some(vec![ctx.own_paths.clone()]),
            ..Default::default()
        },
        query: SearchItemQueryReq {
            ..Default::default()
        },
        adv_query: None,
        sort: None,
        page: SearchItemSearchPageReq {
            number: 1,
            size: 20,
            fetch_total: true,
        }
    }).await;
    assert_eq!(model_templates.records.iter().find(|record| record.key == req_default_model_template_id).unwrap().title, "测试需求默认模板11".to_string());
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
    let model_templates: TardisPage<SearchItemSearchResp> = search_client.put("/ci/item/search", &SearchItemSearchReq {
        tag: "flow_model".to_string(),
        ctx: SearchItemSearchCtxReq {
            tenants: Some(vec![ctx.own_paths.clone(), "".to_string()]),
            ..Default::default()
        },
        query: SearchItemQueryReq {
            ..Default::default()
        },
        adv_query: None,
        sort: None,
        page: SearchItemSearchPageReq {
            number: 1,
            size: 20,
            fetch_total: true,
        }
    }).await;
    assert_eq!(model_templates.total_size, 1);
    assert_eq!(model_templates.records[0].key, req_default_model_template_id);
    let copy_template_model: FlowModelAggResp = flow_client
        .patch(
            &format!("/cc/model/copy/{}", req_default_model_template_id.clone()),
            &json!({}),
        )
        .await;
    sleep(Duration::from_millis(1000)).await;
    let model_templates: TardisPage<SearchItemSearchResp> = search_client.put("/ci/item/search", &SearchItemSearchReq {
        tag: "flow_model".to_string(),
        ctx: SearchItemSearchCtxReq {
            tenants: Some(vec![ctx.own_paths.clone(), "".to_string()]),
            ..Default::default()
        },
        query: SearchItemQueryReq {
            ..Default::default()
        },
        adv_query: None,
        sort: None,
        page: SearchItemSearchPageReq {
            number: 1,
            size: 20,
            fetch_total: true,
        }
    }).await;
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
    assert!(req_models.iter().any(|mdoel| mdoel.id == req_default_model_template_id));
    assert!(req_models.iter().any(|mdoel| mdoel.id == req_model_template_id));
    assert!(req_models.iter().all(|mdoel| mdoel.id != req_model_uninit_template_id));

    let req_models: Vec<FlowModelSummaryResp> = flow_client.get("/cc/model/find_by_rel_template_id?tag=REQ&template=true").await;
    assert_eq!(req_models.len(), 3);
    assert!(req_models.iter().any(|mdoel| mdoel.id == req_default_model_template_id));
    assert!(req_models.iter().all(|mdoel| mdoel.id != req_model_template_id));
    ctx.owner = "u001".to_string();
    ctx.own_paths = "t2".to_string();
    flow_client.set_auth(&ctx)?;
    search_client.set_auth(&ctx)?;
    let req_models: Vec<FlowModelSummaryResp> = flow_client.get("/cc/model/find_by_rel_template_id?tag=REQ&template=true").await;
    assert_eq!(req_models.len(), 3);
    assert!(req_models.iter().any(|mdoel| mdoel.id == req_default_model_template_id));
    assert!(req_models.iter().all(|mdoel| mdoel.id != req_model_template_id));
    Ok(())
}
