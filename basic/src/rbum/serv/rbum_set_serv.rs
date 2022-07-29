use std::time::Duration;

use async_trait::async_trait;
use itertools::Itertools;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::chrono::{DateTime, Utc};
use tardis::db::sea_orm;
use tardis::db::sea_orm::sea_query::*;
use tardis::db::sea_orm::*;
use tardis::tokio::time::sleep;
use tardis::{TardisFuns, TardisFunsInst};

use crate::rbum::domain::{rbum_cert, rbum_item, rbum_rel, rbum_set, rbum_set_cate, rbum_set_item};
use crate::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumSetCateFilterReq, RbumSetFilterReq, RbumSetItemFilterReq};
use crate::rbum::dto::rbum_set_cate_dto::{RbumSetCateAddReq, RbumSetCateDetailResp, RbumSetCateModifyReq, RbumSetCateSummaryResp, RbumSetItemInfoResp, RbumSetTreeResp};
use crate::rbum::dto::rbum_set_dto::{RbumSetAddReq, RbumSetDetailResp, RbumSetModifyReq, RbumSetPathResp, RbumSetSummaryResp};
use crate::rbum::dto::rbum_set_item_dto::{RbumSetItemAddReq, RbumSetItemDetailResp, RbumSetItemModifyReq, RbumSetItemSummaryResp};
use crate::rbum::rbum_config::RbumConfigApi;
use crate::rbum::rbum_enumeration::{RbumCertRelKind, RbumRelFromKind, RbumScopeLevelKind, RbumSetCateLevelQueryKind};
use crate::rbum::serv::rbum_cert_serv::RbumCertServ;
use crate::rbum::serv::rbum_crud_serv::{RbumCrudOperation, RbumCrudQueryPackage};
use crate::rbum::serv::rbum_item_serv::RbumItemServ;
use crate::rbum::serv::rbum_rel_serv::RbumRelServ;

pub struct RbumSetServ;

pub struct RbumSetCateServ;

pub struct RbumSetItemServ;

#[async_trait]
impl<'a> RbumCrudOperation<'a, rbum_set::ActiveModel, RbumSetAddReq, RbumSetModifyReq, RbumSetSummaryResp, RbumSetDetailResp, RbumSetFilterReq> for RbumSetServ {
    fn get_table_name() -> &'static str {
        rbum_set::Entity.table_name()
    }

    async fn package_add(add_req: &RbumSetAddReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<rbum_set::ActiveModel> {
        Ok(rbum_set::ActiveModel {
            id: Set(TardisFuns::field.nanoid()),
            code: Set(add_req.code.to_string()),
            kind: Set(add_req.kind.to_string()),
            name: Set(add_req.name.to_string()),
            note: Set(add_req.note.as_ref().unwrap_or(&"".to_string()).to_string()),
            icon: Set(add_req.icon.as_ref().unwrap_or(&"".to_string()).to_string()),
            sort: Set(add_req.sort.unwrap_or(0)),
            ext: Set(add_req.ext.as_ref().unwrap_or(&"".to_string()).to_string()),
            scope_level: Set(add_req.scope_level.as_ref().unwrap_or(&RbumScopeLevelKind::Private).to_int()),
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

    async fn before_delete_rbum(id: &str, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<Option<RbumSetDetailResp>> {
        Self::check_ownership(id, funs, ctx).await?;
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
        let result = Self::peek_rbum(
            id,
            &RbumSetFilterReq {
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
        let key = &format!("{}{}", funs.rbum_conf_cache_key_set_code_(), result.code);
        funs.cache().del(key).await?;
        Ok(None)
    }

    async fn package_query(is_detail: bool, filter: &RbumSetFilterReq, _: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<SelectStatement> {
        let mut query = Query::select();
        query
            .columns(vec![
                (rbum_set::Entity, rbum_set::Column::Id),
                (rbum_set::Entity, rbum_set::Column::Code),
                (rbum_set::Entity, rbum_set::Column::Kind),
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
        if let Some(kind) = &filter.kind {
            query.and_where(Expr::tbl(rbum_set::Entity, rbum_set::Column::Kind).eq(kind.to_string()));
        }
        if let Some(rbum_item_rel_filter_req) = &filter.rel {
            if rbum_item_rel_filter_req.rel_by_from {
                query.inner_join(
                    rbum_rel::Entity,
                    Expr::tbl(rbum_rel::Entity, rbum_rel::Column::FromRbumId).equals(rbum_set::Entity, rbum_set::Column::Id),
                );
                if let Some(rel_item_id) = &rbum_item_rel_filter_req.rel_item_id {
                    query.and_where(Expr::tbl(rbum_rel::Entity, rbum_rel::Column::ToRbumItemId).eq(rel_item_id.to_string()));
                }
            } else {
                query.inner_join(
                    rbum_rel::Entity,
                    Expr::tbl(rbum_rel::Entity, rbum_rel::Column::ToRbumItemId).equals(rbum_set::Entity, rbum_set::Column::Id),
                );
                if let Some(rel_item_id) = &rbum_item_rel_filter_req.rel_item_id {
                    query.and_where(Expr::tbl(rbum_rel::Entity, rbum_rel::Column::FromRbumId).eq(rel_item_id.to_string()));
                }
            }
            if let Some(tag) = &rbum_item_rel_filter_req.tag {
                query.and_where(Expr::tbl(rbum_rel::Entity, rbum_rel::Column::Tag).eq(tag.to_string()));
            }
            if let Some(from_rbum_kind) = &rbum_item_rel_filter_req.from_rbum_kind {
                query.and_where(Expr::tbl(rbum_rel::Entity, rbum_rel::Column::FromRbumKind).eq(from_rbum_kind.to_int()));
            }
        }
        query.with_filter(Self::get_table_name(), &filter.basic, is_detail, true, ctx);
        Ok(query)
    }
}

impl<'a> RbumSetServ {
    pub async fn get_tree(rbum_set_id: &str, rbum_parent_set_cate_id: Option<&str>, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<Vec<RbumSetTreeResp>> {
        let set_cate_sys_code_node_len = funs.rbum_conf_set_cate_sys_code_node_len();
        let mut resp = Self::do_get_tree(rbum_set_id, rbum_parent_set_cate_id, true, funs, ctx).await?;
        resp.sort_by(|a, b| a.sys_code.cmp(&b.sys_code));
        resp.sort_by(|a, b| a.sort.cmp(&b.sort));
        let rbum_set_items = RbumSetItemServ::find_rbums(
            &RbumSetItemFilterReq {
                basic: RbumBasicFilterReq {
                    // set cate item is only used for connection,
                    // it will do scope checking on both ends of the connection when it is created,
                    // so we can release the own paths restriction here to avoid query errors.
                    // E.g.
                    // A tenant creates a set cate with scope_level=0 and associates a item,
                    // if the permission is not released, all app will not query the data.
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    desc_by_sort: Some(true),
                    ..Default::default()
                },
                rel_rbum_set_id: Some(rbum_set_id.to_string()),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        Ok(resp
            .iter()
            .map(|r| RbumSetTreeResp {
                id: r.id.to_string(),
                bus_code: r.bus_code.to_string(),
                name: r.name.to_string(),
                icon: r.icon.to_string(),
                sort: r.sort,
                ext: r.ext.to_string(),
                own_paths: r.own_paths.to_string(),
                owner: r.owner.to_string(),
                scope_level: r.scope_level.clone(),
                pid: resp.iter().find(|i| i.sys_code == r.sys_code[..r.sys_code.len() - set_cate_sys_code_node_len]).map(|i| i.id.to_string()),
                rbum_set_items: rbum_set_items
                    .iter()
                    .filter(|i| i.rel_rbum_set_cate_id == r.id)
                    .map(|i| RbumSetItemInfoResp {
                        id: i.id.to_string(),
                        sort: i.sort,
                        rel_rbum_item_id: i.rel_rbum_item_id.to_string(),
                        rel_rbum_item_name: i.rel_rbum_item_name.to_string(),
                        own_paths: i.own_paths.to_string(),
                        owner: i.owner.to_string(),
                    })
                    .collect(),
            })
            .collect())
    }

    pub async fn get_tree_by_level(
        rbum_set_id: &str,
        rbum_parent_set_cate_id: Option<&str>,
        funs: &TardisFunsInst<'a>,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<RbumSetCateSummaryResp>> {
        let resp = Self::do_get_tree(rbum_set_id, rbum_parent_set_cate_id, false, funs, ctx).await?;
        Ok(resp
            .into_iter()
            .map(|r| RbumSetCateSummaryResp {
                id: r.id.to_string(),
                sys_code: r.sys_code.to_string(),
                bus_code: r.bus_code.to_string(),
                name: r.name.to_string(),
                icon: r.icon.to_string(),
                sort: r.sort,
                ext: r.ext.to_string(),
                rel_rbum_set_id: rbum_set_id.to_string(),
                own_paths: r.own_paths.to_string(),
                owner: r.owner.to_string(),
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
        ctx: &TardisContext,
    ) -> TardisResult<Vec<RbumSetCateWithLevelResp>> {
        Self::check_scope(rbum_set_id, RbumSetServ::get_table_name(), funs, ctx).await?;
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
                Self::check_scope(parent_set_cate_id, RbumSetCateServ::get_table_name(), funs, ctx).await?;
                let parent_sys_code = RbumSetCateServ::get_sys_code(parent_set_cate_id, funs, ctx).await?;
                query.and_where(Expr::col(rbum_set_cate::Column::SysCode).like(format!("{}%", parent_sys_code).as_str()));
                query.and_where(
                    Expr::expr(Func::char_length(Expr::col(rbum_set_cate::Column::SysCode))).eq((parent_sys_code.len() + funs.rbum_conf_set_cate_sys_code_node_len()) as i32),
                );
            } else {
                query.and_where(Expr::expr(Func::char_length(Expr::col(rbum_set_cate::Column::SysCode))).eq(funs.rbum_conf_set_cate_sys_code_node_len() as i32));
            }
        } else if let Some(parent_set_cate_id) = rbum_parent_set_cate_id {
            Self::check_scope(parent_set_cate_id, RbumSetCateServ::get_table_name(), funs, ctx).await?;
            let parent_sys_code = RbumSetCateServ::get_sys_code(parent_set_cate_id, funs, ctx).await?;
            query.and_where(Expr::col(rbum_set_cate::Column::SysCode).like(format!("{}%", parent_sys_code).as_str()));
        }
        query.order_by(rbum_set_cate::Column::Sort, Order::Asc);
        funs.db().find_dtos(&query).await
    }

    pub async fn get_rbum_set_id_by_code(code: &str, with_sub: bool, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<Option<String>> {
        let key = &format!("{}{}", funs.rbum_conf_cache_key_set_code_(), code);
        if let Some(cached_id) = funs.cache().get(key).await? {
            Ok(Some(cached_id))
        } else if let Some(rbum_set) = Self::find_one_rbum(
            &RbumSetFilterReq {
                basic: RbumBasicFilterReq {
                    code: Some(code.to_string()),
                    with_sub_own_paths: with_sub,
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
        {
            funs.cache().set_ex(key, &rbum_set.id, funs.rbum_conf_cache_key_set_code_expire_sec()).await?;
            Ok(Some(rbum_set.id))
        } else {
            Ok(None)
        }
    }
}

#[async_trait]
impl<'a> RbumCrudOperation<'a, rbum_set_cate::ActiveModel, RbumSetCateAddReq, RbumSetCateModifyReq, RbumSetCateSummaryResp, RbumSetCateDetailResp, RbumSetCateFilterReq>
    for RbumSetCateServ
{
    fn get_table_name() -> &'static str {
        rbum_set_cate::Entity.table_name()
    }

    async fn package_add(add_req: &RbumSetCateAddReq, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<rbum_set_cate::ActiveModel> {
        let sys_code = if let Some(rbum_parent_cate_id) = &add_req.rbum_parent_cate_id {
            Self::package_sys_code(&add_req.rel_rbum_set_id, Some(rbum_parent_cate_id), funs, ctx).await?
        } else {
            Self::package_sys_code(&add_req.rel_rbum_set_id, None, funs, ctx).await?
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
            scope_level: Set(add_req.scope_level.as_ref().unwrap_or(&RbumScopeLevelKind::Private).to_int()),
            ..Default::default()
        })
    }

    async fn before_add_rbum(add_req: &mut RbumSetCateAddReq, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<()> {
        Self::check_scope(&add_req.rel_rbum_set_id, RbumSetServ::get_table_name(), funs, ctx).await?;
        if let Some(rbum_parent_cate_id) = &add_req.rbum_parent_cate_id {
            Self::check_scope(rbum_parent_cate_id, RbumSetCateServ::get_table_name(), funs, ctx).await?;
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

    async fn before_delete_rbum(id: &str, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<Option<RbumSetCateDetailResp>> {
        Self::check_ownership(id, funs, ctx).await?;
        if funs
            .db()
            .count(
                Query::select()
                    .column((rbum_set_item::Entity, rbum_set_item::Column::Id))
                    .from(rbum_set_item::Entity)
                    .inner_join(
                        rbum_set_cate::Entity,
                        Condition::all()
                            .add(Expr::tbl(rbum_set_cate::Entity, rbum_set_cate::Column::SysCode).equals(rbum_set_item::Entity, rbum_set_item::Column::RelRbumSetCateCode))
                            .add(Expr::tbl(rbum_set_cate::Entity, rbum_set_cate::Column::RelRbumSetId).equals(rbum_set_item::Entity, rbum_set_item::Column::RelRbumSetId)),
                    )
                    .and_where(Expr::tbl(rbum_set_cate::Entity, rbum_set_cate::Column::Id).eq(id)),
            )
            .await?
            > 0
        {
            return Err(funs.err().conflict(
                &Self::get_obj_name(),
                "delete",
                &format!("can not delete {}.{} when there are associated by set_item", Self::get_obj_name(), id),
                "409-rbum-*-delete-conflict",
            ));
        }
        let set = Self::peek_rbum(
            id,
            &RbumSetCateFilterReq {
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
        if funs
            .db()
            .count(
                Query::select()
                    .column(rbum_set_cate::Column::Id)
                    .from(rbum_set_cate::Entity)
                    .and_where(Expr::col(rbum_set_cate::Column::Id).ne(id))
                    .and_where(Expr::col(rbum_set_cate::Column::RelRbumSetId).eq(set.rel_rbum_set_id.as_str()))
                    .and_where(Expr::col(rbum_set_cate::Column::SysCode).like(format!("{}%", set.sys_code.as_str()).as_str())),
            )
            .await?
            > 0
        {
            return Err(funs.err().conflict(
                &Self::get_obj_name(),
                "delete",
                &format!("can not delete {}.{} when there are associated by sub set_cate", Self::get_obj_name(), id),
                "409-rbum-*-delete-conflict",
            ));
        }
        Ok(None)
    }

    async fn package_query(is_detail: bool, filter: &RbumSetCateFilterReq, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<SelectStatement> {
        let mut query = Query::select();
        query
            .columns(vec![
                (rbum_set_cate::Entity, rbum_set_cate::Column::Id),
                (rbum_set_cate::Entity, rbum_set_cate::Column::SysCode),
                (rbum_set_cate::Entity, rbum_set_cate::Column::BusCode),
                (rbum_set_cate::Entity, rbum_set_cate::Column::Name),
                (rbum_set_cate::Entity, rbum_set_cate::Column::Icon),
                (rbum_set_cate::Entity, rbum_set_cate::Column::Sort),
                (rbum_set_cate::Entity, rbum_set_cate::Column::Ext),
                (rbum_set_cate::Entity, rbum_set_cate::Column::RelRbumSetId),
                (rbum_set_cate::Entity, rbum_set_cate::Column::OwnPaths),
                (rbum_set_cate::Entity, rbum_set_cate::Column::Owner),
                (rbum_set_cate::Entity, rbum_set_cate::Column::CreateTime),
                (rbum_set_cate::Entity, rbum_set_cate::Column::UpdateTime),
                (rbum_set_cate::Entity, rbum_set_cate::Column::ScopeLevel),
            ])
            .from(rbum_set_cate::Entity);
        if let Some(rel_rbum_set_id) = &filter.rel_rbum_set_id {
            query.and_where(Expr::tbl(rbum_set_cate::Entity, rbum_set_cate::Column::RelRbumSetId).eq(rel_rbum_set_id.to_string()));
        }
        if let Some(sys_code) = &filter.sys_code {
            if let Some(find_filter) = &filter.find_filter {
                match find_filter {
                    RbumSetCateLevelQueryKind::Sub => {
                        query.and_where(Expr::tbl(rbum_set_cate::Entity, rbum_set_cate::Column::SysCode).like(format!("{}%", sys_code).as_str()));
                    }
                    RbumSetCateLevelQueryKind::CurrentAndParent => {
                        let mut sys_codes = Self::get_parent_sys_codes(sys_code, funs)?;
                        sys_codes.insert(0, sys_code.to_string());
                        query.and_where(Expr::tbl(rbum_set_cate::Entity, rbum_set_cate::Column::SysCode).is_in(sys_codes));
                    }
                    RbumSetCateLevelQueryKind::Parent => {
                        let parent_sys_codes = Self::get_parent_sys_codes(sys_code, funs)?;
                        query.and_where(Expr::tbl(rbum_set_cate::Entity, rbum_set_cate::Column::SysCode).is_in(parent_sys_codes));
                    }
                }
            } else {
                query.and_where(Expr::tbl(rbum_set_cate::Entity, rbum_set_cate::Column::SysCode).eq(sys_code.as_str()));
            }
        }
        if let Some(rbum_item_rel_filter_req) = &filter.rel {
            if rbum_item_rel_filter_req.rel_by_from {
                query.inner_join(
                    rbum_rel::Entity,
                    Expr::tbl(rbum_rel::Entity, rbum_rel::Column::FromRbumId).equals(rbum_set_cate::Entity, rbum_set_cate::Column::Id),
                );
                if let Some(rel_item_id) = &rbum_item_rel_filter_req.rel_item_id {
                    query.and_where(Expr::tbl(rbum_rel::Entity, rbum_rel::Column::ToRbumItemId).eq(rel_item_id.to_string()));
                }
            } else {
                query.inner_join(
                    rbum_rel::Entity,
                    Expr::tbl(rbum_rel::Entity, rbum_rel::Column::ToRbumItemId).equals(rbum_set_cate::Entity, rbum_set_cate::Column::Id),
                );
                if let Some(rel_item_id) = &rbum_item_rel_filter_req.rel_item_id {
                    query.and_where(Expr::tbl(rbum_rel::Entity, rbum_rel::Column::FromRbumId).eq(rel_item_id.to_string()));
                }
            }
            if let Some(tag) = &rbum_item_rel_filter_req.tag {
                query.and_where(Expr::tbl(rbum_rel::Entity, rbum_rel::Column::Tag).eq(tag.to_string()));
            }
            if let Some(from_rbum_kind) = &rbum_item_rel_filter_req.from_rbum_kind {
                query.and_where(Expr::tbl(rbum_rel::Entity, rbum_rel::Column::FromRbumKind).eq(from_rbum_kind.to_int()));
            }
        }
        query.with_filter(Self::get_table_name(), &filter.basic, is_detail, true, ctx);
        Ok(query)
    }
}

impl<'a> RbumSetCateServ {
    fn get_parent_sys_codes(sys_code: &str, funs: &TardisFunsInst<'a>) -> TardisResult<Vec<String>> {
        let set_cate_sys_code_node_len = funs.rbum_conf_set_cate_sys_code_node_len();
        let mut level = sys_code.len() / set_cate_sys_code_node_len - 1;
        if level == 0 {
            return Ok(vec![]);
        }
        let mut sys_code_item = Vec::with_capacity(level);
        while level != 0 {
            sys_code_item.push(sys_code[..set_cate_sys_code_node_len * level].to_string());
            level -= 1;
        }
        Ok(sys_code_item)
    }

    async fn package_sys_code(rbum_set_id: &str, rbum_set_parent_cate_id: Option<&str>, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<String> {
        let lock_key = format!("rbum_set_cate_sys_code_{}", rbum_set_id);
        while !funs.cache().set_nx(&lock_key, "waiting").await? {
            sleep(Duration::from_millis(100)).await;
        }
        funs.cache().expire(&lock_key, 10).await?;
        let sys_code = if let Some(rbum_set_parent_cate_id) = rbum_set_parent_cate_id {
            let rel_parent_sys_code = Self::get_sys_code(rbum_set_parent_cate_id, funs, ctx).await?;
            Self::get_max_sys_code_by_level(rbum_set_id, Some(&rel_parent_sys_code), funs, ctx).await
        } else {
            Self::get_max_sys_code_by_level(rbum_set_id, None, funs, ctx).await
        };
        funs.cache().del(&lock_key).await?;
        sys_code
    }

    async fn get_max_sys_code_by_level(rbum_set_id: &str, parent_sys_code: Option<&str>, funs: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<String> {
        let set_cate_sys_code_node_len = funs.rbum_conf_set_cate_sys_code_node_len();
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
                // if level N (N!=1) not empty
                let curr_level_sys_code = max_sys_code[max_sys_code.len() - set_cate_sys_code_node_len..].to_string();
                let parent_sys_code = max_sys_code[..max_sys_code.len() - set_cate_sys_code_node_len].to_string();
                let curr_level_sys_code = TardisFuns::field.incr_by_base36(&curr_level_sys_code).ok_or_else(|| {
                    funs.err().bad_request(
                        &Self::get_obj_name(),
                        "get_sys_code",
                        "current number of nodes is saturated",
                        "400-rbum-set-sys-code-saturated",
                    )
                })?;
                Ok(format!("{}{}", parent_sys_code, curr_level_sys_code))
            } else {
                // if level 1 not empty
                Ok(TardisFuns::field.incr_by_base36(&max_sys_code).ok_or_else(|| {
                    funs.err().bad_request(
                        &Self::get_obj_name(),
                        "get_sys_code",
                        "current number of nodes is saturated",
                        "400-rbum-set-sys-code-saturated",
                    )
                })?)
            }
        } else if let Some(parent_sys_code) = parent_sys_code {
            // if level N (N!=1) is empty
            Ok(format!("{}{}", parent_sys_code, String::from_utf8(vec![b'0'; set_cate_sys_code_node_len])?))
        } else {
            // if level 1 is empty
            Ok(String::from_utf8(vec![b'0'; set_cate_sys_code_node_len])?)
        }
    }

    async fn get_sys_code(rbum_set_cate_id: &str, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<String> {
        Self::check_scope(rbum_set_cate_id, RbumSetCateServ::get_table_name(), funs, ctx).await?;
        let sys_code = funs
            .db()
            .get_dto::<SysCodeResp>(
                Query::select().column(rbum_set_cate::Column::SysCode).from(rbum_set_cate::Entity).and_where(Expr::col(rbum_set_cate::Column::Id).eq(rbum_set_cate_id)),
            )
            .await?
            .ok_or_else(|| {
                funs.err().not_found(
                    &Self::get_obj_name(),
                    "get_sys_code",
                    &format!("not found set cate {}", rbum_set_cate_id),
                    "404-rbum-set-cate-not-exist",
                )
            })?
            .sys_code;
        Ok(sys_code)
    }
}

#[async_trait]
impl<'a> RbumCrudOperation<'a, rbum_set_item::ActiveModel, RbumSetItemAddReq, RbumSetItemModifyReq, RbumSetItemSummaryResp, RbumSetItemDetailResp, RbumSetItemFilterReq>
    for RbumSetItemServ
{
    fn get_table_name() -> &'static str {
        rbum_set_item::Entity.table_name()
    }

    async fn package_add(add_req: &RbumSetItemAddReq, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<rbum_set_item::ActiveModel> {
        let rel_sys_code = RbumSetCateServ::get_sys_code(add_req.rel_rbum_set_cate_id.as_str(), funs, ctx).await?;
        Ok(rbum_set_item::ActiveModel {
            id: Set(TardisFuns::field.nanoid()),
            rel_rbum_set_id: Set(add_req.rel_rbum_set_id.to_string()),
            rel_rbum_set_cate_code: Set(rel_sys_code),
            rel_rbum_item_id: Set(add_req.rel_rbum_item_id.to_string()),
            sort: Set(add_req.sort),
            ..Default::default()
        })
    }

    async fn before_add_rbum(add_req: &mut RbumSetItemAddReq, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<()> {
        Self::check_scope(&add_req.rel_rbum_set_id, RbumSetServ::get_table_name(), funs, ctx).await?;
        Self::check_scope(&add_req.rel_rbum_item_id, RbumItemServ::get_table_name(), funs, ctx).await?;
        Self::check_scope(&add_req.rel_rbum_set_cate_id, RbumSetCateServ::get_table_name(), funs, ctx).await?;
        let rel_sys_code = RbumSetCateServ::get_sys_code(add_req.rel_rbum_set_cate_id.as_str(), funs, ctx).await?;
        if funs
            .db()
            .count(
                Query::select()
                    .column(rbum_set_item::Column::Id)
                    .from(rbum_set_item::Entity)
                    .and_where(Expr::col(rbum_set_item::Column::RelRbumSetId).eq(add_req.rel_rbum_set_id.as_str()))
                    .and_where(Expr::col(rbum_set_item::Column::RelRbumItemId).eq(add_req.rel_rbum_item_id.as_str()))
                    .and_where(Expr::col(rbum_set_item::Column::RelRbumSetCateCode).eq(rel_sys_code.as_str())),
            )
            .await?
            > 0
        {
            return Err(funs.err().conflict(&Self::get_obj_name(), "add", "item already exists", "409-rbum-set-item-exist"));
        }
        Ok(())
    }

    async fn package_modify(id: &str, modify_req: &RbumSetItemModifyReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<rbum_set_item::ActiveModel> {
        Ok(rbum_set_item::ActiveModel {
            id: Set(id.to_string()),
            sort: Set(modify_req.sort),
            ..Default::default()
        })
    }

    async fn package_query(is_detail: bool, filter: &RbumSetItemFilterReq, _: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<SelectStatement> {
        let rel_item_table = Alias::new("relItem");

        let mut query = Query::select();
        query
            .columns(vec![
                (rbum_set_item::Entity, rbum_set_item::Column::Id),
                (rbum_set_item::Entity, rbum_set_item::Column::Sort),
                (rbum_set_item::Entity, rbum_set_item::Column::RelRbumSetId),
                (rbum_set_item::Entity, rbum_set_item::Column::RelRbumItemId),
                (rbum_set_item::Entity, rbum_set_item::Column::OwnPaths),
                (rbum_set_item::Entity, rbum_set_item::Column::Owner),
                (rbum_set_item::Entity, rbum_set_item::Column::CreateTime),
                (rbum_set_item::Entity, rbum_set_item::Column::UpdateTime),
            ])
            .expr_as(Expr::tbl(rbum_set_cate::Entity, rbum_set_cate::Column::Id), Alias::new("rel_rbum_set_cate_id"))
            .expr_as(Expr::tbl(rbum_set_cate::Entity, rbum_set_cate::Column::SysCode), Alias::new("rel_rbum_set_cate_sys_code"))
            .expr_as(Expr::tbl(rbum_set_cate::Entity, rbum_set_cate::Column::Name), Alias::new("rel_rbum_set_cate_name"))
            .expr_as(Expr::tbl(rel_item_table.clone(), rbum_item::Column::Name), Alias::new("rel_rbum_item_name"))
            .from(rbum_set_item::Entity)
            .inner_join(
                rbum_set_cate::Entity,
                Cond::all()
                    .add(Expr::tbl(rbum_set_cate::Entity, rbum_set_cate::Column::SysCode).equals(rbum_set_item::Entity, rbum_set_item::Column::RelRbumSetCateCode))
                    .add(Expr::tbl(rbum_set_cate::Entity, rbum_set_cate::Column::RelRbumSetId).equals(rbum_set_item::Entity, rbum_set_item::Column::RelRbumSetId)),
            )
            .join_as(
                JoinType::InnerJoin,
                rbum_item::Entity,
                rel_item_table.clone(),
                Expr::tbl(rel_item_table, rbum_item::Column::Id).equals(rbum_set_item::Entity, rbum_set_item::Column::RelRbumItemId),
            );
        if let Some(rel_rbum_set_id) = &filter.rel_rbum_set_id {
            query.and_where(Expr::tbl(rbum_set_item::Entity, rbum_set_item::Column::RelRbumSetId).eq(rel_rbum_set_id.to_string()));
        }
        if let Some(rel_rbum_set_cate_id) = &filter.rel_rbum_set_cate_id {
            query.and_where(Expr::tbl(rbum_set_cate::Entity, rbum_set_cate::Column::Id).eq(rel_rbum_set_cate_id.to_string()));
        }
        if let Some(rel_rbum_item_id) = &filter.rel_rbum_item_id {
            query.and_where(Expr::tbl(rbum_set_item::Entity, rbum_set_item::Column::RelRbumItemId).eq(rel_rbum_item_id.to_string()));
        }
        query.with_filter(Self::get_table_name(), &filter.basic, is_detail, false, ctx);
        Ok(query)
    }
}

impl<'a> RbumSetItemServ {
    pub async fn find_set_paths(rbum_item_id: &str, rbum_set_id: &str, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<Vec<Vec<RbumSetPathResp>>> {
        let rbum_set_cate_sys_codes: Vec<String> = Self::find_rbums(
            &RbumSetItemFilterReq {
                rel_rbum_set_id: Some(rbum_set_id.to_string()),
                rel_rbum_item_id: Some(rbum_item_id.to_string()),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?
        .into_iter()
        .map(|item| item.rel_rbum_set_cate_sys_code)
        .collect();
        let mut result: Vec<Vec<RbumSetPathResp>> = Vec::with_capacity(rbum_set_cate_sys_codes.len());
        for rbum_set_cate_sys_code in rbum_set_cate_sys_codes {
            let rbum_set_paths = RbumSetCateServ::find_rbums(
                &RbumSetCateFilterReq {
                    rel_rbum_set_id: Some(rbum_set_id.to_string()),
                    sys_code: Some(rbum_set_cate_sys_code),
                    find_filter: Some(RbumSetCateLevelQueryKind::CurrentAndParent),
                    ..Default::default()
                },
                None,
                None,
                funs,
                ctx,
            )
            .await?
            .into_iter()
            .sorted_by(|a, b| Ord::cmp(&a.sys_code, &b.sys_code))
            .map(|item| RbumSetPathResp {
                id: item.id,
                name: item.name,
                own_paths: item.own_paths,
            })
            .collect();
            result.push(rbum_set_paths);
        }
        Ok(result)
    }
}

#[derive(Debug, sea_orm::FromQueryResult)]
struct SysCodeResp {
    pub sys_code: String,
}

#[derive(Debug, sea_orm::FromQueryResult)]
struct RbumSetCateWithLevelResp {
    pub id: String,
    pub sys_code: String,
    pub bus_code: String,
    pub name: String,
    pub icon: String,
    pub sort: u32,
    pub ext: String,

    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
}
