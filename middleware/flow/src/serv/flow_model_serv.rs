use bios_basic::rbum::{
    dto::{
        rbum_filer_dto::RbumBasicFilterReq,
        rbum_item_dto::{RbumItemKernelAddReq, RbumItemKernelModifyReq},
    },
    serv::{
        rbum_crud_serv::{ID_FIELD, NAME_FIELD, REL_DOMAIN_ID_FIELD, REL_KIND_ID_FIELD},
        rbum_item_serv::{RbumItemCrudOperation, RBUM_ITEM_TABLE},
    },
};
use itertools::Itertools;
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    db::sea_orm::{
        sea_query::{Alias, Cond, Expr, Query, SelectStatement},
        EntityName, EntityTrait, JoinType, QueryFilter, Set,
    },
    serde_json::json,
    web::web_resp::TardisPage,
    TardisFuns, TardisFunsInst,
};

use crate::{
    domain::{flow_model, flow_transition},
    dto::{
        flow_model_dto::{FlowModelAddReq, FlowModelDetailResp, FlowModelFilterReq, FlowModelModifyReq, FlowModelModifyStateReq, FlowModelSummaryResp, ModifyStateOpKind},
        flow_state_dto::FlowStateFilterReq,
        flow_transition_dto::{FlowTransitionAddReq, FlowTransitionDetailResp, FlowTransitionModifyReq},
        flow_var_dto::FlowVarInfo,
    },
    flow_config::FlowBasicInfoManager,
    serv::flow_state_serv::FlowStateServ,
};
use async_trait::async_trait;

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
            tag: Set(add_req.tag.as_ref().unwrap_or(&"".to_string()).to_string()),
            ..Default::default()
        })
    }

    async fn after_add_item(flow_model_id: &str, add_req: &mut FlowModelAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if let Some(transitions) = &add_req.transitions {
            Self::add_transitions(flow_model_id, transitions, funs, ctx).await?;
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
            flow_model.tag = Set(tag.to_string());
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
        Ok(())
    }

    async fn package_ext_query(query: &mut SelectStatement, _: bool, filter: &FlowModelFilterReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<()> {
        query.column((flow_model::Entity, flow_model::Column::Icon));
        query.column((flow_model::Entity, flow_model::Column::Info));
        query.column((flow_model::Entity, flow_model::Column::InitStateId));
        query.column((flow_model::Entity, flow_model::Column::Tag));
        query.expr_as(Expr::val(json! {()}), Alias::new("transitions"));
        if let Some(tag) = &filter.tag {
            query.and_where(Expr::col(flow_model::Column::Tag).eq(tag.as_str()));
        }
        Ok(())
    }

    async fn get_item(flow_model_id: &str, filter: &FlowModelFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<FlowModelDetailResp> {
        let mut flow_model = Self::do_get_item(flow_model_id, filter, funs, ctx).await?;
        let flow_transitions = Self::find_transitions(flow_model_id, funs, ctx).await?;
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
        for mut flow_model in &mut flow_models.records {
            let flow_transitions = Self::find_transitions(&flow_model.id, funs, ctx).await?;
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
        for mut flow_model in &mut flow_models {
            let flow_transitions = Self::find_transitions(&flow_model.id, funs, ctx).await?;
            flow_model.transitions = Some(TardisFuns::json.obj_to_json(&flow_transitions)?);
        }
        Ok(flow_models)
    }
}

impl FlowModelServ {
    pub async fn add_transitions(flow_model_id: &str, add_req: &[FlowTransitionAddReq], funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let flow_state_ids = add_req.iter().map(|req| req.from_flow_state_id.to_string()).chain(add_req.iter().map(|req| req.to_flow_state_id.to_string())).unique().collect_vec();
        let flow_state_ids_len = flow_state_ids.len();
        if FlowStateServ::count_items(
            &FlowStateFilterReq {
                basic: RbumBasicFilterReq {
                    ids: Some(flow_state_ids),
                    with_sub_own_paths: true,
                    enabled: Some(true),
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await? as usize
            != flow_state_ids_len
        {
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
                guard_by_spec_account_ids: Set(req.guard_by_spec_account_ids.as_ref().unwrap_or(&vec![]).clone()),
                guard_by_spec_role_ids: Set(req.guard_by_spec_role_ids.as_ref().unwrap_or(&vec![]).clone()),
                guard_by_other_conds: Set(req.guard_by_other_conds.as_ref().map(|conds| TardisFuns::json.obj_to_json(conds).unwrap()).unwrap_or(json!({}))),

                vars_collect: Set(req.vars_collect.as_ref().map(|vars| TardisFuns::json.obj_to_json(vars).unwrap()).unwrap_or(json!({}))),

                action_by_pre_callback: Set(req.action_by_pre_callback.as_ref().unwrap_or(&"".to_string()).to_string()),
                action_by_post_callback: Set(req.action_by_post_callback.as_ref().unwrap_or(&"".to_string()).to_string()),

                rel_flow_model_id: Set(flow_model_id.to_string()),
                ..Default::default()
            })
            .collect_vec();
        funs.db().insert_many(flow_transitions, ctx).await
    }

    pub async fn modify_transitions(flow_model_id: &str, modify_req: &Vec<FlowTransitionModifyReq>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let flow_state_ids = modify_req
            .iter()
            .filter(|req| req.from_flow_state_id.is_some())
            .map(|req| req.from_flow_state_id.as_ref().unwrap().to_string())
            .chain(modify_req.iter().filter(|req| req.to_flow_state_id.is_some()).map(|req| req.to_flow_state_id.as_ref().unwrap().to_string()))
            .unique()
            .collect_vec();
        let flow_state_ids_len = flow_state_ids.len();
        if FlowStateServ::count_items(
            &FlowStateFilterReq {
                basic: RbumBasicFilterReq {
                    ids: Some(flow_state_ids),
                    with_sub_own_paths: true,
                    enabled: Some(true),
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await? as usize
            != flow_state_ids_len
        {
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
            if let Some(guard_by_spec_account_ids) = &req.guard_by_spec_account_ids {
                flow_transition.guard_by_spec_account_ids = Set(guard_by_spec_account_ids.clone());
            }
            if let Some(guard_by_spec_role_ids) = &req.guard_by_spec_role_ids {
                flow_transition.guard_by_spec_role_ids = Set(guard_by_spec_role_ids.clone());
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

    async fn find_transitions(flow_model_id: &str, funs: &TardisFunsInst, _ctx: &TardisContext) -> TardisResult<Vec<FlowTransitionDetailResp>> {
        let form_state_table = Alias::new("from_state");
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
                (flow_transition::Entity, flow_transition::Column::GuardBySpecAccountIds),
                (flow_transition::Entity, flow_transition::Column::GuardBySpecRoleIds),
                (flow_transition::Entity, flow_transition::Column::GuardByOtherConds),
                (flow_transition::Entity, flow_transition::Column::VarsCollect),
                (flow_transition::Entity, flow_transition::Column::ActionByPreCallback),
                (flow_transition::Entity, flow_transition::Column::ActionByPostCallback),
            ])
            .expr_as(Expr::col((form_state_table.clone(), NAME_FIELD.clone())).if_null(""), Alias::new("from_flow_state_name"))
            .expr_as(Expr::col((to_state_table.clone(), NAME_FIELD.clone())).if_null(""), Alias::new("to_flow_state_name"))
            .from(flow_transition::Entity)
            .join_as(
                JoinType::LeftJoin,
                RBUM_ITEM_TABLE.clone(),
                form_state_table.clone(),
                Cond::all()
                    .add(Expr::col((form_state_table.clone(), ID_FIELD.clone())).equals((flow_transition::Entity, flow_transition::Column::FromFlowStateId)))
                    .add(Expr::col((form_state_table.clone(), REL_KIND_ID_FIELD.clone())).eq(Self::get_rbum_kind_id().unwrap()))
                    .add(Expr::col((form_state_table.clone(), REL_DOMAIN_ID_FIELD.clone())).eq(Self::get_rbum_domain_id().unwrap())),
            )
            .join_as(
                JoinType::LeftJoin,
                RBUM_ITEM_TABLE.clone(),
                to_state_table.clone(),
                Cond::all()
                    .add(Expr::col((to_state_table.clone(), ID_FIELD.clone())).equals((flow_transition::Entity, flow_transition::Column::ToFlowStateId)))
                    .add(Expr::col((to_state_table.clone(), REL_KIND_ID_FIELD.clone())).eq(Self::get_rbum_kind_id().unwrap()))
                    .add(Expr::col((to_state_table.clone(), REL_DOMAIN_ID_FIELD.clone())).eq(Self::get_rbum_domain_id().unwrap())),
            )
            .and_where(Expr::col((flow_transition::Entity, flow_transition::Column::RelFlowModelId)).eq(flow_model_id));
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

    pub async fn modify_state(flow_model_id: &str, modify_req: &mut FlowModelModifyStateReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let model = Self::get_item(
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
        let mut state_ids: Vec<&str> = model.state_ids.split(',').collect();
        match modify_req.op {
            ModifyStateOpKind::Add => {
                if state_ids.iter().any(|id| **id == modify_req.state_id.to_string()) {
                    return Err(funs.err().internal_error(&Self::get_obj_name(), "modify_stats", "stats is exist", "500-insert-stats-duplicate"));
                }
                state_ids.push(&modify_req.state_id);
            }
            ModifyStateOpKind::Delete => {
                state_ids.retain(|id| **id != modify_req.state_id.to_string());
            }
        };

        Self::modify_item(
            flow_model_id,
            &mut FlowModelModifyReq {
                state_ids: Some(state_ids.join(",")),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        Ok(())
    }

    pub async fn modify_init_state(flow_model_id: &str, state_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let model = Self::get_item(
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
        let state_ids: Vec<&str> = model.state_ids.split(',').collect();
        if state_ids.iter().any(|id| *id != state_id) {
            return Err(funs.err().internal_error(&Self::get_obj_name(), "modify_init_state", "stats is not exist", "500-init-stats-not-exist"));
        }
        Self::modify_item(
            flow_model_id,
            &mut FlowModelModifyReq {
                init_state_id: Some(state_id.to_string()),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        Ok(())
    }

    pub async fn modify_transition_var(flow_model_id: &str, transition_id: &str, modify_req: Vec<FlowVarInfo>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let modify = FlowTransitionModifyReq {
            id: transition_id.into(),
            name: None,
            from_flow_state_id: None,
            to_flow_state_id: None,
            transfer_by_auto: None,
            transfer_by_timer: None,
            guard_by_creator: None,
            guard_by_his_operators: None,
            guard_by_spec_account_ids: None,
            guard_by_spec_role_ids: None,
            guard_by_other_conds: None,
            vars_collect: Some(modify_req),
            action_by_pre_callback: None,
            action_by_post_callback: None,
        };
        Self::modify_transitions(flow_model_id, &vec![modify], funs, ctx).await?;
        Ok(())
    }
}
