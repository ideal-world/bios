use std::{collections::HashMap, str::FromStr as _};

use async_recursion::async_recursion;
use bios_basic::rbum::{
    dto::rbum_filer_dto::RbumBasicFilterReq,
    serv::{
        rbum_crud_serv::{CREATE_TIME_FIELD, ID_FIELD, NAME_FIELD, REL_DOMAIN_ID_FIELD, REL_KIND_ID_FIELD, UPDATE_TIME_FIELD},
        rbum_item_serv::{RbumItemCrudOperation, RBUM_ITEM_TABLE},
    },
};
use bios_sdk_invoke::dto::search_item_dto::{
    SearchItemQueryReq, SearchItemSearchCtxReq, SearchItemSearchPageReq, SearchItemSearchReq, SearchItemSearchSortKind, SearchItemSearchSortReq,
};
use itertools::Itertools;
use serde_json::json;
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    chrono::{DateTime, Datelike, Utc},
    db::sea_orm::{
        self,
        sea_query::{Alias, Cond, Expr, Query, SelectStatement},
        JoinType, Order, Set,
    },
    futures_util::future::join_all,
    log::error,
    serde_json::Value,
    web::web_resp::TardisPage,
    TardisFuns, TardisFunsInst,
};

use crate::{
    domain::{flow_inst, flow_model_version},
    dto::{
        flow_cond_dto::BasicQueryCondInfo,
        flow_external_dto::{FlowExternalCallbackOp, FlowExternalParams},
        flow_inst_dto::{
            FLowInstStateApprovalConf, FLowInstStateConf, FLowInstStateFormConf, FlowApprovalResultKind, FlowInstAbortReq, FlowInstArtifacts, FlowInstArtifactsModifyReq,
            FlowInstBatchBindReq, FlowInstBatchBindResp, FlowInstCommentInfo, FlowInstCommentReq, FlowInstDetailResp, FlowInstFilterReq, FlowInstFindNextTransitionResp,
            FlowInstFindNextTransitionsReq, FlowInstFindStateAndTransitionsReq, FlowInstFindStateAndTransitionsResp, FlowInstOperateReq, FlowInstStartReq, FlowInstStateKind,
            FlowInstSummaryResp, FlowInstSummaryResult, FlowInstTransferReq, FlowInstTransferResp, FlowInstTransitionInfo, FlowOperationContext,
        },
        flow_model_dto::{FlowModelAggResp, FlowModelRelTransitionExt},
        flow_model_version_dto::FlowModelVersionFilterReq,
        flow_state_dto::{
            FLowStateKindConf, FlowStateCountersignKind, FlowStateFilterReq, FlowStateKind, FlowStateOperatorKind, FlowStateRelModelExt, FlowStatusAutoStrategyKind,
            FlowStatusMultiApprovalKind, FlowSysStateKind,
        },
        flow_transition_dto::FlowTransitionDetailResp,
        flow_var_dto::FillType,
    },
    flow_constants,
    helper::loop_check_helper,
    serv::{flow_model_serv::FlowModelServ, flow_state_serv::FlowStateServ},
};

use super::{
    clients::{log_client::LogParamOp, search_client::FlowSearchClient},
    flow_event_serv::FlowEventServ,
    flow_external_serv::FlowExternalServ,
    flow_log_serv::FlowLogServ,
    flow_model_version_serv::FlowModelVersionServ,
    flow_rel_serv::{FlowRelKind, FlowRelServ},
    flow_transition_serv::FlowTransitionServ,
};

pub struct FlowInstServ;

impl FlowInstServ {
    pub async fn start(start_req: &FlowInstStartReq, current_state_name: Option<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        if start_req.transition_id.is_none() {
            Self::start_main_flow(start_req, current_state_name, funs, ctx).await
        } else {
            Self::start_secondary_flow(start_req, funs, ctx).await
        }
    }
    async fn start_main_flow(start_req: &FlowInstStartReq, current_state_name: Option<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        if !Self::find_ids(
            &FlowInstFilterReq {
                rel_business_obj_ids: Some(vec![start_req.rel_business_obj_id.clone()]),
                main: Some(true),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
        .is_empty()
        {
            return Err(funs.err().internal_error("flow_inst_serv", "start", "The same instance exist", "500-flow-inst-exist"));
        }
        // get model by own_paths
        let flow_model = FlowModelServ::get_model_id_by_own_paths_and_rel_template_id(&start_req.tag, None, funs, ctx).await?;
        let inst_id = TardisFuns::field.nanoid();
        let current_state_id = if let Some(current_state_name) = &current_state_name {
            if current_state_name.is_empty() {
                flow_model.init_state_id.clone()
            } else {
                FlowStateServ::match_state_id_by_name(&flow_model.current_version_id, current_state_name, funs, ctx).await?
            }
        } else {
            flow_model.init_state_id.clone()
        };
        let flow_inst: flow_inst::ActiveModel = flow_inst::ActiveModel {
            id: Set(inst_id.clone()),
            code: Set(Some("".to_string())),
            tag: Set(Some(flow_model.tag.clone())),
            rel_flow_version_id: Set(flow_model.current_version_id.clone()),
            rel_business_obj_id: Set(start_req.rel_business_obj_id.clone()),

            current_state_id: Set(current_state_id.clone()),

            create_vars: Set(None),
            current_vars: Set(None),
            create_ctx: Set(FlowOperationContext::from_ctx(ctx)),

            own_paths: Set(ctx.own_paths.clone()),
            main: Set(true),
            update_time: Set(Some(Utc::now())),
            ..Default::default()
        };
        funs.db().insert_one(flow_inst, ctx).await?;

        let create_vars = Self::get_new_vars(&flow_model.tag, start_req.rel_business_obj_id.to_string(), funs, ctx).await?;
        let flow_inst = flow_inst::ActiveModel {
            id: Set(inst_id.clone()),
            create_vars: Set(Some(TardisFuns::json.obj_to_json(&create_vars).unwrap_or(json!("")))),
            current_vars: Set(Some(TardisFuns::json.obj_to_json(&create_vars).unwrap_or(json!("")))),
            ..Default::default()
        };
        funs.db().update_one(flow_inst, ctx).await?;

        Self::do_request_webhook(
            None,
            flow_model.transitions().iter().filter(|model_transition| model_transition.to_flow_state_id == flow_model.init_state_id).collect_vec().pop(),
        )
        .await?;

        Ok(inst_id)
    }

    async fn start_secondary_flow(start_req: &FlowInstStartReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        // get model by own_paths
        let flow_model = FlowModelServ::get_model_id_by_own_paths_and_transition_id(&start_req.tag, &start_req.transition_id.clone().unwrap_or_default(), funs, ctx).await?;
        let inst_id = TardisFuns::field.nanoid();
        if !Self::find_ids(
            &FlowInstFilterReq {
                rel_business_obj_ids: Some(vec![start_req.rel_business_obj_id.to_string()]),
                flow_model_id: Some(flow_model.id.clone()),
                finish: Some(false),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
        .is_empty()
        {
            return Err(funs.err().internal_error("flow_inst_serv", "start", "The same instance exist", "500-flow-inst-exist"));
        }
        let create_vars = Self::get_new_vars(&flow_model.tag, start_req.rel_business_obj_id.to_string(), funs, ctx).await?;
        let flow_inst: flow_inst::ActiveModel = flow_inst::ActiveModel {
            id: Set(inst_id.clone()),
            code: Set(Some(Self::gen_inst_code(funs).await?)),
            tag: Set(Some(flow_model.tag.clone())),
            rel_flow_version_id: Set(flow_model.current_version_id.clone()),
            rel_business_obj_id: Set(start_req.rel_business_obj_id.clone()),

            current_state_id: Set(flow_model.init_state_id.clone()),

            create_vars: Set(Some(TardisFuns::json.obj_to_json(&create_vars).unwrap_or(json!("")))),
            current_vars: Set(Some(TardisFuns::json.obj_to_json(&create_vars).unwrap_or(json!("")))),

            create_ctx: Set(FlowOperationContext::from_ctx(ctx)),

            own_paths: Set(ctx.own_paths.clone()),
            main: Set(false),
            ..Default::default()
        };
        funs.db().insert_one(flow_inst, ctx).await?;
        Self::modify_inst_artifacts(
            &inst_id,
            &FlowInstArtifactsModifyReq {
                curr_vars: start_req.create_vars.clone(),
                add_his_operator: Some(ctx.owner.clone()),
                form_state_map: Some(start_req.vars.clone().unwrap_or_default()),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        let rel_transition_names = Self::get_rel_transitions(&start_req.rel_business_obj_id, funs, ctx).await?.into_iter().map(|rel| rel.to_string()).collect_vec();
        FlowSearchClient::modify_business_obj_search(
            &start_req.rel_business_obj_id,
            &flow_model.tag,
            json!({
                "rel_transitions": rel_transition_names,
            }),
            funs,
            ctx,
        )
        .await?;
        let inst = Self::get(&inst_id, funs, ctx).await?;
        FlowLogServ::add_start_log(start_req, &inst, &create_vars, &flow_model, ctx).await?;
        FlowLogServ::add_start_dynamic_log(start_req, &inst, &create_vars, &flow_model, ctx).await?;
        FlowLogServ::add_start_business_log(start_req, &inst, &create_vars, &flow_model, ctx).await?;

        Self::when_enter_state(&inst, &flow_model.init_state_id, &flow_model.id, funs, ctx).await?;
        Self::do_request_webhook(
            None,
            flow_model.transitions().iter().filter(|model_transition| model_transition.to_flow_state_id == flow_model.init_state_id).collect_vec().pop(),
        )
        .await?;
        // 自动流转
        Self::auto_transfer(&inst.id, loop_check_helper::InstancesTransition::default(), funs, ctx).await?;

        FlowSearchClient::async_add_or_modify_instance_search(&inst_id, Box::new(false), funs, ctx).await?;
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
                return Err(funs.err().not_found("flow_inst_serv", "batch_bind", "req is valid", ""));
            }
            current_ctx.own_paths = rel_business_obj.own_paths.clone().unwrap_or_default();
            current_ctx.owner = rel_business_obj.owner.clone().unwrap_or_default();
            let flow_model = if let Some(transition_id) = &batch_bind_req.transition_id {
                FlowModelServ::get_model_id_by_own_paths_and_transition_id(&batch_bind_req.tag, transition_id, funs, ctx).await?
            } else {
                FlowModelServ::get_model_id_by_own_paths_and_rel_template_id(&batch_bind_req.tag, None, funs, ctx).await?
            };
            let current_state_id =
                FlowStateServ::match_state_id_by_name(&flow_model.current_version_id, &rel_business_obj.current_state_name.clone().unwrap_or_default(), funs, ctx).await?;
            let inst_id = if let Some(inst_id) =
                Self::get_inst_ids_by_rel_business_obj_id(vec![rel_business_obj.rel_business_obj_id.clone().unwrap_or_default()], Some(true), funs, ctx).await?.pop()
            {
                inst_id
            } else {
                let id = TardisFuns::field.nanoid();
                let flow_inst: flow_inst::ActiveModel = flow_inst::ActiveModel {
                    id: Set(id.clone()),
                    rel_flow_version_id: Set(flow_model.current_version_id.to_string()),
                    rel_business_obj_id: Set(rel_business_obj.rel_business_obj_id.clone().unwrap_or_default()),

                    current_state_id: Set(current_state_id),

                    create_vars: Set(None),
                    current_vars: Set(None),

                    create_ctx: Set(FlowOperationContext::from_ctx(&current_ctx)),

                    own_paths: Set(rel_business_obj.own_paths.clone().unwrap_or_default()),
                    ..Default::default()
                };
                funs.db().insert_one(flow_inst, &current_ctx).await?;

                let create_vars = Self::get_new_vars(&flow_model.tag, rel_business_obj.rel_business_obj_id.clone().unwrap_or_default(), funs, ctx).await?;
                let flow_inst = flow_inst::ActiveModel {
                    id: Set(id.clone()),
                    create_vars: Set(Some(TardisFuns::json.obj_to_json(&create_vars).unwrap_or(json!("")))),
                    current_vars: Set(Some(TardisFuns::json.obj_to_json(&create_vars).unwrap_or(json!("")))),
                    ..Default::default()
                };
                funs.db().update_one(flow_inst, ctx).await?;

                id
            };
            let current_state_name = Self::get(&inst_id, funs, &current_ctx).await?.current_state_name.unwrap_or_default();
            result.push(FlowInstBatchBindResp {
                rel_business_obj_id: rel_business_obj.rel_business_obj_id.clone().unwrap_or_default(),
                current_state_name,
                inst_id: Some(inst_id),
            });
        }

        Ok(result)
    }

    async fn package_ext_query(query: &mut SelectStatement, filter: &FlowInstFilterReq, _: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let flow_model_version_table = Alias::new("flow_model_version");
        let rel_model_table = Alias::new("rbum_rel");
        let flow_state_item = Alias::new("flow_state_item");
        query
            .columns([
                (flow_inst::Entity, flow_inst::Column::Id),
                (flow_inst::Entity, flow_inst::Column::Code),
                (flow_inst::Entity, flow_inst::Column::RelFlowVersionId),
                (flow_inst::Entity, flow_inst::Column::RelBusinessObjId),
                (flow_inst::Entity, flow_inst::Column::CreateVars),
                (flow_inst::Entity, flow_inst::Column::CurrentStateId),
                (flow_inst::Entity, flow_inst::Column::CreateCtx),
                (flow_inst::Entity, flow_inst::Column::CreateTime),
                (flow_inst::Entity, flow_inst::Column::UpdateTime),
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
            .expr_as(Expr::col((flow_state_item.clone(), NAME_FIELD.clone())).if_null(""), Alias::new("current_state_name"))
            .expr_as(Expr::col((RBUM_ITEM_TABLE.clone(), NAME_FIELD.clone())).if_null(""), Alias::new("rel_flow_model_name"))
            .expr_as(Expr::col((rel_model_table.clone(), Alias::new("ext"))).if_null(""), Alias::new("rel_transition"))
            .from(flow_inst::Entity)
            .join_as(
                JoinType::LeftJoin,
                RBUM_ITEM_TABLE.clone(),
                flow_state_item.clone(),
                Expr::col((flow_state_item.clone(), ID_FIELD.clone())).equals((flow_inst::Entity, flow_inst::Column::CurrentStateId)),
            )
            .left_join(
                flow_model_version_table.clone(),
                Expr::col((flow_model_version_table.clone(), ID_FIELD.clone())).equals((flow_inst::Entity, flow_inst::Column::RelFlowVersionId)),
            )
            .left_join(
                RBUM_ITEM_TABLE.clone(),
                Expr::col((RBUM_ITEM_TABLE.clone(), ID_FIELD.clone())).equals((flow_inst::Entity, flow_inst::Column::RelFlowVersionId)),
            )
            .left_join(
                rel_model_table.clone(),
                Cond::all()
                    .add(Expr::col((rel_model_table.clone(), Alias::new("from_rbum_id"))).equals((flow_model_version_table.clone(), flow_model_version::Column::RelModelId)))
                    .add(Expr::col((rel_model_table.clone(), Alias::new("tag"))).eq("FlowModelTransition".to_string())),
            );
        if let Some(ids) = &filter.ids {
            query.and_where(Expr::col((flow_inst::Entity, flow_inst::Column::Id)).is_in(ids));
        }
        if let Some(code) = &filter.code {
            query.and_where(Expr::col((flow_inst::Entity, flow_inst::Column::Code)).like(format!("{}%", code)));
        }
        if filter.with_sub.unwrap_or(false) {
            query.and_where(Expr::col((flow_inst::Entity, flow_inst::Column::OwnPaths)).like(format!("{}%", ctx.own_paths)));
        } else {
            query.and_where(Expr::col((flow_inst::Entity, flow_inst::Column::OwnPaths)).eq(ctx.own_paths.as_str()));
        }
        if let Some(flow_version_id) = &filter.flow_version_id {
            query.and_where(Expr::col((flow_inst::Entity, flow_inst::Column::RelFlowVersionId)).eq(flow_version_id));
        }
        if let Some(flow_model_id) = &filter.flow_model_id {
            query.and_where(Expr::col((flow_model_version::Entity, flow_model_version::Column::RelModelId)).eq(flow_model_id));
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

    pub async fn find_ids(filter: &FlowInstFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<String>> {
        let mut query = Query::select();
        Self::package_ext_query(&mut query, filter, funs, ctx).await?;
        Ok(funs.db().find_dtos::<FlowInstSummaryResult>(&query).await?.into_iter().map(|inst| inst.id).collect_vec())
    }

    pub async fn find_detail_items(filter: &FlowInstFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<FlowInstDetailResp>> {
        Self::find_detail(Self::find_ids(filter, funs, ctx).await?, None, None, funs, ctx).await
    }

    pub async fn paginate_detail_items(
        filter: &FlowInstFilterReq,
        page_number: u32,
        page_size: u32,
        desc_by_create: Option<bool>,
        desc_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<FlowInstDetailResp>> {
        let inst_ids = Self::find_ids(filter, funs, ctx).await?;
        let total_size = inst_ids.len() as usize;
        let records = Self::find_detail(
            inst_ids[(((page_number - 1) * page_size) as usize).min(total_size)..((page_number * page_size) as usize).min(total_size)].to_vec(),
            desc_by_create,
            desc_by_update,
            funs,
            ctx,
        )
        .await?;
        Ok(TardisPage {
            page_size: page_size as u64,
            page_number: page_number as u64,
            total_size: total_size as u64,
            records,
        })
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
            update_time: Set(Some(Utc::now())),
            finish_abort: Set(Some(true)),
            output_message: Set(Some(abort_req.message.to_string())),
            ..Default::default()
        };
        funs.db().update_one(flow_inst, ctx).await?;

        let flow_inst_detail = Self::get(flow_inst_id, funs, ctx).await?;
        if !flow_inst_detail.main {
            FlowLogServ::add_finish_log(&flow_inst_detail, funs, ctx).await?;
            let rel_transition_names = Self::get_rel_transitions(&flow_inst_detail.rel_business_obj_id, funs, ctx).await?.into_iter().map(|rel| rel.to_string()).collect_vec();
            FlowSearchClient::modify_business_obj_search(
                &flow_inst_detail.rel_business_obj_id,
                &flow_inst_detail.tag,
                json!({
                    "rel_transitions": rel_transition_names,
                }),
                funs,
                ctx,
            )
            .await?;
            FlowSearchClient::async_add_or_modify_instance_search(&flow_inst_detail.id, Box::new(true), funs, ctx).await?;
        }
        Ok(())
    }

    pub async fn get(flow_inst_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<FlowInstDetailResp> {
        let flow_insts = Self::find_detail(vec![flow_inst_id.to_string()], None, None, funs, ctx).await?;
        if let Some(flow_inst) = flow_insts.into_iter().next() {
            Ok(flow_inst)
        } else {
            Err(funs.err().not_found("flow_inst", "get", &format!("flow instance {} not found", flow_inst_id), "404-flow-inst-not-found"))
        }
    }

    pub async fn batch_check_auth(flow_inst_ids: Vec<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<String>> {
        Ok(Self::find_detail(flow_inst_ids, None, None, funs, ctx)
            .await?
            .into_iter()
            .filter(|flow_inst| Self::check_auth(flow_inst, funs, ctx))
            .map(|flow_inst| flow_inst.id)
            .collect_vec())
    }
    fn check_auth(flow_inst: &FlowInstDetailResp, _funs: &TardisFunsInst, ctx: &TardisContext) -> bool {
        flow_inst.create_ctx.owner == ctx.owner // 当前用户不是创建人
            || flow_inst.artifacts.as_ref().map_or_else(|| false, |artifacts| artifacts.his_operators.clone().unwrap_or_default().contains(&ctx.owner)) // 当前用户不是历史操作人
            || flow_inst.current_state_conf.as_ref().map_or_else(|| false, |conf| !conf.operators.is_empty())
        // 当前用户没有任何操作权限
    }

    pub async fn find_detail(
        flow_inst_ids: Vec<String>,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<FlowInstDetailResp>> {
        #[derive(sea_orm::FromQueryResult)]
        pub struct FlowInstDetailResult {
            pub id: String,
            pub code: String,
            pub tag: String,
            pub rel_flow_version_id: String,
            pub rel_flow_model_id: String,
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
            pub update_time: Option<DateTime<Utc>>,

            pub finish_ctx: Option<FlowOperationContext>,
            pub finish_time: Option<DateTime<Utc>>,
            pub finish_abort: Option<bool>,
            pub output_message: Option<String>,

            pub transitions: Option<Value>,
            pub artifacts: Option<Value>,
            pub comments: Option<Value>,

            pub rel_transition: Option<String>,

            pub own_paths: String,

            pub rel_business_obj_id: String,
        }
        let rel_state_table = Alias::new("rel_state");
        let flow_state_table = Alias::new("flow_state");
        let flow_model_version_table = Alias::new("flow_model_version");
        let rel_model_version_table = Alias::new("rel_model_version");
        let rel_state_ext_table = Alias::new("rel_state_ext");
        let rel_model_table = Alias::new("rel_model");
        let mut query = Query::select();
        query
            .columns([
                (flow_inst::Entity, flow_inst::Column::Id),
                (flow_inst::Entity, flow_inst::Column::Code),
                (flow_inst::Entity, flow_inst::Column::Tag),
                (flow_inst::Entity, flow_inst::Column::RelFlowVersionId),
                (flow_inst::Entity, flow_inst::Column::RelBusinessObjId),
                (flow_inst::Entity, flow_inst::Column::CurrentStateId),
                (flow_inst::Entity, flow_inst::Column::Main),
                (flow_inst::Entity, flow_inst::Column::CurrentVars),
                (flow_inst::Entity, flow_inst::Column::CreateVars),
                (flow_inst::Entity, flow_inst::Column::CreateCtx),
                (flow_inst::Entity, flow_inst::Column::CreateTime),
                (flow_inst::Entity, flow_inst::Column::UpdateTime),
                (flow_inst::Entity, flow_inst::Column::FinishCtx),
                (flow_inst::Entity, flow_inst::Column::FinishTime),
                (flow_inst::Entity, flow_inst::Column::FinishAbort),
                (flow_inst::Entity, flow_inst::Column::OutputMessage),
                (flow_inst::Entity, flow_inst::Column::Transitions),
                (flow_inst::Entity, flow_inst::Column::OwnPaths),
                (flow_inst::Entity, flow_inst::Column::Artifacts),
                (flow_inst::Entity, flow_inst::Column::Comments),
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
            .expr_as(Expr::col((rel_state_ext_table.clone(), Alias::new("ext"))).if_null(""), Alias::new("current_state_ext"))
            .expr_as(
                Expr::col((flow_model_version_table.clone(), Alias::new("rel_model_id"))).if_null(""),
                Alias::new("rel_flow_model_id"),
            )
            .expr_as(
                Expr::col((rel_model_version_table.clone(), NAME_FIELD.clone())).if_null(""),
                Alias::new("rel_flow_model_name"),
            )
            .expr_as(Expr::col((rel_model_table.clone(), Alias::new("ext"))).if_null(""), Alias::new("rel_transition"))
            .from(flow_inst::Entity)
            .join_as(
                JoinType::LeftJoin,
                RBUM_ITEM_TABLE.clone(),
                rel_state_table.clone(),
                Cond::all()
                    .add(Expr::col((rel_state_table.clone(), ID_FIELD.clone())).equals((flow_inst::Entity, flow_inst::Column::CurrentStateId)))
                    .add(Expr::col((rel_state_table.clone(), REL_KIND_ID_FIELD.clone())).eq(FlowStateServ::get_rbum_kind_id().unwrap_or_default()))
                    .add(Expr::col((rel_state_table.clone(), REL_DOMAIN_ID_FIELD.clone())).eq(FlowStateServ::get_rbum_domain_id().unwrap_or_default())),
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
                    .add(Expr::col((rel_model_version_table.clone(), REL_KIND_ID_FIELD.clone())).eq(FlowModelVersionServ::get_rbum_kind_id().unwrap_or_default()))
                    .add(Expr::col((rel_model_version_table.clone(), REL_DOMAIN_ID_FIELD.clone())).eq(FlowModelVersionServ::get_rbum_domain_id().unwrap_or_default())),
            )
            .join_as(
                JoinType::LeftJoin,
                Alias::new("rbum_rel"),
                rel_state_ext_table.clone(),
                Cond::all()
                    .add(Expr::col((rel_state_ext_table.clone(), Alias::new("to_rbum_item_id"))).equals((flow_inst::Entity, flow_inst::Column::CurrentStateId)))
                    .add(Expr::col((rel_state_ext_table.clone(), Alias::new("from_rbum_id"))).equals((flow_inst::Entity, flow_inst::Column::RelFlowVersionId)))
                    .add(Expr::col((rel_state_ext_table.clone(), Alias::new("tag"))).eq("FlowModelState".to_string())),
            )
            .join_as(
                JoinType::LeftJoin,
                flow_model_version_table.clone(),
                flow_model_version_table.clone(),
                Cond::all().add(Expr::col((flow_model_version_table.clone(), ID_FIELD.clone())).equals((flow_inst::Entity, flow_inst::Column::RelFlowVersionId))),
            )
            .join_as(
                JoinType::LeftJoin,
                Alias::new("rbum_rel"),
                rel_model_table.clone(),
                Cond::all()
                    .add(Expr::col((rel_model_table.clone(), Alias::new("from_rbum_id"))).equals((flow_model_version_table.clone(), flow_model_version::Column::RelModelId)))
                    .add(Expr::col((rel_model_table.clone(), Alias::new("tag"))).eq("FlowModelTransition".to_string())),
            )
            .and_where(Expr::col((flow_inst::Entity, flow_inst::Column::Id)).is_in(flow_inst_ids))
            .and_where(Expr::col((flow_inst::Entity, flow_inst::Column::OwnPaths)).like(format!("{}%", ctx.own_paths)));
        if let Some(sort) = desc_sort_by_create {
            query.order_by((flow_inst::Entity, CREATE_TIME_FIELD.clone()), if sort { Order::Desc } else { Order::Asc });
        }
        if let Some(sort) = desc_sort_by_update {
            query.order_by((flow_inst::Entity, UPDATE_TIME_FIELD.clone()), if sort { Order::Desc } else { Order::Asc });
        }
        let flow_insts = funs.db().find_dtos::<FlowInstDetailResult>(&query).await?;
        let result = flow_insts
            .into_iter()
            .map(|inst| {
                let current_state_kind_conf = inst
                    .current_state_kind_conf
                    .clone()
                    .map(|current_state_kind_conf| TardisFuns::json.json_to_obj::<FLowStateKindConf>(current_state_kind_conf).unwrap_or_default());
                let artifacts = inst.artifacts.clone().map(|artifacts| TardisFuns::json.json_to_obj::<FlowInstArtifacts>(artifacts).unwrap_or_default());
                let rel_transition = inst.rel_transition.map(|ext| {
                    if ext.is_empty() {
                        return FlowModelRelTransitionExt::default();
                    }
                    TardisFuns::json.str_to_obj::<FlowModelRelTransitionExt>(&ext).unwrap_or_default()
                });
                FlowInstDetailResp {
                    id: inst.id,
                    code: inst.code,
                    rel_flow_version_id: inst.rel_flow_version_id,
                    rel_flow_model_id: inst.rel_flow_model_id,
                    rel_flow_model_name: inst.rel_flow_model_name,
                    tag: inst.tag,
                    main: inst.main,
                    create_vars: inst.create_vars.map(|create_vars| TardisFuns::json.json_to_obj(create_vars).unwrap_or_default()),
                    create_ctx: inst.create_ctx,
                    create_time: inst.create_time,
                    update_time: inst.update_time,
                    finish_ctx: inst.finish_ctx,
                    finish_time: inst.finish_time,
                    finish_abort: inst.finish_abort,
                    output_message: inst.output_message,
                    own_paths: inst.own_paths,
                    transitions: inst.transitions.map(|transitions| TardisFuns::json.json_to_obj(transitions).unwrap_or_default()),
                    artifacts: artifacts.clone(),
                    comments: inst.comments.map(|comments| TardisFuns::json.json_to_obj(comments).unwrap_or_default()),
                    rel_transition,
                    current_state_id: inst.current_state_id.clone(),
                    current_state_name: inst.current_state_name,
                    current_state_color: inst.current_state_color,
                    current_state_sys_kind: inst.current_state_sys_kind,
                    current_state_kind: inst.current_state_kind.clone(),
                    current_state_ext: inst.current_state_ext.map(|ext| {
                        if ext.is_empty() {
                            return FlowStateRelModelExt::default();
                        }
                        TardisFuns::json.str_to_obj::<FlowStateRelModelExt>(&ext).unwrap_or_default()
                    }),
                    current_state_conf: Self::get_state_conf(
                        &inst.current_state_id,
                        &inst.current_state_kind.unwrap_or_default(),
                        current_state_kind_conf,
                        artifacts,
                        inst.finish_time.is_some(),
                        ctx,
                    ),
                    current_vars: inst.current_vars.map(|current_vars| TardisFuns::json.json_to_obj(current_vars).unwrap_or_default()),
                    rel_business_obj_id: inst.rel_business_obj_id,
                }
            })
            .collect_vec();
        Ok(result
            .into_iter()
            .map(|mut inst_detail| {
                let mut artifacts = inst_detail.artifacts.clone();
                if let Some(artifacts) = artifacts.as_mut() {
                    let mut curr_vars = artifacts.curr_vars.clone().unwrap_or_default();
                    curr_vars.extend(Self::get_modify_vars(&inst_detail));

                    artifacts.curr_vars = Some(curr_vars);
                }
                inst_detail.artifacts = artifacts;
                inst_detail
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
                    code: inst.code,
                    rel_flow_version_id: inst.rel_flow_version_id,
                    rel_flow_model_id: inst.rel_flow_model_id,
                    rel_flow_model_name: inst.rel_flow_model_name,
                    create_ctx: TardisFuns::json.json_to_obj(inst.create_ctx).unwrap_or_default(),
                    create_time: inst.create_time,
                    update_time: inst.update_time,
                    finish_ctx: inst.finish_ctx.map(|finish_ctx| TardisFuns::json.json_to_obj(finish_ctx).unwrap_or_default()),
                    finish_time: inst.finish_time,
                    finish_abort: inst.finish_abort.is_some(),
                    output_message: inst.output_message,
                    own_paths: inst.own_paths,
                    current_state_id: inst.current_state_id,
                    current_state_name: inst.current_state_name,
                    rel_business_obj_id: inst.rel_business_obj_id,
                    rel_transition: inst.rel_transition.map(|ext| TardisFuns::json.str_to_obj::<FlowModelRelTransitionExt>(&ext).unwrap_or_default()),
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
        let flow_inst_ids = find_req.iter().map(|req| req.flow_inst_id.to_string()).unique().collect_vec();
        let flow_insts = Self::find_detail(flow_inst_ids.clone(), None, None, funs, ctx).await?;
        if flow_insts.len() != flow_inst_ids.len() {
            return Err(funs.err().not_found("flow_inst", "find_state_and_next_transitions", "some flow instances not found", "404-flow-inst-not-found"));
        }
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
                    if let (Some(req), Some(rel_flow_versions)) = (
                        find_req.iter().find(|req| req.flow_inst_id == flow_inst.id),
                        rel_flow_version_map.get(&flow_inst.tag).cloned(),
                    ) {
                        Self::do_find_next_transitions(flow_inst, None, &req.vars, rel_flow_versions, false, funs, ctx).await.ok()
                    } else {
                        None
                    }
                })
                .collect_vec(),
        )
        .await
        .into_iter()
        .flatten()
        .collect_vec();
        // 若当前数据项存在未结束的审批流，则清空其中的transitions
        let unfinished_approve_flow_inst_ids = Self::find_detail_items(
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
        .filter_map(|inst| flow_insts.iter().find(|flow_inst| flow_inst.rel_business_obj_id == inst.rel_business_obj_id).map(|r| r.id.clone()))
        .collect_vec();
        for item in state_and_next_transitions.iter_mut() {
            if unfinished_approve_flow_inst_ids.contains(&item.flow_inst_id) {
                item.next_flow_transitions.clear();
            }
        }
        Ok(state_and_next_transitions)
    }

    pub async fn find_next_transitions(
        flow_inst: &FlowInstDetailResp,
        next_req: &FlowInstFindNextTransitionsReq,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<FlowInstFindNextTransitionResp>> {
        let rel_flow_versions = FlowTransitionServ::find_rel_model_map(&flow_inst.tag, funs, ctx).await?;
        let state_and_next_transitions = Self::do_find_next_transitions(flow_inst, None, &next_req.vars, rel_flow_versions, false, funs, ctx).await?;
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
        Self::auto_transfer(&flow_inst_detail.id, modified_instance_transations_cp.clone(), funs, ctx).await?;

        if !flow_inst_detail.main {
            FlowSearchClient::async_add_or_modify_instance_search(&flow_inst_detail.id, Box::new(true), funs, ctx).await?;
        }

        if flow_inst_detail.main {
            let flow_inst_cp = flow_inst_detail.clone();
            let flow_transition_id = transfer_req.flow_transition_id.clone();
            let ctx_cp = ctx.clone();
            tardis::tokio::spawn(async move {
                let funs = flow_constants::get_tardis_inst();
                match FlowEventServ::do_post_change(&flow_inst_cp, &flow_transition_id, modified_instance_transations_cp.clone(), &ctx_cp, &funs).await {
                    Ok(_) => {}
                    Err(e) => error!("Flow Instance {} do_post_change error:{:?}", flow_inst_cp.id, e),
                }
                match FlowEventServ::do_front_change(&flow_inst_cp, modified_instance_transations_cp.clone(), &ctx_cp, &funs).await {
                    Ok(_) => {}
                    Err(e) => error!("Flow Instance {} do_front_change error:{:?}", flow_inst_cp.id, e),
                }
            });
        }

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
                    .map(|tran| tran.from_flow_state_id)
                    .unwrap_or_default(),
                ctx,
                funs,
            )
            .await;
        }
        let version_transition = FlowTransitionServ::find_transitions(&flow_model_version.id, None, funs, ctx).await?;

        let next_flow_transition = next_flow_transition.unwrap_or_default();
        let next_transition_detail = version_transition.iter().find(|trans| trans.id == next_flow_transition.next_flow_transition_id).cloned().unwrap_or_default();
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
            update_time: Set(Some(Utc::now())),
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

        let curr_inst = Self::get(&flow_inst_detail.id, funs, ctx).await?;

        if next_flow_state.sys_state == FlowSysStateKind::Finish && !curr_inst.main {
            FlowLogServ::add_finish_log(&curr_inst, funs, ctx).await?;
            let rel_transition_names = Self::get_rel_transitions(&curr_inst.rel_business_obj_id, funs, ctx).await?.into_iter().map(|rel| rel.to_string()).collect_vec();
            FlowSearchClient::modify_business_obj_search(
                &curr_inst.rel_business_obj_id,
                &curr_inst.tag,
                json!({
                    "rel_transitions": rel_transition_names,
                }),
                funs,
                ctx,
            )
            .await?;
        }

        Self::when_leave_state(&curr_inst, &prev_flow_state.id, &flow_model_version.rel_model_id, funs, ctx).await?;
        Self::when_enter_state(&curr_inst, &next_flow_state.id, &flow_model_version.rel_model_id, funs, ctx).await?;

        Self::do_request_webhook(
            from_transition_id.and_then(|id| version_transition.iter().find(|model_transition| model_transition.id == id)),
            Some(&next_transition_detail),
        )
        .await?;

        // notify change state
        if curr_inst.main {
            FlowExternalServ::do_notify_changes(
                &curr_inst.tag,
                &curr_inst.id,
                &curr_inst.rel_business_obj_id,
                next_flow_state.name.clone(),
                next_flow_state.sys_state.clone(),
                prev_flow_state.name.clone(),
                prev_flow_state.sys_state.clone(),
                next_transition_detail.name.clone(),
                next_transition_detail.is_notify,
                Some(!(callback_kind == FlowExternalCallbackOp::PostAction || callback_kind == FlowExternalCallbackOp::ConditionalTrigger)),
                Some(callback_kind),
                ctx,
                funs,
            )
            .await?;
        }
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
                    guard_conf: None,
                });
            }
            if !params.is_empty() && flow_inst_detail.main {
                FlowExternalServ::do_async_modify_field(
                    &flow_inst_detail.tag,
                    Some(next_transition_detail.clone()),
                    &flow_inst_detail.rel_business_obj_id,
                    &flow_inst_detail.id,
                    Some(FlowExternalCallbackOp::VerifyContent),
                    Some(true),
                    None,
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

        Self::gen_transfer_resp(&curr_inst, &prev_flow_state.id, ctx, funs).await
    }

    async fn gen_transfer_resp(flow_inst_detail: &FlowInstDetailResp, prev_flow_state_id: &str, ctx: &TardisContext, funs: &TardisFunsInst) -> TardisResult<FlowInstTransferResp> {
        let global_ctx = TardisContext {
            own_paths: "".to_string(),
            ..ctx.clone()
        };

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
        let next_flow_transitions = Self::do_find_next_transitions(flow_inst_detail, None, &None, rel_flow_versions, false, funs, ctx).await?.next_flow_transitions;

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
        spec_flow_transition_id: Option<String>,
        req_vars: &Option<HashMap<String, Value>>,
        rel_flow_versions: HashMap<String, String>,
        skip_filter: bool,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<FlowInstFindStateAndTransitionsResp> {
        let flow_model_transitions = FlowTransitionServ::find_transitions(&flow_inst.rel_flow_version_id, None, funs, ctx).await?;

        let next_transitions = flow_model_transitions
            .iter()
            .filter(|model_transition| {
                model_transition.from_flow_state_id == flow_inst.current_state_id
                    && (spec_flow_transition_id.is_none() || model_transition.id == spec_flow_transition_id.clone().unwrap_or_default())
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
                        .cloned()
                        .unwrap_or(json!({}))
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
                    // 若 check_or_and_conds 报错，则表示条件配置有问题，忽略无效的配置直接给true
                    if !BasicQueryCondInfo::check_or_and_conds(&guard_by_other_conds, &check_vars).unwrap_or(true) {
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
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let mut new_vars: HashMap<String, Value> = HashMap::new();
        if let Some(old_current_vars) = &flow_inst_detail.current_vars {
            new_vars.extend(old_current_vars.clone());
        }
        new_vars.extend(current_vars.clone());
        let flow_inst = flow_inst::ActiveModel {
            id: Set(flow_inst_detail.id.clone()),
            current_vars: Set(Some(TardisFuns::json.obj_to_json(&new_vars)?)),
            update_time: Set(Some(Utc::now())),
            ..Default::default()
        };
        funs.db().update_one(flow_inst, ctx).await?;
        if flow_inst_detail.main {
            let curr_inst = Self::get(&flow_inst_detail.id, funs, ctx).await?;
            let ctx_cp = ctx.clone();
            let modified_instance_transations_cp = modified_instance_transations.clone();
            tardis::tokio::spawn(async move {
                let funs = flow_constants::get_tardis_inst();
                match FlowEventServ::do_front_change(&curr_inst, modified_instance_transations_cp.clone(), &ctx_cp, &funs).await {
                    Ok(_) => {}
                    Err(e) => error!("Flow Instance {} do_front_change error:{:?}", curr_inst.id, e),
                }
                funs.commit().await.unwrap_or_default();
            });
        }

        Ok(())
    }

    async fn get_new_vars(tag: &str, rel_business_obj_id: String, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<HashMap<String, Value>> {
        let mut resp = FlowExternalServ::do_query_field(tag, vec![rel_business_obj_id.clone()], &ctx.own_paths, ctx, funs)
            .await?
            .objs
            .pop()
            .map(|val| TardisFuns::json.json_to_obj::<HashMap<String, Value>>(val).unwrap_or_default())
            .unwrap_or_default();
        // 添加当前状态名称
        if let Some(flow_id) = Self::get_inst_ids_by_rel_business_obj_id(vec![rel_business_obj_id], Some(true), funs, ctx).await?.pop() {
            let current_state_name = Self::get(&flow_id, funs, ctx).await?.current_state_name.unwrap_or_default();
            resp.insert("status".to_string(), json!(current_state_name));
        }

        Ok(resp)
    }

    pub async fn find_var_by_inst_id(flow_inst: &FlowInstDetailResp, key: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<Value>> {
        let mut current_vars = flow_inst.current_vars.clone();
        if current_vars.is_none() || !current_vars.clone().unwrap_or_default().contains_key(key) {
            let new_vars = Self::get_new_vars(&flow_inst.tag, flow_inst.rel_business_obj_id.clone(), funs, ctx).await?;
            Self::modify_current_vars(flow_inst, &new_vars, loop_check_helper::InstancesTransition::default(), funs, ctx).await?;
            current_vars = Self::get(&flow_inst.id, funs, ctx).await?.current_vars;
        }

        Ok(current_vars.unwrap_or_default().get(key).cloned())
    }

    pub async fn batch_update_when_switch_model(
        new_model: &FlowModelAggResp,
        rel_template_id: Option<String>,
        update_states: Option<HashMap<String, String>>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let mut own_paths_list = vec![];
        if let Some(rel_template_id) = rel_template_id {
            own_paths_list = FlowRelServ::find_to_simple_rels(&FlowRelKind::FlowAppTemplate, &rel_template_id, None, None, funs, ctx)
                .await?
                .into_iter()
                .map(|rel| {
                    if FlowModelServ::get_app_id_by_ctx(ctx).is_some() {
                        rel.rel_own_paths
                    } else {
                        format!("{}/{}", rel.rel_own_paths, rel.rel_id)
                    }
                })
                .collect_vec();
            if own_paths_list.contains(&ctx.own_paths) {
                own_paths_list = vec![ctx.own_paths.clone()];
            }
        } else {
            own_paths_list.push(ctx.own_paths.clone());
        }
        for own_paths in own_paths_list {
            let mock_ctx = TardisContext { own_paths, ..ctx.clone() };
            if let Some(update_states) = &update_states {
                for (old_state, new_state) in update_states {
                    Self::unsafe_modify_state(&new_model.tag, Some(vec![old_state.clone()]), new_state, funs, &mock_ctx).await?;
                }
            } else {
                Self::unsafe_modify_state(&new_model.tag, None, &new_model.init_state_id, funs, &mock_ctx).await?;
            }
            Self::unsafe_modify_rel_model_id(&new_model.tag, &new_model.current_version_id, funs, &mock_ctx).await?;
        }

        Ok(())
    }

    async fn unsafe_modify_rel_model_id(tag: &str, modify_version_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let mut update_statement = Query::update();
        update_statement.table(flow_inst::Entity);
        update_statement.value(flow_inst::Column::RelFlowVersionId, modify_version_id);
        update_statement.and_where(Expr::col((flow_inst::Entity, flow_inst::Column::Tag)).eq(tag));
        update_statement.and_where(Expr::col((flow_inst::Entity, flow_inst::Column::Main)).eq(true));
        update_statement.and_where(Expr::col((flow_inst::Entity, flow_inst::Column::OwnPaths)).eq(ctx.own_paths.as_str()));

        funs.db().execute(&update_statement).await?;

        Ok(())
    }

    pub async fn unsafe_abort_inst(rel_flow_version_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let insts = Self::find_detail_items(
            &FlowInstFilterReq {
                main: Some(false),
                flow_version_id: Some(rel_flow_version_id.to_string()),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
        .into_iter()
        .collect_vec();
        join_all(insts.iter().map(|inst| async { Self::abort(&inst.id, &FlowInstAbortReq { message: "".to_string() }, funs, ctx).await }).collect_vec())
            .await
            .into_iter()
            .collect::<TardisResult<Vec<_>>>()?;
        Ok(())
    }

    pub async fn unsafe_modify_state(tag: &str, modify_model_state_ids: Option<Vec<String>>, state_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let global_ctx = TardisContext {
            own_paths: "".to_string(),
            ..Default::default()
        };
        let insts = Self::find_detail_items(
            &FlowInstFilterReq {
                main: Some(true),
                tag: Some(tag.to_string()),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
        .into_iter()
        .filter(|inst| modify_model_state_ids.is_none() || modify_model_state_ids.clone().unwrap_or_default().contains(&inst.current_state_id))
        .collect_vec();
        let inst_ids = insts.iter().map(|inst| inst.id.clone()).collect_vec();
        let mut update_statement = Query::update();
        update_statement.table(flow_inst::Entity);
        update_statement.value(flow_inst::Column::CurrentStateId, state_id);
        update_statement.and_where(Expr::col((flow_inst::Entity, flow_inst::Column::Id)).is_in(inst_ids));

        funs.db().execute(&update_statement).await?;
        join_all(
            insts
                .iter()
                .map(|inst| async {
                    if let (Ok(original_flow_state), Ok(next_flow_state)) = (
                        FlowStateServ::get_item(
                            &inst.current_state_id,
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
                        .await,
                        FlowStateServ::get_item(
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
                        .await,
                    ) {
                        FlowExternalServ::do_notify_changes(
                            &inst.tag,
                            &inst.id,
                            &inst.rel_business_obj_id,
                            next_flow_state.name.clone(),
                            next_flow_state.sys_state,
                            original_flow_state.name.clone(),
                            original_flow_state.sys_state,
                            "UPDATE".to_string(),
                            false,
                            Some(false),
                            Some(FlowExternalCallbackOp::Auto),
                            ctx,
                            funs,
                        )
                        .await
                    } else {
                        Err(funs.err().not_found("flow_inst", "unsafe_modify_state", "flow state not found", "404-flow-state-not-found"))
                    }
                })
                .collect_vec(),
        )
        .await
        .into_iter()
        .collect::<TardisResult<Vec<_>>>()?;
        Ok(())
    }

    pub async fn auto_transfer(
        flow_inst_id: &str,
        modified_instance_transations: loop_check_helper::InstancesTransition,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let flow_inst_detail = Self::get(flow_inst_id, funs, ctx).await?;
        let transition_ids = Self::do_find_next_transitions(&flow_inst_detail, None, &None, HashMap::default(), false, funs, ctx)
            .await?
            .next_flow_transitions
            .into_iter()
            .map(|tran| tran.next_flow_transition_id)
            .collect_vec();
        let create_vars = flow_inst_detail.create_vars.clone().unwrap_or_default();
        let auto_transitions =
            FlowTransitionServ::find_detail_items(transition_ids, None, None, funs, ctx).await?.into_iter().filter(|transition| transition.transfer_by_auto).collect_vec();
        if !auto_transitions.is_empty() {
            if let Some(auto_transition) = auto_transitions.into_iter().find(|transition| {
                (transition.transfer_by_auto && transition.guard_by_other_conds().is_none())
                    || (transition.transfer_by_auto
                        && transition.guard_by_other_conds().is_some()
                        && BasicQueryCondInfo::check_or_and_conds(&transition.guard_by_other_conds().unwrap_or_default(), &create_vars).unwrap_or(true))
            }) {
                Self::transfer(
                    &flow_inst_detail,
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
            } else {
                Self::abort(&flow_inst_detail.id, &FlowInstAbortReq { message: "".to_string() }, funs, ctx).await?;
            }
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
                if state.own_paths != flow_inst_detail.own_paths {
                    guard_custom_conf.get_local_conf(funs, ctx).await?;
                }
                if form_conf.guard_by_creator {
                    guard_custom_conf.guard_by_spec_account_ids.push(flow_inst_detail.create_ctx.owner.clone());
                }
                if form_conf.guard_by_his_operators {
                    flow_inst_detail.transitions.as_ref().map(|transitions| {
                        transitions.iter().map(|transition| guard_custom_conf.guard_by_spec_account_ids.push(transition.op_ctx.owner.clone())).collect::<Vec<_>>()
                    });
                }
                if form_conf.guard_by_assigned {
                    let _ = flow_inst_detail
                        .create_vars
                        .clone()
                        .unwrap_or_default()
                        .get("assigned_to")
                        .unwrap_or(&json!(""))
                        .as_array()
                        .unwrap_or(&vec![flow_inst_detail
                            .create_vars
                            .clone()
                            .unwrap_or_default()
                            .get("assigned_to")
                            .unwrap_or(&json!(""))
                            .clone()])
                        .iter()
                        .map(|v| v.as_str().unwrap_or(""))
                        .collect_vec()
                        .into_iter()
                        .map(|str| {
                            if !str.is_empty() {
                                guard_custom_conf.guard_by_spec_account_ids.push(str.to_string());
                            }
                        })
                        .collect::<Vec<_>>();
                }
                modify_req.curr_operators = Some(FlowSearchClient::search_guard_accounts(&guard_custom_conf, funs, ctx).await?);
                modify_req.prohibit_guard_conf_account_ids = Some(vec![]);
                modify_req.state = Some(FlowInstStateKind::Form);
                Self::modify_inst_artifacts(&flow_inst_detail.id, &modify_req, funs, ctx).await?;
                // 当操作人为空时的逻辑
                let curr_operators = Self::get(&flow_inst_detail.id, funs, ctx).await?.artifacts.unwrap_or_default().curr_operators.unwrap_or_default();
                if curr_operators.is_empty() && form_conf.auto_transfer_when_empty_kind.is_some() {
                    match form_conf.auto_transfer_when_empty_kind.unwrap_or_default() {
                        FlowStatusAutoStrategyKind::Autoskip => {
                            if let Some(next_transition) = Self::find_next_transitions(flow_inst_detail, &FlowInstFindNextTransitionsReq { vars: None }, funs, ctx).await?.pop() {
                                FlowLogServ::add_operate_log(
                                    &FlowInstOperateReq {
                                        operate: FlowStateOperatorKind::Submit,
                                        vars: None,
                                        all_vars: None,
                                        output_message: None,
                                        operator: None,
                                        log_text: None,
                                    },
                                    flow_inst_detail,
                                    LogParamOp::FormTransfer,
                                    funs,
                                    ctx,
                                )
                                .await?;
                                Self::transfer(
                                    flow_inst_detail,
                                    &FlowInstTransferReq {
                                        flow_transition_id: next_transition.next_flow_transition_id,
                                        message: None,
                                        vars: None,
                                    },
                                    false,
                                    FlowExternalCallbackOp::Auto,
                                    loop_check_helper::InstancesTransition::default(),
                                    ctx,
                                    funs,
                                )
                                .await?;
                            }
                        }
                        FlowStatusAutoStrategyKind::SpecifyAgent => {
                            modify_req.curr_operators =
                                Some(FlowSearchClient::search_guard_accounts(&form_conf.auto_transfer_when_empty_guard_custom_conf.clone().unwrap_or_default(), funs, ctx).await?);
                            modify_req.prohibit_guard_conf_account_ids = Some(vec![]);
                            Self::modify_inst_artifacts(&flow_inst_detail.id, &modify_req, funs, ctx).await?;
                        }
                        FlowStatusAutoStrategyKind::TransferState => {
                            // 当前版本不支持
                        }
                    }
                }
            }
            FlowStateKind::Approval => {
                let mut modify_req = FlowInstArtifactsModifyReq { ..Default::default() };
                let approval_conf = state.kind_conf().unwrap_or_default().approval.unwrap_or_default();
                let mut guard_custom_conf = approval_conf.guard_custom_conf.unwrap_or_default();
                if state.own_paths != flow_inst_detail.own_paths {
                    guard_custom_conf.get_local_conf(funs, ctx).await?;
                }
                if approval_conf.guard_by_creator {
                    guard_custom_conf.guard_by_spec_account_ids.push(flow_inst_detail.create_ctx.owner.clone());
                }
                if approval_conf.guard_by_his_operators {
                    flow_inst_detail.transitions.as_ref().map(|transitions| {
                        transitions.iter().map(|transition| guard_custom_conf.guard_by_spec_account_ids.push(transition.op_ctx.owner.clone())).collect::<Vec<_>>()
                    });
                }
                if approval_conf.guard_by_assigned {
                    let _ = flow_inst_detail
                        .create_vars
                        .clone()
                        .unwrap_or_default()
                        .get("assigned_to")
                        .unwrap_or(&json!(""))
                        .as_array()
                        .unwrap_or(&vec![flow_inst_detail
                            .create_vars
                            .clone()
                            .unwrap_or_default()
                            .get("assigned_to")
                            .unwrap_or(&json!(""))
                            .clone()])
                        .iter()
                        .map(|v| v.as_str().unwrap_or(""))
                        .collect_vec()
                        .into_iter()
                        .map(|str| {
                            if !str.is_empty() {
                                guard_custom_conf.guard_by_spec_account_ids.push(str.to_string());
                            }
                        })
                        .collect::<Vec<_>>();
                }
                let guard_accounts = FlowSearchClient::search_guard_accounts(&guard_custom_conf, funs, ctx).await?;
                let curr_approval_total = guard_accounts.len();
                modify_req.curr_approval_total = Some(curr_approval_total);
                modify_req.curr_operators = Some(guard_accounts);
                modify_req.prohibit_guard_conf_account_ids = Some(vec![]);
                modify_req.state = Some(FlowInstStateKind::Approval);

                Self::modify_inst_artifacts(&flow_inst_detail.id, &modify_req, funs, ctx).await?;
                // 当操作人为空时的逻辑
                if curr_approval_total == 0 && approval_conf.auto_transfer_when_empty_kind.is_some() {
                    match approval_conf.auto_transfer_when_empty_kind.unwrap_or_default() {
                        FlowStatusAutoStrategyKind::Autoskip => {
                            if let Some(next_transition) = Self::find_next_transitions(flow_inst_detail, &FlowInstFindNextTransitionsReq { vars: None }, funs, ctx).await?.pop() {
                                FlowLogServ::add_operate_log(
                                    &FlowInstOperateReq {
                                        operate: FlowStateOperatorKind::Pass,
                                        vars: None,
                                        all_vars: None,
                                        output_message: None,
                                        operator: None,
                                        log_text: None,
                                    },
                                    flow_inst_detail,
                                    LogParamOp::ApprovalTransfer,
                                    funs,
                                    ctx,
                                )
                                .await?;
                                Self::transfer(
                                    flow_inst_detail,
                                    &FlowInstTransferReq {
                                        flow_transition_id: next_transition.next_flow_transition_id,
                                        message: None,
                                        vars: None,
                                    },
                                    false,
                                    FlowExternalCallbackOp::Auto,
                                    loop_check_helper::InstancesTransition::default(),
                                    ctx,
                                    funs,
                                )
                                .await?;
                            }
                        }
                        FlowStatusAutoStrategyKind::SpecifyAgent => {
                            let mut auto_transfer_when_empty_guard_custom_conf = approval_conf.auto_transfer_when_empty_guard_custom_conf.clone().unwrap_or_default();
                            if state.own_paths != flow_inst_detail.own_paths {
                                auto_transfer_when_empty_guard_custom_conf.get_local_conf(funs, ctx).await?;
                            }
                            let guard_accounts = FlowSearchClient::search_guard_accounts(&auto_transfer_when_empty_guard_custom_conf, funs, ctx).await?;
                            modify_req.curr_approval_total = Some(guard_accounts.len());
                            modify_req.curr_operators = Some(guard_accounts);
                            modify_req.prohibit_guard_conf_account_ids = Some(vec![]);
                            Self::modify_inst_artifacts(&flow_inst_detail.id, &modify_req, funs, ctx).await?;
                        }
                        FlowStatusAutoStrategyKind::TransferState => {
                            // @TODO 当前版本不支持
                        }
                    }
                }
            }
            FlowStateKind::Branch => {}
            FlowStateKind::Finish => match rel_transition.as_str() {
                "__EDIT__" => {
                    let vars_collect = Self::get_modify_vars(flow_inst_detail);
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
                        Some(FlowExternalCallbackOp::Auto),
                        None,
                        Some("审批通过".to_string()),
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
                    let vars_collect = Self::get_modify_vars(flow_inst_detail);
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
                        Some("审批通过".to_string()),
                        None,
                        None,
                        None,
                        None,
                        params,
                        ctx,
                        funs,
                    )
                    .await?;
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
    async fn when_leave_state(_flow_inst_detail: &FlowInstDetailResp, state_id: &str, _flow_model_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
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
            FlowStateKind::Form => {}
            FlowStateKind::Approval => {}
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
        if let Some(state) = &modify_artifacts.state {
            inst_artifacts.state = Some(state.clone());
        }
        if let Some(curr_operators) = &modify_artifacts.curr_operators {
            inst_artifacts.curr_operators = Some(curr_operators.clone());
        }
        if let Some(add_his_operator) = &modify_artifacts.add_his_operator {
            let mut his_operators = inst_artifacts.his_operators.clone().unwrap_or_default();
            if !his_operators.contains(add_his_operator) {
                his_operators.push(add_his_operator.clone());
                inst_artifacts.his_operators = Some(his_operators);
            }
        }
        if let Some(prohibit_guard_conf_account_ids) = &modify_artifacts.prohibit_guard_conf_account_ids {
            inst_artifacts.prohibit_guard_by_spec_account_ids = Some(prohibit_guard_conf_account_ids.clone());
        }
        if let Some((add_approval_account_id, add_approval_result)) = &modify_artifacts.add_approval_result {
            let current_state_result = inst_artifacts.approval_result.entry(inst.current_state_id.clone()).or_default();
            let current_account_ids = current_state_result.entry(add_approval_result.to_string()).or_default();
            current_account_ids.push(add_approval_account_id.clone());
        }
        if let Some(curr_approval_total) = modify_artifacts.curr_approval_total {
            let mut approval_total = inst_artifacts.approval_total.clone().unwrap_or_default();
            let approval_state_total = approval_total.entry(inst.current_state_id.clone()).or_default();
            *approval_state_total = curr_approval_total;
            inst_artifacts.approval_total = Some(approval_total);
        }
        if let Some(form_state_vars) = modify_artifacts.form_state_map.clone() {
            let vars_collect = inst_artifacts.form_state_map.entry(inst.current_state_id.clone()).or_default();
            for (key, value) in form_state_vars {
                *vars_collect.entry(key.clone()).or_insert(json!({})) = value.clone();
            }
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
        if let Some(curr_vars) = &modify_artifacts.curr_vars {
            inst_artifacts.curr_vars = Some(curr_vars.clone());
        }
        if let Some((referral_account_id, master_account_id)) = &modify_artifacts.add_referral_map {
            let current_referral_map = inst_artifacts.referral_map.entry(inst.current_state_id.clone()).or_default();
            current_referral_map.entry(referral_account_id.clone()).and_modify(|result| *result = master_account_id.clone()).or_default();
        }
        if let Some(remove_account_id) = &modify_artifacts.remove_referral_map {
            let current_referral_map = inst_artifacts.referral_map.entry(inst.current_state_id.clone()).or_default();
            current_referral_map.remove(remove_account_id);
        }
        if let Some(state_id) = &modify_artifacts.clear_referral_map {
            inst_artifacts.referral_map.remove(state_id);
        }
        let flow_inst = flow_inst::ActiveModel {
            id: Set(inst.id.clone()),
            artifacts: Set(Some(inst_artifacts)),
            update_time: Set(Some(Utc::now())),
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
        finish: bool,
        ctx: &TardisContext,
    ) -> Option<FLowInstStateConf> {
        if let Some(kind_conf) = kind_conf {
            match state_kind {
                FlowStateKind::Form => kind_conf.form.as_ref().map(|form| {
                    let mut operators = HashMap::new();
                    let artifacts = artifacts.clone().unwrap_or_default();
                    if !finish
                        && (artifacts.curr_operators.clone().unwrap_or_default().contains(&ctx.owner)
                            || artifacts.referral_map.get(state_id).map_or_else(|| false, |current_referral_map| current_referral_map.contains_key(&ctx.owner)))
                        && !artifacts.prohibit_guard_by_spec_account_ids.clone().unwrap_or_default().contains(&ctx.owner)
                    {
                        operators.insert(FlowStateOperatorKind::Submit, form.submit_btn_name.clone());
                        if form.referral {
                            operators.insert(FlowStateOperatorKind::Referral, "".to_string());
                        }
                    }
                    FLowInstStateConf {
                        operators,
                        form_conf: Some(FLowInstStateFormConf {
                            form_vars_collect_conf: form.vars_collect.clone(),
                            form_referral_guard_custom_conf: form.referral_guard_custom_conf.clone(),
                        }),
                        approval_conf: None,
                    }
                }),
                FlowStateKind::Approval => kind_conf.approval.as_ref().map(|approval| {
                    let mut operators = HashMap::new();
                    let artifacts = artifacts.clone().unwrap_or_default();
                    if !finish {
                        if (artifacts.curr_operators.clone().unwrap_or_default().contains(&ctx.owner) || artifacts.referral_map.contains_key(&ctx.owner))
                            && !artifacts.prohibit_guard_by_spec_account_ids.clone().unwrap_or_default().contains(&ctx.owner)
                        {
                            operators.insert(FlowStateOperatorKind::Pass, approval.pass_btn_name.clone());
                            operators.insert(FlowStateOperatorKind::Overrule, approval.overrule_btn_name.clone());
                            operators.insert(FlowStateOperatorKind::Back, approval.back_btn_name.clone());
                            if approval.referral {
                                operators.insert(FlowStateOperatorKind::Referral, "".to_string());
                            }
                        }
                        if approval.revoke && ctx.owner == artifacts.prev_non_auto_account_id.unwrap_or_default() {
                            operators.insert(FlowStateOperatorKind::Revoke, "".to_string());
                        }
                    }
                    FLowInstStateConf {
                        operators,
                        form_conf: None,
                        approval_conf: Some(FLowInstStateApprovalConf {
                            approval_vars_collect_conf: Some(approval.vars_collect.clone()),
                            form_vars_collect: artifacts.form_state_map.get(state_id).cloned().unwrap_or_default(),
                            approval_referral_guard_custom_conf: approval.referral_guard_custom_conf.clone(),
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
        let artifacts = inst.artifacts.clone().unwrap_or_default();
        let current_state = FlowStateServ::get_item(
            &inst.current_state_id,
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
        FlowLogServ::add_operate_log(
            operate_req,
            inst,
            if current_state.state_kind == FlowStateKind::Approval {
                LogParamOp::Approval
            } else {
                LogParamOp::Form
            },
            funs,
            ctx,
        )
        .await?;
        FlowLogServ::add_operate_dynamic_log(
            operate_req,
            inst,
            if current_state.state_kind == FlowStateKind::Approval {
                LogParamOp::Approval
            } else {
                LogParamOp::Form
            },
            funs,
            ctx,
        )
        .await?;
        let mut modify_artifacts = FlowInstArtifactsModifyReq {
            add_his_operator: Some(ctx.owner.clone()),
            ..Default::default()
        };
        if let Some(all_vars) = &operate_req.all_vars {
            let mut curr_vars = artifacts.curr_vars.unwrap_or_default();
            curr_vars.extend(all_vars.clone());
            modify_artifacts.curr_vars = Some(curr_vars);
        }
        Self::modify_inst_artifacts(&inst.id, &modify_artifacts, funs, ctx).await?;
        match operate_req.operate {
            // 转办
            FlowStateOperatorKind::Referral => {
                if let Some(operator) = operate_req.operator.clone() {
                    if operator == ctx.owner {
                        return Ok(());
                    }
                    let mut modify_artifacts = FlowInstArtifactsModifyReq::default();
                    let mut curr_operators = artifacts.curr_operators.clone().unwrap_or_default();
                    let mut prohibit_guard_by_spec_account_ids = artifacts.prohibit_guard_by_spec_account_ids.unwrap_or_default();
                    curr_operators = curr_operators.into_iter().filter(|account_id| *account_id != ctx.owner.clone()).collect_vec();
                    modify_artifacts.curr_operators = Some(curr_operators);
                    prohibit_guard_by_spec_account_ids = prohibit_guard_by_spec_account_ids.into_iter().filter(|account_id| *account_id != operator.clone()).collect_vec();
                    prohibit_guard_by_spec_account_ids.push(ctx.owner.clone());
                    modify_artifacts.prohibit_guard_conf_account_ids = Some(prohibit_guard_by_spec_account_ids);

                    let mut master_account_ids = if let Some(current_referral_map) = artifacts.referral_map.get(&inst.current_state_id) {
                        modify_artifacts.remove_referral_map = Some(ctx.owner.clone());
                        current_referral_map.get(&ctx.owner).cloned().unwrap_or_default()
                    } else {
                        vec![]
                    };
                    if artifacts.curr_operators.clone().unwrap_or_default().contains(&ctx.owner) {
                        master_account_ids.push(ctx.owner.clone());
                    }
                    modify_artifacts.add_referral_map = Some((operator.clone(), master_account_ids));
                    Self::modify_inst_artifacts(&inst.id, &modify_artifacts, funs, ctx).await?;
                }
            }
            // 撤销
            FlowStateOperatorKind::Revoke => {
                let mut prev_non_auto_state_id = artifacts.prev_non_auto_state_id.unwrap_or_default();
                let target_state_id = prev_non_auto_state_id.pop();
                if let Some(target_state_id) = target_state_id {
                    Self::modify_inst_artifacts(
                        &inst.id,
                        &FlowInstArtifactsModifyReq {
                            prev_non_auto_state_id: Some(prev_non_auto_state_id),
                            state: Some(FlowInstStateKind::Revoke),
                            ..Default::default()
                        },
                        funs,
                        ctx,
                    )
                    .await?;
                    Self::transfer_spec_state(inst, &target_state_id, funs, ctx).await?;
                } else {
                    Self::abort(&inst.id, &FlowInstAbortReq { message: "".to_string() }, funs, ctx).await?;
                }
            }
            // 提交
            FlowStateOperatorKind::Submit => {
                let mut prev_non_auto_state_id = artifacts.prev_non_auto_state_id.unwrap_or_default();
                prev_non_auto_state_id.push(inst.current_state_id.clone());
                Self::modify_inst_artifacts(
                    &inst.id,
                    &FlowInstArtifactsModifyReq {
                        form_state_map: Some(operate_req.vars.clone().unwrap_or_default()),
                        prev_non_auto_state_id: Some(prev_non_auto_state_id),
                        prev_non_auto_account_id: Some(ctx.owner.clone()),
                        remove_referral_map: Some(ctx.owner.clone()),
                        ..Default::default()
                    },
                    funs,
                    ctx,
                )
                .await?;
                if let Some(next_transition) = Self::find_next_transitions(inst, &FlowInstFindNextTransitionsReq { vars: None }, funs, ctx).await?.pop() {
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
                let mut prev_non_auto_state_id = artifacts.prev_non_auto_state_id.unwrap_or_default();
                if let Some(target_state_id) = prev_non_auto_state_id.pop() {
                    Self::modify_inst_artifacts(
                        &inst.id,
                        &FlowInstArtifactsModifyReq {
                            prev_non_auto_state_id: Some(prev_non_auto_state_id),
                            state: Some(FlowInstStateKind::Back),
                            ..Default::default()
                        },
                        funs,
                        ctx,
                    )
                    .await?;
                    Self::transfer_spec_state(inst, &target_state_id, funs, ctx).await?;
                } else {
                    Self::modify_inst_artifacts(
                        &inst.id,
                        &FlowInstArtifactsModifyReq {
                            state: Some(FlowInstStateKind::Back),
                            ..Default::default()
                        },
                        funs,
                        ctx,
                    )
                    .await?;
                    Self::abort(&inst.id, &FlowInstAbortReq { message: "".to_string() }, funs, ctx).await?;
                }
            }
            // 通过
            FlowStateOperatorKind::Pass => {
                let curr_operators = artifacts.curr_operators.unwrap_or_default();
                if !curr_operators.contains(&ctx.owner) && !artifacts.referral_map.contains_key(&ctx.owner) {
                    return Err(funs.err().internal_error("flow_inst_serv", "operate", "flow inst operate failed", "500-flow-inst-operate-prohibited"));
                }
                if curr_operators.contains(&ctx.owner) {
                    Self::modify_inst_artifacts(
                        &inst.id,
                        &FlowInstArtifactsModifyReq {
                            curr_operators: Some(curr_operators.into_iter().filter(|account_id| *account_id != ctx.owner.clone()).collect_vec()),
                            add_approval_result: Some((ctx.owner.clone(), FlowApprovalResultKind::Pass)),
                            ..Default::default()
                        },
                        funs,
                        ctx,
                    )
                    .await?;
                }
                if artifacts.referral_map.contains_key(&ctx.owner) {
                    if let Some(current_referral_map) = artifacts.referral_map.get(&inst.current_state_id) {
                        let master_account_ids = current_referral_map.get(&ctx.owner).cloned().unwrap_or_default();
                        for master_account_id in master_account_ids {
                            Self::modify_inst_artifacts(
                                &inst.id,
                                &FlowInstArtifactsModifyReq {
                                    add_approval_result: Some((master_account_id.clone(), FlowApprovalResultKind::Pass)),
                                    remove_referral_map: Some(ctx.owner.clone()),
                                    ..Default::default()
                                },
                                funs,
                                ctx,
                            )
                            .await?;
                        }
                    }
                }
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
                let curr_inst = Self::get(&inst.id, funs, ctx).await?;
                if Self::check_approval_cond(&curr_inst, FlowApprovalResultKind::Pass, funs, ctx).await? {
                    if let Some(next_transition) = Self::find_next_transitions(inst, &FlowInstFindNextTransitionsReq { vars: None }, funs, ctx).await?.pop() {
                        FlowLogServ::add_operate_log(
                            operate_req,
                            inst,
                            if current_state.state_kind == FlowStateKind::Approval {
                                LogParamOp::ApprovalTransfer
                            } else {
                                LogParamOp::FormTransfer
                            },
                            funs,
                            ctx,
                        )
                        .await?;
                        let mut prev_non_auto_state_id = artifacts.prev_non_auto_state_id.unwrap_or_default();
                        prev_non_auto_state_id.push(inst.current_state_id.clone());
                        Self::modify_inst_artifacts(
                            &inst.id,
                            &FlowInstArtifactsModifyReq {
                                state: Some(FlowInstStateKind::Pass),
                                curr_operators: Some(vec![]),
                                prev_non_auto_state_id: Some(prev_non_auto_state_id),
                                prev_non_auto_account_id: Some(ctx.owner.clone()),
                                ..Default::default()
                            },
                            funs,
                            ctx,
                        )
                        .await?;
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
            }
            // 拒绝
            FlowStateOperatorKind::Overrule => {
                let curr_operators = artifacts.curr_operators.unwrap_or_default();
                if !curr_operators.contains(&ctx.owner) && !artifacts.referral_map.contains_key(&ctx.owner) {
                    return Err(funs.err().internal_error("flow_inst_serv", "operate", "flow inst operate failed", "500-flow-inst-operate-prohibited"));
                }
                if curr_operators.contains(&ctx.owner) {
                    Self::modify_inst_artifacts(
                        &inst.id,
                        &FlowInstArtifactsModifyReq {
                            curr_operators: Some(curr_operators.into_iter().filter(|account_id| *account_id != ctx.owner.clone()).collect_vec()),
                            add_approval_result: Some((ctx.owner.clone(), FlowApprovalResultKind::Overrule)),
                            ..Default::default()
                        },
                        funs,
                        ctx,
                    )
                    .await?;
                }
                if artifacts.referral_map.contains_key(&ctx.owner) {
                    if let Some(current_referral_map) = artifacts.referral_map.get(&inst.current_state_id) {
                        let master_account_ids = current_referral_map.get(&ctx.owner).cloned().unwrap_or_default();
                        for master_account_id in master_account_ids {
                            Self::modify_inst_artifacts(
                                &inst.id,
                                &FlowInstArtifactsModifyReq {
                                    add_approval_result: Some((master_account_id.clone(), FlowApprovalResultKind::Overrule)),
                                    remove_referral_map: Some(ctx.owner.clone()),
                                    ..Default::default()
                                },
                                funs,
                                ctx,
                            )
                            .await?;
                        }
                    }
                }
                let curr_inst = Self::get(&inst.id, funs, ctx).await?;
                if Self::check_approval_cond(&curr_inst, FlowApprovalResultKind::Overrule, funs, ctx).await? {
                    Self::modify_inst_artifacts(
                        &inst.id,
                        &FlowInstArtifactsModifyReq {
                            state: Some(FlowInstStateKind::Overrule),
                            curr_operators: Some(vec![]),
                            ..Default::default()
                        },
                        funs,
                        ctx,
                    )
                    .await?;
                    FlowLogServ::add_operate_log(
                        operate_req,
                        inst,
                        if current_state.state_kind == FlowStateKind::Approval {
                            LogParamOp::ApprovalTransfer
                        } else {
                            LogParamOp::FormTransfer
                        },
                        funs,
                        ctx,
                    )
                    .await?;
                    Self::abort(&inst.id, &FlowInstAbortReq { message: "".to_string() }, funs, ctx).await?;
                }
            }
        }
        FlowSearchClient::async_add_or_modify_instance_search(&inst.id, Box::new(true), funs, ctx).await?;
        Ok(())
    }

    // 判断审批条件是否满足
    async fn check_approval_cond(inst: &FlowInstDetailResp, kind: FlowApprovalResultKind, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
        let current_state = FlowStateServ::get_item(
            &inst.current_state_id,
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
        let current_state_kind_conf = current_state.kind_conf().unwrap_or_default().approval;
        let artifacts = inst.artifacts.clone().unwrap_or_default();
        let approval_total = artifacts.approval_total.unwrap_or_default().get(&inst.current_state_id).cloned().unwrap_or_default();
        let approval_result = artifacts.approval_result.get(&inst.current_state_id).cloned().unwrap_or_default();
        if let Some(current_state_kind_conf) = current_state_kind_conf {
            // 或签直接通过
            if current_state_kind_conf.multi_approval_kind == FlowStatusMultiApprovalKind::Orsign {
                return Ok(true);
            }
            // 会签但是人数为空，直接通过
            if current_state_kind_conf.multi_approval_kind == FlowStatusMultiApprovalKind::Countersign && approval_total == 0 {
                return Ok(true);
            }
            let countersign_conf = current_state_kind_conf.countersign_conf;
            let mut specified_pass_guard_conf = countersign_conf.specified_pass_guard_conf.clone().unwrap_or_default();
            let mut specified_overrule_guard_conf = countersign_conf.specified_overrule_guard_conf.clone().unwrap_or_default();
            if current_state.own_paths != inst.own_paths {
                specified_pass_guard_conf.get_local_conf(funs, ctx).await?;
                specified_overrule_guard_conf.get_local_conf(funs, ctx).await?;
            }
            // 指定人通过，则通过
            if kind == FlowApprovalResultKind::Pass
                && countersign_conf.specified_pass_guard.unwrap_or(false)
                && countersign_conf.specified_pass_guard_conf.is_some()
                && specified_pass_guard_conf.check(ctx)
            {
                return Ok(true);
            }
            // 指定人拒绝，则拒绝
            if kind == FlowApprovalResultKind::Overrule
                && countersign_conf.specified_overrule_guard.unwrap_or(false)
                && countersign_conf.specified_overrule_guard_conf.is_some()
                && specified_overrule_guard_conf.check(ctx)
            {
                return Ok(true);
            }
            match countersign_conf.kind {
                FlowStateCountersignKind::All => {
                    if kind == FlowApprovalResultKind::Overrule // 要求全数通过则出现一个拒绝，即拒绝
                        || (
                            kind == FlowApprovalResultKind::Pass
                            && approval_result.get(&FlowApprovalResultKind::Pass.to_string()).cloned().unwrap_or_default().len() >= approval_total
                            && approval_result.get(&FlowApprovalResultKind::Overrule.to_string()).cloned().unwrap_or_default().is_empty() // 要求全数通过则通过人数达到审核人数同时没有一个拒绝
                        )
                    {
                        return Ok(true);
                    }
                }
                FlowStateCountersignKind::Most => {
                    if countersign_conf.most_percent.is_none() {
                        return Ok(false);
                    }
                    let pass_total = (approval_total * countersign_conf.most_percent.unwrap_or_default() / 100) + 1; // 需满足通过的人员数量
                    let overrule_total = approval_total - pass_total + 1; // 需满足拒绝的人员数量
                    if (kind == FlowApprovalResultKind::Pass && approval_result.get(&FlowApprovalResultKind::Pass.to_string()).cloned().unwrap_or_default().len() >= pass_total) // 要求大多数通过则通过人数达到通过的比例
                        || (kind == FlowApprovalResultKind::Overrule && approval_result.get(&FlowApprovalResultKind::Overrule.to_string()).cloned().unwrap_or_default().len() >= overrule_total)
                    {
                        // 要求大多数通过则拒绝人数达到拒绝的比例
                        return Ok(true);
                    }
                }
            }
        }
        Ok(false)
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
            update_time: Set(Some(Utc::now())),
            ..Default::default()
        };

        funs.db().update_one(flow_inst, ctx).await?;

        let curr_inst = Self::get(&flow_inst_detail.id, funs, ctx).await?;
        // 删除目标节点的旧记录
        Self::modify_inst_artifacts(
            &flow_inst_detail.id,
            &FlowInstArtifactsModifyReq {
                clear_approval_result: Some(target_state_id.to_string()),
                clear_form_result: Some(target_state_id.to_string()),
                clear_referral_map: Some(target_state_id.to_string()),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;

        Self::when_enter_state(&curr_inst, target_state_id, &flow_model_version.rel_model_id, funs, ctx).await?;

        Ok(())
    }

    pub async fn add_comment(flow_inst_detail: &FlowInstDetailResp, add_comment: &FlowInstCommentReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let comment_id = TardisFuns::field.nanoid();
        let mut comments = flow_inst_detail.comments.clone().unwrap_or_default();
        comments.push(FlowInstCommentInfo {
            id: Some(comment_id.clone()),
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
        Ok(comment_id)
    }

    // 生成实例编码
    async fn gen_inst_code(funs: &TardisFunsInst) -> TardisResult<String> {
        let count = funs
            .db()
            .count(
                Query::select()
                    .columns([flow_inst::Column::Code])
                    .from(flow_inst::Entity)
                    .and_where(Expr::col(flow_inst::Column::CreateTime).gt(Utc::now().date_naive()))
                    .and_where(Expr::col(flow_inst::Column::Main).eq(false)),
            )
            .await?;
        let current_date = Utc::now();
        Ok(format!("SP{}{:0>2}{:0>2}{:0>5}", current_date.year(), current_date.month(), current_date.day(), count + 1).to_string())
    }

    // 获取需要更新的参数列表
    pub fn get_modify_vars(flow_inst_detail: &FlowInstDetailResp) -> HashMap<String, Value> {
        let mut vars_collect = HashMap::new();
        if let Some(artifacts) = &flow_inst_detail.artifacts {
            for tran in flow_inst_detail.transitions.clone().unwrap_or_default() {
                let from_state_id = tran.from_state_id.clone().unwrap_or_default();
                if let Some(form_state_vars) = artifacts.form_state_map.get(&from_state_id) {
                    for (key, value) in form_state_vars {
                        *vars_collect.entry(key.clone()).or_insert(json!({})) = value.clone();
                    }
                }
            }
        };

        vars_collect
    }

    pub async fn get_rel_transitions(rel_business_obj_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<FlowModelRelTransitionExt>> {
        let mut result = vec![];
        let rel_version_ids = Self::find_detail_items(
            &FlowInstFilterReq {
                rel_business_obj_ids: Some(vec![rel_business_obj_id.to_string()]),
                main: Some(false),
                finish: Some(false),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
        .into_iter()
        .map(|inst| inst.rel_flow_version_id)
        .collect_vec();
        for rel_version_id in rel_version_ids {
            if let Some(rel_model_id) = FlowModelVersionServ::find_one_item(
                &FlowModelVersionFilterReq {
                    basic: RbumBasicFilterReq {
                        ids: Some(vec![rel_version_id]),
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
            .map(|version| version.rel_model_id)
            {
                let rel_transition_ext = FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowModelTransition, &rel_model_id, None, None, funs, ctx)
                    .await?
                    .pop()
                    .map(|rel| TardisFuns::json.str_to_obj::<FlowModelRelTransitionExt>(&rel.ext).unwrap_or_default());
                if let Some(ext) = rel_transition_ext {
                    result.push(ext);
                }
            }
        }
        Ok(result)
    }

    pub async fn sync_status(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let mut page_num = 1;
        let page_size = 50;
        loop {
            let mut finish = true;
            let search_tags = FlowSearchClient::get_tag_search_map().values().cloned().collect_vec();
            for (search_tag, kind) in search_tags {
                if let Some(search_result) = FlowSearchClient::search(
                    &SearchItemSearchReq {
                        tag: search_tag,
                        ctx: SearchItemSearchCtxReq { ..Default::default() },
                        query: SearchItemQueryReq {
                            kinds: Some(vec![kind]),
                            ..Default::default()
                        },
                        adv_by_or: None,
                        adv_query: None,
                        sort: Some(vec![SearchItemSearchSortReq {
                            field: "create_time".to_string(),
                            order: SearchItemSearchSortKind::Desc,
                        }]),
                        page: SearchItemSearchPageReq {
                            number: page_num,
                            size: page_size,
                            fetch_total: false,
                        },
                    },
                    funs,
                    ctx,
                )
                .await?
                {
                    finish = false;
                    let inst_ids = search_result
                        .records
                        .iter()
                        .map(|record| record.ext.get("inst_id").unwrap_or(&json!("")).as_str().map(|s| s.to_string()).unwrap_or_default())
                        .filter(|record| !record.is_empty())
                        .collect_vec();
                    if !inst_ids.is_empty() {
                        let flow_insts = Self::find_detail(inst_ids, None, None, funs, ctx).await?;
                        join_all(
                            flow_insts
                                .iter()
                                .map(|flow_inst| async {
                                    if let Some(current_state_name) = &flow_inst.current_state_name {
                                        let ctx = TardisContext {
                                            own_paths: flow_inst.own_paths.clone(),
                                            ..Default::default()
                                        };
                                        FlowSearchClient::modify_business_obj_search(
                                            &flow_inst.rel_business_obj_id,
                                            &flow_inst.tag,
                                            json!({
                                                "status": current_state_name,
                                            }),
                                            funs,
                                            &ctx,
                                        )
                                        .await
                                    } else {
                                        Ok(())
                                    }
                                })
                                .collect_vec(),
                        )
                        .await;
                    }
                }
            }
            if finish {
                break;
            }
            page_num += 1;
        }

        // loop {
        //     let flow_insts = funs
        //         .db()
        //         .paginate_dtos::<FlowInstResult>(
        //             Query::select()
        //                 .columns([
        //                     (flow_inst::Entity, flow_inst::Column::Id),
        //                     (flow_inst::Entity, flow_inst::Column::RelBusinessObjId),
        //                     (flow_inst::Entity, flow_inst::Column::Tag),
        //                     (flow_inst::Entity, flow_inst::Column::OwnPaths),
        //                 ])
        //                 .expr_as(Expr::col((RBUM_ITEM_TABLE.clone(), NAME_FIELD.clone())).if_null(""), Alias::new("current_state_name"))
        //                 .from(flow_inst::Entity)
        //                 .left_join(
        //                     RBUM_ITEM_TABLE.clone(),
        //                     Cond::all()
        //                         .add(Expr::col((RBUM_ITEM_TABLE.clone(), ID_FIELD.clone())).equals((flow_inst::Entity, flow_inst::Column::CurrentStateId)))
        //                         .add(Expr::col((RBUM_ITEM_TABLE.clone(), REL_KIND_ID_FIELD.clone())).eq(FlowStateServ::get_rbum_kind_id().unwrap_or_default()))
        //                         .add(Expr::col((RBUM_ITEM_TABLE.clone(), REL_DOMAIN_ID_FIELD.clone())).eq(FlowStateServ::get_rbum_domain_id().unwrap_or_default())),
        //                 )
        //                 .and_where(Expr::col(flow_inst::Column::Main).eq(true))
        //                 .order_by((flow_inst::Entity, flow_inst::Column::CreateTime), Order::Desc),
        //             page_num,
        //             page_size,
        //         )
        //         .await?
        //         .0;
        //     if flow_insts.is_empty() {
        //         break;
        //     }
        //     join_all(
        //         flow_insts
        //             .iter()
        //             .map(|flow_inst| async {
        //                 if let Some(current_state_name) = &flow_inst.current_state_name {
        //                     let ctx = TardisContext {
        //                         own_paths: flow_inst.own_paths.clone(),
        //                         ..Default::default()
        //                     };
        //                     FlowSearchClient::modify_business_obj_search(
        //                         &flow_inst.rel_business_obj_id,
        //                         &flow_inst.tag,
        //                         json!({
        //                             "status": current_state_name,
        //                         }),
        //                         funs,
        //                         &ctx,
        //                     )
        //                     .await
        //                 } else {
        //                     Ok(())
        //                 }
        //             })
        //             .collect_vec(),
        //     )
        //     .await;

        //     page_num += 1;
        // }

        //
        Ok(())
    }
}
