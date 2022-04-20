use async_trait::async_trait;
use tardis::basic::dto::{TardisContext, TardisFunsInst};
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::chrono::{DateTime, Utc};
use tardis::db::sea_orm::*;
use tardis::db::sea_query::*;
use tardis::TardisFuns;

use crate::rbum::domain::{rbum_cert, rbum_item, rbum_rel, rbum_set, rbum_set_cate, rbum_set_item};
use crate::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumSetItemFilterReq};
use crate::rbum::dto::rbum_set_cate_dto::{RbumSetCateAddReq, RbumSetCateDetailResp, RbumSetCateModifyReq, RbumSetCateSummaryResp, RbumSetCateSummaryWithPidResp};
use crate::rbum::dto::rbum_set_dto::{RbumSetAddReq, RbumSetDetailResp, RbumSetModifyReq, RbumSetSummaryResp};
use crate::rbum::dto::rbum_set_item_dto::{RbumSetItemAddReq, RbumSetItemDetailResp, RbumSetItemModifyReq};
use crate::rbum::rbum_config::RbumConfigManager;
use crate::rbum::rbum_enumeration::{RbumCertRelKind, RbumRelFromKind, RbumScopeLevelKind};
use crate::rbum::serv::rbum_cert_serv::RbumCertServ;
use crate::rbum::serv::rbum_crud_serv::{RbumCrudOperation, RbumCrudQueryPackage};
use crate::rbum::serv::rbum_item_serv::RbumItemServ;
use crate::rbum::serv::rbum_rel_serv::RbumRelServ;

pub struct RbumSetServ;
pub struct RbumSetCateServ;
pub struct RbumSetItemServ;

#[async_trait]
impl<'a> RbumCrudOperation<'a, rbum_set::ActiveModel, RbumSetAddReq, RbumSetModifyReq, RbumSetSummaryResp, RbumSetDetailResp, RbumBasicFilterReq> for RbumSetServ {
    fn get_table_name() -> &'static str {
        rbum_set::Entity.table_name()
    }

    async fn package_add(add_req: &RbumSetAddReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<rbum_set::ActiveModel> {
        Ok(rbum_set::ActiveModel {
            id: Set(TardisFuns::field.nanoid()),
            code: Set(add_req.code.to_string()),
            name: Set(add_req.name.to_string()),
            note: Set(add_req.note.as_ref().unwrap_or(&"".to_string()).to_string()),
            icon: Set(add_req.icon.as_ref().unwrap_or(&"".to_string()).to_string()),
            sort: Set(add_req.sort.unwrap_or(0)),
            ext: Set(add_req.ext.as_ref().unwrap_or(&"".to_string()).to_string()),
            scope_level: Set(add_req.scope_level.to_int()),
            disabled: Set(add_req.disabled.unwrap_or(false)),
            ..Default::default()
        })
    }

    async fn package_modify(id: &str, modify_req: &RbumSetModifyReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<rbum_set::ActiveModel> {
        let mut rbum_set = rbum_set::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(name) = &modify_req.name {
            rbum_set.name = Set(name.to_string());
        }
        if let Some(note) = &modify_req.note {
            rbum_set.note = Set(note.to_string());
        }
        if let Some(icon) = &modify_req.icon {
            rbum_set.icon = Set(icon.to_string());
        }
        if let Some(sort) = modify_req.sort {
            rbum_set.sort = Set(sort);
        }
        if let Some(ext) = &modify_req.ext {
            rbum_set.ext = Set(ext.to_string());
        }
        if let Some(scope_level) = &modify_req.scope_level {
            rbum_set.scope_level = Set(scope_level.to_int());
        }
        if let Some(disabled) = modify_req.disabled {
            rbum_set.disabled = Set(disabled);
        }
        Ok(rbum_set)
    }

    async fn before_delete_rbum(id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Self::check_ownership(id, funs, cxt).await?;
        Self::check_exist_before_delete(id, RbumSetCateServ::get_table_name(), rbum_set_cate::Column::RelRbumSetId.as_str(), funs).await?;
        Self::check_exist_before_delete(id, RbumSetItemServ::get_table_name(), rbum_set_item::Column::RelRbumSetId.as_str(), funs).await?;
        Self::check_exist_with_cond_before_delete(
            RbumRelServ::get_table_name(),
            Cond::any()
                .add(Cond::all().add(Expr::col(rbum_rel::Column::FromRbumKind).eq(RbumRelFromKind::Set.to_int())).add(Expr::col(rbum_rel::Column::FromRbumId).eq(id)))
                .add(Expr::col(rbum_rel::Column::ToRbumItemId).eq(id)),
            funs,
        )
        .await?;
        Self::check_exist_with_cond_before_delete(
            RbumCertServ::get_table_name(),
            Cond::all().add(Expr::col(rbum_cert::Column::RelRbumKind).eq(RbumCertRelKind::Set.to_int())).add(Expr::col(rbum_cert::Column::RelRbumId).eq(id)),
            funs,
        )
        .await?;
        Ok(())
    }

    async fn package_query(is_detail: bool, filter: &RbumBasicFilterReq, _: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<SelectStatement> {
        let mut query = Query::select();
        query
            .columns(vec![
                (rbum_set::Entity, rbum_set::Column::Id),
                (rbum_set::Entity, rbum_set::Column::Code),
                (rbum_set::Entity, rbum_set::Column::Name),
                (rbum_set::Entity, rbum_set::Column::Note),
                (rbum_set::Entity, rbum_set::Column::Icon),
                (rbum_set::Entity, rbum_set::Column::Sort),
                (rbum_set::Entity, rbum_set::Column::Ext),
                (rbum_set::Entity, rbum_set::Column::Disabled),
                (rbum_set::Entity, rbum_set::Column::OwnPaths),
                (rbum_set::Entity, rbum_set::Column::Owner),
                (rbum_set::Entity, rbum_set::Column::CreateTime),
                (rbum_set::Entity, rbum_set::Column::UpdateTime),
                (rbum_set::Entity, rbum_set::Column::ScopeLevel),
            ])
            .from(rbum_set::Entity);
        query.with_filter(Self::get_table_name(), filter, is_detail, true, cxt);
        Ok(query)
    }
}

impl<'a> RbumSetServ {
    pub async fn get_tree_all(rbum_set_id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<Vec<RbumSetCateSummaryWithPidResp>> {
        let set_cate_sys_code_node_len = RbumConfigManager::get(funs.module_code())?.set_cate_sys_code_node_len;
        let mut resp = Self::do_get_tree(rbum_set_id, None, true, funs, cxt).await?;
        resp.sort_by(|a, b| a.sys_code.cmp(&b.sys_code));
        resp.sort_by(|a, b| a.sort.cmp(&b.sort));
        Ok(resp
            .iter()
            .map(|r| RbumSetCateSummaryWithPidResp {
                id: r.id.to_string(),
                bus_code: r.bus_code.to_string(),
                name: r.name.to_string(),
                icon: r.icon.to_string(),
                sort: r.sort,
                ext: r.ext.to_string(),
                own_paths: r.own_paths.to_string(),
                create_time: r.create_time,
                update_time: r.update_time,
                scope_level: r.scope_level.clone(),
                pid: resp.iter().find(|i| i.sys_code == r.sys_code[..r.sys_code.len() - set_cate_sys_code_node_len]).map(|i| i.id.to_string()),
            })
            .collect())
    }

    pub async fn get_tree_by_level(
        rbum_set_id: &str,
        rbum_parent_set_cate_id: Option<&str>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<Vec<RbumSetCateSummaryResp>> {
        let resp = Self::do_get_tree(rbum_set_id, rbum_parent_set_cate_id, false, funs, cxt).await?;
        Ok(resp
            .into_iter()
            .map(|r| RbumSetCateSummaryResp {
                id: r.id.to_string(),
                bus_code: r.bus_code.to_string(),
                name: r.name.to_string(),
                icon: r.icon.to_string(),
                sort: r.sort,
                ext: r.ext.to_string(),
                own_paths: r.own_paths.to_string(),
                create_time: r.create_time,
                update_time: r.update_time,
                scope_level: r.scope_level,
            })
            .collect())
    }

    async fn do_get_tree(
        rbum_set_id: &str,
        rbum_parent_set_cate_id: Option<&str>,
        fetch_all: bool,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<Vec<RbumSetCateWithLevelResp>> {
        Self::check_scope(rbum_set_id, RbumSetServ::get_table_name(), funs, cxt).await?;
        let mut query = Query::select();
        query
            .columns(vec![
                (rbum_set_cate::Column::Id),
                (rbum_set_cate::Column::SysCode),
                (rbum_set_cate::Column::BusCode),
                (rbum_set_cate::Column::Name),
                (rbum_set_cate::Column::Icon),
                (rbum_set_cate::Column::Sort),
                (rbum_set_cate::Column::Ext),
                (rbum_set_cate::Column::OwnPaths),
                (rbum_set_cate::Column::Owner),
                (rbum_set_cate::Column::CreateTime),
                (rbum_set_cate::Column::UpdateTime),
                (rbum_set_cate::Column::ScopeLevel),
            ])
            .from(rbum_set_cate::Entity)
            .and_where(Expr::col(rbum_set_cate::Column::RelRbumSetId).eq(rbum_set_id));

        if !fetch_all {
            if let Some(parent_set_cate_id) = rbum_parent_set_cate_id {
                Self::check_scope(parent_set_cate_id, RbumSetCateServ::get_table_name(), funs, cxt).await?;
                let parent_sys_code = RbumSetCateServ::get_sys_code(parent_set_cate_id, funs, cxt).await?;
                query.and_where(Expr::col(rbum_set_cate::Column::SysCode).like(format!("{}%", parent_sys_code).as_str()));
                query.and_where(
                    Expr::expr(Func::char_length(Expr::col(rbum_set_cate::Column::SysCode)))
                        .eq((parent_sys_code.len() + RbumConfigManager::get(funs.module_code())?.set_cate_sys_code_node_len) as i32),
                );
            } else {
                query.and_where(
                    Expr::expr(Func::char_length(Expr::col(rbum_set_cate::Column::SysCode))).eq(RbumConfigManager::get(funs.module_code())?.set_cate_sys_code_node_len as i32),
                );
            }
            query.order_by(rbum_set_cate::Column::Sort, Order::Asc);
        }
        funs.db().find_dtos(&query).await
    }
}

#[async_trait]
impl<'a> RbumCrudOperation<'a, rbum_set_cate::ActiveModel, RbumSetCateAddReq, RbumSetCateModifyReq, RbumSetCateSummaryResp, RbumSetCateDetailResp, RbumBasicFilterReq>
    for RbumSetCateServ
{
    fn get_table_name() -> &'static str {
        rbum_set_cate::Entity.table_name()
    }

    async fn package_add(add_req: &RbumSetCateAddReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<rbum_set_cate::ActiveModel> {
        let sys_code = if let Some(rbum_parent_cate_id) = &add_req.rbum_parent_cate_id {
            Self::package_sys_code(&add_req.rel_rbum_set_id, Some(rbum_parent_cate_id), funs, cxt).await?
        } else {
            Self::package_sys_code(&add_req.rel_rbum_set_id, None, funs, cxt).await?
        };
        Ok(rbum_set_cate::ActiveModel {
            id: Set(TardisFuns::field.nanoid()),
            sys_code: Set(sys_code),
            bus_code: Set(add_req.bus_code.to_string()),
            name: Set(add_req.name.to_string()),
            icon: Set(add_req.icon.as_ref().unwrap_or(&"".to_string()).to_string()),
            sort: Set(add_req.sort.unwrap_or(0)),
            ext: Set(add_req.ext.as_ref().unwrap_or(&"".to_string()).to_string()),
            rel_rbum_set_id: Set(add_req.rel_rbum_set_id.to_string()),
            scope_level: Set(add_req.scope_level.to_int()),
            ..Default::default()
        })
    }

    async fn before_add_rbum(add_req: &mut RbumSetCateAddReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Self::check_scope(&add_req.rel_rbum_set_id, RbumSetServ::get_table_name(), funs, cxt).await?;
        if let Some(rbum_parent_cate_id) = &add_req.rbum_parent_cate_id {
            Self::check_scope(rbum_parent_cate_id, RbumSetCateServ::get_table_name(), funs, cxt).await?;
        }
        Ok(())
    }

    async fn package_modify(id: &str, modify_req: &RbumSetCateModifyReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<rbum_set_cate::ActiveModel> {
        let mut rbum_set_cate = rbum_set_cate::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(bus_code) = &modify_req.bus_code {
            rbum_set_cate.bus_code = Set(bus_code.to_string());
        }
        if let Some(name) = &modify_req.name {
            rbum_set_cate.name = Set(name.to_string());
        }
        if let Some(icon) = &modify_req.icon {
            rbum_set_cate.icon = Set(icon.to_string());
        }
        if let Some(sort) = modify_req.sort {
            rbum_set_cate.sort = Set(sort);
        }
        if let Some(ext) = &modify_req.ext {
            rbum_set_cate.ext = Set(ext.to_string());
        }
        if let Some(scope_level) = &modify_req.scope_level {
            rbum_set_cate.scope_level = Set(scope_level.to_int());
        }
        Ok(rbum_set_cate)
    }

    async fn before_delete_rbum(id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Self::check_ownership(id, funs, cxt).await?;
        if funs
            .db()
            .count(
                Query::select()
                    .column((rbum_set_item::Entity, rbum_set_item::Column::Id))
                    .from(rbum_set_item::Entity)
                    .inner_join(
                        rbum_set_cate::Entity,
                        Expr::tbl(rbum_set_cate::Entity, rbum_set_cate::Column::SysCode).equals(rbum_set_item::Entity, rbum_set_item::Column::RelRbumSetCateCode),
                    )
                    .and_where(Expr::tbl(rbum_set_cate::Entity, rbum_set_cate::Column::Id).eq(id)),
            )
            .await?
            > 0
        {
            return Err(TardisError::BadRequest("Can not delete rbum_set_cate when there are rbum_set_item".to_string()));
        }
        // TODO check parent
        Ok(())
    }

    async fn package_query(is_detail: bool, filter: &RbumBasicFilterReq, _: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<SelectStatement> {
        let mut query = Query::select();
        query
            .columns(vec![
                (rbum_set_cate::Entity, rbum_set_cate::Column::Id),
                (rbum_set_cate::Entity, rbum_set_cate::Column::BusCode),
                (rbum_set_cate::Entity, rbum_set_cate::Column::Name),
                (rbum_set_cate::Entity, rbum_set_cate::Column::Icon),
                (rbum_set_cate::Entity, rbum_set_cate::Column::Sort),
                (rbum_set_cate::Entity, rbum_set_cate::Column::Ext),
                (rbum_set_cate::Entity, rbum_set_cate::Column::OwnPaths),
                (rbum_set_cate::Entity, rbum_set_cate::Column::Owner),
                (rbum_set_cate::Entity, rbum_set_cate::Column::CreateTime),
                (rbum_set_cate::Entity, rbum_set_cate::Column::UpdateTime),
                (rbum_set_cate::Entity, rbum_set_cate::Column::ScopeLevel),
            ])
            .from(rbum_set_cate::Entity);
        query.with_filter(Self::get_table_name(), filter, is_detail, true, cxt);
        Ok(query)
    }
}

impl<'a> RbumSetCateServ {
    async fn package_sys_code(rbum_set_id: &str, rbum_set_parent_cate_id: Option<&str>, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<String> {
        if let Some(rbum_set_parent_cate_id) = rbum_set_parent_cate_id {
            let rel_parent_sys_code = Self::get_sys_code(rbum_set_parent_cate_id, funs, cxt).await?;
            Self::get_max_sys_code_by_level(rbum_set_id, Some(&rel_parent_sys_code), funs, cxt).await
        } else {
            Self::get_max_sys_code_by_level(rbum_set_id, None, funs, cxt).await
        }
    }

    async fn get_max_sys_code_by_level(rbum_set_id: &str, parent_sys_code: Option<&str>, funs: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<String> {
        let set_cate_sys_code_node_len = RbumConfigManager::get(funs.module_code())?.set_cate_sys_code_node_len;
        let mut query = Query::select();
        query.columns(vec![(rbum_set_cate::Column::SysCode)]).from(rbum_set_cate::Entity).and_where(Expr::col(rbum_set_cate::Column::RelRbumSetId).eq(rbum_set_id));

        if let Some(parent_sys_code) = parent_sys_code {
            query.and_where(Expr::col(rbum_set_cate::Column::SysCode).like(format!("{}%", parent_sys_code).as_str()));
            query.and_where(Expr::expr(Func::char_length(Expr::col(rbum_set_cate::Column::SysCode))).eq((parent_sys_code.len() + set_cate_sys_code_node_len) as i32));
        } else {
            // fetch max code in level 1
            query.and_where(Expr::expr(Func::char_length(Expr::col(rbum_set_cate::Column::SysCode))).eq(set_cate_sys_code_node_len as i32));
        }
        query.order_by(rbum_set_cate::Column::SysCode, Order::Desc);
        let max_sys_code = funs.db().get_dto::<SysCodeResp>(&query).await?.map(|r| r.sys_code);
        if let Some(max_sys_code) = max_sys_code {
            if max_sys_code.len() != set_cate_sys_code_node_len {
                // if level N (N!-1) not empty
                let curr_level_sys_code = max_sys_code[max_sys_code.len() - set_cate_sys_code_node_len..].to_string();
                let parent_sys_code = max_sys_code[..max_sys_code.len() - set_cate_sys_code_node_len].to_string();
                let curr_level_sys_code =
                    TardisFuns::field.incr_by_base36(&curr_level_sys_code).ok_or_else(|| TardisError::BadRequest("the current number of nodes is saturated".to_string()))?;
                Ok(format!("{}{}", parent_sys_code, curr_level_sys_code))
            } else {
                // if level 1 not empty
                Ok(TardisFuns::field.incr_by_base36(&max_sys_code).ok_or_else(|| TardisError::BadRequest("the current number of nodes is saturated".to_string()))?)
            }
        } else if let Some(parent_sys_code) = parent_sys_code {
            // if level N (N!=1) is empty
            Ok(format!("{}{}", parent_sys_code, String::from_utf8(vec![b'a'; set_cate_sys_code_node_len])?))
        } else {
            // if level 1 is empty
            Ok(String::from_utf8(vec![b'a'; set_cate_sys_code_node_len])?)
        }
    }
}

impl<'a> RbumSetCateServ {
    async fn get_sys_code(rbum_set_cate_id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<String> {
        Self::check_ownership(rbum_set_cate_id, funs, cxt).await?;
        let sys_code = funs
            .db()
            .get_dto::<SysCodeResp>(
                Query::select().column(rbum_set_cate::Column::SysCode).from(rbum_set_cate::Entity).and_where(Expr::col(rbum_set_cate::Column::Id).eq(rbum_set_cate_id)),
            )
            .await?
            .ok_or_else(|| TardisError::NotFound(format!("set cate {} does not exist", rbum_set_cate_id)))?
            .sys_code;
        Ok(sys_code)
    }
}

#[async_trait]
impl<'a> RbumCrudOperation<'a, rbum_set_item::ActiveModel, RbumSetItemAddReq, RbumSetItemModifyReq, RbumSetItemDetailResp, RbumSetItemDetailResp, RbumSetItemFilterReq>
    for RbumSetItemServ
{
    fn get_table_name() -> &'static str {
        rbum_set_item::Entity.table_name()
    }

    async fn package_add(add_req: &RbumSetItemAddReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<rbum_set_item::ActiveModel> {
        let rel_sys_code = RbumSetCateServ::get_sys_code(add_req.rel_rbum_set_cate_id.as_str(), funs, cxt).await?;
        Ok(rbum_set_item::ActiveModel {
            id: Set(TardisFuns::field.nanoid()),
            rel_rbum_set_id: Set(add_req.rel_rbum_set_id.to_string()),
            rel_rbum_set_cate_code: Set(rel_sys_code),
            rel_rbum_item_id: Set(add_req.rel_rbum_item_id.to_string()),
            sort: Set(add_req.sort),
            ..Default::default()
        })
    }

    async fn before_add_rbum(add_req: &mut RbumSetItemAddReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Self::check_scope(&add_req.rel_rbum_set_id, RbumSetServ::get_table_name(), funs, cxt).await?;
        Self::check_scope(&add_req.rel_rbum_item_id, RbumItemServ::get_table_name(), funs, cxt).await?;
        Self::check_scope(&add_req.rel_rbum_set_cate_id, RbumSetCateServ::get_table_name(), funs, cxt).await?;
        Ok(())
    }

    async fn package_modify(id: &str, modify_req: &RbumSetItemModifyReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<rbum_set_item::ActiveModel> {
        Ok(rbum_set_item::ActiveModel {
            id: Set(id.to_string()),
            sort: Set(modify_req.sort),
            ..Default::default()
        })
    }

    async fn package_query(_: bool, filter: &RbumSetItemFilterReq, _: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<SelectStatement> {
        let rel_item_table = Alias::new("relItem");

        let mut query = Query::select();
        query
            .columns(vec![
                (rbum_set_item::Entity, rbum_set_item::Column::Id),
                (rbum_set_item::Entity, rbum_set_item::Column::Sort),
                (rbum_set_item::Entity, rbum_set_item::Column::RelRbumItemId),
                (rbum_set_item::Entity, rbum_set_item::Column::OwnPaths),
                (rbum_set_item::Entity, rbum_set_item::Column::Owner),
                (rbum_set_item::Entity, rbum_set_item::Column::CreateTime),
                (rbum_set_item::Entity, rbum_set_item::Column::UpdateTime),
            ])
            .expr_as(Expr::tbl(rbum_set_cate::Entity, rbum_set_cate::Column::Id), Alias::new("rel_rbum_set_cate_id"))
            .expr_as(Expr::tbl(rbum_set_cate::Entity, rbum_set_cate::Column::Name), Alias::new("rel_rbum_set_cate_name"))
            .expr_as(Expr::tbl(rel_item_table.clone(), rbum_item::Column::Name), Alias::new("rel_rbum_item_name"))
            .from(rbum_set_item::Entity)
            .inner_join(
                rbum_set_cate::Entity,
                Expr::tbl(rbum_set_cate::Entity, rbum_set_cate::Column::SysCode).equals(rbum_set_item::Entity, rbum_set_item::Column::RelRbumSetCateCode),
            )
            .join_as(
                JoinType::InnerJoin,
                rbum_item::Entity,
                rel_item_table.clone(),
                Expr::tbl(rel_item_table, rbum_item::Column::Id).equals(rbum_set_item::Entity, rbum_set_item::Column::RelRbumItemId),
            );
        query.and_where(Expr::tbl(rbum_set_item::Entity, rbum_set_item::Column::RelRbumSetId).eq(filter.rel_rbum_set_id.as_str()));
        if let Some(rel_rbum_set_cate_id) = &filter.rel_rbum_set_cate_id {
            query.and_where(Expr::tbl(rbum_set_cate::Entity, rbum_set_cate::Column::Id).eq(rel_rbum_set_cate_id.to_string()));
        }
        query.with_filter(Self::get_table_name(), &filter.basic, true, false, cxt);
        Ok(query)
    }
}

#[derive(Debug, FromQueryResult)]
struct SysCodeResp {
    pub sys_code: String,
}

#[derive(Debug, FromQueryResult)]
struct RbumSetCateWithLevelResp {
    pub id: String,
    pub sys_code: String,
    pub bus_code: String,
    pub name: String,
    pub icon: String,
    pub sort: u32,
    pub ext: String,

    pub own_paths: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
}
