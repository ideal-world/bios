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
    basic::{dto::TardisContext, field::TrimString, result::TardisResult},
    chrono::{DateTime, Datelike, Utc},
    db::sea_orm::{
        self,
        sea_query::{Alias, Cond, Expr, Query, SelectStatement},
        JoinType, Order, Set,
    },
    futures_util::future::join_all,
    log::{error, warn},
    serde_json::Value,
    web::web_resp::TardisPage,
    TardisFuns, TardisFunsInst,
};

use crate::{
    domain::{flow_inst, flow_model_version, flow_state},
    dto::{
        flow_cond_dto::BasicQueryCondInfo,
        flow_external_dto::{FlowExternalCallbackOp, FlowExternalParams},
        flow_inst_dto::{
            FLowInstStateApprovalConf, FLowInstStateConf, FLowInstStateFormConf, FlowApprovalResultKind, FlowInstAbortReq, FlowInstArtifacts, FlowInstArtifactsModifyReq,
            FlowInstBatchBindReq, FlowInstBatchBindResp, FlowInstCommentInfo, FlowInstCommentReq, FlowInstDetailInSearch, FlowInstDetailResp, FlowInstFilterReq,
            FlowInstFindNextTransitionResp, FlowInstFindNextTransitionsReq, FlowInstFindStateAndTransitionsReq, FlowInstFindStateAndTransitionsResp, FlowInstFindTransitionsResp,
            FlowInstOperateReq, FlowInstRelChildObj, FlowInstStartReq, FlowInstStateKind, FlowInstSummaryResp, FlowInstSummaryResult, FlowInstTransferReq, FlowInstTransferResp,
            FlowInstTransitionInfo, FlowOperationContext, ModifyObjSearchExtReq,
        },
        flow_model_dto::{FlowModelAggResp, FlowModelDetailResp, FlowModelFilterReq, FlowModelRelTransitionExt, FlowModelRelTransitionKind},
        flow_model_version_dto::{FlowModelVersionDetailResp, FlowModelVersionFilterReq},
        flow_state_dto::{
            FLowStateKindConf, FlowStateCountersignKind, FlowStateDetailResp, FlowStateFilterReq, FlowStateKind, FlowStateOperatorKind, FlowStateRelModelExt,
            FlowStatusAutoStrategyKind, FlowStatusMultiApprovalKind, FlowSysStateKind,
        },
        flow_transition_dto::{FlowTransitionDetailResp, FlowTransitionFilterReq},
        flow_var_dto::FillType,
    },
    flow_constants,
    helper::loop_check_helper,
    serv::{flow_model_serv::FlowModelServ, flow_state_serv::FlowStateServ},
};

use super::{
    clients::{
        log_client::LogParamOp,
        search_client::{FlowSearchClient, FlowSearchTaskKind},
    },
    flow_cache_serv::FlowCacheServ,
    flow_config_serv::FlowConfigServ,
    flow_event_serv::FlowEventServ,
    flow_external_serv::FlowExternalServ,
    flow_log_serv::FlowLogServ,
    flow_model_version_serv::FlowModelVersionServ,
    flow_rel_serv::{FlowRelKind, FlowRelServ},
    flow_transition_serv::FlowTransitionServ,
};

pub struct FlowInstServ;

impl FlowInstServ {
    pub async fn try_start(start_req: &FlowInstStartReq, current_state_name: Option<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let mut create_vars = if start_req.transition_id.is_some() {
            Self::get_new_vars(&start_req.tag, start_req.rel_business_obj_id.clone(), funs, ctx).await?
        } else {
            HashMap::default()
        };
        if let Some(check_vars) = &start_req.check_vars {
            create_vars.extend(check_vars.clone());
            create_vars.insert("changes".to_string(), json!(check_vars.keys().collect_vec()));
        }
        if let Some(rel_model) = Self::find_rel_model(start_req.transition_id.clone(), &start_req.tag, &create_vars, funs, ctx).await? {
            if start_req.transition_id.is_none() {
                let inst_id = Self::start_main_flow(start_req, &rel_model, current_state_name, funs, ctx).await?;
                if start_req.rel_child_objs.is_some() {
                    Self::modify_inst_artifacts(
                        &inst_id,
                        &FlowInstArtifactsModifyReq {
                            rel_child_objs: start_req.rel_child_objs.clone(),
                            operator_map: start_req.operator_map.clone(),
                            rel_transition_id: start_req.rel_transition_id.clone(),
                            rel_model_version_id: Some(rel_model.current_version_id.clone()),
                            ..Default::default()
                        },
                        funs,
                        ctx,
                    )
                    .await?;
                    let main_inst = Self::get(&inst_id, funs, ctx).await?;
                    // 存入rel_state，用来标记关联的业务项被选中
                    for rel_child_obj in start_req.rel_child_objs.clone().unwrap_or_default() {
                        let modify_serach_ext = TardisFuns::json.obj_to_string(&ModifyObjSearchExtReq {
                            tag: rel_child_obj.tag.clone(),
                            status: Some(flow_constants::SPECIFED_APPROVING_STATE_NAME.to_string()),
                            rel_state: Some(main_inst.artifacts.clone().unwrap_or_default().state.unwrap_or_default().to_string()),
                            rel_transition_state_name: Some("".to_string()),
                        })?;
                        FlowSearchClient::add_search_task(&FlowSearchTaskKind::ModifyBusinessObj, &rel_child_obj.obj_id, &modify_serach_ext, funs, ctx).await?;
                        // 更新业务主流程的artifact的状态为审批中
                        if let Some(main_child_inst_id) = Self::find_ids(
                            &FlowInstFilterReq {
                                rel_business_obj_ids: Some(vec![rel_child_obj.obj_id.clone()]),
                                main: Some(true),
                                ..Default::default()
                            },
                            funs,
                            ctx,
                        )
                        .await?
                        .pop()
                        {
                            Self::modify_inst_artifacts(
                                &main_child_inst_id,
                                &FlowInstArtifactsModifyReq {
                                    state: Some(FlowInstStateKind::Approval),
                                    ..Default::default()
                                },
                                funs,
                                ctx,
                            )
                            .await?;
                        }
                    }
                }
                Ok(inst_id)
            } else {
                let inst_id = Self::start_secondary_flow(start_req, false, &rel_model, None, funs, ctx).await?;
                let inst = Self::get(&inst_id, funs, ctx).await?;
                FlowSearchClient::add_or_modify_instance_search(&inst_id, Box::new(false), funs, ctx).await?;
                let modify_serach_ext = TardisFuns::json.obj_to_string(&ModifyObjSearchExtReq {
                    tag: start_req.tag.clone(),
                    status: Some(flow_constants::SPECIFED_APPROVING_STATE_NAME.to_string()),
                    rel_state: inst.artifacts.unwrap_or_default().state.map(|s| s.to_string()),
                    rel_transition_state_name: Some(inst.current_state_name.unwrap_or_default()),
                })?;
                FlowSearchClient::add_search_task(&FlowSearchTaskKind::ModifyBusinessObj, &start_req.rel_business_obj_id, &modify_serach_ext, funs, ctx).await?;
                Ok(inst_id)
            }
        } else {
            Ok("".to_string())
        }
    }
    pub async fn start(start_req: &FlowInstStartReq, current_state_name: Option<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let mut create_vars = if start_req.transition_id.is_some() {
            Self::get_new_vars(&start_req.tag, start_req.rel_business_obj_id.clone(), funs, ctx).await?
        } else {
            start_req.create_vars.clone().unwrap_or_default()
        };
        if let Some(check_vars) = &start_req.check_vars {
            create_vars.extend(check_vars.clone());
            create_vars.insert("changes".to_string(), json!(check_vars.keys().collect_vec()));
        }
        let rel_model = Self::find_rel_model(start_req.transition_id.clone(), &start_req.tag, &create_vars, funs, ctx)
            .await?
            .ok_or_else(|| funs.err().not_found("flow_inst_serv", "start", "model not found", "404-flow-model-not-found"))?;
        if start_req.transition_id.is_none() {
            let inst_id = Self::start_main_flow(start_req, &rel_model, current_state_name, funs, ctx).await?;
            if start_req.rel_child_objs.is_some() {
                let rel_child_model = Self::find_rel_model(start_req.rel_transition_id.clone(), &start_req.tag, &create_vars, funs, ctx)
                    .await?
                    .ok_or_else(|| funs.err().not_found("flow_inst_serv", "start", "approve model not found", "404-flow-model-not-found"))?;
                Self::modify_inst_artifacts(
                    &inst_id,
                    &FlowInstArtifactsModifyReq {
                        rel_child_objs: start_req.rel_child_objs.clone(),
                        operator_map: start_req.operator_map.clone(),
                        rel_transition_id: start_req.rel_transition_id.clone(),
                        rel_model_version_id: Some(rel_child_model.current_version_id.clone()),
                        ..Default::default()
                    },
                    funs,
                    ctx,
                )
                .await?;
                let main_inst = Self::get(&inst_id, funs, ctx).await?;
                // 存入rel_state，用来标记关联的业务项被选中
                for rel_child_obj in start_req.rel_child_objs.clone().unwrap_or_default() {
                    let modify_serach_ext = TardisFuns::json.obj_to_string(&ModifyObjSearchExtReq {
                        tag: rel_child_obj.tag.clone(),
                        status: Some(flow_constants::SPECIFED_APPROVING_STATE_NAME.to_string()),
                        rel_state: Some(main_inst.artifacts.clone().unwrap_or_default().state.unwrap_or_default().to_string()),
                        rel_transition_state_name: Some("".to_string()),
                    })?;
                    FlowSearchClient::add_search_task(&FlowSearchTaskKind::ModifyBusinessObj, &rel_child_obj.obj_id, &modify_serach_ext, funs, ctx).await?;
                    // 更新业务主流程的artifact的状态为审批中
                    if let Some(main_child_inst_id) = Self::find_ids(
                        &FlowInstFilterReq {
                            rel_business_obj_ids: Some(vec![rel_child_obj.obj_id.clone()]),
                            main: Some(true),
                            ..Default::default()
                        },
                        funs,
                        ctx,
                    )
                    .await?
                    .pop()
                    {
                        Self::modify_inst_artifacts(
                            &main_child_inst_id,
                            &FlowInstArtifactsModifyReq {
                                state: Some(FlowInstStateKind::Approval),
                                ..Default::default()
                            },
                            funs,
                            ctx,
                        )
                        .await?;
                    }
                }
            }
            Ok(inst_id)
        } else {
            let inst_id = Self::start_secondary_flow(start_req, false, &rel_model, None, funs, ctx).await?;
            FlowSearchClient::add_or_modify_instance_search(&inst_id, Box::new(false), funs, ctx).await?;
            Ok(inst_id)
        }
    }

    ///  启动子工作流
    async fn start_child_flow(root_inst_id: &str, rel_child_objs: &[FlowInstRelChildObj], funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let root_inst = Self::get(root_inst_id, funs, ctx).await?;
        let rel_child_model = FlowModelServ::get_item(
            &root_inst.rel_flow_model_id.clone(),
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
        .await?;
        for rel_child_obj in rel_child_objs {
            // 终止未完成的审批流实例
            for unfinished_inst_id in Self::find_ids(
                &FlowInstFilterReq {
                    rel_business_obj_ids: Some(vec![rel_child_obj.obj_id.clone()]),
                    finish: Some(false),
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?
            {
                Self::abort(&unfinished_inst_id, &FlowInstAbortReq { message: "".to_string() }, funs, ctx).await?;
            }
            Self::start_secondary_flow(
                &FlowInstStartReq {
                    rel_business_obj_id: rel_child_obj.obj_id.clone(),
                    tag: rel_child_obj.tag.clone(),
                    transition_id: root_inst.rel_transition_id.clone(),
                    operator_map: root_inst.artifacts.clone().unwrap_or_default().operator_map.clone(),
                    rel_inst_id: Some(root_inst_id.to_string()),
                    ..Default::default()
                },
                true,
                &rel_child_model,
                None,
                funs,
                ctx,
            )
            .await?;
        }
        Ok(())
    }

    async fn start_main_flow(
        start_req: &FlowInstStartReq,
        flow_model: &FlowModelDetailResp,
        current_state_name: Option<String>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<String> {
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
        let mut current_vars = start_req.create_vars.clone().unwrap_or_default();
        if let Some(check_vars) = &start_req.check_vars {
            current_vars.extend(check_vars.clone());
        }
        let flow_inst: flow_inst::ActiveModel = flow_inst::ActiveModel {
            id: Set(inst_id.clone()),
            code: Set(Some("".to_string())),
            tag: Set(Some(start_req.tag.clone())),
            rel_flow_version_id: Set(flow_model.current_version_id.clone()),
            rel_business_obj_id: Set(start_req.rel_business_obj_id.clone()),

            current_state_id: Set(current_state_id.clone()),

            create_vars: Set(Some(TardisFuns::json.obj_to_json(&start_req.create_vars).unwrap_or(json!("")))),
            current_vars: Set(Some(TardisFuns::json.obj_to_json(&current_vars).unwrap_or(json!("")))),
            create_ctx: Set(FlowOperationContext::from_ctx(ctx)),

            own_paths: Set(ctx.own_paths.clone()),
            main: Set(true),
            update_time: Set(Some(Utc::now())),
            data_source: Set(flow_model.data_source.clone().unwrap_or_default()),
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

    async fn start_secondary_flow(
        start_req: &FlowInstStartReq,
        child: bool,
        flow_model: &FlowModelDetailResp,
        flow_version_id: Option<String>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<String> {
        let current_version_id = if let Some(flow_version_id) = flow_version_id {
            flow_version_id
        } else {
            flow_model.current_version_id.clone()
        };
        let rel_transition = start_req.transition_id.clone().unwrap_or_default();
        let rel_transition_ext = flow_model
            .rel_transitions()
            .unwrap_or_default()
            .into_iter()
            .find(|tran| tran.id == rel_transition)
            .ok_or_else(|| funs.err().not_found("flow_inst_serv", "start_secondary_flow", "model is not exist", "404-flow-model-not-found"))?
            .clone();
        if !child && Self::start_dry_run(start_req, &current_version_id, funs, ctx).await?.state_kind == FlowStateKind::Finish {
            let form_map = HashMap::from([(flow_model.init_state_id.clone(), start_req.vars.clone().unwrap_or_default())]);
            Self::finish_approve_flow(
                rel_transition_ext,
                &start_req.tag,
                &start_req.rel_business_obj_id,
                None, // 此时还未创建出父审批流，所以传空
                &FlowInstArtifacts {
                    form_state_map: form_map,
                    ..Default::default()
                },
                &[flow_model.init_state_id.clone()],
                funs,
                ctx,
            )
            .await?;
            return Err(funs.err().internal_error("flow_inst", "start_secondary_flow", "The process is automatically terminated", "500-flow-inst-auto-finish"));
        }
        let inst_id = TardisFuns::field.nanoid();
        // if !Self::find_ids(
        //     &FlowInstFilterReq {
        //         rel_business_obj_ids: Some(vec![start_req.rel_business_obj_id.to_string()]),
        //         flow_model_id: Some(flow_model.id.clone()),
        //         finish: Some(false),
        //         ..Default::default()
        //     },
        //     funs,
        //     ctx,
        // )
        // .await?
        // .is_empty()
        // {
        //     return Err(funs.err().internal_error("flow_inst_serv", "start", "The same instance exist", "500-flow-inst-exist"));
        // }
        let create_vars = Self::get_new_vars(&start_req.tag, start_req.rel_business_obj_id.to_string(), funs, ctx).await?;
        let mut current_vars = create_vars.clone();
        if let Some(check_vars) = &start_req.check_vars {
            current_vars.extend(check_vars.clone());
        }
        let flow_inst: flow_inst::ActiveModel = flow_inst::ActiveModel {
            id: Set(inst_id.clone()),
            code: Set(Some(if !child { Self::gen_inst_code(funs).await? } else { "".to_string() })),
            tag: Set(Some(start_req.tag.clone())),
            rel_flow_version_id: Set(current_version_id.clone()),
            rel_business_obj_id: Set(start_req.rel_business_obj_id.clone()),
            rel_transition_id: Set(start_req.transition_id.clone()),

            current_state_id: Set(flow_model.init_state_id.clone()),

            create_vars: Set(Some(TardisFuns::json.obj_to_json(&create_vars).unwrap_or(json!("")))),
            current_vars: Set(Some(TardisFuns::json.obj_to_json(&current_vars).unwrap_or(json!("")))),

            create_ctx: Set(FlowOperationContext::from_ctx(ctx)),

            own_paths: Set(ctx.own_paths.clone()),
            main: Set(false),
            rel_inst_id: Set(start_req.rel_inst_id.clone()),
            data_source: Set(flow_model.data_source.clone().unwrap_or_default()),
            ..Default::default()
        };
        funs.db().insert_one(flow_inst, ctx).await?;
        Self::modify_inst_artifacts(
            &inst_id,
            &FlowInstArtifactsModifyReq {
                curr_vars: start_req.create_vars.clone(),
                add_his_operator: Some(ctx.owner.clone()),
                form_state_map: Some(start_req.vars.clone().unwrap_or_default()),
                operator_map: start_req.operator_map.clone(),
                rel_transition_id: start_req.rel_transition_id.clone(),
                rel_child_objs: start_req.rel_child_objs.clone(),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        let inst = Self::get(&inst_id, funs, ctx).await?;
        FlowLogServ::add_start_log_async_task(start_req, &inst, &create_vars, funs, ctx).await?;
        FlowLogServ::add_start_dynamic_log_async_task(start_req, &inst, &create_vars, funs, ctx).await?;
        FlowLogServ::add_start_business_log_async_task(start_req, &inst, &create_vars, funs, ctx).await?;

        Self::when_enter_state(&inst, &flow_model.init_state_id, &flow_model.id, funs, ctx).await?;
        Self::do_request_webhook(
            None,
            flow_model.transitions().iter().filter(|model_transition| model_transition.to_flow_state_id == flow_model.init_state_id).collect_vec().pop(),
        )
        .await?;
        // 自动流转
        Self::auto_transfer(&inst.id, loop_check_helper::InstancesTransition::default(), funs, ctx).await?;
        // 更新业务的关联审批流节点名
        FlowSearchClient::refresh_business_obj_search(&start_req.rel_business_obj_id, &start_req.tag, funs, ctx).await?;

        Ok(inst_id)
    }

    // 创建实例（干跑） 返回终止的状态ID
    async fn start_dry_run(start_req: &FlowInstStartReq, rel_flow_version_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<FlowStateDetailResp> {
        let mut create_vars = if start_req.transition_id.is_some() {
            Self::get_new_vars(&start_req.tag, start_req.rel_business_obj_id.clone(), funs, ctx).await?
        } else {
            HashMap::default()
        };
        if let Some(check_vars) = &start_req.check_vars {
            create_vars.extend(check_vars.clone());
        }
        let model_version = FlowModelVersionServ::find_one_item(
            &FlowModelVersionFilterReq {
                basic: RbumBasicFilterReq {
                    ids: Some(vec![rel_flow_version_id.to_string()]),
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
        .ok_or_else(|| funs.err().not_found("flow_inst_serv", "start_dry_run", "model is not exist", "404-flow-model-not-found"))?;
        let mut target_state_id = model_version.init_state_id;
        loop {
            let transition_ids = FlowTransitionServ::find_detail_items(
                &FlowTransitionFilterReq {
                    flow_version_id: Some(model_version.id.clone()),
                    specified_state_ids: Some(vec![target_state_id.clone()]),
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?
            .into_iter()
            .map(|tran| tran.id)
            .collect_vec();
            match Self::find_auto_transition(transition_ids, &create_vars, funs, ctx).await {
                Ok(Some(tran)) => {
                    target_state_id = tran.to_flow_state_id;
                }
                Ok(None) => {
                    break;
                }
                Err(e) => {
                    if e.code == *"404-flow-flow_inst-find_auto_transition" {
                        break;
                    } else {
                        return Err(e);
                    }
                }
            }
        }
        FlowStateServ::get_item(
            &target_state_id,
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
        .await
    }

    // 获取实例所适配的模板
    pub async fn find_rel_model(
        transition_id: Option<String>,
        tag: &str,
        vars: &HashMap<String, Value>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Option<FlowModelDetailResp>> {
        if let Some(transition_id) = &transition_id {
            FlowModelServ::get_model_id_by_own_paths_and_transition_id(tag, transition_id, vars, funs, ctx).await
        } else {
            FlowModelServ::get_model_id_by_own_paths_and_rel_template_id(tag, None, funs, ctx).await
        }
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
            let create_vars = Self::get_new_vars(&batch_bind_req.tag, rel_business_obj.rel_business_obj_id.clone().unwrap_or_default(), funs, ctx).await?;
            let flow_model = Self::find_rel_model(batch_bind_req.transition_id.clone(), &batch_bind_req.tag, &create_vars, funs, ctx)
                .await?
                .ok_or_else(|| funs.err().not_found("flow_inst_serv", "batch_bind", "model not found", "404-flow-model-not-found"))?;
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
                    data_source: Set(flow_model.data_source.clone().unwrap_or_default()),
                    ..Default::default()
                };
                funs.db().insert_one(flow_inst, &current_ctx).await?;

                let flow_inst = flow_inst::ActiveModel {
                    id: Set(id.clone()),
                    create_vars: Set(Some(TardisFuns::json.obj_to_json(&create_vars).unwrap_or(json!("")))),
                    current_vars: Set(Some(TardisFuns::json.obj_to_json(&create_vars).unwrap_or(json!("")))),
                    update_time: Set(Some(Utc::now())),
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
                (flow_inst::Entity, flow_inst::Column::DataSource),
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
                flow_state::Entity,
                Expr::col((flow_state::Entity, ID_FIELD.clone())).equals((flow_inst::Entity, flow_inst::Column::CurrentStateId)),
            )
            .left_join(
                rel_model_table.clone(),
                Cond::all()
                    .add(Expr::col((rel_model_table.clone(), Alias::new("from_rbum_id"))).equals((flow_model_version_table.clone(), flow_model_version::Column::RelModelId)))
                    .add(Expr::col((rel_model_table.clone(), Alias::new("to_rbum_item_id"))).equals((flow_inst::Entity, flow_inst::Column::RelTransitionId)))
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
        if let Some(tags) = &filter.tags {
            query.and_where(Expr::col((flow_inst::Entity, flow_inst::Column::Tag)).is_in(tags));
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
        if let Some(finish_abort) = filter.finish_abort {
            query.and_where(Expr::col((flow_inst::Entity, flow_inst::Column::FinishAbort)).eq(finish_abort));
        }
        if let Some(current_state_id) = &filter.current_state_id {
            query.and_where(Expr::col((flow_inst::Entity, flow_inst::Column::CurrentStateId)).eq(current_state_id));
        }
        if let Some(current_state_sys_kind) = &filter.current_state_sys_kind {
            query.and_where(Expr::col((flow_state::Entity, flow_state::Column::SysState)).eq(current_state_sys_kind.clone()));
        }
        if let Some(rel_business_obj_ids) = &filter.rel_business_obj_ids {
            query.and_where(Expr::col((flow_inst::Entity, flow_inst::Column::RelBusinessObjId)).is_in(rel_business_obj_ids));
        }
        if let Some(rel_inst_ids) = &filter.rel_inst_ids {
            query.and_where(Expr::col((flow_inst::Entity, flow_inst::Column::RelInstId)).is_in(rel_inst_ids));
        }
        if let Some(data_source) = &filter.data_source {
            query.and_where(Expr::col((flow_inst::Entity, flow_inst::Column::DataSource)).eq(data_source));
        }

        if let Some(create_time_start) = &filter.create_time_start {
            query.and_where(Expr::col((flow_inst::Entity, flow_inst::Column::CreateTime)).gte(create_time_start.to_string()));
        }
        if let Some(create_time_end) = &filter.create_time_end {
            query.and_where(Expr::col((flow_inst::Entity, flow_inst::Column::CreateTime)).lte(create_time_end.to_string()));
        }
        if let Some(update_time_start) = &filter.update_time_start {
            query.and_where(Expr::col((flow_inst::Entity, flow_inst::Column::UpdateTime)).gte(update_time_start.to_string()));
        }
        if let Some(update_time_end) = &filter.update_time_end {
            query.and_where(Expr::col((flow_inst::Entity, flow_inst::Column::UpdateTime)).lte(update_time_end.to_string()));
        }

        Ok(())
    }

    pub async fn find_ids(filter: &FlowInstFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<String>> {
        let mut query = Query::select();
        Self::package_ext_query(&mut query, filter, funs, ctx).await?;
        Ok(funs.db().find_dtos::<FlowInstSummaryResult>(&query).await?.into_iter().map(|inst| inst.id).collect_vec())
    }

    pub async fn find_items(filter: &FlowInstFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<FlowInstSummaryResult>> {
        let mut query = Query::select();
        Self::package_ext_query(&mut query, filter, funs, ctx).await?;
        funs.db().find_dtos::<FlowInstSummaryResult>(&query).await
    }

    pub async fn find_detail_items(filter: &FlowInstFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<FlowInstDetailResp>> {
        let flow_inst_ids = Self::find_ids(filter, funs, ctx).await?;
        if !flow_inst_ids.is_empty() {
            Self::find_detail(flow_inst_ids, None, None, funs, ctx).await
        } else {
            Ok(vec![])
        }
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

    #[async_recursion]
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

        Self::abort_child_inst(flow_inst_id, funs, ctx).await?;
        let flow_inst_detail = Self::get(flow_inst_id, funs, ctx).await?;
        if !flow_inst_detail.main {
            FlowLogServ::add_finish_business_log_async_task(&flow_inst_detail, Some(abort_req.message.to_string()), funs, ctx).await?;
            FlowLogServ::add_finish_log_async_task(&flow_inst_detail, Some(abort_req.message.to_string()), funs, ctx).await?;
            if flow_inst_detail.rel_inst_id.as_ref().is_none_or(|id| id.is_empty()) {
                FlowSearchClient::refresh_business_obj_search(&flow_inst_detail.rel_business_obj_id, &flow_inst_detail.tag, funs, ctx).await?;
            }
            FlowSearchClient::add_search_task(&FlowSearchTaskKind::ModifyInstance, &flow_inst_detail.id, "", funs, ctx).await?;
            // 更新业务主流程的artifact的状态为审批拒绝
            if let Some(main_inst_id) = Self::find_ids(
                &FlowInstFilterReq {
                    rel_business_obj_ids: Some(vec![flow_inst_detail.rel_business_obj_id.clone()]),
                    main: Some(true),
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?
            .pop()
            {
                Self::modify_inst_artifacts(
                    &main_inst_id,
                    &FlowInstArtifactsModifyReq {
                        state: Some(FlowInstStateKind::Overrule),
                        ..Default::default()
                    },
                    funs,
                    ctx,
                )
                .await?;
            }
            // 流程结束时，更新对应的主审批流的search状态
            if let Some(main_inst) = Self::find_detail_items(
                &FlowInstFilterReq {
                    rel_business_obj_ids: Some(vec![flow_inst_detail.rel_business_obj_id.clone()]),
                    main: Some(true),
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?
            .pop()
            {
                let modify_serach_ext = TardisFuns::json.obj_to_string(&ModifyObjSearchExtReq {
                    tag: main_inst.tag.to_string(),
                    status: main_inst.current_state_name.clone(),
                    rel_state: Some("".to_string()),
                    rel_transition_state_name: Some("".to_string()),
                })?;
                FlowSearchClient::add_search_task(&FlowSearchTaskKind::ModifyBusinessObj, &flow_inst_detail.rel_business_obj_id, &modify_serach_ext, funs, ctx).await?;
            }
        }
        // 携带子审批流的审批流
        if !flow_inst_detail.main && flow_inst_detail.artifacts.as_ref().is_some_and(|artifacts| artifacts.rel_child_objs.is_some()) {
            // 主审批流中止时，流转对应业务的状态流为结束
            if flow_inst_detail.rel_inst_id.as_ref().is_none_or(|id| id.is_empty()) {
                if let Some(main_inst) = Self::find_detail_items(
                    &FlowInstFilterReq {
                        rel_business_obj_ids: Some(vec![flow_inst_detail.rel_business_obj_id.clone()]),
                        main: Some(true),
                        ..Default::default()
                    },
                    funs,
                    ctx,
                )
                .await?
                .pop()
                {
                    let next_trans = Self::find_next_transitions(&main_inst, &FlowInstFindNextTransitionsReq { vars: None }, funs, ctx).await?;
                    for next_tran in next_trans {
                        let next_state = FlowStateServ::get_item(
                            &next_tran.next_flow_state_id,
                            &FlowStateFilterReq {
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
                        .await?;
                        if next_state.sys_state == FlowSysStateKind::Finish {
                            Self::transfer(
                                &main_inst,
                                &FlowInstTransferReq {
                                    flow_transition_id: next_tran.next_flow_transition_id,
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
                            break;
                        }
                    }
                }
            }
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
            pub rel_transition_id: Option<String>,
            pub rel_inst_id: Option<String>,

            pub data_source: Option<String>,
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
                (flow_inst::Entity, flow_inst::Column::RelTransitionId),
                (flow_inst::Entity, flow_inst::Column::RelInstId),
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
                (flow_inst::Entity, flow_inst::Column::DataSource),
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
                    .add(Expr::col((rel_model_table.clone(), Alias::new("to_rbum_item_id"))).equals((flow_inst::Entity, flow_inst::Column::RelTransitionId)))
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
                    rel_transition_id: inst.rel_transition_id,
                    rel_inst_id: inst.rel_inst_id,
                    tag: inst.tag,
                    main: inst.main,
                    data_source: inst.data_source.clone().unwrap_or_default(),
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
                    curr_vars.extend(Self::get_modify_vars(
                        artifacts,
                        &inst_detail.transitions.clone().unwrap_or_default().into_iter().map(|tran| tran.from_state_id.unwrap_or_default()).collect_vec(),
                    ));

                    artifacts.curr_vars = Some(curr_vars);
                }
                inst_detail.artifacts = artifacts;
                inst_detail
            })
            .collect_vec())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn paginate(
        flow_version_id: Option<String>,
        tags: Option<Vec<String>>,
        finish: Option<bool>,
        finish_abort: Option<bool>,
        main: Option<bool>,
        current_state_id: Option<String>,
        current_state_sys_kind: Option<FlowSysStateKind>,
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
                tags,
                finish,
                finish_abort,
                main,
                current_state_id,
                current_state_sys_kind,
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
        // 若当前数据项存在未结束的审批流，则清空其中的transitions
        let unfinished_approve_flow_obj_ids = Self::find_detail_items(
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
        .map(|inst| inst.rel_business_obj_id)
        .unique()
        .collect_vec();
        let state_and_next_transitions = join_all(
            flow_insts
                .iter()
                .map(|flow_inst| async {
                    if let (Some(req), Some(rel_flow_versions)) = (
                        find_req.iter().find(|req| req.flow_inst_id == flow_inst.id),
                        rel_flow_version_map.get(&flow_inst.tag).cloned(),
                    ) {
                        Self::do_find_next_transitions(flow_inst, None, &req.vars, false, funs, ctx).await.ok().map(|resp| FlowInstFindStateAndTransitionsResp {
                            flow_inst_id: resp.flow_inst_id,
                            rel_business_obj_id: flow_inst.rel_business_obj_id.clone(),
                            current_flow_state_name: resp.current_flow_state_name,
                            current_flow_state_sys_kind: resp.current_flow_state_sys_kind,
                            current_flow_state_color: resp.current_flow_state_color,
                            current_flow_state_ext: resp.current_flow_state_ext,
                            finish_time: resp.finish_time,
                            next_flow_transitions: if (unfinished_approve_flow_obj_ids.contains(&flow_inst.rel_business_obj_id)
                                && flow_inst.artifacts.clone().unwrap_or_default().rel_transition_id.is_none())
                                || flow_inst.artifacts.clone().unwrap_or_default().state == Some(FlowInstStateKind::Approval)
                            {
                                vec![]
                            } else {
                                resp.next_flow_transitions
                            },
                            rel_flow_versions,
                        })
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

        Ok(state_and_next_transitions)
    }

    pub async fn find_next_transitions(
        flow_inst: &FlowInstDetailResp,
        next_req: &FlowInstFindNextTransitionsReq,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<FlowInstFindNextTransitionResp>> {
        Ok(Self::do_find_next_transitions(flow_inst, None, &next_req.vars, false, funs, ctx).await?.next_flow_transitions)
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
        let vars_collect = FlowTransitionServ::find_detail_items(
            &FlowTransitionFilterReq {
                flow_version_id: Some(flow_model_version.id.clone()),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
        .into_iter()
        .find(|trans| trans.id == transfer_req.flow_transition_id)
        .ok_or_else(|| funs.err().not_found("flow_inst", "check_transfer_vars", "illegal response", "404-flow-transition-not-found"))?
        .vars_collect();
        if let Some(vars_collect) = vars_collect {
            for var in vars_collect {
                if var.required == Some(true) && transfer_req.vars.as_ref().is_none_or(|map| !map.contains_key(&var.name)) {
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
        if flow_inst_detail.main && FlowCacheServ::exist_sync_modify_inst(&flow_inst_detail.own_paths, &flow_inst_detail.tag, &flow_inst_detail.id, funs).await? {
            return Err(funs.err().not_found("flow_inst", "transfer", "instance is locked", "500-flow-inst-tranfer-lock"));
        }
        let mut modified_instance_transations_cp = modified_instance_transations.clone();
        if !modified_instance_transations_cp.check(flow_inst_detail.id.clone(), transfer_req.flow_transition_id.clone()) {
            return Self::gen_transfer_resp(flow_inst_detail, &flow_inst_detail.current_state_id, ctx, funs).await;
        }

        let result = Self::do_transfer(flow_inst_detail, transfer_req, skip_filter, callback_kind, funs, ctx).await;
        let new_inst_detail = Self::get(&flow_inst_detail.id, funs, ctx).await?;
        Self::auto_transfer(&flow_inst_detail.id, modified_instance_transations_cp.clone(), funs, ctx).await?;

        let artifacts = new_inst_detail.artifacts.clone().unwrap_or_default();
        // 若当前处理的是状态流
        if artifacts.rel_transition_id.is_some() && flow_inst_detail.main {
            // 触发开始动作，则创建对应业务的审批流，同时创建子审批流
            if flow_inst_detail.current_state_sys_kind == Some(FlowSysStateKind::Start) && new_inst_detail.current_state_sys_kind != Some(FlowSysStateKind::Finish) {
                let approve_model_version = FlowModelVersionServ::get_item(
                    &artifacts.rel_model_version_id.clone().unwrap_or_default(),
                    &FlowModelVersionFilterReq {
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
                .await?;
                let approve_model = FlowModelServ::get_item(
                    &approve_model_version.rel_model_id,
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
                .await?;
                let root_inst_id = Self::start_secondary_flow(
                    &FlowInstStartReq {
                        rel_business_obj_id: flow_inst_detail.rel_business_obj_id.clone(),
                        tag: flow_inst_detail.tag.clone(),
                        create_vars: None,
                        check_vars: transfer_req.vars.clone(),
                        transition_id: artifacts.rel_transition_id.clone(),
                        vars: transfer_req.vars.clone(),
                        rel_transition_id: artifacts.rel_transition_id.clone(),
                        rel_child_objs: artifacts.rel_child_objs.clone(),
                        operator_map: artifacts.operator_map.clone(),
                        log_text: None,
                        rel_inst_id: None,
                        data_source: Some(flow_inst_detail.data_source.clone()),
                    },
                    false,
                    &approve_model,
                    Some(approve_model_version.id),
                    funs,
                    ctx,
                )
                .await?;
                FlowSearchClient::add_or_modify_instance_search(&root_inst_id, Box::new(false), funs, ctx).await?;
                if let Some(rel_child_objs) = &artifacts.rel_child_objs {
                    Self::start_child_flow(&root_inst_id, rel_child_objs, funs, ctx).await?;
                }
            }
            // 触发结束动作时，将对应业务的审批流结束
            if new_inst_detail.current_state_sys_kind == Some(FlowSysStateKind::Finish) {
                if let Some(approve_inst) = Self::find_detail_items(
                    &FlowInstFilterReq {
                        rel_business_obj_ids: Some(vec![flow_inst_detail.rel_business_obj_id.clone()]),
                        tags: Some(vec![flow_inst_detail.tag.clone()]),
                        main: Some(false),
                        finish: Some(false),
                        ..Default::default()
                    },
                    funs,
                    ctx,
                )
                .await?
                .pop()
                {
                    if let Some(next_transition) = Self::find_next_transitions(&approve_inst, &FlowInstFindNextTransitionsReq { vars: None }, funs, ctx).await?.pop() {
                        let next_state = FlowStateServ::get_item(
                            &next_transition.next_flow_state_id,
                            &FlowStateFilterReq {
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
                        .await?;
                        if next_state.sys_state == FlowSysStateKind::Finish {
                            Self::transfer_root_inst(&approve_inst.id, true, funs, ctx).await?;
                        } else {
                            Self::abort(&approve_inst.id, &FlowInstAbortReq { message: "".to_string() }, funs, ctx).await?;
                        }
                    } else {
                        Self::abort(&approve_inst.id, &FlowInstAbortReq { message: "".to_string() }, funs, ctx).await?;
                    }
                } else {
                    // 不存在审批流则按照关联的工作项刷新对应的search
                    if let Some(rel_child_objs) = new_inst_detail.artifacts.clone().unwrap_or_default().rel_child_objs {
                        for rel_child_obj in rel_child_objs {
                            if let Some(main_inst) = Self::find_items(
                                &FlowInstFilterReq {
                                    rel_business_obj_ids: Some(vec![rel_child_obj.obj_id.clone()]),
                                    main: Some(true),
                                    ..Default::default()
                                },
                                funs,
                                ctx,
                            )
                            .await?
                            .pop()
                            {
                                Self::modify_inst_artifacts(
                                    &main_inst.id,
                                    &FlowInstArtifactsModifyReq {
                                        state: Some(FlowInstStateKind::Overrule),
                                        ..Default::default()
                                    },
                                    funs,
                                    ctx,
                                )
                                .await?;
                                let modify_serach_ext = TardisFuns::json.obj_to_string(&ModifyObjSearchExtReq {
                                    tag: rel_child_obj.tag.clone(),
                                    status: Some(main_inst.current_state_name.clone()),
                                    rel_state: Some("".to_string()),
                                    rel_transition_state_name: Some("".to_string()),
                                })?;
                                FlowSearchClient::add_search_task(&FlowSearchTaskKind::ModifyBusinessObj, &rel_child_obj.obj_id, &modify_serach_ext, funs, ctx).await?;
                            }
                        }
                    }
                }
            } else {
                // 不存在审批流则按照关联的工作项刷新对应的search
                if let Some(rel_child_objs) = new_inst_detail.artifacts.clone().unwrap_or_default().rel_child_objs {
                    for rel_child_obj in rel_child_objs {
                        FlowSearchClient::refresh_business_obj_search(&rel_child_obj.obj_id, &rel_child_obj.tag, funs, ctx).await?;
                    }
                }
            }
        }

        if !flow_inst_detail.main && flow_inst_detail.rel_inst_id.as_ref().is_none_or(|id| id.is_empty()) {
            FlowSearchClient::add_search_task(&FlowSearchTaskKind::ModifyInstance, &flow_inst_detail.id, "", funs, ctx).await?;
        }

        if flow_inst_detail.main {
            let flow_inst_cp = flow_inst_detail.clone();
            let new_inst_detail_cp = new_inst_detail.clone();
            let flow_transition_id = transfer_req.flow_transition_id.clone();
            let ctx_cp = ctx.clone();
            tardis::tokio::spawn(async move {
                let funs = flow_constants::get_tardis_inst();
                match FlowEventServ::do_post_change(&flow_inst_cp, &flow_transition_id, modified_instance_transations_cp.clone(), &ctx_cp, &funs).await {
                    Ok(_) => {}
                    Err(e) => error!("Flow Instance {} do_post_change error:{:?}", flow_inst_cp.id, e),
                }
                match FlowEventServ::do_front_change(&new_inst_detail_cp, modified_instance_transations_cp.clone(), &ctx_cp, &funs).await {
                    Ok(_) => {}
                    Err(e) => error!("Flow Instance {} do_front_change error:{:?}", flow_inst_cp.id, e),
                }
            });
        }

        result
    }

    // 中止子工作流实例
    async fn abort_child_inst(root_inst_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let child_insts = Self::find_detail_items(
            &FlowInstFilterReq {
                with_sub: Some(true),
                rel_inst_ids: Some(vec![root_inst_id.to_string()]),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        for child_inst in child_insts {
            Self::abort(&child_inst.id, &FlowInstAbortReq { message: "".to_string() }, funs, ctx).await?;
            let current_state_name = FlowInstServ::find_detail_items(
                &FlowInstFilterReq {
                    rel_business_obj_ids: Some(vec![child_inst.rel_business_obj_id.clone()]),
                    tags: Some(vec![child_inst.tag.clone()]),
                    main: Some(true),
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?
            .pop()
            .map(|inst| inst.current_state_name.unwrap_or_default());
            let (rel_state, rel_transition_state_name) = FlowInstServ::find_detail_items(
                &FlowInstFilterReq {
                    rel_business_obj_ids: Some(vec![child_inst.rel_business_obj_id.clone()]),
                    main: Some(false),
                    finish: Some(false),
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?
            .pop()
            .map(|inst| (inst.artifacts.unwrap_or_default().state, inst.current_state_name))
            .unwrap_or_default();
            let modify_serach_ext = TardisFuns::json.obj_to_string(&ModifyObjSearchExtReq {
                tag: child_inst.tag.clone(),
                status: current_state_name,
                rel_state: Some(rel_state.map_or("".to_string(), |s| s.to_string())),
                rel_transition_state_name: Some(rel_transition_state_name.unwrap_or_default()),
            })?;
            FlowSearchClient::add_search_task(&FlowSearchTaskKind::ModifyBusinessObj, &child_inst.rel_business_obj_id, &modify_serach_ext, funs, ctx).await?;
        }

        Ok(())
    }

    async fn do_transfer(
        flow_inst_detail: &FlowInstDetailResp,
        transfer_req: &FlowInstTransferReq,
        skip_filter: bool,
        callback_kind: FlowExternalCallbackOp,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<FlowInstTransferResp> {
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
        let next_flow_transition = Self::do_find_next_transitions(
            flow_inst_detail,
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
                flow_inst_detail,
                &FlowTransitionServ::find_detail_items(
                    &FlowTransitionFilterReq {
                        flow_version_id: Some(flow_model_version.id.clone()),
                        ..Default::default()
                    },
                    funs,
                    ctx,
                )
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
        let version_transition = FlowTransitionServ::find_detail_items(
            &FlowTransitionFilterReq {
                flow_version_id: Some(flow_model_version.id.clone()),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;

        let next_flow_transition = next_flow_transition.unwrap_or_default();
        let next_transition_detail = version_transition.iter().find(|trans| trans.id == next_flow_transition.next_flow_transition_id).cloned().unwrap_or_default();
        let prev_flow_state = FlowStateServ::get_item(
            &flow_inst_detail.current_state_id,
            &FlowStateFilterReq {
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
        .await?;
        let next_flow_state = FlowStateServ::get_item(
            &next_flow_transition.next_flow_state_id,
            &FlowStateFilterReq {
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
        let prev_flow_state = FlowStateServ::get_item(
            prev_flow_state_id,
            &FlowStateFilterReq {
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
        .await?;
        let next_flow_state = FlowStateServ::get_item(
            &flow_inst_detail.current_state_id,
            &FlowStateFilterReq {
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
        .await?;
        let next_flow_transitions = Self::do_find_next_transitions(flow_inst_detail, None, &None, false, funs, ctx).await?.next_flow_transitions;

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
        skip_filter: bool,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<FlowInstFindTransitionsResp> {
        let flow_model_transitions = FlowTransitionServ::find_detail_items(
            &FlowTransitionFilterReq {
                flow_version_id: Some(flow_inst.rel_flow_version_id.clone()),
                specified_state_ids: Some(vec![flow_inst.current_state_id.clone()]),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;

        let next_transitions = flow_model_transitions
            .iter()
            .filter(|model_transition| spec_flow_transition_id.is_none() || model_transition.id == spec_flow_transition_id.clone().unwrap_or_default())
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

        let state_and_next_transitions = FlowInstFindTransitionsResp {
            flow_inst_id: flow_inst.id.to_string(),
            finish_time: flow_inst.finish_time,
            current_flow_state_name: flow_inst.current_state_name.as_ref().unwrap_or(&"".to_string()).to_string(),
            current_flow_state_color: flow_inst.current_state_color.as_ref().unwrap_or(&"".to_string()).to_string(),
            current_flow_state_sys_kind: flow_inst.current_state_sys_kind.as_ref().unwrap_or(&FlowSysStateKind::Start).clone(),
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
                let mut funs = flow_constants::get_tardis_inst();
                funs.begin().await.unwrap_or_default();
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
        let resp = FlowExternalServ::do_query_field(tag, vec![rel_business_obj_id.clone()], &ctx.own_paths, ctx, funs)
            .await?
            .objs
            .pop()
            .map(|val| TardisFuns::json.json_to_obj::<HashMap<String, Value>>(val).unwrap_or_default())
            .unwrap_or_default();
        // 去除key的custom_前缀
        let mut new_vars = HashMap::new();
        for (key, value) in &resp {
            if key.contains("custom_") {
                new_vars.insert(key[7..key.len()].to_string(), value.clone());
            } else {
                new_vars.insert(key.clone(), value.clone());
            }
        }
        // 添加当前状态名称
        if let Some(flow_id) = Self::get_inst_ids_by_rel_business_obj_id(vec![rel_business_obj_id], Some(true), funs, ctx).await?.pop() {
            let current_state_name = Self::get(&flow_id, funs, ctx).await?.current_state_name.unwrap_or_default();
            new_vars.insert("status".to_string(), json!(current_state_name));
        }

        Ok(new_vars)
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
                    if old_state != new_state {
                        Self::async_unsafe_modify_state(
                            &FlowInstFilterReq {
                                main: Some(true),
                                tags: Some(vec![new_model.tag.clone()]),
                                current_state_id: Some(old_state.clone()),
                                ..Default::default()
                            },
                            new_state,
                            funs,
                            &mock_ctx,
                        )
                        .await?;
                    }
                }
            } else {
                Self::async_unsafe_modify_state(
                    &FlowInstFilterReq {
                        main: Some(true),
                        tags: Some(vec![new_model.tag.clone()]),
                        ..Default::default()
                    },
                    &new_model.init_state_id,
                    funs,
                    &mock_ctx,
                )
                .await?;
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
                finish_abort: Some(false),
                flow_version_id: Some(rel_flow_version_id.to_string()),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
        .into_iter()
        .collect_vec();
        join_all(
            insts
                .iter()
                .map(|inst| async {
                    let ctx_cp = ctx.clone();
                    let result = Self::abort(&inst.id, &FlowInstAbortReq { message: "".to_string() }, funs, &ctx_cp).await;
                    match FlowSearchClient::execute_async_task(&ctx_cp).await {
                        Ok(_) => {}
                        Err(e) => error!("flow Instance {} add search task error:{:?}", inst.id, e),
                    }
                    match ctx_cp.execute_task().await {
                        Ok(_) => {}
                        Err(e) => error!("flow Instance {} execute_task error:{:?}", inst.id, e),
                    }
                    result
                })
                .collect_vec(),
        )
        .await
        .into_iter()
        .collect::<TardisResult<Vec<_>>>()?;
        Ok(())
    }

    pub async fn unsafe_modify_state(filter: &FlowInstFilterReq, state_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let insts = Self::find_items(filter, funs, ctx).await?.into_iter().filter(|inst| inst.current_state_id != *state_id).collect_vec();
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
                                    own_paths: Some("".to_string()),
                                    with_sub_own_paths: true,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            funs,
                            ctx,
                        )
                        .await,
                        FlowStateServ::get_item(
                            state_id,
                            &FlowStateFilterReq {
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
                        Err(funs.err().not_found("flow_inst", "unsafe_modify_state", "flow state is not found", "404-flow-state-not-found"))
                    }
                })
                .collect_vec(),
        )
        .await
        .into_iter()
        .collect::<TardisResult<Vec<_>>>()?;

        Ok(())
    }

    pub async fn async_unsafe_modify_state(filter: &FlowInstFilterReq, state_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let insts = Self::find_items(filter, funs, ctx).await?.into_iter().filter(|inst| inst.current_state_id != *state_id).collect_vec();
        let inst_ids = insts.iter().map(|inst| inst.id.clone()).collect_vec();
        let mut update_statement = Query::update();
        update_statement.table(flow_inst::Entity);
        update_statement.value(flow_inst::Column::CurrentStateId, state_id);
        update_statement.and_where(Expr::col((flow_inst::Entity, flow_inst::Column::Id)).is_in(inst_ids));

        funs.db().execute(&update_statement).await?;

        join_all(insts.iter().map(|inst| async { FlowCacheServ::add_sync_modify_inst(&inst.own_paths, &inst.tag, &inst.id, funs).await }).collect_vec())
            .await
            .into_iter()
            .collect::<TardisResult<Vec<_>>>()?;

        let state_id_cp = state_id.to_string();
        let ctx_cp = ctx.clone();
        tardis::tokio::spawn(async move {
            warn!("start notify change status: {:?}", insts);
            let funs = flow_constants::get_tardis_inst();
            let mut versions: Vec<FlowModelVersionDetailResp> = vec![];
            let mut num = 0;
            for inst in insts {
                num += 1;
                if num % 2000 == 0 {
                    tardis::tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
                let states = if let Some(version) = versions.iter().find(|version| version.id == inst.rel_flow_version_id) {
                    Some(version.states())
                } else {
                    let version = FlowModelVersionServ::get_item(
                        &inst.rel_flow_version_id,
                        &FlowModelVersionFilterReq {
                            basic: RbumBasicFilterReq {
                                with_sub_own_paths: true,
                                own_paths: Some("".to_string()),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        &funs,
                        &ctx_cp,
                    )
                    .await
                    .ok();
                    if let Some(version) = &version {
                        versions.push(version.clone());
                    }
                    version.map(|v| v.states())
                };
                if let Some(states) = states {
                    if let (Some(original_flow_state), Some(next_flow_state)) = (
                        states.iter().find(|state| state.id == inst.current_state_id),
                        states.iter().find(|state| state.id == state_id_cp),
                    ) {
                        match FlowExternalServ::do_notify_changes(
                            &inst.tag,
                            &inst.id,
                            &inst.rel_business_obj_id,
                            next_flow_state.name.clone(),
                            next_flow_state.sys_state.clone(),
                            original_flow_state.name.clone(),
                            original_flow_state.sys_state.clone(),
                            "UPDATE".to_string(),
                            false,
                            Some(false),
                            Some(FlowExternalCallbackOp::Auto),
                            &ctx_cp,
                            &funs,
                        )
                        .await
                        {
                            Ok(_) => {}
                            Err(e) => error!("Flow Instance {} modify state error:{:?}", inst.id, e),
                        }
                    } else {
                        error!("Flow Instance {}: flow state not found", inst.id);
                    }
                }
                match FlowCacheServ::del_sync_modify_inst(&inst.own_paths, &inst.tag, &inst.id, &funs).await {
                    Ok(_) => {}
                    Err(e) => error!("Flow Instance {} del_sync_modify_inst error:{:?}", inst.id, e),
                }
            }
        });

        Ok(())
    }

    pub async fn auto_transfer(
        flow_inst_id: &str,
        modified_instance_transations: loop_check_helper::InstancesTransition,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let flow_inst_detail = Self::get(flow_inst_id, funs, ctx).await?;
        let transition_ids = Self::do_find_next_transitions(&flow_inst_detail, None, &None, false, funs, ctx)
            .await?
            .next_flow_transitions
            .into_iter()
            .map(|tran| tran.next_flow_transition_id)
            .collect_vec();
        let check_vars = if let Some(rel_inst_id) = &flow_inst_detail.rel_inst_id {
            let root_inst = Self::get(rel_inst_id, funs, ctx).await?;
            let mut original_vars = root_inst.create_vars.clone().unwrap_or_default();
            original_vars.extend(Self::get_modify_vars(
                &root_inst.artifacts.clone().unwrap_or_default(),
                &root_inst.transitions.clone().unwrap_or_default().into_iter().map(|tran| tran.from_state_id.unwrap_or_default()).collect_vec(),
            ));
            BasicQueryCondInfo::transform(original_vars)?
        } else {
            let mut original_vars = flow_inst_detail.create_vars.clone().unwrap_or_default();
            original_vars.extend(Self::get_modify_vars(
                &flow_inst_detail.artifacts.clone().unwrap_or_default(),
                &flow_inst_detail.transitions.clone().unwrap_or_default().into_iter().map(|tran| tran.from_state_id.unwrap_or_default()).collect_vec(),
            ));
            BasicQueryCondInfo::transform(original_vars)?
        };
        match Self::find_auto_transition(transition_ids, &check_vars, funs, ctx).await {
            Ok(auto_transition) => {
                if let Some(auto_transition) = auto_transition {
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
                Ok(())
            }
            Err(e) => {
                if e.code == *"404-flow-flow_inst-find_auto_transition" {
                    Ok(())
                } else {
                    Err(e)
                }
            }
        }
    }

    // 获取自动流转节点
    async fn find_auto_transition(
        transition_ids: Vec<String>,
        check_vars: &HashMap<String, Value>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Option<FlowTransitionDetailResp>> {
        let auto_transitions = FlowTransitionServ::find_detail_items(
            &FlowTransitionFilterReq {
                ids: Some(transition_ids),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
        .into_iter()
        .filter(|transition| transition.transfer_by_auto)
        .collect_vec();
        if auto_transitions.is_empty() {
            return Err(funs.err().not_found("flow_inst", "find_auto_transition", "auto transition not found", "404-flow-transition-not-found"));
        }
        Ok(auto_transitions.into_iter().find(|transition| {
            (transition.transfer_by_auto && transition.guard_by_other_conds().is_none())
                || (transition.transfer_by_auto
                    && transition.guard_by_other_conds().is_some()
                    && BasicQueryCondInfo::check_or_and_conds(&transition.guard_by_other_conds().unwrap_or_default(), check_vars).unwrap_or(true))
        }))
    }

    // 获取当前操作人
    async fn get_curr_operators(
        flow_inst_detail: &FlowInstDetailResp,
        state_detail: &FlowStateDetailResp,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<String>> {
        let (mut guard_custom_conf, guard_by_creator, guard_by_his_operators, guard_by_assigned) = match state_detail.state_kind {
            FlowStateKind::Form => Some((
                state_detail.kind_conf().unwrap_or_default().form.unwrap_or_default().guard_custom_conf.unwrap_or_default(),
                state_detail.kind_conf().unwrap_or_default().form.unwrap_or_default().guard_by_creator,
                state_detail.kind_conf().unwrap_or_default().form.unwrap_or_default().guard_by_his_operators,
                state_detail.kind_conf().unwrap_or_default().form.unwrap_or_default().guard_by_assigned,
            )),
            FlowStateKind::Approval => Some((
                state_detail.kind_conf().unwrap_or_default().approval.unwrap_or_default().guard_custom_conf.unwrap_or_default(),
                state_detail.kind_conf().unwrap_or_default().approval.unwrap_or_default().guard_by_creator,
                state_detail.kind_conf().unwrap_or_default().approval.unwrap_or_default().guard_by_his_operators,
                state_detail.kind_conf().unwrap_or_default().approval.unwrap_or_default().guard_by_assigned,
            )),
            _ => None,
        }
        .ok_or_else(|| funs.err().not_found("flow_inst", "get_curr_operators", "flow state is not found", "404-flow-state-not-found"))?;
        if state_detail.own_paths != flow_inst_detail.own_paths {
            guard_custom_conf.get_local_conf(funs, ctx).await?;
        }
        if guard_by_creator {
            guard_custom_conf.guard_by_spec_account_ids.push(flow_inst_detail.create_ctx.owner.clone());
        }
        if guard_by_his_operators {
            flow_inst_detail
                .transitions
                .as_ref()
                .map(|transitions| transitions.iter().map(|transition| guard_custom_conf.guard_by_spec_account_ids.push(transition.op_ctx.owner.clone())).collect::<Vec<_>>());
        }
        if guard_by_assigned
            // 当配置了对应的操作人权限则忽略guard_by_assigned规则
            && !flow_inst_detail.artifacts.clone().unwrap_or_default().operator_map.unwrap_or_default().contains_key(&flow_inst_detail.current_state_id)
        {
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
        let mut result = FlowSearchClient::search_guard_accounts(&guard_custom_conf, funs, ctx).await?;
        // 若当前节点已配置对应的操作人权限则直接使用
        if let Some(mut operators) = flow_inst_detail.artifacts.clone().unwrap_or_default().operator_map.unwrap_or_default().get(&flow_inst_detail.current_state_id).cloned() {
            result.append(&mut operators);
            result = result.into_iter().unique().collect_vec();
        }
        Ok(result)
    }

    // 当进入该节点时
    async fn when_enter_state(flow_inst_detail: &FlowInstDetailResp, state_id: &str, _flow_model_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if flow_inst_detail.main {
            return Ok(());
        }
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
                modify_req.curr_operators = Some(Self::get_curr_operators(flow_inst_detail, &state, funs, ctx).await?);
                modify_req.state = Some(FlowInstStateKind::Form);
                Self::modify_inst_artifacts(&flow_inst_detail.id, &modify_req, funs, ctx).await?;
                // 当操作人为空时的逻辑
                let curr_operators = Self::get(&flow_inst_detail.id, funs, ctx).await?.artifacts.unwrap_or_default().curr_operators.unwrap_or_default();
                if curr_operators.is_empty() && form_conf.auto_transfer_when_empty_kind.is_some() {
                    match form_conf.auto_transfer_when_empty_kind.unwrap_or_default() {
                        FlowStatusAutoStrategyKind::Autoskip => {
                            if let Some(next_transition) = Self::find_next_transitions(flow_inst_detail, &FlowInstFindNextTransitionsReq { vars: None }, funs, ctx).await?.pop() {
                                FlowLogServ::add_operate_log_async_task(
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
                let guard_accounts = Self::get_curr_operators(flow_inst_detail, &state, funs, ctx).await?;
                let curr_approval_total = guard_accounts.len();
                modify_req.curr_approval_total = Some(curr_approval_total);
                modify_req.curr_operators = Some(guard_accounts);
                modify_req.state = Some(FlowInstStateKind::Approval);

                Self::modify_inst_artifacts(&flow_inst_detail.id, &modify_req, funs, ctx).await?;
                // 当操作人为空时的逻辑
                if curr_approval_total == 0 && approval_conf.auto_transfer_when_empty_kind.is_some() {
                    match approval_conf.auto_transfer_when_empty_kind.unwrap_or_default() {
                        FlowStatusAutoStrategyKind::Autoskip => {
                            if let Some(next_transition) = Self::find_next_transitions(flow_inst_detail, &FlowInstFindNextTransitionsReq { vars: None }, funs, ctx).await?.pop() {
                                FlowLogServ::add_operate_log_async_task(
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
                            Self::modify_inst_artifacts(&flow_inst_detail.id, &modify_req, funs, ctx).await?;
                        }
                        FlowStatusAutoStrategyKind::TransferState => {
                            // 当前版本不支持
                        }
                    }
                }
            }
            FlowStateKind::Branch => {}
            FlowStateKind::Finish => {
                // 子审批流不需要触发结束事件
                if flow_inst_detail.rel_inst_id.as_ref().is_none_or(|id| id.is_empty()) {
                    Self::finish_approve_flow(
                        flow_inst_detail.rel_transition.clone().unwrap_or_default(),
                        &flow_inst_detail.tag,
                        &flow_inst_detail.rel_business_obj_id,
                        Some(flow_inst_detail.id.clone()),
                        &flow_inst_detail.artifacts.clone().unwrap_or_default(),
                        &flow_inst_detail.transitions.clone().unwrap_or_default().into_iter().map(|tran| tran.from_state_id.unwrap_or_default()).collect_vec(),
                        funs,
                        ctx,
                    )
                    .await?;
                    FlowLogServ::add_finish_log_async_task(flow_inst_detail, None, funs, ctx).await?;
                }
            }
            _ => {}
        }
        if flow_inst_detail.rel_inst_id.as_ref().is_none_or(|id| id.is_empty()) && !flow_inst_detail.main {
            FlowSearchClient::refresh_business_obj_search(&flow_inst_detail.rel_business_obj_id, &flow_inst_detail.tag, funs, ctx).await?;
            // 更新关联业务的search
            let child_insts = Self::find_detail_items(
                &FlowInstFilterReq {
                    rel_inst_ids: Some(vec![flow_inst_detail.id.clone()]),
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?;
            let child_main_insts = Self::find_detail_items(
                &FlowInstFilterReq {
                    rel_business_obj_ids: Some(
                        flow_inst_detail
                            .artifacts
                            .clone()
                            .unwrap_or_default()
                            .rel_child_objs
                            .unwrap_or_default()
                            .into_iter()
                            .map(|rel_child_obj| rel_child_obj.obj_id)
                            .collect_vec(),
                    ),
                    main: Some(true),
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?;
            for child_inst in child_insts {
                let child_main_inst = child_main_insts.iter().find(|inst| child_inst.rel_business_obj_id == inst.rel_business_obj_id).ok_or_else(|| {
                    funs.err().not_found(
                        "flow_inst_serv",
                        "when_enter_state",
                        &format!("inst is not found by business_obj_id {}", child_inst.rel_business_obj_id),
                        "404-flow-inst-not-found",
                    )
                })?;
                let modify_serach_ext = TardisFuns::json.obj_to_string(&ModifyObjSearchExtReq {
                    tag: child_inst.tag.clone(),
                    status: if flow_inst_detail.finish_time.is_some() {
                        child_main_inst.current_state_name.clone()
                    } else {
                        Some(flow_constants::SPECIFED_APPROVING_STATE_NAME.to_string())
                    },
                    rel_state: if flow_inst_detail.finish_time.is_some() {
                        Some("".to_string())
                    } else {
                        child_inst.artifacts.clone().unwrap_or_default().state.map(|s| s.to_string())
                    },
                    rel_transition_state_name: if flow_inst_detail.finish_time.is_some() {
                        Some("".to_string())
                    } else {
                        child_inst.current_state_name
                    },
                })?;
                FlowSearchClient::add_search_task(&FlowSearchTaskKind::ModifyBusinessObj, &child_inst.rel_business_obj_id, &modify_serach_ext, funs, ctx).await?;
            }
        }

        Ok(())
    }

    // 结束审批流
    async fn finish_approve_flow(
        rel_transition: FlowModelRelTransitionExt,
        tag: &str,
        rel_business_obj_id: &str,
        root_inst_id: Option<String>,
        artifacts: &FlowInstArtifacts,
        state_ids: &[String],
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let inst_id = Self::get_inst_ids_by_rel_business_obj_id(vec![rel_business_obj_id.to_string()], Some(true), funs, ctx).await?.pop().ok_or_else(|| {
            funs.err().not_found(
                "flow_inst_serv",
                "finish_approve_flow",
                &format!("inst is not found by business_obj_id {}", rel_business_obj_id),
                "404-flow-inst-not-found",
            )
        })?;
        let inst_detail = Self::get(&inst_id, funs, ctx).await?;
        // 流程结束时，更新对应的主审批流的search状态
        if let Some(main_inst) = Self::find_detail_items(
            &FlowInstFilterReq {
                rel_business_obj_ids: Some(vec![rel_business_obj_id.to_string()]),
                main: Some(true),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
        .pop()
        {
            let modify_serach_ext = TardisFuns::json.obj_to_string(&ModifyObjSearchExtReq {
                tag: main_inst.tag.to_string(),
                status: main_inst.current_state_name.clone(),
                rel_state: Some("".to_string()),
                rel_transition_state_name: Some("".to_string()),
            })?;
            FlowSearchClient::add_search_task(&FlowSearchTaskKind::ModifyBusinessObj, rel_business_obj_id, &modify_serach_ext, funs, ctx).await?;
        }

        match FlowModelRelTransitionKind::from(rel_transition) {
            FlowModelRelTransitionKind::Edit => {
                let vars_collect = Self::get_modify_vars(artifacts, state_ids);
                let params = vars_collect
                    .into_iter()
                    .map(|(key, value)| FlowExternalParams {
                        var_name: Some(key),
                        value: Some(value),
                        ..Default::default()
                    })
                    .collect_vec();
                FlowExternalServ::do_async_modify_field(
                    tag,
                    None,
                    rel_business_obj_id,
                    &inst_id,
                    Some(FlowExternalCallbackOp::Auto),
                    None,
                    Some("审批通过".to_string()),
                    inst_detail.current_state_name.clone(),
                    inst_detail.current_state_sys_kind.clone(),
                    inst_detail.current_state_name.clone(),
                    inst_detail.current_state_sys_kind.clone(),
                    params,
                    ctx,
                    funs,
                )
                .await?;
            }
            FlowModelRelTransitionKind::Delete => {
                FlowExternalServ::do_delete_rel_obj(tag, rel_business_obj_id, &inst_id, ctx, funs).await?;
            }
            FlowModelRelTransitionKind::Related => {
                let vars_collect = Self::get_modify_vars(artifacts, state_ids);
                FlowExternalServ::do_update_related_obj(tag, rel_business_obj_id, &inst_id, &vars_collect, ctx, funs).await?;
            }
            FlowModelRelTransitionKind::Review => {
                if let Some(main_inst) = Self::find_detail_items(
                    &FlowInstFilterReq {
                        rel_business_obj_ids: Some(vec![rel_business_obj_id.to_string()]),
                        main: Some(true),
                        ..Default::default()
                    },
                    funs,
                    ctx,
                )
                .await?
                .pop()
                {
                    // 关联子流程的处理
                    let root_config = FlowConfigServ::get_root_config(&main_inst.tag, funs, ctx).await?;
                    let rel_child_objs = main_inst.artifacts.clone().unwrap_or_default().rel_child_objs.unwrap_or_default();
                    if let Some(root_inst_id) = root_inst_id {
                        Self::modify_inst_artifacts(
                            &root_inst_id,
                            &FlowInstArtifactsModifyReq {
                                state: Some(FlowInstStateKind::Pass),
                                ..Default::default()
                            },
                            funs,
                            ctx,
                        )
                        .await?;
                        let child_insts = Self::find_detail_items(
                            &FlowInstFilterReq {
                                rel_business_obj_ids: Some(rel_child_objs.iter().map(|rel_child_obj| rel_child_obj.obj_id.clone()).collect_vec()),
                                main: Some(false),
                                rel_inst_ids: Some(vec![root_inst_id.clone()]),
                                ..Default::default()
                            },
                            funs,
                            ctx,
                        )
                        .await?;
                        for child_inst in child_insts {
                            let artifacts = child_inst.artifacts.clone().unwrap_or_default();
                            if let Some(conf) = FlowConfigServ::get_root_config_by_tag(&root_config, &child_inst.tag)? {
                                if let Some(child_main_inst_id) =
                                    Self::get_inst_ids_by_rel_business_obj_id(vec![child_inst.rel_business_obj_id.clone()], Some(true), funs, ctx).await?.pop()
                                {
                                    FlowLogServ::add_finish_business_log_async_task(&child_inst, None, funs, ctx).await?;
                                    if artifacts.state == Some(FlowInstStateKind::Pass) {
                                        // 更新业务主流程的artifact的状态为审批通过
                                        Self::modify_inst_artifacts(
                                            &child_main_inst_id,
                                            &FlowInstArtifactsModifyReq {
                                                state: Some(FlowInstStateKind::Pass),
                                                ..Default::default()
                                            },
                                            funs,
                                            ctx,
                                        )
                                        .await?;
                                        Self::unsafe_modify_state(
                                            &FlowInstFilterReq {
                                                ids: Some(vec![child_main_inst_id.clone()]),
                                                with_sub: Some(true),
                                                ..Default::default()
                                            },
                                            &conf.pass_status,
                                            funs,
                                            ctx,
                                        )
                                        .await?;
                                    } else {
                                        // 更新业务主流程的artifact的状态为审批拒绝
                                        Self::modify_inst_artifacts(
                                            &child_main_inst_id,
                                            &FlowInstArtifactsModifyReq {
                                                state: Some(FlowInstStateKind::Overrule),
                                                ..Default::default()
                                            },
                                            funs,
                                            ctx,
                                        )
                                        .await?;
                                        Self::unsafe_modify_state(
                                            &FlowInstFilterReq {
                                                ids: Some(vec![child_main_inst_id.clone()]),
                                                with_sub: Some(true),
                                                ..Default::default()
                                            },
                                            &conf.unpass_status,
                                            funs,
                                            ctx,
                                        )
                                        .await?;
                                    }
                                }
                            }
                        }
                    } else {
                        // 未传入root_inst_id，代表流程直接中止，刷新业务的search中的信息
                        for rel_child_obj in rel_child_objs {
                            let current_state_name = FlowInstServ::find_detail_items(
                                &FlowInstFilterReq {
                                    rel_business_obj_ids: Some(vec![rel_child_obj.obj_id.clone()]),
                                    tags: Some(vec![rel_child_obj.tag.clone()]),
                                    main: Some(true),
                                    ..Default::default()
                                },
                                funs,
                                ctx,
                            )
                            .await?
                            .pop()
                            .map(|inst| inst.current_state_name.unwrap_or_default());
                            let modify_ext_req = ModifyObjSearchExtReq {
                                tag: rel_child_obj.tag.clone(),
                                status: current_state_name,
                                rel_state: None,
                                rel_transition_state_name: Some("".to_string()),
                            };
                            let modify_serach_ext = TardisFuns::json.obj_to_string(&modify_ext_req)?;
                            FlowSearchClient::add_search_task(&FlowSearchTaskKind::ModifyBusinessObj, &rel_child_obj.obj_id, &modify_serach_ext, funs, ctx).await?;
                        }
                    }

                    // 更新业务主流程的artifact的状态为审批通过
                    Self::modify_inst_artifacts(
                        &main_inst.id,
                        &FlowInstArtifactsModifyReq {
                            state: Some(FlowInstStateKind::Pass),
                            ..Default::default()
                        },
                        funs,
                        ctx,
                    )
                    .await?;
                    // 结束主流程的状态流实例
                    let next_trans = Self::find_next_transitions(&main_inst, &FlowInstFindNextTransitionsReq { vars: None }, funs, ctx).await?;
                    for next_tran in next_trans {
                        let next_state = FlowStateServ::get_item(
                            &next_tran.next_flow_state_id,
                            &FlowStateFilterReq {
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
                        .await?;
                        if next_state.sys_state == FlowSysStateKind::Finish {
                            Self::transfer(
                                &main_inst,
                                &FlowInstTransferReq {
                                    flow_transition_id: next_tran.next_flow_transition_id,
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
                            break;
                        }
                    }
                }
            }
            FlowModelRelTransitionKind::Transfer(tran) => {
                let vars_collect = Self::get_modify_vars(artifacts, state_ids);
                let params = vars_collect
                    .into_iter()
                    .map(|(key, value)| FlowExternalParams {
                        var_name: Some(key),
                        value: Some(value),
                        ..Default::default()
                    })
                    .collect_vec();
                FlowExternalServ::do_async_modify_field(
                    tag,
                    None,
                    rel_business_obj_id,
                    &inst_id,
                    None,
                    None,
                    Some("审批通过".to_string()),
                    inst_detail.current_state_name.clone(),
                    inst_detail.current_state_sys_kind.clone(),
                    inst_detail.current_state_name.clone(),
                    inst_detail.current_state_sys_kind.clone(),
                    params,
                    ctx,
                    funs,
                )
                .await?;
                if let Some(inst_id) = Self::get_inst_ids_by_rel_business_obj_id(vec![rel_business_obj_id.to_string()], Some(true), funs, ctx).await?.pop() {
                    let inst_detail = Self::get(&inst_id, funs, ctx).await?;
                    Self::transfer(
                        &inst_detail,
                        &FlowInstTransferReq {
                            flow_transition_id: tran.id.clone(),
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
        }
        Ok(())
    }

    // 当离开该节点时
    async fn when_leave_state(flow_inst_detail: &FlowInstDetailResp, state_id: &str, _flow_model_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if flow_inst_detail.main {
            return Ok(());
        }
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
        if let Some(state) = modify_artifacts.state {
            inst_artifacts.state = Some(state);
        }
        if let Some(rel_child_objs) = &modify_artifacts.rel_child_objs {
            inst_artifacts.rel_child_objs = Some(rel_child_objs.clone());
        }
        if let Some(rel_model_version_id) = &modify_artifacts.rel_model_version_id {
            inst_artifacts.rel_model_version_id = Some(rel_model_version_id.clone());
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
        if let Some(operator_map) = &modify_artifacts.operator_map {
            inst_artifacts.operator_map = Some(operator_map.clone());
        }
        if let Some(rel_transition_id) = &modify_artifacts.rel_transition_id {
            inst_artifacts.rel_transition_id = Some(rel_transition_id.clone());
        }
        if let Some((referral_account_id, master_account_ids)) = &modify_artifacts.add_referral_map {
            let mut referral_map = inst_artifacts.referral_map.clone().unwrap_or_default();
            let current_referral_map = referral_map.entry(inst.current_state_id.clone()).or_default();
            let current_referral_account_ids = current_referral_map.entry(referral_account_id.clone()).or_insert(vec![]);
            current_referral_account_ids.clear();
            for master_account_id in master_account_ids {
                current_referral_account_ids.push(master_account_id.clone());
            }
            inst_artifacts.referral_map = Some(referral_map);
        }
        if let Some(remove_account_id) = &modify_artifacts.remove_referral_map {
            let mut referral_map = inst_artifacts.referral_map.clone().unwrap_or_default();
            let current_referral_map = referral_map.entry(inst.current_state_id.clone()).or_default();
            current_referral_map.remove(remove_account_id);
            inst_artifacts.referral_map = Some(referral_map);
        }
        if let Some(state_id) = &modify_artifacts.clear_referral_map {
            let mut referral_map = inst_artifacts.referral_map.clone().unwrap_or_default();
            referral_map.remove(state_id);
            inst_artifacts.referral_map = Some(referral_map);
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
                            || artifacts
                                .referral_map
                                .clone()
                                .unwrap_or_default()
                                .get(state_id)
                                .map_or_else(|| false, |current_referral_map| current_referral_map.contains_key(&ctx.owner)))
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
                        if artifacts.curr_operators.clone().unwrap_or_default().contains(&ctx.owner)
                            || artifacts
                                .referral_map
                                .clone()
                                .unwrap_or_default()
                                .get(state_id)
                                .map_or_else(|| false, |current_referral_map| current_referral_map.contains_key(&ctx.owner))
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

    pub async fn batch_operate(inst_id: &str, batch_operate_req: &HashMap<String, FlowInstOperateReq>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let inst = Self::get(inst_id, funs, ctx).await?;
        let approve_inst = FlowInstServ::find_detail_items(
            &FlowInstFilterReq {
                rel_business_obj_ids: Some(vec![inst.rel_business_obj_id.clone()]),
                finish: Some(false),
                main: Some(false),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
        .pop()
        .ok_or_else(|| {
            funs.err().not_found(
                "flow_inst_serv",
                "batch_operate",
                &format!("inst is not found by business_obj_id {}", inst.rel_business_obj_id),
                "404-flow-inst-not-found",
            )
        })?;
        FlowLogServ::add_operate_log_async_task(
            &FlowInstOperateReq {
                operate: FlowStateOperatorKind::Submit,
                vars: None,
                all_vars: None,
                output_message: None,
                operator: None,
                log_text: None,
            },
            &approve_inst,
            if approve_inst.current_state_kind == Some(FlowStateKind::Approval) {
                LogParamOp::Approval
            } else {
                LogParamOp::Form
            },
            funs,
            ctx,
        )
        .await?;
        let rel_business_obj_ids = FlowInstServ::find_detail_items(
            &FlowInstFilterReq {
                with_sub: Some(true),
                current_state_id: Some(approve_inst.current_state_id.clone()),
                rel_inst_ids: Some(vec![approve_inst.id.to_string()]),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
        .into_iter()
        .filter(|inst| inst.artifacts.as_ref().is_some_and(|artifacts| artifacts.state == Some(FlowInstStateKind::Approval)))
        .map(|inst| inst.rel_business_obj_id)
        .sorted()
        .collect_vec();
        let req_business_obj_ids = batch_operate_req.keys().cloned().sorted().collect_vec();
        if req_business_obj_ids != rel_business_obj_ids {
            return Err(funs.err().not_found("flow_inst", "batch_operate", "some flow instances not found", "404-flow-inst-not-found"));
        }
        let curr_operators = approve_inst.artifacts.clone().unwrap_or_default().curr_operators.unwrap_or_default();
        if curr_operators.contains(&ctx.owner) {
            Self::modify_inst_artifacts(
                &approve_inst.id,
                &FlowInstArtifactsModifyReq {
                    add_his_operator: Some(ctx.owner.clone()),
                    curr_operators: Some(curr_operators.into_iter().filter(|account_id| *account_id != ctx.owner.clone()).collect_vec()),
                    add_approval_result: Some((ctx.owner.clone(), FlowApprovalResultKind::Review)),
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?;
        }

        let ctx_cp = ctx.clone();
        let batch_operate_req_cp = batch_operate_req.clone();
        let approve_inst_id = approve_inst.id.clone();
        tardis::tokio::spawn(async move {
            let funs_cp = flow_constants::get_tardis_inst();
            for (rel_business_obj_id, operate_req) in batch_operate_req_cp {
                if let Some(child_inst_id) = FlowInstServ::find_ids(
                    &FlowInstFilterReq {
                        with_sub: Some(true),
                        rel_business_obj_ids: Some(vec![rel_business_obj_id.clone()]),
                        rel_inst_ids: Some(vec![approve_inst_id.clone()]),
                        ..Default::default()
                    },
                    &funs_cp,
                    &ctx_cp,
                )
                .await
                .unwrap_or_default()
                .pop()
                {
                    if let Ok(inst) = FlowInstServ::get(&child_inst_id, &funs_cp, &ctx_cp).await {
                        let mut funs_cp2 = flow_constants::get_tardis_inst();
                        funs_cp2.begin().await.unwrap_or_default();
                        let result = FlowInstServ::operate(&inst, &operate_req, &funs_cp2, &ctx_cp).await;
                        match result {
                            Ok(_) => {
                                funs_cp2.commit().await.unwrap_or_default();
                                FlowSearchClient::execute_async_task(&ctx_cp).await.unwrap_or_default();
                                ctx_cp.execute_task().await.unwrap_or_default();
                            }
                            Err(e) => error!("Flow Instance {} batch_operate error:{:?}", inst.id, e),
                        }
                    }
                }
            }
        });
        FlowSearchClient::add_search_task(&FlowSearchTaskKind::ModifyInstance, &approve_inst.id, "", funs, ctx).await?;
        Ok(())
    }

    pub async fn operate(inst: &FlowInstDetailResp, operate_req: &FlowInstOperateReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let artifacts = inst.artifacts.clone().unwrap_or_default();
        FlowLogServ::add_operate_log_async_task(
            operate_req,
            inst,
            if inst.current_state_kind == Some(FlowStateKind::Approval) {
                LogParamOp::Approval
            } else {
                LogParamOp::Form
            },
            funs,
            ctx,
        )
        .await?;
        FlowLogServ::add_operate_dynamic_log_async_task(
            operate_req,
            inst,
            if inst.current_state_kind == Some(FlowStateKind::Approval) {
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
                    curr_operators = curr_operators.into_iter().filter(|account_id| *account_id != ctx.owner.clone()).collect_vec();
                    modify_artifacts.curr_operators = Some(curr_operators);

                    let mut master_account_ids = if let Some(current_referral_map) = artifacts.referral_map.clone().unwrap_or_default().get(&inst.current_state_id) {
                        modify_artifacts.remove_referral_map = Some(ctx.owner.clone());
                        current_referral_map.get(&operator).cloned().unwrap_or_default()
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
                        add_approval_result: Some((ctx.owner.clone(), FlowApprovalResultKind::Form)),
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
                let referral_map = artifacts.referral_map.clone().unwrap_or_default();
                if !curr_operators.contains(&ctx.owner)
                    && !referral_map.get(&inst.current_state_id).map_or_else(|| false, |current_referral_map| current_referral_map.contains_key(&ctx.owner))
                {
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
                if referral_map.get(&inst.current_state_id).map_or_else(|| false, |current_referral_map| current_referral_map.contains_key(&ctx.owner)) {
                    if let Some(current_referral_map) = referral_map.get(&inst.current_state_id) {
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
                        FlowLogServ::add_operate_log_async_task(
                            operate_req,
                            inst,
                            if inst.current_state_kind == Some(FlowStateKind::Approval) {
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
                        if let Some(rel_inst_id) = &inst.rel_inst_id {
                            Self::transfer_root_inst(rel_inst_id, false, funs, ctx).await?;
                        } else {
                            Self::transfer(
                                inst,
                                &FlowInstTransferReq {
                                    flow_transition_id: next_transition.next_flow_transition_id.clone(),
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
            }
            // 拒绝
            FlowStateOperatorKind::Overrule => {
                let curr_operators = artifacts.curr_operators.unwrap_or_default();
                let referral_map = artifacts.referral_map.unwrap_or_default();
                if !curr_operators.contains(&ctx.owner)
                    && !referral_map.get(&inst.current_state_id).map_or_else(|| false, |current_referral_map| current_referral_map.contains_key(&ctx.owner))
                {
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
                if referral_map.get(&inst.current_state_id).map_or_else(|| false, |current_referral_map| current_referral_map.contains_key(&ctx.owner)) {
                    if let Some(current_referral_map) = referral_map.get(&inst.current_state_id) {
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
                    FlowLogServ::add_operate_log_async_task(
                        operate_req,
                        inst,
                        if inst.current_state_kind == Some(FlowStateKind::Approval) {
                            LogParamOp::ApprovalTransfer
                        } else {
                            LogParamOp::FormTransfer
                        },
                        funs,
                        ctx,
                    )
                    .await?;
                    Self::abort(&inst.id, &FlowInstAbortReq { message: "".to_string() }, funs, ctx).await?;
                    if let Some(rel_inst_id) = &inst.rel_inst_id {
                        Self::transfer_root_inst(rel_inst_id, false, funs, ctx).await?;
                    }
                }
            }
        }
        FlowSearchClient::add_search_task(&FlowSearchTaskKind::ModifyInstance, &inst.id, "", funs, ctx).await?;
        Ok(())
    }

    async fn transfer_root_inst(root_inst_id: &str, end: bool, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let root_inst = Self::get(root_inst_id, funs, ctx).await?;
        let all_child_insts = Self::find_detail_items(
            &FlowInstFilterReq {
                rel_inst_ids: Some(vec![root_inst_id.to_string()]),
                current_state_id: Some(root_inst.current_state_id.clone()),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        // 所有子审批流在当前节点都明确审批结果后，将父审批流实例流转至下一节点
        if all_child_insts.iter().all(|child| {
            child
                .artifacts
                .as_ref()
                .is_some_and(|artifacts| artifacts.state.unwrap_or_default() == FlowInstStateKind::Pass || artifacts.state.unwrap_or_default() == FlowInstStateKind::Overrule)
        }) {
            let pass_child_inst = all_child_insts
                .iter()
                .filter(|child| child.artifacts.as_ref().is_some_and(|artifacts| artifacts.state.unwrap_or_default() == FlowInstStateKind::Pass))
                .collect_vec();
            if let Some(next_transition) = Self::find_next_transitions(&root_inst, &FlowInstFindNextTransitionsReq { vars: None }, funs, ctx).await?.pop() {
                let next_state = FlowStateServ::get_item(
                    &next_transition.next_flow_state_id,
                    &FlowStateFilterReq {
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
                .await?;
                // 若所有子审批流都拒绝且当前流转不是结束时，直接中断
                if pass_child_inst.is_empty() && next_state.sys_state != FlowSysStateKind::Finish {
                    let root_config = FlowConfigServ::get_root_config(&root_inst.tag, funs, ctx).await?;
                    for child_inst in all_child_insts {
                        if let Some(conf) = FlowConfigServ::get_root_config_by_tag(&root_config, &child_inst.tag)? {
                            if let Some(child_main_inst_id) =
                                Self::get_inst_ids_by_rel_business_obj_id(vec![child_inst.rel_business_obj_id.clone()], Some(true), funs, ctx).await?.pop()
                            {
                                // 更新业务主流程的artifact的状态为审批拒绝
                                Self::modify_inst_artifacts(
                                    &child_main_inst_id,
                                    &FlowInstArtifactsModifyReq {
                                        state: Some(FlowInstStateKind::Overrule),
                                        ..Default::default()
                                    },
                                    funs,
                                    ctx,
                                )
                                .await?;
                                Self::unsafe_modify_state(
                                    &FlowInstFilterReq {
                                        ids: Some(vec![child_main_inst_id.clone()]),
                                        with_sub: Some(true),
                                        ..Default::default()
                                    },
                                    &conf.unpass_status,
                                    funs,
                                    ctx,
                                )
                                .await?;
                            }
                        }
                    }
                    Self::abort(root_inst_id, &FlowInstAbortReq { message: "".to_string() }, funs, ctx).await?;
                    return Ok(());
                }
                for child_inst in pass_child_inst {
                    Self::transfer(
                        child_inst,
                        &FlowInstTransferReq {
                            flow_transition_id: next_transition.next_flow_transition_id.clone(),
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
                Self::transfer(
                    &root_inst,
                    &FlowInstTransferReq {
                        flow_transition_id: next_transition.next_flow_transition_id.clone(),
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
        } else {
            if end {
                Self::abort(root_inst_id, &FlowInstAbortReq { message: "".to_string() }, funs, ctx).await?;
            }
        }
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
                && specified_pass_guard_conf.check(approval_result.get(&FlowApprovalResultKind::Pass.to_string()).cloned().unwrap_or_default())
            {
                return Ok(true);
            }
            // 指定人拒绝，则拒绝
            if kind == FlowApprovalResultKind::Overrule
                && countersign_conf.specified_overrule_guard.unwrap_or(false)
                && countersign_conf.specified_overrule_guard_conf.is_some()
                && specified_overrule_guard_conf.check(approval_result.get(&FlowApprovalResultKind::Overrule.to_string()).cloned().unwrap_or_default())
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
            update_time: Set(Some(Utc::now())),
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
                    .and_where(Expr::col(flow_inst::Column::Code).ne("")),
            )
            .await?;
        let current_date = Utc::now();
        Ok(format!("SP{}{:0>2}{:0>2}{:0>5}", current_date.year(), current_date.month(), current_date.day(), count + 1).to_string())
    }

    // 获取需要更新的参数列表
    pub fn get_modify_vars(artifacts: &FlowInstArtifacts, state_ids: &[String]) -> HashMap<String, Value> {
        let mut vars_collect = HashMap::new();
        for from_state_id in state_ids {
            if let Some(form_state_vars) = artifacts.form_state_map.get(from_state_id) {
                for (key, value) in form_state_vars {
                    *vars_collect.entry(key.clone()).or_insert(json!({})) = value.clone();
                }
            }
        }

        vars_collect
    }

    pub async fn sync_status(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        pub struct FlowInstResult {
            id: String,
            state: String,
        }
        let mut page_num = 1;
        let page_size = 500;
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
                    if !search_result.records.is_empty() {
                        finish = false;
                    }
                    let insts = search_result
                        .records
                        .iter()
                        .map(|record| FlowInstResult {
                            id: record.ext.get("inst_id").unwrap_or(&json!("")).as_str().map(|s| s.to_string()).unwrap_or_default(),
                            state: record.ext.get("status").unwrap_or(&json!("")).as_str().map(|s| s.to_string()).unwrap_or_default(),
                        })
                        .filter(|record| !record.id.is_empty())
                        .collect_vec();
                    if !insts.is_empty() {
                        let flow_insts = Self::find_detail(insts.iter().map(|inst| inst.id.clone()).collect_vec(), None, None, funs, ctx).await?;
                        join_all(
                            flow_insts
                                .iter()
                                .map(|flow_inst| async {
                                    if let Some(current_state_name) = &flow_inst.current_state_name {
                                        if *current_state_name != insts.iter().find(|inst| inst.id == flow_inst.id).map(|inst| inst.state.clone()).unwrap_or_default() {
                                            let ctx = TardisContext {
                                                own_paths: flow_inst.own_paths.clone(),
                                                ..Default::default()
                                            };
                                            FlowSearchClient::modify_business_obj_search_ext_status(&flow_inst.rel_business_obj_id, &flow_inst.tag, current_state_name, funs, &ctx)
                                                .await
                                        } else {
                                            Ok(())
                                        }
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
        Ok(())
    }

    pub async fn get_search_item(flow_inst_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<FlowInstDetailInSearch> {
        let mut inst = Self::get(flow_inst_id, funs, ctx).await?;
        // 若为子审批流，则按父审批流为准
        if let Some(rel_inst_id) = &inst.rel_inst_id {
            if !rel_inst_id.is_empty() {
                inst = Self::get(rel_inst_id, funs, ctx).await?;
            }
        }
        let name = inst.create_vars.clone().unwrap_or_default().get("name").unwrap_or(&json!("")).as_str().unwrap_or("").to_string();
        Ok(FlowInstDetailInSearch {
            id: inst.id,
            code: inst.code.clone(),
            title: Some(format!("{} {}", inst.code, name.clone())),
            name: Some(name.clone()),
            content: Some(format!("{} {}", inst.code, name)),
            owner: inst.create_ctx.owner.clone(),
            own_paths: inst.own_paths.clone(),
            tag: Some(inst.tag),
            rel_business_obj_name: Some(name.clone()),
            rel_business_obj_id: Some(inst.rel_business_obj_id),
            current_state_id: Some(inst.current_state_id.clone()),
            current_state_name: inst.current_state_name.clone(),
            current_state_kind: inst.current_state_kind.clone(),
            finish_time: inst.finish_time,
            op_time: inst.update_time,
            state: inst.artifacts.as_ref().map(|artifacts| artifacts.state.unwrap_or_default()),
            rel_transition: inst.rel_transition.clone().map(FlowModelRelTransitionKind::from),
            his_operators: inst.artifacts.as_ref().map(|artifacts| artifacts.his_operators.clone().unwrap_or_default()),
            curr_operators: inst.artifacts.as_ref().map(|artifacts| artifacts.curr_operators.clone().unwrap_or_default()),
            curr_referral: inst
                .artifacts
                .as_ref()
                .map(|artifacts| artifacts.referral_map.clone().unwrap_or_default().get(&inst.current_state_id).cloned().unwrap_or_default().keys().cloned().collect_vec()),
            create_time: Some(inst.create_time),
            update_time: inst.update_time,
            rel_inst_id: inst.rel_inst_id.clone(),
        })
    }

    pub async fn stat_inst_count(app_ids: &[String], filter: &FlowInstFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<HashMap<String, u64>> {
        let mut result = HashMap::new();
        for app_id in app_ids {
            let mock_ctx = TardisContext {
                own_paths: format!("{}/{}", ctx.own_paths, app_id),
                ..ctx.clone()
            };
            let mut query = Query::select();
            Self::package_ext_query(&mut query, filter, funs, &mock_ctx).await?;
            let total_size = funs.db().count(&query).await?;
            result.insert(app_id.clone(), total_size);
        }
        Ok(result)
    }

    pub async fn sync_deleted_instances(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<String>> {
        let tag_search_map = FlowSearchClient::get_tag_search_map();
        let mut result = vec![];
        let unfinished_insts = Self::find_items(
            &FlowInstFilterReq {
                finish: Some(false),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        let max_size = unfinished_insts.len();
        let mut page = 0;
        let page_size = 500;
        loop {
            let current_insts = &unfinished_insts[((page * page_size).min(max_size))..(((page + 1) * page_size).min(max_size))];
            if current_insts.is_empty() {
                break;
            }
            let mut inst_group_by_tag = HashMap::new();
            for current_inst in current_insts {
                inst_group_by_tag
                    .entry(current_inst.tag.clone())
                    .and_modify(|inst_ids: &mut Vec<FlowInstSummaryResult>| inst_ids.push(current_inst.clone()))
                    .or_insert(vec![current_inst.clone()]);
            }
            for (tag, insts) in inst_group_by_tag {
                if let Some((table, kind)) = tag_search_map.get(&tag) {
                    let exist_obj_ids = FlowSearchClient::search(
                        &SearchItemSearchReq {
                            tag: table.clone(),
                            ctx: SearchItemSearchCtxReq { ..Default::default() },
                            query: SearchItemQueryReq {
                                keys: Some(insts.iter().map(|inst| TrimString(inst.rel_business_obj_id.clone())).collect_vec()),
                                kinds: Some(vec![kind.clone()]),
                                ..Default::default()
                            },
                            adv_by_or: None,
                            adv_query: None,
                            sort: None,
                            page: SearchItemSearchPageReq {
                                number: 1,
                                size: 100,
                                fetch_total: false,
                            },
                        },
                        funs,
                        ctx,
                    )
                    .await?
                    .map(|resp| resp.records)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|item| item.key)
                    .collect_vec();
                    let deleted_insts = insts.into_iter().filter(|inst| !exist_obj_ids.contains(&inst.rel_business_obj_id)).collect_vec();
                    result.extend(deleted_insts.iter().map(|inst| inst.id.clone()).collect_vec());
                    join_all(
                        deleted_insts.iter().map(|inst| async { FlowInstServ::abort(&inst.id, &FlowInstAbortReq { message: "".to_string() }, funs, ctx).await }).collect_vec(),
                    )
                    .await
                    .into_iter()
                    .collect::<TardisResult<Vec<()>>>()?;
                }
            }
            tardis::tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            page += 1;
        }

        let unabort_insts = Self::find_items(
            &FlowInstFilterReq {
                finish: Some(true),
                finish_abort: Some(false),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        let max_size = unabort_insts.len();
        let mut page = 0;
        let page_size = 500;
        loop {
            let current_insts = &unabort_insts[((page * page_size).min(max_size))..(((page + 1) * page_size).min(max_size))];
            if current_insts.is_empty() {
                break;
            }
            let mut inst_group_by_tag = HashMap::new();
            for current_inst in current_insts {
                inst_group_by_tag
                    .entry(current_inst.tag.clone())
                    .and_modify(|inst_ids: &mut Vec<FlowInstSummaryResult>| inst_ids.push(current_inst.clone()))
                    .or_insert(vec![current_inst.clone()]);
            }
            for (tag, insts) in inst_group_by_tag {
                if let Some((table, kind)) = tag_search_map.get(&tag) {
                    let exist_obj_ids = FlowSearchClient::search(
                        &SearchItemSearchReq {
                            tag: table.clone(),
                            ctx: SearchItemSearchCtxReq { ..Default::default() },
                            query: SearchItemQueryReq {
                                keys: Some(insts.iter().map(|inst| TrimString(inst.rel_business_obj_id.clone())).collect_vec()),
                                kinds: Some(vec![kind.clone()]),
                                ..Default::default()
                            },
                            adv_by_or: None,
                            adv_query: None,
                            sort: None,
                            page: SearchItemSearchPageReq {
                                number: 1,
                                size: 100,
                                fetch_total: false,
                            },
                        },
                        funs,
                        ctx,
                    )
                    .await?
                    .map(|resp| resp.records)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|item| item.key)
                    .collect_vec();
                    let deleted_insts = insts.into_iter().filter(|inst| !exist_obj_ids.contains(&inst.rel_business_obj_id)).collect_vec();
                    result.extend(deleted_insts.iter().map(|inst| inst.id.clone()).collect_vec());
                    join_all(
                        deleted_insts.iter().map(|inst| async { FlowInstServ::abort(&inst.id, &FlowInstAbortReq { message: "".to_string() }, funs, ctx).await }).collect_vec(),
                    )
                    .await
                    .into_iter()
                    .collect::<TardisResult<Vec<()>>>()?;
                }
            }
            tardis::tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            page += 1;
        }

        Ok(result)
    }
}
