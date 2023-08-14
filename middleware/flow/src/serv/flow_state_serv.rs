use std::collections::HashMap;

use bios_basic::rbum::{
    dto::{
        rbum_filer_dto::RbumBasicFilterReq,
        rbum_item_dto::{RbumItemKernelAddReq, RbumItemKernelModifyReq},
    },
    serv::rbum_item_serv::RbumItemCrudOperation,
};
use tardis::{
    basic::{dto::TardisContext, field::TrimString, result::TardisResult},
    db::sea_orm::{
        sea_query::{Expr, SelectStatement},
        EntityName, Set,
    },
    serde_json::json,
    TardisFuns, TardisFunsInst,
};

use crate::{
    domain::flow_state,
    dto::flow_state_dto::{FlowStateAddReq, FlowStateDetailResp, FlowStateFilterReq, FlowStateKind, FlowStateModifyReq, FlowStateSummaryResp},
    flow_config::FlowBasicInfoManager,
};
use async_trait::async_trait;

use super::flow_model_serv::FlowModelServ;

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

    async fn before_modify_item(id: &str, modify_req: &mut FlowStateModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        // Modifications are allowed only where non-key fields are modified or not used
        if (modify_req.scope_level.is_some()
            || modify_req.disabled.is_some()
            || modify_req.sys_state.is_some()
            || modify_req.state_kind.is_some()
            || modify_req.kind_conf.is_some())
            && FlowModelServ::state_is_used(id, funs, ctx).await?
        {
            return Err(funs.err().conflict(&Self::get_obj_name(), "modify", &format!("state {id} already used"), "409-flow-state-already-used"));
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
        Ok(None)
    }

    async fn package_ext_query(query: &mut SelectStatement, _: bool, filter: &FlowStateFilterReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<()> {
        query.column((flow_state::Entity, flow_state::Column::Icon));
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
            query.and_where(Expr::col(flow_state::Column::Tags).like(format!("%{}%", tag)));
        }
        if let Some(state_kind) = &filter.state_kind {
            query.and_where(Expr::col(flow_state::Column::StateKind).eq(state_kind.clone()));
        }
        if let Some(template) = filter.template {
            query.and_where(Expr::col(flow_state::Column::Template).eq(template));
        }
        Ok(())
    }
}

impl FlowStateServ {
    pub(crate) async fn find_names(ids: Vec<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<HashMap<String, String>> {
        Self::find_id_name_items(
            &FlowStateFilterReq {
                basic: RbumBasicFilterReq {
                    ids: Some(ids),
                    ..Default::default()
                },
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await
    }

    // For the old data migration, this function match id by old state name
    pub(crate) async fn match_state_id_by_name(tag: &str, mut name: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        if tag == "ISSUE" {
            name = match name {
                "待开始" => "待处理",
                "进行中" => "修复中",
                "存在风险" => "修复中",
                "已完成" => "已解决",
                "已关闭" => "已关闭",
                _ => "",
            };
        }
        let state = Self::paginate_detail_items(
            &FlowStateFilterReq {
                basic: RbumBasicFilterReq {
                    name: Some(name.to_string()),
                    ..Default::default()
                },
                tag: None,
                ..Default::default()
            },
            1,
            1,
            None,
            None,
            funs,
            ctx,
        )
        .await?
        .records
        .pop();
        if let Some(state) = state {
            Ok(state.name)
        } else {
            Err(funs.err().not_found("flow_state_serv", "find_state_id_by_name", "state_id not match", ""))
        }
    }
}
