use std::str::FromStr;

use async_trait::async_trait;
use tardis::basic::dto::TardisContext;
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisRelDBlConnection;
use tardis::db::sea_orm::*;
use tardis::db::sea_query::*;
use tardis::TardisFuns;

use crate::rbum::domain::{rbum_item, rbum_set, rbum_set_cate, rbum_set_item};
use crate::rbum::dto::filer_dto::RbumBasicFilterReq;
use crate::rbum::dto::rbum_set_cate_dto::{RbumSetCateAddReq, RbumSetCateDetailResp, RbumSetCateModifyReq, RbumSetCateSummaryResp};
use crate::rbum::dto::rbum_set_dto::{RbumSetAddReq, RbumSetDetailResp, RbumSetModifyReq, RbumSetSummaryResp};
use crate::rbum::dto::rbum_set_item_dto::{RbumSetItemAddReq, RbumSetItemDetailResp, RbumSetItemModifyReq};
use crate::rbum::enumeration::RbumScopeKind;
use crate::rbum::serv::rbum_crud_serv::{RbumCrudOperation, RbumCrudQueryPackage};

pub struct RbumSetServ;
pub struct RbumSetCateServ;
pub struct RbumSetItemServ;

const SYS_CODE_NODE_LEN: usize = 4;

#[async_trait]
impl<'a> RbumCrudOperation<'a, rbum_set::ActiveModel, RbumSetAddReq, RbumSetModifyReq, RbumSetSummaryResp, RbumSetDetailResp> for RbumSetServ {
    fn get_table_name() -> &'static str {
        rbum_set::Entity.table_name()
    }

    async fn package_add(add_req: &RbumSetAddReq, _: &TardisRelDBlConnection<'a>, _: &TardisContext) -> TardisResult<rbum_set::ActiveModel> {
        Ok(rbum_set::ActiveModel {
            name: Set(add_req.name.to_string()),
            note: Set(add_req.note.as_ref().unwrap_or(&"".to_string()).to_string()),
            icon: Set(add_req.icon.as_ref().unwrap_or(&"".to_string()).to_string()),
            sort: Set(add_req.sort.unwrap_or(0)),
            tags: Set(add_req.tags.as_ref().unwrap_or(&"".to_string()).to_string()),
            scope_kind: Set(add_req.scope_kind.as_ref().unwrap_or(&RbumScopeKind::App).to_string()),
            ..Default::default()
        })
    }

    async fn package_modify(id: &str, modify_req: &RbumSetModifyReq, _: &TardisRelDBlConnection<'a>, _: &TardisContext) -> TardisResult<rbum_set::ActiveModel> {
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
        if let Some(scope_kind) = &modify_req.scope_kind {
            rbum_set.scope_kind = Set(scope_kind.to_string());
        }
        Ok(rbum_set)
    }

    async fn package_query(is_detail: bool, filter: &RbumBasicFilterReq, _: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<SelectStatement> {
        let mut query = Query::select();
        query
            .columns(vec![
                (rbum_set::Entity, rbum_set::Column::Id),
                (rbum_set::Entity, rbum_set::Column::Name),
                (rbum_set::Entity, rbum_set::Column::Note),
                (rbum_set::Entity, rbum_set::Column::Icon),
                (rbum_set::Entity, rbum_set::Column::Sort),
                (rbum_set::Entity, rbum_set::Column::Tags),
                (rbum_set::Entity, rbum_set::Column::RelAppId),
                (rbum_set::Entity, rbum_set::Column::RelTenantId),
                (rbum_set::Entity, rbum_set::Column::UpdaterId),
                (rbum_set::Entity, rbum_set::Column::CreateTime),
                (rbum_set::Entity, rbum_set::Column::UpdateTime),
                (rbum_set::Entity, rbum_set::Column::ScopeKind),
            ])
            .from(rbum_set::Entity);

        if is_detail {
            query.query_with_safe(Self::get_table_name());
        }

        query.query_with_filter(Self::get_table_name(), filter, cxt);
        query.query_with_scope(Self::get_table_name(), cxt);

        Ok(query)
    }
}

#[async_trait]
impl<'a> RbumCrudOperation<'a, rbum_set_cate::ActiveModel, RbumSetCateAddReq, RbumSetCateModifyReq, RbumSetCateSummaryResp, RbumSetCateDetailResp> for RbumSetCateServ {
    fn get_table_name() -> &'static str {
        rbum_set_cate::Entity.table_name()
    }

    async fn package_add(add_req: &RbumSetCateAddReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<rbum_set_cate::ActiveModel> {
        let sys_code = if let Some(code) = &add_req.rbum_parent_cate_code {
            Self::package_sys_code(code, true)?
        } else if let Some(code) = &add_req.rbum_sibling_cate_code {
            Self::package_sys_code(code, false)?
        } else {
            return Err(TardisError::BadRequest("rbum_parent_cate_code or rbum_sibling_cate_code is required".to_string()));
        };
        Ok(rbum_set_cate::ActiveModel {
            sys_code: Set(sys_code.to_string()),
            bus_code: Set(add_req.bus_code.to_string()),
            name: Set(add_req.name.to_string()),
            sort: Set(add_req.sort.unwrap_or(0)),
            scope_kind: Set(add_req.scope_kind.as_ref().unwrap_or(&RbumScopeKind::App).to_string()),
            rel_rbum_set_id: Set(add_req.rel_rbum_set_id.to_string()),
            ..Default::default()
        })
    }

    async fn package_modify(id: &str, modify_req: &RbumSetCateModifyReq, _: &TardisRelDBlConnection<'a>, _: &TardisContext) -> TardisResult<rbum_set_cate::ActiveModel> {
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
        if let Some(scope_kind) = &modify_req.scope_kind {
            rbum_set_cate.scope_kind = Set(scope_kind.to_string());
        }
        Ok(rbum_set_cate)
    }

    async fn package_query(is_detail: bool, filter: &RbumBasicFilterReq, _: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<SelectStatement> {
        let mut query = Query::select();
        query
            .columns(vec![
                (rbum_set_cate::Entity, rbum_set_cate::Column::Id),
                (rbum_set_cate::Entity, rbum_set_cate::Column::BusCode),
                (rbum_set_cate::Entity, rbum_set_cate::Column::Name),
                (rbum_set_cate::Entity, rbum_set_cate::Column::Sort),
                (rbum_set_cate::Entity, rbum_set_cate::Column::RelAppId),
                (rbum_set_cate::Entity, rbum_set_cate::Column::RelTenantId),
                (rbum_set_cate::Entity, rbum_set_cate::Column::UpdaterId),
                (rbum_set_cate::Entity, rbum_set_cate::Column::CreateTime),
                (rbum_set_cate::Entity, rbum_set_cate::Column::UpdateTime),
                (rbum_set_cate::Entity, rbum_set_cate::Column::ScopeKind),
            ])
            .from(rbum_set_cate::Entity);

        if is_detail {
            query.query_with_safe(Self::get_table_name());
        }

        query.query_with_filter(Self::get_table_name(), filter, cxt);
        query.query_with_scope(Self::get_table_name(), cxt);

        Ok(query)
    }

    async fn before_add_rbum(add_req: &mut RbumSetCateAddReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Self::check_ownership_with_table_name(&add_req.rel_rbum_set_id, RbumSetServ::get_table_name(), db, cxt).await?;
        Ok(())
    }
}

impl<'a> RbumSetCateServ {
    fn package_sys_code(rel_sys_code: &str, is_next: bool) -> TardisResult<String> {
        if is_next {
            let new_sys_code_node = String::from_utf8(vec![b'a'; SYS_CODE_NODE_LEN])?;
            Ok(rel_sys_code.to_string() + &new_sys_code_node)
        } else {
            let sys_code_node = rel_sys_code[rel_sys_code.len() - SYS_CODE_NODE_LEN..].to_string();
            if let Some(sys_code_node) = TardisFuns::field.incr_by_base62(sys_code_node.as_str()) {
                Ok(rel_sys_code[..rel_sys_code.len() - SYS_CODE_NODE_LEN].to_string() + &sys_code_node)
            } else {
                Err(TardisError::BadRequest("the current number of nodes is saturated".to_string()))
            }
        }
    }
}

#[async_trait]
impl<'a> RbumCrudOperation<'a, rbum_set_item::ActiveModel, RbumSetItemAddReq, RbumSetItemModifyReq, RbumSetItemDetailResp, RbumSetItemDetailResp> for RbumSetItemServ {
    fn get_table_name() -> &'static str {
        rbum_set_item::Entity.table_name()
    }

    async fn package_add(add_req: &RbumSetItemAddReq, _: &TardisRelDBlConnection<'a>, _: &TardisContext) -> TardisResult<rbum_set_item::ActiveModel> {
        Ok(rbum_set_item::ActiveModel {
            rel_rbum_set_id: Set(add_req.rel_rbum_set_id.to_string()),
            rel_rbum_set_cate_code: Set(add_req.rel_rbum_set_cate_code.to_string()),
            rel_rbum_item_id: Set(add_req.rel_rbum_item_id.to_string()),
            sort: Set(add_req.sort),
            ..Default::default()
        })
    }

    async fn package_modify(id: &str, modify_req: &RbumSetItemModifyReq, _: &TardisRelDBlConnection<'a>, _: &TardisContext) -> TardisResult<rbum_set_item::ActiveModel> {
        Ok(rbum_set_item::ActiveModel {
            id: Set(id.to_string()),
            sort: Set(modify_req.sort),
            ..Default::default()
        })
    }

    async fn package_query(_: bool, filter: &RbumBasicFilterReq, _: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<SelectStatement> {
        let rel_item_table = Alias::new("relItem");

        let mut query = Query::select();
        query
            .columns(vec![
                (rbum_set_item::Entity, rbum_set_item::Column::Id),
                (rbum_set_item::Entity, rbum_set_item::Column::Sort),
                (rbum_set_item::Entity, rbum_set_item::Column::RelRbumSetCateCode),
                (rbum_set_item::Entity, rbum_set_item::Column::RelRbumItemId),
                (rbum_set_item::Entity, rbum_set_item::Column::RelAppId),
                (rbum_set_item::Entity, rbum_set_item::Column::RelTenantId),
                (rbum_set_item::Entity, rbum_set_item::Column::UpdaterId),
                (rbum_set_item::Entity, rbum_set_item::Column::CreateTime),
                (rbum_set_item::Entity, rbum_set_item::Column::UpdateTime),
            ])
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

    async fn before_add_rbum(add_req: &mut RbumSetItemAddReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Self::check_ownership_with_table_name(&add_req.rel_rbum_set_id, RbumSetServ::get_table_name(), db, cxt).await?;
        Self::check_ownership_with_table_name(&add_req.rel_rbum_item_id, RbumSetServ::get_table_name(), db, cxt).await?;
        Ok(())
    }
}

#[derive(Debug, FromQueryResult)]
struct SysCodeResp {
    pub sys_code: String,
}
