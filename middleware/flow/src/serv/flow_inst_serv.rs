use std::collections::HashMap;

use bios_basic::{
    dto::BasicQueryCondInfo,
    rbum::{
        dto::rbum_filer_dto::RbumBasicFilterReq,
        serv::{
            rbum_crud_serv::{ID_FIELD, NAME_FIELD, REL_DOMAIN_ID_FIELD, REL_KIND_ID_FIELD},
            rbum_item_serv::{RbumItemCrudOperation, RBUM_ITEM_TABLE},
        },
    },
};
use itertools::Itertools;
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    chrono::{DateTime, Utc},
    db::sea_orm::{
        self,
        sea_query::{Alias, Cond, Expr, Query},
        JoinType, Set,
    },
    futures_util::future::join_all,
    serde_json::Value,
    web::web_resp::TardisPage,
    TardisFuns, TardisFunsInst,
};

use crate::{
    domain::flow_inst,
    dto::{
        flow_inst_dto::{
            FlowInstAbortReq, FlowInstDetailResp, FlowInstFindNextTransitionResp, FlowInstFindNextTransitionsReq, FlowInstFindStateAndTransitionsReq,
            FlowInstFindStateAndTransitionsResp, FlowInstStartReq, FlowInstSummaryResp, FlowInstTransferReq, FlowInstTransferResp, FlowInstTransitionInfo, FlowOperationContext,
        },
        flow_model_dto::{FlowModelDetailResp, FlowModelFilterReq},
        flow_state_dto::{FlowStateFilterReq, FlowSysStateKind},
    },
    serv::{flow_model_serv::FlowModelServ, flow_state_serv::FlowStateServ},
};

pub struct FlowInstServ;

impl FlowInstServ {
    pub async fn start(flow_model_id: &str, start_req: &FlowInstStartReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let flow_model = FlowModelServ::find_items(
            &FlowModelFilterReq {
                basic: RbumBasicFilterReq {
                    ids: Some(vec![flow_model_id.to_string()]),
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
        .pop();
        if flow_model.is_none() {
            return Err(funs.err().not_found(
                "flow_inst",
                "start",
                &format!("flow model {} not found", flow_model_id),
                "404-flow-inst-rel-model-not-found",
            ));
        }
        let id = TardisFuns::field.nanoid();
        let flow_inst: flow_inst::ActiveModel = flow_inst::ActiveModel {
            id: Set(id.clone()),
            rel_flow_model_id: Set(flow_model_id.to_string()),

            current_state_id: Set(flow_model.unwrap().init_state_id),

            create_vars: Set(start_req.create_vars.as_ref().map(|vars| TardisFuns::json.obj_to_json(vars).unwrap())),
            create_ctx: Set(FlowOperationContext::from_ctx(ctx)),

            own_paths: Set(ctx.own_paths.to_string()),
            ..Default::default()
        };
        funs.db().insert_one(flow_inst, ctx).await?;
        Ok(id)
    }

    pub async fn abort(flow_inst_id: &str, abort_req: &FlowInstAbortReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if funs
            .db()
            .count(
                &Query::select()
                    .column((flow_inst::Entity, flow_inst::Column::Id))
                    .from(flow_inst::Entity)
                    .and_where(Expr::col((flow_inst::Entity, flow_inst::Column::Id)).eq(flow_inst_id.to_string()))
                    .and_where(Expr::col((flow_inst::Entity, flow_inst::Column::OwnPaths)).like(format!("{}%", ctx.own_paths))),
            )
            .await?
            == 0
        {
            return Err(funs.err().not_found("flow_inst", "abort", &format!("flow instance {} not found", flow_inst_id), "404-flow-inst-not-found"));
        }

        let flow_inst = flow_inst::ActiveModel {
            id: Set(flow_inst_id.to_string()),
            finish_ctx: Set(Some(FlowOperationContext::from_ctx(ctx))),
            finish_time: Set(Some(Utc::now())),
            finish_abort: Set(Some(true)),
            output_message: Set(Some(abort_req.message.to_string())),
            ..Default::default()
        };
        funs.db().update_one(flow_inst, ctx).await?;
        Ok(())
    }

    pub async fn get(flow_inst_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<FlowInstDetailResp> {
        let mut flow_insts = Self::find_detail(vec![flow_inst_id.to_string()], funs, ctx).await?;
        if flow_insts.len() == 1 {
            Ok(flow_insts.pop().unwrap())
        } else {
            Err(funs.err().not_found("flow_inst", "get", &format!("flow instance {} not found", flow_inst_id), "404-flow-inst-not-found"))
        }
    }

    async fn find_detail(flow_inst_ids: Vec<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<FlowInstDetailResp>> {
        #[derive(sea_orm::FromQueryResult)]
        pub struct FlowInstDetailResult {
            pub id: String,
            pub rel_flow_model_id: String,
            pub rel_flow_model_name: String,

            pub current_state_id: String,
            pub current_state_name: Option<String>,
            pub current_vars: Option<Value>,

            pub create_vars: Option<Value>,
            pub create_ctx: FlowOperationContext,
            pub create_time: DateTime<Utc>,

            pub finish_ctx: Option<FlowOperationContext>,
            pub finish_time: Option<DateTime<Utc>>,
            pub finish_abort: Option<bool>,
            pub output_message: Option<String>,

            pub transitions: Option<Value>,

            pub own_paths: String,
        }
        let rel_state_table = Alias::new("rel_state");
        let rel_model_table = Alias::new("rel_model");
        let mut query = Query::select();
        query
            .columns([
                (flow_inst::Entity, flow_inst::Column::Id),
                (flow_inst::Entity, flow_inst::Column::RelFlowModelId),
                (flow_inst::Entity, flow_inst::Column::CurrentStateId),
                (flow_inst::Entity, flow_inst::Column::CurrentVars),
                (flow_inst::Entity, flow_inst::Column::CreateVars),
                (flow_inst::Entity, flow_inst::Column::CreateCtx),
                (flow_inst::Entity, flow_inst::Column::CreateTime),
                (flow_inst::Entity, flow_inst::Column::FinishCtx),
                (flow_inst::Entity, flow_inst::Column::FinishTime),
                (flow_inst::Entity, flow_inst::Column::FinishAbort),
                (flow_inst::Entity, flow_inst::Column::OutputMessage),
                (flow_inst::Entity, flow_inst::Column::Transitions),
                (flow_inst::Entity, flow_inst::Column::OwnPaths),
            ])
            .expr_as(Expr::col((rel_state_table.clone(), NAME_FIELD.clone())).if_null(""), Alias::new("current_state_name"))
            .expr_as(Expr::col((rel_model_table.clone(), NAME_FIELD.clone())).if_null(""), Alias::new("rel_flow_model_name"))
            .from(flow_inst::Entity)
            .join_as(
                JoinType::LeftJoin,
                RBUM_ITEM_TABLE.clone(),
                rel_state_table.clone(),
                Cond::all()
                    .add(Expr::col((rel_state_table.clone(), ID_FIELD.clone())).equals((flow_inst::Entity, flow_inst::Column::CurrentStateId)))
                    .add(Expr::col((rel_state_table.clone(), REL_KIND_ID_FIELD.clone())).eq(FlowStateServ::get_rbum_kind_id().unwrap()))
                    .add(Expr::col((rel_state_table.clone(), REL_DOMAIN_ID_FIELD.clone())).eq(FlowStateServ::get_rbum_domain_id().unwrap())),
            )
            .join_as(
                JoinType::LeftJoin,
                RBUM_ITEM_TABLE.clone(),
                rel_model_table.clone(),
                Cond::all()
                    .add(Expr::col((rel_model_table.clone(), ID_FIELD.clone())).equals((flow_inst::Entity, flow_inst::Column::RelFlowModelId)))
                    .add(Expr::col((rel_model_table.clone(), REL_KIND_ID_FIELD.clone())).eq(FlowModelServ::get_rbum_kind_id().unwrap()))
                    .add(Expr::col((rel_model_table.clone(), REL_DOMAIN_ID_FIELD.clone())).eq(FlowModelServ::get_rbum_domain_id().unwrap())),
            )
            .and_where(Expr::col((flow_inst::Entity, flow_inst::Column::Id)).is_in(flow_inst_ids))
            .and_where(Expr::col((flow_inst::Entity, flow_inst::Column::OwnPaths)).like(format!("{}%", ctx.own_paths)));

        let flow_insts = funs.db().find_dtos::<FlowInstDetailResult>(&query).await?;
        Ok(flow_insts
            .into_iter()
            .map(|inst| FlowInstDetailResp {
                id: inst.id,
                rel_flow_model_id: inst.rel_flow_model_id,
                rel_flow_model_name: inst.rel_flow_model_name,
                create_vars: inst.create_vars.map(|create_vars| TardisFuns::json.json_to_obj(create_vars).unwrap()),
                create_ctx: inst.create_ctx,
                create_time: inst.create_time,
                finish_ctx: inst.finish_ctx,
                finish_time: inst.finish_time,
                finish_abort: inst.finish_abort,
                output_message: inst.output_message,
                own_paths: inst.own_paths,
                transitions: inst.transitions.map(|transitions| TardisFuns::json.json_to_obj(transitions).unwrap()),
                current_state_id: inst.current_state_id,
                current_state_name: inst.current_state_name,
                current_vars: inst.current_vars.map(|current_vars| TardisFuns::json.json_to_obj(current_vars).unwrap()),
            })
            .collect_vec())
    }

    pub async fn paginate(
        flow_model_id: Option<String>,
        finish: Option<bool>,
        with_sub: Option<bool>,
        page_number: u32,
        page_size: u32,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<FlowInstSummaryResp>> {
        #[derive(sea_orm::FromQueryResult)]
        pub struct FlowInstSummaryResult {
            pub id: String,
            pub rel_flow_model_id: String,
            pub rel_flow_model_name: String,

            pub current_state_id: String,

            pub create_ctx: Value,
            pub create_time: DateTime<Utc>,

            pub finish_ctx: Option<Value>,
            pub finish_time: Option<DateTime<Utc>>,
            pub finish_abort: bool,
            pub output_message: Option<String>,

            pub own_paths: String,
        }
        let mut query = Query::select();
        query
            .columns([
                (flow_inst::Entity, flow_inst::Column::Id),
                (flow_inst::Entity, flow_inst::Column::RelFlowModelId),
                (flow_inst::Entity, flow_inst::Column::CurrentStateId),
                (flow_inst::Entity, flow_inst::Column::CreateCtx),
                (flow_inst::Entity, flow_inst::Column::CreateTime),
                (flow_inst::Entity, flow_inst::Column::FinishCtx),
                (flow_inst::Entity, flow_inst::Column::FinishTime),
                (flow_inst::Entity, flow_inst::Column::FinishAbort),
                (flow_inst::Entity, flow_inst::Column::OutputMessage),
                (flow_inst::Entity, flow_inst::Column::OwnPaths),
            ])
            .expr_as(Expr::col((RBUM_ITEM_TABLE.clone(), NAME_FIELD.clone())).if_null(""), Alias::new("rel_flow_model_name"))
            .from(flow_inst::Entity)
            .left_join(
                RBUM_ITEM_TABLE.clone(),
                Expr::col((RBUM_ITEM_TABLE.clone(), ID_FIELD.clone())).equals((flow_inst::Entity, flow_inst::Column::RelFlowModelId)),
            );
        if with_sub.unwrap_or(false) {
            query.and_where(Expr::col((flow_inst::Entity, flow_inst::Column::OwnPaths)).like(format!("{}%", ctx.own_paths)));
        } else {
            query.and_where(Expr::col((flow_inst::Entity, flow_inst::Column::OwnPaths)).eq(ctx.own_paths.as_str()));
        }
        if let Some(flow_model_id) = flow_model_id {
            query.and_where(Expr::col((flow_inst::Entity, flow_inst::Column::RelFlowModelId)).eq(flow_model_id));
        }
        if let Some(finish) = finish {
            if finish {
                query.and_where(Expr::col((flow_inst::Entity, flow_inst::Column::FinishTime)).is_not_null());
            } else {
                query.and_where(Expr::col((flow_inst::Entity, flow_inst::Column::FinishTime)).is_null());
            }
        }
        let (flow_insts, total_size) = funs.db().paginate_dtos::<FlowInstSummaryResult>(&query, page_number as u64, page_size as u64).await?;
        Ok(TardisPage {
            page_size: page_size as u64,
            page_number: page_number as u64,
            total_size,
            records: flow_insts
                .into_iter()
                .map(|inst| FlowInstSummaryResp {
                    id: inst.id,
                    rel_flow_model_id: inst.rel_flow_model_id,
                    rel_flow_model_name: inst.rel_flow_model_name,
                    create_ctx: TardisFuns::json.json_to_obj(inst.create_ctx).unwrap(),
                    create_time: inst.create_time,
                    finish_ctx: inst.finish_ctx.map(|finish_ctx| TardisFuns::json.json_to_obj(finish_ctx).unwrap()),
                    finish_time: inst.finish_time,
                    finish_abort: inst.finish_abort,
                    output_message: inst.output_message,
                    own_paths: inst.own_paths,
                    current_state_id: inst.current_state_id,
                })
                .collect_vec(),
        })
    }

    pub(crate) async fn find_state_and_next_transitions(
        find_req: &Vec<FlowInstFindStateAndTransitionsReq>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<FlowInstFindStateAndTransitionsResp>> {
        let flow_insts = Self::find_detail(find_req.iter().map(|req| req.flow_inst_id.to_string()).collect_vec(), funs, ctx).await?;
        if flow_insts.len() != find_req.len() {
            return Err(funs.err().not_found("flow_inst", "find_state_and_next_transitions", "some flow instances not found", "404-flow-inst-not-found"));
        }
        let flow_models = FlowModelServ::find_detail_items(
            &FlowModelFilterReq {
                basic: RbumBasicFilterReq {
                    ids: Some(flow_insts.iter().map(|inst| inst.rel_flow_model_id.to_string()).collect_vec()),
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
        .await?;
        if flow_insts.len() != flow_models.len() {
            return Err(funs.err().not_found(
                "flow_inst",
                "find_state_and_next_transitions",
                "some flow models not found",
                "404-flow-inst-rel-model-not-found",
            ));
        }
        let state_and_next_transitions = join_all(
            flow_insts
                .iter()
                .map(|flow_inst| async {
                    let req = find_req.iter().find(|req| req.flow_inst_id == flow_inst.id).unwrap();
                    let flow_model = flow_models.iter().find(|model| model.id == flow_inst.rel_flow_model_id).unwrap();
                    Self::do_find_next_transitions(flow_inst, flow_model, None, &req.vars, ctx).await.unwrap()
                })
                .collect_vec(),
        )
        .await;
        Ok(state_and_next_transitions)
    }

    pub async fn find_next_transitions(
        flow_inst_id: &str,
        next_req: &FlowInstFindNextTransitionsReq,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<FlowInstFindNextTransitionResp>> {
        let flow_inst = Self::get(flow_inst_id, funs, ctx).await?;
        let flow_model = FlowModelServ::get_item(
            &flow_inst.rel_flow_model_id,
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
        let state_and_next_transitions = Self::do_find_next_transitions(&flow_inst, &flow_model, None, &next_req.vars, ctx).await?;
        Ok(state_and_next_transitions.next_flow_transitions)
    }

    pub async fn transfer(flow_inst_id: &str, transfer_req: &FlowInstTransferReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<FlowInstTransferResp> {
        let flow_inst = Self::get(flow_inst_id, funs, ctx).await?;
        let flow_model = FlowModelServ::get_item(
            &flow_inst.rel_flow_model_id,
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
        let next_flow_transition =
            Self::do_find_next_transitions(&flow_inst, &flow_model, Some(transfer_req.flow_transition_id.to_string()), &transfer_req.vars, ctx).await?.next_flow_transitions.pop();
        if next_flow_transition.is_none() {
            return Err(funs.err().not_found("flow_inst", "transfer", "no transferable state", "404-flow-inst-transfer-state-not-found"));
        }
        let next_flow_transition = next_flow_transition.unwrap();
        let next_flow_state = FlowStateServ::get_item(
            &next_flow_transition.next_flow_state_id,
            &FlowStateFilterReq {
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

        let mut new_vars: HashMap<String, Value> = HashMap::new();
        if let Some(current_vars) = &flow_inst.current_vars {
            new_vars.extend(current_vars.clone());
        }
        if let Some(req_vars) = &transfer_req.vars {
            new_vars.extend(req_vars.clone());
        }
        let mut new_transitions = Vec::new();
        if let Some(transitions) = flow_inst.transitions {
            new_transitions.extend(transitions);
        }
        new_transitions.push(FlowInstTransitionInfo {
            id: next_flow_transition.next_flow_transition_id.to_string(),
            start_time: Utc::now(),
            op_ctx: FlowOperationContext::from_ctx(ctx),
            output_message: transfer_req.message.clone(),
        });

        let mut flow_inst = flow_inst::ActiveModel {
            id: Set(flow_inst_id.to_string()),
            current_state_id: Set(next_flow_state.id.to_string()),
            current_vars: Set(Some(TardisFuns::json.obj_to_json(&new_vars).unwrap())),
            transitions: Set(Some(TardisFuns::json.obj_to_json(&new_transitions).unwrap())),
            ..Default::default()
        };
        if next_flow_state.sys_state == FlowSysStateKind::Finish {
            flow_inst.finish_ctx = Set(Some(FlowOperationContext::from_ctx(ctx)));
            flow_inst.finish_time = Set(Some(Utc::now()));
            flow_inst.finish_abort = Set(Some(false));
            flow_inst.output_message = Set(transfer_req.message.as_ref().map(|message| message.to_string()));
        }

        funs.db().update_one(flow_inst, ctx).await?;

        Ok(FlowInstTransferResp {
            new_flow_state_id: next_flow_transition.next_flow_state_id,
            new_flow_state_name: next_flow_transition.next_flow_state_name,
            vars: Some(new_vars),
        })
    }

    /// The kernel function of flow processing
    async fn do_find_next_transitions(
        flow_inst: &FlowInstDetailResp,
        flow_model: &FlowModelDetailResp,
        spec_flow_transition_id: Option<String>,
        req_vars: &Option<HashMap<String, Value>>,
        ctx: &TardisContext,
    ) -> TardisResult<FlowInstFindStateAndTransitionsResp> {
        let flow_model_transitions = flow_model.transitions();

        let next_transitions = flow_model_transitions
            .iter()
            .filter(|model_transition| {
                model_transition.from_flow_state_id == flow_inst.current_state_id
                    && (spec_flow_transition_id.is_none() || &model_transition.id == spec_flow_transition_id.as_ref().unwrap())
            })
            .filter(|model_transition| {
                if model_transition.guard_by_creator && (flow_inst.create_ctx.own_paths != ctx.own_paths || flow_inst.create_ctx.owner != ctx.owner) {
                    return false;
                }
                if !model_transition.guard_by_spec_account_ids.is_empty() && !model_transition.guard_by_spec_account_ids.contains(&ctx.owner) {
                    return false;
                }
                if !model_transition.guard_by_spec_role_ids.is_empty() && !model_transition.guard_by_spec_role_ids.iter().any(|role_ids| ctx.roles.contains(role_ids)) {
                    return false;
                }
                if model_transition.guard_by_his_operators
                    && flow_inst
                        .transitions
                        .as_ref()
                        .map(|inst_transitions| {
                            !inst_transitions.iter().any(|inst_transition| inst_transition.op_ctx.own_paths == ctx.own_paths && inst_transition.op_ctx.owner == ctx.owner)
                        })
                        .unwrap_or(false)
                {
                    return false;
                }
                if let Some(guard_by_other_conds) = model_transition.guard_by_other_conds() {
                    let mut check_vars: HashMap<String, Value> = HashMap::new();
                    if let Some(current_vars) = &flow_inst.current_vars {
                        check_vars.extend(current_vars.clone());
                    }
                    if let Some(req_vars) = &req_vars {
                        check_vars.extend(req_vars.clone());
                    }
                    if !BasicQueryCondInfo::check_or_and_conds(&guard_by_other_conds, &check_vars).unwrap() {
                        return false;
                    }
                }
                return true;
            })
            .map(|model_transition| FlowInstFindNextTransitionResp {
                next_flow_transition_id: model_transition.id.to_string(),
                next_flow_transition_name: model_transition.name.to_string(),
                vars_collect: model_transition.vars_collect(),
                next_flow_state_id: model_transition.to_flow_state_id.to_string(),
                next_flow_state_name: model_transition.from_flow_state_name.to_string(),
            })
            .collect_vec();
        let state_and_next_transitions = FlowInstFindStateAndTransitionsResp {
            flow_inst_id: flow_inst.id.to_string(),
            current_flow_state_name: flow_inst.current_state_name.as_ref().unwrap_or(&"".to_string()).to_string(),
            next_flow_transitions: next_transitions,
        };
        Ok(state_and_next_transitions)
    }
}
