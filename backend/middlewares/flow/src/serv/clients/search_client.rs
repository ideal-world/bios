use std::vec;

use bios_basic::rbum::{dto::rbum_filer_dto::RbumBasicFilterReq, helper::rbum_scope_helper, rbum_enumeration::RbumScopeLevelKind, serv::rbum_item_serv::RbumItemCrudOperation};
use bios_sdk_invoke::{clients::spi_search_client::{SpiSearchClient, SpiSearchEventExt}, dto::search_item_dto::{SearchItemAddReq, SearchItemModifyReq, SearchItemVisitKeysReq}, invoke_config::InvokeConfigApi};
use serde_json::json;
use tardis::{basic::{dto::TardisContext, field::TrimString, result::TardisResult}, tokio, TardisFunsInst};

use crate::{dto::flow_model_dto::{FlowModelDetailResp, FlowModelFilterReq}, flow_constants, flow_initializer::{default_search_avatar, ws_search_client}, serv::flow_model_serv::FlowModelServ};

const SEARCH_TAG: &str = "flow_model";

pub struct IamSearchClient;

impl IamSearchClient {
    pub async fn async_add_or_modify_model_search(model_id: &str, is_modify: Box<bool>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let ctx_clone = ctx.clone();
        let mock_ctx = TardisContext {
            own_paths: "".to_string(),
            ..ctx.clone()
        };
        if let Some(model_resp) = FlowModelServ::find_one_detail_item(
            &FlowModelFilterReq {
                basic: RbumBasicFilterReq {
                    ids: Some(vec![model_id.to_string()]),
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
        .await? {
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
        } else {
            Ok(())
        }
        
    }

    // flow model 全局搜索埋点方法
    pub async fn add_or_modify_model_search(
        model_resp: &FlowModelDetailResp,
        is_modify: Box<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let model_id = &model_resp.id;

        let key = model_id.clone();
        if *is_modify {
            let modify_req = SearchItemModifyReq {
                kind: Some(SEARCH_TAG.to_string()),
                title: Some(model_resp.name.clone()),
                name: Some(model_resp.name.clone()),
                content: Some(model_resp.name.clone()),
                owner: Some(model_resp.owner.clone()),
                own_paths: Some(model_resp.own_paths.clone()),
                create_time: Some(model_resp.create_time),
                update_time: Some(model_resp.update_time),
                ext: Some(json!({
                    "icon": model_resp.icon,
                    "info": model_resp.info,
                    "rel_template_ids": model_resp.rel_template_ids,
                    "scope_level": model_resp.scope_level,
                })),
                ext_override: Some(true),
                visit_keys: Some(SearchItemVisitKeysReq {
                    accounts: None,
                    apps: Some(vec![rbum_scope_helper::get_path_item(RbumScopeLevelKind::L2.to_int(), &model_resp.own_paths).unwrap_or_default()]),
                    tenants: Some(vec![rbum_scope_helper::get_path_item(RbumScopeLevelKind::L1.to_int(), &model_resp.own_paths).unwrap_or_default()]),
                    roles: None,
                    groups: None,
                }),
            };
            if let Some(ws_client) = ws_search_client().await {
                ws_client.publish_modify_item(SEARCH_TAG.to_string(), key, &modify_req, default_search_avatar().await.clone(), funs.invoke_conf_spi_app_id(), ctx).await?;
            } else {
                SpiSearchClient::modify_item(SEARCH_TAG, &key, &modify_req, funs, ctx).await?;
            }
        } else {
            let add_req = SearchItemAddReq {
                tag: SEARCH_TAG.to_string(),
                kind: SEARCH_TAG.to_string(),
                key: TrimString(key),
                title: model_resp.name.clone(),
                name: Some(model_resp.name.clone()),
                content: model_resp.name.clone(),
                owner: Some(model_resp.owner.clone()),
                own_paths: Some(model_resp.own_paths.clone()),
                create_time: Some(model_resp.create_time),
                update_time: Some(model_resp.update_time),
                ext: Some(json!({
                    "icon": model_resp.icon,
                    "info": model_resp.info,
                    "rel_template_ids": model_resp.rel_template_ids,
                    "scope_level": model_resp.scope_level,
                })),
                visit_keys: Some(SearchItemVisitKeysReq {
                    accounts: None,
                    apps: Some(vec![rbum_scope_helper::get_path_item(RbumScopeLevelKind::L2.to_int(), &model_resp.own_paths).unwrap_or_default()]),
                    tenants: Some(vec![rbum_scope_helper::get_path_item(RbumScopeLevelKind::L1.to_int(), &model_resp.own_paths).unwrap_or_default()]),
                    roles: None,
                    groups: None,
                }),
            };
            if let Some(ws_client) = ws_search_client().await {
                ws_client.publish_add_item(&add_req, default_search_avatar().await.clone(), funs.invoke_conf_spi_app_id(), ctx).await?;
            } else {
                SpiSearchClient::add_item(&add_req, funs, ctx).await?;
            }
        }
        Ok(())
    }
}