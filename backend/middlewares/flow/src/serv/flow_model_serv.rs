use std::{collections::HashMap, vec};

use bios_basic::rbum::{
    dto::{
        rbum_filer_dto::{RbumBasicFilterReq, RbumItemRelFilterReq},
        rbum_item_dto::{RbumItemKernelAddReq, RbumItemKernelModifyReq},
        rbum_rel_dto::RbumRelModifyReq,
    },
    helper::rbum_scope_helper,
    rbum_enumeration::{RbumRelFromKind, RbumScopeLevelKind},
    serv::{
        rbum_crud_serv::{ID_FIELD, NAME_FIELD, REL_DOMAIN_ID_FIELD, REL_KIND_ID_FIELD},
        rbum_item_serv::{RbumItemCrudOperation, RBUM_ITEM_TABLE},
    },
};
use itertools::Itertools;
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    chrono::Utc,
    db::sea_orm::{
        sea_query::{Alias, Cond, Expr, Query, SelectStatement},
        EntityName, EntityTrait, JoinType, Order, QueryFilter, Set,
    },
    futures::future::join_all,
    serde_json::json,
    tokio,
    web::web_resp::TardisPage,
    TardisFuns, TardisFunsInst,
};

use crate::{
    domain::{flow_model, flow_state, flow_transition},
    dto::{
        flow_model_dto::{
            FlowModelAddReq, FlowModelAggResp, FlowModelAssociativeOperationKind, FlowModelBindStateReq, FlowModelDetailResp, FlowModelFilterReq, FlowModelFindRelStateResp,
            FlowModelModifyReq, FlowModelSummaryResp,
        },
        flow_state_dto::{FlowStateAggResp, FlowStateDetailResp, FlowStateFilterReq, FlowStateRelModelExt, FlowStateRelModelModifyReq},
        flow_transition_dto::{
            FlowTransitionActionChangeKind, FlowTransitionAddReq, FlowTransitionDetailResp, FlowTransitionInitInfo, FlowTransitionModifyReq, FlowTransitionPostActionInfo,
        },
    },
    flow_config::FlowBasicInfoManager,
    flow_constants,
    serv::flow_state_serv::FlowStateServ,
};
use async_trait::async_trait;

use super::{
    clients::{
        flow_log_client::{FlowLogClient, LogParamContent, LogParamTag},
        search_client::FlowSearchClient,
    },
    flow_inst_serv::FlowInstServ,
    flow_rel_serv::{FlowRelKind, FlowRelServ},
};

pub struct FlowModelServ;

#[async_trait]
impl RbumItemCrudOperation<flow_model::ActiveModel, FlowModelAddReq, FlowModelModifyReq, FlowModelSummaryResp, FlowModelDetailResp, FlowModelFilterReq> for FlowModelServ {
    fn get_ext_table_name() -> &'static str {
        flow_model::Entity.table_name()
    }

    fn get_rbum_kind_id() -> Option<String> {
        Some(FlowBasicInfoManager::get_config(|conf: &crate::flow_config::BasicInfo| conf.kind_model_id.clone()))
    }

    fn get_rbum_domain_id() -> Option<String> {
        Some(FlowBasicInfoManager::get_config(|conf: &crate::flow_config::BasicInfo| conf.domain_flow_id.clone()))
    }

    async fn package_item_add(add_req: &FlowModelAddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<RbumItemKernelAddReq> {
        Ok(RbumItemKernelAddReq {
            name: add_req.name.clone(),
            disabled: add_req.disabled,
            scope_level: add_req.scope_level.clone(),
            ..Default::default()
        })
    }

    async fn package_ext_add(id: &str, add_req: &FlowModelAddReq, _: &TardisFunsInst, _ctx: &TardisContext) -> TardisResult<flow_model::ActiveModel> {
        Ok(flow_model::ActiveModel {
            id: Set(id.to_string()),
            icon: Set(add_req.icon.as_ref().unwrap_or(&"".to_string()).to_string()),
            info: Set(add_req.info.as_ref().unwrap_or(&"".to_string()).to_string()),
            init_state_id: Set(add_req.init_state_id.to_string()),
            tag: Set(add_req.tag.clone()),
            rel_model_id: Set(add_req.rel_model_id.as_ref().unwrap_or(&"".to_string()).to_string()),
            template: Set(add_req.template),
            ..Default::default()
        })
    }

    async fn before_add_item(_add_req: &mut FlowModelAddReq, _funs: &TardisFunsInst, _ctx: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    async fn after_add_item(flow_model_id: &str, add_req: &mut FlowModelAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if let Some(states) = &add_req.states {
            join_all(states.iter().map(|state| async { Self::bind_state(flow_model_id, state, funs, ctx).await }).collect_vec())
                .await
                .into_iter()
                .collect::<TardisResult<Vec<()>>>()?;
        }
        if let Some(rel_template_ids) = &add_req.rel_template_ids {
            join_all(
                rel_template_ids
                    .iter()
                    .map(|rel_template_id| async {
                        FlowRelServ::add_simple_rel(&FlowRelKind::FlowModelTemplate, flow_model_id, rel_template_id, None, None, false, true, None, funs, ctx).await
                    })
                    .collect_vec(),
            )
            .await
            .into_iter()
            .collect::<TardisResult<Vec<()>>>()?;
        }
        if let Some(transitions) = &add_req.transitions {
            if !transitions.is_empty() {
                Self::add_transitions(flow_model_id, transitions, funs, ctx).await?;
                // check transition post action endless loop
                let model_desp = Self::get_item(flow_model_id, &FlowModelFilterReq::default(), funs, ctx).await?;
                if Self::check_post_action_ring(&model_desp, funs, ctx).await? {
                    return Err(funs.err().not_found(
                        "flow_model_Serv",
                        "after_add_item",
                        "this post action exist endless loop",
                        "500-flow-transition-endless-loop",
                    ));
                }
            }
        }
        if add_req.template && add_req.rel_model_id.clone().map_or(true, |id| id.is_empty()) {
            FlowSearchClient::async_add_or_modify_model_search(flow_model_id, Box::new(false), funs, ctx).await?;
            FlowLogClient::add_ctx_task(
                LogParamTag::DynamicLog,
                Some(flow_model_id.to_string()),
                LogParamContent {
                    subject: "工作流模板".to_string(),
                    name: add_req.name.to_string(),
                    sub_kind: "flow_template".to_string(),
                },
                Some(json!({
                    "name": add_req.name.to_string(),
                    "info": add_req.info.clone().unwrap_or_default(),
                    "rel_template_ids":add_req.rel_template_ids.clone().unwrap_or_default(),
                    "scope_level": add_req.scope_level.clone(),
                    "tag": add_req.tag.clone().unwrap_or_default(),
                })),
                Some("dynamic_log_tenant_config".to_string()),
                Some("新建".to_string()),
                rbum_scope_helper::get_path_item(RbumScopeLevelKind::L1.to_int(), &ctx.own_paths),
                ctx,
                false,
            )
            .await?;
        }

        Ok(())
    }

    async fn package_item_modify(_: &str, modify_req: &FlowModelModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<Option<RbumItemKernelModifyReq>> {
        if modify_req.name.is_none() && modify_req.scope_level.is_none() && modify_req.disabled.is_none() {
            return Ok(None);
        }
        Ok(Some(RbumItemKernelModifyReq {
            code: None,
            name: modify_req.name.clone(),
            scope_level: modify_req.scope_level.clone(),
            disabled: modify_req.disabled,
        }))
    }

    async fn package_ext_modify(id: &str, modify_req: &FlowModelModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<Option<flow_model::ActiveModel>> {
        if modify_req.icon.is_none() && modify_req.info.is_none() && modify_req.init_state_id.is_none() && modify_req.tag.is_none() && modify_req.rel_model_id.is_none() {
            return Ok(None);
        }
        let mut flow_model = flow_model::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(icon) = &modify_req.icon {
            flow_model.icon = Set(icon.to_string());
        }
        if let Some(info) = &modify_req.info {
            flow_model.info = Set(info.to_string());
        }
        if let Some(init_state_id) = &modify_req.init_state_id {
            flow_model.init_state_id = Set(init_state_id.to_string());
        }
        if let Some(tag) = &modify_req.tag {
            flow_model.tag = Set(Some(tag.clone()));
        }
        if let Some(rel_model_id) = &modify_req.rel_model_id {
            flow_model.rel_model_id = Set(rel_model_id.clone());
        }
        Ok(Some(flow_model))
    }

    async fn before_modify_item(flow_model_id: &str, _: &mut FlowModelModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let current_model = Self::get_item(
            flow_model_id,
            &FlowModelFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    own_paths: Some("".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        if current_model.own_paths != ctx.own_paths {
            return Err(funs.err().internal_error(
                "flow_model_serv",
                "before_modify_item",
                "The own_paths of current mode isn't the own_paths of ctx",
                "404-flow-model-not-found",
            ));
        }
        Ok(())
    }

    async fn after_modify_item(flow_model_id: &str, modify_req: &mut FlowModelModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let model_detail = Self::get_item(flow_model_id, &FlowModelFilterReq::default(), funs, ctx).await?;
        if let Some(bind_states) = &modify_req.bind_states {
            for bind_state in bind_states {
                Self::bind_state(flow_model_id, bind_state, funs, ctx).await?;
            }
        }
        if let Some(unbind_states) = &modify_req.unbind_states {
            for unbind_state in unbind_states {
                Self::unbind_state(flow_model_id, unbind_state, funs, ctx).await?;
            }
        }
        if let Some(modify_states) = &modify_req.modify_states {
            for modify_state in modify_states {
                Self::modify_rel_state_ext(flow_model_id, modify_state, funs, ctx).await?;
            }
        }
        if let Some(add_transitions) = &modify_req.add_transitions {
            Self::add_transitions(flow_model_id, add_transitions, funs, ctx).await?;
        }
        if let Some(modify_transitions) = &modify_req.modify_transitions {
            Self::modify_transitions(flow_model_id, modify_transitions, &model_detail, funs, ctx).await?;
        }
        if let Some(delete_transitions) = &modify_req.delete_transitions {
            Self::delete_transitions(flow_model_id, delete_transitions, funs, ctx).await?;
        }
        if let Some(rel_template_ids) = &modify_req.rel_template_ids {
            join_all(
                FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowModelTemplate, flow_model_id, None, None, funs, ctx)
                    .await?
                    .into_iter()
                    .map(|rel| async move { FlowRelServ::delete_simple_rel(&FlowRelKind::FlowModelTemplate, flow_model_id, &rel.rel_id, funs, ctx).await })
                    .collect_vec(),
            )
            .await
            .into_iter()
            .collect::<TardisResult<Vec<()>>>()?;
            join_all(
                rel_template_ids
                    .iter()
                    .map(|rel_template_id| async {
                        FlowRelServ::add_simple_rel(&FlowRelKind::FlowModelTemplate, flow_model_id, rel_template_id, None, None, false, true, None, funs, ctx).await
                    })
                    .collect_vec(),
            )
            .await
            .into_iter()
            .collect::<TardisResult<Vec<()>>>()?;
        }
        if modify_req.add_transitions.is_some() || modify_req.modify_transitions.is_some() {
            // check transition post action endless loop
            if Self::check_post_action_ring(&model_detail, funs, ctx).await? {
                return Err(funs.err().not_found(
                    "flow_model_Serv",
                    "after_modify_item",
                    "this post action exist endless loop",
                    "500-flow-transition-endless-loop",
                ));
            }
        }
        let model = Self::get_item_detail_aggs(flow_model_id, false, funs, ctx).await?;
        if model.template && model.rel_model_id.is_empty() {
            FlowSearchClient::async_add_or_modify_model_search(flow_model_id, Box::new(true), funs, ctx).await?;
            FlowLogClient::add_ctx_task(
                LogParamTag::DynamicLog,
                Some(flow_model_id.to_string()),
                LogParamContent {
                    subject: "工作流模板".to_string(),
                    name: model.name.clone(),
                    sub_kind: "flow_template".to_string(),
                },
                Some(json!({
                    "name": model.name.to_string(),
                    "info": model.info.clone(),
                    "rel_template_ids":model.rel_template_ids.clone(),
                    "scope_level": model.scope_level.clone(),
                    "tag": model.tag.clone(),
                })),
                Some("dynamic_log_tenant_config".to_string()),
                Some("编辑".to_string()),
                rbum_scope_helper::get_path_item(RbumScopeLevelKind::L1.to_int(), &ctx.own_paths),
                ctx,
                false,
            )
            .await?;
        }

        // 同步修改所有引用的下级模型
        if model.template {
            let parent_model_transitions = model_detail.transitions();
            let child_models = Self::find_detail_items(
                &FlowModelFilterReq {
                    rel_model_ids: Some(vec![flow_model_id.to_string()]),
                    ..Default::default()
                },
                None,
                None,
                funs,
                ctx,
            )
            .await?;
            for child_model in child_models {
                let ctx_clone = TardisContext {
                    own_paths: child_model.own_paths.clone(),
                    ..ctx.clone()
                };
                let child_model_transitions = child_model.transitions();
                let mut modify_req_clone = modify_req.clone();
                if let Some(ref mut add_transitions) = &mut modify_req_clone.add_transitions {
                    for add_transition in add_transitions.iter_mut() {
                        if let Some(ref mut action_by_post_changes) = &mut add_transition.action_by_post_changes {
                            for action_by_post_change in action_by_post_changes.iter_mut() {
                                action_by_post_change.is_edit = Some(false); // 引用复制时，置为不可编辑
                            }
                        }
                    }
                }
                if let Some(ref mut modify_transitions) = &mut modify_req_clone.modify_transitions {
                    for modify_transition in modify_transitions.iter_mut() {
                        let parent_model_transition = parent_model_transitions.iter().find(|trans| trans.id == modify_transition.id.to_string()).unwrap();
                        modify_transition.id = child_model_transitions
                            .iter()
                            .find(|child_tran| {
                                child_tran.from_flow_state_id == parent_model_transition.from_flow_state_id
                                    && child_tran.to_flow_state_id == parent_model_transition.to_flow_state_id
                            })
                            .map(|trans| trans.id.clone())
                            .unwrap_or_default()
                            .into();
                    }
                }
                if let Some(delete_transitions) = &mut modify_req_clone.delete_transitions {
                    let mut child_delete_transitions = vec![];
                    for delete_transition_id in delete_transitions.iter_mut() {
                        let parent_model_transition = parent_model_transitions.iter().find(|trans| trans.id == delete_transition_id.clone()).unwrap();
                        child_delete_transitions.push(
                            child_model_transitions
                                .iter()
                                .find(|tran| {
                                    tran.from_flow_state_id == parent_model_transition.from_flow_state_id && tran.to_flow_state_id == parent_model_transition.to_flow_state_id
                                })
                                .map(|trans| trans.id.clone())
                                .unwrap_or_default(),
                        );
                    }
                    modify_req_clone.delete_transitions = Some(child_delete_transitions);
                }
                ctx.add_async_task(Box::new(|| {
                    Box::pin(async move {
                        let task_handle = tokio::spawn(async move {
                            let funs = flow_constants::get_tardis_inst();
                            let _ = Self::modify_item(&child_model.id, &mut modify_req_clone, &funs, &ctx_clone).await;
                        });
                        task_handle.await.unwrap();
                        Ok(())
                    })
                }))
                .await?;
            }
        }

        Ok(())
    }

    async fn before_delete_item(flow_model_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<FlowModelDetailResp>> {
        if !Self::find_id_items(
            &FlowModelFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    own_paths: Some("".to_string()),
                    ..Default::default()
                },
                rel_model_ids: Some(vec![flow_model_id.to_string()]),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?
        .is_empty()
        {
            return Err(funs.err().not_found(&Self::get_obj_name(), "delete_item", "the model prohibit delete", "500-flow_model-prohibit-delete"));
        }
        let detail = Self::get_item(flow_model_id, &FlowModelFilterReq::default(), funs, ctx).await?;

        join_all(
            FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowModelTemplate, flow_model_id, None, None, funs, ctx)
                .await?
                .into_iter()
                .map(|rel| async move { FlowRelServ::delete_simple_rel(&FlowRelKind::FlowModelTemplate, flow_model_id, &rel.rel_id, funs, ctx).await })
                .collect_vec(),
        )
        .await
        .into_iter()
        .collect::<TardisResult<Vec<()>>>()?;
        join_all(
            FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowModelState, flow_model_id, None, None, funs, ctx)
                .await?
                .into_iter()
                .map(|rel| async move { FlowRelServ::delete_simple_rel(&FlowRelKind::FlowModelState, flow_model_id, &rel.rel_id, funs, ctx).await })
                .collect_vec(),
        )
        .await
        .into_iter()
        .collect::<TardisResult<Vec<()>>>()?;

        Ok(Some(detail))
    }

    async fn after_delete_item(flow_model_id: &str, detail: &Option<FlowModelDetailResp>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if detail.is_some() && detail.as_ref().unwrap().template && detail.as_ref().unwrap().rel_model_id.is_empty() {
            FlowSearchClient::async_delete_model_search(flow_model_id.to_string(), funs, ctx).await?;
            FlowLogClient::add_ctx_task(
                LogParamTag::DynamicLog,
                Some(flow_model_id.to_string()),
                LogParamContent {
                    subject: "工作流模板".to_string(),
                    name: detail.as_ref().unwrap().name.clone(),
                    sub_kind: "flow_template".to_string(),
                },
                Some(json!({
                    "name": detail.as_ref().unwrap().name.to_string(),
                    "info": detail.as_ref().unwrap().info.clone(),
                    "rel_template_ids":detail.as_ref().unwrap().rel_template_ids.clone(),
                    "scope_level": detail.as_ref().unwrap().scope_level.clone(),
                    "tag": detail.as_ref().unwrap().tag.clone(),
                })),
                Some("dynamic_log_tenant_config".to_string()),
                Some("删除".to_string()),
                rbum_scope_helper::get_path_item(RbumScopeLevelKind::L1.to_int(), &ctx.own_paths),
                ctx,
                false,
            )
            .await?;
        }
        Ok(())
    }

    async fn package_ext_query(query: &mut SelectStatement, _: bool, filter: &FlowModelFilterReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<()> {
        query
            .column((flow_model::Entity, flow_model::Column::Icon))
            .column((flow_model::Entity, flow_model::Column::Info))
            .column((flow_model::Entity, flow_model::Column::InitStateId))
            .column((flow_model::Entity, flow_model::Column::Template))
            .column((flow_model::Entity, flow_model::Column::RelModelId))
            .column((flow_model::Entity, flow_model::Column::Tag))
            .expr_as(Expr::val(json! {()}), Alias::new("transitions"))
            .expr_as(Expr::val(json! {()}), Alias::new("states"))
            .expr_as(Expr::val(vec!["".to_string()]), Alias::new("rel_template_ids"));
        if let Some(tags) = filter.tags.clone() {
            query.and_where(Expr::col(flow_model::Column::Tag).is_in(tags));
        }
        if let Some(template) = filter.template {
            query.and_where(Expr::col(flow_model::Column::Template).eq(template));
        }
        if let Some(own_paths) = filter.own_paths.clone() {
            query.and_where(Expr::col((flow_model::Entity, flow_model::Column::OwnPaths)).is_in(own_paths));
        }
        if let Some(rel_model_ids) = filter.rel_model_ids.clone() {
            query.and_where(Expr::col(flow_model::Column::RelModelId).is_in(rel_model_ids));
        }

        Ok(())
    }

    async fn get_item(flow_model_id: &str, filter: &FlowModelFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<FlowModelDetailResp> {
        let mut flow_model = Self::do_get_item(flow_model_id, filter, funs, ctx).await?;
        let flow_transitions = Self::find_transitions(flow_model_id, filter.specified_state_ids.as_deref(), funs, ctx).await?;
        flow_model.transitions = Some(TardisFuns::json.obj_to_json(&flow_transitions)?);

        let flow_states = FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowModelState, flow_model_id, None, None, funs, ctx)
            .await?
            .into_iter()
            .sorted_by_key(|rel| TardisFuns::json.str_to_obj::<FlowStateRelModelExt>(&rel.ext).unwrap_or_default().sort)
            .map(|rel| FlowStateAggResp {
                id: rel.rel_id.clone(),
                name: rel.rel_name.clone(),
                is_init: flow_model.init_state_id == rel.rel_id,
                ext: TardisFuns::json.str_to_obj::<FlowStateRelModelExt>(&rel.ext).unwrap_or_default(),
                transitions: flow_transitions.clone().into_iter().filter(|tran| tran.from_flow_state_id == rel.rel_id).collect_vec(),
            })
            .collect_vec();
        flow_model.states = Some(TardisFuns::json.obj_to_json(&flow_states)?);

        let rel_template_ids =
            FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowModelTemplate, flow_model_id, None, None, funs, ctx).await?.into_iter().map(|rel| rel.rel_id).collect_vec();
        flow_model.rel_template_ids = rel_template_ids;

        Ok(flow_model)
    }

    async fn paginate_detail_items(
        filter: &FlowModelFilterReq,
        page_number: u32,
        page_size: u32,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<FlowModelDetailResp>> {
        let mut flow_models = Self::do_paginate_detail_items(filter, page_number, page_size, desc_sort_by_create, desc_sort_by_update, funs, ctx).await?;
        for flow_model in &mut flow_models.records {
            let flow_transitions = Self::find_transitions(&flow_model.id, filter.specified_state_ids.as_deref(), funs, ctx).await?;
            flow_model.transitions = Some(TardisFuns::json.obj_to_json(&flow_transitions)?);
        }
        Ok(flow_models)
    }

    async fn find_detail_items(
        filter: &FlowModelFilterReq,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<FlowModelDetailResp>> {
        let mut flow_models = Self::do_find_detail_items(filter, desc_sort_by_create, desc_sort_by_update, funs, ctx).await?;
        for flow_model in &mut flow_models {
            let flow_transitions = Self::find_transitions(&flow_model.id, filter.specified_state_ids.as_deref(), funs, ctx).await?;
            flow_model.transitions = Some(TardisFuns::json.obj_to_json(&flow_transitions)?);
        }
        Ok(flow_models)
    }
}

impl FlowModelServ {
    pub async fn init_model(
        tag: &str,
        init_state_id: String,
        state_ids: Vec<String>,
        model_name: &str,
        transitions: Vec<FlowTransitionInitInfo>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<String> {
        let mut bind_states = vec![];
        // states
        for (i, state_id) in state_ids.into_iter().enumerate() {
            bind_states.push(FlowModelBindStateReq {
                state_id,
                ext: FlowStateRelModelExt { sort: i as i64, show_btns: None },
            });
        }
        // transitions
        let mut add_transitions = vec![];
        for transition in transitions {
            add_transitions.push(FlowTransitionAddReq::try_from(transition)?);
        }
        // add model
        let model_id = Self::add_item(
            &mut FlowModelAddReq {
                name: model_name.into(),
                init_state_id: init_state_id.clone(),
                rel_template_ids: None,
                icon: None,
                info: None,
                transitions: Some(add_transitions),
                states: Some(bind_states),
                tag: Some(tag.to_string()),
                scope_level: Some(RbumScopeLevelKind::Root),
                disabled: None,
                template: true,
                rel_model_id: None,
            },
            funs,
            ctx,
        )
        .await?;

        Ok(model_id)
    }

    pub async fn add_transitions(flow_model_id: &str, add_req: &[FlowTransitionAddReq], funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let flow_state_ids =
            FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowModelState, flow_model_id, None, None, funs, ctx).await?.iter().map(|rel| rel.rel_id.clone()).collect::<Vec<_>>();
        if add_req.iter().any(|req| !flow_state_ids.contains(&req.from_flow_state_id) || !flow_state_ids.contains(&req.to_flow_state_id)) {
            return Err(funs.err().not_found(
                &Self::get_obj_name(),
                "add_transitions",
                "the states to be added is not legal",
                "404-flow-state-add-not-legal",
            ));
        }
        let flow_transitions = add_req
            .iter()
            .map(|req| flow_transition::ActiveModel {
                id: Set(TardisFuns::field.nanoid()),
                name: Set(req.name.as_ref().map(|name| name.to_string()).unwrap_or("".to_string())),

                from_flow_state_id: Set(req.from_flow_state_id.to_string()),
                to_flow_state_id: Set(req.to_flow_state_id.to_string()),

                transfer_by_auto: Set(req.transfer_by_auto.unwrap_or(false)),
                transfer_by_timer: Set(req.transfer_by_timer.as_ref().unwrap_or(&"".to_string()).to_string()),

                guard_by_creator: Set(req.guard_by_creator.unwrap_or(false)),
                guard_by_his_operators: Set(req.guard_by_his_operators.unwrap_or(false)),
                guard_by_assigned: Set(req.guard_by_assigned.unwrap_or(false)),
                guard_by_spec_account_ids: Set(req.guard_by_spec_account_ids.as_ref().unwrap_or(&vec![]).clone()),
                guard_by_spec_role_ids: Set(req.guard_by_spec_role_ids.as_ref().unwrap_or(&vec![]).clone()),
                guard_by_spec_org_ids: Set(req.guard_by_spec_org_ids.as_ref().unwrap_or(&vec![]).clone()),
                guard_by_other_conds: Set(req.guard_by_other_conds.as_ref().map(|conds| TardisFuns::json.obj_to_json(conds).unwrap()).unwrap_or(json!([]))),

                vars_collect: Set(req.vars_collect.clone().unwrap_or_default()),
                double_check: Set(req.double_check.clone().unwrap_or_default()),
                is_notify: Set(req.is_notify.unwrap_or(true)),

                action_by_pre_callback: Set(req.action_by_pre_callback.as_ref().unwrap_or(&"".to_string()).to_string()),
                action_by_post_callback: Set(req.action_by_post_callback.as_ref().unwrap_or(&"".to_string()).to_string()),
                action_by_post_changes: Set(req.action_by_post_changes.clone().unwrap_or_default()),
                action_by_front_changes: Set(req.action_by_front_changes.clone().unwrap_or_default()),

                rel_flow_model_id: Set(flow_model_id.to_string()),
                sort: Set(req.sort.unwrap_or(0)),
                ..Default::default()
            })
            .collect_vec();
        funs.db().insert_many(flow_transitions, ctx).await
    }

    pub async fn modify_transitions(
        flow_model_id: &str,
        modify_req: &[FlowTransitionModifyReq],
        model_detail: &FlowModelDetailResp,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let flow_state_ids = modify_req
            .iter()
            .filter(|req| req.from_flow_state_id.is_some())
            .map(|req| req.from_flow_state_id.as_ref().unwrap().to_string())
            .chain(modify_req.iter().filter(|req| req.to_flow_state_id.is_some()).map(|req| req.to_flow_state_id.as_ref().unwrap().to_string()))
            .unique()
            .collect_vec();
        if modify_req.iter().any(|req| {
            if let Some(from_flow_state_id) = &req.from_flow_state_id {
                if !flow_state_ids.contains(from_flow_state_id) {
                    return true;
                }
            }
            if let Some(to_flow_state_id) = &req.to_flow_state_id {
                if !flow_state_ids.contains(to_flow_state_id) {
                    return true;
                }
            }
            false
        }) {
            return Err(funs.err().not_found(
                &Self::get_obj_name(),
                "modify_transitions",
                "the states to be added is not legal",
                "404-flow-state-add-not-legal",
            ));
        }

        let flow_transition_ids = modify_req.iter().map(|req: &FlowTransitionModifyReq| req.id.to_string()).collect_vec();
        let flow_transition_ids_lens = flow_transition_ids.len();
        if funs
            .db()
            .count(
                Query::select()
                    .column((flow_transition::Entity, flow_transition::Column::Id))
                    .from(flow_transition::Entity)
                    .and_where(Expr::col((flow_transition::Entity, flow_transition::Column::RelFlowModelId)).eq(flow_model_id.to_string()))
                    .and_where(Expr::col((flow_transition::Entity, flow_transition::Column::Id)).is_in(flow_transition_ids)),
            )
            .await? as usize
            != flow_transition_ids_lens
        {
            return Err(funs.err().not_found(
                &Self::get_obj_name(),
                "modify_transitions",
                "the transition of related models not legal",
                "404-flow-transition-rel-model-not-legal",
            ));
        }
        let model_transitions = model_detail.transitions();
        for req in modify_req {
            let transiton = model_transitions.iter().find(|trans| trans.id == req.id.to_string());
            if transiton.is_none() {
                continue;
            }

            let mut flow_transition = flow_transition::ActiveModel {
                id: Set(req.id.to_string()),
                ..Default::default()
            };
            if let Some(name) = &req.name {
                flow_transition.name = Set(name.to_string());
            }
            if let Some(from_flow_state_id) = &req.from_flow_state_id {
                flow_transition.from_flow_state_id = Set(from_flow_state_id.to_string());
            }
            if let Some(to_flow_state_id) = &req.to_flow_state_id {
                flow_transition.to_flow_state_id = Set(to_flow_state_id.to_string());
            }

            if let Some(transfer_by_auto) = req.transfer_by_auto {
                flow_transition.transfer_by_auto = Set(transfer_by_auto);
            }
            if let Some(transfer_by_timer) = &req.transfer_by_timer {
                flow_transition.transfer_by_timer = Set(transfer_by_timer.to_string());
            }

            if let Some(guard_by_creator) = req.guard_by_creator {
                flow_transition.guard_by_creator = Set(guard_by_creator);
            }
            if let Some(guard_by_his_operators) = req.guard_by_his_operators {
                flow_transition.guard_by_his_operators = Set(guard_by_his_operators);
            }
            if let Some(guard_by_assigned) = req.guard_by_assigned {
                flow_transition.guard_by_assigned = Set(guard_by_assigned);
            }
            if let Some(guard_by_spec_account_ids) = &req.guard_by_spec_account_ids {
                flow_transition.guard_by_spec_account_ids = Set(guard_by_spec_account_ids.clone());
            }
            if let Some(guard_by_spec_role_ids) = &req.guard_by_spec_role_ids {
                flow_transition.guard_by_spec_role_ids = Set(guard_by_spec_role_ids.clone());
            }
            if let Some(guard_by_spec_org_ids) = &req.guard_by_spec_org_ids {
                flow_transition.guard_by_spec_org_ids = Set(guard_by_spec_org_ids.clone());
            }
            if let Some(guard_by_other_conds) = &req.guard_by_other_conds {
                flow_transition.guard_by_other_conds = Set(TardisFuns::json.obj_to_json(guard_by_other_conds)?);
            }

            if let Some(vars_collect) = &req.vars_collect {
                flow_transition.vars_collect = Set(vars_collect.clone());
            }

            if let Some(action_by_pre_callback) = &req.action_by_pre_callback {
                flow_transition.action_by_pre_callback = Set(action_by_pre_callback.to_string());
            }
            if let Some(action_by_post_callback) = &req.action_by_post_callback {
                flow_transition.action_by_post_callback = Set(action_by_post_callback.to_string());
            }
            if let Some(action_by_front_changes) = &req.action_by_front_changes {
                flow_transition.action_by_front_changes = Set(action_by_front_changes.clone());
            }
            if let Some(action_by_post_changes) = &req.action_by_post_changes {
                flow_transition.action_by_post_changes = Set(action_by_post_changes.clone());
            }
            if let Some(action_by_post_var_changes) = &req.action_by_post_var_changes {
                let mut state_post_changes =
                    transiton.unwrap().action_by_post_changes().into_iter().filter(|post| post.kind == FlowTransitionActionChangeKind::State).collect_vec();
                let mut action_by_post_changes = action_by_post_var_changes.clone();
                action_by_post_changes.append(&mut state_post_changes);
                flow_transition.action_by_post_changes = Set(action_by_post_changes.clone());
            }
            if let Some(action_by_post_state_changes) = &req.action_by_post_state_changes {
                let mut var_post_changes = transiton.unwrap().action_by_post_changes().into_iter().filter(|post| post.kind == FlowTransitionActionChangeKind::Var).collect_vec();
                let mut action_by_post_changes = action_by_post_state_changes.clone();
                action_by_post_changes.append(&mut var_post_changes);
                flow_transition.action_by_post_changes = Set(action_by_post_changes.clone());
            }
            if let Some(double_check) = &req.double_check {
                flow_transition.double_check = Set(double_check.clone());
            }
            if let Some(is_notify) = &req.is_notify {
                flow_transition.is_notify = Set(*is_notify);
            }
            if let Some(sort) = &req.sort {
                flow_transition.sort = Set(*sort);
            }
            flow_transition.update_time = Set(Utc::now());
            funs.db().update_one(flow_transition, ctx).await?;
        }
        Ok(())
    }

    pub async fn delete_transitions(flow_model_id: &str, delete_flow_transition_ids: &Vec<String>, funs: &TardisFunsInst, _ctx: &TardisContext) -> TardisResult<()> {
        let delete_flow_transition_ids_lens = delete_flow_transition_ids.len();
        if funs
            .db()
            .count(
                Query::select()
                    .column((flow_transition::Entity, flow_transition::Column::Id))
                    .from(flow_transition::Entity)
                    .and_where(Expr::col((flow_transition::Entity, flow_transition::Column::RelFlowModelId)).eq(flow_model_id.to_string()))
                    .and_where(Expr::col((flow_transition::Entity, flow_transition::Column::Id)).is_in(delete_flow_transition_ids)),
            )
            .await? as usize
            != delete_flow_transition_ids_lens
        {
            return Err(funs.err().not_found(
                &Self::get_obj_name(),
                "delete_transitions",
                "the transition of related models not legal",
                "404-flow-transition-rel-model-not-legal",
            ));
        }
        funs.db()
            .soft_delete_custom(
                flow_transition::Entity::find().filter(Expr::col(flow_transition::Column::Id).is_in(delete_flow_transition_ids)),
                "id",
            )
            .await?;
        Ok(())
    }

    async fn find_transitions(
        flow_model_id: &str,
        specified_state_ids: Option<&[String]>,
        funs: &TardisFunsInst,
        _ctx: &TardisContext,
    ) -> TardisResult<Vec<FlowTransitionDetailResp>> {
        let from_state_rbum_table = Alias::new("from_state_rbum");
        let from_state_table = Alias::new("from_state");
        let to_state_rbum_table = Alias::new("to_state_rbum");
        let to_state_table = Alias::new("to_state");
        let mut query = Query::select();
        query
            .columns([
                (flow_transition::Entity, flow_transition::Column::Id),
                (flow_transition::Entity, flow_transition::Column::Name),
                (flow_transition::Entity, flow_transition::Column::FromFlowStateId),
                (flow_transition::Entity, flow_transition::Column::ToFlowStateId),
                (flow_transition::Entity, flow_transition::Column::TransferByAuto),
                (flow_transition::Entity, flow_transition::Column::TransferByTimer),
                (flow_transition::Entity, flow_transition::Column::GuardByCreator),
                (flow_transition::Entity, flow_transition::Column::GuardByHisOperators),
                (flow_transition::Entity, flow_transition::Column::GuardByAssigned),
                (flow_transition::Entity, flow_transition::Column::GuardBySpecAccountIds),
                (flow_transition::Entity, flow_transition::Column::GuardBySpecRoleIds),
                (flow_transition::Entity, flow_transition::Column::GuardBySpecOrgIds),
                (flow_transition::Entity, flow_transition::Column::GuardByOtherConds),
                (flow_transition::Entity, flow_transition::Column::VarsCollect),
                (flow_transition::Entity, flow_transition::Column::ActionByPreCallback),
                (flow_transition::Entity, flow_transition::Column::ActionByPostCallback),
                (flow_transition::Entity, flow_transition::Column::ActionByPostChanges),
                (flow_transition::Entity, flow_transition::Column::ActionByFrontChanges),
                (flow_transition::Entity, flow_transition::Column::DoubleCheck),
                (flow_transition::Entity, flow_transition::Column::IsNotify),
                (flow_transition::Entity, flow_transition::Column::RelFlowModelId),
                (flow_transition::Entity, flow_transition::Column::Sort),
            ])
            .expr_as(
                Expr::col((from_state_rbum_table.clone(), NAME_FIELD.clone())).if_null(""),
                Alias::new("from_flow_state_name"),
            )
            .expr_as(Expr::col((from_state_table.clone(), Alias::new("color"))).if_null(""), Alias::new("from_flow_state_color"))
            .expr_as(Expr::col((to_state_rbum_table.clone(), NAME_FIELD.clone())).if_null(""), Alias::new("to_flow_state_name"))
            .expr_as(Expr::col((to_state_table.clone(), Alias::new("color"))).if_null(""), Alias::new("to_flow_state_color"))
            .from(flow_transition::Entity)
            .join_as(
                JoinType::LeftJoin,
                RBUM_ITEM_TABLE.clone(),
                from_state_rbum_table.clone(),
                Cond::all()
                    .add(Expr::col((from_state_rbum_table.clone(), ID_FIELD.clone())).equals((flow_transition::Entity, flow_transition::Column::FromFlowStateId)))
                    .add(Expr::col((from_state_rbum_table.clone(), REL_KIND_ID_FIELD.clone())).eq(FlowStateServ::get_rbum_kind_id().unwrap()))
                    .add(Expr::col((from_state_rbum_table.clone(), REL_DOMAIN_ID_FIELD.clone())).eq(Self::get_rbum_domain_id().unwrap())),
            )
            .join_as(
                JoinType::LeftJoin,
                flow_state::Entity,
                from_state_table.clone(),
                Cond::all().add(Expr::col((from_state_table.clone(), ID_FIELD.clone())).equals((flow_transition::Entity, flow_transition::Column::FromFlowStateId))),
            )
            .join_as(
                JoinType::LeftJoin,
                RBUM_ITEM_TABLE.clone(),
                to_state_rbum_table.clone(),
                Cond::all()
                    .add(Expr::col((to_state_rbum_table.clone(), ID_FIELD.clone())).equals((flow_transition::Entity, flow_transition::Column::ToFlowStateId)))
                    .add(Expr::col((to_state_rbum_table.clone(), REL_KIND_ID_FIELD.clone())).eq(FlowStateServ::get_rbum_kind_id().unwrap()))
                    .add(Expr::col((to_state_rbum_table.clone(), REL_DOMAIN_ID_FIELD.clone())).eq(Self::get_rbum_domain_id().unwrap())),
            )
            .join_as(
                JoinType::LeftJoin,
                flow_state::Entity,
                to_state_table.clone(),
                Cond::all().add(Expr::col((to_state_table.clone(), ID_FIELD.clone())).equals((flow_transition::Entity, flow_transition::Column::ToFlowStateId))),
            )
            .and_where(Expr::col((flow_transition::Entity, flow_transition::Column::RelFlowModelId)).eq(flow_model_id));
        if let Some(specified_state_ids) = specified_state_ids {
            query.and_where(Expr::col((flow_transition::Entity, flow_transition::Column::FromFlowStateId)).is_in(specified_state_ids));
        }
        query
            .order_by((flow_transition::Entity, flow_transition::Column::Sort), Order::Asc)
            .order_by((flow_transition::Entity, flow_transition::Column::CreateTime), Order::Asc)
            .order_by((flow_transition::Entity, flow_transition::Column::Id), Order::Asc);
        let flow_transitions: Vec<FlowTransitionDetailResp> = funs.db().find_dtos(&query).await?;
        Ok(flow_transitions)
    }

    pub async fn state_is_used(flow_state_id: &str, funs: &TardisFunsInst, _ctx: &TardisContext) -> TardisResult<bool> {
        if funs
            .db()
            .count(
                Query::select().column((flow_transition::Entity, flow_transition::Column::Id)).from(flow_transition::Entity).cond_where(
                    Cond::any().add(
                        Cond::any()
                            .add(Expr::col((flow_transition::Entity, flow_transition::Column::FromFlowStateId)).eq(flow_state_id))
                            .add(Expr::col((flow_transition::Entity, flow_transition::Column::ToFlowStateId)).eq(flow_state_id)),
                    ),
                ),
            )
            .await?
            != 0
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn get_item_detail_aggs(flow_model_id: &str, is_state_detail: bool, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<FlowModelAggResp> {
        let model_detail = Self::get_item(
            flow_model_id,
            &FlowModelFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    own_paths: Some("".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;

        let mut states = Vec::new();
        if is_state_detail {
            // find rel state
            let state_ids = FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowModelState, flow_model_id, None, None, funs, ctx)
                .await?
                .iter()
                .sorted_by_key(|rel| TardisFuns::json.str_to_obj::<FlowStateRelModelExt>(&rel.ext).unwrap_or_default().sort)
                .map(|rel| {
                    (
                        rel.rel_id.clone(),
                        rel.rel_name.clone(),
                        TardisFuns::json.str_to_obj::<FlowStateRelModelExt>(&rel.ext).unwrap_or_default(),
                    )
                })
                .collect::<Vec<_>>();
            for (state_id, state_name, ext) in state_ids {
                let state_detail = FlowStateAggResp {
                    id: state_id.clone(),
                    name: state_name,
                    ext,
                    is_init: model_detail.init_state_id == state_id,
                    transitions: model_detail
                        .transitions()
                        .into_iter()
                        .filter(|transition| transition.from_flow_state_id == state_id.clone())
                        .map(|transition| {
                            let mut action_by_post_changes = vec![];
                            for action_by_post_change in transition.action_by_post_changes() {
                                if action_by_post_change.is_edit.is_none() {
                                    action_by_post_changes.push(FlowTransitionPostActionInfo {
                                        is_edit: Some(false), // 默认为不可编辑，若用户需要编辑，可手动处理数据
                                        ..action_by_post_change.clone()
                                    });
                                } else {
                                    action_by_post_changes.push(FlowTransitionPostActionInfo { ..action_by_post_change.clone() });
                                }
                            }
                            FlowTransitionDetailResp {
                                action_by_post_changes: TardisFuns::json.obj_to_json(&action_by_post_changes).unwrap_or_default(),
                                ..transition.clone()
                            }
                        })
                        .collect_vec(),
                };
                states.push(state_detail);
            }
        }

        Ok(FlowModelAggResp {
            id: model_detail.id.clone(),
            name: model_detail.name,
            icon: model_detail.icon,
            info: model_detail.info,
            init_state_id: model_detail.init_state_id,
            template: model_detail.template,
            rel_model_id: model_detail.rel_model_id,
            rel_template_ids: FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowModelTemplate, &model_detail.id, None, None, funs, ctx)
                .await?
                .into_iter()
                .map(|rel| rel.rel_id)
                .collect_vec(),
            states,
            own_paths: model_detail.own_paths,
            owner: model_detail.owner,
            create_time: model_detail.create_time,
            update_time: model_detail.update_time,
            tag: model_detail.tag,
            scope_level: model_detail.scope_level,
            disabled: model_detail.disabled,
        })
    }

    // Find the specified models, or add it if it doesn't exist.
    pub async fn find_or_add_models(
        tags: Vec<String>,
        template_id: Option<String>,
        is_shared: bool,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<HashMap<String, FlowModelSummaryResp>> {
        let mut result = Self::find_rel_models(template_id.clone(), is_shared, funs, ctx).await?;
        // Iterate over the tag based on the existing result and get the default model
        for tag in tags {
            if !result.contains_key(&tag) {
                // copy custom model
                let model_id = Self::add_custom_model(&tag, None, template_id.clone(), funs, ctx).await?;
                let added_model = Self::find_one_item(
                    &FlowModelFilterReq {
                        basic: RbumBasicFilterReq {
                            ids: Some(vec![model_id]),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    funs,
                    ctx,
                )
                .await?
                .unwrap_or_default();
                result.insert(tag.to_string(), added_model);
            }
        }

        Ok(result)
    }

    // Find the rel models.
    pub async fn find_rel_models(template_id: Option<String>, _is_shared: bool, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<HashMap<String, FlowModelSummaryResp>> {
        let global_ctx = TardisContext {
            own_paths: "".to_string(),
            ..ctx.clone()
        };
        let mut result = HashMap::new();

        let filter_ids = if template_id.is_none() {
            if let Some(app_id) = Self::get_app_id_by_ctx(ctx) {
                Some(FlowRelServ::find_model_ids_by_app_id(&app_id, funs, ctx).await.unwrap_or_default())
            } else {
                Some(vec![])
            }
        } else {
            None
        };
        let mut filter = FlowModelFilterReq {
            basic: RbumBasicFilterReq {
                ids: filter_ids,
                ignore_scope: true,
                with_sub_own_paths: true,
                ..Default::default()
            },
            rel: FlowRelServ::get_template_rel_filter(template_id.as_deref()),
            ..Default::default()
        };
        let mut models = Self::find_items(&filter, None, None, funs, &global_ctx).await?;
        if models.is_empty() {
            filter.basic.ids = None;
            models = Self::find_items(&filter, None, None, funs, ctx).await?;
        }

        // First iterate over the models
        for model in models {
            result.insert(model.tag.clone(), model);
        }

        Ok(result)
    }

    /// 创建或引用模型
    /// params:
    /// rel_model_id：关联模型ID
    /// rel_template_id: 绑定模板ID,可选参数（仅在创建模型，即创建副本或op为复制时生效）
    /// rel_own_paths: 绑定实例ID（仅在引用且不创建模型时生效）
    /// （rel_model_id：关联模型ID, rel_template_id: 绑定模板ID,可选参数（仅在创建模型，即创建副本或op为复制时生效）, op：关联模型操作类型（复制或者引用），is_create_copy：是否创建副本（当op为复制时需指定，默认不需要））
    pub async fn copy_or_reference_model(
        rel_model_id: &str,
        rel_own_paths: Option<String>,
        op: &FlowModelAssociativeOperationKind,
        is_create_copy: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<FlowModelAggResp> {
        let mock_ctx = if let Some(own_paths) = rel_own_paths.clone() {
            TardisContext { own_paths, ..ctx.clone() }
        } else {
            ctx.clone()
        };
        let rel_model = FlowModelServ::get_item(
            rel_model_id,
            &FlowModelFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ignore_scope: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        // .ok_or_else(|| funs.err().not_found(&Self::get_obj_name(), "copy_or_reference_model", "rel model not found", "404-flow-model-not-found"))?;
        let result = match op {
            FlowModelAssociativeOperationKind::Reference => {
                if is_create_copy.unwrap_or(false) {
                    let mut add_transitions = rel_model.transitions().into_iter().map(FlowTransitionAddReq::from).collect_vec();
                    for add_transition in add_transitions.iter_mut() {
                        if let Some(ref mut action_by_post_changes) = &mut add_transition.action_by_post_changes {
                            for action_by_post_change in action_by_post_changes.iter_mut() {
                                action_by_post_change.is_edit = Some(false); // 引用复制时，置为不可编辑
                            }
                        }
                    }
                    Self::add_item(
                        &mut FlowModelAddReq {
                            rel_model_id: Some(rel_model_id.to_string()),
                            rel_template_ids: None,
                            transitions: Some(add_transitions),
                            ..rel_model.clone().into()
                        },
                        funs,
                        &mock_ctx,
                    )
                    .await?
                } else {
                    if let Some(template_id) =
                        FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowModelTemplate, rel_model_id, None, None, funs, ctx).await?.pop().map(|rel| rel.rel_id)
                    {
                        if !FlowRelServ::exist_rels(
                            &FlowRelKind::FlowAppTemplate,
                            &template_id,
                            Self::get_app_id_by_ctx(ctx).unwrap_or_default().as_str(),
                            funs,
                            ctx,
                        )
                        .await?
                        {
                            FlowRelServ::add_simple_rel(
                                &FlowRelKind::FlowAppTemplate,
                                Self::get_app_id_by_ctx(&mock_ctx).unwrap_or_default().as_str(),
                                &template_id,
                                None,
                                None,
                                true,
                                true,
                                None,
                                funs,
                                ctx,
                            )
                            .await?;
                        }
                    }
                    rel_model_id.to_string()
                }
            }
            FlowModelAssociativeOperationKind::Copy => {
                Self::add_item(
                    &mut FlowModelAddReq {
                        rel_model_id: if rbum_scope_helper::get_scope_level_by_context(&mock_ctx)? != RbumScopeLevelKind::L2 {
                            Some(rel_model_id.to_string())
                        } else {
                            None
                        },
                        rel_template_ids: None,
                        template: rbum_scope_helper::get_scope_level_by_context(&mock_ctx)? != RbumScopeLevelKind::L2,
                        scope_level: if rbum_scope_helper::get_scope_level_by_context(&mock_ctx)? != RbumScopeLevelKind::L2 {
                            Some(rel_model.clone().scope_level)
                        } else {
                            None
                        },
                        ..rel_model.clone().into()
                    },
                    funs,
                    &mock_ctx,
                )
                .await?
            }
        };
        let new_model = Self::get_item_detail_aggs(&result, true, funs, ctx).await?;

        Ok(new_model)
    }

    // copy model by template model
    // rel_template_id: Associated parent template id
    // current_template_id: Current template id
    pub async fn add_custom_model(
        tag: &str,
        parent_template_id: Option<String>,
        rel_template_id: Option<String>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<String> {
        let current_model = Self::find_one_detail_item(
            &FlowModelFilterReq {
                basic: RbumBasicFilterReq {
                    ignore_scope: true,
                    ..Default::default()
                },
                tags: Some(vec![tag.to_string()]),
                rel: FlowRelServ::get_template_rel_filter(rel_template_id.as_deref()),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        if let Some(current_model) = current_model {
            return Ok(current_model.id);
        }

        let global_ctx = TardisContext {
            own_paths: "".to_string(),
            ..ctx.clone()
        };
        // First, get the parent model, if the parent model does not exist, then get the default template
        // 首先，获取父级model，若父级model不存在，则获取默认模板
        let parent_model = if let Some(parent_model) = Self::find_one_detail_item(
            // There are shared templates, so you need to ignore the permission judgment of own_path if the parent ID is passed in.
            // 由于存在共享模板的情况，所以父级ID传入的情况下需要忽略 own_path 的权限判断
            &FlowModelFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: parent_template_id.is_some(),
                    ignore_scope: parent_template_id.is_some(),
                    ..Default::default()
                },
                tags: Some(vec![tag.to_string()]),
                // When no parent ID is passed, indicating that the default template is directly obtained, parent_template_id is passed into the empty string
                // 没有传入父级ID时，说明直接获取默认模板，则 parent_template_id 传入空字符串
                rel: FlowRelServ::get_template_rel_filter(Some(&parent_template_id.unwrap_or_default())),
                template: Some(true),
                ..Default::default()
            },
            funs,
            &global_ctx,
        )
        .await?
        {
            parent_model
        } else {
            Self::find_one_detail_item(
                &FlowModelFilterReq {
                    basic: RbumBasicFilterReq {
                        ignore_scope: true,
                        ..Default::default()
                    },
                    tags: Some(vec![tag.to_string()]),
                    ..Default::default()
                },
                funs,
                &global_ctx,
            )
            .await?
            .ok_or_else(|| funs.err().internal_error("flow_model_serv", "add_custom_model", "default model is not exist", "404-flow-model-not-found"))?
        };

        // add model
        let model_id = Self::add_item(
            &mut FlowModelAddReq {
                name: parent_model.name.clone().into(),
                icon: Some(parent_model.icon.clone()),
                info: Some(parent_model.info.clone()),
                init_state_id: parent_model.init_state_id.clone(),
                template: rel_template_id.is_some(),
                rel_template_ids: rel_template_id.clone().map(|id| vec![id]),
                transitions: Some(parent_model.transitions().into_iter().map(|trans| trans.into()).collect_vec()),
                states: Some(
                    FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowModelState, &parent_model.id, None, None, funs, &global_ctx)
                        .await?
                        .iter()
                        .sorted_by_key(|rel| TardisFuns::json.str_to_obj::<FlowStateRelModelExt>(&rel.ext).unwrap_or_default().sort)
                        .map(|rel| FlowModelBindStateReq {
                            state_id: rel.rel_id.clone(),
                            ext: TardisFuns::json.str_to_obj::<FlowStateRelModelExt>(&rel.ext).unwrap_or_default(),
                        })
                        .collect_vec(),
                ),
                rel_model_id: Some(parent_model.id.clone()),
                tag: Some(parent_model.tag.clone()),
                scope_level: if rel_template_id.is_some() { Some(RbumScopeLevelKind::Root) } else { None },
                disabled: Some(parent_model.disabled),
            },
            funs,
            ctx,
        )
        .await?;

        Ok(model_id)
    }

    // add or modify model by own_paths
    pub async fn modify_model(flow_model_id: &str, modify_req: &mut FlowModelModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let current_model = Self::get_item(
            flow_model_id,
            &FlowModelFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;

        Self::modify_item(&current_model.id, modify_req, funs, ctx).await?;

        Ok(())
    }

    async fn bind_state(flow_model_id: &str, req: &FlowModelBindStateReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let global_ctx = TardisContext {
            own_paths: "".to_string(),
            ..ctx.clone()
        };
        if let Ok(state) = FlowStateServ::get_item(
            &req.state_id,
            &FlowStateFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            &global_ctx,
        )
        .await
        {
            let model_detail = Self::get_item(flow_model_id, &FlowModelFilterReq::default(), funs, ctx).await?;
            if !state.tags.is_empty() && !state.tags.split(',').collect_vec().contains(&model_detail.tag.as_str()) {
                return Err(funs.err().internal_error("flow_model_serv", "bind_state", "The flow state is not found", "404-flow-state-not-found"));
            }
        } else {
            return Err(funs.err().internal_error("flow_model_serv", "bind_state", "The flow state is not found", "404-flow-state-not-found"));
        }
        FlowRelServ::add_simple_rel(
            &FlowRelKind::FlowModelState,
            flow_model_id,
            &req.state_id,
            None,
            None,
            false,
            true,
            Some(json!(req.ext).to_string()),
            funs,
            ctx,
        )
        .await?;

        Ok(())
    }

    pub async fn unbind_state(flow_model_id: &str, state_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        // Can only be deleted when not in use
        if FlowInstServ::state_is_used(flow_model_id, state_id, funs, ctx).await? {
            return Err(funs.err().conflict(
                &Self::get_obj_name(),
                "unbind_state",
                &format!("state {state_id} already used"),
                "409-flow-state-already-used",
            ));
        }
        //delete rel transitions
        let trans_ids = Self::find_transitions_by_state_id(flow_model_id, Some(vec![state_id.to_string()]), None, funs, ctx).await?.into_iter().map(|trans| trans.id).collect_vec();
        Self::delete_transitions(flow_model_id, &trans_ids, funs, ctx).await?;
        let trans_ids = Self::find_transitions_by_state_id(flow_model_id, None, Some(vec![state_id.to_string()]), funs, ctx).await?.into_iter().map(|trans| trans.id).collect_vec();
        Self::delete_transitions(flow_model_id, &trans_ids, funs, ctx).await?;

        FlowRelServ::delete_simple_rel(&FlowRelKind::FlowModelState, flow_model_id, state_id, funs, ctx).await?;

        Ok(())
    }

    pub async fn modify_rel_state_ext(flow_model_id: &str, modify_req: &FlowStateRelModelModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let mut ext = TardisFuns::json.str_to_obj::<FlowStateRelModelExt>(
            &FlowRelServ::find_simple_rels(&FlowRelKind::FlowModelState, Some(flow_model_id), Some(modify_req.id.as_str()), true, None, None, funs, ctx)
                .await?
                .pop()
                .ok_or_else(|| funs.err().internal_error("flow_model_serv", "modify_rel_state", "rel not found", "404-rel-not-found"))?
                .ext,
        )?;
        if let Some(sort) = modify_req.sort {
            ext.sort = sort;
        }
        if let Some(show_btns) = modify_req.show_btns.clone() {
            ext.show_btns = Some(show_btns);
        }
        FlowRelServ::modify_simple_rel(
            &FlowRelKind::FlowModelState,
            flow_model_id,
            &modify_req.id,
            &mut RbumRelModifyReq {
                tag: None,
                note: None,
                ext: Some(json!(ext).to_string()),
            },
            funs,
            ctx,
        )
        .await?;

        Ok(())
    }

    async fn find_transitions_by_state_id(
        flow_model_id: &str,
        current_state_id: Option<Vec<String>>,
        target_state_id: Option<Vec<String>>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<FlowTransitionDetailResp>> {
        Ok(Self::find_transitions(flow_model_id, None, funs, ctx)
            .await?
            .into_iter()
            .filter(|tran_detail| {
                if let Some(target_state_id) = target_state_id.as_ref() {
                    target_state_id.contains(&tran_detail.to_flow_state_id)
                } else {
                    true
                }
            })
            .filter(|tran_detail| {
                if let Some(current_state_id) = current_state_id.as_ref() {
                    current_state_id.contains(&tran_detail.from_flow_state_id)
                } else {
                    true
                }
            })
            .collect_vec())
    }

    // 1、template_id为None，则根据own_paths批量获取关联模型
    // 2、template_id为单元素数组，则表示
    pub async fn check_post_action_ring(model_desp: &FlowModelDetailResp, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
        let mut model_details = HashMap::new();
        if model_desp.template {
            model_details.insert(model_desp.tag.clone(), model_desp.clone());
        } else {
            let template_ids = if model_desp.rel_template_ids.is_empty() {
                None
            } else {
                model_desp.rel_template_ids.clone().pop()
            };
            let models = Self::find_rel_models(template_ids, false, funs, ctx).await?;

            for (tag, model) in models {
                let model_detail = Self::find_one_detail_item(
                    &FlowModelFilterReq {
                        basic: RbumBasicFilterReq {
                            ids: Some(vec![model.id]),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    funs,
                    ctx,
                )
                .await?
                .unwrap();
                model_details.insert(tag, model_detail);
            }
        }

        // Ok(!loop_check_helper::check(&model_details))
        Ok(false)
    }

    pub async fn find_rel_states(tags: Vec<&str>, rel_template_id: Option<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<FlowModelFindRelStateResp>> {
        let mut result = Vec::new();
        for tag in tags {
            let flow_model_id = Self::get_model_id_by_own_paths_and_rel_template_id(tag, rel_template_id.clone(), funs, ctx).await?;
            let mut states = Self::find_sorted_rel_states_by_model_id(&flow_model_id, funs, ctx)
                .await?
                .into_iter()
                .map(|state_detail| FlowModelFindRelStateResp {
                    id: state_detail.id.clone(),
                    name: state_detail.name.clone(),
                    color: state_detail.color.clone(),
                })
                .collect_vec();
            result.append(&mut states);
        }
        Ok(result)
    }

    async fn find_sorted_rel_states_by_model_id(flow_model_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<FlowStateDetailResp>> {
        Ok(join_all(
            FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowModelState, flow_model_id, None, None, funs, ctx)
                .await?
                .into_iter()
                .sorted_by_key(|rel| TardisFuns::json.str_to_obj::<FlowStateRelModelExt>(&rel.ext).unwrap_or_default().sort)
                .map(|rel| async move {
                    FlowStateServ::find_one_detail_item(
                        &FlowStateFilterReq {
                            basic: RbumBasicFilterReq {
                                ids: Some(vec![rel.rel_id]),
                                with_sub_own_paths: true,
                                own_paths: Some("".to_string()),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        funs,
                        ctx,
                    )
                    .await
                    .unwrap_or_default()
                    .unwrap()
                })
                .collect::<Vec<_>>(),
        )
        .await)
    }

    pub async fn get_model_id_by_own_paths_and_rel_template_id(tag: &str, rel_template_id: Option<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let mut own_paths = ctx.own_paths.clone();
        let mut scope_level = rbum_scope_helper::get_scope_level_by_context(ctx)?.to_int();

        let mut result = None;
        // Prioritize confirming the existence of mods related to own_paths
        if let Some(rel_model) = Self::find_items(
            &FlowModelFilterReq {
                basic: RbumBasicFilterReq {
                    ids: Some(FlowRelServ::find_model_ids_by_app_id(Self::get_app_id_by_ctx(ctx).unwrap_or_default().as_str(), funs, ctx).await.unwrap_or_default()),
                    ignore_scope: true,
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?
        .into_iter()
        .find(|rel_model| rel_model.tag.as_str() == tag)
        {
            return Ok(rel_model.id);
        }
        // try get model in tenant path or app path
        while !own_paths.is_empty() {
            result = FlowModelServ::find_one_item(
                &FlowModelFilterReq {
                    basic: RbumBasicFilterReq {
                        own_paths: Some(own_paths.clone()),
                        ignore_scope: true,
                        ..Default::default()
                    },
                    tags: Some(vec![tag.to_string()]),
                    template: Some(rel_template_id.is_some()),
                    rel: FlowRelServ::get_template_rel_filter(rel_template_id.as_deref()),
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await
            .unwrap_or_default();
            if result.is_some() {
                break;
            } else {
                own_paths = rbum_scope_helper::get_path_item(scope_level, &ctx.own_paths).unwrap_or_default();
                scope_level -= 1;
            }
        }
        if result.is_none() {
            result = FlowModelServ::find_one_item(
                &FlowModelFilterReq {
                    basic: RbumBasicFilterReq {
                        own_paths: Some("".to_string()),
                        ignore_scope: true,
                        ..Default::default()
                    },
                    tags: Some(vec![tag.to_string()]),
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?;
        }
        match result {
            Some(model) => Ok(model.id),
            None => Err(funs.err().not_found("flow_inst_serv", "get_model_id_by_own_paths", "model not found", "404-flow-model-not-found")),
        }
    }

    pub async fn find_models_by_rel_template_id(
        tag: String,
        template: Option<bool>,
        rel_template_id: Option<String>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<FlowModelSummaryResp>> {
        let mut result = vec![];
        let mut not_bind_template_models = join_all(
            FlowModelServ::find_items(
                &FlowModelFilterReq {
                    basic: RbumBasicFilterReq {
                        with_sub_own_paths: false,
                        ..Default::default()
                    },
                    tags: Some(vec![tag.clone()]),
                    template,
                    rel_model_ids: Some(vec!["".to_string()]), // rel_model_id is empty and template is true, which means it is a workflow template.
                    ..Default::default()
                },
                Some(true),
                None,
                funs,
                ctx,
            )
            .await?
            .into_iter()
            .map(|model| async move {
                let funs = flow_constants::get_tardis_inst();
                let global_ctx: TardisContext = TardisContext::default();
                if FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowModelTemplate, &model.id, None, None, &funs, &global_ctx).await.unwrap().is_empty() {
                    Some(model)
                } else {
                    None
                }
            }),
        )
        .await
        .into_iter()
        .flatten()
        .collect_vec();
        result.append(&mut not_bind_template_models);
        if let Some(rel_template_id) = rel_template_id {
            let mut rel_template_models = FlowModelServ::find_items(
                &FlowModelFilterReq {
                    basic: RbumBasicFilterReq {
                        with_sub_own_paths: false,
                        ..Default::default()
                    },
                    tags: Some(vec![tag.clone()]),
                    template,
                    rel_model_ids: Some(vec!["".to_string()]), // rel_model_id is empty and template is true, which means it is a workflow template.
                    rel: Some(RbumItemRelFilterReq {
                        optional: false,
                        rel_by_from: true,
                        tag: Some(FlowRelKind::FlowModelTemplate.to_string()),
                        from_rbum_kind: Some(RbumRelFromKind::Item),
                        rel_item_id: Some(rel_template_id),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                Some(true),
                None,
                funs,
                ctx,
            )
            .await?;
            result.append(&mut rel_template_models);
        }

        Ok(result.into_iter().filter(|model| !model.init_state_id.is_empty()).collect_vec())
    }

    // 清除当前关联的模型数据（用于更新配置）
    /**
     * 当rel_template_id为空时：
     * 1、去除ModelPath引用关系
     * 2、删除当前own_path下的model
     * 当rel_template_id不为空时：
     * 1、去除ModelTemplate引用关系
     * 2、去除ModelPath引用关系
     * 3、删除当前rel_template_id下的model
     */
    pub async fn clean_rel_models(
        rel_template_id: Option<String>,
        orginal_model_ids: Option<Vec<String>>,
        spec_tags: Option<Vec<String>>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<HashMap<String, FlowModelSummaryResp>> {
        let global_ctx = TardisContext {
            own_paths: "".to_string(),
            ..ctx.clone()
        };
        let models = Self::find_rel_models(rel_template_id.clone(), false, funs, ctx).await?;
        for (tag, model) in models.iter() {
            if let Some(spec_tags) = spec_tags.clone() {
                if !spec_tags.contains(tag) {
                    continue;
                }
            }
            if let Some(orginal_model_ids) = orginal_model_ids.clone() {
                if orginal_model_ids.contains(&model.id) {
                    continue;
                }
            }
            if rel_template_id.clone().is_some() {
                for rel in FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowModelTemplate, &model.id, None, None, funs, &global_ctx).await? {
                    FlowRelServ::delete_simple_rel(&FlowRelKind::FlowModelTemplate, &model.id, &rel.rel_id, funs, &global_ctx).await?;
                }
            } else {
                // clean reference template rel
                for rel in FlowRelServ::find_from_simple_rels(
                    &FlowRelKind::FlowAppTemplate,
                    Self::get_app_id_by_ctx(ctx).unwrap_or_default().as_str(),
                    None,
                    None,
                    funs,
                    &global_ctx,
                )
                .await?
                .into_iter()
                {
                    FlowRelServ::delete_simple_rel(
                        &FlowRelKind::FlowAppTemplate,
                        Self::get_app_id_by_ctx(ctx).unwrap_or_default().as_str(),
                        &rel.rel_id,
                        funs,
                        &global_ctx,
                    )
                    .await?;
                }
            }
            if ctx.own_paths == model.own_paths {
                Self::delete_item(&model.id, funs, ctx).await?;
            }
        }
        Ok(models)
    }

    fn get_app_id_by_ctx(ctx: &TardisContext) -> Option<String> {
        rbum_scope_helper::get_max_level_id_by_context(ctx)
    }
}
