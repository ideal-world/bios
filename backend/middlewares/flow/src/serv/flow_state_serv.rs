use std::collections::{HashMap, HashSet};

use bios_basic::rbum::{
    dto::{
        rbum_filer_dto::RbumBasicFilterReq,
        rbum_item_dto::{RbumItemKernelAddReq, RbumItemKernelModifyReq},
    },
    helper::rbum_scope_helper,
    rbum_enumeration::RbumScopeLevelKind,
    serv::{rbum_item_serv::RbumItemCrudOperation, rbum_kind_serv::RbumKindServ, rbum_rel_serv::RbumRelServ},
};
use itertools::Itertools;
use tardis::{
    basic::{dto::TardisContext, field::TrimString, result::TardisResult},
    db::sea_orm::{
        sea_query::{Cond, Expr, SelectStatement},
        ColumnTrait, EntityName, EntityTrait, QueryFilter, Set,
    },
    futures::future::join_all,
    serde_json::json,
    TardisFuns, TardisFunsInst,
};

use crate::{
    domain::{flow_inst, flow_model, flow_state, flow_transition},
    dto::{
        flow_model_dto::FlowModelFilterReq,
        flow_state_dto::{
            FlowStateAddReq, FlowStateCountGroupByStateReq, FlowStateCountGroupByStateResp, FlowStateDetailResp, FlowStateFilterReq, FlowStateKind, FlowStateModifyReq,
            FlowStateNameResp, FlowStateSummaryResp, FlowSysStateKind,
        },
    },
    flow_config::FlowBasicInfoManager,
    flow_constants,
};
use async_trait::async_trait;

use super::{
    clients::log_client::{FlowLogClient, LogParamContent, LogParamTag},
    flow_inst_serv::FlowInstServ,
    flow_model_serv::FlowModelServ,
    flow_rel_serv::{FlowRelKind, FlowRelServ},
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
        let id = format!(
            "{}{}",
            add_req.id_prefix.as_ref().map(|prefix| format!("{}-", prefix)).unwrap_or("".to_string()),
            TardisFuns::field.nanoid()
        );
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
            kind_conf: Set(add_req.kind_conf.as_ref().unwrap_or(&json!({})).clone()),
            template: Set(add_req.template.unwrap_or(false)),
            rel_state_id: Set(add_req.rel_state_id.as_ref().unwrap_or(&"".to_string()).to_string()),
            tags: Set(add_req.tags.as_ref().unwrap_or(&vec![]).to_vec().join(",")),
            ..Default::default()
        })
    }

    async fn after_add_item(id: &str, add_req: &mut FlowStateAddReq, _funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        FlowLogClient::add_ctx_task(
            LogParamTag::DynamicLog,
            Some(id.to_string()),
            LogParamContent {
                subject: "工作流状态".to_string(),
                name: add_req.name.clone().unwrap_or_default().to_string(),
                sub_kind: "flow_state".to_string(),
            },
            None,
            Some("dynamic_log_tenant_config".to_string()),
            Some("新建".to_string()),
            rbum_scope_helper::get_path_item(RbumScopeLevelKind::L1.to_int(), &ctx.own_paths),
            ctx,
        )
        .await?;
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
        FlowLogClient::add_ctx_task(
            LogParamTag::DynamicLog,
            Some(id.to_string()),
            LogParamContent {
                subject: "工作流状态".to_string(),
                name: Self::get_item(id, &FlowStateFilterReq::default(), funs, ctx).await?.name,
                sub_kind: "flow_state".to_string(),
            },
            None,
            Some("dynamic_log_tenant_config".to_string()),
            Some("编辑".to_string()),
            rbum_scope_helper::get_path_item(RbumScopeLevelKind::L1.to_int(), &ctx.own_paths),
            ctx,
        )
        .await?;
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
            flow_state.kind_conf = Set(kind_conf.clone());
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
            FlowLogClient::add_ctx_task(
                LogParamTag::DynamicLog,
                Some(id.to_string()),
                LogParamContent {
                    subject: "工作流状态".to_string(),
                    name: detail.name.clone(),
                    sub_kind: "flow_state".to_string(),
                },
                None,
                Some("dynamic_log_tenant_config".to_string()),
                Some("删除".to_string()),
                rbum_scope_helper::get_path_item(RbumScopeLevelKind::L1.to_int(), &ctx.own_paths),
                ctx,
            )
            .await?;
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
        if let Some(template) = filter.template {
            query.and_where(Expr::col(flow_state::Column::Template).eq(template));
        }
        if let Some(flow_model_ids) = filter.flow_model_ids.clone() {
            // find rel state
            let mut state_id = HashSet::new();
            for flow_model_id in flow_model_ids {
                let rel_state_id = FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowModelState, &flow_model_id, None, None, funs, ctx)
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
            },
            funs,
            ctx,
        )
        .await
    }
    pub(crate) async fn find_names(
        ids: Option<Vec<String>>,
        tag: Option<String>,
        app_ids: Option<Vec<String>>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<FlowStateNameResp>> {
        let mut flow_model_ids = None;
        if let Some(app_ids) = app_ids {
            let tenant_own_path = rbum_scope_helper::get_path_item(1, &ctx.own_paths);
            if let Some(tenant_own_path) = tenant_own_path {
                // collect app own path
                let app_own_paths = app_ids.into_iter().map(|app_id| format!("{}/{}", &tenant_own_path, &app_id)).collect_vec();
                // find flow models
                flow_model_ids = Some(
                    FlowModelServ::find_id_items(
                        &FlowModelFilterReq {
                            basic: RbumBasicFilterReq {
                                with_sub_own_paths: true,
                                ..Default::default()
                            },
                            tags: tag.clone().map(|tag| vec![tag]),
                            own_paths: Some(app_own_paths),
                            ..Default::default()
                        },
                        None,
                        None,
                        funs,
                        ctx,
                    )
                    .await?,
                );
            }
        }
        let names = Self::find_detail_items(
            &FlowStateFilterReq {
                basic: RbumBasicFilterReq {
                    ids,
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                tag,
                flow_model_ids,
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
            key: state_detail.name.clone(),
            name: state_detail.name,
        })
        .collect_vec();
        Ok(names)
    }

    // For the old data migration, this function match id by old state name
    pub(crate) async fn match_state_id_by_name(tag: &str, flow_model_id: &str, mut name: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        if tag == "ISSUE" {
            name = match name {
                "待开始" => "待处理",
                "进行中" => "修复中",
                "存在风险" => "修复中",
                "已完成" => "已解决",
                "已关闭" => "已关闭",
                _ => name,
            };
        }
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
        let insts = FlowInstServ::find_detail(req.inst_ids.clone(), funs, ctx).await?;
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

    pub async fn merge_state_by_name(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let kind_state_id = RbumKindServ::get_rbum_kind_id_by_code(flow_constants::RBUM_KIND_STATE_CODE, funs)
            .await?
            .ok_or_else(|| funs.err().not_found("flow", "merge_state_by_name", "not found state kind", ""))?;
        let states = Self::find_items(
            &FlowStateFilterReq {
                basic: RbumBasicFilterReq {
                    ignore_scope: true,
                    own_paths: Some("".to_string()),
                    rel_ctx_owner: true,
                    rbum_kind_id: Some(kind_state_id),
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
        let mut exists_states: HashMap<String, FlowStateSummaryResp> = HashMap::new();
        for state in states {
            if let Some(exists_state) = exists_states.get(&state.name) {
                // flow inst
                flow_inst::Entity::update_many()
                    .col_expr(flow_inst::Column::CurrentStateId, Expr::value(exists_state.id.as_str()))
                    .filter(flow_inst::Column::CurrentStateId.eq(&state.id))
                    .exec(funs.db().raw_conn())
                    .await?;
                // flow model
                flow_model::Entity::update_many()
                    .col_expr(flow_model::Column::InitStateId, Expr::value(exists_state.id.as_str()))
                    .filter(flow_model::Column::InitStateId.eq(&state.id))
                    .exec(funs.db().raw_conn())
                    .await?;
                // flow transition
                flow_transition::Entity::update_many()
                    .col_expr(flow_transition::Column::FromFlowStateId, Expr::value(exists_state.id.as_str()))
                    .filter(flow_transition::Column::FromFlowStateId.eq(&state.id))
                    .exec(funs.db().raw_conn())
                    .await?;
                flow_transition::Entity::update_many()
                    .col_expr(flow_transition::Column::ToFlowStateId, Expr::value(exists_state.id.as_str()))
                    .filter(flow_transition::Column::ToFlowStateId.eq(&state.id))
                    .exec(funs.db().raw_conn())
                    .await?;
                // rbum rel
                join_all(
                    RbumRelServ::find_to_rels("FlowModelState", &state.id, None, None, funs, ctx)
                        .await?
                        .into_iter()
                        .map(|rel| async move {
                            let mock_ctx = TardisContext {
                                own_paths: rel.rel.own_paths,
                                ..Default::default()
                            };
                            FlowRelServ::add_simple_rel(
                                &FlowRelKind::FlowModelState,
                                &rel.rel.from_rbum_id,
                                &exists_state.id,
                                None,
                                None,
                                true,
                                true,
                                Some(rel.rel.ext),
                                funs,
                                &mock_ctx,
                            )
                            .await
                            .unwrap();
                            FlowRelServ::delete_simple_rel(&FlowRelKind::FlowModelState, &rel.rel.from_rbum_id, &rel.rel.to_rbum_item_id, funs, &mock_ctx).await.unwrap();
                        })
                        .collect::<Vec<_>>(),
                )
                .await;
                // flow state
                Self::modify_item(
                    &exists_state.id,
                    &mut FlowStateModifyReq {
                        tags: Some(vec![]),
                        ..Default::default()
                    },
                    funs,
                    ctx,
                )
                .await?;
                Self::delete_item(&state.id, funs, ctx).await?;
            } else {
                exists_states.insert(state.name.clone(), state);
            }
        }
        Ok(())
    }
}
