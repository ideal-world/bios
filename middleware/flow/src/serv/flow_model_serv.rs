use std::collections::HashMap;

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
        flow_model_dto::{FlowModelAddReq, FlowModelAggResp, FlowModelDetailResp, FlowModelFilterReq, FlowModelModifyReq, FlowModelSummaryResp, FlowStateAggResp, FlowTagKind},
        flow_state_dto::{FlowStateAddReq, FlowStateFilterReq, FlowSysStateKind},
        flow_transition_dto::{FlowTransitionAddReq, FlowTransitionDetailResp, FlowTransitionModifyReq},
    },
    flow_config::FlowBasicInfoManager,
    serv::flow_state_serv::FlowStateServ,
};
use async_trait::async_trait;

use super::{
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

    async fn before_add_item(add_req: &mut FlowModelAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let result = FlowModelServ::find_one_item(
            &FlowModelFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some(ctx.own_paths.clone()),
                    ..Default::default()
                },
                tag: add_req.tag.clone(),
            },
            funs,
            ctx,
        )
        .await?;
        if result.is_some() {
            return Err(funs.err().internal_error(
                "flow_model_serv",
                "before_add_item",
                "There can only be one model under the same tag and own_paths",
                "500-mx-flow-internal-error",
            ));
        }
        Ok(())
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
        Ok(())
    }

    async fn package_ext_query(query: &mut SelectStatement, _: bool, filter: &FlowModelFilterReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<()> {
        query.column((flow_model::Entity, flow_model::Column::Icon));
        query.column((flow_model::Entity, flow_model::Column::Info));
        query.column((flow_model::Entity, flow_model::Column::InitStateId));
        query.column((flow_model::Entity, flow_model::Column::Tag));
        query.expr_as(Expr::val(json! {()}), Alias::new("transitions"));
        if let Some(tag) = filter.tag.clone() {
            query.and_where(Expr::col(flow_model::Column::Tag).eq(tag));
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
    pub async fn init_model(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        // Work Order
        // add state
        let pending_state_id = FlowStateServ::add_item(
            &mut FlowStateAddReq {
                id_prefix: None,
                name: Some("待处理".into()),
                icon: None,
                sys_state: FlowSysStateKind::Start,
                info: None,
                state_kind: None,
                kind_conf: None,
                template: None,
                rel_state_id: None,
                tags: Some(vec!["ticket_states".to_string()]),
                scope_level: None,
                disabled: None,
            },
            funs,
            ctx,
        )
        .await?;
        let handling_state_id = FlowStateServ::add_item(
            &mut FlowStateAddReq {
                id_prefix: None,
                name: Some("处理中".into()),
                icon: None,
                sys_state: FlowSysStateKind::Progress,
                info: None,
                state_kind: None,
                kind_conf: None,
                template: None,
                rel_state_id: None,
                tags: Some(vec!["ticket_states".to_string()]),
                scope_level: None,
                disabled: None,
            },
            funs,
            ctx,
        )
        .await?;
        let confirmed_state_id = FlowStateServ::add_item(
            &mut FlowStateAddReq {
                id_prefix: None,
                name: Some("待确认".into()),
                icon: None,
                sys_state: FlowSysStateKind::Progress,
                info: None,
                state_kind: None,
                kind_conf: None,
                template: None,
                rel_state_id: None,
                tags: Some(vec!["ticket_states".to_string()]),
                scope_level: None,
                disabled: None,
            },
            funs,
            ctx,
        )
        .await?;
        let closed_state_id = FlowStateServ::add_item(
            &mut FlowStateAddReq {
                id_prefix: None,
                name: Some("已关闭".into()),
                icon: None,
                sys_state: FlowSysStateKind::Finish,
                info: None,
                state_kind: None,
                kind_conf: None,
                template: None,
                rel_state_id: None,
                tags: Some(vec!["ticket_states".to_string()]),
                scope_level: None,
                disabled: None,
            },
            funs,
            ctx,
        )
        .await?;
        let revoked_state_id = FlowStateServ::add_item(
            &mut FlowStateAddReq {
                id_prefix: None,
                name: Some("已撤销".into()),
                icon: None,
                sys_state: FlowSysStateKind::Finish,
                info: None,
                state_kind: None,
                kind_conf: None,
                template: None,
                rel_state_id: None,
                tags: Some(vec!["ticket_states".to_string()]),
                scope_level: None,
                disabled: None,
            },
            funs,
            ctx,
        )
        .await?;
        // add model
        let model_id = Self::add_item(
            &mut FlowModelAddReq {
                name: "默认工单流程".into(),
                init_state_id: pending_state_id.clone(),
                icon: None,
                info: None,
                transitions: Some(vec![
                    FlowTransitionAddReq {
                        from_flow_state_id: pending_state_id.clone(),
                        to_flow_state_id: handling_state_id.clone(),
                        name: Some("立即处理".into()),
                        transfer_by_auto: None,
                        transfer_by_timer: None,
                        guard_by_creator: Some(true),
                        guard_by_his_operators: None,
                        guard_by_assigned: None,
                        guard_by_spec_account_ids: None,
                        guard_by_spec_role_ids: None,
                        guard_by_other_conds: None,
                        vars_collect: None,
                        action_by_pre_callback: None,
                        action_by_post_callback: None,
                    },
                    FlowTransitionAddReq {
                        from_flow_state_id: pending_state_id.clone(),
                        to_flow_state_id: revoked_state_id.clone(),
                        name: Some("撤销".into()),
                        transfer_by_auto: None,
                        transfer_by_timer: None,
                        guard_by_creator: Some(true),
                        guard_by_his_operators: None,
                        guard_by_assigned: None,
                        guard_by_spec_account_ids: None,
                        guard_by_spec_role_ids: None,
                        guard_by_other_conds: None,
                        vars_collect: None,
                        action_by_pre_callback: None,
                        action_by_post_callback: None,
                    },
                    FlowTransitionAddReq {
                        from_flow_state_id: handling_state_id.clone(),
                        to_flow_state_id: confirmed_state_id.clone(),
                        name: Some("处理完成".into()),
                        transfer_by_auto: None,
                        transfer_by_timer: None,
                        guard_by_creator: None,
                        guard_by_his_operators: Some(true),
                        guard_by_assigned: None,
                        guard_by_spec_account_ids: None,
                        guard_by_spec_role_ids: None,
                        guard_by_other_conds: None,
                        vars_collect: None,
                        action_by_pre_callback: None,
                        action_by_post_callback: None,
                    },
                    FlowTransitionAddReq {
                        from_flow_state_id: handling_state_id.clone(),
                        to_flow_state_id: closed_state_id.clone(),
                        name: Some("关闭".into()),
                        transfer_by_auto: None,
                        transfer_by_timer: None,
                        guard_by_creator: Some(true),
                        guard_by_his_operators: None,
                        guard_by_assigned: None,
                        guard_by_spec_account_ids: None,
                        guard_by_spec_role_ids: None,
                        guard_by_other_conds: None,
                        vars_collect: None,
                        action_by_pre_callback: None,
                        action_by_post_callback: None,
                    },
                    FlowTransitionAddReq {
                        from_flow_state_id: confirmed_state_id.clone(),
                        to_flow_state_id: closed_state_id.clone(),
                        name: Some("确认解决".into()),
                        transfer_by_auto: None,
                        transfer_by_timer: None,
                        guard_by_creator: Some(true),
                        guard_by_his_operators: None,
                        guard_by_assigned: None,
                        guard_by_spec_account_ids: None,
                        guard_by_spec_role_ids: None,
                        guard_by_other_conds: None,
                        vars_collect: None,
                        action_by_pre_callback: None,
                        action_by_post_callback: None,
                    },
                    FlowTransitionAddReq {
                        from_flow_state_id: confirmed_state_id.clone(),
                        to_flow_state_id: handling_state_id.clone(),
                        name: Some("未解决".into()),
                        transfer_by_auto: None,
                        transfer_by_timer: None,
                        guard_by_creator: Some(true),
                        guard_by_his_operators: None,
                        guard_by_assigned: None,
                        guard_by_spec_account_ids: None,
                        guard_by_spec_role_ids: None,
                        guard_by_other_conds: None,
                        vars_collect: None,
                        action_by_pre_callback: None,
                        action_by_post_callback: None,
                    },
                ]),
                tag: Some(FlowTagKind::Ticket),
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
        FlowRelServ::add_simple_rel(&FlowRelKind::FlowModelState, &model_id, &pending_state_id, None, None, false, false, funs, ctx).await?;
        FlowRelServ::add_simple_rel(&FlowRelKind::FlowModelState, &model_id, &handling_state_id, None, None, false, false, funs, ctx).await?;
        FlowRelServ::add_simple_rel(&FlowRelKind::FlowModelState, &model_id, &confirmed_state_id, None, None, false, false, funs, ctx).await?;
        FlowRelServ::add_simple_rel(&FlowRelKind::FlowModelState, &model_id, &closed_state_id, None, None, false, false, funs, ctx).await?;
        FlowRelServ::add_simple_rel(&FlowRelKind::FlowModelState, &model_id, &revoked_state_id, None, None, false, false, funs, ctx).await?;

        Ok(())
    }

    pub async fn add_transitions(flow_model_id: &str, add_req: &[FlowTransitionAddReq], funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let flow_state_ids =
            FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowModelState, flow_model_id, None, None, funs, ctx).await?.iter().map(|rel| rel.rel_id.clone()).collect::<Vec<_>>();
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
                guard_by_assigned: Set(req.guard_by_assigned.unwrap_or(false)),
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
            if let Some(guard_by_assigned) = req.guard_by_assigned {
                flow_transition.guard_by_assigned = Set(guard_by_assigned);
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
                (flow_transition::Entity, flow_transition::Column::GuardByAssigned),
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

    pub async fn get_item_detail_aggs(flow_model_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<FlowModelAggResp> {
        let model_detail = Self::get_item(
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

        // find rel state
        let state_ids = FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowModelState, flow_model_id, Some(false), None, funs, ctx)
            .await?
            .iter()
            .map(|rel| (rel.rel_id.clone(), rel.rel_name.clone()))
            .collect::<Vec<_>>();
        let mut states = HashMap::new();
        for (state_id, state_name) in state_ids {
            let state_detail = FlowStateAggResp {
                id: state_id.clone(),
                name: state_name,
                is_init: model_detail.init_state_id == state_id,
                transitions: model_detail.transitions().into_iter().filter(|transition| transition.from_flow_state_id == state_id.clone()).collect_vec(),
            };
            states.insert(state_id, state_detail);
        }

        Ok(FlowModelAggResp {
            id: model_detail.id,
            name: model_detail.name,
            icon: model_detail.icon,
            info: model_detail.info,
            init_state_id: model_detail.init_state_id,
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

    // add or modify model by own_paths
    pub async fn add_or_modify_model(flow_model_id: &str, modify_req: &mut FlowModelModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let tag = Self::get_item(
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
        .await?
        .tag;
        // when the own_paths of current mode isn't the own_paths of ctx,it shows that I need add a new model with this model
        let mut models = Self::paginate_detail_items(
            &FlowModelFilterReq {
                tag: Some(tag),
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    own_paths: Some("".to_string()),
                    ..Default::default()
                },
            },
            1,
            10,
            Some(false),
            None,
            funs,
            ctx,
        )
        .await?
        .records;

        let current_model = models.pop().ok_or_else(|| funs.err().internal_error("flow_model_serv", "add_or_modify_model", "modify model error", "500-mx-flow-internal-error"))?;
        let result = if current_model.own_paths == ctx.own_paths {
            // modify
            Self::modify_item(&current_model.id, modify_req, funs, ctx).await?;
            current_model.id.clone()
        } else {
            // add model
            let transitions = current_model.transitions();
            let model_id = Self::add_item(
                &mut FlowModelAddReq {
                    name: modify_req.name.clone().map_or(current_model.name.into(), |name| name),
                    icon: modify_req.icon.clone().map_or(Some(current_model.icon), Some),
                    info: modify_req.info.clone().map_or(Some(current_model.info), Some),
                    init_state_id: modify_req.init_state_id.clone().map_or(current_model.init_state_id, |init_state_id| init_state_id),
                    transitions: modify_req.add_transitions.clone().map_or(Some(transitions.into_iter().map(|trans| trans.into()).collect_vec()), Some),
                    template: false,
                    rel_model_id: Some(flow_model_id.to_string()),
                    tag: modify_req.tag.clone().map_or(Some(current_model.tag), Some),
                    scope_level: modify_req.scope_level.clone().map_or(Some(current_model.scope_level), Some),
                    disabled: modify_req.disabled.map_or(Some(current_model.disabled), Some),
                },
                funs,
                ctx,
            )
            .await?;
            // bind states
            for state_id in FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowModelState, flow_model_id, None, None, funs, ctx)
                .await?
                .iter()
                .map(|rel| rel.rel_id.clone())
                .collect::<Vec<_>>()
            {
                FlowRelServ::add_simple_rel(&FlowRelKind::FlowModelState, &model_id, &state_id, None, None, false, false, funs, ctx).await?;
            }
            let mock_ctx = TardisContext {
                own_paths: "".to_string(),
                ..ctx.clone()
            };

            Self::modify_item(
                flow_model_id,
                &mut FlowModelModifyReq {
                    template: Some(true),
                    ..Default::default()
                },
                funs,
                &mock_ctx,
            )
            .await?;
            model_id
        };

        Ok(result)
    }

    pub async fn bind_state(
        flow_rel_kind: &FlowRelKind,
        flow_model_id: &str,
        flow_state_id: &str,
        start_timestamp: Option<i64>,
        end_timestamp: Option<i64>,
        ignore_exist_error: bool,
        to_is_outside: bool,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<String> {
        let mut result = "".to_string();
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
        if current_model.own_paths == ctx.own_paths {
            FlowRelServ::add_simple_rel(
                flow_rel_kind,
                flow_model_id,
                flow_state_id,
                start_timestamp,
                end_timestamp,
                ignore_exist_error,
                to_is_outside,
                funs,
                ctx,
            )
            .await?;
            result = flow_model_id.to_string();
        } else {
            let model_id = Self::add_or_modify_model(flow_model_id, &mut FlowModelModifyReq::default(), funs, ctx).await?;
            FlowRelServ::add_simple_rel(
                flow_rel_kind,
                &model_id,
                flow_state_id,
                start_timestamp,
                end_timestamp,
                ignore_exist_error,
                to_is_outside,
                funs,
                ctx,
            )
            .await?;
            result = model_id;
        }
        Ok(result)
    }

    pub async fn unbind_state(flow_rel_kind: &FlowRelKind, flow_model_id: &str, flow_state_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let mut result = "".to_string();
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
        if current_model.own_paths == ctx.own_paths {
            FlowRelServ::delete_simple_rel(flow_rel_kind, flow_model_id, flow_state_id, funs, ctx).await?;
            result = flow_model_id.to_string();
        } else {
            let model_id = Self::add_or_modify_model(flow_model_id, &mut FlowModelModifyReq::default(), funs, ctx).await?;
            FlowRelServ::delete_simple_rel(flow_rel_kind, &model_id, flow_state_id, funs, ctx).await?;
            result = model_id;
        }
        Ok(result)
    }
}
