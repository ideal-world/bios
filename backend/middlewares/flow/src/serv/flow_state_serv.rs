use std::collections::{HashMap, HashSet};

use bios_basic::rbum::{
    dto::{
        rbum_filer_dto::RbumBasicFilterReq,
        rbum_item_dto::{RbumItemKernelAddReq, RbumItemKernelModifyReq},
        rbum_rel_dto::RbumRelModifyReq,
    },
    helper::rbum_scope_helper,
    rbum_enumeration::RbumScopeLevelKind,
    serv::rbum_item_serv::RbumItemCrudOperation,
};
use itertools::Itertools;
use tardis::{
    basic::{dto::TardisContext, field::TrimString, result::TardisResult},
    db::sea_orm::{
        sea_query::{Cond, Expr, SelectStatement},
        EntityName, Set,
    },
    futures::future::join_all,
    serde_json::json,
    TardisFuns, TardisFunsInst,
};

use crate::{
    domain::flow_state,
    dto::{
        flow_state_dto::{
            FlowStateAddReq, FlowStateAggResp, FlowStateCountGroupByStateReq, FlowStateCountGroupByStateResp, FlowStateDetailResp, FlowStateFilterReq, FlowStateKind,
            FlowStateModifyReq, FlowStateNameResp, FlowStateRelModelExt, FlowStateRelModelModifyReq, FlowStateSummaryResp, FlowSysStateKind,
        },
        flow_transition_dto::FlowTransitionFilterReq,
    },
    flow_config::FlowBasicInfoManager,
};
use async_trait::async_trait;

use super::{
    clients::log_client::{FlowLogClient, LogParamContent, LogParamTag},
    flow_inst_serv::FlowInstServ,
    flow_model_serv::FlowModelServ,
    flow_rel_serv::{FlowRelKind, FlowRelServ},
    flow_transition_serv::FlowTransitionServ,
};

pub struct FlowStateServ;

#[async_trait]
impl RbumItemCrudOperation<flow_state::ActiveModel, FlowStateAddReq, FlowStateModifyReq, FlowStateSummaryResp, FlowStateDetailResp, FlowStateFilterReq> for FlowStateServ {
    fn get_ext_table_name() -> &'static str {
        flow_state::Entity.table_name()
    }

    fn get_rbum_kind_id() -> Option<String> {
        Some(FlowBasicInfoManager::get_config(|conf: &crate::flow_config::BasicInfo| conf.kind_state_id.clone()))
    }

    fn get_rbum_domain_id() -> Option<String> {
        Some(FlowBasicInfoManager::get_config(|conf: &crate::flow_config::BasicInfo| conf.domain_flow_id.clone()))
    }

    async fn package_item_add(add_req: &FlowStateAddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<RbumItemKernelAddReq> {
        let id = if let Some(id) = &add_req.id {
            id.to_string()
        } else {
            format!(
                "{}{}",
                add_req.id_prefix.as_ref().map(|prefix| format!("{}-", prefix)).unwrap_or("".to_string()),
                TardisFuns::field.nanoid()
            )
        };
        Ok(RbumItemKernelAddReq {
            id: Some(TrimString(id)),
            name: add_req.name.as_ref().unwrap_or(&TrimString("".to_string())).clone(),
            scope_level: add_req.scope_level.clone(),
            disabled: add_req.disabled,
            ..Default::default()
        })
    }

    async fn package_ext_add(id: &str, add_req: &FlowStateAddReq, _: &TardisFunsInst, _ctx: &TardisContext) -> TardisResult<flow_state::ActiveModel> {
        Ok(flow_state::ActiveModel {
            id: Set(id.to_string()),
            icon: Set(add_req.icon.as_ref().unwrap_or(&"".to_string()).to_string()),
            color: Set(add_req.color.as_ref().unwrap_or(&"".to_string()).to_string()),
            sys_state: Set(add_req.sys_state.clone()),
            info: Set(add_req.info.as_ref().unwrap_or(&"".to_string()).to_string()),
            state_kind: Set(add_req.state_kind.clone().unwrap_or(FlowStateKind::Simple)),
            kind_conf: Set(add_req.kind_conf.clone()),
            template: Set(add_req.template.unwrap_or(false)),
            rel_state_id: Set(add_req.rel_state_id.as_ref().unwrap_or(&"".to_string()).to_string()),
            tags: Set(add_req.tags.as_ref().unwrap_or(&vec![]).to_vec().join(",")),
            main: Set(add_req.main.unwrap_or_default()),
            ..Default::default()
        })
    }

    async fn after_add_item(id: &str, add_req: &mut FlowStateAddReq, _funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if add_req.main.unwrap_or(false) {
            FlowLogClient::add_ctx_task(
                LogParamTag::DynamicLog,
                Some(id.to_string()),
                LogParamContent {
                    subject: Some("工作流状态".to_string()),
                    name: Some(add_req.name.clone().unwrap_or_default().to_string()),
                    sub_kind: Some("flow_state".to_string()),
                    ..Default::default()
                },
                None,
                Some("dynamic_log_tenant_config".to_string()),
                Some("新建".to_string()),
                rbum_scope_helper::get_path_item(RbumScopeLevelKind::L1.to_int(), &ctx.own_paths),
                false,
                ctx,
                false,
            )
            .await?;
        }

        Ok(())
    }

    async fn before_modify_item(_id: &str, _modify_req: &mut FlowStateModifyReq, _funs: &TardisFunsInst, _ctx: &TardisContext) -> TardisResult<()> {
        // Modifications are allowed only where non-key fields are modified or not used
        // if (modify_req.scope_level.is_some()
        //     || modify_req.disabled.is_some()
        //     || modify_req.sys_state.is_some()
        //     || modify_req.state_kind.is_some()
        //     || modify_req.kind_conf.is_some())
        //     && FlowModelServ::state_is_used(id, funs, ctx).await?
        // {
        //     return Err(funs.err().conflict(&Self::get_obj_name(), "modify", &format!("state {id} already used"), "409-flow-state-already-used"));
        // }
        Ok(())
    }

    async fn after_modify_item(id: &str, _: &mut FlowStateModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let state = Self::get_item(id, &FlowStateFilterReq::default(), funs, ctx).await?;
        if state.main {
            FlowLogClient::add_ctx_task(
                LogParamTag::DynamicLog,
                Some(id.to_string()),
                LogParamContent {
                    subject: Some("工作流状态".to_string()),
                    name: Some(state.name),
                    sub_kind: Some("flow_state".to_string()),
                    ..Default::default()
                },
                None,
                Some("dynamic_log_tenant_config".to_string()),
                Some("编辑".to_string()),
                rbum_scope_helper::get_path_item(RbumScopeLevelKind::L1.to_int(), &ctx.own_paths),
                false,
                ctx,
                false,
            )
            .await?;
        }
        Ok(())
    }
    async fn package_item_modify(_: &str, modify_req: &FlowStateModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<Option<RbumItemKernelModifyReq>> {
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

    async fn package_ext_modify(id: &str, modify_req: &FlowStateModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<Option<flow_state::ActiveModel>> {
        if modify_req.icon.is_none()
            && modify_req.color.is_none()
            && modify_req.sys_state.is_none()
            && modify_req.info.is_none()
            && modify_req.state_kind.is_none()
            && modify_req.kind_conf.is_none()
            && modify_req.template.is_none()
            && modify_req.rel_state_id.is_none()
            && modify_req.tags.is_none()
        {
            return Ok(None);
        }
        let mut flow_state = flow_state::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(icon) = &modify_req.icon {
            flow_state.icon = Set(icon.to_string());
        }
        if let Some(color) = &modify_req.color {
            flow_state.color = Set(color.to_string());
        }
        if let Some(sys_state) = &modify_req.sys_state {
            flow_state.sys_state = Set(sys_state.clone());
        }
        if let Some(info) = &modify_req.info {
            flow_state.info = Set(info.to_string());
        }
        if let Some(state_kind) = &modify_req.state_kind {
            flow_state.state_kind = Set(state_kind.clone());
        }
        if let Some(kind_conf) = &modify_req.kind_conf {
            flow_state.kind_conf = Set(Some(kind_conf.clone()));
        }
        if let Some(template) = modify_req.template {
            flow_state.template = Set(template);
        }
        if let Some(rel_state_id) = &modify_req.rel_state_id {
            flow_state.rel_state_id = Set(rel_state_id.to_string());
        }
        if let Some(tags) = &modify_req.tags {
            flow_state.tags = Set(tags.to_vec().join(","));
        }
        Ok(Some(flow_state))
    }

    async fn before_delete_item(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<FlowStateDetailResp>> {
        // Can only be deleted when not in use
        if FlowModelServ::state_is_used(id, funs, ctx).await? {
            return Err(funs.err().conflict(&Self::get_obj_name(), "delete", &format!("state {id} already used"), "409-flow-state-already-used"));
        }
        Ok(Some(Self::get_item(id, &FlowStateFilterReq::default(), funs, ctx).await?))
    }

    async fn after_delete_item(id: &str, detail: &Option<FlowStateDetailResp>, _funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if let Some(detail) = detail {
            if detail.main {
                FlowLogClient::add_ctx_task(
                    LogParamTag::DynamicLog,
                    Some(id.to_string()),
                    LogParamContent {
                        subject: Some("工作流状态".to_string()),
                        name: Some(detail.name.clone()),
                        sub_kind: Some("flow_state".to_string()),
                        ..Default::default()
                    },
                    None,
                    Some("dynamic_log_tenant_config".to_string()),
                    Some("删除".to_string()),
                    rbum_scope_helper::get_path_item(RbumScopeLevelKind::L1.to_int(), &ctx.own_paths),
                    false,
                    ctx,
                    false,
                )
                .await?;
            }
        }
        Ok(())
    }

    async fn package_ext_query(query: &mut SelectStatement, _: bool, filter: &FlowStateFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        query.column((flow_state::Entity, flow_state::Column::Icon));
        query.column((flow_state::Entity, flow_state::Column::Color));
        query.column((flow_state::Entity, flow_state::Column::SysState));
        query.column((flow_state::Entity, flow_state::Column::Info));
        query.column((flow_state::Entity, flow_state::Column::StateKind));
        query.column((flow_state::Entity, flow_state::Column::KindConf));
        query.column((flow_state::Entity, flow_state::Column::Template));
        query.column((flow_state::Entity, flow_state::Column::RelStateId));
        query.column((flow_state::Entity, flow_state::Column::Tags));
        query.column((flow_state::Entity, flow_state::Column::Main));

        if let Some(sys_state) = &filter.sys_state {
            query.and_where(Expr::col(flow_state::Column::SysState).eq(sys_state.clone()));
        }
        if let Some(tag) = &filter.tag {
            let mut cond = Cond::any();
            cond = cond.add(Expr::col(flow_state::Column::Tags).eq(""));
            for tag in tag.split(',') {
                cond = cond.add(Expr::col(flow_state::Column::Tags).eq(tag));
                cond = cond.add(Expr::col(flow_state::Column::Tags).like(format!("{},%", tag)));
                cond = cond.add(Expr::col(flow_state::Column::Tags).like(format!("%,{}", tag)));
                cond = cond.add(Expr::col(flow_state::Column::Tags).like(format!("%,{},%", tag)));
            }
            query.cond_where(cond);
        }
        if let Some(state_kind) = &filter.state_kind {
            query.and_where(Expr::col(flow_state::Column::StateKind).eq(state_kind.clone()));
        }
        if let Some(main) = filter.main {
            query.and_where(Expr::col(flow_state::Column::Main).eq(main));
        }
        if let Some(template) = filter.template {
            query.and_where(Expr::col(flow_state::Column::Template).eq(template));
        }
        if let Some(flow_version_ids) = filter.flow_version_ids.clone() {
            // find rel state
            let mut state_id = HashSet::new();
            for flow_version_id in flow_version_ids {
                let rel_state_id = FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowModelState, &flow_version_id, None, None, funs, ctx)
                    .await?
                    .iter()
                    .map(|rel| rel.rel_id.clone())
                    .collect::<Vec<_>>();
                state_id.extend(rel_state_id.into_iter());
            }

            query.and_where(Expr::col((flow_state::Entity, flow_state::Column::Id)).is_in(state_id));
        }
        Ok(())
    }
}

impl FlowStateServ {
    pub async fn init_state(tag: &str, state_name: &str, sys_state: FlowSysStateKind, color: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        FlowStateServ::add_item(
            &mut FlowStateAddReq {
                id: None,
                id_prefix: None,
                name: Some(state_name.into()),
                icon: None,
                color: Some(color.to_string()),
                sys_state,
                info: None,
                state_kind: None,
                kind_conf: None,
                template: None,
                rel_state_id: None,
                tags: Some(vec![tag.to_string()]),
                scope_level: Some(RbumScopeLevelKind::Root),
                disabled: None,
                main: Some(true),
            },
            funs,
            ctx,
        )
        .await
    }
    pub(crate) async fn find_names(
        ids: Option<Vec<String>>,
        tag: Option<String>,
        main: Option<bool>,
        app_ids: Option<Vec<String>>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<FlowStateNameResp>> {
        let mut flow_version_ids = vec![];
        if let Some(app_ids) = app_ids {
            let tag_ref = &tag;
            let flow_version_id_maps = join_all(
                app_ids
                    .iter()
                    .map(|app_id| async move {
                        let tag_cp = tag_ref;
                        if let Some(tenant_own_path) = rbum_scope_helper::get_path_item(1, &ctx.own_paths) {
                            let mock_ctx = TardisContext {
                                own_paths: format!("{}/{}", &tenant_own_path, &app_id),
                                ..ctx.clone()
                            };
                            if let Ok(models) = FlowModelServ::find_rel_models(None, true, tag_cp.clone().map(|tag| vec![tag]), funs, &mock_ctx).await {
                                models.into_iter().map(|model| model.current_version_id).collect_vec()
                            } else {
                                vec![]
                            }
                        } else {
                            vec![]
                        }
                    })
                    .collect_vec(),
            )
            .await;
            for app_flow_version_ids in flow_version_id_maps {
                flow_version_ids.extend(app_flow_version_ids);
            }
        }

        let mut names = Self::find_id_name_items(
            &FlowStateFilterReq {
                basic: RbumBasicFilterReq {
                    ids: ids.clone(),
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                tag: tag.clone(),
                main: Some(main.unwrap_or(true)),
                flow_version_ids: if flow_version_ids.is_empty() { None } else { Some(flow_version_ids) },
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?
        .into_iter()
        .map(|state_detail| FlowStateNameResp {
            key: state_detail.1.clone(),
            name: state_detail.1,
        })
        .collect_vec();
        names.push(FlowStateNameResp {
            key: crate::flow_constants::SPECIFED_APPROVING_STATE_NAME.to_string(),
            name: crate::flow_constants::SPECIFED_APPROVING_STATE_NAME.to_string(),
        });
        Ok(names)
    }

    // For the old data migration, this function match id by old state name
    pub(crate) async fn match_state_id_by_name(flow_model_id: &str, name: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        Ok(FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowModelState, flow_model_id, None, None, funs, ctx)
            .await?
            .into_iter()
            .find(|state| state.rel_name == name)
            .ok_or_else(|| funs.err().not_found("flow_state_serv", "find_state_id_by_name", &format!("state_name: {} not match", name), ""))?
            .rel_id)
    }

    pub async fn count_group_by_state(req: &FlowStateCountGroupByStateReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<FlowStateCountGroupByStateResp>> {
        let states = FlowModelServ::find_rel_states(vec![&req.tag], None, funs, ctx).await?;
        let mut result = HashMap::new();
        let insts = FlowInstServ::find_detail(req.inst_ids.clone(), None, None, funs, ctx).await?;
        for state in states {
            let mut inst_ids = insts.iter().filter(|inst| inst.current_state_id == state.id).map(|inst| inst.id.clone()).collect_vec();
            result
                .entry(state.name.clone())
                .and_modify(|resp: &mut FlowStateCountGroupByStateResp| {
                    resp.inst_ids.append(&mut inst_ids);
                    resp.count = (resp.count.parse::<usize>().unwrap_or_default() + inst_ids.len()).to_string()
                })
                .or_insert(FlowStateCountGroupByStateResp {
                    state_name: state.name,
                    count: inst_ids.len().to_string(),
                    inst_ids,
                });
        }
        Ok(result.into_values().collect_vec())
    }

    pub async fn get_rel_state_ext(flow_version_id: &str, state_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<FlowStateRelModelExt> {
        TardisFuns::json.str_to_obj::<FlowStateRelModelExt>(
            &FlowRelServ::find_simple_rels(&FlowRelKind::FlowModelState, Some(flow_version_id), Some(state_id), true, None, None, funs, ctx)
                .await?
                .pop()
                .ok_or_else(|| funs.err().internal_error("flow_model_serv", "modify_rel_state", "rel not found", "404-rel-not-found"))?
                .ext,
        )
    }

    pub async fn modify_rel_state_ext(flow_version_id: &str, modify_req: &FlowStateRelModelModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let mut ext = Self::get_rel_state_ext(flow_version_id, &modify_req.id, funs, ctx).await?;
        if let Some(sort) = modify_req.sort {
            ext.sort = sort;
        }
        if let Some(show_btns) = modify_req.show_btns.clone() {
            ext.show_btns = Some(show_btns);
        }
        FlowRelServ::modify_simple_rel(
            &FlowRelKind::FlowModelState,
            flow_version_id,
            &modify_req.id,
            &mut RbumRelModifyReq {
                tag: None,
                note: None,
                ext: Some(json!(ext).to_string()),
            },
            funs,
            ctx,
        )
        .await?;

        Ok(())
    }

    pub async fn aggregate(id: &str, flow_version_id: &str, init_state_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<FlowStateAggResp> {
        let state = Self::get_item(
            id,
            &FlowStateFilterReq {
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
        let transitions = FlowTransitionServ::find_detail_items(
            &FlowTransitionFilterReq {
                flow_version_id: Some(flow_version_id.to_string()),
                specified_state_ids: Some(vec![state.id.clone()]),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        let kind_conf = state.kind_conf();
        Ok(FlowStateAggResp {
            id: state.id.clone(),
            name: state.name,
            is_init: state.id == *init_state_id,
            sys_state: state.sys_state,
            tags: state.tags,
            scope_level: state.scope_level,
            disabled: state.disabled,
            main: state.main,
            ext: Self::get_rel_state_ext(flow_version_id, &state.id, funs, ctx).await?,
            state_kind: state.state_kind,
            kind_conf,
            transitions,
        })
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
}
