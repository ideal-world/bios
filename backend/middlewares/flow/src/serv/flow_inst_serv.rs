use std::{
    collections::{HashMap, HashSet},
    str::FromStr as _,
};

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
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    chrono::{DateTime, Utc},
    db::sea_orm::{
        self,
        sea_query::{Alias, Cond, Expr, Query, SelectStatement},
        JoinType, Set,
    },
    futures_util::future::join_all,
    log::{debug, error},
    serde_json::Value,
    web::web_resp::TardisPage,
    TardisFuns, TardisFunsInst,
};

use crate::{
    domain::{flow_inst, flow_model, flow_transition},
    dto::{
        flow_external_dto::{FlowExternalCallbackOp, FlowExternalParams},
        flow_inst_dto::{
            FlowInstAbortReq, FlowInstBatchBindReq, FlowInstBatchBindResp, FlowInstDetailResp, FlowInstFilterReq, FlowInstFindNextTransitionResp, FlowInstFindNextTransitionsReq,
            FlowInstFindStateAndTransitionsReq, FlowInstFindStateAndTransitionsResp, FlowInstStartReq, FlowInstSummaryResp, FlowInstSummaryResult, FlowInstTransferReq,
            FlowInstTransferResp, FlowInstTransitionInfo, FlowOperationContext,
        },
        flow_model_dto::{FlowModelDetailResp, FlowModelFilterReq},
        flow_state_dto::{FlowStateAggResp, FlowStateFilterReq, FlowStateRelModelExt, FlowSysStateKind},
        flow_transition_dto::{FlowTransitionDetailResp, FlowTransitionFrontActionInfo},
        flow_var_dto::FillType,
    },
    flow_constants,
    helper::loop_check_helper,
    serv::{flow_model_serv::FlowModelServ, flow_state_serv::FlowStateServ},
};

use super::{
    flow_event_serv::FlowEventServ,
    flow_external_serv::FlowExternalServ,
    flow_rel_serv::{FlowRelKind, FlowRelServ},
};

pub struct FlowInstServ;

impl FlowInstServ {
    pub async fn start(start_req: &FlowInstStartReq, current_state_name: Option<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        // get model by own_paths
        let flow_model_id = FlowModelServ::get_model_id_by_own_paths_and_rel_template_id(&start_req.tag, None, funs, ctx).await?;
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
        let inst_id = TardisFuns::field.nanoid();
        let current_state_id = if let Some(current_state_name) = &current_state_name {
            if current_state_name.is_empty() {
                flow_model.init_state_id.clone()
            } else {
                FlowStateServ::match_state_id_by_name(&flow_model_id, current_state_name, funs, ctx).await?
            }
        } else {
            flow_model.init_state_id.clone()
        };
        let flow_inst: flow_inst::ActiveModel = flow_inst::ActiveModel {
            id: Set(inst_id.clone()),
            tag: Set(Some(flow_model.tag.clone())),
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

        Ok(inst_id)
    }

    pub async fn batch_bind(batch_bind_req: &FlowInstBatchBindReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<FlowInstBatchBindResp>> {
        let mut result = vec![];
        let mut current_ctx = ctx.clone();
        for rel_business_obj in &batch_bind_req.rel_business_objs {
            if rel_business_obj.rel_business_obj_id.is_none()
                || rel_business_obj.current_state_name.is_none()
                || rel_business_obj.own_paths.is_none()
                || rel_business_obj.owner.is_none()
            {
                debug!("rel_business_obj: {:?}", rel_business_obj);
                return Err(funs.err().not_found("flow_inst_serv", "batch_bind", "req is valid", ""));
            }
            current_ctx.own_paths = rel_business_obj.own_paths.clone().unwrap_or_default();
            current_ctx.owner = rel_business_obj.owner.clone().unwrap_or_default();
            let flow_model_id = FlowModelServ::get_model_id_by_own_paths_and_rel_template_id(&batch_bind_req.tag, None, funs, ctx).await?;

            let current_state_id = FlowStateServ::match_state_id_by_name(&flow_model_id, &rel_business_obj.current_state_name.clone().unwrap_or_default(), funs, ctx).await?;
            let mut inst_id = Self::get_inst_ids_by_rel_business_obj_id(vec![rel_business_obj.rel_business_obj_id.clone().unwrap_or_default()], funs, ctx).await?.pop();
            if inst_id.is_none() {
                let id = TardisFuns::field.nanoid();
                let flow_inst: flow_inst::ActiveModel = flow_inst::ActiveModel {
                    id: Set(id.clone()),
                    rel_flow_model_id: Set(flow_model_id.to_string()),
                    rel_business_obj_id: Set(rel_business_obj.rel_business_obj_id.clone().unwrap_or_default()),

                    current_state_id: Set(current_state_id),

                    create_ctx: Set(FlowOperationContext::from_ctx(&current_ctx)),

                    own_paths: Set(rel_business_obj.own_paths.clone().unwrap_or_default()),
                    ..Default::default()
                };
                funs.db().insert_one(flow_inst, &current_ctx).await?;
                inst_id = Some(id);
            }
            let current_state_name = Self::get(inst_id.as_ref().unwrap(), funs, &current_ctx).await?.current_state_name.unwrap_or_default();
            result.push(FlowInstBatchBindResp {
                rel_business_obj_id: rel_business_obj.rel_business_obj_id.clone().unwrap_or_default(),
                current_state_name,
                inst_id,
            });
        }

        Ok(result)
    }

    async fn package_ext_query(query: &mut SelectStatement, filter: &FlowInstFilterReq, _: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        query
            .columns([
                (flow_inst::Entity, flow_inst::Column::Id),
                (flow_inst::Entity, flow_inst::Column::RelFlowModelId),
                (flow_inst::Entity, flow_inst::Column::RelBusinessObjId),
                (flow_inst::Entity, flow_inst::Column::CreateVars),
                (flow_inst::Entity, flow_inst::Column::CurrentStateId),
                (flow_inst::Entity, flow_inst::Column::CreateCtx),
                (flow_inst::Entity, flow_inst::Column::CreateTime),
                (flow_inst::Entity, flow_inst::Column::FinishCtx),
                (flow_inst::Entity, flow_inst::Column::FinishTime),
                (flow_inst::Entity, flow_inst::Column::FinishAbort),
                (flow_inst::Entity, flow_inst::Column::OutputMessage),
                (flow_inst::Entity, flow_inst::Column::OwnPaths),
                (flow_inst::Entity, flow_inst::Column::Tag),
            ])
            .expr_as(Expr::col((RBUM_ITEM_TABLE.clone(), NAME_FIELD.clone())).if_null(""), Alias::new("rel_flow_model_name"))
            .from(flow_inst::Entity)
            .left_join(
                RBUM_ITEM_TABLE.clone(),
                Expr::col((RBUM_ITEM_TABLE.clone(), ID_FIELD.clone())).equals((flow_inst::Entity, flow_inst::Column::RelFlowModelId)),
            )
            .left_join(
                flow_model::Entity,
                Expr::col((flow_model::Entity, flow_model::Column::Id)).equals((flow_inst::Entity, flow_inst::Column::RelFlowModelId)),
            );
        if filter.with_sub.unwrap_or(false) {
            query.and_where(Expr::col((flow_inst::Entity, flow_inst::Column::OwnPaths)).like(format!("{}%", ctx.own_paths)));
        } else {
            query.and_where(Expr::col((flow_inst::Entity, flow_inst::Column::OwnPaths)).eq(ctx.own_paths.as_str()));
        }
        if let Some(flow_model_id) = &filter.flow_model_id {
            query.and_where(Expr::col((flow_inst::Entity, flow_inst::Column::RelFlowModelId)).eq(flow_model_id));
        }
        if let Some(tag) = &filter.tag {
            query.and_where(Expr::col((flow_inst::Entity, flow_inst::Column::Tag)).eq(tag));
        }
        if let Some(finish) = filter.finish {
            if finish {
                query.and_where(Expr::col((flow_inst::Entity, flow_inst::Column::FinishTime)).is_not_null());
            } else {
                query.and_where(Expr::col((flow_inst::Entity, flow_inst::Column::FinishTime)).is_null());
            }
        }
        if let Some(current_state_id) = &filter.current_state_id {
            query.and_where(Expr::col((flow_inst::Entity, flow_inst::Column::CurrentStateId)).eq(current_state_id));
        }

        Ok(())
    }

    pub async fn find_details(filter: &FlowInstFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<FlowInstSummaryResult>> {
        let mut query = Query::select();
        Self::package_ext_query(&mut query, filter, funs, ctx).await?;
        funs.db().find_dtos::<FlowInstSummaryResult>(&query).await
    }

    pub async fn get_inst_ids_by_rel_business_obj_id(rel_business_obj_ids: Vec<String>, funs: &TardisFunsInst, _ctx: &TardisContext) -> TardisResult<Vec<String>> {
        #[derive(sea_orm::FromQueryResult)]
        pub struct FlowInstIdsResult {
            id: String,
        }
        let result = funs
            .db()
            .find_dtos::<FlowInstIdsResult>(
                Query::select().columns([flow_inst::Column::Id]).from(flow_inst::Entity).and_where(Expr::col(flow_inst::Column::RelBusinessObjId).is_in(&rel_business_obj_ids)),
            )
            .await?
            .iter()
            .map(|rel_inst| rel_inst.id.clone())
            .collect_vec();
        Ok(result)
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

    pub async fn get(flow_inst_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<FlowInstDetailResp> {
        let mut flow_insts = Self::find_detail(vec![flow_inst_id.to_string()], funs, ctx).await?;
        if flow_insts.len() == 1 {
            Ok(flow_insts.pop().unwrap())
        } else {
            Err(funs.err().not_found("flow_inst", "get", &format!("flow instance {} not found", flow_inst_id), "404-flow-inst-not-found"))
        }
    }

    pub async fn find_detail(flow_inst_ids: Vec<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<FlowInstDetailResp>> {
        #[derive(sea_orm::FromQueryResult)]
        pub struct FlowInstDetailResult {
            pub id: String,
            pub rel_flow_model_id: String,
            pub rel_flow_model_name: String,

            pub current_state_id: String,
            pub current_state_name: Option<String>,
            pub current_state_color: Option<String>,
            pub current_state_kind: Option<FlowSysStateKind>,
            pub current_state_ext: Option<String>,

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
        let flow_state_table = Alias::new("flow_state");
        let rel_model_table = Alias::new("rel_model");
        let rbum_rel_table = Alias::new("rbum_rel");
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
            ])
            .expr_as(Expr::col((rel_state_table.clone(), NAME_FIELD.clone())).if_null(""), Alias::new("current_state_name"))
            .expr_as(Expr::col((flow_state_table.clone(), Alias::new("color"))).if_null(""), Alias::new("current_state_color"))
            .expr_as(Expr::col((flow_state_table.clone(), Alias::new("sys_state"))).if_null(""), Alias::new("current_state_kind"))
            .expr_as(Expr::col((rbum_rel_table.clone(), Alias::new("ext"))).if_null(""), Alias::new("current_state_ext"))
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
                Alias::new("flow_state"),
                flow_state_table.clone(),
                Expr::col((flow_state_table.clone(), ID_FIELD.clone())).equals((flow_inst::Entity, flow_inst::Column::CurrentStateId)),
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
            .join_as(
                JoinType::LeftJoin,
                rbum_rel_table.clone(),
                rbum_rel_table.clone(),
                Cond::all()
                    .add(Expr::col((rbum_rel_table.clone(), Alias::new("to_rbum_item_id"))).equals((flow_inst::Entity, flow_inst::Column::CurrentStateId)))
                    .add(Expr::col((rbum_rel_table.clone(), Alias::new("from_rbum_id"))).equals((flow_inst::Entity, flow_inst::Column::RelFlowModelId)))
                    .add(Expr::col((rbum_rel_table.clone(), Alias::new("tag"))).eq("FlowModelState".to_string())),
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
                current_state_color: inst.current_state_color,
                current_state_kind: inst.current_state_kind,
                current_state_ext: inst.current_state_ext.map(|ext| TardisFuns::json.str_to_obj::<FlowStateRelModelExt>(&ext).unwrap_or_default()),
                current_vars: inst.current_vars.map(|current_vars| TardisFuns::json.json_to_obj(current_vars).unwrap()),
                rel_business_obj_id: inst.rel_business_obj_id,
            })
            .collect_vec())
    }

    pub async fn paginate(
        flow_model_id: Option<String>,
        tag: Option<String>,
        finish: Option<bool>,
        current_state_id: Option<String>,
        with_sub: Option<bool>,
        page_number: u32,
        page_size: u32,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<FlowInstSummaryResp>> {
        let mut query = Query::select();
        Self::package_ext_query(
            &mut query,
            &FlowInstFilterReq {
                flow_model_id,
                tag,
                finish,
                current_state_id,
                with_sub,
            },
            funs,
            ctx,
        )
        .await?;
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
                    finish_abort: inst.finish_abort.is_some(),
                    output_message: inst.output_message,
                    own_paths: inst.own_paths,
                    current_state_id: inst.current_state_id,
                    rel_business_obj_id: inst.rel_business_obj_id,
                    tag: inst.tag,
                })
                .collect_vec(),
        })
    }

    pub(crate) async fn find_state_and_next_transitions(
        find_req: &[FlowInstFindStateAndTransitionsReq],
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
                    ids: Some(flow_insts.iter().map(|inst| inst.rel_flow_model_id.to_string()).collect::<HashSet<_>>().into_iter().collect()),
                    with_sub_own_paths: true,
                    own_paths: Some("".to_string()),
                    ..Default::default()
                },
                specified_state_ids: Some(flow_insts.iter().map(|inst| inst.current_state_id.clone()).collect::<HashSet<_>>().into_iter().collect()),
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
                    Self::do_find_next_transitions(flow_inst, flow_model, None, &req.vars, false, funs, ctx).await.unwrap()
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
        let state_and_next_transitions = Self::do_find_next_transitions(&flow_inst, &flow_model, None, &next_req.vars, false, funs, ctx).await?;
        Ok(state_and_next_transitions.next_flow_transitions)
    }

    pub async fn check_transfer_vars(flow_inst_id: &str, transfer_req: &mut FlowInstTransferReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let flow_inst_detail: FlowInstDetailResp = Self::get(flow_inst_id, funs, ctx).await?;
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
        let vars_collect = flow_model
            .transitions()
            .into_iter()
            .find(|trans| trans.id == transfer_req.flow_transition_id)
            .ok_or_else(|| funs.err().not_found("flow_inst", "check_transfer_vars", "illegal response", "404-flow-transition-not-found"))?
            .vars_collect();
        if let Some(vars_collect) = vars_collect {
            for var in vars_collect {
                if var.required == Some(true) && transfer_req.vars.as_ref().map_or(true, |map| !map.contains_key(&var.name)) {
                    return Err(funs.err().internal_error("flow_inst", "check_transfer_vars", "missing required field", "400-flow-inst-vars-field-missing"));
                }
            }
        }

        Ok(())
    }

    #[async_recursion]
    pub async fn transfer(
        flow_inst_id: &str,
        transfer_req: &FlowInstTransferReq,
        skip_filter: bool,
        callback_kind: FlowExternalCallbackOp,
        modified_instance_transations: loop_check_helper::InstancesTransition,
        ctx: &TardisContext,
    ) -> TardisResult<FlowInstTransferResp> {
        let mut funs = flow_constants::get_tardis_inst();
        let mut modified_instance_transations_cp = modified_instance_transations.clone();
        if !modified_instance_transations_cp.check(flow_inst_id.to_string(), transfer_req.flow_transition_id.clone()) {
            let flow_inst_detail = Self::get(flow_inst_id, &funs, ctx).await?;
            return Self::gen_transfer_resp(flow_inst_id, &flow_inst_detail.current_state_id, ctx, &funs).await;
        }
        funs.begin().await?;
        let result = Self::do_transfer(flow_inst_id, transfer_req, skip_filter, callback_kind, &funs, ctx).await;
        funs.commit().await?;
        let flow_inst_id_cp = flow_inst_id.to_string();
        let flow_transition_id = transfer_req.flow_transition_id.clone();
        let ctx_cp = ctx.clone();
        tardis::tokio::spawn(async move {
            let funs = flow_constants::get_tardis_inst();
            match FlowEventServ::do_post_change(&flow_inst_id_cp, &flow_transition_id, modified_instance_transations_cp.clone(), &ctx_cp, &funs).await {
                Ok(_) => {}
                Err(e) => error!("Flow Instance {} do_post_change error:{:?}", flow_inst_id_cp, e),
            }
            match FlowEventServ::do_front_change(&flow_inst_id_cp, modified_instance_transations_cp.clone(), &ctx_cp, &funs).await {
                Ok(_) => {}
                Err(e) => error!("Flow Instance {} do_front_change error:{:?}", flow_inst_id_cp, e),
            }
        });

        result
    }

    async fn do_transfer(
        flow_inst_id: &str,
        transfer_req: &FlowInstTransferReq,
        skip_filter: bool,
        callback_kind: FlowExternalCallbackOp,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<FlowInstTransferResp> {
        let global_ctx = TardisContext {
            own_paths: "".to_string(),
            ..ctx.clone()
        };
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
            skip_filter,
            funs,
            ctx,
        )
        .await?
        .next_flow_transitions
        .pop();
        if next_flow_transition.is_none() {
            return Self::gen_transfer_resp(
                flow_inst_id,
                &flow_model.transitions().into_iter().find(|trans| trans.id == transfer_req.flow_transition_id).unwrap().from_flow_state_id,
                ctx,
                funs,
            )
            .await;
        }
        let model_transition = flow_model.transitions();
        let next_transition_detail = model_transition.iter().find(|trans| trans.id == transfer_req.flow_transition_id).unwrap().to_owned();
        // if FlowModelServ::check_post_action_ring(&flow_model, funs, ctx).await? {
        //     return Err(funs.err().not_found("flow_inst", "transfer", "this post action exist endless loop", "500-flow-transition-endless-loop"));
        // }

        let next_flow_transition = next_flow_transition.unwrap();
        let prev_flow_state = FlowStateServ::get_item(
            &flow_inst_detail.current_state_id,
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
        .await?;
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
            &global_ctx,
        )
        .await?;

        // notify modify vars
        if let Some(vars) = &transfer_req.vars {
            let mut params = vec![];
            for (var_name, value) in vars {
                params.push(FlowExternalParams {
                    rel_kind: None,
                    rel_tag: None,
                    var_name: Some(var_name.clone()),
                    var_id: None,
                    value: Some(value.clone()),
                    changed_kind: None,
                });
            }
            if !params.is_empty() {
                FlowExternalServ::do_async_modify_field(
                    &flow_model.tag,
                    &next_transition_detail,
                    &flow_inst_detail.rel_business_obj_id,
                    &flow_inst_detail.id,
                    FlowExternalCallbackOp::VerifyContent,
                    next_flow_state.name.clone(),
                    next_flow_state.sys_state.clone(),
                    prev_flow_state.name.clone(),
                    prev_flow_state.sys_state.clone(),
                    params,
                    ctx,
                    funs,
                )
                .await?;
            }
        }

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
            transitions: Set(Some(new_transitions.clone())),
            ..Default::default()
        };
        if next_flow_state.sys_state == FlowSysStateKind::Finish {
            flow_inst.finish_ctx = Set(Some(FlowOperationContext::from_ctx(ctx)));
            flow_inst.finish_time = Set(Some(Utc::now()));
            flow_inst.finish_abort = Set(Some(false));
            flow_inst.output_message = Set(transfer_req.message.as_ref().map(|message| message.to_string()));
        } else {
            flow_inst.finish_ctx = Set(None);
            flow_inst.finish_time = Set(None);
        }

        funs.db().update_one(flow_inst, ctx).await?;

        // get updated instance detail
        let flow_inst_detail = Self::get(flow_inst_id, funs, ctx).await?;

        Self::do_request_webhook(
            from_transition_id.and_then(|id: String| model_transition.iter().find(|model_transition| model_transition.id == id)),
            Some(&next_transition_detail),
        )
        .await?;

        // notify change state
        FlowExternalServ::do_notify_changes(
            &flow_model.tag,
            &flow_inst_detail.id,
            &flow_inst_detail.rel_business_obj_id,
            next_flow_state.name.clone(),
            next_flow_state.sys_state.clone(),
            prev_flow_state.name.clone(),
            prev_flow_state.sys_state.clone(),
            next_transition_detail.name.clone(),
            next_transition_detail.is_notify,
            Some(callback_kind),
            ctx,
            funs,
        )
        .await?;

        Self::gen_transfer_resp(flow_inst_id, &prev_flow_state.id, ctx, funs).await
    }

    async fn gen_transfer_resp(flow_inst_id: &str, prev_flow_state_id: &str, ctx: &TardisContext, funs: &TardisFunsInst) -> TardisResult<FlowInstTransferResp> {
        let global_ctx = TardisContext {
            own_paths: "".to_string(),
            ..ctx.clone()
        };

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

        let prev_flow_state = FlowStateServ::get_item(
            prev_flow_state_id,
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
        .await?;
        let next_flow_state = FlowStateServ::get_item(
            &flow_inst_detail.current_state_id,
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
        .await?;
        let next_flow_transitions = Self::do_find_next_transitions(&flow_inst_detail, &flow_model, None, &None, false, funs, ctx).await?.next_flow_transitions;

        Ok(FlowInstTransferResp {
            prev_flow_state_id: prev_flow_state.id,
            prev_flow_state_name: prev_flow_state.name,
            prev_flow_state_color: prev_flow_state.color,
            new_flow_state_ext: TardisFuns::json.str_to_obj::<FlowStateRelModelExt>(
                &FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowModelState, &flow_inst_detail.rel_flow_model_id, None, None, funs, ctx)
                    .await?
                    .into_iter()
                    .find(|rel| next_flow_state.id == rel.rel_id)
                    .ok_or_else(|| funs.err().not_found("flow_inst", "do_find_next_transitions", "flow state is not found", "404-flow-state-not-found"))?
                    .ext,
            )?,
            new_flow_state_id: next_flow_state.id,
            new_flow_state_name: next_flow_state.name,
            new_flow_state_color: next_flow_state.color,
            finish_time: flow_inst_detail.finish_time,
            vars: flow_inst_detail.current_vars,
            next_flow_transitions,
        })
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
    pub async fn do_find_next_transitions(
        flow_inst: &FlowInstDetailResp,
        flow_model: &FlowModelDetailResp,
        spec_flow_transition_id: Option<String>,
        req_vars: &Option<HashMap<String, Value>>,
        skip_filter: bool,
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
                if skip_filter {
                    return true;
                }
                if !model_transition.guard_by_creator
                    && model_transition.guard_by_spec_account_ids.is_empty()
                    && model_transition.guard_by_spec_role_ids.is_empty()
                    && model_transition.guard_by_spec_org_ids.is_empty()
                    && !model_transition.guard_by_his_operators
                    && !model_transition.guard_by_assigned
                {
                    return true;
                }
                if model_transition.guard_by_creator && !(flow_inst.create_ctx.own_paths != ctx.own_paths || flow_inst.create_ctx.owner != ctx.owner) {
                    return true;
                }
                if !model_transition.guard_by_spec_account_ids.is_empty() && model_transition.guard_by_spec_account_ids.contains(&ctx.owner) {
                    return true;
                }
                if !model_transition.guard_by_spec_role_ids.is_empty()
                    && model_transition.guard_by_spec_role_ids.iter().any(|role_id| {
                        ctx.roles
                            .clone()
                            .into_iter()
                            .map(|ctx_role_id| ctx_role_id.split(':').collect_vec().first().unwrap_or(&"").to_string())
                            .collect_vec()
                            .contains(&role_id.split(':').collect_vec().first().unwrap_or(&"").to_string())
                    })
                {
                    return true;
                }
                if !model_transition.guard_by_spec_org_ids.is_empty() && model_transition.guard_by_spec_org_ids.iter().any(|org_id| ctx.groups.contains(org_id)) {
                    return true;
                }
                if model_transition.guard_by_assigned
                    && flow_inst.current_vars.clone().unwrap_or_default().contains_key("assigned_to")
                    && flow_inst
                        .current_vars
                        .clone()
                        .unwrap_or_default()
                        .get("assigned_to")
                        .unwrap()
                        .as_str()
                        .unwrap_or_default()
                        .split(',')
                        .collect_vec()
                        .contains(&ctx.owner.as_str())
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
            .map(|model_transition| {
                Ok(FlowInstFindNextTransitionResp {
                    next_flow_transition_id: model_transition.id.to_string(),
                    next_flow_transition_name: model_transition.name.to_string(),
                    next_flow_state_id: model_transition.to_flow_state_id.to_string(),
                    next_flow_state_name: model_transition.to_flow_state_name.to_string(),
                    next_flow_state_color: model_transition.to_flow_state_color.to_string(),
                    vars_collect: model_transition
                        .vars_collect()
                        .map(|vars| {
                            vars.into_iter()
                                .map(|mut var| {
                                    if let Some(default) = var.default_value.clone() {
                                        let default_value = match default.value_type {
                                            crate::dto::flow_var_dto::DefaultValueType::Custom => default.value,
                                            crate::dto::flow_var_dto::DefaultValueType::AssociatedAttr => {
                                                if let Some(current_vars) = flow_inst.current_vars.as_ref() {
                                                    current_vars.get(default.value.as_str().unwrap_or(&var.name)).cloned().unwrap_or_default()
                                                } else {
                                                    Value::String("".to_string())
                                                }
                                            }
                                            crate::dto::flow_var_dto::DefaultValueType::AutoFill => {
                                                match FillType::from_str(default.value.as_str().ok_or_else(|| {
                                                    funs.err().bad_request(
                                                        "flow_transitions",
                                                        "default_value_type_parse",
                                                        "AutoFill default value type is not string",
                                                        "400-flow-inst-vars-field-missing",
                                                    )
                                                })?)
                                                .map_err(|err| {
                                                    funs.err().internal_error("flow_transitions", "default_value_type_parse", &err.to_string(), "400-flow-inst-vars-field-missing")
                                                })? {
                                                    FillType::Time => Value::Number(Utc::now().timestamp_millis().into()),
                                                    FillType::Person => Value::String(ctx.owner.clone()),
                                                }
                                            }
                                        };
                                        var.dyn_default_value = Some(default_value);
                                    };
                                    Ok(var)
                                })
                                .collect::<TardisResult<Vec<_>>>()
                        })
                        .transpose()?,
                    double_check: model_transition.double_check(),
                })
            })
            .collect::<TardisResult<Vec<_>>>()?;

        let state_and_next_transitions = FlowInstFindStateAndTransitionsResp {
            flow_inst_id: flow_inst.id.to_string(),
            finish_time: flow_inst.finish_time,
            current_flow_state_name: flow_inst.current_state_name.as_ref().unwrap_or(&"".to_string()).to_string(),
            current_flow_state_color: flow_inst.current_state_color.as_ref().unwrap_or(&"".to_string()).to_string(),
            current_flow_state_kind: flow_inst.current_state_kind.as_ref().unwrap_or(&FlowSysStateKind::Start).clone(),
            current_flow_state_ext: flow_inst.current_state_ext.clone().unwrap_or_default(),
            next_flow_transitions: next_transitions,
        };
        Ok(state_and_next_transitions)
    }

    pub async fn state_is_used(flow_model_id: &str, flow_state_id: &str, funs: &TardisFunsInst, _ctx: &TardisContext) -> TardisResult<bool> {
        if funs
            .db()
            .count(
                Query::select()
                    .column((flow_inst::Entity, flow_inst::Column::Id))
                    .from(flow_inst::Entity)
                    .and_where(Expr::col((flow_inst::Entity, flow_inst::Column::CurrentStateId)).eq(flow_state_id))
                    .and_where(Expr::col((flow_inst::Entity, flow_inst::Column::RelFlowModelId)).eq(flow_model_id))
                    .and_where(
                        Expr::col((flow_inst::Entity, flow_inst::Column::FinishAbort)).ne(true).or(Expr::col((flow_inst::Entity, flow_inst::Column::FinishAbort)).is_null()),
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

    pub async fn modify_current_vars(
        flow_inst_id: &str,
        current_vars: &HashMap<String, Value>,
        modified_instance_transations: loop_check_helper::InstancesTransition,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        let flow_inst_detail = Self::get(flow_inst_id, &funs, ctx).await?;
        let mut new_vars: HashMap<String, Value> = HashMap::new();
        if let Some(old_current_vars) = &flow_inst_detail.current_vars {
            new_vars.extend(old_current_vars.clone());
        }
        new_vars.extend(current_vars.clone());
        let flow_inst = flow_inst::ActiveModel {
            id: Set(flow_inst_id.to_string()),
            current_vars: Set(Some(TardisFuns::json.obj_to_json(&new_vars)?)),
            ..Default::default()
        };
        funs.db().update_one(flow_inst, ctx).await.unwrap();
        funs.commit().await?;

        let funs = flow_constants::get_tardis_inst();

        FlowEventServ::do_front_change(flow_inst_id, modified_instance_transations, ctx, &funs).await?;

        Ok(())
    }

    async fn get_new_vars(flow_inst_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Value> {
        let flow_inst_detail = Self::find_detail(vec![flow_inst_id.to_string()], funs, ctx)
            .await?
            .into_iter()
            .next()
            .ok_or_else(|| funs.err().not_found("flow_inst", "get_new_vars", "illegal response", "404-flow-inst-not-found"))?;
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

        Ok(
            FlowExternalServ::do_query_field(&flow_model.tag, vec![flow_inst_detail.rel_business_obj_id.clone()], &flow_inst_detail.own_paths, ctx, funs)
                .await?
                .objs
                .into_iter()
                .next()
                .unwrap_or_default(),
        )
    }

    pub async fn trigger_front_action(funs: &TardisFunsInst) -> TardisResult<()> {
        #[derive(sea_orm::FromQueryResult)]
        pub struct FloTransitionsResult {
            rel_flow_model_id: String,
            action_by_front_changes: Value,
            from_flow_state_id: String,
        }
        #[derive(sea_orm::FromQueryResult)]
        pub struct FlowInstanceResult {
            id: String,
            own_paths: String,
        }

        let global_ctx = TardisContext::default();
        let flow_transition_list = funs
            .db()
            .find_dtos::<FloTransitionsResult>(
                Query::select()
                    .columns([
                        flow_transition::Column::RelFlowModelId,
                        flow_transition::Column::ActionByFrontChanges,
                        flow_transition::Column::FromFlowStateId,
                    ])
                    .from(flow_transition::Entity),
            )
            .await?
            .into_iter()
            .filter(|res| !TardisFuns::json.json_to_obj::<Vec<FlowTransitionFrontActionInfo>>(res.action_by_front_changes.clone()).unwrap_or_default().is_empty())
            .collect_vec();
        // get instance
        for flow_transition in flow_transition_list {
            let flow_insts = funs
                .db()
                .find_dtos::<FlowInstanceResult>(
                    Query::select()
                        .columns([flow_inst::Column::Id, flow_inst::Column::OwnPaths])
                        .from(flow_inst::Entity)
                        .and_where(Expr::col(flow_inst::Column::RelFlowModelId).eq(&flow_transition.rel_flow_model_id))
                        .and_where(Expr::col(flow_inst::Column::CurrentStateId).eq(&flow_transition.from_flow_state_id)),
                )
                .await?;
            for flow_inst in flow_insts {
                let ctx = TardisContext {
                    own_paths: flow_inst.own_paths,
                    ..global_ctx.clone()
                };
                let new_vars = Self::get_new_vars(&flow_inst.id, funs, &ctx).await?;
                Self::modify_current_vars(
                    &flow_inst.id,
                    &TardisFuns::json.json_to_obj::<HashMap<String, Value>>(new_vars).unwrap_or_default(),
                    loop_check_helper::InstancesTransition::default(),
                    &ctx,
                )
                .await?;
            }
        }

        Ok(())
    }

    pub async fn find_var_by_inst_id(inst_id: &str, key: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<Value>> {
        let mut current_vars = Self::get(inst_id, funs, ctx).await?.current_vars;
        if current_vars.is_none() || !current_vars.clone().unwrap_or_default().contains_key(key) {
            let new_vars = Self::get_new_vars(inst_id, funs, ctx).await?;
            Self::modify_current_vars(
                inst_id,
                &TardisFuns::json.json_to_obj::<HashMap<String, Value>>(new_vars).unwrap_or_default(),
                loop_check_helper::InstancesTransition::default(),
                ctx,
            )
            .await?;
            current_vars = Self::get(inst_id, funs, ctx).await?.current_vars;
        }

        Ok(current_vars.unwrap_or_default().get(key).cloned())
    }

    pub async fn batch_update_when_switch_model(
        rel_template_id: Option<String>,
        tag: &str,
        modify_model_id: &str,
        modify_model_states: Vec<FlowStateAggResp>,
        state_id: &str,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let mut own_paths_list = vec![];
        if let Some(rel_template_id) = rel_template_id {
            own_paths_list = FlowRelServ::find_to_simple_rels(&FlowRelKind::FlowAppTemplate, &rel_template_id, None, None, funs, ctx)
                .await?
                .into_iter()
                .map(|rel| format!("{}/{}", rel.rel_own_paths, rel.rel_id))
                .collect_vec();
            if own_paths_list.contains(&ctx.own_paths) {
                own_paths_list = vec![ctx.own_paths.clone()];
            }
        } else {
            own_paths_list.push(ctx.own_paths.clone());
        }
        for own_paths in own_paths_list {
            let mock_ctx = TardisContext { own_paths, ..ctx.clone() };
            Self::unsafe_modify_state(tag, modify_model_id, modify_model_states.clone(), state_id, funs, &mock_ctx).await?;
            Self::unsafe_modify_rel_model_id(tag, modify_model_id, funs, &mock_ctx).await?;
        }

        Ok(())
    }

    async fn unsafe_modify_rel_model_id(tag: &str, modify_model_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let mut update_statement = Query::update();
        update_statement.table(flow_inst::Entity);
        update_statement.value(flow_inst::Column::RelFlowModelId, modify_model_id);
        update_statement.and_where(Expr::col((flow_inst::Entity, flow_inst::Column::Tag)).eq(tag));
        update_statement.and_where(Expr::col((flow_inst::Entity, flow_inst::Column::OwnPaths)).eq(ctx.own_paths.as_str()));

        funs.db().execute(&update_statement).await?;

        Ok(())
    }

    pub async fn unsafe_modify_state(
        tag: &str,
        modify_model_id: &str,
        modify_model_states: Vec<FlowStateAggResp>,
        state_id: &str,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let insts = Self::find_details(
            &FlowInstFilterReq {
                tag: Some(tag.to_string()),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
        .into_iter()
        .filter(|inst| !modify_model_states.iter().any(|state| state.id == inst.current_state_id))
        .collect_vec();
        join_all(
            insts
                .iter()
                .map(|inst| async {
                    let mock_ctx = TardisContext {
                        own_paths: inst.own_paths.clone(),
                        ..ctx.clone()
                    };
                    let global_ctx = TardisContext::default();

                    let flow_inst = flow_inst::ActiveModel {
                        id: Set(inst.id.clone()),
                        current_state_id: Set(state_id.to_string()),
                        transitions: Set(Some(vec![])),
                        ..Default::default()
                    };
                    funs.db().update_one(flow_inst, &mock_ctx).await.unwrap();
                    let model_tag = FlowModelServ::get_item(
                        modify_model_id,
                        &FlowModelFilterReq {
                            basic: RbumBasicFilterReq {
                                own_paths: Some("".to_string()),
                                with_sub_own_paths: true,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        funs,
                        ctx,
                    )
                    .await
                    .map(|detail| detail.tag);
                    let next_flow_state = FlowStateServ::get_item(
                        state_id,
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
                    .unwrap();

                    FlowExternalServ::do_notify_changes(
                        model_tag.unwrap_or_default().as_str(),
                        &inst.id,
                        &inst.rel_business_obj_id,
                        "".to_string(),
                        FlowSysStateKind::default(),
                        next_flow_state.name.clone(),
                        next_flow_state.sys_state,
                        "".to_string(),
                        false,
                        Some(FlowExternalCallbackOp::Default),
                        ctx,
                        funs,
                    )
                    .await
                })
                .collect_vec(),
        )
        .await
        .into_iter()
        .collect::<TardisResult<Vec<_>>>()?;
        Ok(())
    }
}
