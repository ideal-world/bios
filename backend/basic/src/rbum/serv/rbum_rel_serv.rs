use std::collections::HashMap;

use async_trait::async_trait;
use itertools::Itertools;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::IdResp;
use tardis::db::sea_orm::sea_query::*;
use tardis::db::sea_orm::IdenStatic;
use tardis::db::sea_orm::*;
use tardis::web::web_resp::TardisPage;
use tardis::TardisFuns;
use tardis::TardisFunsInst;

use crate::rbum::domain::{rbum_item, rbum_kind_attr, rbum_rel, rbum_rel_attr, rbum_rel_env, rbum_set, rbum_set_cate};
use crate::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumRelExtFilterReq, RbumRelFilterReq, RbumSetCateFilterReq, RbumSetItemFilterReq};
use crate::rbum::dto::rbum_rel_agg_dto::{RbumRelAggAddReq, RbumRelAggResp};
use crate::rbum::dto::rbum_rel_attr_dto::{RbumRelAttrAddReq, RbumRelAttrDetailResp, RbumRelAttrModifyReq};
use crate::rbum::dto::rbum_rel_dto::RbumRelEnvCheckReq;
use crate::rbum::dto::rbum_rel_dto::{RbumRelAddReq, RbumRelBoneResp, RbumRelCheckReq, RbumRelDetailResp, RbumRelModifyReq, RbumRelSimpleFindReq};
use crate::rbum::dto::rbum_rel_env_dto::{RbumRelEnvAddReq, RbumRelEnvDetailResp, RbumRelEnvModifyReq};
use crate::rbum::rbum_enumeration::{RbumRelEnvKind, RbumRelFromKind, RbumSetCateLevelQueryKind};
use crate::rbum::serv::rbum_crud_serv::{NameResp, RbumCrudOperation, RbumCrudQueryPackage};
use crate::rbum::serv::rbum_item_serv::RbumItemServ;
use crate::rbum::serv::rbum_kind_serv::RbumKindAttrServ;
use crate::rbum::serv::rbum_set_serv::{RbumSetCateServ, RbumSetItemServ, RbumSetServ};

use super::rbum_cert_serv::RbumCertServ;

pub struct RbumRelServ;

pub struct RbumRelAttrServ;

pub struct RbumRelEnvServ;

#[async_trait]
impl RbumCrudOperation<rbum_rel::ActiveModel, RbumRelAddReq, RbumRelModifyReq, RbumRelDetailResp, RbumRelDetailResp, RbumRelFilterReq> for RbumRelServ {
    fn get_table_name() -> &'static str {
        rbum_rel::Entity.table_name()
    }

    async fn before_add_rbum(add_req: &mut RbumRelAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let rel_rbum_table_name = match add_req.from_rbum_kind {
            RbumRelFromKind::Item => RbumItemServ::get_table_name(),
            RbumRelFromKind::Set => RbumSetServ::get_table_name(),
            RbumRelFromKind::SetCate => RbumSetCateServ::get_table_name(),
            RbumRelFromKind::Cert => RbumCertServ::get_table_name(),
        };
        if RbumRelFromKind::Cert == add_req.from_rbum_kind {
            RbumCertServ::check_ownership(&add_req.from_rbum_id, funs, ctx).await?;
        } else {
            // The relationship check is changed from check_ownership to check_scope.
            // for example, the account corresponding to the tenant can be associated to the app,
            // where the account belongs to the tenant but scope=1, so it can be used by the application.
            //
            // 这里的关系检查从check_ownership改为check_scope。
            // 比如租户对应的账户可以关联到应用，账户属于租户但scope=1，所以可以被应用使用。
            Self::check_scope(&add_req.from_rbum_id, rel_rbum_table_name, funs, ctx).await?;
        }

        if add_req.to_rbum_item_id.trim().is_empty() {
            return Err(funs.err().bad_request(&Self::get_obj_name(), "add", "to_rbum_item_id can not be empty", "400-rbum-rel-not-empty-item"));
        }
        // It may not be possible to get the data of to_rbum_item_id when there are multiple database instances.
        //
        // 当存在多个数据库实例时，可能无法获取to_rbum_item_id的数据。
        if !add_req.to_is_outside {
            Self::check_scope(&add_req.to_rbum_item_id, RbumItemServ::get_table_name(), funs, ctx).await?;
        }
        Ok(())
    }

    async fn package_add(add_req: &RbumRelAddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<rbum_rel::ActiveModel> {
        Ok(rbum_rel::ActiveModel {
            id: Set(TardisFuns::field.nanoid()),
            tag: Set(add_req.tag.to_string()),
            note: Set(add_req.note.as_ref().unwrap_or(&"".to_string()).to_string()),
            from_rbum_kind: Set(add_req.from_rbum_kind.to_int()),
            from_rbum_id: Set(add_req.from_rbum_id.to_string()),
            to_rbum_item_id: Set(add_req.to_rbum_item_id.to_string()),
            to_own_paths: Set(add_req.to_own_paths.to_string()),
            ext: Set(add_req.ext.as_ref().unwrap_or(&"".to_string()).to_string()),
            ..Default::default()
        })
    }

    async fn package_modify(id: &str, modify_req: &RbumRelModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<rbum_rel::ActiveModel> {
        let mut rbum_rel = rbum_rel::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(tag) = &modify_req.tag {
            rbum_rel.tag = Set(tag.to_string());
        }
        if let Some(note) = &modify_req.note {
            rbum_rel.note = Set(note.to_string());
        }
        if let Some(ext) = &modify_req.ext {
            rbum_rel.ext = Set(ext.to_string());
        }
        Ok(rbum_rel)
    }

    async fn before_delete_rbum(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<RbumRelDetailResp>> {
        let mut query = Query::select();
        query.column(rbum_rel::Column::Id).from(rbum_rel::Entity).and_where(Expr::col(rbum_rel::Column::Id).eq(id)).cond_where(all![any![
            Expr::col(rbum_rel::Column::OwnPaths).like(format!("{}%", ctx.own_paths).as_str()),
            Expr::col(rbum_rel::Column::ToOwnPaths).like(format!("{}%", ctx.own_paths).as_str())
        ]]);
        if funs.db().count(&query).await? == 0 {
            return Err(funs.err().not_found(
                &Self::get_obj_name(),
                "delete",
                &format!("ownership {}.{} is illegal by {}", Self::get_obj_name(), id, ctx.owner),
                "404-rbum-*-ownership-illegal",
            ));
        }
        Self::check_exist_before_delete(id, RbumRelAttrServ::get_table_name(), rbum_rel_attr::Column::RelRbumRelId.as_str(), funs).await?;
        Self::check_exist_before_delete(id, RbumRelEnvServ::get_table_name(), rbum_rel_env::Column::RelRbumRelId.as_str(), funs).await?;
        Ok(None)
    }

    async fn package_query(_: bool, filter: &RbumRelFilterReq, _: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<SelectStatement> {
        let from_rbum_item_table = Alias::new("fromRbumItem");
        let to_rbum_item_table = Alias::new("toRbumItem");
        let mut query = Query::select();
        query
            .columns(vec![
                (rbum_rel::Entity, rbum_rel::Column::Id),
                (rbum_rel::Entity, rbum_rel::Column::Tag),
                (rbum_rel::Entity, rbum_rel::Column::Note),
                (rbum_rel::Entity, rbum_rel::Column::FromRbumKind),
                (rbum_rel::Entity, rbum_rel::Column::FromRbumId),
                (rbum_rel::Entity, rbum_rel::Column::ToRbumItemId),
                (rbum_rel::Entity, rbum_rel::Column::ToOwnPaths),
                (rbum_rel::Entity, rbum_rel::Column::Ext),
                (rbum_rel::Entity, rbum_rel::Column::OwnPaths),
                (rbum_rel::Entity, rbum_rel::Column::Owner),
                (rbum_rel::Entity, rbum_rel::Column::CreateTime),
                (rbum_rel::Entity, rbum_rel::Column::UpdateTime),
            ])
            .expr_as(
                Expr::col((from_rbum_item_table.clone(), rbum_item::Column::Name)).if_null(""),
                Alias::new("from_rbum_item_name"),
            )
            .expr_as(Expr::col((rbum_set::Entity, rbum_set::Column::Name)).if_null(""), Alias::new("from_rbum_set_name"))
            .expr_as(
                Expr::col((rbum_set_cate::Entity, rbum_set_cate::Column::Name)).if_null(""),
                Alias::new("from_rbum_set_cate_name"),
            )
            .expr_as(
                Expr::col((to_rbum_item_table.clone(), rbum_item::Column::Name)).if_null(""),
                Alias::new("to_rbum_item_name"),
            )
            .from(rbum_rel::Entity)
            .join_as(
                JoinType::LeftJoin,
                rbum_item::Entity,
                from_rbum_item_table.clone(),
                all![
                    Expr::col((from_rbum_item_table.clone(), rbum_item::Column::Id)).equals((rbum_rel::Entity, rbum_rel::Column::FromRbumId)),
                    Expr::col((rbum_rel::Entity, rbum_rel::Column::FromRbumKind)).eq(RbumRelFromKind::Item.to_int())
                ],
            )
            .left_join(
                rbum_set::Entity,
                all![
                    Expr::col((rbum_set::Entity, rbum_set::Column::Id)).equals((rbum_rel::Entity, rbum_rel::Column::FromRbumId)),
                    Expr::col((rbum_rel::Entity, rbum_rel::Column::FromRbumKind)).eq(RbumRelFromKind::Set.to_int())
                ],
            )
            .left_join(
                rbum_set_cate::Entity,
                all![
                    Expr::col((rbum_set_cate::Entity, rbum_set_cate::Column::Id)).equals((rbum_rel::Entity, rbum_rel::Column::FromRbumId)),
                    Expr::col((rbum_rel::Entity, rbum_rel::Column::FromRbumKind)).eq(RbumRelFromKind::SetCate.to_int())
                ],
            )
            .join_as(
                JoinType::LeftJoin,
                rbum_item::Entity,
                to_rbum_item_table.clone(),
                Expr::col((to_rbum_item_table.clone(), rbum_item::Column::Id)).equals((rbum_rel::Entity, rbum_rel::Column::ToRbumItemId)),
            );

        if let Some(tag) = &filter.tag {
            query.and_where(Expr::col((rbum_rel::Entity, rbum_rel::Column::Tag)).eq(tag.to_string()));
        }
        if let Some(from_rbum_kind) = &filter.from_rbum_kind {
            query.and_where(Expr::col((rbum_rel::Entity, rbum_rel::Column::FromRbumKind)).eq(from_rbum_kind.to_int()));
        }
        if let Some(from_rbum_id) = &filter.from_rbum_id {
            query.and_where(Expr::col((rbum_rel::Entity, rbum_rel::Column::FromRbumId)).eq(from_rbum_id.to_string()));
        }
        if let Some(from_rbum_ids) = &filter.from_rbum_ids {
            query.and_where(Expr::col((rbum_rel::Entity, rbum_rel::Column::FromRbumId)).is_in(from_rbum_ids));
        }
        if let Some(to_rbum_item_id) = &filter.to_rbum_item_id {
            query.and_where(Expr::col((rbum_rel::Entity, rbum_rel::Column::ToRbumItemId)).eq(to_rbum_item_id.to_string()));
        }
        if let Some(to_rbum_item_ids) = &filter.to_rbum_item_ids {
            query.and_where(Expr::col((rbum_rel::Entity, rbum_rel::Column::ToRbumItemId)).is_in(to_rbum_item_ids));
        }
        if let Some(from_rbum_scope_levels) = &filter.from_rbum_scope_levels {
            query.and_where(Expr::col((from_rbum_item_table, rbum_item::Column::ScopeLevel)).is_in(from_rbum_scope_levels.clone()));
        }
        if let Some(to_rbum_item_scope_levels) = &filter.to_rbum_item_scope_levels {
            query.and_where(Expr::col((to_rbum_item_table, rbum_item::Column::ScopeLevel)).is_in(to_rbum_item_scope_levels.clone()));
        }
        if let Some(to_own_paths) = &filter.to_own_paths {
            query.and_where(Expr::col((rbum_rel::Entity, rbum_rel::Column::ToOwnPaths)).eq(to_own_paths.to_string()));
        }
        if let Some(ext_eq) = &filter.ext_eq {
            query.and_where(Expr::col((rbum_rel::Entity, rbum_rel::Column::Ext)).eq(ext_eq.to_string()));
        }
        if let Some(ext_like) = &filter.ext_like {
            query.and_where(Expr::col((rbum_rel::Entity, rbum_rel::Column::Ext)).like(format!("%{ext_like}%").as_str()));
        }
        query.with_filter(Self::get_table_name(), &filter.basic, true, false, ctx);
        Ok(query)
    }
}

impl RbumRelServ {
    /// Add a simple relationship
    ///
    /// 添加简单的关联关系
    ///
    /// The relationship source is the ``resource item``.
    ///
    /// 关联的来源方为``资源项``。
    pub async fn add_simple_rel(tag: &str, from_rbum_id: &str, to_rbum_item_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        RbumRelServ::add_rbum(
            &mut RbumRelAddReq {
                tag: tag.to_string(),
                note: None,
                from_rbum_kind: RbumRelFromKind::Item,
                from_rbum_id: from_rbum_id.to_string(),
                to_rbum_item_id: to_rbum_item_id.to_string(),
                to_own_paths: ctx.own_paths.to_string(),
                to_is_outside: false,
                ext: None,
            },
            funs,
            ctx,
        )
        .await?;
        Ok(())
    }

    /// Add a relationship
    ///
    /// 添加关联关系
    pub async fn add_rel(add_req: &mut RbumRelAggAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let rbum_rel_id = Self::add_rbum(&mut add_req.rel, funs, ctx).await?;
        for attr in &add_req.attrs {
            RbumRelAttrServ::add_rbum(
                &mut RbumRelAttrAddReq {
                    is_from: attr.is_from,
                    value: attr.value.to_string(),
                    name: attr.name.clone(),
                    rel_rbum_rel_id: rbum_rel_id.to_string(),
                    rel_rbum_kind_attr_id: attr.rel_rbum_kind_attr_id.clone(),
                    record_only: attr.record_only,
                },
                funs,
                ctx,
            )
            .await?;
        }
        for env in &add_req.envs {
            RbumRelEnvServ::add_rbum(
                &mut RbumRelEnvAddReq {
                    kind: env.kind.clone(),
                    value1: env.value1.to_string(),
                    value2: env.value2.clone(),
                    rel_rbum_rel_id: rbum_rel_id.to_string(),
                },
                funs,
                ctx,
            )
            .await?;
        }
        Ok(rbum_rel_id)
    }

    /// Find the relationship target ids of the specified condition
    ///
    /// 查找指定条件的关联目标id集合
    pub async fn find_from_id_rels(
        tag: &str,
        from_rbum_kind: &RbumRelFromKind,
        with_sub: bool,
        from_rbum_id: &str,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<String>> {
        Self::find_from_simple_rels(tag, from_rbum_kind, with_sub, from_rbum_id, desc_sort_by_create, desc_sort_by_update, funs, ctx)
            .await
            .map(|r| r.into_iter().map(|item| item.rel_id).collect())
    }

    /// Find the relationship target summary information set of the specified condition
    ///
    /// 查找指定条件的关联目标概要信息集合
    pub async fn find_from_simple_rels(
        tag: &str,
        from_rbum_kind: &RbumRelFromKind,
        with_sub: bool,
        from_rbum_id: &str,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<RbumRelBoneResp>> {
        Self::find_rbums(
            &RbumRelFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: with_sub,
                    ..Default::default()
                },
                tag: Some(tag.to_string()),
                from_rbum_kind: Some(from_rbum_kind.clone()),
                from_rbum_id: Some(from_rbum_id.to_string()),
                ..Default::default()
            },
            desc_sort_by_create,
            desc_sort_by_update,
            funs,
            ctx,
        )
        .await
        .map(|r| r.into_iter().map(|item| RbumRelBoneResp::new(item, true)).collect())
    }

    /// Find the relationship aggregation detail information set of the specified condition
    ///
    /// 查找指定条件的关联聚合信息集合
    pub async fn find_from_rels(
        tag: &str,
        from_rbum_kind: &RbumRelFromKind,
        with_sub: bool,
        from_rbum_id: &str,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<RbumRelAggResp>> {
        Self::find_rels(
            &RbumRelFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: with_sub,
                    ..Default::default()
                },
                tag: Some(tag.to_string()),
                from_rbum_kind: Some(from_rbum_kind.clone()),
                from_rbum_id: Some(from_rbum_id.to_string()),
                ..Default::default()
            },
            desc_sort_by_create,
            desc_sort_by_update,
            funs,
            ctx,
        )
        .await
    }

    /// Page to get the relationship target ids of the specified condition
    ///
    /// 分页查找指定条件的关联目标id集合
    pub async fn paginate_from_id_rels(
        tag: &str,
        from_rbum_kind: &RbumRelFromKind,
        with_sub: bool,
        from_rbum_id: &str,
        page_number: u32,
        page_size: u32,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<String>> {
        let result = Self::paginate_from_simple_rels(
            tag,
            from_rbum_kind,
            with_sub,
            from_rbum_id,
            page_number,
            page_size,
            desc_sort_by_create,
            desc_sort_by_update,
            funs,
            ctx,
        )
        .await?;
        Ok(TardisPage {
            page_size: result.page_size,
            page_number: result.page_number,
            total_size: result.total_size,
            records: result.records.into_iter().map(|item| item.rel_id).collect(),
        })
    }

    /// Page to get the relationship target summary information set of the specified condition
    ///
    /// 分页查找指定条件的关联目标概要信息集合
    pub async fn paginate_from_simple_rels(
        tag: &str,
        from_rbum_kind: &RbumRelFromKind,
        with_sub: bool,
        from_rbum_id: &str,
        page_number: u32,
        page_size: u32,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<RbumRelBoneResp>> {
        let result = Self::paginate_rbums(
            &RbumRelFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: with_sub,
                    ..Default::default()
                },
                tag: Some(tag.to_string()),
                from_rbum_kind: Some(from_rbum_kind.clone()),
                from_rbum_id: Some(from_rbum_id.to_string()),
                ..Default::default()
            },
            page_number,
            page_size,
            desc_sort_by_create,
            desc_sort_by_update,
            funs,
            ctx,
        )
        .await?;
        Ok(TardisPage {
            page_size: result.page_size,
            page_number: result.page_number,
            total_size: result.total_size,
            records: result.records.into_iter().map(|item| RbumRelBoneResp::new(item, true)).collect(),
        })
    }

    /// Page to get the relationship aggregation detail information set of the specified condition
    ///
    /// 分页查找指定条件的关联聚合信息集合
    pub async fn paginate_from_rels(
        tag: &str,
        from_rbum_kind: &RbumRelFromKind,
        with_sub: bool,
        from_rbum_id: &str,
        page_number: u32,
        page_size: u32,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<RbumRelAggResp>> {
        Self::paginate_rels(
            &RbumRelFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: with_sub,
                    ..Default::default()
                },
                tag: Some(tag.to_string()),
                from_rbum_kind: Some(from_rbum_kind.clone()),
                from_rbum_id: Some(from_rbum_id.to_string()),
                ..Default::default()
            },
            page_number,
            page_size,
            desc_sort_by_create,
            desc_sort_by_update,
            funs,
            ctx,
        )
        .await
    }

    /// Statistics the number of the specified condition
    ///
    /// 统计指定条件的关联记录数
    pub async fn count_from_rels(tag: &str, from_rbum_kind: &RbumRelFromKind, with_sub: bool, from_rbum_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<u64> {
        Self::count_rels(
            &RbumRelFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: with_sub,
                    ..Default::default()
                },
                tag: Some(tag.to_string()),
                from_rbum_kind: Some(from_rbum_kind.clone()),
                from_rbum_id: Some(from_rbum_id.to_string()),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await
    }

    /// Find the relationship source ids of the specified condition
    ///
    /// 查找指定条件的关联来源id集合
    pub async fn find_to_id_rels(
        tag: &str,
        to_rbum_item_id: &str,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<String>> {
        Self::find_to_simple_rels(tag, to_rbum_item_id, desc_sort_by_create, desc_sort_by_update, funs, ctx).await.map(|r| r.into_iter().map(|item| item.rel_id).collect())
    }

    /// Find the relationship source summary information set of the specified condition
    ///
    /// 查找指定条件的关联来源概要信息集合
    pub async fn find_to_simple_rels(
        tag: &str,
        to_rbum_item_id: &str,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<RbumRelBoneResp>> {
        Self::find_rbums(
            &RbumRelFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some(ctx.own_paths.to_string()),
                    with_sub_own_paths: true,
                    ignore_scope: true,
                    ..Default::default()
                },
                tag: Some(tag.to_string()),
                to_rbum_item_id: Some(to_rbum_item_id.to_string()),
                ..Default::default()
            },
            desc_sort_by_create,
            desc_sort_by_update,
            funs,
            ctx,
        )
        .await
        .map(|r| r.into_iter().map(|item| RbumRelBoneResp::new(item, false)).collect())
    }

    /// Find the relationship aggregation detail information set of the specified condition
    ///
    /// 查找指定条件的关联聚合信息集合
    pub async fn find_to_rels(
        tag: &str,
        to_rbum_item_id: &str,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<RbumRelAggResp>> {
        Self::find_rels(
            &RbumRelFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some(ctx.own_paths.to_string()),
                    with_sub_own_paths: true,
                    ignore_scope: true,
                    ..Default::default()
                },
                tag: Some(tag.to_string()),
                to_rbum_item_id: Some(to_rbum_item_id.to_string()),
                ..Default::default()
            },
            desc_sort_by_create,
            desc_sort_by_update,
            funs,
            ctx,
        )
        .await
    }

    /// Page to get the relationship source ids of the specified condition
    ///
    /// 分页查找指定条件的关联来源id集合
    pub async fn paginate_to_id_rels(
        tag: &str,
        to_rbum_item_id: &str,
        page_number: u32,
        page_size: u32,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<String>> {
        let result = Self::paginate_to_simple_rels(tag, to_rbum_item_id, page_number, page_size, desc_sort_by_create, desc_sort_by_update, funs, ctx).await?;
        Ok(TardisPage {
            page_size: result.page_size,
            page_number: result.page_number,
            total_size: result.total_size,
            records: result.records.into_iter().map(|item| item.rel_id).collect(),
        })
    }

    /// Page to get the relationship source summary information set of the specified condition
    ///
    /// 分页查找指定条件的关联来源概要信息集合
    pub async fn paginate_to_simple_rels(
        tag: &str,
        to_rbum_item_id: &str,
        page_number: u32,
        page_size: u32,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<RbumRelBoneResp>> {
        let result = Self::paginate_rbums(
            &RbumRelFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some(ctx.own_paths.to_string()),
                    with_sub_own_paths: true,
                    ignore_scope: true,
                    ..Default::default()
                },
                tag: Some(tag.to_string()),
                to_rbum_item_id: Some(to_rbum_item_id.to_string()),
                ..Default::default()
            },
            page_number,
            page_size,
            desc_sort_by_create,
            desc_sort_by_update,
            funs,
            ctx,
        )
        .await?;
        Ok(TardisPage {
            page_size: result.page_size,
            page_number: result.page_number,
            total_size: result.total_size,
            records: result.records.into_iter().map(|item| RbumRelBoneResp::new(item, false)).collect(),
        })
    }

    /// Page to get the relationship aggregation detail information set of the specified condition
    ///
    /// 分页查找指定条件的关联聚合信息集合
    pub async fn paginate_to_rels(
        tag: &str,
        to_rbum_item_id: &str,
        page_number: u32,
        page_size: u32,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<RbumRelAggResp>> {
        Self::paginate_rels(
            &RbumRelFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some(ctx.own_paths.to_string()),
                    with_sub_own_paths: true,
                    ignore_scope: true,
                    ..Default::default()
                },
                tag: Some(tag.to_string()),
                to_rbum_item_id: Some(to_rbum_item_id.to_string()),
                ..Default::default()
            },
            page_number,
            page_size,
            desc_sort_by_create,
            desc_sort_by_update,
            funs,
            ctx,
        )
        .await
    }

    /// Statistics the number of the specified condition
    ///
    /// 统计指定条件的关联记录数
    pub async fn count_to_rels(tag: &str, to_rbum_item_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<u64> {
        Self::count_rels(
            &RbumRelFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some(ctx.own_paths.to_string()),
                    with_sub_own_paths: true,
                    ignore_scope: true,
                    ..Default::default()
                },
                tag: Some(tag.to_string()),
                to_rbum_item_id: Some(to_rbum_item_id.to_string()),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await
    }

    /// Statistics the number of the specified condition
    ///
    /// 统计指定条件的关联记录数
    async fn count_rels(filter: &RbumRelFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<u64> {
        RbumRelServ::count_rbums(filter, funs, ctx).await
    }

    /// Find the relationship summary information set of the specified condition
    ///
    /// 查找指定条件的关联概要信息集合
    ///
    /// If ``package_to_info = true``, return the relationship target information, otherwise, return the relationship source information.
    ///
    /// 当 ``package_to_info = true`` 时返回关联目标信息，反之，返回关联来源信息。
    pub async fn find_simple_rels(
        filter: &RbumRelFilterReq,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        package_to_info: bool,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<RbumRelBoneResp>> {
        RbumRelServ::find_rbums(filter, desc_sort_by_create, desc_sort_by_update, funs, ctx)
            .await
            .map(|r| r.into_iter().map(|item| RbumRelBoneResp::new(item, package_to_info)).collect())
    }

    /// Find the relationship aggregation detail information set of the specified condition
    ///
    /// 查找指定条件的关联聚合信息集合
    pub async fn find_rels(
        filter: &RbumRelFilterReq,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<RbumRelAggResp>> {
        let rbum_rels = RbumRelServ::find_rbums(filter, desc_sort_by_create, desc_sort_by_update, funs, ctx).await?;
        Self::package_agg_rels(rbum_rels, filter, funs, ctx).await
    }

    /// Page to get the relationship summary information set of the specified condition
    ///
    /// 分页查找指定条件的关联概要信息集合
    ///
    /// If ``package_to_info = true``, return the relationship target information, otherwise, return the relationship source information.
    ///
    /// 当 ``package_to_info = true`` 时返回关联目标信息，反之，返回关联来源信息。
    pub async fn paginate_simple_rels(
        filter: &RbumRelFilterReq,
        page_number: u32,
        page_size: u32,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        package_to_info: bool,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<RbumRelBoneResp>> {
        let result = RbumRelServ::paginate_rbums(filter, page_number, page_size, desc_sort_by_create, desc_sort_by_update, funs, ctx).await?;
        Ok(TardisPage {
            page_size: result.page_size,
            page_number: result.page_number,
            total_size: result.total_size,
            records: result.records.into_iter().map(|item| RbumRelBoneResp::new(item, package_to_info)).collect(),
        })
    }

    /// Page to get the relationship aggregation detail information set of the specified condition
    ///
    /// 分页查找指定条件的关联聚合信息集合
    pub async fn paginate_rels(
        filter: &RbumRelFilterReq,
        page_number: u32,
        page_size: u32,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<RbumRelAggResp>> {
        let rbum_rels = RbumRelServ::paginate_rbums(filter, page_number, page_size, desc_sort_by_create, desc_sort_by_update, funs, ctx).await?;
        let result = Self::package_agg_rels(rbum_rels.records, filter, funs, ctx).await?;
        Ok(TardisPage {
            page_size: page_size as u64,
            page_number: page_number as u64,
            total_size: rbum_rels.total_size,
            records: result,
        })
    }

    /// Package relationship aggregation information
    ///
    /// 组装关联聚合信息
    async fn package_agg_rels(rels: Vec<RbumRelDetailResp>, filter: &RbumRelFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<RbumRelAggResp>> {
        let mut result = Vec::with_capacity(rels.len());
        for rel in rels {
            let rbum_rel_id = rel.id.to_string();
            let resp = RbumRelAggResp {
                rel,
                attrs: RbumRelAttrServ::find_rbums(
                    &RbumRelExtFilterReq {
                        basic: RbumBasicFilterReq {
                            own_paths: filter.basic.own_paths.clone(),
                            with_sub_own_paths: filter.basic.with_sub_own_paths.clone(),
                            ..Default::default()
                        },
                        rel_rbum_rel_id: Some(rbum_rel_id.clone()),
                    },
                    None,
                    None,
                    funs,
                    ctx,
                )
                .await?,
                envs: RbumRelEnvServ::find_rbums(
                    &RbumRelExtFilterReq {
                        basic: RbumBasicFilterReq {
                            own_paths: filter.basic.own_paths.clone(),
                            with_sub_own_paths: filter.basic.with_sub_own_paths.clone(),
                            ..Default::default()
                        },
                        rel_rbum_rel_id: Some(rbum_rel_id.clone()),
                    },
                    None,
                    None,
                    funs,
                    ctx,
                )
                .await?,
            };
            result.push(resp);
        }
        Ok(result)
    }

    /// Find the relationship id set of the specified condition
    ///
    /// 查找指定条件的关联id集合
    pub async fn find_rel_ids(find_req: &RbumRelSimpleFindReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<String>> {
        let ids = funs.db().find_dtos::<IdResp>(&Self::package_simple_rel_query(find_req, ctx)).await?.iter().map(|i| i.id.to_string()).collect::<Vec<String>>();
        Ok(ids)
    }

    /// Check whether the relationship of the specified simple condition exists
    ///
    /// 检查指定的简单条件的关联是否存在
    pub async fn check_simple_rel(find_req: &RbumRelSimpleFindReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
        funs.db().count(&Self::package_simple_rel_query(find_req, ctx)).await.map(|i| i > 0)
    }

    fn package_simple_rel_query(find_req: &RbumRelSimpleFindReq, ctx: &TardisContext) -> SelectStatement {
        let mut query = Query::select();
        query.column(rbum_rel::Column::Id).from(rbum_rel::Entity);
        if let Some(tag) = &find_req.tag {
            query.and_where(Expr::col(rbum_rel::Column::Tag).eq(tag.to_string()));
        }
        if let Some(from_rbum_kind) = &find_req.from_rbum_kind {
            query.and_where(Expr::col(rbum_rel::Column::FromRbumKind).eq(from_rbum_kind.to_int()));
        }
        if let Some(from_rbum_id) = &find_req.from_rbum_id {
            query.and_where(Expr::col(rbum_rel::Column::FromRbumId).eq(from_rbum_id.to_string()));
        }
        if let Some(to_rbum_item_id) = &find_req.to_rbum_item_id {
            query.and_where(Expr::col(rbum_rel::Column::ToRbumItemId).eq(to_rbum_item_id.to_string()));
        }
        if let Some(from_own_paths) = &find_req.from_own_paths {
            query.and_where(Expr::col(rbum_rel::Column::OwnPaths).eq(from_own_paths.to_string()));
        }
        if let Some(to_rbum_own_paths) = &find_req.to_rbum_own_paths {
            query.and_where(Expr::col(rbum_rel::Column::ToOwnPaths).eq(to_rbum_own_paths.to_string()));
        }
        query.cond_where(all![any![
            Expr::col(rbum_rel::Column::OwnPaths).like(format!("{}%", ctx.own_paths).as_str()),
            Expr::col(rbum_rel::Column::ToOwnPaths).like(format!("{}%", ctx.own_paths).as_str())
        ]]);
        query
    }

    /// Check whether the relationship of the specified condition exists
    ///
    /// 检查指定的条件的关联是否存在
    pub async fn check_rel(check_req: &RbumRelCheckReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
        // 1. Check whether the direct association exists
        //
        // 1. 检查直接关联是否存在
        if Self::do_check_rel(
            &check_req.tag,
            Some(vec![check_req.from_rbum_kind.clone()]),
            Some(vec![check_req.from_rbum_id.clone()]),
            Some(check_req.to_rbum_item_id.clone()),
            &check_req.from_attrs,
            &check_req.to_attrs,
            &check_req.envs,
            funs,
            ctx,
        )
        .await?
        {
            return Ok(true);
        }
        // 2. Get the resource set categories(nodes)
        //
        // 2. 获取对应的资源集分类（节点）集合
        //
        // rel_rbum_set_cates : HashMap<set id, Vec(category id, system code)>
        let rel_rbum_set_cates = if check_req.from_rbum_kind == RbumRelFromKind::Item {
            RbumSetItemServ::find_rbums(
                &RbumSetItemFilterReq {
                    basic: Default::default(),
                    rel_rbum_item_ids: Some(vec![check_req.from_rbum_id.clone()]),
                    ..Default::default()
                },
                None,
                None,
                funs,
                ctx,
            )
            .await?
            .into_iter()
            .into_group_map_by(|i| i.rel_rbum_set_id.clone())
            .into_iter()
            .map(|(set_id, cates)| {
                (
                    set_id,
                    cates.into_iter().map(|cate| (cate.rel_rbum_set_cate_id.unwrap_or_default(), cate.rel_rbum_set_cate_sys_code.unwrap_or_default())).collect(),
                )
            })
            .collect::<HashMap<String, Vec<(String, String)>>>()
        } else if check_req.from_rbum_kind == RbumRelFromKind::SetCate {
            let set_cate = RbumSetCateServ::peek_rbum(&check_req.from_rbum_id, &RbumSetCateFilterReq::default(), funs, ctx).await?;
            HashMap::from([(set_cate.rel_rbum_set_id.clone(), vec![(set_cate.id, set_cate.sys_code)])])
        } else {
            return Ok(false);
        };

        for (set_id, cates) in rel_rbum_set_cates {
            // 3. Get the parent id set of the associated resource set category
            //
            // 3. 获取关联的资源集分类的父级id集合
            let mut set_cate_parent_ids = RbumSetCateServ::find_id_rbums(
                &RbumSetCateFilterReq {
                    basic: Default::default(),
                    rel_rbum_set_id: Some(set_id.clone()),
                    sys_codes: Some(cates.iter().map(|i| i.1.clone()).collect()),
                    sys_code_query_kind: Some(RbumSetCateLevelQueryKind::Parent),
                    ..Default::default()
                },
                None,
                None,
                funs,
                ctx,
            )
            .await?;
            if check_req.from_rbum_kind == RbumRelFromKind::Item {
                // If the source type of the request is ``item``, the source id needs to be added to the parent id set.
                // Because the source type has been switched, the source id needs to be rejudged.
                //
                // 如果请求的来源类型是``item``, 则需要把来源id加入到父级id集合中。因为后续切换了来源类型，故这个来源id需要重新判断。
                set_cate_parent_ids.insert(0, check_req.from_rbum_id.clone());
            }
            // Add the id of the resource set to the parent id set.
            //
            // 把资源集的id也添加进来。
            set_cate_parent_ids.push(set_id);

            // 4. Check whether the association on the resource set/resource set category(node) exists
            //
            // 4. 检查资源集/资源集分类（节点）上的关联是否存在
            if Self::do_check_rel(
                &check_req.tag,
                // Two source types are used here, and the ids of these two types are nanoid, so the conflict probability is very low.
                //
                // 这里使用了两个来源类型，这两个类型的id都是nanoid，冲突的概率很低。
                Some(vec![RbumRelFromKind::SetCate, RbumRelFromKind::Set]),
                Some(set_cate_parent_ids),
                Some(check_req.to_rbum_item_id.clone()),
                &check_req.from_attrs,
                &check_req.to_attrs,
                &check_req.envs,
                funs,
                ctx,
            )
            .await?
            {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Execute the relationship check
    ///
    ///
    /// 执行关联关系检查
    ///
    /// This function's core SQL is similar to the following:
    ///
    /// ```sql
    /// select
    ///  rel.id,
    ///  attr_cond.attr_count,
    ///  attr_all.attr_count
    ///from
    ///  rbum_rel rel
    /// -- Fetch the number of related attribute records with matching conditions
    ///  left join (
    ///    select
    ///      rel_rbum_rel_id,
    ///      count(1) attr_count
    ///    from
    ///      rbum_rel_attr attr
    ///    where
    ///      attr.rel_rbum_kind_attr_id = 'a001'
    ///      and attr.value = 'jzy'
    ///      or attr.rel_rbum_kind_attr_id = 'a002'
    ///      and attr.value = '30'
    ///    GROUP by
    ///      rel_rbum_rel_id
    ///  ) attr_cond on attr_cond.rel_rbum_rel_id = rel.id
    /// -- Fetch the number of records of all related attributes
    ///  left join (
    ///    select
    ///      rel_rbum_rel_id,
    ///      count(1) attr_count
    ///    from
    ///      rbum_rel_attr attr
    ///    GROUP by
    ///      rel_rbum_rel_id
    ///  ) attr_all on attr_all.rel_rbum_rel_id = rel.id
    ///where
    ///  rel.from_rbum_id = 'f01'
    ///  and (
    ///  -- The number of related attribute records with matching conditions is equal to the number of all related attribute records
    ///    attr_cond is null and attr_all is null
    ///    or attr_cond.attr_count = attr_all.attr_count
    ///  )
    ///
    /// ```
    async fn do_check_rel(
        tag: &str,
        from_rbum_kinds: Option<Vec<RbumRelFromKind>>,
        from_rbum_ids: Option<Vec<String>>,
        to_rbum_item_id: Option<String>,
        from_attrs: &HashMap<String, String>,
        to_attrs: &HashMap<String, String>,
        envs: &Vec<RbumRelEnvCheckReq>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<bool> {
        let mut query = Query::select();
        query.column((rbum_rel::Entity, rbum_rel::Column::Id)).from(rbum_rel::Entity);
        query.and_where(Expr::col((rbum_rel::Entity, rbum_rel::Column::Tag)).eq(tag));
        if let Some(from_rbum_kinds) = from_rbum_kinds {
            query.and_where(Expr::col((rbum_rel::Entity, rbum_rel::Column::FromRbumKind)).is_in(from_rbum_kinds.into_iter().map(|i| i.to_int()).collect::<Vec<_>>()));
        }
        if let Some(from_rbum_ids) = from_rbum_ids {
            query.and_where(Expr::col((rbum_rel::Entity, rbum_rel::Column::FromRbumId)).is_in(from_rbum_ids));
        }
        if let Some(to_rbum_item_id) = to_rbum_item_id {
            query.and_where(Expr::col((rbum_rel::Entity, rbum_rel::Column::ToRbumItemId)).eq(to_rbum_item_id));
        }
        query.cond_where(all![any![
            Expr::col((rbum_rel::Entity, rbum_rel::Column::OwnPaths)).like(format!("{}%", ctx.own_paths).as_str()),
            Expr::col((rbum_rel::Entity, rbum_rel::Column::ToOwnPaths)).like(format!("{}%", ctx.own_paths).as_str())
        ]]);
        if !from_attrs.is_empty() || !to_attrs.is_empty() {
            let attr_table_without_cond = Alias::new(format!("{}_without_cond", RbumRelAttrServ::get_table_name()));
            let attr_table_with_cond = Alias::new(format!("{}_with_cond", RbumRelAttrServ::get_table_name()));

            query.join_subquery(
                JoinType::LeftJoin,
                Query::select()
                    .column(rbum_rel_attr::Column::RelRbumRelId)
                    .expr_as(Expr::col(rbum_rel_attr::Column::RelRbumRelId).count(), Alias::new("attr_count"))
                    .from(rbum_rel_attr::Entity)
                    .and_where(Expr::col(rbum_rel_attr::Column::RecordOnly).eq(false))
                    .group_by_col(rbum_rel_attr::Column::RelRbumRelId)
                    .take(),
                attr_table_without_cond.clone(),
                Expr::col((attr_table_without_cond.clone(), rbum_rel_attr::Column::RelRbumRelId)).equals((rbum_rel::Entity, rbum_rel::Column::Id)),
            );

            let mut attr_conds = Cond::any();
            for (name, value) in from_attrs {
                attr_conds = attr_conds.add(all![
                    Expr::col(rbum_rel_attr::Column::Name).eq(name),
                    Expr::col(rbum_rel_attr::Column::Value).eq(value),
                    Expr::col(rbum_rel_attr::Column::IsFrom).eq(true)
                ]);
            }
            for (name, value) in to_attrs {
                attr_conds = attr_conds.add(all![
                    Expr::col(rbum_rel_attr::Column::Name).eq(name),
                    Expr::col(rbum_rel_attr::Column::Value).eq(value),
                    Expr::col(rbum_rel_attr::Column::IsFrom).eq(false)
                ]);
            }
            query.join_subquery(
                JoinType::LeftJoin,
                Query::select()
                    .column(rbum_rel_attr::Column::RelRbumRelId)
                    .expr_as(Expr::col(rbum_rel_attr::Column::RelRbumRelId).count(), Alias::new("attr_count"))
                    .from(rbum_rel_attr::Entity)
                    .and_where(Expr::col(rbum_rel_attr::Column::RecordOnly).eq(false))
                    .cond_where(attr_conds)
                    .group_by_col(rbum_rel_attr::Column::RelRbumRelId)
                    .take(),
                attr_table_with_cond.clone(),
                Expr::col((attr_table_with_cond.clone(), rbum_rel_attr::Column::RelRbumRelId)).equals((rbum_rel::Entity, rbum_rel::Column::Id)),
            );

            query.cond_where(any![
                all![
                    Expr::col((attr_table_without_cond.clone(), Alias::new("attr_count"))).is_null(),
                    Expr::col((attr_table_with_cond.clone(), Alias::new("attr_count"))).is_null()
                ],
                Expr::col((attr_table_without_cond.clone(), Alias::new("attr_count"))).eq(Expr::col((attr_table_with_cond.clone(), Alias::new("attr_count"))))
            ]);
        } else {
            // The incoming association property is empty, so the actual association property is also required to be empty
            //
            // 传入的关联属性为空，所以要求实际的关联属性也为空
            query.left_join(
                rbum_rel_attr::Entity,
                all![
                    Expr::col((rbum_rel_attr::Entity, rbum_rel_attr::Column::RelRbumRelId)).equals((rbum_rel::Entity, rbum_rel::Column::Id)),
                    Expr::col((rbum_rel_attr::Entity, rbum_rel_attr::Column::RecordOnly)).eq(false)
                ],
            );

            query.and_where(Expr::col((rbum_rel_attr::Entity, rbum_rel_attr::Column::Id)).is_null());
        }

        if !envs.is_empty() {
            let env_table_without_cond = Alias::new(format!("{}_without_cond", RbumRelEnvServ::get_table_name()));
            let env_table_with_cond = Alias::new(format!("{}_with_cond", RbumRelEnvServ::get_table_name()));

            query.join_subquery(
                JoinType::LeftJoin,
                Query::select()
                    .column(rbum_rel_env::Column::RelRbumRelId)
                    .expr_as(Expr::col(rbum_rel_env::Column::RelRbumRelId).count(), Alias::new("env_count"))
                    .from(rbum_rel_env::Entity)
                    .group_by_col(rbum_rel_env::Column::RelRbumRelId)
                    .take(),
                env_table_without_cond.clone(),
                Expr::col((env_table_without_cond.clone(), rbum_rel_env::Column::RelRbumRelId)).equals((rbum_rel::Entity, rbum_rel::Column::Id)),
            );
            let mut env_conds = Cond::any();
            for env in envs {
                match env.kind {
                    RbumRelEnvKind::DatetimeRange | RbumRelEnvKind::TimeRange => match env.value.parse::<i64>() {
                        Ok(num) => {
                            env_conds = env_conds.add(all![
                                Expr::col(rbum_rel_env::Column::Kind).eq(env.kind.to_int()),
                                Expr::expr(Func::cast_as(Expr::col(rbum_rel_env::Column::Value1), Alias::new("INTEGER"))).lte(num),
                                Expr::expr(Func::cast_as(Expr::col(rbum_rel_env::Column::Value2), Alias::new("INTEGER"))).gte(num)
                            ]);
                        }
                        Err(_) => {
                            return Err(funs.err().bad_request(
                                &Self::get_obj_name(),
                                "check",
                                &format!("env value {} is not a number", env.value),
                                "400-rbum-rel-env-value-not-number",
                            ));
                        }
                    },
                    RbumRelEnvKind::CallFrequency | RbumRelEnvKind::CallCount => match env.value.parse::<i64>() {
                        Ok(num) => {
                            env_conds = env_conds.add(all![
                                Expr::col(rbum_rel_env::Column::Kind).eq(env.kind.to_int()),
                                Expr::expr(Func::cast_as(Expr::col(rbum_rel_env::Column::Value1), Alias::new("INTEGER"))).gte(num)
                            ]);
                        }
                        Err(_) => {
                            return Err(funs.err().bad_request(
                                &Self::get_obj_name(),
                                "check",
                                &format!("env value {} is not a number", env.value),
                                "400-rbum-rel-env-value-not-number",
                            ));
                        }
                    },
                    RbumRelEnvKind::Ips => {
                        env_conds = env_conds.add(all![
                            Expr::col(rbum_rel_env::Column::Kind).eq(env.kind.to_int()),
                            Expr::col(rbum_rel_env::Column::Value1).like(format!("%{}%", env.value))
                        ]);
                    }
                }
            }
            query.join_subquery(
                JoinType::LeftJoin,
                Query::select()
                    .column(rbum_rel_env::Column::RelRbumRelId)
                    .expr_as(Expr::col(rbum_rel_env::Column::RelRbumRelId).count(), Alias::new("env_count"))
                    .from(rbum_rel_env::Entity)
                    .cond_where(env_conds)
                    .group_by_col(rbum_rel_env::Column::RelRbumRelId)
                    .take(),
                env_table_with_cond.clone(),
                Expr::col((env_table_with_cond.clone(), rbum_rel_env::Column::RelRbumRelId)).equals((rbum_rel::Entity, rbum_rel::Column::Id)),
            );
            query.cond_where(any![
                all![
                    Expr::col((env_table_without_cond.clone(), Alias::new("env_count"))).is_null(),
                    Expr::col((env_table_with_cond.clone(), Alias::new("env_count"))).is_null()
                ],
                Expr::col((env_table_without_cond.clone(), Alias::new("env_count"))).eq(Expr::col((env_table_with_cond.clone(), Alias::new("env_count"))))
            ]);
        } else {
            // The incoming association environment is empty, so the actual association environment is also required to be empty
            //
            // 传入的关联环境为空，所以要求实际的关联环境也为空
            query.left_join(
                rbum_rel_env::Entity,
                Expr::col((rbum_rel_env::Entity, rbum_rel_env::Column::RelRbumRelId)).equals((rbum_rel::Entity, rbum_rel::Column::Id)),
            );

            query.and_where(Expr::col((rbum_rel_env::Entity, rbum_rel_env::Column::Id)).is_null());
        }

        funs.db().count(&query).await.map(|i| i > 0)
    }

    /// Delete the relationship(Including all conditions)
    ///
    /// 删除关联关系（包含所有限定条件）
    pub async fn delete_rel_with_ext(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<u64> {
        let rbum_rel_env_ids = RbumRelEnvServ::find_id_rbums(
            &RbumRelExtFilterReq {
                basic: Default::default(),
                rel_rbum_rel_id: Some(id.to_string()),
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        let rbum_rel_attr_ids = RbumRelAttrServ::find_id_rbums(
            &RbumRelExtFilterReq {
                basic: Default::default(),
                rel_rbum_rel_id: Some(id.to_string()),
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        for rbum_rel_env_id in rbum_rel_env_ids {
            RbumRelEnvServ::delete_rbum(&rbum_rel_env_id, funs, ctx).await?;
        }
        for rbum_rel_attr_id in rbum_rel_attr_ids {
            RbumRelAttrServ::delete_rbum(&rbum_rel_attr_id, funs, ctx).await?;
        }
        RbumRelServ::delete_rbum(id, funs, ctx).await
    }
}

#[async_trait]
impl RbumCrudOperation<rbum_rel_attr::ActiveModel, RbumRelAttrAddReq, RbumRelAttrModifyReq, RbumRelAttrDetailResp, RbumRelAttrDetailResp, RbumRelExtFilterReq> for RbumRelAttrServ {
    fn get_table_name() -> &'static str {
        rbum_rel_attr::Entity.table_name()
    }

    async fn before_add_rbum(add_req: &mut RbumRelAttrAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        Self::check_ownership_with_table_name(&add_req.rel_rbum_rel_id, RbumRelServ::get_table_name(), funs, ctx).await?;
        if let Some(rel_rbum_kind_attr_id) = &add_req.rel_rbum_kind_attr_id {
            Self::check_scope(rel_rbum_kind_attr_id, RbumKindAttrServ::get_table_name(), funs, ctx).await?;
        }
        Ok(())
    }

    async fn package_add(add_req: &RbumRelAttrAddReq, funs: &TardisFunsInst, _: &TardisContext) -> TardisResult<rbum_rel_attr::ActiveModel> {
        let rbum_rel_attr_name = if let Some(rel_rbum_kind_attr_id) = &add_req.rel_rbum_kind_attr_id {
            funs.db()
                .get_dto::<NameResp>(
                    Query::select().column(rbum_kind_attr::Column::Name).from(rbum_kind_attr::Entity).and_where(Expr::col(rbum_kind_attr::Column::Id).eq(rel_rbum_kind_attr_id)),
                )
                .await?
                .ok_or_else(|| {
                    funs.err().not_found(
                        &Self::get_obj_name(),
                        "add",
                        &format!("not found rbum_kind_attr {}", rel_rbum_kind_attr_id),
                        "404-rbum-rel-not-exist-kind-attr",
                    )
                })?
                .name
        } else if let Some(name) = &add_req.name {
            name.to_string()
        } else {
            return Err(funs.err().not_found(
                &Self::get_obj_name(),
                "add",
                "[rel_rbum_kind_attr_id] and [name] cannot be empty at the same time",
                "400-rbum-rel-name-require",
            ));
        };

        if funs
            .db()
            .count(
                Query::select()
                    .column(rbum_rel_attr::Column::Id)
                    .from(rbum_rel_attr::Entity)
                    .and_where(Expr::col(rbum_rel_attr::Column::RelRbumRelId).eq(&add_req.rel_rbum_rel_id))
                    .and_where(Expr::col(rbum_rel_attr::Column::Name).eq(&rbum_rel_attr_name)),
            )
            .await?
            > 0
        {
            return Err(funs.err().conflict(
                &Self::get_obj_name(),
                "add",
                &format!("name {} already exists", rbum_rel_attr_name),
                "409-rbum-*-name-exist",
            ));
        }

        Ok(rbum_rel_attr::ActiveModel {
            id: Set(TardisFuns::field.nanoid()),
            is_from: Set(add_req.is_from),
            value: Set(add_req.value.to_string()),
            name: Set(rbum_rel_attr_name),
            record_only: Set(add_req.record_only),
            rel_rbum_kind_attr_id: Set(add_req.rel_rbum_kind_attr_id.as_ref().unwrap_or(&"".to_string()).to_string()),
            rel_rbum_rel_id: Set(add_req.rel_rbum_rel_id.to_string()),
            ..Default::default()
        })
    }

    async fn package_modify(id: &str, modify_req: &RbumRelAttrModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<rbum_rel_attr::ActiveModel> {
        let mut rbum_rel_attr = rbum_rel_attr::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        rbum_rel_attr.value = Set(modify_req.value.to_string());
        Ok(rbum_rel_attr)
    }

    async fn package_query(_: bool, filter: &RbumRelExtFilterReq, _: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<SelectStatement> {
        let mut query = Query::select();
        query
            .columns(vec![
                (rbum_rel_attr::Entity, rbum_rel_attr::Column::Id),
                (rbum_rel_attr::Entity, rbum_rel_attr::Column::IsFrom),
                (rbum_rel_attr::Entity, rbum_rel_attr::Column::Value),
                (rbum_rel_attr::Entity, rbum_rel_attr::Column::Name),
                (rbum_rel_attr::Entity, rbum_rel_attr::Column::RecordOnly),
                (rbum_rel_attr::Entity, rbum_rel_attr::Column::RelRbumKindAttrId),
                (rbum_rel_attr::Entity, rbum_rel_attr::Column::RelRbumRelId),
                (rbum_rel_attr::Entity, rbum_rel_attr::Column::OwnPaths),
                (rbum_rel_attr::Entity, rbum_rel_attr::Column::Owner),
                (rbum_rel_attr::Entity, rbum_rel_attr::Column::CreateTime),
                (rbum_rel_attr::Entity, rbum_rel_attr::Column::UpdateTime),
            ])
            .expr_as(
                Expr::col((rbum_kind_attr::Entity, rbum_kind_attr::Column::Name)).if_null(""),
                Alias::new("rel_rbum_kind_attr_name"),
            )
            .from(rbum_rel_attr::Entity)
            .left_join(
                rbum_kind_attr::Entity,
                Expr::col((rbum_kind_attr::Entity, rbum_kind_attr::Column::Id)).equals((rbum_rel_attr::Entity, rbum_rel_attr::Column::RelRbumKindAttrId)),
            );
        if let Some(rel_rbum_rel_id) = &filter.rel_rbum_rel_id {
            query.and_where(Expr::col((rbum_rel_attr::Entity, rbum_rel_attr::Column::RelRbumRelId)).eq(rel_rbum_rel_id.to_string()));
        }
        query.with_filter(Self::get_table_name(), &filter.basic, true, false, ctx);
        Ok(query)
    }
}

#[async_trait]
impl RbumCrudOperation<rbum_rel_env::ActiveModel, RbumRelEnvAddReq, RbumRelEnvModifyReq, RbumRelEnvDetailResp, RbumRelEnvDetailResp, RbumRelExtFilterReq> for RbumRelEnvServ {
    fn get_table_name() -> &'static str {
        rbum_rel_env::Entity.table_name()
    }

    async fn before_add_rbum(add_req: &mut RbumRelEnvAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        Self::check_ownership_with_table_name(&add_req.rel_rbum_rel_id, RbumRelServ::get_table_name(), funs, ctx).await?;
        Ok(())
    }

    async fn package_add(add_req: &RbumRelEnvAddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<rbum_rel_env::ActiveModel> {
        Ok(rbum_rel_env::ActiveModel {
            id: Set(TardisFuns::field.nanoid()),
            kind: Set(add_req.kind.to_int()),
            value1: Set(add_req.value1.to_string()),
            value2: Set(add_req.value2.as_ref().unwrap_or(&"".to_string()).to_string()),
            rel_rbum_rel_id: Set(add_req.rel_rbum_rel_id.to_string()),
            ..Default::default()
        })
    }

    async fn package_modify(id: &str, modify_req: &RbumRelEnvModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<rbum_rel_env::ActiveModel> {
        let mut rbum_rel_env = rbum_rel_env::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(value1) = &modify_req.value1 {
            rbum_rel_env.value1 = Set(value1.to_string());
        }
        if let Some(value2) = &modify_req.value2 {
            rbum_rel_env.value2 = Set(value2.to_string());
        }
        Ok(rbum_rel_env)
    }

    async fn package_query(_: bool, filter: &RbumRelExtFilterReq, _: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<SelectStatement> {
        let mut query = Query::select();
        query
            .columns(vec![
                (rbum_rel_env::Entity, rbum_rel_env::Column::Id),
                (rbum_rel_env::Entity, rbum_rel_env::Column::Kind),
                (rbum_rel_env::Entity, rbum_rel_env::Column::Value1),
                (rbum_rel_env::Entity, rbum_rel_env::Column::Value2),
                (rbum_rel_env::Entity, rbum_rel_env::Column::RelRbumRelId),
                (rbum_rel_env::Entity, rbum_rel_env::Column::OwnPaths),
                (rbum_rel_env::Entity, rbum_rel_env::Column::Owner),
                (rbum_rel_env::Entity, rbum_rel_env::Column::CreateTime),
                (rbum_rel_env::Entity, rbum_rel_env::Column::UpdateTime),
            ])
            .from(rbum_rel_env::Entity);

        if let Some(rel_rbum_rel_id) = &filter.rel_rbum_rel_id {
            query.and_where(Expr::col((rbum_rel_env::Entity, rbum_rel_env::Column::RelRbumRelId)).eq(rel_rbum_rel_id.to_string()));
        }
        query.with_filter(Self::get_table_name(), &filter.basic, true, false, ctx);
        Ok(query)
    }
}
