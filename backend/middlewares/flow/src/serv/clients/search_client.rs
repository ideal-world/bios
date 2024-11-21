use std::{collections::HashMap, vec};

use bios_basic::rbum::{dto::rbum_filer_dto::RbumBasicFilterReq, helper::rbum_scope_helper, rbum_enumeration::RbumScopeLevelKind, serv::rbum_item_serv::RbumItemCrudOperation};
use bios_sdk_invoke::{
    clients::{
        event_client::{get_topic, EventCenterClient, SPI_RPC_TOPIC},
        spi_search_client::SpiSearchClient,
    },
    dto::search_item_dto::{SearchItemAddReq, SearchItemModifyReq, SearchItemVisitKeysReq},
};
use itertools::Itertools;
use serde_json::json;
use tardis::{
    basic::{dto::TardisContext, field::TrimString, result::TardisResult}, tokio, TardisFuns, TardisFunsInst
};

use crate::{
    dto::{flow_inst_dto::FlowInstFilterReq, flow_model_dto::{FlowModelDetailResp, FlowModelFilterReq, FlowModelRelTransitionExt}, flow_model_version_dto::FlowModelVersionFilterReq},
    flow_constants,
    serv::{flow_inst_serv::FlowInstServ, flow_model_serv::FlowModelServ, flow_model_version_serv::FlowModelVersionServ, flow_rel_serv::{FlowRelKind, FlowRelServ}},
};

const SEARCH_TAG: &str = "flow_model";

pub struct FlowSearchClient;

impl FlowSearchClient {
    pub async fn async_modify_business_obj_search(inst_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let ctx_clone = ctx.clone();
        let inst_detail = FlowInstServ::get(inst_id, funs, ctx)
        .await?;
        ctx.add_async_task(Box::new(|| {
            Box::pin(async move {
                let task_handle = tokio::spawn(async move {
                    let funs = flow_constants::get_tardis_inst();
                    let _ = Self::modify_business_obj_search(&inst_detail.rel_business_obj_id, &inst_detail.tag, &funs, &ctx_clone).await;
                });
                task_handle.await.unwrap();
                Ok(())
            })
        }))
        .await
    }

    pub async fn modify_business_obj_search(rel_business_obj_id: &str, tag: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let tag_search_map = HashMap::from([
            ("CTS", "idp_test"),
            ("ISSUE", "idp_test"),
            ("ITER", "idp_project"),
            ("MS", "idp_project"),
            ("PROJ", "idp_project"),
            ("REQ", "idp_project"),
            ("TASK", "idp_project"),
            ("TICKET", "ticket"),
            ("TP", "idp_test"),
            ("TS", "idp_test"),
        ]);
        let rel_version_ids = FlowInstServ::find_details(&FlowInstFilterReq {
            rel_business_obj_id: Some(rel_business_obj_id.to_string()),
            main: Some(false),
            finish: Some(false),
            ..Default::default()
        }, funs, ctx).await?.into_iter().map(|inst| inst.rel_flow_version_id).collect_vec();
        let mut rel_transition_names = vec![];
        for rel_version_id in rel_version_ids {
            if let Some(rel_model_id) = FlowModelVersionServ::find_one_item(&FlowModelVersionFilterReq {
                basic: RbumBasicFilterReq {
                    ids: Some(vec![rel_version_id]),
                    with_sub_own_paths: true,
                    own_paths: Some("".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            }, funs, ctx).await?.map(|version| version.rel_model_id) {
                let rel_transition_ext = FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowModelTransition, &rel_model_id, None, None, funs, ctx).await?
                .pop()
                .map(|rel| TardisFuns::json.str_to_obj::<FlowModelRelTransitionExt>(&rel.ext).unwrap_or_default());
                if let Some(ext) = rel_transition_ext {
                    rel_transition_names.push(
                        match ext.id.as_str() {
                            "__EDIT__" => "编辑".to_string(),
                            "__DELETE__" => "删除".to_string(),
                            _ => format!("{}({})", ext.name, ext.from_flow_state_name),
                        }
                    );
                }
            }
        }
        if let Some(table) = tag_search_map.get(tag) {
            SpiSearchClient::modify_item_and_name(
                table,
                rel_business_obj_id,
                &SearchItemModifyReq {
                    kind: None,
                    title: None,
                    name: None,
                    content: None,
                    owner: None,
                    own_paths: None,
                    create_time: None,
                    update_time: None,
                    ext: Some(json!({
                        "rel_transitions": rel_transition_names,
                    })),
                    ext_override: None,
                    visit_keys: None,
                    kv_disable: None,
                },
                funs,
                ctx,
            )
            .await
            .unwrap_or_default();
        }

        Ok(())
    }

    pub async fn async_add_or_modify_model_search(model_id: &str, is_modify: Box<bool>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let ctx_clone = ctx.clone();
        let mock_ctx = TardisContext {
            own_paths: "".to_string(),
            ..ctx.clone()
        };
        let model_resp = FlowModelServ::get_item(
            model_id,
            &FlowModelFilterReq {
                basic: RbumBasicFilterReq {
                    ignore_scope: true,
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            &mock_ctx,
        )
        .await?;
        ctx.add_async_task(Box::new(|| {
            Box::pin(async move {
                let task_handle = tokio::spawn(async move {
                    let funs = flow_constants::get_tardis_inst();
                    let _ = Self::add_or_modify_model_search(&model_resp, is_modify, &funs, &ctx_clone).await;
                });
                task_handle.await.unwrap();
                Ok(())
            })
        }))
        .await
    }

    pub async fn async_delete_model_search(model_id: String, _funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let ctx_clone = ctx.clone();
        ctx.add_async_task(Box::new(|| {
            Box::pin(async move {
                let task_handle = tokio::spawn(async move {
                    let funs = flow_constants::get_tardis_inst();
                    let _ = Self::delete_model_search(&model_id, &funs, &ctx_clone).await;
                });
                task_handle.await.unwrap();
                Ok(())
            })
        }))
        .await
    }

    // flow model 全局搜索埋点方法
    pub async fn add_or_modify_model_search(model_resp: &FlowModelDetailResp, is_modify: Box<bool>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let model_id = &model_resp.id;
        // 数据共享权限处理
        let mut visit_tenants = rbum_scope_helper::get_path_item(RbumScopeLevelKind::L1.to_int(), &model_resp.own_paths).map(|tenant| vec![tenant]).unwrap_or_default();
        let mut visit_apps = rbum_scope_helper::get_path_item(RbumScopeLevelKind::L2.to_int(), &model_resp.own_paths).map(|app| vec![app]).unwrap_or_default();
        let mut own_paths = Some(model_resp.own_paths.clone());
        if model_resp.scope_level == RbumScopeLevelKind::Root {
            visit_apps.push("".to_string());
            visit_tenants.push("".to_string());
            own_paths = Some("".to_string());
        }
        let key = model_id.clone();
        if *is_modify {
            let modify_req = SearchItemModifyReq {
                kind: Some(SEARCH_TAG.to_string()),
                title: Some(model_resp.name.clone()),
                name: Some(model_resp.name.clone()),
                content: Some(model_resp.name.clone()),
                owner: Some(model_resp.owner.clone()),
                own_paths,
                create_time: Some(model_resp.create_time),
                update_time: Some(model_resp.update_time),
                ext: Some(json!({
                    "tag": model_resp.tag,
                    "icon": model_resp.icon,
                    "info": model_resp.info,
                    "rel_template_ids": model_resp.rel_template_ids,
                    "scope_level": model_resp.scope_level,
                    "tenant_id": model_resp.own_paths.clone(),
                })),
                ext_override: Some(true),
                visit_keys: Some(SearchItemVisitKeysReq {
                    accounts: None,
                    apps: Some(visit_apps),
                    tenants: Some(visit_tenants),
                    roles: None,
                    groups: None,
                }),
                kv_disable: None,
            };
            if let Some(_topic) = get_topic(&SPI_RPC_TOPIC) {
                EventCenterClient { topic_code: SPI_RPC_TOPIC }.modify_item_and_name(SEARCH_TAG, &key, &modify_req, funs, ctx).await?;
            } else {
                SpiSearchClient::modify_item_and_name(SEARCH_TAG, &key, &modify_req, funs, ctx).await?;
            }
        } else {
            let add_req = SearchItemAddReq {
                tag: SEARCH_TAG.to_string(),
                kind: SEARCH_TAG.to_string(),
                key: TrimString(key),
                title: model_resp.name.clone(),
                content: model_resp.name.clone(),
                owner: Some(model_resp.owner.clone()),
                own_paths,
                create_time: Some(model_resp.create_time),
                update_time: Some(model_resp.update_time),
                ext: Some(json!({
                    "tag": model_resp.tag,
                    "icon": model_resp.icon,
                    "info": model_resp.info,
                    "rel_template_ids": model_resp.rel_template_ids,
                    "scope_level": model_resp.scope_level,
                    "tenant_id": model_resp.own_paths.clone(),
                })),
                visit_keys: Some(SearchItemVisitKeysReq {
                    accounts: None,
                    apps: Some(visit_apps),
                    tenants: Some(visit_tenants),
                    roles: None,
                    groups: None,
                }),
                kv_disable: None,
            };
            if let Some(_topic) = get_topic(&SPI_RPC_TOPIC) {
                EventCenterClient { topic_code: SPI_RPC_TOPIC }.add_item_and_name(&add_req, Some(model_resp.name.clone()), funs, ctx).await?;
            } else {
                SpiSearchClient::add_item_and_name(&add_req, Some(model_resp.name.clone()), funs, ctx).await?;
            }
        }
        Ok(())
    }

    // model 全局搜索删除埋点方法
    pub async fn delete_model_search(model_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if let Some(_topic) = get_topic(&SPI_RPC_TOPIC) {
            EventCenterClient { topic_code: SPI_RPC_TOPIC }.delete_item_and_name(SEARCH_TAG, model_id, funs, ctx).await?;
        } else {
            SpiSearchClient::delete_item_and_name(SEARCH_TAG, model_id, funs, ctx).await?;
        }
        Ok(())
    }
}
