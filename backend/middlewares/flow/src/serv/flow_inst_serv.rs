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
use serde_json::json;
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
    domain::flow_inst,
    dto::{
        flow_external_dto::{FlowExternalCallbackOp, FlowExternalParams},
        flow_inst_dto::{
            FLowInstStateApprovalConf, FLowInstStateConf, FLowInstStateFormConf, FlowApprovalResultKind, FlowInstAbortReq, FlowInstArtifacts, FlowInstArtifactsModifyReq,
            FlowInstBatchBindReq, FlowInstBatchBindResp, FlowInstCommentInfo, FlowInstCommentReq, FlowInstDetailResp, FlowInstFilterReq, FlowInstFindNextTransitionResp,
            FlowInstFindNextTransitionsReq, FlowInstFindStateAndTransitionsReq, FlowInstFindStateAndTransitionsResp, FlowInstOperateReq, FlowInstQueryKind, FlowInstSearchPageReq,
            FlowInstSearchReq, FlowInstSearchSortReq, FlowInstStartReq, FlowInstSummaryResp, FlowInstSummaryResult, FlowInstTransferReq, FlowInstTransferResp,
            FlowInstTransitionInfo, FlowOperationContext,
        },
        flow_model_dto::FlowModelFilterReq,
        flow_model_version_dto::{FlowModelVersionDetailResp, FlowModelVersionFilterReq},
        flow_state_dto::{FLowStateKindConf, FlowStateAggResp, FlowStateFilterReq, FlowStateKind, FlowStateOperatorKind, FlowStateRelModelExt, FlowSysStateKind},
        flow_transition_dto::FlowTransitionDetailResp,
        flow_var_dto::FillType,
    },
    flow_constants,
    helper::loop_check_helper,
    serv::{flow_model_serv::FlowModelServ, flow_state_serv::FlowStateServ},
};

use super::{
    clients::search_client::FlowSearchClient,
    flow_event_serv::FlowEventServ,
    flow_external_serv::FlowExternalServ,
    flow_model_version_serv::FlowModelVersionServ,
    flow_rel_serv::{FlowRelKind, FlowRelServ},
    flow_transition_serv::FlowTransitionServ,
};

pub struct FlowInstServ;

impl FlowInstServ {
    pub async fn start(start_req: &FlowInstStartReq, current_state_name: Option<String>, _funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        // get model by own_paths
        let flow_model = if let Some(transition_id) = &start_req.transition_id {
            FlowModelServ::get_model_id_by_own_paths_and_transition_id(&start_req.tag, transition_id, &funs, ctx).await?
        } else {
            FlowModelServ::get_model_id_by_own_paths_and_rel_template_id(&start_req.tag, None, &funs, ctx).await?
        };
        let inst_id = TardisFuns::field.nanoid();
        let current_state_id = if let Some(current_state_name) = &current_state_name {
            if current_state_name.is_empty() {
                flow_model.init_state_id.clone()
            } else {
                FlowStateServ::match_state_id_by_name(&flow_model.current_version_id, current_state_name, &funs, ctx).await?
            }
        } else {
            flow_model.init_state_id.clone()
        };
        if !Self::find_details(
            &FlowInstFilterReq {
                rel_business_obj_ids: Some(vec![start_req.rel_business_obj_id.to_string()]),
                flow_version_id: Some(flow_model.current_version_id.clone()),
                finish: Some(false),
                ..Default::default()
            },
            &funs,
            ctx,
        )
        .await?
        .is_empty()
        {
            return Err(funs.err().internal_error("flow_inst_serv", "start", "The same instance exist", "500-flow-inst-exist"));
        }
        let main = start_req.transition_id.is_none();
        let flow_inst: flow_inst::ActiveModel = flow_inst::ActiveModel {
            id: Set(inst_id.clone()),
            tag: Set(Some(flow_model.tag.clone())),
            rel_flow_version_id: Set(flow_model.current_version_id.clone()),
            rel_business_obj_id: Set(start_req.rel_business_obj_id.to_string()),

            current_state_id: Set(current_state_id.clone()),

            create_vars: Set(start_req.create_vars.as_ref().map(|vars| TardisFuns::json.obj_to_json(vars).unwrap())),
            create_ctx: Set(FlowOperationContext::from_ctx(ctx)),

            own_paths: Set(ctx.own_paths.to_string()),
            main: Set(main),
            ..Default::default()
        };
        funs.db().insert_one(flow_inst, ctx).await?;
        let inst = Self::get(&inst_id, &funs, ctx).await?;
        Self::when_enter_state(&inst, &current_state_id, &flow_model.id, &funs, ctx).await?;
        if !main {
            FlowSearchClient::modify_business_obj_search(&start_req.rel_business_obj_id, &flow_model.tag, &funs, ctx).await?;
        }
        Self::do_request_webhook(
            None,
            flow_model.transitions().iter().filter(|model_transition| model_transition.to_flow_state_id == flow_model.init_state_id).collect_vec().pop(),
        )
        .await?;
        funs.commit().await?;
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        // 自动流转
        Self::auto_transfer(&inst, loop_check_helper::InstancesTransition::default(), &funs, ctx).await?;
        funs.commit().await?;

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
            let flow_model = if let Some(transition_id) = &batch_bind_req.transition_id {
                FlowModelServ::get_model_id_by_own_paths_and_transition_id(&batch_bind_req.tag, transition_id, funs, ctx).await?
            } else {
                FlowModelServ::get_model_id_by_own_paths_and_rel_template_id(&batch_bind_req.tag, None, funs, ctx).await?
            };

            let current_state_id = FlowStateServ::match_state_id_by_name(&flow_model.current_version_id, &rel_business_obj.current_state_name.clone().unwrap_or_default(), funs, ctx).await?;
            let mut inst_id = Self::get_inst_ids_by_rel_business_obj_id(vec![rel_business_obj.rel_business_obj_id.clone().unwrap_or_default()], Some(true), funs, ctx).await?.pop();
            if inst_id.is_none() {
                let id = TardisFuns::field.nanoid();
                let flow_inst: flow_inst::ActiveModel = flow_inst::ActiveModel {
                    id: Set(id.clone()),
                    rel_flow_version_id: Set(flow_model.current_version_id.to_string()),
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
        let flow_model_version_table = Alias::new("flow_model_version");
        query
            .columns([
                (flow_inst::Entity, flow_inst::Column::Id),
                (flow_inst::Entity, flow_inst::Column::RelFlowVersionId),
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
            .expr_as(
                Expr::col((flow_model_version_table.clone(), Alias::new("rel_model_id"))).if_null(""),
                Alias::new("rel_flow_model_id"),
            )
            .expr_as(Expr::col((RBUM_ITEM_TABLE.clone(), NAME_FIELD.clone())).if_null(""), Alias::new("rel_flow_model_name"))
            .from(flow_inst::Entity)
            .left_join(
                flow_model_version_table.clone(),
                Expr::col((flow_model_version_table.clone(), ID_FIELD.clone())).equals((flow_inst::Entity, flow_inst::Column::RelFlowVersionId)),
            )
            .left_join(
                RBUM_ITEM_TABLE.clone(),
                Expr::col((RBUM_ITEM_TABLE.clone(), ID_FIELD.clone())).equals((flow_inst::Entity, flow_inst::Column::RelFlowVersionId)),
            );
        if let Some(ids) = &filter.ids {
            query.and_where(Expr::col((flow_inst::Entity, flow_inst::Column::Id)).is_in(ids));
        }
        if filter.with_sub.unwrap_or(false) {
            query.and_where(Expr::col((flow_inst::Entity, flow_inst::Column::OwnPaths)).like(format!("{}%", ctx.own_paths)));
        } else {
            query.and_where(Expr::col((flow_inst::Entity, flow_inst::Column::OwnPaths)).eq(ctx.own_paths.as_str()));
        }
        if let Some(flow_version_id) = &filter.flow_version_id {
            query.and_where(Expr::col((flow_inst::Entity, flow_inst::Column::RelFlowVersionId)).eq(flow_version_id));
        }
        if let Some(tag) = &filter.tag {
            query.and_where(Expr::col((flow_inst::Entity, flow_inst::Column::Tag)).eq(tag));
        }
        if let Some(main) = filter.main {
            query.and_where(Expr::col((flow_inst::Entity, flow_inst::Column::Main)).eq(main));
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
        if let Some(rel_business_obj_ids) = &filter.rel_business_obj_ids {
            query.and_where(Expr::col((flow_inst::Entity, flow_inst::Column::RelBusinessObjId)).is_in(rel_business_obj_ids));
        }

        Ok(())
    }

    pub async fn find_details(filter: &FlowInstFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<FlowInstSummaryResult>> {
        let mut query = Query::select();
        Self::package_ext_query(&mut query, filter, funs, ctx).await?;
        funs.db().find_dtos::<FlowInstSummaryResult>(&query).await
    }

    pub async fn get_inst_ids_by_rel_business_obj_id(
        rel_business_obj_ids: Vec<String>,
        main: Option<bool>,
        funs: &TardisFunsInst,
        _ctx: &TardisContext,
    ) -> TardisResult<Vec<String>> {
        #[derive(sea_orm::FromQueryResult)]
        pub struct FlowInstIdsResult {
            id: String,
        }
        let result = funs
            .db()
            .find_dtos::<FlowInstIdsResult>(
                Query::select()
                    .columns([flow_inst::Column::Id])
                    .from(flow_inst::Entity)
                    .and_where(Expr::col(flow_inst::Column::RelBusinessObjId).is_in(&rel_business_obj_ids))
                    .and_where(Expr::col(flow_inst::Column::Main).eq(main)),
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
            pub tag: String,
            pub rel_flow_version_id: String,
            pub rel_flow_model_name: String,
            pub main: bool,

            pub current_state_id: String,
            pub current_state_name: Option<String>,
            pub current_state_color: Option<String>,
            pub current_state_sys_kind: Option<FlowSysStateKind>,
            pub current_state_kind: Option<FlowStateKind>,
            pub current_state_kind_conf: Option<Value>,
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
            pub artifacts: Option<Value>,
            pub comments: Option<Value>,

            pub own_paths: String,

            pub rel_business_obj_id: String,
        }
        let rel_state_table = Alias::new("rel_state");
        let flow_state_table = Alias::new("flow_state");
        let rel_model_version_table = Alias::new("rel_model_version");
        let rbum_rel_table = Alias::new("rbum_rel");
        let mut query = Query::select();
        query
            .columns([
                (flow_inst::Entity, flow_inst::Column::Id),
                (flow_inst::Entity, flow_inst::Column::Tag),
                (flow_inst::Entity, flow_inst::Column::RelFlowVersionId),
                (flow_inst::Entity, flow_inst::Column::RelBusinessObjId),
                (flow_inst::Entity, flow_inst::Column::CurrentStateId),
                (flow_inst::Entity, flow_inst::Column::Main),
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
                (flow_inst::Entity, flow_inst::Column::Artifacts),
            ])
            .expr_as(Expr::col((rel_state_table.clone(), NAME_FIELD.clone())).if_null(""), Alias::new("current_state_name"))
            .expr_as(Expr::col((flow_state_table.clone(), Alias::new("color"))).if_null(""), Alias::new("current_state_color"))
            .expr_as(
                Expr::col((flow_state_table.clone(), Alias::new("sys_state"))).if_null(FlowSysStateKind::Start),
                Alias::new("current_state_sys_kind"),
            )
            .expr_as(
                Expr::col((flow_state_table.clone(), Alias::new("state_kind"))).if_null(FlowStateKind::Simple),
                Alias::new("current_state_kind"),
            )
            .expr_as(
                Expr::col((flow_state_table.clone(), Alias::new("kind_conf"))).if_null(json!({})),
                Alias::new("current_state_kind_conf"),
            )
            .expr_as(Expr::col((rbum_rel_table.clone(), Alias::new("ext"))).if_null(""), Alias::new("current_state_ext"))
            .expr_as(
                Expr::col((rel_model_version_table.clone(), NAME_FIELD.clone())).if_null(""),
                Alias::new("rel_flow_model_name"),
            )
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
                rel_model_version_table.clone(),
                Cond::all()
                    .add(Expr::col((rel_model_version_table.clone(), ID_FIELD.clone())).equals((flow_inst::Entity, flow_inst::Column::RelFlowVersionId)))
                    .add(Expr::col((rel_model_version_table.clone(), REL_KIND_ID_FIELD.clone())).eq(FlowModelVersionServ::get_rbum_kind_id().unwrap()))
                    .add(Expr::col((rel_model_version_table.clone(), REL_DOMAIN_ID_FIELD.clone())).eq(FlowModelVersionServ::get_rbum_domain_id().unwrap())),
            )
            .join_as(
                JoinType::LeftJoin,
                rbum_rel_table.clone(),
                rbum_rel_table.clone(),
                Cond::all()
                    .add(Expr::col((rbum_rel_table.clone(), Alias::new("to_rbum_item_id"))).equals((flow_inst::Entity, flow_inst::Column::CurrentStateId)))
                    .add(Expr::col((rbum_rel_table.clone(), Alias::new("from_rbum_id"))).equals((flow_inst::Entity, flow_inst::Column::RelFlowVersionId)))
                    .add(Expr::col((rbum_rel_table.clone(), Alias::new("tag"))).eq("FlowModelState".to_string())),
            )
            .and_where(Expr::col((flow_inst::Entity, flow_inst::Column::Id)).is_in(flow_inst_ids))
            .and_where(Expr::col((flow_inst::Entity, flow_inst::Column::OwnPaths)).like(format!("{}%", ctx.own_paths)));

        let flow_insts = funs.db().find_dtos::<FlowInstDetailResult>(&query).await?;
        Ok(flow_insts
            .into_iter()
            .map(|inst| {
                let current_state_kind_conf = inst
                    .current_state_kind_conf
                    .clone()
                    .map(|current_state_kind_conf| TardisFuns::json.json_to_obj::<FLowStateKindConf>(current_state_kind_conf).unwrap_or_default());
                let artifacts = inst.artifacts.clone().map(|artifacts| TardisFuns::json.json_to_obj::<FlowInstArtifacts>(artifacts).unwrap_or_default());
                FlowInstDetailResp {
                    id: inst.id,
                    rel_flow_version_id: inst.rel_flow_version_id,
                    rel_flow_model_name: inst.rel_flow_model_name,
                    tag: inst.tag,
                    main: inst.main,
                    create_vars: inst.create_vars.map(|create_vars| TardisFuns::json.json_to_obj(create_vars).unwrap()),
                    create_ctx: inst.create_ctx,
                    create_time: inst.create_time,
                    finish_ctx: inst.finish_ctx,
                    finish_time: inst.finish_time,
                    finish_abort: inst.finish_abort,
                    output_message: inst.output_message,
                    own_paths: inst.own_paths,
                    transitions: inst.transitions.map(|transitions| TardisFuns::json.json_to_obj(transitions).unwrap()),
                    artifacts: inst.artifacts.map(|artifacts| TardisFuns::json.json_to_obj(artifacts).unwrap()),
                    comments: inst.comments.map(|comments| TardisFuns::json.json_to_obj(comments).unwrap()),
                    current_state_id: inst.current_state_id.clone(),
                    current_state_name: inst.current_state_name,
                    current_state_color: inst.current_state_color,
                    current_state_sys_kind: inst.current_state_sys_kind,
                    current_state_kind: inst.current_state_kind.clone(),
                    current_state_ext: inst.current_state_ext.map(|ext| TardisFuns::json.str_to_obj::<FlowStateRelModelExt>(&ext).unwrap_or_default()),
                    current_state_conf: Self::get_state_conf(
                        &inst.current_state_id,
                        &inst.current_state_kind.unwrap_or_default(),
                        current_state_kind_conf,
                        artifacts,
                        ctx,
                    ),
                    current_vars: inst.current_vars.map(|current_vars| TardisFuns::json.json_to_obj(current_vars).unwrap()),
                    rel_business_obj_id: inst.rel_business_obj_id,
                }
            })
            .collect_vec())
    }

    pub async fn paginate(
        flow_version_id: Option<String>,
        tag: Option<String>,
        finish: Option<bool>,
        main: Option<bool>,
        current_state_id: Option<String>,
        rel_business_obj_id: Option<String>,
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
                flow_version_id,
                tag,
                finish,
                main,
                current_state_id,
                rel_business_obj_ids: rel_business_obj_id.map(|id| vec![id]),
                with_sub,
                ..Default::default()
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
                    rel_flow_version_id: inst.rel_flow_version_id,
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
        let flow_model_versions = FlowModelVersionServ::find_detail_items(
            &FlowModelVersionFilterReq {
                basic: RbumBasicFilterReq {
                    ids: Some(flow_insts.iter().map(|inst| inst.rel_flow_version_id.to_string()).collect::<HashSet<_>>().into_iter().collect()),
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
        let mut rel_flow_version_map = HashMap::new();
        for flow_inst in flow_insts.iter() {
            if !rel_flow_version_map.contains_key(&flow_inst.tag) {
                let rel_flow_versions = FlowTransitionServ::find_rel_model_map(&flow_inst.tag, funs, ctx).await?;
                rel_flow_version_map.insert(flow_inst.tag.clone(), rel_flow_versions);
            }
        }
        let mut state_and_next_transitions = join_all(
            flow_insts
                .iter()
                .map(|flow_inst| async {
                    let req = find_req.iter().find(|req| req.flow_inst_id == flow_inst.id).unwrap();
                    let flow_model_version = flow_model_versions.iter().find(|version| version.id == flow_inst.rel_flow_version_id).unwrap();
                    let rel_flow_versions = rel_flow_version_map.get(&flow_inst.tag).unwrap().clone();
                    Self::do_find_next_transitions(flow_inst, flow_model_version, None, &req.vars, rel_flow_versions, false, funs, ctx).await.unwrap()
                })
                .collect_vec(),
        )
        .await;
        // 若当前数据项存在未结束的审批流，则清空其中的transitions
        let unfinished_approve_flow_insts = Self::find_details(
            &FlowInstFilterReq {
                rel_business_obj_ids: Some(flow_insts.iter().map(|flow_inst| flow_inst.rel_business_obj_id.clone()).collect_vec()),
                main: Some(false),
                finish: Some(false),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
        .into_iter()
        .map(|inst| inst.id.clone())
        .collect_vec();
        let _ = state_and_next_transitions.iter_mut().map(|item| {
            if unfinished_approve_flow_insts.contains(&item.flow_inst_id) {
                item.next_flow_transitions.clear();
            }
        });
        Ok(state_and_next_transitions)
    }

    pub async fn find_next_transitions(
        flow_inst: &FlowInstDetailResp,
        next_req: &FlowInstFindNextTransitionsReq,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<FlowInstFindNextTransitionResp>> {
        let flow_model_version = FlowModelVersionServ::get_item(
            &flow_inst.rel_flow_version_id,
            &FlowModelVersionFilterReq {
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
        let rel_flow_versions = FlowTransitionServ::find_rel_model_map(&flow_inst.tag, funs, ctx).await?;
        let state_and_next_transitions = Self::do_find_next_transitions(flow_inst, &flow_model_version, None, &next_req.vars, rel_flow_versions, false, funs, ctx).await?;
        Ok(state_and_next_transitions.next_flow_transitions)
    }

    pub async fn check_transfer_vars(
        flow_inst_detail: &FlowInstDetailResp,
        transfer_req: &mut FlowInstTransferReq,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let flow_model_version = FlowModelVersionServ::get_item(
            &flow_inst_detail.rel_flow_version_id,
            &FlowModelVersionFilterReq {
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
        let vars_collect = FlowTransitionServ::find_transitions(&flow_model_version.id, None, funs, ctx)
            .await?
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
        flow_inst_detail: &FlowInstDetailResp,
        transfer_req: &FlowInstTransferReq,
        skip_filter: bool,
        callback_kind: FlowExternalCallbackOp,
        modified_instance_transations: loop_check_helper::InstancesTransition,
        ctx: &TardisContext,
        funs: &TardisFunsInst,
    ) -> TardisResult<FlowInstTransferResp> {
        let mut modified_instance_transations_cp = modified_instance_transations.clone();
        if !modified_instance_transations_cp.check(flow_inst_detail.id.clone(), transfer_req.flow_transition_id.clone()) {
            return Self::gen_transfer_resp(flow_inst_detail, &flow_inst_detail.current_state_id, ctx, funs).await;
        }

        let result = Self::do_transfer(flow_inst_detail, transfer_req, skip_filter, callback_kind, funs, ctx).await;
        Self::auto_transfer(flow_inst_detail, modified_instance_transations_cp.clone(), funs, ctx).await?;
        let flow_inst_id_cp = flow_inst_detail.id.clone();
        let flow_transition_id = transfer_req.flow_transition_id.clone();
        let ctx_cp = ctx.clone();
        tardis::tokio::spawn(async move {
            let mut funs = flow_constants::get_tardis_inst();
            let flow_inst_cp = Self::get(&flow_inst_id_cp, &funs, &ctx_cp).await.unwrap();
            funs.begin().await.unwrap();
            match FlowEventServ::do_post_change(&flow_inst_cp, &flow_transition_id, modified_instance_transations_cp.clone(), &ctx_cp, &funs).await {
                Ok(_) => {}
                Err(e) => error!("Flow Instance {} do_post_change error:{:?}", flow_inst_id_cp, e),
            }
            match FlowEventServ::do_front_change(&flow_inst_cp, modified_instance_transations_cp.clone(), &ctx_cp, &funs).await {
                Ok(_) => {}
                Err(e) => error!("Flow Instance {} do_front_change error:{:?}", flow_inst_id_cp, e),
            }
            funs.commit().await.unwrap();
        });

        result
    }

    async fn do_transfer(
        flow_inst_detail: &FlowInstDetailResp,
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
        let flow_model_version = FlowModelVersionServ::get_item(
            &flow_inst_detail.rel_flow_version_id,
            &FlowModelVersionFilterReq {
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
        let rel_flow_versions = FlowTransitionServ::find_rel_model_map(&flow_inst_detail.tag, funs, ctx).await?;
        let next_flow_transition = Self::do_find_next_transitions(
            flow_inst_detail,
            &flow_model_version,
            Some(transfer_req.flow_transition_id.to_string()),
            &transfer_req.vars,
            rel_flow_versions,
            skip_filter,
            funs,
            ctx,
        )
        .await?
        .next_flow_transitions
        .pop();
        if next_flow_transition.is_none() {
            return Self::gen_transfer_resp(
                flow_inst_detail,
                &FlowTransitionServ::find_transitions(&flow_model_version.id, None, funs, ctx)
                    .await?
                    .into_iter()
                    .find(|trans| trans.id == transfer_req.flow_transition_id)
                    .unwrap()
                    .from_flow_state_id,
                ctx,
                funs,
            )
            .await;
        }
        let version_transition = FlowTransitionServ::find_transitions(&flow_model_version.id, None, funs, ctx).await?;
        let next_transition_detail = version_transition.iter().find(|trans| trans.id == transfer_req.flow_transition_id).unwrap().to_owned();

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
            if !params.is_empty() && flow_inst_detail.main {
                FlowExternalServ::do_async_modify_field(
                    &flow_inst_detail.tag,
                    Some(next_transition_detail.clone()),
                    &flow_inst_detail.rel_business_obj_id,
                    &flow_inst_detail.id,
                    Some(FlowExternalCallbackOp::VerifyContent),
                    Some(next_flow_state.name.clone()),
                    Some(next_flow_state.sys_state.clone()),
                    Some(prev_flow_state.name.clone()),
                    Some(prev_flow_state.sys_state.clone()),
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
            from_state_id: Some(prev_flow_state.id.clone()),
            from_state_name: Some(prev_flow_state.name.clone()),
            target_state_id: Some(next_flow_state.id.clone()),
            target_state_name: Some(next_flow_state.name.clone()),
        });

        let mut flow_inst = flow_inst::ActiveModel {
            id: Set(flow_inst_detail.id.clone()),
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

        Self::when_leave_state(flow_inst_detail, &prev_flow_state.id, &flow_model_version.rel_model_id, funs, ctx).await?;
        Self::when_enter_state(flow_inst_detail, &next_flow_state.id, &flow_model_version.rel_model_id, funs, ctx).await?;

        Self::do_request_webhook(
            from_transition_id.and_then(|id| version_transition.iter().find(|model_transition| model_transition.id == id)),
            Some(&next_transition_detail),
        )
        .await?;

        // notify change state
        if flow_inst_detail.main {
            FlowExternalServ::do_notify_changes(
                &flow_inst_detail.tag,
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
        }

        Self::gen_transfer_resp(flow_inst_detail, &prev_flow_state.id, ctx, funs).await
    }

    async fn gen_transfer_resp(flow_inst_detail: &FlowInstDetailResp, prev_flow_state_id: &str, ctx: &TardisContext, funs: &TardisFunsInst) -> TardisResult<FlowInstTransferResp> {
        let global_ctx = TardisContext {
            own_paths: "".to_string(),
            ..ctx.clone()
        };

        let flow_model_version = FlowModelVersionServ::get_item(
            &flow_inst_detail.rel_flow_version_id,
            &FlowModelVersionFilterReq {
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
        let rel_flow_versions = FlowTransitionServ::find_rel_model_map(&flow_inst_detail.tag, funs, ctx).await?;
        let next_flow_transitions =
            Self::do_find_next_transitions(flow_inst_detail, &flow_model_version, None, &None, rel_flow_versions, false, funs, ctx).await?.next_flow_transitions;

        Ok(FlowInstTransferResp {
            prev_flow_state_id: prev_flow_state.id,
            prev_flow_state_name: prev_flow_state.name,
            prev_flow_state_color: prev_flow_state.color,
            new_flow_state_ext: TardisFuns::json.str_to_obj::<FlowStateRelModelExt>(
                &FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowModelState, &flow_inst_detail.rel_flow_version_id, None, None, funs, ctx)
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
            vars: flow_inst_detail.current_vars.clone(),
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
        flow_model_version: &FlowModelVersionDetailResp,
        spec_flow_transition_id: Option<String>,
        req_vars: &Option<HashMap<String, Value>>,
        rel_flow_versions: HashMap<String, String>,
        skip_filter: bool,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<FlowInstFindStateAndTransitionsResp> {
        let flow_model_transitions = FlowTransitionServ::find_transitions(&flow_model_version.id, None, funs, ctx).await?;

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
            current_flow_state_sys_kind: flow_inst.current_state_sys_kind.as_ref().unwrap_or(&FlowSysStateKind::Start).clone(),
            current_flow_state_ext: flow_inst.current_state_ext.clone().unwrap_or_default(),
            next_flow_transitions: next_transitions,
            rel_flow_versions,
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
                    .and_where(Expr::col((flow_inst::Entity, flow_inst::Column::RelFlowVersionId)).eq(flow_model_id))
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
        flow_inst_detail: &FlowInstDetailResp,
        current_vars: &HashMap<String, Value>,
        modified_instance_transations: loop_check_helper::InstancesTransition,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        let mut new_vars: HashMap<String, Value> = HashMap::new();
        if let Some(old_current_vars) = &flow_inst_detail.current_vars {
            new_vars.extend(old_current_vars.clone());
        }
        new_vars.extend(current_vars.clone());
        let flow_inst = flow_inst::ActiveModel {
            id: Set(flow_inst_detail.id.clone()),
            current_vars: Set(Some(TardisFuns::json.obj_to_json(&new_vars)?)),
            ..Default::default()
        };
        funs.db().update_one(flow_inst, ctx).await.unwrap();
        funs.commit().await?;

        let funs = flow_constants::get_tardis_inst();

        FlowEventServ::do_front_change(flow_inst_detail, modified_instance_transations, ctx, &funs).await?;

        Ok(())
    }

    async fn get_new_vars(flow_inst_detail: &FlowInstDetailResp, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Value> {
        let flow_model_version = FlowModelVersionServ::get_item(
            &flow_inst_detail.rel_flow_version_id,
            &FlowModelVersionFilterReq {
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
        let flow_model = FlowModelServ::get_item(
            &flow_model_version.rel_model_id,
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

    pub async fn find_var_by_inst_id(flow_inst: &FlowInstDetailResp, key: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<Value>> {
        let mut current_vars = flow_inst.current_vars.clone();
        if current_vars.is_none() || !current_vars.clone().unwrap_or_default().contains_key(key) {
            let new_vars = Self::get_new_vars(flow_inst, funs, ctx).await?;
            Self::modify_current_vars(
                flow_inst,
                &TardisFuns::json.json_to_obj::<HashMap<String, Value>>(new_vars).unwrap_or_default(),
                loop_check_helper::InstancesTransition::default(),
                ctx,
            )
            .await?;
            current_vars = Self::get(&flow_inst.id, funs, ctx).await?.current_vars;
        }

        Ok(current_vars.unwrap_or_default().get(key).cloned())
    }

    pub async fn batch_update_when_switch_model(
        rel_template_id: Option<String>,
        tag: &str,
        modify_version_id: &str,
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
            Self::unsafe_modify_state(tag, modify_model_states.clone(), state_id, funs, &mock_ctx).await?;
            Self::unsafe_modify_rel_model_id(tag, modify_version_id, funs, &mock_ctx).await?;
        }

        Ok(())
    }

    async fn unsafe_modify_rel_model_id(tag: &str, modify_version_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let mut update_statement = Query::update();
        update_statement.table(flow_inst::Entity);
        update_statement.value(flow_inst::Column::RelFlowVersionId, modify_version_id);
        update_statement.and_where(Expr::col((flow_inst::Entity, flow_inst::Column::Tag)).eq(tag));
        update_statement.and_where(Expr::col((flow_inst::Entity, flow_inst::Column::OwnPaths)).eq(ctx.own_paths.as_str()));

        funs.db().execute(&update_statement).await?;

        Ok(())
    }

    pub async fn unsafe_modify_state(tag: &str, modify_model_states: Vec<FlowStateAggResp>, state_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
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
                        &inst.tag,
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

    pub async fn auto_transfer(
        flow_inst_detail: &FlowInstDetailResp,
        modified_instance_transations: loop_check_helper::InstancesTransition,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let model_version = FlowModelVersionServ::get_item(
            &flow_inst_detail.rel_flow_version_id,
            &FlowModelVersionFilterReq {
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
        let transition_ids = Self::do_find_next_transitions(flow_inst_detail, &model_version, None, &None, HashMap::default(), false, funs, ctx)
            .await?
            .next_flow_transitions
            .into_iter()
            .map(|tran| tran.next_flow_transition_id)
            .collect_vec();
        let current_var = flow_inst_detail.current_vars.clone().unwrap_or_default();
        let auto_transition = FlowTransitionServ::find_detail_items(transition_ids, None, None, funs, ctx).await?.into_iter().find(|transition| {
            (transition.transfer_by_auto && transition.guard_by_other_conds().is_none())
                || (transition.transfer_by_auto
                    && transition.guard_by_other_conds().is_some()
                    && BasicQueryCondInfo::check_or_and_conds(&transition.guard_by_other_conds().unwrap(), &current_var).unwrap())
        });
        if let Some(auto_transition) = auto_transition {
            Self::transfer(
                flow_inst_detail,
                &FlowInstTransferReq {
                    flow_transition_id: auto_transition.id,
                    message: None,
                    vars: None,
                },
                false,
                FlowExternalCallbackOp::Auto,
                modified_instance_transations.clone(),
                ctx,
                funs,
            )
            .await?;
        }

        Ok(())
    }

    // 当进入该节点时
    async fn when_enter_state(flow_inst_detail: &FlowInstDetailResp, state_id: &str, flow_model_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let rel_transition =
            FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowModelTransition, flow_model_id, None, None, funs, ctx).await?.pop().map(|rel| rel.rel_id).unwrap_or_default();
        let state = FlowStateServ::get_item(
            state_id,
            &FlowStateFilterReq {
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
        match state.state_kind {
            FlowStateKind::Start => {}
            FlowStateKind::Form => {
                let mut modify_req = FlowInstArtifactsModifyReq { ..Default::default() };
                let form_conf = state.kind_conf().unwrap_or_default().form.unwrap_or_default();
                let mut guard_custom_conf = form_conf.guard_custom_conf.unwrap_or_default();
                if form_conf.guard_by_creator {
                    guard_custom_conf.guard_by_spec_account_ids.push(ctx.owner.clone());
                }
                if form_conf.guard_by_his_operators {
                    flow_inst_detail.transitions.as_ref().map(|transitions| {
                        transitions.iter().map(|transition| guard_custom_conf.guard_by_spec_account_ids.push(transition.op_ctx.owner.clone())).collect::<Vec<_>>()
                    });
                }
                modify_req.guard_conf = Some(guard_custom_conf);
                Self::modify_inst_artifacts(&flow_inst_detail.id, &modify_req, funs, ctx).await?;
            }
            FlowStateKind::Approval => {
                let mut modify_req = FlowInstArtifactsModifyReq { ..Default::default() };
                let approval_conf = state.kind_conf().unwrap_or_default().approval.unwrap_or_default();
                let mut guard_custom_conf = approval_conf.guard_custom_conf.unwrap_or_default();
                if approval_conf.guard_by_creator {
                    guard_custom_conf.guard_by_spec_account_ids.push(ctx.owner.clone());
                }
                if approval_conf.guard_by_his_operators {
                    flow_inst_detail.transitions.as_ref().map(|transitions| {
                        transitions.iter().map(|transition| guard_custom_conf.guard_by_spec_account_ids.push(transition.op_ctx.owner.clone())).collect::<Vec<_>>()
                    });
                }
                modify_req.guard_conf = Some(guard_custom_conf);
                Self::modify_inst_artifacts(&flow_inst_detail.id, &modify_req, funs, ctx).await?;
            }
            FlowStateKind::Branch => {}
            FlowStateKind::Finish => match rel_transition.as_str() {
                "__EDIT__" => {
                    let transition = FlowTransitionServ::find_transitions(&flow_inst_detail.rel_flow_version_id, None, funs, ctx).await?;
                    let mut state_ids = vec![];
                    let mut vars_collect = HashMap::new();
                    let artifacts = flow_inst_detail.artifacts.clone().unwrap_or_default();
                    for tran in flow_inst_detail.transitions.clone().unwrap_or_default() {
                        if let Some(tran) = transition.iter().find(|version_tran| tran.id == version_tran.id) {
                            let current_state_id = tran.to_flow_state_id.clone();
                            if !state_ids.contains(&current_state_id) {
                                state_ids.push(current_state_id.clone());
                                if let Some(form_state_vars) = artifacts.form_state_map.get(&current_state_id) {
                                    for (key, value) in form_state_vars {
                                        *vars_collect.entry(key.clone()).or_insert(json!({})) = value.clone();
                                    }
                                }
                            }
                        }
                    }
                    let params = vars_collect
                        .into_iter()
                        .map(|(key, value)| FlowExternalParams {
                            var_name: Some(key),
                            value: Some(value),
                            ..Default::default()
                        })
                        .collect_vec();
                    FlowExternalServ::do_async_modify_field(
                        &flow_inst_detail.tag,
                        None,
                        &flow_inst_detail.rel_business_obj_id,
                        &flow_inst_detail.id,
                        None,
                        None,
                        None,
                        None,
                        None,
                        params,
                        ctx,
                        funs,
                    )
                    .await?;
                }
                "__DELETE__" => {
                    FlowExternalServ::do_delete_rel_obj(&flow_inst_detail.tag, &flow_inst_detail.rel_business_obj_id, &flow_inst_detail.id, ctx, funs).await?;
                }
                _ => {
                    if let Some(inst_id) = Self::get_inst_ids_by_rel_business_obj_id(vec![flow_inst_detail.rel_business_obj_id.clone()], Some(true), funs, ctx).await?.pop() {
                        let inst_detail = Self::get(&inst_id, funs, ctx).await?;
                        Self::transfer(
                            &inst_detail,
                            &FlowInstTransferReq {
                                flow_transition_id: rel_transition.clone(),
                                message: None,
                                vars: None,
                            },
                            true,
                            FlowExternalCallbackOp::Default,
                            loop_check_helper::InstancesTransition::default(),
                            ctx,
                            funs,
                        )
                        .await?;
                    }
                }
            },
            _ => {}
        }
        Ok(())
    }

    // 当离开该节点时
    async fn when_leave_state(flow_inst_detail: &FlowInstDetailResp, state_id: &str, _flow_model_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        // let rel_transition = FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowModelTransition, flow_model_id, None, None, funs, ctx).await?.pop().map(|rel| rel.rel_id).unwrap_or_default();
        let state = FlowStateServ::get_item(
            state_id,
            &FlowStateFilterReq {
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
        match state.state_kind {
            FlowStateKind::Start => {}
            FlowStateKind::Form => {
                let modify_req = FlowInstArtifactsModifyReq {
                    prev_non_auto_state_id: Some(state_id.to_string()),
                    prev_non_auto_account_id: Some(ctx.owner.clone()),
                    ..Default::default()
                };
                Self::modify_inst_artifacts(&flow_inst_detail.id, &modify_req, funs, ctx).await?;
            }
            FlowStateKind::Approval => {
                let modify_req = FlowInstArtifactsModifyReq {
                    prev_non_auto_state_id: Some(state_id.to_string()),
                    prev_non_auto_account_id: Some(ctx.owner.clone()),
                    ..Default::default()
                };
                Self::modify_inst_artifacts(&flow_inst_detail.id, &modify_req, funs, ctx).await?;
            }
            FlowStateKind::Branch => {}
            FlowStateKind::Finish => {}
            _ => {}
        }
        Ok(())
    }

    // 修改实例的数据对象
    async fn modify_inst_artifacts(inst_id: &str, modify_artifacts: &FlowInstArtifactsModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let inst = Self::get(inst_id, funs, ctx).await?;
        let mut inst_artifacts = inst.artifacts.unwrap_or_default();
        if let Some(guard_conf) = &modify_artifacts.guard_conf {
            inst_artifacts.guard_conf = guard_conf.clone();
        }
        if let Some(add_guard_conf_account_id) = &modify_artifacts.add_guard_conf_account_id {
            if !inst_artifacts.guard_conf.guard_by_spec_account_ids.contains(add_guard_conf_account_id) {
                inst_artifacts.guard_conf.guard_by_spec_account_ids.push(add_guard_conf_account_id.clone());
            }
        }
        if let Some(delete_guard_conf_account_id) = &modify_artifacts.delete_guard_conf_account_id {
            inst_artifacts.guard_conf.guard_by_spec_account_ids =
                inst_artifacts.guard_conf.guard_by_spec_account_ids.into_iter().filter(|account_id| account_id != delete_guard_conf_account_id).collect_vec();
        }
        if let Some((add_approval_account_id, add_approval_result)) = &modify_artifacts.add_approval_result {
            let current_state_result = inst_artifacts.approval_result.entry(inst.current_state_id.clone()).or_default();
            let current_account_ids = current_state_result.entry(add_approval_result.to_string()).or_default();
            current_account_ids.push(add_approval_account_id.clone());
        }
        if let Some(form_state_vars) = modify_artifacts.form_state_map.clone() {
            inst_artifacts.form_state_map.insert(inst.current_state_id.clone(), form_state_vars.clone());
        }
        if let Some(state_id) = &modify_artifacts.clear_form_result {
            inst_artifacts.form_state_map.remove(state_id);
        }
        if let Some(state_id) = &modify_artifacts.clear_approval_result {
            inst_artifacts.approval_result.remove(state_id);
        }
        if let Some(prev_non_auto_state_id) = &modify_artifacts.prev_non_auto_state_id {
            inst_artifacts.prev_non_auto_state_id = Some(prev_non_auto_state_id.clone());
        }
        if let Some(prev_non_auto_account_id) = &modify_artifacts.prev_non_auto_account_id {
            inst_artifacts.prev_non_auto_account_id = Some(prev_non_auto_account_id.clone());
        }
        let flow_inst = flow_inst::ActiveModel {
            id: Set(inst.id.clone()),
            artifacts: Set(Some(inst_artifacts)),
            ..Default::default()
        };
        funs.db().update_one(flow_inst, ctx).await?;

        Ok(())
    }

    fn get_state_conf(
        state_id: &str,
        state_kind: &FlowStateKind,
        kind_conf: Option<FLowStateKindConf>,
        artifacts: Option<FlowInstArtifacts>,
        ctx: &TardisContext,
    ) -> Option<FLowInstStateConf> {
        if let Some(kind_conf) = kind_conf {
            match state_kind {
                FlowStateKind::Form => kind_conf.form.as_ref().map(|form| {
                    let mut operators = HashMap::new();
                    if artifacts.clone().unwrap_or_default().guard_conf.check(ctx) {
                        operators.insert(FlowStateOperatorKind::Submit, form.submit_btn_name.clone());
                        if form.referral {
                            if let Some(referral_guard_custom_conf) = &form.referral_guard_custom_conf {
                                if referral_guard_custom_conf.check(ctx) {
                                    operators.insert(FlowStateOperatorKind::Referral, "".to_string());
                                }
                            } else {
                                operators.insert(FlowStateOperatorKind::Referral, "".to_string());
                            }
                        }
                    }
                    FLowInstStateConf {
                        operators,
                        form_conf: Some(FLowInstStateFormConf {
                            form_vars_collect_conf: form.vars_collect.clone(),
                        }),
                        approval_conf: None,
                    }
                }),
                FlowStateKind::Approval => kind_conf.approval.as_ref().map(|approval| {
                    let mut operators = HashMap::new();
                    let approval_conf = artifacts.clone().unwrap_or_default();
                    if approval_conf.guard_conf.check(ctx) {
                        operators.insert(FlowStateOperatorKind::Pass, approval.pass_btn_name.clone());
                        operators.insert(FlowStateOperatorKind::Overrule, approval.overrule_btn_name.clone());
                        operators.insert(FlowStateOperatorKind::Back, approval.back_btn_name.clone());
                        if let Some(referral_guard_custom_conf) = &approval.referral_guard_custom_conf {
                            if referral_guard_custom_conf.check(ctx) {
                                operators.insert(FlowStateOperatorKind::Referral, "".to_string());
                            }
                        } else {
                            operators.insert(FlowStateOperatorKind::Referral, "".to_string());
                        }
                    }
                    if ctx.own_paths == approval_conf.prev_non_auto_account_id.unwrap_or_default() {
                        operators.insert(FlowStateOperatorKind::Revoke, "".to_string());
                    }
                    let operators = HashMap::from([
                        (FlowStateOperatorKind::Referral, "".to_string()),
                        (FlowStateOperatorKind::Revoke, "".to_string()),
                        (FlowStateOperatorKind::Pass, approval.pass_btn_name.clone()),
                        (FlowStateOperatorKind::Overrule, approval.overrule_btn_name.clone()),
                        (FlowStateOperatorKind::Back, approval.back_btn_name.clone()),
                    ]);
                    FLowInstStateConf {
                        operators,
                        form_conf: None,
                        approval_conf: Some(FLowInstStateApprovalConf {
                            approval_vars_collect_conf: Some(approval.vars_collect.clone()),
                            form_vars_collect: artifacts.unwrap_or_default().form_state_map.get(state_id).cloned().unwrap_or_default(),
                        }),
                    }
                }),
                _ => None,
            }
        } else {
            None
        }
    }

    pub async fn operate(inst: &FlowInstDetailResp, operate_req: &FlowInstOperateReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        match operate_req.operate {
            // 转办
            FlowStateOperatorKind::Referral => {
                Self::modify_inst_artifacts(
                    &inst.id,
                    &FlowInstArtifactsModifyReq {
                        add_guard_conf_account_id: operate_req.operator.clone(),
                        delete_guard_conf_account_id: Some(ctx.owner.clone()),
                        ..Default::default()
                    },
                    funs,
                    ctx,
                )
                .await?;
            }
            // 撤销
            FlowStateOperatorKind::Revoke => {
                let artifacts = inst.artifacts.clone().unwrap_or_default();
                if let Some(target_state_id) = artifacts.prev_non_auto_state_id {
                    Self::transfer_spec_state(inst, &target_state_id, funs, ctx).await?;
                } else {
                    Self::abort(&inst.id, &FlowInstAbortReq { message: "".to_string() }, funs, ctx).await?;
                }
            }
            // 提交
            FlowStateOperatorKind::Submit => {
                Self::modify_inst_artifacts(
                    &inst.id,
                    &FlowInstArtifactsModifyReq {
                        form_state_map: Some(operate_req.vars.clone().unwrap_or_default()),
                        ..Default::default()
                    },
                    funs,
                    ctx,
                )
                .await?;
                if let Some(next_transition) = FlowInstServ::find_next_transitions(inst, &FlowInstFindNextTransitionsReq { vars: None }, funs, ctx).await?.pop() {
                    Self::transfer(
                        inst,
                        &FlowInstTransferReq {
                            flow_transition_id: next_transition.next_flow_transition_id,
                            message: None,
                            vars: None,
                        },
                        false,
                        FlowExternalCallbackOp::Default,
                        loop_check_helper::InstancesTransition::default(),
                        ctx,
                        funs,
                    )
                    .await?;
                }
            }
            // 退回
            FlowStateOperatorKind::Back => {
                let artifacts = inst.artifacts.clone().unwrap_or_default();
                if let Some(target_state_id) = artifacts.prev_non_auto_state_id {
                    Self::transfer_spec_state(inst, &target_state_id, funs, ctx).await?;
                } else {
                    Self::abort(&inst.id, &FlowInstAbortReq { message: "".to_string() }, funs, ctx).await?;
                }
            }
            // 通过
            FlowStateOperatorKind::Pass => {
                Self::modify_inst_artifacts(
                    &inst.id,
                    &FlowInstArtifactsModifyReq {
                        add_approval_result: Some((ctx.owner.clone(), FlowApprovalResultKind::Pass)),
                        ..Default::default()
                    },
                    funs,
                    ctx,
                )
                .await?;
                if let Some(next_transition) = FlowInstServ::find_next_transitions(inst, &FlowInstFindNextTransitionsReq { vars: None }, funs, ctx).await?.pop() {
                    Self::transfer(
                        inst,
                        &FlowInstTransferReq {
                            flow_transition_id: next_transition.next_flow_transition_id,
                            message: None,
                            vars: None,
                        },
                        false,
                        FlowExternalCallbackOp::Default,
                        loop_check_helper::InstancesTransition::default(),
                        ctx,
                        funs,
                    )
                    .await?;
                }
            }
            // 拒绝
            FlowStateOperatorKind::Overrule => {
                Self::modify_inst_artifacts(
                    &inst.id,
                    &FlowInstArtifactsModifyReq {
                        add_approval_result: Some((ctx.owner.clone(), FlowApprovalResultKind::Overrule)),
                        ..Default::default()
                    },
                    funs,
                    ctx,
                )
                .await?;
                Self::abort(&inst.id, &FlowInstAbortReq { message: "".to_string() }, funs, ctx).await?;
            }
        }
        Ok(())
    }

    async fn transfer_spec_state(flow_inst_detail: &FlowInstDetailResp, target_state_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let flow_model_version = FlowModelVersionServ::get_item(
            &flow_inst_detail.rel_flow_version_id,
            &FlowModelVersionFilterReq {
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
            &flow_inst_detail.current_state_id,
            &FlowStateFilterReq {
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
        let next_flow_state = FlowStateServ::get_item(
            target_state_id,
            &FlowStateFilterReq {
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
        if FlowRelServ::find_to_simple_rels(&FlowRelKind::FlowModelState, target_state_id, None, None, funs, ctx)
            .await?
            .into_iter()
            .filter(|rel| rel.rel_id == flow_inst_detail.rel_flow_version_id)
            .collect_vec()
            .is_empty()
        {
            return Err(funs.err().internal_error("flow_inst_serv", "transfer_spec_state", "flow state is not found", "404-flow-state-not-found"));
        }

        let mut new_transitions = Vec::new();
        if let Some(transitions) = &flow_inst_detail.transitions {
            new_transitions.extend(transitions.clone());
        }
        new_transitions.push(FlowInstTransitionInfo {
            id: "".to_string(),
            start_time: Utc::now(),
            op_ctx: FlowOperationContext::from_ctx(ctx),
            output_message: None,
            from_state_id: Some(prev_flow_state.id.clone()),
            from_state_name: Some(prev_flow_state.name.clone()),
            target_state_id: Some(next_flow_state.id.clone()),
            target_state_name: Some(next_flow_state.name.clone()),
        });

        let flow_inst = flow_inst::ActiveModel {
            id: Set(flow_inst_detail.id.clone()),
            current_state_id: Set(target_state_id.to_string()),
            transitions: Set(Some(new_transitions.clone())),
            ..Default::default()
        };

        funs.db().update_one(flow_inst, ctx).await?;
        // 删除目标节点的旧记录
        Self::modify_inst_artifacts(
            &flow_inst_detail.id,
            &FlowInstArtifactsModifyReq {
                clear_approval_result: Some(target_state_id.to_string()),
                clear_form_result: Some(target_state_id.to_string()),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;

        Self::when_leave_state(flow_inst_detail, &flow_inst_detail.current_state_id, &flow_model_version.rel_model_id, funs, ctx).await?;
        Self::when_enter_state(flow_inst_detail, target_state_id, &flow_model_version.rel_model_id, funs, ctx).await?;

        Ok(())
    }

    pub async fn add_comment(flow_inst_detail: &FlowInstDetailResp, add_comment: &FlowInstCommentReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let mut comments = flow_inst_detail.comments.clone().unwrap_or_default();
        comments.push(FlowInstCommentInfo {
            output_message: add_comment.output_message.clone(),
            owner: ctx.owner.clone(),
            parent_comment_id: add_comment.parent_comment_id.clone(),
            parent_owner: add_comment.parent_owner.clone(),
            create_time: Utc::now(),
        });
        let flow_inst = flow_inst::ActiveModel {
            id: Set(flow_inst_detail.id.clone()),

            comments: Set(Some(comments.clone())),
            ..Default::default()
        };
        funs.db().update_one(flow_inst, ctx).await?;
        Ok(())
    }
    // search
    pub async fn search(search_req: &mut FlowInstSearchReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<TardisPage<FlowInstSummaryResp>> {
        let mut where_fragments: Vec<String> = vec!["1=1".to_string()];
        let mut sql_vals: Vec<sea_orm::Value> = vec![];
        let table_alias_name = "flow_inst";

        Self::package_query(table_alias_name, search_req.query.clone(), &mut sql_vals, &mut where_fragments, funs, ctx)?;
        Self::package_query_kind(
            table_alias_name,
            search_req.query_kind.clone().unwrap_or(FlowInstQueryKind::All),
            &mut sql_vals,
            &mut where_fragments,
            funs,
            ctx,
        )?;
        let order_fragments = Self::package_order(table_alias_name, search_req.sort.clone())?;
        let page_fragments = Self::package_page(search_req.page.clone(), &mut sql_vals)?;
        let result = funs
            .db()
            .query_all(
                format!(
                    r#"SELECT
                flow_inst.id,
                flow_inst.rel_flow_version_id,
                flow_inst.rel_business_obj_id,
                flow_inst.current_state_id,
                flow_inst.create_ctx,
                flow_inst.create_time,
                flow_inst.finish_ctx,
                flow_inst.finish_time,
                flow_inst.finish_abort,
                flow_inst.output_message,
                flow_inst.own_paths,
                flow_inst.tag,
                model_version.name as rel_flow_model_name,
                flow_model_version.rel_model_id as rel_flow_model_id
                {}
            FROM
            flow_inst
            LEFT JOIN flow_state AS current_state ON flow_inst.current_state_id = current_state.id
            LEFT JOIN rbum_item AS model_version ON flow_inst.rel_flow_version_id = model_version.id
            LEFT JOIN flow_model_version ON flow_inst.rel_flow_version_id = flow_model_version.id
            WHERE  
            {}
            {}
            {};"#,
                    if search_req.page.fetch_total { ", count(*) OVER() AS total" } else { "" },
                    where_fragments.join(" AND "),
                    if order_fragments.is_empty() {
                        "".to_string()
                    } else {
                        format!("ORDER BY {}", order_fragments.join(", "))
                    },
                    page_fragments
                )
                .as_str(),
                sql_vals,
            )
            .await?;

        let mut total_size: i64 = 0;
        let result = result
            .into_iter()
            .map(|item| {
                if search_req.page.fetch_total && total_size == 0 {
                    total_size = item.try_get("", "total")?;
                }
                Ok(FlowInstSummaryResp {
                    id: item.try_get("", "id")?,
                    rel_flow_version_id: item.try_get("", "rel_flow_version_id")?,
                    rel_flow_model_id: item.try_get("", "rel_flow_model_id")?,
                    rel_flow_model_name: item.try_get("", "rel_flow_model_name")?,
                    rel_business_obj_id: item.try_get("", "rel_business_obj_id")?,
                    current_state_id: item.try_get("", "current_state_id")?,
                    create_ctx: item.try_get("", "create_ctx")?,
                    create_time: item.try_get("", "create_time")?,
                    finish_ctx: item.try_get("", "finish_ctx")?,
                    finish_time: item.try_get("", "finish_time")?,
                    finish_abort: item.try_get("", "finish_abort").unwrap_or_default(),
                    output_message: item.try_get("", "output_message")?,
                    own_paths: item.try_get("", "own_paths")?,
                    tag: item.try_get("", "tag")?,
                })
            })
            .collect::<TardisResult<Vec<FlowInstSummaryResp>>>()?;

        Ok(TardisPage {
            page_size: search_req.page.size as u64,
            page_number: search_req.page.number as u64,
            total_size: total_size as u64,
            records: result,
        })
    }

    fn package_query_kind(
        table_alias_name: &str,
        query_kind: FlowInstQueryKind,
        sql_vals: &mut Vec<sea_orm::Value>,
        where_fragments: &mut Vec<String>,
        _funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        match query_kind {
            FlowInstQueryKind::Create => {
                sql_vals.push(sea_orm::Value::from(ctx.owner.clone()));
                where_fragments.push(format!("{}.create_ctx ->> 'owner' = ${}", table_alias_name, sql_vals.len()));
            }
            FlowInstQueryKind::Form => {
                let mut child_or_where_fragments = vec![];
                sql_vals.push(sea_orm::Value::from(ctx.owner.clone()));
                child_or_where_fragments.push(format!(
                    "EXISTS (SELECT 1 FROM jsonb_array_elements_text({}.artifacts->'guard_conf'->'guard_by_spec_account_ids') AS elem WHERE elem IN (${}))",
                    table_alias_name,
                    sql_vals.len()
                ));
                if !ctx.roles.is_empty() {
                    sql_vals.push(sea_orm::Value::from(ctx.roles.clone().join(", ").to_string()));
                    child_or_where_fragments.push(format!(
                        "EXISTS (SELECT 1 FROM jsonb_array_elements_text({}.artifacts->'guard_conf'->'guard_by_spec_role_ids') AS elem WHERE elem IN (${}))",
                        table_alias_name,
                        sql_vals.len()
                    ));
                }
                if !ctx.groups.is_empty() {
                    sql_vals.push(sea_orm::Value::from(ctx.groups.clone().join(", ").to_string()));
                    child_or_where_fragments.push(format!(
                        "EXISTS (SELECT 1 FROM jsonb_array_elements_text({}.artifacts->'guard_conf'->'guard_by_spec_org_ids') AS elem WHERE elem IN (${}))",
                        table_alias_name,
                        sql_vals.len()
                    ));
                }
                where_fragments.push(format!("current_state.state_kind = 'form' AND ({})", child_or_where_fragments.join(" OR ")));
            }
            FlowInstQueryKind::Approval => {
                let mut child_or_where_fragments = vec![];
                sql_vals.push(sea_orm::Value::from(ctx.owner.clone()));
                child_or_where_fragments.push(format!(
                    "EXISTS (SELECT 1 FROM jsonb_array_elements_text({}.artifacts->'guard_conf'->'guard_by_spec_account_ids') AS elem WHERE elem IN (${}))",
                    table_alias_name,
                    sql_vals.len()
                ));
                if !ctx.roles.is_empty() {
                    sql_vals.push(sea_orm::Value::from(ctx.roles.clone().join(", ").to_string()));
                    child_or_where_fragments.push(format!(
                        "EXISTS (SELECT 1 FROM jsonb_array_elements_text({}.artifacts->'guard_conf'->'guard_by_spec_role_ids') AS elem WHERE elem IN (${}))",
                        table_alias_name,
                        sql_vals.len()
                    ));
                }
                if !ctx.groups.is_empty() {
                    sql_vals.push(sea_orm::Value::from(ctx.groups.clone().join(", ").to_string()));
                    child_or_where_fragments.push(format!(
                        "EXISTS (SELECT 1 FROM jsonb_array_elements_text({}.artifacts->'guard_conf'->'guard_by_spec_org_ids') AS elem WHERE elem IN (${}))",
                        table_alias_name,
                        sql_vals.len()
                    ));
                }
                where_fragments.push(format!("current_state.state_kind = 'approval' AND ({})", child_or_where_fragments.join(" OR ")));
            }
            FlowInstQueryKind::All => {
                let mut or_where_fragments = vec![];
                sql_vals.push(sea_orm::Value::from(ctx.owner.clone()));
                or_where_fragments.push(format!("({}.create_ctx ->> 'owner' = ${})", table_alias_name, sql_vals.len()));

                let mut form_or_where_fragments = vec![];
                sql_vals.push(sea_orm::Value::from(ctx.owner.clone()));
                form_or_where_fragments.push(format!(
                    "EXISTS (SELECT 1 FROM jsonb_array_elements_text({}.artifacts->'guard_conf'->'guard_by_spec_account_ids') AS elem WHERE elem IN (${}))",
                    table_alias_name,
                    sql_vals.len()
                ));
                if !ctx.roles.is_empty() {
                    sql_vals.push(sea_orm::Value::from(ctx.roles.clone().join(", ").to_string()));
                    form_or_where_fragments.push(format!(
                        "EXISTS (SELECT 1 FROM jsonb_array_elements_text({}.artifacts->'guard_conf'->'guard_by_spec_role_ids') AS elem WHERE elem IN (${}))",
                        table_alias_name,
                        sql_vals.len()
                    ));
                }
                if !ctx.groups.is_empty() {
                    sql_vals.push(sea_orm::Value::from(ctx.groups.clone().join(", ").to_string()));
                    form_or_where_fragments.push(format!(
                        "EXISTS (SELECT 1 FROM jsonb_array_elements_text({}.artifacts->'guard_conf'->'guard_by_spec_org_ids') AS elem WHERE elem IN (${}))",
                        table_alias_name,
                        sql_vals.len()
                    ));
                }
                or_where_fragments.push(format!("(current_state.state_kind = 'form' AND ({}))", form_or_where_fragments.join(" OR ")));

                let mut approval_or_where_fragments = vec![];
                sql_vals.push(sea_orm::Value::from(ctx.owner.clone()));
                approval_or_where_fragments.push(format!(
                    "EXISTS (SELECT 1 FROM jsonb_array_elements_text({}.artifacts->'guard_conf'->'guard_by_spec_account_ids') AS elem WHERE elem IN (${}))",
                    table_alias_name,
                    sql_vals.len()
                ));
                if !ctx.roles.is_empty() {
                    sql_vals.push(sea_orm::Value::from(ctx.roles.clone().join(", ").to_string()));
                    approval_or_where_fragments.push(format!(
                        "EXISTS (SELECT 1 FROM jsonb_array_elements_text({}.artifacts->'guard_conf'->'guard_by_spec_role_ids') AS elem WHERE elem IN (${}))",
                        table_alias_name,
                        sql_vals.len()
                    ));
                }
                if !ctx.groups.is_empty() {
                    sql_vals.push(sea_orm::Value::from(ctx.groups.clone().join(", ").to_string()));
                    approval_or_where_fragments.push(format!(
                        "EXISTS (SELECT 1 FROM jsonb_array_elements_text({}.artifacts->'guard_conf'->'guard_by_spec_org_ids') AS elem WHERE elem IN (${}))",
                        table_alias_name,
                        sql_vals.len()
                    ));
                }
                or_where_fragments.push(format!("(current_state.state_kind = 'approval' AND ({}))", approval_or_where_fragments.join(" OR ")));
                where_fragments.push(format!("( {} )", or_where_fragments.join(" OR ")));
            }
        }
        Ok(())
    }

    fn package_query(
        table_alias_name: &str,
        query: FlowInstFilterReq,
        sql_vals: &mut Vec<sea_orm::Value>,
        where_fragments: &mut Vec<String>,
        _funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        if let Some(ids) = query.ids {
            if !ids.is_empty() {
                where_fragments.push(format!(
                    "{}.id = ANY (ARRAY[{}])",
                    table_alias_name,
                    (0..ids.len()).map(|idx| format!("${}", sql_vals.len() + idx + 1)).collect::<Vec<String>>().join(",")
                ));
                for id in ids {
                    sql_vals.push(sea_orm::Value::from(id.to_string()));
                }
            }
        }
        if let Some(flow_version_id) = query.flow_version_id {
            sql_vals.push(sea_orm::Value::from(flow_version_id));
            where_fragments.push(format!("{}.rel_flow_version_id = ${}", table_alias_name, sql_vals.len()));
        }
        if let Some(rel_business_obj_ids) = query.rel_business_obj_ids {
            if !rel_business_obj_ids.is_empty() {
                where_fragments.push(format!(
                    "{}.rel_business_obj_id = ANY (ARRAY[{}])",
                    table_alias_name,
                    (0..rel_business_obj_ids.len()).map(|idx| format!("${}", sql_vals.len() + idx + 1)).collect::<Vec<String>>().join(",")
                ));
                for rel_business_obj_id in rel_business_obj_ids {
                    sql_vals.push(sea_orm::Value::from(rel_business_obj_id.to_string()));
                }
            }
        }
        if let Some(tag) = query.tag {
            sql_vals.push(sea_orm::Value::from(tag));
            where_fragments.push(format!("{}.tag = ${}", table_alias_name, sql_vals.len()));
        }
        if let Some(main) = query.main {
            sql_vals.push(sea_orm::Value::from(main));
            where_fragments.push(format!("{}.main = ${}", table_alias_name, sql_vals.len()));
        }
        if let Some(finish) = query.finish {
            if finish {
                where_fragments.push(format!("{}.finish_time is not null", table_alias_name));
            } else {
                where_fragments.push(format!("{}.finish_time is null", table_alias_name));
            }
        }
        if let Some(current_state_id) = query.current_state_id {
            sql_vals.push(sea_orm::Value::from(current_state_id));
            where_fragments.push(format!("{}.current_state_id = ${}", table_alias_name, sql_vals.len()));
        }
        if query.with_sub.unwrap_or(false) {
            sql_vals.push(sea_orm::Value::from(format!("{}%", ctx.own_paths)));
            where_fragments.push(format!("{}.own_paths like ${}", table_alias_name, sql_vals.len()));
        } else {
            sql_vals.push(sea_orm::Value::from(ctx.own_paths.clone()));
            where_fragments.push(format!("{}.own_paths = ${}", table_alias_name, sql_vals.len()));
        }
        Ok(())
    }

    fn package_page(page: FlowInstSearchPageReq, sql_vals: &mut Vec<sea_orm::Value>) -> TardisResult<String> {
        sql_vals.push(sea_orm::Value::from(page.size));
        sql_vals.push(sea_orm::Value::from((page.number - 1) * page.size as u32));
        Ok(format!("LIMIT ${} OFFSET ${}", sql_vals.len() - 1, sql_vals.len()))
    }

    fn package_order(table_alias_name: &str, sort: Option<Vec<FlowInstSearchSortReq>>) -> TardisResult<Vec<String>> {
        let mut order_fragments: Vec<String> = Vec::new();
        if let Some(sort) = &sort {
            for sort_item in sort {
                if let Some(in_field) = &sort_item.in_field {
                    order_fragments.push(format!("{}.{} -> '{}' {}", table_alias_name, in_field, sort_item.field, sort_item.order.to_sql()));
                } else {
                    order_fragments.push(format!("{}.{} {}", table_alias_name, sort_item.field, sort_item.order.to_sql()));
                }
            }
        }
        Ok(order_fragments)
    }
}
