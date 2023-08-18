use std::collections::HashMap;

use async_recursion::async_recursion;
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
use serde_json::json;
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
            FlowInstAbortReq, FlowInstBindReq, FlowInstBindResp, FlowInstDetailResp, FlowInstFindNextTransitionResp, FlowInstFindNextTransitionsReq,
            FlowInstFindStateAndTransitionsReq, FlowInstFindStateAndTransitionsResp, FlowInstStartReq, FlowInstSummaryResp, FlowInstTransferReq, FlowInstTransferResp,
            FlowInstTransitionInfo, FlowOperationContext,
        },
        flow_model_dto::{FlowModelDetailResp, FlowModelFilterReq},
        flow_state_dto::{FlowStateFilterReq, FlowSysStateKind},
        flow_transition_dto::{
            FlowTransitionActionByStateChangeInfo, FlowTransitionActionChangeAgg, FlowTransitionActionChangeInfo, FlowTransitionActionChangeKind, FlowTransitionDetailResp,
            StateChangeConditionOp,
        },
    },
    serv::{flow_model_serv::FlowModelServ, flow_state_serv::FlowStateServ},
};

use super::flow_external_serv::FlowExternalServ;

pub struct FlowInstServ;

impl FlowInstServ {
    pub async fn start(start_req: &FlowInstStartReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        // get model by own_paths
        let flow_model_id = Self::get_model_id_by_own_paths(&start_req.tag, funs, ctx).await?;
        let flow_model = FlowModelServ::get_item(
            &flow_model_id,
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
        let id = TardisFuns::field.nanoid();
        let current_state_id = if let Some(current_state_name) = &start_req.current_state_name {
            FlowStateServ::match_state_id_and_name_by_name(&start_req.tag, current_state_name, funs, ctx).await?.0
        } else {
            flow_model.init_state_id.clone()
        };
        let flow_inst: flow_inst::ActiveModel = flow_inst::ActiveModel {
            id: Set(id.clone()),
            rel_flow_model_id: Set(flow_model_id.to_string()),
            rel_business_obj_id: Set(start_req.rel_business_obj_id.to_string()),

            current_state_id: Set(current_state_id),

            create_vars: Set(start_req.create_vars.as_ref().map(|vars| TardisFuns::json.obj_to_json(vars).unwrap())),
            create_ctx: Set(FlowOperationContext::from_ctx(ctx)),

            own_paths: Set(ctx.own_paths.to_string()),
            ..Default::default()
        };
        funs.db().insert_one(flow_inst, ctx).await?;

        Self::do_request_webhook(
            None,
            flow_model.transitions().iter().filter(|model_transition| model_transition.to_flow_state_id == flow_model.init_state_id).collect_vec().pop(),
        )
        .await?;

        Ok(id)
    }

    pub async fn batch_bind(bind_req: &FlowInstBindReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<FlowInstBindResp>> {
        let mut result = vec![];
        for rel_business_obj in &bind_req.rel_business_objs {
            let flow_model_id = Self::get_model_id_by_own_paths(&bind_req.tag, funs, ctx).await?;

            let (current_state_id, current_state_name) = FlowStateServ::match_state_id_and_name_by_name(&bind_req.tag, &rel_business_obj.current_state_name, funs, ctx).await?;
            let id = TardisFuns::field.nanoid();
            let flow_inst: flow_inst::ActiveModel = flow_inst::ActiveModel {
                id: Set(id.clone()),
                rel_flow_model_id: Set(flow_model_id.to_string()),
                rel_business_obj_id: Set(rel_business_obj.rel_business_obj_id.to_string()),

                current_state_id: Set(current_state_id),

                create_ctx: Set(FlowOperationContext::from_ctx(ctx)),

                own_paths: Set(rel_business_obj.own_paths.to_string()),
                ..Default::default()
            };
            let resp = if funs.db().insert_one(flow_inst, ctx).await.is_ok() {
                FlowInstBindResp {
                    rel_business_obj_id: rel_business_obj.rel_business_obj_id.clone(),
                    current_state_name,
                    inst_id: Some(id),
                }
            } else {
                FlowInstBindResp {
                    rel_business_obj_id: rel_business_obj.rel_business_obj_id.clone(),
                    current_state_name,
                    inst_id: None,
                }
            };
            result.push(resp);
        }

        Ok(result)
    }

    pub async fn get_model_id_by_own_paths(tag: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        // try get model in app path
        let mut result = FlowModelServ::find_one_item(
            &FlowModelFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some(ctx.own_paths.clone()),
                    ..Default::default()
                },
                tag: Some(tag.to_string()),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        // try get model in tenant path or default model
        if result.is_none() {
            result = FlowModelServ::find_one_item(
                &FlowModelFilterReq {
                    basic: RbumBasicFilterReq {
                        own_paths: Some(ctx.own_paths.split_once('/').unwrap_or_default().0.to_string()),
                        ..Default::default()
                    },
                    tag: Some(tag.to_string()),
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?;
        }
        match result {
            Some(model) => Ok(model.id),
            None => Err(funs.err().not_found("flow_inst_serv", "get_model_id_by_own_paths", "model not found", "404-model-not-found")),
        }
    }

    pub async fn abort(flow_inst_id: &str, abort_req: &FlowInstAbortReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if funs
            .db()
            .count(
                Query::select()
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

    pub async fn modify_assigned(flow_inst_id: &str, assigned_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if funs
            .db()
            .count(
                Query::select()
                    .column((flow_inst::Entity, flow_inst::Column::Id))
                    .from(flow_inst::Entity)
                    .and_where(Expr::col((flow_inst::Entity, flow_inst::Column::Id)).eq(flow_inst_id.to_string()))
                    .and_where(Expr::col((flow_inst::Entity, flow_inst::Column::OwnPaths)).like(format!("{}%", ctx.own_paths))),
            )
            .await?
            == 0
        {
            return Err(funs.err().not_found(
                "flow_inst",
                "modify_assigned",
                &format!("flow instance {} not found", flow_inst_id),
                "404-flow-inst-not-found",
            ));
        }
        let flow_inst = flow_inst::ActiveModel {
            id: Set(flow_inst_id.to_string()),
            current_assigned: Set(Some(assigned_id.to_string())),
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
            pub current_assigned: Option<String>,

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

            pub rel_business_obj_id: String,
        }
        let rel_state_table = Alias::new("rel_state");
        let rel_model_table = Alias::new("rel_model");
        let mut query = Query::select();
        query
            .columns([
                (flow_inst::Entity, flow_inst::Column::Id),
                (flow_inst::Entity, flow_inst::Column::RelFlowModelId),
                (flow_inst::Entity, flow_inst::Column::RelBusinessObjId),
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
                (flow_inst::Entity, flow_inst::Column::CurrentAssigned),
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
                current_assigned: inst.current_assigned,
                current_state_name: inst.current_state_name,
                current_vars: inst.current_vars.map(|current_vars| TardisFuns::json.json_to_obj(current_vars).unwrap()),
                rel_business_obj_id: inst.rel_business_obj_id,
            })
            .collect_vec())
    }

    pub async fn paginate(
        flow_model_id: Option<String>,
        tag: Option<String>,
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
            pub rel_business_obj_id: String,

            pub create_ctx: Value,
            pub create_time: DateTime<Utc>,

            pub finish_ctx: Option<Value>,
            pub finish_time: Option<DateTime<Utc>>,
            pub finish_abort: bool,
            pub output_message: Option<String>,

            pub own_paths: String,
            pub current_assigned: Option<String>,
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
                (flow_inst::Entity, flow_inst::Column::CurrentAssigned),
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
        if let Some(tag) = &tag {
            let flow_model_id = Self::get_model_id_by_own_paths(tag, funs, ctx).await?;
            query.and_where(Expr::col((flow_inst::Entity, flow_inst::Column::RelFlowModelId)).eq(flow_model_id));
        }
        if let Some(tag) = &tag {
            let flow_model_id = Self::get_model_id_by_own_paths(tag, funs, ctx).await?;
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
                    rel_business_obj_id: inst.rel_business_obj_id,
                    current_assigned: inst.current_assigned,
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
                    own_paths: Some("".to_string()),
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
        let state_and_next_transitions = join_all(
            flow_insts
                .iter()
                .map(|flow_inst| async {
                    let req = find_req.iter().find(|req| req.flow_inst_id == flow_inst.id).unwrap();
                    let flow_model = flow_models.iter().find(|model| model.id == flow_inst.rel_flow_model_id).unwrap();
                    Self::do_find_next_transitions(flow_inst, flow_model, None, &req.vars, funs, ctx).await.unwrap()
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
                    own_paths: Some("".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        let state_and_next_transitions = Self::do_find_next_transitions(&flow_inst, &flow_model, None, &next_req.vars, funs, ctx).await?;
        Ok(state_and_next_transitions.next_flow_transitions)
    }

    #[async_recursion]
    pub async fn transfer(flow_inst_id: &str, transfer_req: &FlowInstTransferReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<FlowInstTransferResp> {
        let flow_inst_detail = Self::get(flow_inst_id, funs, ctx).await?;
        let flow_model = FlowModelServ::get_item(
            &flow_inst_detail.rel_flow_model_id,
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
        let next_flow_transition = Self::do_find_next_transitions(
            &flow_inst_detail,
            &flow_model,
            Some(transfer_req.flow_transition_id.to_string()),
            &transfer_req.vars,
            funs,
            ctx,
        )
        .await?
        .next_flow_transitions
        .pop();
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
        if let Some(current_vars) = &flow_inst_detail.current_vars {
            new_vars.extend(current_vars.clone());
        }
        if let Some(req_vars) = &transfer_req.vars {
            new_vars.extend(req_vars.clone());
        }
        let mut new_transitions = Vec::new();
        if let Some(transitions) = &flow_inst_detail.transitions {
            new_transitions.extend(transitions.clone());
        }
        let from_transition_id = new_transitions.last().map(|from_transition| from_transition.id.clone());
        new_transitions.push(FlowInstTransitionInfo {
            id: next_flow_transition.next_flow_transition_id.to_string(),
            start_time: Utc::now(),
            op_ctx: FlowOperationContext::from_ctx(ctx),
            output_message: transfer_req.message.clone(),
        });

        let mut flow_inst = flow_inst::ActiveModel {
            id: Set(flow_inst_id.to_string()),
            current_state_id: Set(next_flow_state.id.to_string()),
            current_vars: Set(Some(TardisFuns::json.obj_to_json(&new_vars)?)),
            transitions: Set(Some(TardisFuns::json.obj_to_json(&new_transitions)?)),
            ..Default::default()
        };
        if next_flow_state.sys_state == FlowSysStateKind::Finish {
            flow_inst.finish_ctx = Set(Some(FlowOperationContext::from_ctx(ctx)));
            flow_inst.finish_time = Set(Some(Utc::now()));
            flow_inst.finish_abort = Set(Some(false));
            flow_inst.output_message = Set(transfer_req.message.as_ref().map(|message| message.to_string()));
        }

        funs.db().update_one(flow_inst, ctx).await?;
        // notify changes
        FlowExternalServ::do_notify_changes(
            &flow_model.tag,
            &flow_inst_detail.rel_business_obj_id,
            Some(next_flow_state.name.to_string()),
            vec![
                json!({
                    "current_state_id":next_flow_state.id
                }),
                json!({ "current_vars": &new_vars }),
                json!({ "transitions": &new_transitions }),
            ],
            ctx,
            funs,
        )
        .await?;

        let model_transition = flow_model.transitions();
        Self::do_request_webhook(
            from_transition_id.and_then(|id: String| model_transition.iter().find(|model_transition| model_transition.id == id)),
            model_transition.iter().find(|model_transition| model_transition.id == next_flow_transition.next_flow_transition_id),
        )
        .await?;

        let post_changes =
            model_transition.into_iter().find(|model_transition| model_transition.id == next_flow_transition.next_flow_transition_id).unwrap_or_default().action_by_post_changes();
        if !post_changes.is_empty() {
            Self::do_post_change(&flow_inst_detail, &flow_model, post_changes, ctx, funs).await?;
        }

        Ok(FlowInstTransferResp {
            prev_flow_state_id: flow_inst_detail.current_state_id,
            prev_flow_state_name: flow_inst_detail.current_state_name,
            new_flow_state_id: next_flow_transition.next_flow_state_id,
            new_flow_state_name: next_flow_transition.next_flow_state_name,
            vars: Some(new_vars),
        })
    }

    /// handling post change when the transition occurs
    async fn do_post_change(
        current_inst: &FlowInstDetailResp,
        current_model: &FlowModelDetailResp,
        post_changes: Vec<FlowTransitionActionChangeInfo>,
        ctx: &TardisContext,
        funs: &TardisFunsInst,
    ) -> TardisResult<()> {
        for post_change in post_changes {
            let post_change = FlowTransitionActionChangeAgg::from(post_change);
            match post_change.kind {
                FlowTransitionActionChangeKind::Var => {
                    if let Some(change_info) = post_change.var_change_info {
                        if let Some(rel_tag) = change_info.obj_tag.clone() {
                            let mut resp = FlowExternalServ::do_fetch_rel_obj(&current_model.tag, &current_inst.rel_business_obj_id, vec![rel_tag.clone()], ctx, funs).await?;
                            if !resp.rel_bus_objs.is_empty() {
                                for rel_bus_obj_id in resp.rel_bus_objs.pop().unwrap().rel_bus_obj_ids {
                                    FlowExternalServ::do_modify_field(&rel_tag, &rel_bus_obj_id, &change_info, ctx, funs).await?;
                                }
                            }
                        } else {
                            FlowExternalServ::do_modify_field(&current_model.tag, &current_inst.rel_business_obj_id, &change_info, ctx, funs).await?;
                        }
                    }
                }
                FlowTransitionActionChangeKind::State => {
                    if let Some(change_info) = post_change.state_change_info {
                        let mut resp =
                            FlowExternalServ::do_fetch_rel_obj(&current_model.tag, &current_inst.rel_business_obj_id, vec![change_info.obj_tag.clone()], ctx, funs).await?;
                        if !resp.rel_bus_objs.is_empty() {
                            let inst_ids = Self::find_inst_ids_by_rel_obj_ids(resp.rel_bus_objs.pop().unwrap().rel_bus_obj_ids, &change_info, funs, ctx).await?;
                            Self::do_modify_state_by_post_action(inst_ids, &change_info, funs, ctx).await?;
                        }
                    }
                }
            }
        }

        Ok(())
    }
    async fn find_inst_ids_by_rel_obj_ids(
        rel_bus_obj_ids: Vec<String>,
        change_info: &FlowTransitionActionByStateChangeInfo,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<String>> {
        #[derive(sea_orm::FromQueryResult)]
        pub struct FlowInstIdsResult {
            pub id: String,
        }
        let mut result_rel_obj_ids = Self::filter_rel_obj_ids_by_state(&rel_bus_obj_ids, &change_info.obj_current_state_id, funs, ctx).await?;

        if let Some(change_condition) = change_info.change_condition.clone() {
            // Check mismatch rel_obj_ids and filter them
            let mut mismatch_rel_obj_ids = vec![];
            for rel_obj_id in result_rel_obj_ids.iter() {
                if change_condition.current {
                    // collect rel tags
                    let mut rel_tags = vec![];
                    for condition_item in change_condition.conditions.iter() {
                        if condition_item.obj_tag.is_some() && !condition_item.state_id.is_empty() {
                            rel_tags.push(condition_item.obj_tag.clone().unwrap());
                        }
                    }
                    let resp = FlowExternalServ::do_fetch_rel_obj(&change_info.obj_tag, rel_obj_id, rel_tags, ctx, funs).await?;
                    if !resp.rel_bus_objs.is_empty() {
                        for rel_bus_obj in resp.rel_bus_objs {
                            let condition = change_condition
                                .conditions
                                .iter()
                                .find(|condition| condition.obj_tag.is_some() && condition.obj_tag.clone().unwrap() == rel_bus_obj.rel_tag.clone())
                                .unwrap();
                            let rel_obj_ids = Self::filter_rel_obj_ids_by_state(&rel_bus_obj.rel_bus_obj_ids, &Some(condition.state_id.clone()), funs, ctx).await?;
                            match condition.op {
                                StateChangeConditionOp::And => {
                                    if condition.state_id.len() != rel_obj_ids.len() {
                                        mismatch_rel_obj_ids.push(rel_obj_id.clone());
                                        continue;
                                    }
                                }
                                StateChangeConditionOp::Or => {
                                    if rel_obj_ids.is_empty() {
                                        mismatch_rel_obj_ids.push(rel_obj_id.clone());
                                        continue;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            result_rel_obj_ids = result_rel_obj_ids.into_iter().filter(|result_rel_obj_id| !mismatch_rel_obj_ids.contains(result_rel_obj_id)).collect_vec();
        }

        let result = funs
            .db()
            .find_dtos::<FlowInstIdsResult>(
                Query::select()
                    .columns([flow_inst::Column::Id, flow_inst::Column::CurrentStateId, flow_inst::Column::RelBusinessObjId])
                    .from(flow_inst::Entity)
                    .and_where(Expr::col(flow_inst::Column::RelBusinessObjId).is_in(&result_rel_obj_ids)),
            )
            .await?
            .iter()
            .map(|rel_inst| rel_inst.id.clone())
            .collect_vec();
        Ok(result)
    }

    async fn filter_rel_obj_ids_by_state(
        rel_bus_obj_ids: &Vec<String>,
        obj_current_state_id: &Option<Vec<String>>,
        funs: &TardisFunsInst,
        _ctx: &TardisContext,
    ) -> TardisResult<Vec<String>> {
        #[derive(sea_orm::FromQueryResult)]
        pub struct FlowInstRelObjIdsResult {
            pub id: String,
            pub current_state_id: String,
            pub rel_business_obj_id: String,
        }
        let rel_insts = funs
            .db()
            .find_dtos::<FlowInstRelObjIdsResult>(
                Query::select()
                    .columns([flow_inst::Column::Id, flow_inst::Column::CurrentStateId, flow_inst::Column::RelBusinessObjId])
                    .from(flow_inst::Entity)
                    .and_where(Expr::col(flow_inst::Column::RelBusinessObjId).is_in(rel_bus_obj_ids.clone())),
            )
            .await?;
        if rel_bus_obj_ids.len() != rel_insts.len() {
            return Err(funs.err().not_found("flow_inst", "do_post_change", "some flow instances not found", "404-flow-inst-not-found"));
        }
        let rel_inst_ids = rel_insts
            .iter()
            .filter(|inst_result| {
                if let Some(obj_current_state_id) = obj_current_state_id.clone() {
                    if !obj_current_state_id.contains(&inst_result.current_state_id) {
                        return false;
                    }
                }
                true
            })
            .map(|inst_result| inst_result.rel_business_obj_id.clone())
            .collect_vec();
        Ok(rel_inst_ids)
    }

    async fn do_modify_state_by_post_action(
        rel_inst_ids: Vec<String>,
        change_info: &FlowTransitionActionByStateChangeInfo,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let insts = Self::find_detail(rel_inst_ids, funs, ctx).await?;
        for rel_inst in insts {
            // find transition
            let flow_model = FlowModelServ::get_item(
                &rel_inst.rel_flow_model_id,
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
            let transition = Self::do_find_next_transitions(&rel_inst, &flow_model, None, &None, funs, ctx)
                .await?
                .next_flow_transitions
                .into_iter()
                .filter(|transition_detail| *transition_detail.next_flow_state_id == change_info.changed_state_id)
                .collect_vec()
                .pop();
            if let Some(transition) = transition {
                Self::transfer(
                    &rel_inst.id,
                    &FlowInstTransferReq {
                        flow_transition_id: transition.next_flow_transition_id,
                        message: None,
                        vars: None,
                    },
                    funs,
                    ctx,
                )
                .await?;
            }
        }
        Ok(())
    }

    /// request webhook when the transition occurs
    async fn do_request_webhook(from_transition_detail: Option<&FlowTransitionDetailResp>, to_transition_detail: Option<&FlowTransitionDetailResp>) -> TardisResult<()> {
        if let Some(from_transition_detail) = from_transition_detail {
            if !from_transition_detail.action_by_post_callback.is_empty() {
                let callback_url = format!(
                    "{}?transition={}",
                    from_transition_detail.action_by_post_callback.as_str(),
                    from_transition_detail.to_flow_state_name
                );
                let _ = TardisFuns::web_client().get_to_str(&callback_url, None).await?;
            }
        }
        if let Some(to_transition_detail) = to_transition_detail {
            if !to_transition_detail.action_by_pre_callback.is_empty() {
                let callback_url = format!(
                    "{}?transition={}",
                    to_transition_detail.action_by_pre_callback.as_str(),
                    to_transition_detail.to_flow_state_name
                );
                let _ = TardisFuns::web_client().get_to_str(&callback_url, None).await?;
            }
        }

        Ok(())
    }

    /// The kernel function of flow processing
    async fn do_find_next_transitions(
        flow_inst: &FlowInstDetailResp,
        flow_model: &FlowModelDetailResp,
        spec_flow_transition_id: Option<String>,
        req_vars: &Option<HashMap<String, Value>>,
        funs: &TardisFunsInst,
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
                if !model_transition.guard_by_creator
                    && model_transition.guard_by_spec_account_ids.is_empty()
                    && model_transition.guard_by_spec_role_ids.is_empty()
                    && model_transition.guard_by_spec_org_ids.is_empty()
                    && !model_transition.guard_by_his_operators
                {
                    return true;
                }
                if model_transition.guard_by_creator && !(flow_inst.create_ctx.own_paths != ctx.own_paths || flow_inst.create_ctx.owner != ctx.owner) {
                    return true;
                }
                if !model_transition.guard_by_spec_account_ids.is_empty() && model_transition.guard_by_spec_account_ids.contains(&ctx.owner) {
                    return true;
                }
                if !model_transition.guard_by_spec_role_ids.is_empty() && model_transition.guard_by_spec_role_ids.iter().any(|role_ids| ctx.roles.contains(role_ids)) {
                    return true;
                }
                if !model_transition.guard_by_spec_org_ids.is_empty() && model_transition.guard_by_spec_org_ids.iter().any(|role_ids| ctx.groups.contains(role_ids)) {
                    return true;
                }
                if model_transition.guard_by_assigned
                    && flow_inst.current_assigned.is_some()
                    && !(flow_inst.current_assigned.clone().unwrap() != ctx.own_paths || flow_inst.current_assigned.clone().unwrap() != ctx.owner)
                {
                    return true;
                }
                if model_transition.guard_by_his_operators
                    && flow_inst
                        .transitions
                        .as_ref()
                        .map(|inst_transitions| {
                            // except creator
                            inst_transitions
                                .iter()
                                .filter(|inst_transition| inst_transition.op_ctx.owner != flow_inst.create_ctx.owner)
                                .any(|inst_transition| inst_transition.op_ctx.own_paths == ctx.own_paths && inst_transition.op_ctx.owner == ctx.owner)
                        })
                        .unwrap_or(false)
                {
                    return true;
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
                false
            })
            .map(|model_transition| FlowInstFindNextTransitionResp {
                next_flow_transition_id: model_transition.id.to_string(),
                next_flow_transition_name: model_transition.name.to_string(),
                next_flow_state_id: model_transition.to_flow_state_id.to_string(),
                next_flow_state_name: model_transition.to_flow_state_name.to_string(),
                vars_collect: model_transition.vars_collect(),
                double_check: model_transition.double_check(),
            })
            .collect_vec();
        let current_flow_state_sys_state = if let Some(state) = FlowStateServ::find_one_item(
            &FlowStateFilterReq {
                basic: RbumBasicFilterReq {
                    ids: Some(vec![flow_inst.current_state_id.to_string()]),
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
        {
            state.sys_state
        } else {
            FlowSysStateKind::Finish
        };
        let state_and_next_transitions = FlowInstFindStateAndTransitionsResp {
            flow_inst_id: flow_inst.id.to_string(),
            current_flow_state_name: flow_inst.current_state_name.as_ref().unwrap_or(&"".to_string()).to_string(),
            current_flow_state_kind: current_flow_state_sys_state,
            next_flow_transitions: next_transitions,
        };
        Ok(state_and_next_transitions)
    }
}
