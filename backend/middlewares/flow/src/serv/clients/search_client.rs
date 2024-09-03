use std::vec;

use bios_basic::rbum::{dto::rbum_filer_dto::RbumBasicFilterReq, helper::rbum_scope_helper, rbum_enumeration::RbumScopeLevelKind, serv::rbum_item_serv::RbumItemCrudOperation};
use bios_sdk_invoke::{
    clients::{
        event_client::{get_topic, EventCenterClient, SPI_RPC_TOPIC},
        flow_client::event::FLOW_AVATAR,
        spi_search_client::SpiSearchClient,
    },
    dto::search_item_dto::{SearchItemAddReq, SearchItemModifyReq, SearchItemVisitKeysReq},
};
use serde_json::json;
use tardis::{
    basic::{dto::TardisContext, field::TrimString, result::TardisResult},
    tokio, TardisFuns, TardisFunsInst,
};

use crate::{
    dto::flow_model_dto::{FlowModelDetailResp, FlowModelFilterReq},
    flow_constants,
    serv::flow_model_serv::FlowModelServ,
};

const SEARCH_TAG: &str = "flow_model";

pub struct FlowSearchClient;

impl FlowSearchClient {
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
            SpiSearchClient::add_item_and_name(&add_req, Some(model_resp.name.clone()), funs, ctx).await?;
        }
        Ok(())
    }

    // model 全局搜索删除埋点方法
    pub async fn delete_model_search(model_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        SpiSearchClient::delete_item_and_name(SEARCH_TAG, model_id, funs, ctx).await?;
        Ok(())
    }
}
