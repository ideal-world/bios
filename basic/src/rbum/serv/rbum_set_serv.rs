use async_trait::async_trait;
use tardis::basic::dto::TardisContext;
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::chrono::{DateTime, Utc};
use tardis::db::reldb_client::TardisRelDBlConnection;
use tardis::db::sea_orm::*;
use tardis::db::sea_query::*;
use tardis::TardisFuns;

use crate::rbum::domain::{rbum_item, rbum_set, rbum_set_cate, rbum_set_item};
use crate::rbum::dto::filer_dto::RbumBasicFilterReq;
use crate::rbum::dto::rbum_set_cate_dto::{RbumSetCateAddReq, RbumSetCateDetailResp, RbumSetCateModifyReq, RbumSetCateSummaryResp, RbumSetCateSummaryWithPidResp};
use crate::rbum::dto::rbum_set_dto::{RbumSetAddReq, RbumSetDetailResp, RbumSetModifyReq, RbumSetSummaryResp};
use crate::rbum::dto::rbum_set_item_dto::{RbumSetItemAddReq, RbumSetItemDetailResp, RbumSetItemModifyReq};
use crate::rbum::rbum_constants::RBUM_REL_CATE_SYS_CODE_NODE_LEN;
use crate::rbum::serv::rbum_crud_serv::{RbumCrudOperation, RbumCrudQueryPackage};
use crate::rbum::serv::rbum_item_serv::RbumItemServ;

pub struct RbumSetServ;
pub struct RbumSetCateServ;
pub struct RbumSetItemServ;

#[async_trait]
impl<'a> RbumCrudOperation<'a, rbum_set::ActiveModel, RbumSetAddReq, RbumSetModifyReq, RbumSetSummaryResp, RbumSetDetailResp> for RbumSetServ {
    fn get_table_name() -> &'static str {
        rbum_set::Entity.table_name()
    }

    async fn package_add(add_req: &RbumSetAddReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<rbum_set::ActiveModel> {
        Ok(rbum_set::ActiveModel {
            id: Set(TardisFuns::field.nanoid()),
            name: Set(add_req.name.to_string()),
            note: Set(add_req.note.as_ref().unwrap_or(&"".to_string()).to_string()),
            icon: Set(add_req.icon.as_ref().unwrap_or(&"".to_string()).to_string()),
            sort: Set(add_req.sort.unwrap_or(0)),
            tags: Set(add_req.tags.as_ref().unwrap_or(&"".to_string()).to_string()),
            scope_level: Set(add_req.scope_level),
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
        if let Some(tags) = &modify_req.tags {
            rbum_set.tags = Set(tags.to_string());
        }
        if let Some(scope_level) = modify_req.scope_level {
            rbum_set.scope_level = Set(scope_level);
        }
        Ok(rbum_set)
    }

    async fn package_query(is_detail: bool, filter: &RbumBasicFilterReq, _: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<SelectStatement> {
        let mut query = Query::select();
        query
            .columns(vec![
                (rbum_set::Entity, rbum_set::Column::Id),
                (rbum_set::Entity, rbum_set::Column::Name),
                (rbum_set::Entity, rbum_set::Column::Note),
                (rbum_set::Entity, rbum_set::Column::Icon),
                (rbum_set::Entity, rbum_set::Column::Sort),
                (rbum_set::Entity, rbum_set::Column::Tags),
                (rbum_set::Entity, rbum_set::Column::OwnPaths),
                (rbum_set::Entity, rbum_set::Column::Owner),
                (rbum_set::Entity, rbum_set::Column::CreateTime),
                (rbum_set::Entity, rbum_set::Column::UpdateTime),
                (rbum_set::Entity, rbum_set::Column::ScopeLevel),
            ])
            .from(rbum_set::Entity);

        if is_detail {
            query.query_with_safe(Self::get_table_name());
        }

        query.query_with_filter(Self::get_table_name(), filter, cxt);
        query.query_with_scope(Self::get_table_name(), cxt);

        Ok(query)
    }

    async fn before_delete_rbum(id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Self::check_ownership(id, db, cxt).await?;
        if db.count(Query::select().column(rbum_set_cate::Column::Id).from(rbum_set_cate::Entity).and_where(Expr::col(rbum_set_cate::Column::RelRbumSetId).eq(id))).await? > 0 {
            return Err(TardisError::BadRequest("can not delete rbum set when there are rbum set cate".to_string()));
        }
        if db.count(Query::select().column(rbum_set_item::Column::Id).from(rbum_set_item::Entity).and_where(Expr::col(rbum_set_item::Column::RelRbumSetId).eq(id))).await? > 0 {
            return Err(TardisError::BadRequest("can not delete rbum set when there are rbum set item".to_string()));
        }
        Ok(())
    }
}

impl<'a> RbumSetServ {
    pub async fn get_tree_all(rbum_set_id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<Vec<RbumSetCateSummaryWithPidResp>> {
        let mut resp = Self::do_get_tree(rbum_set_id, None, true, db, cxt).await?;
        resp.sort_by(|a, b| a.sys_code.cmp(&b.sys_code));
        resp.sort_by(|a, b| a.sort.cmp(&b.sort));
        Ok(resp
            .iter()
            .map(|r| RbumSetCateSummaryWithPidResp {
                id: r.id.to_string(),
                bus_code: r.bus_code.to_string(),
                name: r.name.to_string(),
                sort: r.sort,
                own_paths: r.own_paths.to_string(),
                create_time: r.create_time,
                update_time: r.update_time,
                scope_level: r.scope_level,
                pid: resp.iter().find(|i| i.sys_code == r.sys_code[..r.sys_code.len() - RBUM_REL_CATE_SYS_CODE_NODE_LEN]).map(|i| i.id.to_string()),
            })
            .collect())
    }

    pub async fn get_tree_by_level(
        rbum_set_id: &str,
        rbum_parent_set_cate_id: Option<&str>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<Vec<RbumSetCateSummaryResp>> {
        let resp = Self::do_get_tree(rbum_set_id, rbum_parent_set_cate_id, false, db, cxt).await?;
        Ok(resp
            .into_iter()
            .map(|r| RbumSetCateSummaryResp {
                id: r.id.to_string(),
                bus_code: r.bus_code.to_string(),
                name: r.name.to_string(),
                sort: r.sort,
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
        Self::check_scope(rbum_set_id, RbumSetServ::get_table_name(), db, cxt).await?;

        let mut query = Query::select();
        query
            .columns(vec![
                (rbum_set_cate::Column::Id),
                (rbum_set_cate::Column::SysCode),
                (rbum_set_cate::Column::BusCode),
                (rbum_set_cate::Column::Name),
                (rbum_set_cate::Column::Sort),
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
                Self::check_scope(parent_set_cate_id, RbumSetCateServ::get_table_name(), db, cxt).await?;
                let parent_sys_code = RbumSetCateServ::get_sys_code(parent_set_cate_id, db, cxt).await?;
                query.and_where(Expr::col(rbum_set_cate::Column::SysCode).like(format!("{}%", parent_sys_code).as_str()));
                query.and_where(Expr::expr(Func::char_length(Expr::col(rbum_set_cate::Column::SysCode))).eq((parent_sys_code.len() + RBUM_REL_CATE_SYS_CODE_NODE_LEN) as i32));
            } else {
                query.and_where(Expr::expr(Func::char_length(Expr::col(rbum_set_cate::Column::SysCode))).eq(RBUM_REL_CATE_SYS_CODE_NODE_LEN as i32));
            }
            query.order_by(rbum_set_cate::Column::Sort, Order::Asc);
        }

        db.find_dtos(&query).await
    }
}

#[async_trait]
impl<'a> RbumCrudOperation<'a, rbum_set_cate::ActiveModel, RbumSetCateAddReq, RbumSetCateModifyReq, RbumSetCateSummaryResp, RbumSetCateDetailResp> for RbumSetCateServ {
    fn get_table_name() -> &'static str {
        rbum_set_cate::Entity.table_name()
    }

    async fn package_add(add_req: &RbumSetCateAddReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<rbum_set_cate::ActiveModel> {
        let sys_code = if let Some(rbum_parent_cate_id) = &add_req.rbum_parent_cate_id {
            Self::package_sys_code(&add_req.rel_rbum_set_id, Some(rbum_parent_cate_id), true, db, cxt).await?
        } else if let Some(rbum_sibling_cate_id) = &add_req.rbum_sibling_cate_id {
            Self::package_sys_code(&add_req.rel_rbum_set_id, Some(rbum_sibling_cate_id), false, db, cxt).await?
        } else {
            Self::package_sys_code(&add_req.rel_rbum_set_id, None, false, db, cxt).await?
        };
        Ok(rbum_set_cate::ActiveModel {
            id: Set(TardisFuns::field.nanoid()),
            sys_code: Set(sys_code),
            bus_code: Set(add_req.bus_code.to_string()),
            name: Set(add_req.name.to_string()),
            sort: Set(add_req.sort.unwrap_or(0)),
            scope_level: Set(add_req.scope_level),
            rel_rbum_set_id: Set(add_req.rel_rbum_set_id.to_string()),
            ..Default::default()
        })
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
        if let Some(sort) = modify_req.sort {
            rbum_set_cate.sort = Set(sort);
        }
        if let Some(scope_level) = modify_req.scope_level {
            rbum_set_cate.scope_level = Set(scope_level);
        }
        Ok(rbum_set_cate)
    }

    async fn package_query(is_detail: bool, filter: &RbumBasicFilterReq, _: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<SelectStatement> {
        let mut query = Query::select();
        query
            .columns(vec![
                (rbum_set_cate::Entity, rbum_set_cate::Column::Id),
                (rbum_set_cate::Entity, rbum_set_cate::Column::BusCode),
                (rbum_set_cate::Entity, rbum_set_cate::Column::Name),
                (rbum_set_cate::Entity, rbum_set_cate::Column::Sort),
                (rbum_set_cate::Entity, rbum_set_cate::Column::OwnPaths),
                (rbum_set_cate::Entity, rbum_set_cate::Column::Owner),
                (rbum_set_cate::Entity, rbum_set_cate::Column::CreateTime),
                (rbum_set_cate::Entity, rbum_set_cate::Column::UpdateTime),
                (rbum_set_cate::Entity, rbum_set_cate::Column::ScopeLevel),
            ])
            .from(rbum_set_cate::Entity);

        if is_detail {
            query.query_with_safe(Self::get_table_name());
        }

        query.query_with_filter(Self::get_table_name(), filter, cxt);
        query.query_with_scope(Self::get_table_name(), cxt);

        Ok(query)
    }

    async fn before_add_rbum(add_req: &mut RbumSetCateAddReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Self::check_ownership_with_table_name(&add_req.rel_rbum_set_id, RbumSetServ::get_table_name(), db, cxt).await?;
        if let Some(rbum_parent_cate_id) = &add_req.rbum_parent_cate_id {
            Self::check_ownership(rbum_parent_cate_id, db, cxt).await?;
        }
        if let Some(rbum_sibling_cate_id) = &add_req.rbum_sibling_cate_id {
            Self::check_ownership(rbum_sibling_cate_id, db, cxt).await?;
        }
        Ok(())
    }

    async fn before_delete_rbum(id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Self::check_ownership(id, db, cxt).await?;
        if db
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
            return Err(TardisError::BadRequest("can not delete rbum set cate when there are rbum set item".to_string()));
        }
        Ok(())
    }
}

impl<'a> RbumSetCateServ {
    async fn package_sys_code(rbum_set_id: &str, rbum_set_cate_id: Option<&str>, is_next: bool, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<String> {
        if let Some(rbum_set_cate_id) = rbum_set_cate_id {
            let rel_sys_code = Self::get_sys_code(rbum_set_cate_id, db, cxt).await?;
            if is_next {
                Self::get_max_sys_code_by_level(rbum_set_id, Some(&rel_sys_code), db, cxt).await
            } else {
                let parent_sys_code = rel_sys_code[..rel_sys_code.len() - RBUM_REL_CATE_SYS_CODE_NODE_LEN].to_string();
                Self::get_max_sys_code_by_level(rbum_set_id, Some(&parent_sys_code), db, cxt).await
            }
        } else {
            Self::get_max_sys_code_by_level(rbum_set_id, None, db, cxt).await
        }
    }

    async fn get_max_sys_code_by_level(rbum_set_id: &str, parent_sys_code: Option<&str>, funs: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<String> {
        let mut query = Query::select();
        query.columns(vec![(rbum_set_cate::Column::SysCode)]).from(rbum_set_cate::Entity).and_where(Expr::col(rbum_set_cate::Column::RelRbumSetId).eq(rbum_set_id));

        if let Some(parent_sys_code) = parent_sys_code {
            query.and_where(Expr::col(rbum_set_cate::Column::SysCode).like(format!("{}%", parent_sys_code).as_str()));
            query.and_where(Expr::expr(Func::char_length(Expr::col(rbum_set_cate::Column::SysCode))).eq((parent_sys_code.len() + RBUM_REL_CATE_SYS_CODE_NODE_LEN) as i32));
        } else {
            // fetch max code in level 1
            query.and_where(Expr::expr(Func::char_length(Expr::col(rbum_set_cate::Column::SysCode))).eq(RBUM_REL_CATE_SYS_CODE_NODE_LEN as i32));
        }
        query.order_by(rbum_set_cate::Column::SysCode, Order::Desc);
        let max_sys_code = db.get_dto::<SysCodeResp>(&query).await?.map(|r| r.sys_code);
        if let Some(max_sys_code) = max_sys_code {
            if max_sys_code.len() != RBUM_REL_CATE_SYS_CODE_NODE_LEN {
                // if level N (N!-1) not empty
                let curr_level_sys_code = max_sys_code[max_sys_code.len() - RBUM_REL_CATE_SYS_CODE_NODE_LEN..].to_string();
                let parent_sys_code = max_sys_code[..max_sys_code.len() - RBUM_REL_CATE_SYS_CODE_NODE_LEN].to_string();
                let curr_level_sys_code =
                    TardisFuns::field.incr_by_base36(&curr_level_sys_code).ok_or_else(|| TardisError::BadRequest("the current number of nodes is saturated".to_string()))?;
                Ok(format!("{}{}", parent_sys_code, curr_level_sys_code))
            } else {
                // if level 1 not empty
                Ok(TardisFuns::field.incr_by_base36(&max_sys_code).ok_or_else(|| TardisError::BadRequest("the current number of nodes is saturated".to_string()))?)
            }
        } else if let Some(parent_sys_code) = parent_sys_code {
            // if level N (N!=1) is empty
            Ok(format!("{}{}", parent_sys_code, String::from_utf8(vec![b'a'; RBUM_REL_CATE_SYS_CODE_NODE_LEN])?))
        } else {
            // if level 1 is empty
            Ok(String::from_utf8(vec![b'a'; RBUM_REL_CATE_SYS_CODE_NODE_LEN])?)
        }
    }
}

impl<'a> RbumSetCateServ {
    async fn get_sys_code(rbum_set_cate_id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<String> {
        Self::check_ownership(rbum_set_cate_id, db, cxt).await?;
        let sys_code = db
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
impl<'a> RbumCrudOperation<'a, rbum_set_item::ActiveModel, RbumSetItemAddReq, RbumSetItemModifyReq, RbumSetItemDetailResp, RbumSetItemDetailResp> for RbumSetItemServ {
    fn get_table_name() -> &'static str {
        rbum_set_item::Entity.table_name()
    }

    async fn package_add(add_req: &RbumSetItemAddReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<rbum_set_item::ActiveModel> {
        let rel_sys_code = RbumSetCateServ::get_sys_code(add_req.rel_rbum_set_cate_id.as_str(), db, cxt).await?;
        Ok(rbum_set_item::ActiveModel {
            id: Set(format!("{}{}{}", add_req.rel_rbum_set_id, add_req.rel_rbum_set_cate_id, TardisFuns::field.nanoid())),
            rel_rbum_set_id: Set(add_req.rel_rbum_set_id.to_string()),
            rel_rbum_set_cate_code: Set(rel_sys_code),
            rel_rbum_item_id: Set(add_req.rel_rbum_item_id.to_string()),
            sort: Set(add_req.sort),
            ..Default::default()
        })
    }

    async fn package_modify(id: &str, modify_req: &RbumSetItemModifyReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<rbum_set_item::ActiveModel> {
        Ok(rbum_set_item::ActiveModel {
            id: Set(id.to_string()),
            sort: Set(modify_req.sort),
            ..Default::default()
        })
    }

    async fn package_query(_: bool, filter: &RbumBasicFilterReq, _: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<SelectStatement> {
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

        query.query_with_safe(Self::get_table_name());

        query.query_with_filter(Self::get_table_name(), filter, cxt);

        Ok(query)
    }

    async fn before_add_rbum(add_req: &mut RbumSetItemAddReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Self::check_ownership_with_table_name(&add_req.rel_rbum_set_id, RbumSetServ::get_table_name(), db, cxt).await?;
        Self::check_ownership_with_table_name(&add_req.rel_rbum_item_id, RbumItemServ::get_table_name(), db, cxt).await?;
        Self::check_ownership_with_table_name(&add_req.rel_rbum_set_cate_id, RbumSetCateServ::get_table_name(), db, cxt).await?;
        Ok(())
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
    pub sort: i32,

    pub own_paths: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: i32,
}
