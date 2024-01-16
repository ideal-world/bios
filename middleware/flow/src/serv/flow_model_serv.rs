use std::collections::HashMap;

use async_recursion::async_recursion;
use bios_basic::rbum::{
    dto::{
        rbum_filer_dto::RbumBasicFilterReq,
        rbum_item_dto::{RbumItemKernelAddReq, RbumItemKernelModifyReq},
        rbum_rel_dto::RbumRelModifyReq,
    },
    rbum_enumeration::RbumScopeLevelKind,
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
    serde_json::json,
    web::web_resp::TardisPage,
    TardisFuns, TardisFunsInst,
};

use crate::{
    domain::{flow_model, flow_state, flow_transition},
    dto::{
        flow_model_dto::{
            FlowModelAddReq, FlowModelAggResp, FlowModelBindStateReq, FlowModelDetailResp, FlowModelFilterReq, FlowModelFindRelStateResp, FlowModelModifyReq,
            FlowModelSortStatesReq, FlowModelSummaryResp, FlowModelUnbindStateReq, FlowStateAggResp, FlowTemplateModelResp,
        },
        flow_state_dto::{FlowStateAddReq, FlowStateFilterReq, FlowStateRelModelExt, FlowSysStateKind},
        flow_transition_dto::{
            FlowTransitionActionChangeAgg, FlowTransitionActionChangeKind, FlowTransitionAddReq, FlowTransitionDetailResp, FlowTransitionDoubleCheckInfo, FlowTransitionInitInfo,
            FlowTransitionModifyReq, FlowTransitionSortStatesReq,
        },
    },
    flow_config::FlowBasicInfoManager,
    serv::flow_state_serv::FlowStateServ,
};
use async_trait::async_trait;

use super::{
    flow_external_serv::FlowExternalServ,
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
            rel_template_id: Set(add_req.rel_template_id.as_ref().unwrap_or(&"".to_string()).to_string()),
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
        if let Some(transitions) = &add_req.transitions {
            Self::add_transitions(flow_model_id, transitions, funs, ctx).await?;
            // check transition post action endless loop
            for transition_detail in Self::get_item(flow_model_id, &FlowModelFilterReq::default(), funs, ctx).await?.transitions() {
                if Self::check_post_action_ring(transition_detail, (false, vec![]), funs, ctx).await?.0 {
                    return Err(funs.err().not_found(
                        "flow_model_Serv",
                        "after_add_item",
                        "this post action exist endless loop",
                        "500-flow-transition-endless-loop",
                    ));
                }
            }
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
        if modify_req.icon.is_none() && modify_req.info.is_none() && modify_req.init_state_id.is_none() && modify_req.tag.is_none() {
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
        Ok(Some(flow_model))
    }

    async fn after_modify_item(flow_model_id: &str, modify_req: &mut FlowModelModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if let Some(add_transitions) = &modify_req.add_transitions {
            Self::add_transitions(flow_model_id, add_transitions, funs, ctx).await?;
        }
        if let Some(modify_transitions) = &modify_req.modify_transitions {
            Self::modify_transitions(flow_model_id, modify_transitions, funs, ctx).await?;
        }
        if let Some(delete_transitions) = &modify_req.delete_transitions {
            Self::delete_transitions(flow_model_id, delete_transitions, funs, ctx).await?;
        }

        if modify_req.add_transitions.is_some() || modify_req.modify_transitions.is_some() {
            // check transition post action endless loop
            for transition_detail in Self::get_item(flow_model_id, &FlowModelFilterReq::default(), funs, ctx).await?.transitions() {
                if Self::check_post_action_ring(transition_detail, (false, vec![]), funs, ctx).await?.0 {
                    return Err(funs.err().not_found(
                        "flow_model_Serv",
                        "after_modify_item",
                        "this post action exist endless loop",
                        "500-flow-transition-endless-loop",
                    ));
                }
            }
        }

        Ok(())
    }

    async fn package_ext_query(query: &mut SelectStatement, _: bool, filter: &FlowModelFilterReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<()> {
        query.column((flow_model::Entity, flow_model::Column::Icon));
        query.column((flow_model::Entity, flow_model::Column::Info));
        query.column((flow_model::Entity, flow_model::Column::InitStateId));
        query.column((flow_model::Entity, flow_model::Column::Tag));
        query.column((flow_model::Entity, flow_model::Column::RelTemplateId));
        query.expr_as(Expr::val(json! {()}), Alias::new("transitions"));
        if let Some(tags) = filter.tags.clone() {
            query.and_where(Expr::col(flow_model::Column::Tag).is_in(tags));
        }
        if let Some(rel_template_id) = filter.rel_template_id.clone() {
            query.and_where(Expr::col(flow_model::Column::RelTemplateId).eq(rel_template_id));
        }
        if let Some(template) = filter.template {
            query.and_where(Expr::col(flow_model::Column::Template).eq(template));
        }
        if let Some(own_paths) = filter.own_paths.clone() {
            query.and_where(Expr::col((flow_model::Entity, flow_model::Column::OwnPaths)).is_in(own_paths));
        }

        Ok(())
    }

    async fn get_item(flow_model_id: &str, filter: &FlowModelFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<FlowModelDetailResp> {
        let mut flow_model = Self::do_get_item(flow_model_id, filter, funs, ctx).await?;
        let flow_transitions = Self::find_transitions(flow_model_id, filter.specified_state_ids.as_deref(), funs, ctx).await?;
        flow_model.transitions = Some(TardisFuns::json.obj_to_json(&flow_transitions)?);
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
        states: Vec<(&str, FlowSysStateKind, &str)>,
        model_name: &str,
        transitions: Vec<FlowTransitionInitInfo>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let mut states_map = HashMap::new();
        let mut init_state_id = "".to_string();
        for (state_name, sys_state, color) in states.clone() {
            let state_id = FlowStateServ::add_item(
                &mut FlowStateAddReq {
                    id_prefix: None,
                    name: Some(state_name.into()),
                    icon: None,
                    color: Some(color.to_string()),
                    sys_state,
                    info: None,
                    state_kind: None,
                    kind_conf: None,
                    template: None,
                    rel_state_id: None,
                    tags: Some(vec![tag.to_string()]),
                    scope_level: Some(RbumScopeLevelKind::Root),
                    disabled: None,
                },
                funs,
                ctx,
            )
            .await?;
            if init_state_id.is_empty() {
                init_state_id = state_id.clone();
            }
            states_map.insert(state_name, state_id);
        }
        // add model
        let model_id = Self::add_item(
            &mut FlowModelAddReq {
                name: model_name.into(),
                init_state_id: init_state_id.clone(),
                rel_template_id: None,
                icon: None,
                info: None,
                transitions: None,
                tag: Some(tag.to_string()),
                scope_level: None,
                disabled: None,
                template: true,
                rel_model_id: None,
            },
            funs,
            ctx,
        )
        .await?;

        // add rel
        for (i, (state_name, _, _)) in states.iter().enumerate() {
            let ext = FlowStateRelModelExt {
                sort: i as i64,
                ..Default::default()
            };
            FlowRelServ::add_simple_rel(
                &FlowRelKind::FlowModelState,
                &model_id,
                states_map.get(state_name).ok_or_else(|| funs.err().internal_error("flow_model_serv", "init_model", "to_flow_state_name is illegal", ""))?,
                None,
                None,
                false,
                false,
                Some(json!(ext).to_string()),
                funs,
                ctx,
            )
            .await?;
        }
        let mut add_transitions = vec![];
        for transition in transitions {
            add_transitions.push(FlowTransitionAddReq {
                from_flow_state_id: states_map
                    .get(transition.from_flow_state_name.as_str())
                    .ok_or_else(|| funs.err().internal_error("flow_model_serv", "init_model", "from_flow_state_name is illegal", ""))?
                    .to_string(),
                to_flow_state_id: states_map
                    .get(transition.to_flow_state_name.as_str())
                    .ok_or_else(|| funs.err().internal_error("flow_model_serv", "init_model", "to_flow_state_name is illegal", ""))?
                    .to_string(),
                name: Some(transition.name.into()),
                transfer_by_auto: transition.transfer_by_auto,
                transfer_by_timer: transition.transfer_by_timer,
                guard_by_creator: transition.guard_by_creator,
                guard_by_his_operators: transition.guard_by_his_operators,
                guard_by_assigned: transition.guard_by_assigned,
                guard_by_spec_account_ids: transition.guard_by_spec_account_ids,
                guard_by_spec_role_ids: transition.guard_by_spec_role_ids,
                guard_by_spec_org_ids: transition.guard_by_spec_org_ids,
                guard_by_other_conds: transition.guard_by_other_conds,
                vars_collect: transition.vars_collect,
                action_by_pre_callback: transition.action_by_pre_callback,
                action_by_post_callback: transition.action_by_post_callback,
                action_by_post_changes: Some(transition.action_by_post_changes),
                action_by_front_changes: Some(transition.action_by_front_changes),
                double_check: transition.double_check,
                is_notify: Some(true),
                sort: transition.sort,
            });
        }
        Self::add_transitions(&model_id, &add_transitions, funs, ctx).await?;

        Ok(())
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

                vars_collect: Set(req.vars_collect.as_ref().map(|vars| TardisFuns::json.obj_to_json(vars).unwrap()).unwrap_or(json!([]))),
                double_check: Set(TardisFuns::json.obj_to_json(&req.double_check).unwrap_or(json!(FlowTransitionDoubleCheckInfo::default()))),
                is_notify: Set(req.is_notify.unwrap_or(true)),

                action_by_pre_callback: Set(req.action_by_pre_callback.as_ref().unwrap_or(&"".to_string()).to_string()),
                action_by_post_callback: Set(req.action_by_post_callback.as_ref().unwrap_or(&"".to_string()).to_string()),
                action_by_post_changes: Set(TardisFuns::json.obj_to_json(&req.action_by_post_changes).unwrap_or(json!([]))),
                action_by_front_changes: Set(TardisFuns::json.obj_to_json(&req.action_by_front_changes).unwrap_or(json!([]))),

                rel_flow_model_id: Set(flow_model_id.to_string()),
                sort: Set(req.sort.unwrap_or(0)),
                ..Default::default()
            })
            .collect_vec();
        funs.db().insert_many(flow_transitions, ctx).await
    }

    pub async fn modify_transitions(flow_model_id: &str, modify_req: &[FlowTransitionModifyReq], funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
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

        for req in modify_req {
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
                flow_transition.vars_collect = Set(TardisFuns::json.obj_to_json(vars_collect)?);
            }

            if let Some(action_by_pre_callback) = &req.action_by_pre_callback {
                flow_transition.action_by_pre_callback = Set(action_by_pre_callback.to_string());
            }
            if let Some(action_by_post_callback) = &req.action_by_post_callback {
                flow_transition.action_by_post_callback = Set(action_by_post_callback.to_string());
            }
            if let Some(action_by_front_changes) = &req.action_by_front_changes {
                flow_transition.action_by_front_changes = Set(TardisFuns::json.obj_to_json(action_by_front_changes)?);
            }
            if let Some(action_by_post_changes) = &req.action_by_post_changes {
                flow_transition.action_by_post_changes = Set(TardisFuns::json.obj_to_json(action_by_post_changes)?);
            }
            if let Some(double_check) = &req.double_check {
                flow_transition.double_check = Set(TardisFuns::json.obj_to_json(double_check)?);
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
                Cond::all().add(Expr::col((to_state_table.clone(), ID_FIELD.clone())).equals((flow_transition::Entity, flow_transition::Column::FromFlowStateId))),
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

    pub async fn get_item_detail_aggs(flow_model_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<FlowModelAggResp> {
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
        let mut states = Vec::new();
        for (state_id, state_name, ext) in state_ids {
            let state_detail = FlowStateAggResp {
                id: state_id.clone(),
                name: state_name,
                ext,
                is_init: model_detail.init_state_id == state_id,
                transitions: model_detail.transitions().into_iter().filter(|transition| transition.from_flow_state_id == state_id.clone()).collect_vec(),
            };
            states.push(state_detail);
        }

        Ok(FlowModelAggResp {
            id: model_detail.id,
            name: model_detail.name,
            icon: model_detail.icon,
            info: model_detail.info,
            init_state_id: model_detail.init_state_id,
            rel_template_id: model_detail.rel_template_id,
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

    // Find model by tag and template id
    pub async fn get_models(tags: Vec<&str>, template_id: Option<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<HashMap<String, FlowTemplateModelResp>> {
        let global_ctx = TardisContext {
            own_paths: "".to_string(),
            ..ctx.clone()
        };
        let mut result = HashMap::new();

        let models = if let Some(template_id) = &template_id {
            // Since the default template is not bound to model, you can use global_ctx to find the association through the template_id
            // 因为默认模板没有绑定模型，所以通过template_id查找模型可以使用global_ctx
            FlowModelServ::paginate_items(
                &FlowModelFilterReq {
                    basic: RbumBasicFilterReq {
                        ignore_scope: true,
                        with_sub_own_paths: true,
                        ..Default::default()
                    },
                    rel_template_id: Some(template_id.clone()),
                    ..Default::default()
                },
                1,
                20,
                None,
                None,
                funs,
                &global_ctx,
            )
            .await?
        } else {
            // If no template_id is passed, the real own_paths are used
            FlowModelServ::paginate_items(
                &FlowModelFilterReq {
                    basic: RbumBasicFilterReq {
                        ignore_scope: true,
                        ..Default::default()
                    },
                    tags: Some(tags.iter().map(|tag| tag.to_string()).collect_vec()),
                    ..Default::default()
                },
                1,
                20,
                None,
                None,
                funs,
                ctx,
            )
            .await?
        };

        // First iterate over the models
        for model in models.records {
            if tags.contains(&model.tag.as_str()) {
                result.insert(
                    model.tag.clone(),
                    FlowTemplateModelResp {
                        id: model.id,
                        name: model.name,
                        create_time: model.create_time,
                        update_time: model.update_time,
                    },
                );
            }
        }
        // Iterate over the tag based on the existing result and get the default model
        for tag in tags {
            if !result.contains_key(tag) {
                // copy custom model
                let model_id = Self::add_custom_model(tag, "", template_id.clone(), funs, ctx).await?;
                let custom_model = Self::get_item(
                    &model_id,
                    &FlowModelFilterReq {
                        basic: RbumBasicFilterReq { ..Default::default() },
                        ..Default::default()
                    },
                    funs,
                    ctx,
                )
                .await?;
                result.insert(
                    tag.to_string(),
                    FlowTemplateModelResp {
                        id: custom_model.id.clone(),
                        name: custom_model.name.clone(),
                        create_time: custom_model.create_time,
                        update_time: custom_model.update_time,
                    },
                );
            }
        }

        Ok(result)
    }

    // add custom model by template model
    // rel_template_id: Associated parent template id
    // current_template_id: Current template id
    pub async fn add_custom_model(tag: &str, parent_template_id: &str, rel_template_id: Option<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let current_model = Self::find_one_detail_item(
            &FlowModelFilterReq {
                basic: RbumBasicFilterReq {
                    ignore_scope: true,
                    ..Default::default()
                },
                tags: Some(vec![tag.to_string()]),
                rel_template_id: rel_template_id.clone(),
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

        let basic = if !parent_template_id.is_empty() {
            RbumBasicFilterReq {
                with_sub_own_paths: true,
                ignore_scope: true,
                ..Default::default()
            }
        } else {
            RbumBasicFilterReq::default()
        };
        let parent_model = if let Some(parent_model) = Self::find_one_detail_item(
            &FlowModelFilterReq {
                basic,
                tags: Some(vec![tag.to_string()]),
                rel_template_id: Some(parent_template_id.to_string()),
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

        let mut transitions = parent_model.transitions();
        // add model
        let model_id = Self::add_item(
            &mut FlowModelAddReq {
                name: parent_model.name.into(),
                icon: Some(parent_model.icon),
                info: Some(parent_model.info),
                init_state_id: parent_model.init_state_id,
                template: rel_template_id.is_some(),
                rel_template_id,
                transitions: None,
                rel_model_id: Some(parent_model.id.clone()),
                tag: Some(parent_model.tag),
                scope_level: None,
                disabled: Some(parent_model.disabled),
            },
            funs,
            ctx,
        )
        .await?;
        // bind states
        let states = FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowModelState, &parent_model.id, None, None, funs, &global_ctx)
            .await?
            .iter()
            .sorted_by_key(|rel| TardisFuns::json.str_to_obj::<FlowStateRelModelExt>(&rel.ext).unwrap_or_default().sort)
            .map(|rel| (rel.rel_id.clone(), TardisFuns::json.str_to_obj::<FlowStateRelModelExt>(&rel.ext).unwrap_or_default()))
            .collect::<Vec<_>>();
        for (state_id, ext) in states.iter() {
            FlowRelServ::add_simple_rel(
                &FlowRelKind::FlowModelState,
                &model_id,
                state_id,
                None,
                None,
                false,
                true,
                Some(json!(ext).to_string()),
                funs,
                ctx,
            )
            .await?;
        }
        // add transition
        // sub role_id instead of role_id
        for transition in &mut transitions {
            for role_id in &mut transition.guard_by_spec_role_ids {
                *role_id = FlowExternalServ::do_find_embed_subrole_id(role_id, ctx, funs).await.unwrap_or(role_id.to_string());
            }
        }
        Self::add_transitions(&model_id, &transitions.into_iter().map(|trans| trans.into()).collect_vec(), funs, ctx).await?;

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

    pub async fn bind_state(flow_rel_kind: &FlowRelKind, flow_model_id: &str, req: &FlowModelBindStateReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let flow_state_id = &req.state_id;
        if FlowStateServ::get_item(flow_state_id, &FlowStateFilterReq::default(), funs, ctx).await.is_err() {
            return Err(funs.err().internal_error("flow_model_serv", "bind_state", "The flow state is not found", "404-flow-state-not-found"));
        }
        let sort = req.sort;
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
                "bind_state",
                "The own_paths of current mode isn't the own_paths of ctx",
                "404-flow-model-not-found",
            ));
        }
        let ext = FlowStateRelModelExt { sort, ..Default::default() };
        FlowRelServ::add_simple_rel(
            flow_rel_kind,
            flow_model_id,
            flow_state_id,
            None,
            None,
            false,
            true,
            Some(json!(ext).to_string()),
            funs,
            ctx,
        )
        .await?;

        Self::modify_item(
            flow_model_id,
            &mut FlowModelModifyReq {
                name: Some(Self::get_model_name(flow_model_id, funs, ctx).await?.into()),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;

        Ok(())
    }

    pub async fn unbind_state(flow_rel_kind: &FlowRelKind, flow_model_id: &str, req: &FlowModelUnbindStateReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let flow_state_id = &req.state_id;
        // Can only be deleted when not in use
        if FlowInstServ::state_is_used(flow_model_id, flow_state_id, funs, ctx).await? {
            return Err(funs.err().conflict(
                &Self::get_obj_name(),
                "unbind_state",
                &format!("state {flow_state_id} already used"),
                "409-flow-state-already-used",
            ));
        }
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
                "unbind_state",
                "The own_paths of current mode isn't the own_paths of ctx",
                "404-flow-model-not-found",
            ));
        }
        FlowRelServ::delete_simple_rel(flow_rel_kind, flow_model_id, flow_state_id, funs, ctx).await?;

        //delete transitions
        let trans_ids =
            Self::find_transitions_by_state_id(flow_model_id, Some(vec![flow_state_id.to_string()]), None, funs, ctx).await?.into_iter().map(|trans| trans.id).collect_vec();
        Self::delete_transitions(flow_model_id, &trans_ids, funs, ctx).await?;
        let trans_ids =
            Self::find_transitions_by_state_id(flow_model_id, None, Some(vec![flow_state_id.to_string()]), funs, ctx).await?.into_iter().map(|trans| trans.id).collect_vec();
        Self::delete_transitions(flow_model_id, &trans_ids, funs, ctx).await?;

        Self::modify_item(
            flow_model_id,
            &mut FlowModelModifyReq {
                name: Some(Self::get_model_name(flow_model_id, funs, ctx).await?.into()),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;

        Ok(())
    }

    pub async fn resort_state(flow_rel_kind: &FlowRelKind, flow_model_id: &str, sort_req: &FlowModelSortStatesReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let sort_states = &sort_req.sort_states;
        let states = FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowModelState, flow_model_id, None, None, funs, ctx).await?;
        for sort_state in sort_states {
            let show_btns = TardisFuns::json
                .str_to_obj::<FlowStateRelModelExt>(
                    &states
                        .iter()
                        .find(|state| state.rel_id == sort_state.state_id)
                        .ok_or_else(|| funs.err().internal_error("flow_model_serv", "resort_state", "rel not found", "404-rel-not-found"))?
                        .ext,
                )
                .unwrap_or_default()
                .show_btns;
            let ext = FlowStateRelModelExt { sort: sort_state.sort, show_btns };
            FlowRelServ::modify_simple_rel(
                flow_rel_kind,
                flow_model_id,
                &sort_state.state_id,
                &mut RbumRelModifyReq {
                    tag: None,
                    note: None,
                    ext: Some(json!(ext).to_string()),
                },
                funs,
                ctx,
            )
            .await?;
        }
        Self::modify_item(
            flow_model_id,
            &mut FlowModelModifyReq {
                name: Some(Self::get_model_name(flow_model_id, funs, ctx).await?.into()),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;

        Ok(())
    }

    pub async fn modify_rel_state(flow_model_id: &str, state_id: &str, new_ext: &FlowStateRelModelExt, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let mut ext = TardisFuns::json.str_to_obj::<FlowStateRelModelExt>(
            &FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowModelState, flow_model_id, None, None, funs, ctx)
                .await?
                .into_iter()
                .find(|rel| rel.rel_id == state_id)
                .ok_or_else(|| funs.err().internal_error("flow_model_serv", "modify_rel_state", "rel not found", "404-rel-not-found"))?
                .ext,
        )?;
        if let Some(show_btns) = new_ext.show_btns.clone() {
            ext.show_btns = Some(show_btns);
        }
        FlowRelServ::modify_simple_rel(
            &FlowRelKind::FlowModelState,
            flow_model_id,
            state_id,
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

    pub async fn resort_transition(flow_model_id: &str, sort_req: &FlowTransitionSortStatesReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        // @TODO wait tardis updated
        let modify_trans = sort_req
            .sort_states
            .iter()
            .map(|sort_req| FlowTransitionModifyReq {
                id: sort_req.id.clone().into(),
                name: None,
                from_flow_state_id: None,
                to_flow_state_id: None,
                transfer_by_auto: None,
                transfer_by_timer: None,
                guard_by_creator: None,
                guard_by_his_operators: None,
                guard_by_assigned: None,
                guard_by_spec_account_ids: None,
                guard_by_spec_role_ids: None,
                guard_by_spec_org_ids: None,
                guard_by_other_conds: None,
                vars_collect: None,
                double_check: None,
                is_notify: None,
                action_by_pre_callback: None,
                action_by_post_callback: None,
                action_by_post_changes: None,
                action_by_front_changes: None,
                sort: Some(sort_req.sort),
            })
            .collect_vec();
        Self::modify_transitions(flow_model_id, &modify_trans, funs, ctx).await?;

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

    #[async_recursion]
    pub async fn check_post_action_ring(
        transition_detail: FlowTransitionDetailResp,
        current_result: (bool, Vec<String>),
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<(bool, Vec<String>)> {
        let (mut is_ring, mut current_chain) = current_result.clone();
        if is_ring || current_chain.iter().any(|trans_id| trans_id == &transition_detail.id) {
            return Ok((true, current_chain));
        }
        current_chain.push(transition_detail.id.clone());

        let model_detail = Self::get_item(&transition_detail.rel_flow_model_id, &FlowModelFilterReq::default(), funs, ctx).await?;
        // check post changes
        let post_changes = transition_detail
            .action_by_post_changes()
            .into_iter()
            .filter(|trans| trans.kind == FlowTransitionActionChangeKind::State)
            .map(FlowTransitionActionChangeAgg::from)
            .collect_vec();
        if !post_changes.is_empty() {
            for post_change in post_changes {
                if let Some(change_info) = &post_change.state_change_info {
                    if let Some(flow_model_id) = Self::find_id_items(
                        &FlowModelFilterReq {
                            basic: RbumBasicFilterReq {
                                ignore_scope: true,
                                ..Default::default()
                            },
                            tags: Some(vec![change_info.obj_tag.clone()]),
                            rel_template_id: if model_detail.rel_template_id.is_empty() {
                                None
                            } else {
                                Some(model_detail.rel_template_id.clone())
                            },
                            ..Default::default()
                        },
                        None,
                        None,
                        funs,
                        ctx,
                    )
                    .await?
                    .pop()
                    {
                        let transitions = FlowModelServ::find_transitions_by_state_id(
                            &flow_model_id,
                            change_info.obj_current_state_id.clone(),
                            Some(vec![change_info.changed_state_id.clone()]),
                            funs,
                            ctx,
                        )
                        .await?;
                        for transition_detail in transitions {
                            (is_ring, current_chain) = Self::check_post_action_ring(transition_detail, (is_ring, current_chain.clone()), funs, ctx).await?;
                            if is_ring {
                                return Ok((true, current_chain));
                            }
                        }
                    }
                }
            }
        }
        // check front changes
        let flow_transitions = model_detail
            .transitions()
            .into_iter()
            .filter(|trans| trans.from_flow_state_id == transition_detail.to_flow_state_id && !trans.action_by_front_changes().is_empty())
            .sorted_by_key(|trans| trans.sort)
            .collect_vec();
        for transition_detail in flow_transitions {
            (is_ring, current_chain) = Self::check_post_action_ring(transition_detail, (is_ring, current_chain.clone()), funs, ctx).await?;
            if is_ring {
                return Ok((true, current_chain));
            }
        }

        Ok((is_ring, current_chain))
    }

    pub async fn find_rel_states(tags: Vec<&str>, rel_template_id: Option<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<FlowModelFindRelStateResp>> {
        let mut result = Vec::new();
        for tag in tags {
            let flow_model_id = FlowInstServ::get_model_id_by_own_paths_and_rel_template_id(tag, rel_template_id.clone(), funs, ctx).await?;
            let mut states = Self::find_sorted_rel_states_by_model_id(&flow_model_id, funs, ctx).await?;
            result.append(&mut states);
        }
        Ok(result)
    }

    async fn find_sorted_rel_states_by_model_id(flow_model_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<FlowModelFindRelStateResp>> {
        let state_ids = FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowModelState, flow_model_id, None, None, funs, ctx)
            .await?
            .iter()
            .sorted_by_key(|rel| TardisFuns::json.str_to_obj::<FlowStateRelModelExt>(&rel.ext).unwrap_or_default().sort)
            .map(|rel| rel.rel_id.clone())
            .collect::<Vec<_>>();
        Ok(FlowStateServ::find_detail_items(
            &FlowStateFilterReq {
                basic: RbumBasicFilterReq {
                    ids: Some(state_ids),
                    with_sub_own_paths: true,
                    own_paths: Some("".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
            None,
            Some(true),
            funs,
            ctx,
        )
        .await?
        .iter()
        .map(|state_detail| FlowModelFindRelStateResp {
            id: state_detail.id.clone(),
            name: state_detail.name.clone(),
            color: state_detail.color.clone(),
        })
        .collect_vec())
    }

    async fn get_model_name(flow_model_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        Ok(Self::find_sorted_rel_states_by_model_id(flow_model_id, funs, ctx).await?.into_iter().map(|state| state.name).collect_vec().join("-"))
    }
}
