use std::{collections::HashMap, vec};

use bios_basic::rbum::{
    dto::{
        rbum_filer_dto::{RbumBasicFilterReq, RbumItemRelFilterReq},
        rbum_item_dto::{RbumItemKernelAddReq, RbumItemKernelModifyReq},
    },
    helper::rbum_scope_helper,
    rbum_enumeration::{RbumRelFromKind, RbumScopeLevelKind},
    serv::rbum_item_serv::RbumItemCrudOperation,
};
use itertools::Itertools;
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    db::sea_orm::{
        sea_query::{Alias, Cond, Expr, Query, SelectStatement},
        EntityName, Set,
    },
    futures::future::join_all,
    log::error,
    serde_json::json,
    tokio,
    web::web_resp::TardisPage,
    TardisFuns, TardisFunsInst,
};

use crate::{
    domain::{flow_model, flow_transition},
    dto::{
        flow_model_dto::{
            FlowModelAddReq, FlowModelAggResp, FlowModelAssociativeOperationKind, FlowModelBindNewStateReq, FlowModelBindStateReq, FlowModelDetailResp, FlowModelFilterReq,
            FlowModelFindRelStateResp, FlowModelKind, FlowModelModifyReq, FlowModelRelTransitionExt, FlowModelStatus, FlowModelSummaryResp, FlowModelSyncModifiedFieldReq,
            FlowModelUnbindStateReq,
        },
        flow_model_version_dto::{
            FlowModelVersionAddReq, FlowModelVersionBindState, FlowModelVersionDetailResp, FlowModelVersionFilterReq, FlowModelVersionModifyReq, FlowModelVersionModifyState,
            FlowModelVesionState,
        },
        flow_state_dto::{FLowStateIdAndName, FlowStateAddReq, FlowStateAggResp, FlowStateKind, FlowStateModifyReq, FlowStateRelModelExt, FlowStateVar, FlowSysStateKind},
        flow_transition_dto::{
            FlowTransitionAddReq, FlowTransitionDetailResp, FlowTransitionInitInfo, FlowTransitionModifyReq, FlowTransitionPostActionInfo, FlowTransitionSortStatesReq,
        },
    },
    flow_config::FlowBasicInfoManager,
    flow_constants,
};
use async_trait::async_trait;

use super::{
    clients::{
        log_client::{FlowLogClient, LogParamContent, LogParamTag},
        search_client::FlowSearchClient,
    },
    flow_inst_serv::FlowInstServ,
    flow_model_version_serv::FlowModelVersionServ,
    flow_rel_serv::{FlowRelKind, FlowRelServ},
    flow_state_serv::FlowStateServ,
    flow_transition_serv::FlowTransitionServ,
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
            icon: Set(add_req.icon.clone().unwrap_or_default()),
            info: Set(add_req.info.clone().unwrap_or_default()),
            current_version_id: Set(add_req.current_version_id.clone().unwrap_or_default()),
            kind: Set(add_req.kind.clone()),
            status: Set(add_req.status.clone()),
            tag: Set(add_req.tag.clone()),
            rel_model_id: Set(add_req.rel_model_id.clone().unwrap_or_default()),
            template: Set(add_req.template),
            main: Set(add_req.main),
            ..Default::default()
        })
    }

    async fn before_add_item(add_req: &mut FlowModelAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if let Some(rel_transition_ids) = &add_req.rel_transition_ids {
            for rel_transition_id in rel_transition_ids {
                if Self::find_one_detail_item(
                    &FlowModelFilterReq {
                        basic: RbumBasicFilterReq {
                            enabled: Some(true),
                            ..Default::default()
                        },
                        main: Some(false),
                        rel_template_id: add_req.rel_template_ids.clone().unwrap_or_default().first().cloned(),
                        tags: Some(vec![add_req.tag.clone().unwrap_or_default()]),
                        rel: Some(RbumItemRelFilterReq {
                            optional: false,
                            rel_by_from: true,
                            tag: Some(FlowRelKind::FlowModelTransition.to_string()),
                            from_rbum_kind: Some(RbumRelFromKind::Item),
                            rel_item_id: Some(rel_transition_id.clone()),
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                    funs,
                    ctx,
                )
                .await?
                .is_some()
                {
                    return Err(funs.err().not_found(&Self::get_obj_name(), "before_add_item", "The model is not repeatable", "400-flow-model-duplicate"));
                }
            }
        }

        Ok(())
    }

    async fn after_add_item(flow_model_id: &str, add_req: &mut FlowModelAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if let Some(rel_transition_ids) = add_req.rel_transition_ids.clone() {
            let transitions = FlowTransitionServ::find_detail_items(rel_transition_ids.clone(), None, None, funs, ctx).await?;
            for rel_transition_id in rel_transition_ids {
                let mut ext = FlowModelRelTransitionExt {
                    id: rel_transition_id.clone(),
                    ..Default::default()
                };
                if let Some(transition) = transitions.iter().find(|tran| tran.id == rel_transition_id) {
                    ext.name = transition.name.clone();
                    ext.from_flow_state_name = transition.from_flow_state_name.clone();
                    ext.to_flow_state_name = Some(transition.to_flow_state_name.clone());
                }
                FlowRelServ::add_simple_rel(
                    &FlowRelKind::FlowModelTransition,
                    flow_model_id,
                    &rel_transition_id,
                    None,
                    None,
                    true,
                    true,
                    Some(json!(ext).to_string()),
                    funs,
                    ctx,
                )
                .await?;
            }
        }

        let mut add_version = if let Some(mut add_version) = add_req.add_version.clone() {
            add_version.rel_model_id = Some(flow_model_id.to_string());
            add_version
        } else if add_req.main {
            FlowModelVersionAddReq {
                name: add_req.name.clone(),
                rel_model_id: Some(flow_model_id.to_string()),
                bind_states: None,
                status: FlowModelVesionState::Enabled,
                scope_level: add_req.scope_level.clone(),
                disabled: add_req.disabled,
            }
        } else {
            let start_state_id = TardisFuns::field.nanoid();
            let finish_state_id = TardisFuns::field.nanoid();
            FlowModelVersionAddReq {
                name: add_req.name.clone(),
                rel_model_id: Some(flow_model_id.to_string()),
                // 初始化时增加开始结束两个节点
                bind_states: Some(vec![
                    FlowModelVersionBindState {
                        bind_new_state: Some(FlowModelBindNewStateReq {
                            new_state: FlowStateAddReq {
                                id: Some(start_state_id.clone().into()),
                                name: Some("开始".into()),
                                sys_state: FlowSysStateKind::Start,
                                state_kind: Some(FlowStateKind::Start),
                                tags: Some(vec![add_req.tag.clone().unwrap_or_default()]),
                                main: Some(false),
                                ..Default::default()
                            },
                            ext: FlowStateRelModelExt { sort: 0, show_btns: None },
                        }),
                        add_transitions: Some(vec![FlowTransitionAddReq {
                            name: Some("开始".into()),
                            from_flow_state_id: start_state_id.clone(),
                            to_flow_state_id: finish_state_id.clone(),
                            transfer_by_auto: Some(true),
                            ..Default::default()
                        }]),
                        is_init: true,
                        ..Default::default()
                    },
                    FlowModelVersionBindState {
                        bind_new_state: Some(FlowModelBindNewStateReq {
                            new_state: FlowStateAddReq {
                                id: Some(finish_state_id.clone().into()),
                                name: Some("结束".into()),
                                sys_state: FlowSysStateKind::Finish,
                                state_kind: Some(FlowStateKind::Finish),
                                tags: Some(vec![add_req.tag.clone().unwrap_or_default()]),
                                main: Some(false),
                                ..Default::default()
                            },
                            ext: FlowStateRelModelExt { sort: 1, show_btns: None },
                        }),
                        is_init: false,
                        ..Default::default()
                    },
                ]),
                status: FlowModelVesionState::Editing,
                scope_level: add_req.scope_level.clone(),
                disabled: add_req.disabled,
            }
        };

        let version_id = FlowModelVersionServ::add_item(&mut add_version, funs, ctx).await?;
        if add_version.status == FlowModelVesionState::Enabled {
            FlowModelVersionServ::enable_version(&version_id, funs, ctx).await?;
            Self::modify_item(
                flow_model_id,
                &mut FlowModelModifyReq {
                    current_version_id: Some(version_id),
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?;
        }

        if let Some(rel_template_ids) = &add_req.rel_template_ids {
            join_all(
                rel_template_ids
                    .iter()
                    .map(|rel_template_id| async {
                        FlowRelServ::add_simple_rel(&FlowRelKind::FlowModelTemplate, flow_model_id, rel_template_id, None, None, false, true, None, funs, ctx).await
                    })
                    .collect_vec(),
            )
            .await
            .into_iter()
            .collect::<TardisResult<Vec<()>>>()?;
        }
        if add_req.template && add_req.main && add_req.rel_model_id.clone().map_or(true, |id| id.is_empty()) {
            FlowSearchClient::async_add_or_modify_model_search(flow_model_id, Box::new(false), funs, ctx).await?;
            FlowLogClient::add_ctx_task(
                LogParamTag::DynamicLog,
                Some(flow_model_id.to_string()),
                LogParamContent {
                    subject: Some("工作流模板".to_string()),
                    name: Some(add_req.name.to_string()),
                    sub_kind: Some("flow_template".to_string()),
                    ..Default::default()
                },
                Some(json!({
                    "name": add_req.name.to_string(),
                    "info": add_req.info.clone().unwrap_or_default(),
                    "rel_template_ids":add_req.rel_template_ids.clone().unwrap_or_default(),
                    "scope_level": add_req.scope_level.clone(),
                    "tag": add_req.tag.clone().unwrap_or_default(),
                })),
                Some("dynamic_log_tenant_config".to_string()),
                Some("新建".to_string()),
                rbum_scope_helper::get_path_item(RbumScopeLevelKind::L1.to_int(), &ctx.own_paths),
                true,
                ctx,
                false,
            )
            .await?;
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
        if modify_req.icon.is_none()
            && modify_req.info.is_none()
            && modify_req.tag.is_none()
            && modify_req.rel_model_id.is_none()
            && modify_req.current_version_id.is_none()
            && modify_req.status.is_none()
        {
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
        if let Some(tag) = &modify_req.tag {
            flow_model.tag = Set(Some(tag.clone()));
        }
        if let Some(status) = &modify_req.status {
            flow_model.status = Set(status.clone());
        }
        if let Some(rel_model_id) = &modify_req.rel_model_id {
            flow_model.rel_model_id = Set(rel_model_id.clone());
        }
        if let Some(current_version_id) = &modify_req.current_version_id {
            flow_model.current_version_id = Set(current_version_id.clone());
        }
        Ok(Some(flow_model))
    }

    async fn before_modify_item(flow_model_id: &str, modify_req: &mut FlowModelModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
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
                "before_modify_item",
                "The own_paths of current mode isn't the own_paths of ctx",
                "404-flow-model-not-found",
            ));
        }
        if let Some(mut modify_version) = modify_req.modify_version.clone() {
            FlowModelVersionServ::modify_item(&current_model.current_version_id, &mut modify_version, funs, ctx).await?;
        }
        if modify_req.status == Some(FlowModelStatus::Enabled) && current_model.current_version_id.is_empty() {
            modify_req.current_version_id = FlowModelVersionServ::find_id_items(
                &FlowModelVersionFilterReq {
                    rel_model_ids: Some(vec![flow_model_id.to_string()]),
                    ..Default::default()
                },
                Some(true),
                None,
                funs,
                ctx,
            )
            .await?
            .first()
            .cloned();
        }
        Ok(())
    }

    async fn after_modify_item(flow_model_id: &str, modify_req: &mut FlowModelModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let model_detail = Self::get_item(flow_model_id, &FlowModelFilterReq::default(), funs, ctx).await?;
        let rel_version_ids = FlowModelVersionServ::find_id_items(
            &FlowModelVersionFilterReq {
                rel_model_ids: Some(vec![flow_model_id.to_string()]),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        if modify_req.name.is_some() {
            // 同步修改名称到版本
            for model_id in FlowModelVersionServ::find_id_items(
                &FlowModelVersionFilterReq {
                    rel_model_ids: Some(vec![flow_model_id.to_string()]),
                    ..Default::default()
                },
                None,
                None,
                funs,
                ctx,
            )
            .await?
            {
                FlowModelVersionServ::modify_item(
                    &model_id,
                    &mut FlowModelVersionModifyReq {
                        name: modify_req.name.clone(),
                        ..Default::default()
                    },
                    funs,
                    ctx,
                )
                .await?;
            }
        }
        if modify_req.status == Some(FlowModelStatus::Enabled) && model_detail.current_version_id.is_empty() {
            return Err(funs.err().internal_error("flow_model_serv", "after_modify_item", "Current model is not enabled", "500-flow-model-prohibit-enabled"));
        }
        if modify_req.disabled == Some(true) {
            for rel_version_id in &rel_version_ids {
                FlowInstServ::unsafe_abort_inst(rel_version_id, funs, ctx).await?;
            }
        }
        if let Some(rel_template_ids) = &modify_req.rel_template_ids {
            join_all(
                FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowModelTemplate, flow_model_id, None, None, funs, ctx)
                    .await?
                    .into_iter()
                    .map(|rel| async move { FlowRelServ::delete_simple_rel(&FlowRelKind::FlowModelTemplate, flow_model_id, &rel.rel_id, funs, ctx).await })
                    .collect_vec(),
            )
            .await
            .into_iter()
            .collect::<TardisResult<Vec<()>>>()?;
            join_all(
                rel_template_ids
                    .iter()
                    .map(|rel_template_id| async {
                        FlowRelServ::add_simple_rel(&FlowRelKind::FlowModelTemplate, flow_model_id, rel_template_id, None, None, false, true, None, funs, ctx).await
                    })
                    .collect_vec(),
            )
            .await
            .into_iter()
            .collect::<TardisResult<Vec<()>>>()?;
        }
        if model_detail.template && model_detail.main && model_detail.rel_model_id.is_empty() {
            FlowSearchClient::async_add_or_modify_model_search(flow_model_id, Box::new(true), funs, ctx).await?;
            FlowLogClient::add_ctx_task(
                LogParamTag::DynamicLog,
                Some(flow_model_id.to_string()),
                LogParamContent {
                    subject: Some("工作流模板".to_string()),
                    name: Some(model_detail.name.clone()),
                    sub_kind: Some("flow_template".to_string()),
                    ..Default::default()
                },
                Some(json!({
                    "name": model_detail.name.to_string(),
                    "info": model_detail.info.clone(),
                    "rel_template_ids":model_detail.rel_template_ids.clone(),
                    "scope_level": model_detail.scope_level.clone(),
                    "tag": model_detail.tag.clone(),
                })),
                Some("dynamic_log_tenant_config".to_string()),
                Some("编辑".to_string()),
                rbum_scope_helper::get_path_item(RbumScopeLevelKind::L1.to_int(), &ctx.own_paths),
                true,
                ctx,
                false,
            )
            .await?;
        }

        // 同步修改所有引用的下级模型
        if model_detail.template {
            let child_models = Self::find_detail_items(
                &FlowModelFilterReq {
                    basic: RbumBasicFilterReq {
                        own_paths: Some("".to_string()),
                        with_sub_own_paths: true,
                        ..Default::default()
                    },
                    rel_model_ids: Some(vec![flow_model_id.to_string()]),
                    ..Default::default()
                },
                None,
                None,
                funs,
                ctx,
            )
            .await?;
            for child_model in child_models {
                Self::sync_child_model(&child_model, &model_detail, modify_req, funs, ctx).await?;
            }
        }

        // 同步scope_level和disabled字段到相关的version数据
        if modify_req.scope_level.is_some() || modify_req.disabled.is_some() {
            let mut version_modify_req = FlowModelVersionModifyReq::default();
            if let Some(scope_level) = modify_req.scope_level.clone() {
                version_modify_req.scope_level = Some(scope_level);
            }
            if let Some(disabled) = modify_req.disabled {
                version_modify_req.disabled = Some(disabled);
            }
            for rel_version_id in &rel_version_ids {
                let mut version_modify_req_clone = version_modify_req.clone();
                FlowModelVersionServ::modify_item(rel_version_id, &mut version_modify_req_clone, funs, ctx).await?;
            }
        }

        Ok(())
    }

    async fn before_delete_item(flow_model_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<FlowModelDetailResp>> {
        if !Self::find_id_items(
            &FlowModelFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    own_paths: Some("".to_string()),
                    ..Default::default()
                },
                rel_model_ids: Some(vec![flow_model_id.to_string()]),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?
        .is_empty()
        {
            return Err(funs.err().not_found(&Self::get_obj_name(), "delete_item", "the model prohibit delete", "500-flow-model-prohibit-delete"));
        }
        let detail = Self::get_item(flow_model_id, &FlowModelFilterReq::default(), funs, ctx).await?;

        join_all(
            FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowModelTemplate, flow_model_id, None, None, funs, ctx)
                .await?
                .into_iter()
                .map(|rel| async move { FlowRelServ::delete_simple_rel(&FlowRelKind::FlowModelTemplate, flow_model_id, &rel.rel_id, funs, ctx).await })
                .collect_vec(),
        )
        .await
        .into_iter()
        .collect::<TardisResult<Vec<()>>>()?;
        join_all(
            FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowModelTransition, flow_model_id, None, None, funs, ctx)
                .await?
                .into_iter()
                .map(|rel| async move { FlowRelServ::delete_simple_rel(&FlowRelKind::FlowModelTransition, flow_model_id, &rel.rel_id, funs, ctx).await })
                .collect_vec(),
        )
        .await
        .into_iter()
        .collect::<TardisResult<Vec<()>>>()?;
        let rel_version_ids = FlowModelVersionServ::find_id_items(
            &FlowModelVersionFilterReq {
                rel_model_ids: Some(vec![flow_model_id.to_string()]),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        for rel_version_id in &rel_version_ids {
            FlowInstServ::unsafe_abort_inst(rel_version_id, funs, ctx).await?;
        }
        for rel_version_id in &rel_version_ids {
            FlowModelVersionServ::delete_item(rel_version_id, funs, ctx).await?;
        }

        Ok(Some(detail))
    }

    async fn after_delete_item(flow_model_id: &str, detail: &Option<FlowModelDetailResp>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if let Some(detail) = detail {
            if detail.template && detail.rel_model_id.is_empty() {
                FlowSearchClient::async_delete_model_search(flow_model_id.to_string(), funs, ctx).await?;
                FlowLogClient::add_ctx_task(
                    LogParamTag::DynamicLog,
                    Some(flow_model_id.to_string()),
                    LogParamContent {
                        subject: Some("工作流模板".to_string()),
                        name: Some(detail.name.clone()),
                        sub_kind: Some("flow_template".to_string()),
                        ..Default::default()
                    },
                    Some(json!({
                        "name": detail.name.to_string(),
                        "info": detail.info.clone(),
                        "rel_template_ids":detail.rel_template_ids.clone(),
                        "scope_level": detail.scope_level.clone(),
                        "tag": detail.tag.clone(),
                    })),
                    Some("dynamic_log_tenant_config".to_string()),
                    Some("删除".to_string()),
                    rbum_scope_helper::get_path_item(RbumScopeLevelKind::L1.to_int(), &ctx.own_paths),
                    true,
                    ctx,
                    false,
                )
                .await?;
            }
        }
        Ok(())
    }

    async fn package_ext_query(query: &mut SelectStatement, _: bool, filter: &FlowModelFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        query
            .column((flow_model::Entity, flow_model::Column::Icon))
            .column((flow_model::Entity, flow_model::Column::Info))
            .column((flow_model::Entity, flow_model::Column::Template))
            .column((flow_model::Entity, flow_model::Column::Main))
            .column((flow_model::Entity, flow_model::Column::RelModelId))
            .column((flow_model::Entity, flow_model::Column::Tag))
            .column((flow_model::Entity, flow_model::Column::Kind))
            .column((flow_model::Entity, flow_model::Column::Status))
            .column((flow_model::Entity, flow_model::Column::CurrentVersionId))
            .expr_as(Expr::val("".to_string()), Alias::new("init_state_id"))
            .expr_as(Expr::val(json! {()}), Alias::new("transitions"))
            .expr_as(Expr::val(json! {()}), Alias::new("states"))
            .expr_as(Expr::val(json! {()}), Alias::new("rel_transition"))
            .expr_as(Expr::val(vec!["".to_string()]), Alias::new("rel_template_ids"));
        if let Some(tags) = filter.tags.clone() {
            query.and_where(Expr::col(flow_model::Column::Tag).is_in(tags));
        }
        if let Some(template) = filter.template {
            query.and_where(Expr::col(flow_model::Column::Template).eq(template));
        }
        if let Some(main) = filter.main {
            query.and_where(Expr::col(flow_model::Column::Main).eq(main));
        }
        if let Some(own_paths) = filter.own_paths.clone() {
            query.and_where(Expr::col((flow_model::Entity, flow_model::Column::OwnPaths)).is_in(own_paths));
        }
        if let Some(rel_model_ids) = filter.rel_model_ids.clone() {
            query.and_where(Expr::col(flow_model::Column::RelModelId).is_in(rel_model_ids));
        }
        if let Some(kinds) = filter.kinds.clone() {
            query.and_where(Expr::col(flow_model::Column::Kind).is_in(kinds));
        }
        if let Some(status) = filter.status.clone() {
            query.and_where(Expr::col(flow_model::Column::Status).eq(status));
        }
        if let Some(rel_template_id) = filter.rel_template_id.clone() {
            let rel_model_ids =
                FlowRelServ::find_to_simple_rels(&FlowRelKind::FlowModelTemplate, &rel_template_id, None, None, funs, ctx).await?.into_iter().map(|rel| rel.rel_id).collect_vec();
            query.and_where(Expr::col((flow_model::Entity, flow_model::Column::Id)).is_in(rel_model_ids));
        }
        Ok(())
    }

    async fn get_item(flow_model_id: &str, filter: &FlowModelFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<FlowModelDetailResp> {
        let mut flow_model = Self::do_get_item(flow_model_id, filter, funs, ctx).await?;

        if !flow_model.current_version_id.is_empty() {
            let flow_transitions = FlowTransitionServ::find_transitions(&flow_model.current_version_id, filter.specified_state_ids.clone(), funs, ctx).await?;
            flow_model.transitions = Some(TardisFuns::json.obj_to_json(&flow_transitions)?);

            let current_version = FlowModelVersionServ::get_item(
                &flow_model.current_version_id,
                &FlowModelVersionFilterReq {
                    basic: RbumBasicFilterReq {
                        ids: None,
                        ..filter.basic.clone()
                    },
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?;
            flow_model.states = Some(current_version.states.unwrap_or_default());

            let rel_template_ids =
                FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowModelTemplate, flow_model_id, None, None, funs, ctx).await?.into_iter().map(|rel| rel.rel_id).collect_vec();
            flow_model.rel_template_ids = rel_template_ids;

            flow_model.init_state_id = current_version.init_state_id;
        }
        let rel_transition =
            FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowModelTransition, flow_model_id, None, None, funs, ctx).await?.into_iter().map(|rel| rel.ext).collect_vec().pop();
        flow_model.rel_transition = rel_transition.map(|rel_transition| TardisFuns::json.str_to_json(&rel_transition).unwrap_or_default());

        Ok(flow_model)
    }

    async fn find_items(
        filter: &FlowModelFilterReq,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<FlowModelSummaryResp>> {
        let mut res = Self::do_find_items(filter, desc_sort_by_create, desc_sort_by_update, funs, ctx).await?;
        for item in res.iter_mut() {
            if !item.current_version_id.is_empty() {
                let version = FlowModelVersionServ::get_item(
                    &item.current_version_id,
                    &FlowModelVersionFilterReq {
                        basic: RbumBasicFilterReq {
                            ids: None,
                            ..filter.basic.clone()
                        },
                        ..Default::default()
                    },
                    funs,
                    ctx,
                )
                .await?;
                item.init_state_id = version.init_state_id;

                let states = FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowModelState, &item.current_version_id, None, None, funs, ctx)
                    .await?
                    .into_iter()
                    .map(|rel| FLowStateIdAndName {
                        id: rel.rel_id,
                        name: rel.rel_name,
                    })
                    .collect_vec();
                item.states = TardisFuns::json.obj_to_json(&states).unwrap_or_default();
            }

            let rel_transition =
                FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowModelTransition, &item.id, None, None, funs, ctx).await?.into_iter().map(|rel| rel.ext).collect_vec().pop();
            item.rel_transition = rel_transition.map(|rel_transition| TardisFuns::json.str_to_json(&rel_transition).unwrap_or_default());
        }
        Ok(res)
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
            let flow_transitions = FlowTransitionServ::find_transitions(&flow_model.current_version_id, filter.specified_state_ids.clone(), funs, ctx).await?;
            flow_model.transitions = Some(TardisFuns::json.obj_to_json(&flow_transitions)?);
            let rel_transition = FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowModelTransition, &flow_model.id, None, None, funs, ctx)
                .await?
                .into_iter()
                .map(|rel| rel.ext)
                .collect_vec()
                .pop();
            flow_model.rel_transition = rel_transition.map(|rel_transition| TardisFuns::json.str_to_json(&rel_transition).unwrap_or_default());
            flow_model.rel_template_ids =
                FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowModelTemplate, &flow_model.id, None, None, funs, ctx).await?.into_iter().map(|rel| rel.rel_id).collect_vec();
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
            if !flow_model.current_version_id.is_empty() {
                let flow_transitions = FlowTransitionServ::find_transitions(&flow_model.current_version_id, filter.specified_state_ids.clone(), funs, ctx).await?;
                flow_model.transitions = Some(TardisFuns::json.obj_to_json(&flow_transitions)?);

                let current_version = FlowModelVersionServ::get_item(
                    &flow_model.current_version_id,
                    &FlowModelVersionFilterReq {
                        basic: RbumBasicFilterReq {
                            ids: None,
                            ..filter.basic.clone()
                        },
                        ..Default::default()
                    },
                    funs,
                    ctx,
                )
                .await?;
                flow_model.states = Some(current_version.states.unwrap_or_default());

                let rel_template_ids = FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowModelTemplate, &flow_model.id, None, None, funs, ctx)
                    .await?
                    .into_iter()
                    .map(|rel| rel.rel_id)
                    .collect_vec();
                flow_model.rel_template_ids = rel_template_ids;

                flow_model.init_state_id = current_version.init_state_id;
            }
            let rel_transition = FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowModelTransition, &flow_model.id, None, None, funs, ctx)
                .await?
                .into_iter()
                .map(|rel| rel.ext)
                .collect_vec()
                .pop();
            flow_model.rel_transition = rel_transition.map(|rel_transition| TardisFuns::json.str_to_json(&rel_transition).unwrap_or_default());
        }

        Ok(flow_models)
    }
}

impl FlowModelServ {
    pub async fn init_model(
        tag: &str,
        init_state_id: String,
        state_ids: Vec<String>,
        model_name: &str,
        transitions: Vec<FlowTransitionInitInfo>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<String> {
        // transitions
        let mut add_transitions = vec![];
        for transition in transitions {
            add_transitions.push(FlowTransitionAddReq::try_from(transition)?);
        }
        let mut bind_states = vec![];
        // states FlowModelVersionBindState
        for (i, state_id) in state_ids.into_iter().enumerate() {
            bind_states.push(FlowModelVersionBindState {
                exist_state: Some(FlowModelBindStateReq {
                    state_id: state_id.clone(),
                    ext: FlowStateRelModelExt { sort: i as i64, show_btns: None },
                }),
                add_transitions: Some(add_transitions.clone().into_iter().filter(|tran| tran.from_flow_state_id == state_id).collect_vec()),
                is_init: state_id == init_state_id,
                ..Default::default()
            });
        }

        // add model
        let model_id = Self::add_item(
            &mut FlowModelAddReq {
                name: model_name.into(),
                kind: FlowModelKind::AsTemplateAndAsModel,
                status: FlowModelStatus::Enabled,
                add_version: Some(FlowModelVersionAddReq {
                    name: model_name.into(),
                    rel_model_id: None,
                    bind_states: Some(bind_states),
                    status: FlowModelVesionState::Enabled,
                    scope_level: Some(RbumScopeLevelKind::Root),
                    disabled: None,
                }),
                rel_template_ids: None,
                rel_transition_ids: None,
                current_version_id: None,
                icon: None,
                info: None,
                tag: Some(tag.to_string()),
                scope_level: Some(RbumScopeLevelKind::Root),
                disabled: None,
                template: true,
                main: true,
                rel_model_id: None,
            },
            funs,
            ctx,
        )
        .await?;

        Ok(model_id)
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

    pub async fn get_item_detail_aggs(flow_model_id: &str, is_state_detail: bool, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<FlowModelAggResp> {
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

        let mut states = Vec::new();
        if is_state_detail {
            // find rel state
            for state in model_detail.states() {
                let state_detail = FlowStateAggResp {
                    id: state.id.clone(),
                    name: state.name.clone(),
                    ext: state.ext.clone(),
                    state_kind: state.state_kind,
                    kind_conf: state.kind_conf,
                    sys_state: state.sys_state,
                    tags: state.tags,
                    scope_level: state.scope_level,
                    disabled: state.disabled,
                    is_init: model_detail.init_state_id == state.id,
                    main: state.main,
                    transitions: model_detail
                        .transitions()
                        .into_iter()
                        .filter(|transition| transition.from_flow_state_id == state.id.clone())
                        .map(|transition| {
                            let mut action_by_post_changes = vec![];
                            for action_by_post_change in transition.action_by_post_changes() {
                                if action_by_post_change.is_edit.is_none() {
                                    action_by_post_changes.push(FlowTransitionPostActionInfo {
                                        is_edit: Some(false), // 默认为不可编辑，若用户需要编辑，可手动处理数据
                                        ..action_by_post_change.clone()
                                    });
                                } else {
                                    action_by_post_changes.push(FlowTransitionPostActionInfo { ..action_by_post_change.clone() });
                                }
                            }
                            FlowTransitionDetailResp {
                                action_by_post_changes: TardisFuns::json.obj_to_json(&action_by_post_changes).unwrap_or_default(),
                                ..transition.clone()
                            }
                        })
                        .collect_vec(),
                };
                states.push(state_detail);
            }
        }

        Ok(FlowModelAggResp {
            id: model_detail.id.clone(),
            name: model_detail.name,
            icon: model_detail.icon,
            info: model_detail.info,
            init_state_id: model_detail.init_state_id,
            template: model_detail.template,
            current_version_id: model_detail.current_version_id,
            edit_version_id: FlowModelVersionServ::find_one_item(
                &FlowModelVersionFilterReq {
                    rel_model_ids: Some(vec![model_detail.id.clone()]),
                    status: Some(vec![FlowModelVesionState::Editing]),
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?
            .map(|version| version.id)
            .unwrap_or_default(),
            rel_model_id: model_detail.rel_model_id,
            rel_template_ids: FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowModelTemplate, &model_detail.id, None, None, funs, ctx)
                .await?
                .into_iter()
                .map(|rel| rel.rel_id)
                .collect_vec(),
            states,
            own_paths: model_detail.own_paths,
            owner: model_detail.owner,
            create_time: model_detail.create_time,
            update_time: model_detail.update_time,
            tag: model_detail.tag,
            scope_level: model_detail.scope_level,
            disabled: model_detail.disabled,
            main: model_detail.main,
        })
    }

    // Find the rel models.
    pub async fn find_rel_models(
        template_id: Option<String>,
        main: bool,
        tags: Option<Vec<String>>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<FlowModelSummaryResp>> {
        let global_ctx = TardisContext {
            own_paths: "".to_string(),
            ..ctx.clone()
        };

        let filter_ids = if template_id.is_none() {
            if let Some(app_id) = Self::get_app_id_by_ctx(ctx) {
                Some(FlowRelServ::find_model_ids_by_app_id(&app_id, funs, ctx).await?.unwrap_or_default())
            } else {
                Some(vec![])
            }
        } else {
            None
        };
        let mut filter = FlowModelFilterReq {
            basic: RbumBasicFilterReq {
                ids: filter_ids,
                ignore_scope: true,
                with_sub_own_paths: true,
                enabled: Some(true),
                ..Default::default()
            },
            tags,
            main: Some(main),
            status: Some(FlowModelStatus::Enabled),
            rel: FlowRelServ::get_template_rel_filter(template_id.as_deref()),
            ..Default::default()
        };
        let mut models = Self::find_items(&filter, None, None, funs, &global_ctx).await?;
        if models.is_empty() {
            filter.basic.ids = None;
            models = Self::find_items(&filter, None, None, funs, ctx).await?;
        }

        Ok(models)
    }

    // Find the rel models.
    pub async fn find_rel_model_map(template_id: Option<String>, main: bool, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<HashMap<String, FlowModelSummaryResp>> {
        let models = Self::find_rel_models(template_id, main, None, funs, ctx).await?;

        let mut result: HashMap<String, FlowModelSummaryResp> = HashMap::new();
        // First iterate over the models
        for model in models {
            result.insert(model.tag.clone(), model);
        }

        Ok(result)
    }

    /// 创建或引用模型
    /// 当op为复制时，表示按原有配置复制一套新的模型。
    /// 当op为引用时，表示建立原有配置和当前own_paths的引用而不生成新的模型数据。
    /// 当op为引用和复制时，表示按原有配置复制一套新的模型同时建立引用，此时允许用户修改新模型，同时新模型也会被旧模型修改影响到。
    pub async fn copy_or_reference_model(
        rel_model_id: &str,
        op: &FlowModelAssociativeOperationKind,
        kind: FlowModelKind,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<FlowModelAggResp> {
        let mut rel_model = FlowModelServ::get_item(
            rel_model_id,
            &FlowModelFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ignore_scope: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        let target_model_id = match op {
            FlowModelAssociativeOperationKind::Copy => {
                if kind == FlowModelKind::AsModel {
                    rel_model.rel_model_id = "".to_string();
                    rel_model.template = false;
                    rel_model.scope_level = rbum_scope_helper::get_scope_level_by_context(ctx)?;
                    let mut rel_transition = rel_model.rel_transition().unwrap_or_default();
                    if !rel_transition.id.is_empty() && (rel_transition.id != *"__EDIT__" || rel_transition.id != *"__DETELE__") {
                        if let Some(rel_main_model_tran) = FlowModelServ::find_one_detail_item(
                            &FlowModelFilterReq {
                                basic: RbumBasicFilterReq {
                                    ignore_scope: true,
                                    ..Default::default()
                                },
                                tags: Some(vec![rel_model.tag.clone()]),
                                main: Some(true),
                                ..Default::default()
                            },
                            funs,
                            ctx,
                        )
                        .await?
                        .map(|model| {
                            model.transitions().into_iter().find(|tran| {
                                tran.from_flow_state_name == rel_transition.from_flow_state_name && Some(tran.to_flow_state_name.clone()) == rel_transition.to_flow_state_name
                            })
                        })
                        .unwrap_or(None)
                        {
                            rel_transition.id = rel_main_model_tran.id;
                            rel_model.rel_transition = Some(json!(rel_transition));
                        }
                    }
                }
                Self::add_item(
                    &mut FlowModelAddReq {
                        kind,
                        rel_template_ids: None,
                        ..rel_model.clone().into()
                    },
                    funs,
                    ctx,
                )
                .await?
            }
            FlowModelAssociativeOperationKind::Reference => {
                if let Some(template_id) =
                    FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowModelTemplate, rel_model_id, None, None, funs, ctx).await?.pop().map(|rel| rel.rel_id)
                {
                    if !FlowRelServ::exist_rels(
                        &FlowRelKind::FlowAppTemplate,
                        &template_id,
                        Self::get_app_id_by_ctx(ctx).unwrap_or_default().as_str(),
                        funs,
                        ctx,
                    )
                    .await?
                    {
                        FlowRelServ::add_simple_rel(
                            &FlowRelKind::FlowAppTemplate,
                            Self::get_app_id_by_ctx(ctx).unwrap_or_default().as_str(),
                            &template_id,
                            None,
                            None,
                            true,
                            true,
                            None,
                            funs,
                            ctx,
                        )
                        .await?;
                    }
                }
                rel_model_id.to_string()
            }
            FlowModelAssociativeOperationKind::ReferenceOrCopy => {
                let mut add_transitions = rel_model.transitions().into_iter().map(FlowTransitionAddReq::from).collect_vec();
                for add_transition in add_transitions.iter_mut() {
                    if let Some(ref mut action_by_post_changes) = &mut add_transition.action_by_post_changes {
                        for action_by_post_change in action_by_post_changes.iter_mut() {
                            action_by_post_change.is_edit = Some(false); // 引用复制时，置为不可编辑
                        }
                    }
                }
                let states = rel_model
                    .states()
                    .into_iter()
                    .map(|state| FlowModelVersionBindState {
                        exist_state: Some(FlowModelBindStateReq {
                            state_id: state.id.clone(),
                            ext: state.ext,
                        }),
                        add_transitions: Some(add_transitions.clone().into_iter().filter(|tran| tran.from_flow_state_id == state.id).collect_vec()),
                        is_init: state.id == rel_model.init_state_id,
                        ..Default::default()
                    })
                    .collect_vec();
                Self::add_item(
                    &mut FlowModelAddReq {
                        rel_model_id: Some(rel_model_id.to_string()),
                        kind,
                        rel_template_ids: None,
                        add_version: Some(FlowModelVersionAddReq {
                            name: rel_model.name.clone().into(),
                            rel_model_id: None,
                            bind_states: Some(states),
                            status: FlowModelVesionState::Enabled,
                            scope_level: Some(rel_model.scope_level.clone()),
                            disabled: Some(rel_model.disabled),
                        }),
                        ..rel_model.clone().into()
                    },
                    funs,
                    ctx,
                )
                .await?
            }
        };
        let new_model = Self::get_item_detail_aggs(&target_model_id, true, funs, ctx).await?;

        Ok(new_model)
    }

    pub async fn copy_models_by_template_id(
        from_template_id: &str,
        to_template_id: &str,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<HashMap<String, FlowModelAggResp>> {
        let mut result = HashMap::new();
        for from_model in Self::find_detail_items(
            &FlowModelFilterReq {
                basic: RbumBasicFilterReq {
                    ids: Some(
                        FlowRelServ::find_to_simple_rels(&FlowRelKind::FlowModelTemplate, from_template_id, None, None, funs, ctx)
                            .await?
                            .into_iter()
                            .map(|rel| rel.rel_id)
                            .collect_vec(),
                    ),
                    ignore_scope: true,
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
        {
            let added_model = Self::copy_or_reference_model(
                &from_model.id,
                &FlowModelAssociativeOperationKind::ReferenceOrCopy,
                FlowModelKind::AsTemplateAndAsModel,
                funs,
                ctx,
            )
            .await?;
            Self::modify_model(
                &added_model.id,
                &mut FlowModelModifyReq {
                    rel_model_id: Some(from_model.rel_model_id.clone()),
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?;
            FlowRelServ::add_simple_rel(&FlowRelKind::FlowModelTemplate, &added_model.id, to_template_id, None, None, false, true, None, funs, ctx).await?;
            result.insert(from_model.rel_model_id.clone(), added_model);
        }
        Ok(result)
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

    pub async fn find_rel_states(tags: Vec<&str>, rel_template_id: Option<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<FlowModelFindRelStateResp>> {
        let mut result = Vec::new();
        for tag in tags {
            let flow_model_id = Self::get_model_id_by_own_paths_and_rel_template_id(tag, rel_template_id.clone(), funs, ctx).await?.id;
            let flow_model = Self::get_item(&flow_model_id, &FlowModelFilterReq::default(), funs, ctx).await?;
            let mut states = FlowModelVersionServ::find_sorted_rel_states_by_version_id(&flow_model.current_version_id, funs, ctx)
                .await?
                .into_iter()
                .map(|state_detail| FlowModelFindRelStateResp {
                    id: state_detail.id.clone(),
                    name: state_detail.name.clone(),
                    color: state_detail.color.clone(),
                })
                .collect_vec();
            result.append(&mut states);
        }
        Ok(result)
    }

    pub async fn get_model_id_by_own_paths_and_transition_id(tag: &str, transition_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<FlowModelDetailResp> {
        let app_id = Self::get_app_id_by_ctx(ctx).ok_or_else(|| {
            funs.err().not_found(
                &Self::get_obj_name(),
                "get_model_id_by_own_paths_and_transition_id",
                "Only instances at the application layer are supported to use the approval flow",
                "404-flow-inst-rel-model-not-found",
            )
        })?;
        let model_detail = if let Some(model_ids) = FlowRelServ::find_model_ids_by_app_id(&app_id, funs, ctx).await? {
            // 引用租户模板的审批流
            Self::find_one_detail_item(
                &FlowModelFilterReq {
                    basic: RbumBasicFilterReq {
                        ids: Some(model_ids),
                        enabled: Some(true),
                        own_paths: Some("".to_string()),
                        with_sub_own_paths: true,
                        ..Default::default()
                    },
                    tags: Some(vec![tag.to_string()]),
                    rel: Some(RbumItemRelFilterReq {
                        optional: false,
                        rel_by_from: true,
                        tag: Some(FlowRelKind::FlowModelTransition.to_string()),
                        from_rbum_kind: Some(RbumRelFromKind::Item),
                        rel_item_id: Some(transition_id.to_string()),
                        ..Default::default()
                    }),
                    status: Some(FlowModelStatus::Enabled),
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?
        } else {
            // 当前项目内的审批流
            Self::find_one_detail_item(
                &FlowModelFilterReq {
                    basic: RbumBasicFilterReq {
                        enabled: Some(true),
                        ..Default::default()
                    },
                    tags: Some(vec![tag.to_string()]),
                    rel: Some(RbumItemRelFilterReq {
                        optional: false,
                        rel_by_from: true,
                        tag: Some(FlowRelKind::FlowModelTransition.to_string()),
                        from_rbum_kind: Some(RbumRelFromKind::Item),
                        rel_item_id: Some(transition_id.to_string()),
                        own_paths: Some(ctx.own_paths.clone()),
                        ..Default::default()
                    }),
                    status: Some(FlowModelStatus::Enabled),
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?
        };
        match model_detail {
            Some(result) => Ok(result),
            None => Err(funs.err().not_found(
                &Self::get_obj_name(),
                "get_model_id_by_own_paths_and_transition_id",
                "rel model not found",
                "404-flow-model-not-found",
            )),
        }
    }
    /// 根据own_paths和rel_template_id获取模型ID
    /// 规则1：如果rel_template_id不为空，优先通过rel_template_id查找rel表类型为FlowModelTemplate关联的模型ID，找不到则直接返回默认模板ID
    /// 规则2：如果rel_template_id为空，则通过own_paths获取rel表类型为FlowAppTemplate关联的模型ID
    /// 规则3：如果按照规则2未找到关联的模型，则通过own_paths直接获取model表中存在的模型ID
    /// 规则4：如果按照规则3未找到关联的模型，则直接返回默认的模板ID
    pub async fn get_model_id_by_own_paths_and_rel_template_id(
        tag: &str,
        rel_template_id: Option<String>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<FlowModelDetailResp> {
        let mut result = None;
        if let Some(rel_template_id) = rel_template_id {
            // 规则1
            result = FlowModelServ::find_detail_items(
                &FlowModelFilterReq {
                    basic: RbumBasicFilterReq {
                        own_paths: Some("".to_string()),
                        ignore_scope: true,
                        ..Default::default()
                    },
                    template: Some(true),
                    main: Some(true),
                    rel: FlowRelServ::get_template_rel_filter(Some(rel_template_id.as_str())),
                    tags: Some(vec![tag.to_string()]),
                    ..Default::default()
                },
                None,
                None,
                funs,
                ctx,
            )
            .await?
            .pop();
        } else {
            // 规则2
            result = FlowModelServ::find_detail_items(
                &FlowModelFilterReq {
                    basic: RbumBasicFilterReq {
                        ids: Some(FlowRelServ::find_model_ids_by_app_id(Self::get_app_id_by_ctx(ctx).unwrap_or_default().as_str(), funs, ctx).await?.unwrap_or_default()),
                        ignore_scope: true,
                        own_paths: Some("".to_string()),
                        with_sub_own_paths: true,
                        ..Default::default()
                    },
                    main: Some(true),
                    tags: Some(vec![tag.to_string()]),
                    ..Default::default()
                },
                None,
                None,
                funs,
                ctx,
            )
            .await?
            .pop();
            if result.is_none() {
                // 规则3
                result = FlowModelServ::find_detail_items(
                    &FlowModelFilterReq {
                        basic: RbumBasicFilterReq {
                            own_paths: Some(ctx.own_paths.clone()),
                            ignore_scope: true,
                            ..Default::default()
                        },
                        tags: Some(vec![tag.to_string()]),
                        template: Some(false),
                        main: Some(true),
                        ..Default::default()
                    },
                    None,
                    None,
                    funs,
                    ctx,
                )
                .await?
                .pop();
            }
        }
        // 规则4
        if result.is_none() {
            result = FlowModelServ::find_detail_items(
                &FlowModelFilterReq {
                    basic: RbumBasicFilterReq {
                        own_paths: Some("".to_string()),
                        ignore_scope: true,
                        ..Default::default()
                    },
                    main: Some(true),
                    tags: Some(vec![tag.to_string()]),
                    ..Default::default()
                },
                None,
                None,
                funs,
                ctx,
            )
            .await?
            .pop();
        }
        match result {
            Some(model) => Ok(model),
            None => Err(funs.err().not_found("flow_model_serv", "get_model_id_by_own_paths", "model not found", "404-flow-model-not-found")),
        }
    }

    pub async fn find_models_by_rel_template_id(
        tag: String,
        template: Option<bool>,
        rel_template_id: Option<String>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<FlowModelSummaryResp>> {
        let mut result = vec![];
        let mut not_bind_template_models = join_all(
            FlowModelServ::find_items(
                &FlowModelFilterReq {
                    basic: RbumBasicFilterReq {
                        with_sub_own_paths: false,
                        ..Default::default()
                    },
                    tags: Some(vec![tag.clone()]),
                    template,
                    main: Some(true),
                    rel_model_ids: Some(vec!["".to_string()]), // rel_model_id is empty and template is true, which means it is a workflow template.
                    ..Default::default()
                },
                Some(true),
                None,
                funs,
                ctx,
            )
            .await?
            .into_iter()
            .map(|model| async move {
                let funs = flow_constants::get_tardis_inst();
                let global_ctx: TardisContext = TardisContext::default();
                if FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowModelTemplate, &model.id, None, None, &funs, &global_ctx).await.unwrap_or_default().is_empty() {
                    Some(model)
                } else {
                    None
                }
            }),
        )
        .await
        .into_iter()
        .flatten()
        .collect_vec();
        result.append(&mut not_bind_template_models);
        if let Some(rel_template_id) = rel_template_id {
            let mut rel_template_models = FlowModelServ::find_items(
                &FlowModelFilterReq {
                    basic: RbumBasicFilterReq {
                        with_sub_own_paths: false,
                        ..Default::default()
                    },
                    tags: Some(vec![tag.clone()]),
                    template,
                    main: Some(true),
                    rel_model_ids: Some(vec!["".to_string()]), // rel_model_id is empty and template is true, which means it is a workflow template.
                    rel: Some(RbumItemRelFilterReq {
                        optional: false,
                        rel_by_from: true,
                        tag: Some(FlowRelKind::FlowModelTemplate.to_string()),
                        from_rbum_kind: Some(RbumRelFromKind::Item),
                        rel_item_id: Some(rel_template_id),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                Some(true),
                None,
                funs,
                ctx,
            )
            .await?;
            result.append(&mut rel_template_models);
        }

        Ok(result.into_iter().filter(|model| !model.init_state_id.is_empty()).collect_vec())
    }

    // 清除当前关联的模型数据（用于更新配置）
    /**
     * 当rel_template_id为空时：
     * 1、去除ModelPath引用关系
     * 2、删除当前own_path下的model
     * 当rel_template_id不为空时：
     * 1、去除ModelTemplate引用关系
     * 2、去除ModelPath引用关系
     * 3、删除当前rel_template_id下的model
     */
    pub async fn clean_rel_models(
        rel_template_id: Option<String>,
        orginal_model_ids: Option<Vec<String>>,
        spec_tags: Option<Vec<String>>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<HashMap<String, FlowModelSummaryResp>> {
        let global_ctx = TardisContext {
            own_paths: "".to_string(),
            ..ctx.clone()
        };
        let models = Self::find_rel_model_map(rel_template_id.clone(), true, funs, ctx).await?;
        let mut non_main_models = vec![];
        if let Some(rel_template_id) = rel_template_id.clone() {
            let rel_model_ids = FlowRelServ::find_to_simple_rels(&FlowRelKind::FlowModelTemplate, &rel_template_id, None, None, funs, &global_ctx)
                .await?
                .into_iter()
                .map(|rel| rel.rel_id)
                .collect_vec();
            let main_model_ids = Self::find_id_items(
                &FlowModelFilterReq {
                    basic: RbumBasicFilterReq {
                        with_sub_own_paths: true,
                        own_paths: Some("".to_string()),
                        ids: Some(rel_model_ids),
                        ..Default::default()
                    },
                    main: Some(true),
                    ..Default::default()
                },
                None,
                None,
                funs,
                ctx,
            )
            .await?;
            for main_model_id in main_model_ids {
                if let Some(orginal_model_ids) = orginal_model_ids.clone() {
                    if orginal_model_ids.contains(&main_model_id) {
                        continue;
                    }
                }
                FlowRelServ::delete_simple_rel(&FlowRelKind::FlowModelTemplate, &main_model_id, &rel_template_id, funs, &global_ctx).await?;
            }
        } else {
            // clean reference template rel
            for rel in FlowRelServ::find_from_simple_rels(
                &FlowRelKind::FlowAppTemplate,
                Self::get_app_id_by_ctx(ctx).unwrap_or_default().as_str(),
                None,
                None,
                funs,
                &global_ctx,
            )
            .await?
            .into_iter()
            {
                FlowRelServ::delete_simple_rel(
                    &FlowRelKind::FlowAppTemplate,
                    Self::get_app_id_by_ctx(ctx).unwrap_or_default().as_str(),
                    &rel.rel_id,
                    funs,
                    &global_ctx,
                )
                .await?;
            }
            non_main_models = Self::find_items(
                &FlowModelFilterReq {
                    basic: RbumBasicFilterReq {
                        enabled: Some(true),
                        ..Default::default()
                    },
                    main: Some(false),
                    ..Default::default()
                },
                None,
                None,
                funs,
                ctx,
            )
            .await?
        }
        for (tag, model) in models.iter() {
            if let Some(spec_tags) = spec_tags.clone() {
                if !spec_tags.contains(tag) {
                    continue;
                }
            }
            if let Some(orginal_model_ids) = orginal_model_ids.clone() {
                if orginal_model_ids.contains(&model.id) {
                    continue;
                }
            }
            if ctx.own_paths == model.own_paths {
                Self::delete_item(&model.id, funs, ctx).await?;
            }
        }
        // clean non-main flow model
        for model in non_main_models {
            if let Some(spec_tags) = spec_tags.clone() {
                if !spec_tags.contains(&model.tag) {
                    continue;
                }
            }
            // abort instances with current ctx
            let rel_version_ids = FlowModelVersionServ::find_id_items(
                &FlowModelVersionFilterReq {
                    rel_model_ids: Some(vec![model.id.clone()]),
                    ..Default::default()
                },
                None,
                None,
                funs,
                ctx,
            )
            .await?;
            for rel_version_id in rel_version_ids {
                FlowInstServ::unsafe_abort_inst(&rel_version_id, funs, ctx).await?;
            }
            Self::delete_item(&model.id, funs, ctx).await?;
        }

        Ok(models)
    }

    pub fn get_app_id_by_ctx(ctx: &TardisContext) -> Option<String> {
        rbum_scope_helper::get_path_item(2, &ctx.own_paths)
    }

    async fn sync_child_model(
        child_model: &FlowModelDetailResp,
        parent_model: &FlowModelDetailResp,
        modify_req: &FlowModelModifyReq,
        _funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let ctx_clone = TardisContext {
            own_paths: child_model.own_paths.clone(),
            ..ctx.clone()
        };
        let parent_model_transitions = parent_model.transitions();
        let child_model_transitions = child_model.transitions();
        let mut modify_req_clone = modify_req.clone();
        if let Some(ref mut modify_version) = &mut modify_req_clone.modify_version {
            if let Some(ref mut bind_states) = &mut modify_version.bind_states {
                for bind_state in bind_states.iter_mut() {
                    if let Some(ref mut add_transitions) = &mut bind_state.add_transitions {
                        for add_transition in add_transitions.iter_mut() {
                            if let Some(ref mut action_by_post_changes) = &mut add_transition.action_by_post_changes {
                                for action_by_post_change in action_by_post_changes.iter_mut() {
                                    action_by_post_change.is_edit = Some(false);
                                    // 引用复制时，置为不可编辑
                                }
                            }
                        }
                    }
                    if let Some(ref mut modify_transitions) = &mut bind_state.modify_transitions {
                        for modify_transition in modify_transitions.iter_mut() {
                            if let Some(parent_model_transition) = parent_model_transitions.iter().find(|trans| trans.id == modify_transition.id.to_string()) {
                                modify_transition.id = child_model_transitions
                                    .iter()
                                    .find(|child_tran| {
                                        child_tran.from_flow_state_id == parent_model_transition.from_flow_state_id
                                            && child_tran.to_flow_state_id == parent_model_transition.to_flow_state_id
                                    })
                                    .map(|trans| trans.id.clone())
                                    .unwrap_or_default()
                                    .into();
                            }
                        }
                    }
                    if let Some(delete_transitions) = &mut bind_state.delete_transitions {
                        let mut child_delete_transitions = vec![];
                        for delete_transition_id in delete_transitions.iter_mut() {
                            if let Some(parent_model_transition) = parent_model_transitions.iter().find(|trans| trans.id == delete_transition_id.clone()) {
                                child_delete_transitions.push(
                                    child_model_transitions
                                        .iter()
                                        .find(|child_tran| {
                                            child_tran.from_flow_state_id == parent_model_transition.from_flow_state_id
                                                && child_tran.to_flow_state_id == parent_model_transition.to_flow_state_id
                                        })
                                        .map(|trans| trans.id.clone())
                                        .unwrap_or_default(),
                                );
                            }
                        }
                        bind_state.delete_transitions = Some(child_delete_transitions);
                    }
                }
            }
            if let Some(ref mut modify_states) = &mut modify_version.modify_states {
                for modify_state in modify_states.iter_mut() {
                    if let Some(ref mut add_transitions) = &mut modify_state.add_transitions {
                        for add_transition in add_transitions.iter_mut() {
                            if let Some(ref mut action_by_post_changes) = &mut add_transition.action_by_post_changes {
                                for action_by_post_change in action_by_post_changes.iter_mut() {
                                    action_by_post_change.is_edit = Some(false);
                                    // 引用复制时，置为不可编辑
                                }
                            }
                        }
                    }
                    if let Some(ref mut modify_transitions) = &mut modify_state.modify_transitions {
                        for modify_transition in modify_transitions.iter_mut() {
                            if let Some(parent_model_transition) = parent_model_transitions.iter().find(|trans| trans.id == modify_transition.id.to_string()) {
                                modify_transition.id = child_model_transitions
                                    .iter()
                                    .find(|child_tran| {
                                        child_tran.from_flow_state_id == parent_model_transition.from_flow_state_id
                                            && child_tran.to_flow_state_id == parent_model_transition.to_flow_state_id
                                    })
                                    .map(|trans| trans.id.clone())
                                    .unwrap_or_default()
                                    .into();
                            }
                        }
                    }
                    if let Some(delete_transitions) = &mut modify_state.delete_transitions {
                        let mut child_delete_transitions = vec![];
                        for delete_transition_id in delete_transitions.iter_mut() {
                            if let Some(parent_model_transition) = parent_model_transitions.iter().find(|trans| trans.id == delete_transition_id.clone()) {
                                child_delete_transitions.push(
                                    child_model_transitions
                                        .iter()
                                        .find(|child_tran| {
                                            child_tran.from_flow_state_id == parent_model_transition.from_flow_state_id
                                                && child_tran.to_flow_state_id == parent_model_transition.to_flow_state_id
                                        })
                                        .map(|trans| trans.id.clone())
                                        .unwrap_or_default(),
                                );
                            }
                        }
                        modify_state.delete_transitions = Some(child_delete_transitions);
                    }
                }
            }
        }

        let child_model_clone = child_model.clone();
        ctx.add_async_task(Box::new(|| {
            Box::pin(async move {
                let child_model_id = child_model_clone.id.clone();
                let task_handle = tokio::spawn(async move {
                    let funs = flow_constants::get_tardis_inst();
                    let _ = Self::modify_item(&child_model_id, &mut modify_req_clone, &funs, &ctx_clone).await;
                });
                match task_handle.await {
                    Ok(_) => {}
                    Err(e) => error!("Flow Model {} sync_child_model error:{:?}", child_model_clone.id, e),
                }
                Ok(())
            })
        }))
        .await?;
        Ok(())
    }

    pub async fn resort_transition(flow_model_id: &str, resort_req: &FlowTransitionSortStatesReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let model_detail = Self::get_item(flow_model_id, &FlowModelFilterReq::default(), funs, ctx).await?;
        let modify_trans = resort_req
            .sort_states
            .clone()
            .into_iter()
            .map(|sort_req| FlowTransitionModifyReq {
                id: sort_req.id.clone().into(),
                sort: Some(sort_req.sort),
                ..Default::default()
            })
            .collect_vec();
        let mut modify_states = HashMap::new();
        let transitions = FlowTransitionServ::find_transitions(&model_detail.current_version_id, None, funs, ctx).await?;
        for modify_tran in modify_trans {
            if let Some(tansition) = transitions.iter().find(|tran| tran.id == modify_tran.id.to_string()) {
                let modify_transitons = modify_states.entry(&tansition.from_flow_state_id).or_insert(vec![]);
                modify_transitons.push(modify_tran);
            }
        }
        FlowModelServ::modify_model(
            flow_model_id,
            &mut FlowModelModifyReq {
                modify_version: Some(FlowModelVersionModifyReq {
                    modify_states: Some(
                        modify_states
                            .into_iter()
                            .map(|(id, modify_transitions)| FlowModelVersionModifyState {
                                id: Some(id.clone()),
                                modify_state: None,
                                modify_rel: None,
                                add_transitions: None,
                                modify_transitions: Some(modify_transitions),
                                delete_transitions: None,
                            })
                            .collect_vec(),
                    ),
                    ..Default::default()
                }),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        Ok(())
    }

    pub async fn get_rel_transitions(flow_model_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<FlowTransitionDetailResp>> {
        let model = Self::get_item(
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
        FlowTransitionServ::find_transitions(&model.current_version_id, None, funs, ctx).await
    }

    pub async fn find_editing_verion(flow_model_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<FlowModelVersionDetailResp> {
        let version = if let Some(version) = FlowModelVersionServ::find_one_detail_item(
            &FlowModelVersionFilterReq {
                rel_model_ids: Some(vec![flow_model_id.to_string()]),
                status: Some(vec![FlowModelVesionState::Editing]),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
        {
            Some(version)
        } else {
            // 当不存在正在编辑的版本时，按照当前启用版本复制一套作为最新的编辑版本
            let current_version_id = Self::get_item(flow_model_id, &FlowModelFilterReq::default(), funs, ctx).await?.current_version_id;
            Some(FlowModelVersionServ::create_editing_version(&current_version_id, funs, ctx).await?)
        };
        match version {
            Some(version) => Ok(version),
            None => Err(funs.err().not_found("flow_model_serv", "find_editing_verion", "model not found", "404-flow-model-not-found")),
        }
    }

    pub async fn sync_modified_field(req: &FlowModelSyncModifiedFieldReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let model_ids = if Self::get_app_id_by_ctx(ctx).is_some() {
            Self::find_id_items(
                &FlowModelFilterReq {
                    basic: RbumBasicFilterReq {
                        enabled: Some(true),
                        ..Default::default()
                    },
                    main: Some(false),
                    tags: Some(vec![req.tag.clone()]),
                    ..Default::default()
                },
                None,
                None,
                funs,
                ctx,
            )
            .await?
        } else {
            Self::find_id_items(
                &FlowModelFilterReq {
                    basic: RbumBasicFilterReq {
                        enabled: Some(true),
                        ..Default::default()
                    },
                    main: Some(false),
                    rel_template_id: req.rel_template_id.clone(),
                    tags: Some(vec![req.tag.clone()]),
                    ..Default::default()
                },
                None,
                None,
                funs,
                ctx,
            )
            .await?
        };
        let model_versions = FlowModelVersionServ::find_detail_items(
            &FlowModelVersionFilterReq {
                rel_model_ids: Some(model_ids),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        for model_version in model_versions {
            let states = model_version.states();
            for state in states {
                let add_default_conf = match state.state_kind {
                    FlowStateKind::Form => state.kind_conf.clone().unwrap_or_default().form.unwrap_or_default().add_default_field.unwrap_or_default(),
                    FlowStateKind::Approval => state.kind_conf.clone().unwrap_or_default().approval.unwrap_or_default().add_default_field.unwrap_or_default(),
                    _ => FlowStateVar::default(),
                };
                let mut kind_conf = state.kind_conf.clone().unwrap_or_default();
                for add_field in req.add_fields.clone().unwrap_or_default() {
                    match state.state_kind {
                        FlowStateKind::Form => {
                            kind_conf.form.as_mut().map(|form| form.vars_collect.insert(add_field, add_default_conf.clone()));
                        }
                        FlowStateKind::Approval => {
                            kind_conf.approval.as_mut().map(|form| form.vars_collect.insert(add_field, add_default_conf.clone()));
                        }
                        _ => {}
                    }
                }
                for delete_field in req.delete_fields.clone().unwrap_or_default() {
                    match state.state_kind {
                        FlowStateKind::Form => {
                            kind_conf.form.as_mut().map(|form| form.vars_collect.remove(&delete_field));
                        }
                        FlowStateKind::Approval => {
                            kind_conf.approval.as_mut().map(|form| form.vars_collect.remove(&delete_field));
                        }
                        _ => {}
                    }
                }
                FlowStateServ::modify_item(
                    &state.id,
                    &mut FlowStateModifyReq {
                        kind_conf: Some(kind_conf),
                        ..Default::default()
                    },
                    funs,
                    ctx,
                )
                .await?;
            }
        }

        Ok(())
    }

    pub async fn unbind_state(flow_model_id: &str, req: &FlowModelUnbindStateReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let flow_model = Self::get_item(flow_model_id, &FlowModelFilterReq::default(), funs, ctx).await?;

        let mut own_paths_list = vec![];
        if let Some(rel_template_id) = FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowModelTemplate, flow_model_id, None, None, funs, ctx).await?.pop().map(|rel| rel.rel_id)
        {
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
            FlowInstServ::unsafe_modify_state(&flow_model.tag, Some(vec![req.state_id.clone()]), &req.new_state_id, funs, &mock_ctx).await?;
        }
        Self::modify_model(
            flow_model_id,
            &mut FlowModelModifyReq {
                modify_version: Some(FlowModelVersionModifyReq {
                    unbind_states: Some(vec![req.state_id.clone()]),
                    ..Default::default()
                }),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        Ok(())
    }
}
