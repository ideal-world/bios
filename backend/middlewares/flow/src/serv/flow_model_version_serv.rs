use std::collections::HashMap;

use bios_basic::rbum::{
    dto::{
        rbum_filer_dto::RbumBasicFilterReq,
        rbum_item_dto::{RbumItemKernelAddReq, RbumItemKernelModifyReq},
    },
    serv::rbum_item_serv::RbumItemCrudOperation,
};
use itertools::Itertools;
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    chrono::Utc,
    db::sea_orm::{
        prelude::Expr,
        sea_query::{Alias, SelectStatement},
        EntityName, Set,
    },
    futures::future::join_all,
    serde_json::json,
    TardisFuns, TardisFunsInst,
};

use crate::{
    domain::flow_model_version,
    dto::{
        flow_model_dto::{FlowModelBindNewStateReq, FlowModelBindStateReq, FlowModelFilterReq, FlowModelModifyReq, FlowModelStatus},
        flow_model_version_dto::{
            FlowModelVersionAddReq, FlowModelVersionBindState, FlowModelVersionDetailResp, FlowModelVersionFilterReq, FlowModelVersionModifyReq, FlowModelVersionSummaryResp,
            FlowModelVesionState,
        },
        flow_state_dto::{FlowStateAddReq, FlowStateAggResp, FlowStateDetailResp, FlowStateFilterReq, FlowStateKind, FlowStateRelModelExt},
        flow_transition_dto::FlowTransitionAddReq,
    },
    flow_config::FlowBasicInfoManager,
};
use async_trait::async_trait;

use super::{
    flow_inst_serv::FlowInstServ,
    flow_model_serv::FlowModelServ,
    flow_rel_serv::{FlowRelKind, FlowRelServ},
    flow_state_serv::FlowStateServ,
    flow_transition_serv::FlowTransitionServ,
};

pub struct FlowModelVersionServ;

#[async_trait]
impl
    RbumItemCrudOperation<
        flow_model_version::ActiveModel,
        FlowModelVersionAddReq,
        FlowModelVersionModifyReq,
        FlowModelVersionSummaryResp,
        FlowModelVersionDetailResp,
        FlowModelVersionFilterReq,
    > for FlowModelVersionServ
{
    fn get_ext_table_name() -> &'static str {
        flow_model_version::Entity.table_name()
    }

    fn get_rbum_kind_id() -> Option<String> {
        Some(FlowBasicInfoManager::get_config(|conf: &crate::flow_config::BasicInfo| conf.kind_model_version_id.clone()))
    }

    fn get_rbum_domain_id() -> Option<String> {
        Some(FlowBasicInfoManager::get_config(|conf: &crate::flow_config::BasicInfo| conf.domain_flow_id.clone()))
    }

    async fn package_item_add(add_req: &FlowModelVersionAddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<RbumItemKernelAddReq> {
        Ok(RbumItemKernelAddReq {
            name: add_req.name.clone(),
            disabled: add_req.disabled,
            scope_level: add_req.scope_level.clone(),
            ..Default::default()
        })
    }

    async fn package_ext_add(id: &str, add_req: &FlowModelVersionAddReq, _: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<flow_model_version::ActiveModel> {
        Ok(flow_model_version::ActiveModel {
            id: Set(id.to_string()),
            init_state_id: Set("".to_string()),
            rel_model_id: Set(add_req.rel_model_id.clone().unwrap_or_default()),
            create_by: Set(ctx.owner.clone()),
            update_by: Set(ctx.owner.clone()),
            own_paths: Set(ctx.own_paths.clone()),
            status: Set(add_req.status.clone()),
            ..Default::default()
        })
    }

    async fn before_add_item(_add_req: &mut FlowModelVersionAddReq, _funs: &TardisFunsInst, _ctx: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    async fn after_add_item(flow_version_id: &str, add_req: &mut FlowModelVersionAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let version_detail = Self::peek_item(flow_version_id, &FlowModelVersionFilterReq::default(), funs, ctx).await?;
        if let Some(bind_states) = &add_req.bind_states {
            Self::bind_states_and_transitions(flow_version_id, bind_states, funs, ctx).await?;
        }
        if add_req.status == FlowModelVesionState::Enabled {
            Self::enable_version(flow_version_id, funs, ctx).await?;
            FlowModelServ::modify_model(
                &version_detail.rel_model_id,
                &mut FlowModelModifyReq {
                    current_version_id: Some(flow_version_id.to_string()),
                    status: Some(FlowModelStatus::Enabled),
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?;
        }

        Ok(())
    }

    async fn package_item_modify(_: &str, modify_req: &FlowModelVersionModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<Option<RbumItemKernelModifyReq>> {
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

    async fn package_ext_modify(id: &str, modify_req: &FlowModelVersionModifyReq, _: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<flow_model_version::ActiveModel>> {
        if modify_req.init_state_id.is_none() && modify_req.status.is_none() {
            return Ok(None);
        }
        let mut flow_mode_version = flow_model_version::ActiveModel {
            id: Set(id.to_string()),
            update_time: Set(Utc::now()),
            ..Default::default()
        };
        if let Some(status) = &modify_req.status {
            flow_mode_version.status = Set(status.clone());
            if *status == FlowModelVesionState::Enabled {
                flow_mode_version.publish_by = Set(Some(ctx.owner.clone()));
                flow_mode_version.publish_time = Set(Some(Utc::now()));
            }
        }
        if let Some(init_state_id) = &modify_req.init_state_id {
            flow_mode_version.init_state_id = Set(init_state_id.clone());
        }
        Ok(Some(flow_mode_version))
    }

    async fn after_modify_item(id: &str, modify_req: &mut FlowModelVersionModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let version_detail = Self::get_item(id, &FlowModelVersionFilterReq::default(), funs, ctx).await?;
        if let Some(status) = &modify_req.status {
            if *status == FlowModelVesionState::Enabled {
                Self::enable_version(id, funs, ctx).await?;
                FlowModelServ::modify_item(
                    &version_detail.rel_model_id,
                    &mut FlowModelModifyReq {
                        current_version_id: Some(id.to_string()),
                        status: Some(FlowModelStatus::Enabled),
                        ..Default::default()
                    },
                    funs,
                    ctx,
                )
                .await?;
            }
        }
        if let Some(bind_states) = &modify_req.bind_states {
            Self::bind_states_and_transitions(id, bind_states, funs, ctx).await?;
        }
        if let Some(modify_states) = &modify_req.modify_states {
            for modify_state in modify_states {
                if let Some(state_id) = &modify_state.id {
                    if let Some(mut modify_state) = modify_state.modify_state.clone() {
                        FlowStateServ::modify_item(state_id, &mut modify_state, funs, ctx).await?;
                    }
                    if let Some(add_transitions) = &modify_state.add_transitions {
                        FlowTransitionServ::add_transitions(id, state_id, add_transitions, funs, ctx).await?;
                    }
                }
                if let Some(modify_rel) = &modify_state.modify_rel {
                    FlowStateServ::modify_rel_state_ext(id, modify_rel, funs, ctx).await?;
                }
                if let Some(modify_transitions) = &modify_state.modify_transitions {
                    FlowTransitionServ::modify_transitions(id, modify_transitions, funs, ctx).await?;
                }
                if let Some(delete_transitions) = &modify_state.delete_transitions {
                    FlowTransitionServ::delete_transitions(id, delete_transitions, funs, ctx).await?;
                }
            }
        }
        if let Some(unbind_states) = &modify_req.unbind_states {
            for delete_state in unbind_states {
                Self::unbind_state(id, delete_state, funs, ctx).await?;
            }
        }
        if let Some(delete_states) = &modify_req.delete_states {
            for delete_state in delete_states {
                Self::delete_state(id, delete_state, funs, ctx).await?;
            }
        }

        Ok(())
    }

    async fn before_delete_item(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<FlowModelVersionDetailResp>> {
        let detail = Self::get_item(id, &FlowModelVersionFilterReq::default(), funs, ctx).await?;
        join_all(
            FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowModelState, id, None, None, funs, ctx)
                .await?
                .into_iter()
                .map(|rel| async move { FlowRelServ::delete_simple_rel(&FlowRelKind::FlowModelState, id, &rel.rel_id, funs, ctx).await })
                .collect_vec(),
        )
        .await
        .into_iter()
        .collect::<TardisResult<Vec<()>>>()?;
        Ok(Some(detail))
    }

    async fn package_ext_query(query: &mut SelectStatement, _: bool, filter: &FlowModelVersionFilterReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<()> {
        query
            .column((flow_model_version::Entity, flow_model_version::Column::InitStateId))
            .column((flow_model_version::Entity, flow_model_version::Column::Status))
            .column((flow_model_version::Entity, flow_model_version::Column::RelModelId))
            .column((flow_model_version::Entity, flow_model_version::Column::CreateBy))
            .column((flow_model_version::Entity, flow_model_version::Column::CreateTime))
            .column((flow_model_version::Entity, flow_model_version::Column::UpdateBy))
            .column((flow_model_version::Entity, flow_model_version::Column::UpdateTime))
            .column((flow_model_version::Entity, flow_model_version::Column::PublishBy))
            .column((flow_model_version::Entity, flow_model_version::Column::PublishTime))
            .expr_as(Expr::val(json! {()}), Alias::new("states"));

        if let Some(own_paths) = filter.own_paths.clone() {
            query.and_where(Expr::col((flow_model_version::Entity, flow_model_version::Column::OwnPaths)).is_in(own_paths));
        }
        if let Some(status) = filter.status.clone() {
            query.and_where(Expr::col((flow_model_version::Entity, flow_model_version::Column::Status)).is_in(status));
        }
        if let Some(rel_model_ids) = filter.rel_model_ids.clone() {
            query.and_where(Expr::col((flow_model_version::Entity, flow_model_version::Column::RelModelId)).is_in(rel_model_ids));
        }

        Ok(())
    }

    async fn get_item(flow_version_id: &str, filter: &FlowModelVersionFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<FlowModelVersionDetailResp> {
        let mut flow_version = Self::do_get_item(flow_version_id, filter, funs, ctx).await?;
        let init_state_id = flow_version.init_state_id.clone();
        let flow_states = Self::get_rel_states(flow_version_id, &init_state_id, filter.specified_state_ids.clone(), funs, ctx).await;

        flow_version.states = Some(TardisFuns::json.obj_to_json(&flow_states)?);

        Ok(flow_version)
    }

    async fn find_detail_items(
        filter: &FlowModelVersionFilterReq,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<FlowModelVersionDetailResp>> {
        let mut flow_model_versions = Self::do_find_detail_items(filter, desc_sort_by_create, desc_sort_by_update, funs, ctx).await?;
        for flow_model_version in &mut flow_model_versions {
            let flow_states = Self::get_rel_states(&flow_model_version.id, &flow_model_version.init_state_id, filter.specified_state_ids.clone(), funs, ctx).await;
            flow_model_version.states = Some(TardisFuns::json.obj_to_json(&flow_states)?);
        }

        Ok(flow_model_versions)
    }
}

impl FlowModelVersionServ {
    async fn bind_state(flow_version_id: &str, req: &FlowModelBindStateReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let global_ctx = TardisContext {
            own_paths: "".to_string(),
            ..ctx.clone()
        };
        if let Ok(state) = FlowStateServ::get_item(
            &req.state_id,
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
        {
            let version_detail = Self::peek_item(flow_version_id, &FlowModelVersionFilterReq::default(), funs, ctx).await?;
            let tag = FlowModelServ::get_item(&version_detail.rel_model_id, &FlowModelFilterReq::default(), funs, ctx).await?.tag;
            if !state.tags.is_empty() && !state.tags.split(',').collect_vec().contains(&tag.as_str()) {
                return Err(funs.err().internal_error("flow_model_serv", "bind_state", "The flow state is not found", "404-flow-state-not-found"));
            }
        } else {
            return Err(funs.err().internal_error("flow_model_serv", "bind_state", "The flow state is not found", "404-flow-state-not-found"));
        }
        FlowRelServ::add_simple_rel(
            &FlowRelKind::FlowModelState,
            flow_version_id,
            &req.state_id,
            None,
            None,
            false,
            true,
            Some(json!(req.ext).to_string()),
            funs,
            ctx,
        )
        .await
    }

    async fn get_rel_states(
        flow_version_id: &str,
        init_state_id: &str,
        specified_state_ids: Option<Vec<String>>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> Vec<FlowStateAggResp> {
        join_all(
            FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowModelState, flow_version_id, None, None, funs, ctx)
                .await
                .expect("not found state")
                .into_iter()
                .filter(|rel| specified_state_ids.is_none() || (specified_state_ids.is_some() && specified_state_ids.clone().unwrap_or_default().contains(&rel.rel_id)))
                .sorted_by_key(|rel| TardisFuns::json.str_to_obj::<FlowStateRelModelExt>(&rel.ext).unwrap_or_default().sort)
                .map(|rel| async {
                    let state_id = rel.rel_id;
                    FlowStateServ::aggregate(&state_id, flow_version_id, init_state_id, funs, ctx).await.expect("not found state")
                })
                .collect_vec(),
        )
        .await
    }

    pub async fn bind_states_and_transitions(flow_version_id: &str, states: &[FlowModelVersionBindState], funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let mut binded_states = vec![];
        for bind_state in states {
            let (state_id, bind_state_req) = if let Some(bind_req) = bind_state.exist_state.clone() {
                (bind_req.state_id.clone(), bind_req)
            } else if let Some(mut bind_new_state) = bind_state.bind_new_state.clone() {
                let state_id = FlowStateServ::add_item(&mut bind_new_state.new_state, funs, ctx).await?;
                (
                    state_id.clone(),
                    FlowModelBindStateReq {
                        state_id,
                        ext: bind_new_state.ext,
                    },
                )
            } else {
                return Err(funs.err().conflict(&Self::get_obj_name(), "bind_states", "miss exist_state or new_state", "400-flow-inst-vars-field-missing"));
            };
            Self::bind_state(flow_version_id, &bind_state_req, funs, ctx).await?;
            binded_states.push((state_id, bind_state));
        }
        for (binded_state_id, bind_req) in binded_states {
            if let Some(add_transitions) = bind_req.add_transitions.clone() {
                FlowTransitionServ::add_transitions(flow_version_id, &binded_state_id, &add_transitions, funs, ctx).await?;
            }
            if let Some(modify_transitions) = &bind_req.modify_transitions {
                FlowTransitionServ::modify_transitions(flow_version_id, modify_transitions, funs, ctx).await?;
            }
            if let Some(delete_transitions) = &bind_req.delete_transitions {
                FlowTransitionServ::delete_transitions(flow_version_id, delete_transitions, funs, ctx).await?;
            }
            if bind_req.is_init {
                Self::modify_item(
                    flow_version_id,
                    &mut FlowModelVersionModifyReq {
                        init_state_id: Some(binded_state_id),
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

    // 版本发布操作（发布时将同模板的其他版本置为关闭状态）
    pub async fn enable_version(flow_version_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let version_detail = Self::peek_item(flow_version_id, &FlowModelVersionFilterReq::default(), funs, ctx).await?;
        let versions = Self::find_items(
            &FlowModelVersionFilterReq {
                rel_model_ids: Some(vec![version_detail.rel_model_id.clone()]),
                status: Some(vec![FlowModelVesionState::Enabled, FlowModelVesionState::Editing]),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;

        for version in versions {
            if flow_version_id != version.id {
                Self::modify_item(
                    &version.id,
                    &mut FlowModelVersionModifyReq {
                        status: Some(FlowModelVesionState::Disabled),
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

    pub async fn unbind_state(flow_version_id: &str, state_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        // Can only be deleted when not in use
        if FlowInstServ::state_is_used(flow_version_id, state_id, funs, ctx).await? {
            return Err(funs.err().conflict(
                &Self::get_obj_name(),
                "unbind_state",
                &format!("state {state_id} already used"),
                "409-flow-state-already-used",
            ));
        }
        // 获取指向当前节点的动作
        let to_trans = FlowTransitionServ::find_transitions_by_state_id(flow_version_id, Some(vec![state_id.to_string()]), None, funs, ctx).await?;
        FlowTransitionServ::delete_transitions(flow_version_id, &to_trans.into_iter().map(|tran| tran.id).collect_vec(), funs, ctx).await?;
        // 获取当前节点指向的动作
        let from_trans = FlowTransitionServ::find_transitions_by_state_id(flow_version_id, None, Some(vec![state_id.to_string()]), funs, ctx).await?;
        FlowTransitionServ::delete_transitions(flow_version_id, &from_trans.into_iter().map(|tran| tran.id).collect_vec(), funs, ctx).await?;
        FlowRelServ::delete_simple_rel(&FlowRelKind::FlowModelState, flow_version_id, state_id, funs, ctx).await
    }

    pub async fn delete_state(flow_version_id: &str, state_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let _state = FlowStateServ::find_one_detail_item(
            &FlowStateFilterReq {
                basic: RbumBasicFilterReq {
                    ids: Some(vec![state_id.to_string()]),
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
        .ok_or_else(|| funs.err().not_found(&Self::get_obj_name(), "delete_state", "flow state is not found", "404-flow-state-not-found"))?;
        Self::delete_single_state(flow_version_id, state_id, funs, ctx).await
    }

    async fn delete_single_state(flow_version_id: &str, state_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let to_trans = FlowTransitionServ::find_transitions_by_state_id(flow_version_id, Some(vec![state_id.to_string()]), None, funs, ctx).await?;
        // 获取当前节点指向的动作
        let from_trans = FlowTransitionServ::find_transitions_by_state_id(flow_version_id, None, Some(vec![state_id.to_string()]), funs, ctx).await?;
        let mut delete_flow_transition_ids = vec![];
        for from_tran in &from_trans {
            delete_flow_transition_ids.push(from_tran.id.clone());
        }
        for to_tran in &to_trans {
            delete_flow_transition_ids.push(to_tran.id.clone());
        }

        FlowTransitionServ::delete_transitions(flow_version_id, &delete_flow_transition_ids, funs, ctx).await?;
        Self::unbind_state(flow_version_id, state_id, funs, ctx).await?;
        FlowStateServ::delete_item(state_id, funs, ctx).await?;
        Ok(())
    }

    pub async fn find_sorted_rel_states_by_version_id(flow_version_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<FlowStateDetailResp>> {
        Ok(join_all(
            FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowModelState, flow_version_id, None, None, funs, ctx)
                .await?
                .into_iter()
                .sorted_by_key(|rel| TardisFuns::json.str_to_obj::<FlowStateRelModelExt>(&rel.ext).unwrap_or_default().sort)
                .map(|rel| async move {
                    FlowStateServ::find_one_detail_item(
                        &FlowStateFilterReq {
                            basic: RbumBasicFilterReq {
                                ids: Some(vec![rel.rel_id]),
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
                    .unwrap_or_default()
                })
                .collect::<Vec<_>>(),
        )
        .await
        .into_iter()
        .flatten()
        .collect_vec())
    }

    pub async fn create_editing_version(flow_version_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<FlowModelVersionDetailResp> {
        let version = FlowModelVersionServ::get_item(flow_version_id, &FlowModelVersionFilterReq::default(), funs, ctx).await?;
        // 将当前正在编辑的版本删除
        let editing_version_ids = Self::find_id_items(
            &FlowModelVersionFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    own_paths: Some("".to_string()),
                    ..Default::default()
                },
                rel_model_ids: Some(vec![version.rel_model_id.clone()]),
                status: Some(vec![FlowModelVesionState::Editing]),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        for editing_version_id in editing_version_ids {
            Self::delete_item(&editing_version_id, funs, ctx).await?;
        }
        let mut states = version.states();
        let mut update_state_map = HashMap::new();
        for state in states.iter_mut() {
            let old_state_id = state.id.clone();
            state.id = TardisFuns::field.nanoid();
            update_state_map.insert(old_state_id, state.id.clone());
        }
        for state in states.iter_mut() {
            for transition in state.transitions.iter_mut() {
                transition.from_flow_state_id = update_state_map.get(&transition.from_flow_state_id).cloned().unwrap_or_default();
                transition.to_flow_state_id = update_state_map.get(&transition.to_flow_state_id).cloned().unwrap_or_default();
            }
        }
        let editind_version_id = FlowModelVersionServ::add_item(
            &mut FlowModelVersionAddReq {
                name: version.name.into(),
                rel_model_id: Some(version.rel_model_id.clone()),
                bind_states: Some(
                    states
                        .into_iter()
                        .map(|state| FlowModelVersionBindState {
                            bind_new_state: Some(FlowModelBindNewStateReq {
                                new_state: FlowStateAddReq {
                                    id: Some(state.id.clone().into()),
                                    name: Some(state.name.clone().into()),
                                    sys_state: state.sys_state.clone(),
                                    state_kind: Some(state.state_kind.clone()),
                                    kind_conf: state.kind_conf,
                                    tags: Some(state.tags.clone().split(',').map(|id| id.to_string()).collect_vec()),
                                    scope_level: Some(state.scope_level.clone()),
                                    disabled: Some(state.disabled),
                                    main: Some(state.main),
                                    ..Default::default()
                                },
                                ext: state.ext,
                            }),
                            add_transitions: Some(state.transitions.into_iter().map(FlowTransitionAddReq::from).collect_vec()),
                            is_init: state.state_kind == FlowStateKind::Start,
                            ..Default::default()
                        })
                        .collect_vec(),
                ),
                status: FlowModelVesionState::Editing,
                scope_level: Some(version.scope_level.clone()),
                disabled: Some(version.disabled),
            },
            funs,
            ctx,
        )
        .await?;
        FlowModelVersionServ::get_item(&editind_version_id, &FlowModelVersionFilterReq::default(), funs, ctx).await
    }
}
