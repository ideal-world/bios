use std::str::FromStr;

use async_trait::async_trait;
use tardis::basic::dto::{TardisContext, TardisFunsInst};
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::chrono::Utc;
use tardis::db::reldb_client::IdResp;
use tardis::db::sea_orm::*;
use tardis::db::sea_query::*;
use tardis::web::web_resp::TardisPage;
use tardis::TardisFuns;

use crate::rbum::domain::{rbum_item, rbum_kind_attr, rbum_rel, rbum_rel_attr, rbum_rel_env, rbum_set, rbum_set_cate};
use crate::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumRelExtFilterReq, RbumRelFilterReq, RbumSetCateFilterReq, RbumSetItemFilterReq};
use crate::rbum::dto::rbum_rel_agg_dto::{RbumRelAggAddReq, RbumRelAggResp};
use crate::rbum::dto::rbum_rel_attr_dto::{RbumRelAttrAddReq, RbumRelAttrDetailResp, RbumRelAttrModifyReq};
use crate::rbum::dto::rbum_rel_dto::{RbumRelAddReq, RbumRelCheckReq, RbumRelDetailResp, RbumRelFindReq, RbumRelModifyReq};
use crate::rbum::dto::rbum_rel_env_dto::{RbumRelEnvAddReq, RbumRelEnvDetailResp, RbumRelEnvModifyReq};
use crate::rbum::rbum_enumeration::{RbumRelEnvKind, RbumRelFromKind};
use crate::rbum::serv::rbum_crud_serv::{NameResp, RbumCrudOperation, RbumCrudQueryPackage};
use crate::rbum::serv::rbum_item_serv::RbumItemServ;
use crate::rbum::serv::rbum_kind_serv::RbumKindAttrServ;
use crate::rbum::serv::rbum_set_serv::{RbumSetCateServ, RbumSetItemServ, RbumSetServ};

pub struct RbumRelServ;

pub struct RbumRelAttrServ;

pub struct RbumRelEnvServ;

#[async_trait]
impl<'a> RbumCrudOperation<'a, rbum_rel::ActiveModel, RbumRelAddReq, RbumRelModifyReq, RbumRelDetailResp, RbumRelDetailResp, RbumRelFilterReq> for RbumRelServ {
    fn get_table_name() -> &'static str {
        rbum_rel::Entity.table_name()
    }

    async fn package_add(add_req: &RbumRelAddReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<rbum_rel::ActiveModel> {
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

    async fn before_add_rbum(add_req: &mut RbumRelAddReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        let rel_rbum_table_name = match add_req.from_rbum_kind {
            RbumRelFromKind::Item => RbumItemServ::get_table_name(),
            RbumRelFromKind::Set => RbumSetServ::get_table_name(),
            RbumRelFromKind::SetCate => RbumSetCateServ::get_table_name(),
        };
        // The relationship check is changed from check_ownership to check_scope.
        // for example, the account corresponding to the tenant can be associated to the app,
        // where the account belongs to the tenant but scope=1, so it can be used by the application.
        Self::check_scope(&add_req.from_rbum_id, rel_rbum_table_name, funs, cxt).await?;
        if add_req.to_rbum_item_id.is_empty() {
            return Err(TardisError::BadRequest("to_rbum_item_id is empty".to_string()));
        }
        // It may not be possible to get the data of to_rbum_item_id when there are multiple database instances
        if !add_req.to_is_outside {
            Self::check_scope(&add_req.to_rbum_item_id, RbumItemServ::get_table_name(), funs, cxt).await?;
        }
        Ok(())
    }

    async fn package_modify(id: &str, modify_req: &RbumRelModifyReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<rbum_rel::ActiveModel> {
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

    async fn before_delete_rbum(id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        let mut query = Query::select();
        query.column(rbum_rel::Column::Id).from(rbum_rel::Entity).and_where(Expr::col(rbum_rel::Column::Id).eq(id)).cond_where(
            Cond::all().add(
                Cond::any()
                    .add(Expr::col(rbum_rel::Column::OwnPaths).like(format!("{}%", cxt.own_paths).as_str()))
                    .add(Expr::col(rbum_rel::Column::ToOwnPaths).like(format!("{}%", cxt.own_paths).as_str())),
            ),
        );
        if funs.db().count(&query).await? == 0 {
            return Err(TardisError::NotFound(format!("The ownership of {}.{} is illegal", Self::get_table_name(), id)));
        }
        Self::check_exist_before_delete(id, RbumRelAttrServ::get_table_name(), rbum_rel_attr::Column::RelRbumRelId.as_str(), funs).await?;
        Self::check_exist_before_delete(id, RbumRelEnvServ::get_table_name(), rbum_rel_env::Column::RelRbumRelId.as_str(), funs).await?;
        Ok(())
    }

    async fn package_query(_: bool, filter: &RbumRelFilterReq, _: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<SelectStatement> {
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
                Expr::tbl(from_rbum_item_table.clone(), rbum_item::Column::Name).if_null(""),
                Alias::new("from_rbum_item_name"),
            )
            .expr_as(Expr::tbl(rbum_set::Entity, rbum_set::Column::Name).if_null(""), Alias::new("from_rbum_set_name"))
            .expr_as(
                Expr::tbl(rbum_set_cate::Entity, rbum_set_cate::Column::Name).if_null(""),
                Alias::new("from_rbum_set_cate_name"),
            )
            .expr_as(Expr::tbl(to_rbum_item_table.clone(), rbum_item::Column::Name).if_null(""), Alias::new("to_rbum_item_name"))
            .from(rbum_rel::Entity)
            .join_as(
                JoinType::LeftJoin,
                rbum_item::Entity,
                from_rbum_item_table.clone(),
                Cond::all()
                    .add(Expr::tbl(from_rbum_item_table, rbum_item::Column::Id).equals(rbum_rel::Entity, rbum_rel::Column::FromRbumId))
                    .add(Expr::tbl(rbum_rel::Entity, rbum_rel::Column::FromRbumKind).eq(RbumRelFromKind::Item.to_int())),
            )
            .left_join(
                rbum_set::Entity,
                Cond::all()
                    .add(Expr::tbl(rbum_set::Entity, rbum_set::Column::Id).equals(rbum_rel::Entity, rbum_rel::Column::FromRbumId))
                    .add(Expr::tbl(rbum_rel::Entity, rbum_rel::Column::FromRbumKind).eq(RbumRelFromKind::Set.to_int())),
            )
            .left_join(
                rbum_set_cate::Entity,
                Cond::all()
                    .add(Expr::tbl(rbum_set_cate::Entity, rbum_set_cate::Column::Id).equals(rbum_rel::Entity, rbum_rel::Column::FromRbumId))
                    .add(Expr::tbl(rbum_rel::Entity, rbum_rel::Column::FromRbumKind).eq(RbumRelFromKind::SetCate.to_int())),
            )
            .join_as(
                JoinType::LeftJoin,
                rbum_item::Entity,
                to_rbum_item_table.clone(),
                Expr::tbl(to_rbum_item_table, rbum_item::Column::Id).equals(rbum_rel::Entity, rbum_rel::Column::ToRbumItemId),
            );

        if let Some(tag) = &filter.tag {
            query.and_where(Expr::tbl(rbum_rel::Entity, rbum_rel::Column::Tag).eq(tag.to_string()));
        }
        if let Some(from_rbum_kind) = &filter.from_rbum_kind {
            query.and_where(Expr::tbl(rbum_rel::Entity, rbum_rel::Column::FromRbumKind).eq(from_rbum_kind.to_int()));
        }
        if let Some(from_rbum_id) = &filter.from_rbum_id {
            query.and_where(Expr::tbl(rbum_rel::Entity, rbum_rel::Column::FromRbumId).eq(from_rbum_id.to_string()));
        }
        if let Some(to_rbum_item_id) = &filter.to_rbum_item_id {
            query.and_where(Expr::tbl(rbum_rel::Entity, rbum_rel::Column::ToRbumItemId).eq(to_rbum_item_id.to_string()));
        }
        if let Some(to_own_paths) = &filter.to_own_paths {
            query.and_where(Expr::tbl(rbum_rel::Entity, rbum_rel::Column::ToOwnPaths).eq(to_own_paths.to_string()));
        }
        query.with_filter(Self::get_table_name(), &filter.basic, true, false, cxt);
        Ok(query)
    }
}

impl<'a> RbumRelServ {
    pub async fn add_simple_rel(tag: &str, from_rbum_id: &str, to_rbum_item_id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        RbumRelServ::add_rbum(
            &mut RbumRelAddReq {
                tag: tag.to_string(),
                note: None,
                from_rbum_kind: RbumRelFromKind::Item,
                from_rbum_id: from_rbum_id.to_string(),
                to_rbum_item_id: to_rbum_item_id.to_string(),
                to_own_paths: cxt.own_paths.to_string(),
                to_is_outside: false,
                ext: None,
            },
            funs,
            cxt,
        )
        .await?;
        Ok(())
    }

    pub async fn add_rel(add_req: &mut RbumRelAggAddReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<String> {
        let rbum_rel_id = Self::add_rbum(&mut add_req.rel, funs, cxt).await?;
        for attr in &add_req.attrs {
            RbumRelAttrServ::add_rbum(
                &mut RbumRelAttrAddReq {
                    is_from: attr.is_from,
                    value: attr.value.to_string(),
                    name: attr.name.to_string(),
                    rel_rbum_rel_id: rbum_rel_id.to_string(),
                    rel_rbum_kind_attr_id: attr.rel_rbum_kind_attr_id.to_string(),
                    record_only: attr.record_only,
                },
                funs,
                cxt,
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
                cxt,
            )
            .await?;
        }
        Ok(rbum_rel_id)
    }

    pub async fn paginate_from_rels(
        tag: &str,
        from_rbum_kind: &RbumRelFromKind,
        from_rbum_id: &str,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<RbumRelAggResp>> {
        Self::paginate_rels(
            &RbumRelFilterReq {
                basic: Default::default(),
                tag: Some(tag.to_string()),
                from_rbum_kind: Some(from_rbum_kind.clone()),
                from_rbum_id: Some(from_rbum_id.to_string()),
                to_rbum_item_id: None,
                to_own_paths: None,
            },
            page_number,
            page_size,
            desc_sort_by_create,
            desc_sort_by_update,
            funs,
            cxt,
        )
        .await
    }

    pub async fn paginate_to_rels(
        tag: &str,
        to_rbum_item_id: &str,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<RbumRelAggResp>> {
        Self::paginate_rels(
            &RbumRelFilterReq {
                basic: RbumBasicFilterReq {
                    ignore_scope: true,
                    ..Default::default()
                },
                tag: Some(tag.to_string()),
                from_rbum_kind: None,
                from_rbum_id: None,
                to_rbum_item_id: Some(to_rbum_item_id.to_string()),
                to_own_paths: Some(cxt.own_paths.to_string()),
            },
            page_number,
            page_size,
            desc_sort_by_create,
            desc_sort_by_update,
            funs,
            cxt,
        )
        .await
    }

    async fn paginate_rels(
        filter: &RbumRelFilterReq,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<RbumRelAggResp>> {
        let rbum_rels = RbumRelServ::paginate_rbums(filter, page_number, page_size, desc_sort_by_create, desc_sort_by_update, funs, cxt).await?;
        let rbum_rel_ids: Vec<String> = rbum_rels.records.iter().map(|r| r.id.to_string()).collect();
        let mut result = Vec::with_capacity(rbum_rel_ids.len());
        for record in rbum_rels.records {
            let rbum_rel_id = record.id.to_string();
            let resp = RbumRelAggResp {
                rel: record,
                attrs: RbumRelAttrServ::find_rbums(
                    &RbumRelExtFilterReq {
                        basic: filter.basic.clone(),
                        rel_rbum_rel_id: Some(rbum_rel_id.clone()),
                    },
                    None,
                    None,
                    funs,
                    cxt,
                )
                .await?,
                envs: RbumRelEnvServ::find_rbums(
                    &RbumRelExtFilterReq {
                        basic: filter.basic.clone(),
                        rel_rbum_rel_id: Some(rbum_rel_id.clone()),
                    },
                    None,
                    None,
                    funs,
                    cxt,
                )
                .await?,
            };
            result.push(resp);
        }
        Ok(TardisPage {
            page_number,
            total_size: rbum_rel_ids.len() as u64,
            page_size,
            records: result,
        })
    }

    pub async fn find_rel_ids(find_req: &RbumRelFindReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<Vec<String>> {
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
        query.cond_where(
            Cond::all().add(
                Cond::any()
                    .add(Expr::col(rbum_rel::Column::OwnPaths).like(format!("{}%", cxt.own_paths).as_str()))
                    .add(Expr::col(rbum_rel::Column::ToOwnPaths).like(format!("{}%", cxt.own_paths).as_str())),
            ),
        );
        let ids = funs.db().find_dtos::<IdResp>(&query).await?.iter().map(|i| i.id.to_string()).collect::<Vec<String>>();
        Ok(ids)
    }

    pub async fn check_rel(check_req: &mut RbumRelCheckReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<bool> {
        if Self::do_check_rel(check_req, funs, cxt).await? {
            return Ok(true);
        }
        let rel_rbum_set_cate_ids = if check_req.from_rbum_kind == RbumRelFromKind::Item {
            // check set category
            RbumSetItemServ::find_rbums(
                &RbumSetItemFilterReq {
                    basic: Default::default(),
                    rel_rbum_set_id: None,
                    rel_rbum_set_cate_id: None,
                    rel_rbum_item_id: Some(check_req.from_rbum_id.clone()),
                },
                None,
                None,
                funs,
                cxt,
            )
            .await?
            .into_iter()
            .map(|i| i.rel_rbum_set_cate_id)
            .collect::<Vec<String>>()
        } else if check_req.from_rbum_kind == RbumRelFromKind::SetCate {
            vec![check_req.from_rbum_id.clone()]
        } else {
            return Ok(false);
        };
        for rel_rbum_set_cate_id in rel_rbum_set_cate_ids {
            let rbum_set_cate_base = RbumSetCateServ::peek_rbum(&rel_rbum_set_cate_id, &RbumSetCateFilterReq::default(), funs, cxt).await?;
            if check_req.from_rbum_kind != RbumRelFromKind::SetCate {
                check_req.from_rbum_kind = RbumRelFromKind::SetCate;
                check_req.from_rbum_id = rbum_set_cate_base.id.clone();
                // Check directly related records
                if Self::do_check_rel(check_req, funs, cxt).await? {
                    return Ok(true);
                }
            }
            check_req.from_rbum_kind = RbumRelFromKind::Set;
            check_req.from_rbum_id = rbum_set_cate_base.rel_rbum_set_id.clone();
            if Self::do_check_rel(check_req, funs, cxt).await? {
                return Ok(true);
            }
            let rbum_set_cate_with_rels = RbumSetCateServ::find_rbums(
                &RbumSetCateFilterReq {
                    basic: Default::default(),
                    rel_rbum_set_id: Some(rbum_set_cate_base.rel_rbum_set_id.clone()),
                    sys_code: Some(rbum_set_cate_base.sys_code.clone()),
                    find_parent: Some(true),
                },
                None,
                None,
                funs,
                cxt,
            )
            .await?;
            for rbum_set_cate in rbum_set_cate_with_rels {
                check_req.from_rbum_kind = RbumRelFromKind::SetCate;
                check_req.from_rbum_id = rbum_set_cate.id.clone();
                // Check indirectly related records
                if Self::do_check_rel(check_req, funs, cxt).await? {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    async fn do_check_rel(check_req: &RbumRelCheckReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<bool> {
        let rbum_rel_ids = Self::find_rel_ids(
            &RbumRelFindReq {
                tag: Some(check_req.tag.clone()),
                from_rbum_kind: Some(check_req.from_rbum_kind.clone()),
                from_rbum_id: Some(check_req.from_rbum_id.clone()),
                to_rbum_item_id: Some(check_req.to_rbum_item_id.clone()),
            },
            funs,
            cxt,
        )
        .await?;
        for rbum_rel_id in rbum_rel_ids {
            let mut found = true;
            let rbum_rel_attrs = funs
                .db()
                .find_dtos::<NameAndValueResp>(
                    Query::select()
                        .column(rbum_rel_attr::Column::IsFrom)
                        .column(rbum_rel_attr::Column::Name)
                        .column(rbum_rel_attr::Column::Value)
                        .from(rbum_rel_attr::Entity)
                        .and_where(Expr::col(rbum_rel_attr::Column::RelRbumRelId).eq(rbum_rel_id.clone()))
                        .and_where(Expr::col(rbum_rel_attr::Column::RecordOnly).eq(false)),
                )
                .await?;
            for rbum_rel_attr in rbum_rel_attrs {
                if rbum_rel_attr.is_from {
                    if let Some(value) = check_req.from_attrs.get(&rbum_rel_attr.name) {
                        if value != rbum_rel_attr.value.as_str() {
                            found = false;
                            break;
                        }
                    } else {
                        found = false;
                        break;
                    }
                } else if let Some(value) = check_req.to_attrs.get(&rbum_rel_attr.name) {
                    if value != rbum_rel_attr.value.as_str() {
                        found = false;
                        break;
                    }
                } else {
                    found = false;
                    break;
                }
            }
            let rbum_rel_envs = funs
                .db()
                .find_dtos::<KindAndValueResp>(
                    Query::select()
                        .column(rbum_rel_env::Column::Kind)
                        .column(rbum_rel_env::Column::Value1)
                        .column(rbum_rel_env::Column::Value2)
                        .from(rbum_rel_env::Entity)
                        .and_where(Expr::col(rbum_rel_env::Column::RelRbumRelId).eq(rbum_rel_id.clone())),
                )
                .await?;
            for rbum_rel_env in rbum_rel_envs {
                match rbum_rel_env.kind {
                    RbumRelEnvKind::DatetimeRange => {
                        if i64::from_str(rbum_rel_env.value1.as_str())? > Utc::now().timestamp() || i64::from_str(rbum_rel_env.value2.as_str())? < Utc::now().timestamp() {
                            found = false;
                            break;
                        }
                    }
                    RbumRelEnvKind::TimeRange => {
                        // TODO
                    }
                    RbumRelEnvKind::Ips => {
                        // TODO
                    }
                }
            }
            if found {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub async fn delete_rel_with_ext(id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        let rbum_rel_env_ids = RbumRelEnvServ::find_rbums(
            &RbumRelExtFilterReq {
                basic: Default::default(),
                rel_rbum_rel_id: Some(id.to_string()),
            },
            None,
            None,
            funs,
            cxt,
        )
        .await?
        .iter()
        .map(|rbum_rel_env| rbum_rel_env.id.clone())
        .collect::<Vec<String>>();
        let rbum_rel_attr_ids = RbumRelAttrServ::find_rbums(
            &RbumRelExtFilterReq {
                basic: Default::default(),
                rel_rbum_rel_id: Some(id.to_string()),
            },
            None,
            None,
            funs,
            cxt,
        )
        .await?
        .iter()
        .map(|rbum_rel_attr| rbum_rel_attr.id.clone())
        .collect::<Vec<String>>();
        for rbum_rel_env_id in rbum_rel_env_ids {
            RbumRelEnvServ::delete_rbum(&rbum_rel_env_id, funs, cxt).await?;
        }
        for rbum_rel_attr_id in rbum_rel_attr_ids {
            RbumRelAttrServ::delete_rbum(&rbum_rel_attr_id, funs, cxt).await?;
        }
        RbumRelServ::delete_rbum(id, funs, cxt).await
    }
}

#[async_trait]
impl<'a> RbumCrudOperation<'a, rbum_rel_attr::ActiveModel, RbumRelAttrAddReq, RbumRelAttrModifyReq, RbumRelAttrDetailResp, RbumRelAttrDetailResp, RbumRelExtFilterReq>
    for RbumRelAttrServ
{
    fn get_table_name() -> &'static str {
        rbum_rel_attr::Entity.table_name()
    }

    async fn package_add(add_req: &RbumRelAttrAddReq, funs: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<rbum_rel_attr::ActiveModel> {
        let rbum_rel_attr_name = funs
            .db()
            .get_dto::<NameResp>(
                Query::select()
                    .column(rbum_kind_attr::Column::Name)
                    .from(rbum_kind_attr::Entity)
                    .and_where(Expr::col(rbum_kind_attr::Column::Id).eq(add_req.rel_rbum_kind_attr_id.as_str())),
            )
            .await?
            .ok_or_else(|| TardisError::NotFound(format!("rbum_kind_attr not found: {}", add_req.rel_rbum_kind_attr_id.as_str())))?
            .name;
        Ok(rbum_rel_attr::ActiveModel {
            id: Set(TardisFuns::field.nanoid()),
            is_from: Set(add_req.is_from),
            value: Set(add_req.value.to_string()),
            name: Set(rbum_rel_attr_name),
            record_only: Set(add_req.record_only),
            rel_rbum_kind_attr_id: Set(add_req.rel_rbum_kind_attr_id.to_string()),
            rel_rbum_rel_id: Set(add_req.rel_rbum_rel_id.to_string()),
            ..Default::default()
        })
    }

    async fn before_add_rbum(add_req: &mut RbumRelAttrAddReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Self::check_ownership_with_table_name(&add_req.rel_rbum_rel_id, RbumRelServ::get_table_name(), funs, cxt).await?;
        Self::check_scope(&add_req.rel_rbum_kind_attr_id, RbumKindAttrServ::get_table_name(), funs, cxt).await?;
        Ok(())
    }

    async fn package_modify(id: &str, modify_req: &RbumRelAttrModifyReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<rbum_rel_attr::ActiveModel> {
        let mut rbum_rel_attr = rbum_rel_attr::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(value) = &modify_req.value {
            rbum_rel_attr.value = Set(value.to_string());
        }
        if let Some(name) = &modify_req.name {
            rbum_rel_attr.name = Set(name.to_string());
        }
        Ok(rbum_rel_attr)
    }

    async fn package_query(_: bool, filter: &RbumRelExtFilterReq, _: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<SelectStatement> {
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
            .expr_as(Expr::tbl(rbum_kind_attr::Entity, rbum_kind_attr::Column::Name), Alias::new("rel_rbum_kind_attr_name"))
            .from(rbum_rel_attr::Entity)
            .inner_join(
                rbum_kind_attr::Entity,
                Expr::tbl(rbum_kind_attr::Entity, rbum_kind_attr::Column::Id).equals(rbum_rel_attr::Entity, rbum_rel_attr::Column::RelRbumKindAttrId),
            );
        if let Some(rel_rbum_rel_id) = &filter.rel_rbum_rel_id {
            query.and_where(Expr::tbl(rbum_rel_attr::Entity, rbum_rel_attr::Column::RelRbumRelId).eq(rel_rbum_rel_id.to_string()));
        }
        query.with_filter(Self::get_table_name(), &filter.basic, true, false, cxt);
        Ok(query)
    }
}

#[async_trait]
impl<'a> RbumCrudOperation<'a, rbum_rel_env::ActiveModel, RbumRelEnvAddReq, RbumRelEnvModifyReq, RbumRelEnvDetailResp, RbumRelEnvDetailResp, RbumRelExtFilterReq>
    for RbumRelEnvServ
{
    fn get_table_name() -> &'static str {
        rbum_rel_env::Entity.table_name()
    }

    async fn package_add(add_req: &RbumRelEnvAddReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<rbum_rel_env::ActiveModel> {
        Ok(rbum_rel_env::ActiveModel {
            id: Set(TardisFuns::field.nanoid()),
            kind: Set(add_req.kind.to_int()),
            value1: Set(add_req.value1.to_string()),
            value2: Set(add_req.value2.as_ref().unwrap_or(&"".to_string()).to_string()),
            rel_rbum_rel_id: Set(add_req.rel_rbum_rel_id.to_string()),
            ..Default::default()
        })
    }

    async fn before_add_rbum(add_req: &mut RbumRelEnvAddReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Self::check_ownership_with_table_name(&add_req.rel_rbum_rel_id, RbumRelServ::get_table_name(), funs, cxt).await?;
        Ok(())
    }

    async fn package_modify(id: &str, modify_req: &RbumRelEnvModifyReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<rbum_rel_env::ActiveModel> {
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

    async fn package_query(_: bool, filter: &RbumRelExtFilterReq, _: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<SelectStatement> {
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
            query.and_where(Expr::tbl(rbum_rel_env::Entity, rbum_rel_env::Column::RelRbumRelId).eq(rel_rbum_rel_id.to_string()));
        }
        query.with_filter(Self::get_table_name(), &filter.basic, true, false, cxt);
        Ok(query)
    }
}

#[derive(Debug, FromQueryResult)]
struct KindAndValueResp {
    pub kind: RbumRelEnvKind,
    pub value1: String,
    pub value2: String,
}

#[derive(Debug, FromQueryResult)]
struct NameAndValueResp {
    pub is_from: bool,
    pub name: String,
    pub value: String,
}
