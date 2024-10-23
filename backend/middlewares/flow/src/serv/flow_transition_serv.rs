use bios_basic::rbum::serv::{
    rbum_crud_serv::{ID_FIELD, NAME_FIELD, REL_DOMAIN_ID_FIELD, REL_KIND_ID_FIELD},
    rbum_item_serv::{RbumItemCrudOperation, RBUM_ITEM_TABLE},
};
use itertools::Itertools;
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    chrono::Utc,
    db::sea_orm::{
        prelude::Expr,
        sea_query::{Alias, Cond, Query},
        EntityTrait, JoinType, Order, QueryFilter, Set,
    },
    serde_json::json,
    TardisFuns, TardisFunsInst,
};

use crate::{
    domain::{flow_state, flow_transition},
    dto::flow_transition_dto::{FlowTransitionActionChangeKind, FlowTransitionAddReq, FlowTransitionDetailResp, FlowTransitionModifyReq},
};

use super::{
    flow_rel_serv::{FlowRelKind, FlowRelServ},
    flow_state_serv::FlowStateServ,
};

pub struct FlowTransitionServ;

impl FlowTransitionServ {
    pub async fn add_transitions(
        flow_version_id: &str,
        from_flow_state_id: &str,
        add_req: &[FlowTransitionAddReq],
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let flow_state_ids = FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowModelState, flow_version_id, None, None, funs, ctx)
            .await?
            .iter()
            .map(|rel| rel.rel_id.clone())
            .collect::<Vec<_>>();
        if add_req.iter().any(|req| !flow_state_ids.contains(&from_flow_state_id.to_string()) || !flow_state_ids.contains(&req.to_flow_state_id)) {
            return Err(funs.err().not_found("flow_transition", "add_transitions", "the states to be added is not legal", "404-flow-state-add-not-legal"));
        }
        if add_req.is_empty() {
            return Ok(());
        }
        let flow_transitions = add_req
            .iter()
            .map(|req| flow_transition::ActiveModel {
                id: Set(TardisFuns::field.nanoid()),
                name: Set(req.name.as_ref().map(|name| name.to_string()).unwrap_or("".to_string())),

                from_flow_state_id: Set(from_flow_state_id.to_string()),
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

                rel_flow_model_version_id: Set(flow_version_id.to_string()),
                sort: Set(req.sort.unwrap_or(0)),
                ..Default::default()
            })
            .collect_vec();
        funs.db().insert_many(flow_transitions, ctx).await
    }

    pub async fn modify_transitions(flow_version_id: &str, modify_req: &[FlowTransitionModifyReq], funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
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
                "flow_transition",
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
                    .and_where(Expr::col((flow_transition::Entity, flow_transition::Column::RelFlowModelVersionId)).eq(flow_version_id.to_string()))
                    .and_where(Expr::col((flow_transition::Entity, flow_transition::Column::Id)).is_in(flow_transition_ids)),
            )
            .await? as usize
            != flow_transition_ids_lens
        {
            return Err(funs.err().not_found(
                "flow_transition",
                "modify_transitions",
                "the transition of related models not legal",
                "404-flow-transition-rel-model-not-legal",
            ));
        }
        let model_transitions = Self::find_transitions(flow_version_id, None, funs, ctx).await?;
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

    pub async fn delete_transitions(flow_version_id: &str, delete_flow_transition_ids: &Vec<String>, funs: &TardisFunsInst, _ctx: &TardisContext) -> TardisResult<()> {
        let delete_flow_transition_ids_lens = delete_flow_transition_ids.len();
        if funs
            .db()
            .count(
                Query::select()
                    .column((flow_transition::Entity, flow_transition::Column::Id))
                    .from(flow_transition::Entity)
                    .and_where(Expr::col((flow_transition::Entity, flow_transition::Column::RelFlowModelVersionId)).eq(flow_version_id.to_string()))
                    .and_where(Expr::col((flow_transition::Entity, flow_transition::Column::Id)).is_in(delete_flow_transition_ids)),
            )
            .await? as usize
            != delete_flow_transition_ids_lens
        {
            return Err(funs.err().not_found(
                "flow_transition",
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

    pub async fn find_transitions(
        flow_version_id: &str,
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
                (flow_transition::Entity, flow_transition::Column::RelFlowModelVersionId),
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
                    .add(Expr::col((from_state_rbum_table.clone(), REL_DOMAIN_ID_FIELD.clone())).eq(FlowStateServ::get_rbum_domain_id().unwrap())),
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
                    .add(Expr::col((to_state_rbum_table.clone(), REL_DOMAIN_ID_FIELD.clone())).eq(FlowStateServ::get_rbum_domain_id().unwrap())),
            )
            .join_as(
                JoinType::LeftJoin,
                flow_state::Entity,
                to_state_table.clone(),
                Cond::all().add(Expr::col((to_state_table.clone(), ID_FIELD.clone())).equals((flow_transition::Entity, flow_transition::Column::ToFlowStateId))),
            )
            .and_where(Expr::col((flow_transition::Entity, flow_transition::Column::RelFlowModelVersionId)).eq(flow_version_id));
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

    pub async fn find_transitions_by_state_id(
        flow_version_id: &str,
        current_state_id: Option<Vec<String>>,
        target_state_id: Option<Vec<String>>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<FlowTransitionDetailResp>> {
        Ok(Self::find_transitions(flow_version_id, None, funs, ctx)
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

    // pub async fn bind_state(transition_id: &str, state_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    //     FlowRelServ::add_simple_rel(
    //         &FlowRelKind::FlowStateTransition,
    //         state_id,
    //         transition_id,
    //         None,
    //         None,
    //         true,
    //         false,
    //         None,
    //         funs,
    //         ctx,
    //     )
    //     .await
    // }

    // pub async fn unbind_state(transition_id: &str, state_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    //     FlowRelServ::delete_simple_rel(
    //         &FlowRelKind::FlowStateTransition,
    //         state_id,
    //         transition_id,
    //         funs,
    //         ctx,
    //     )
    //     .await
    // }
}
