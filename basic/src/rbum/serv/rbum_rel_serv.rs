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

use crate::rbum::domain::{rbum_kind_attr, rbum_rel, rbum_rel_attr, rbum_rel_env};
use crate::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumRelExtFilterReq, RbumRelFilterReq};
use crate::rbum::dto::rbum_rel_agg_dto::{RbumRelAggAddReq, RbumRelAggResp};
use crate::rbum::dto::rbum_rel_attr_dto::{RbumRelAttrAddReq, RbumRelAttrDetailResp, RbumRelAttrModifyReq};
use crate::rbum::dto::rbum_rel_dto::{RbumRelAddReq, RbumRelCheckReq, RbumRelDetailResp, RbumRelFindReq, RbumRelModifyReq};
use crate::rbum::dto::rbum_rel_env_dto::{RbumRelEnvAddReq, RbumRelEnvDetailResp, RbumRelEnvModifyReq};
use crate::rbum::rbum_enumeration::{RbumRelEnvKind, RbumRelFromKind};
use crate::rbum::serv::rbum_crud_serv::{NameResp, RbumCrudOperation, RbumCrudQueryPackage};
use crate::rbum::serv::rbum_item_serv::RbumItemServ;
use crate::rbum::serv::rbum_kind_serv::RbumKindAttrServ;
use crate::rbum::serv::rbum_set_serv::{RbumSetCateServ, RbumSetServ};

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
        Self::check_scope(&add_req.from_rbum_id, rel_rbum_table_name, funs, cxt).await?;
        Self::check_scope(&add_req.to_rbum_item_id, RbumItemServ::get_table_name(), funs, cxt).await?;
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
        Self::check_ownership(id, funs, cxt).await?;
        Self::check_exist_before_delete(id, RbumRelAttrServ::get_table_name(), rbum_rel_attr::Column::RelRbumRelId.as_str(), funs).await?;
        Self::check_exist_before_delete(id, RbumRelEnvServ::get_table_name(), rbum_rel_env::Column::RelRbumRelId.as_str(), funs).await?;
        Ok(())
    }

    async fn package_query(_: bool, filter: &RbumRelFilterReq, _: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<SelectStatement> {
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
            .from(rbum_rel::Entity);

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
                ..Default::default()
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
                ..Default::default()
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
                        rel_rbum_rel_id: Some(rbum_rel_id.clone()),
                        ..Default::default()
                    },
                    None,
                    None,
                    funs,
                    cxt,
                )
                .await?,
                envs: RbumRelEnvServ::find_rbums(
                    &RbumRelExtFilterReq {
                        rel_rbum_rel_id: Some(rbum_rel_id.clone()),
                        ..Default::default()
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

    pub async fn find_rel_id(find_req: &RbumRelFindReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<Option<String>> {
        let mut query = Query::select();
        query
            .column(rbum_rel::Column::Id)
            .from(rbum_rel::Entity)
            .and_where(Expr::col(rbum_rel::Column::Tag).eq(find_req.tag.as_str()))
            .and_where(Expr::col(rbum_rel::Column::FromRbumKind).eq(find_req.from_rbum_kind.to_int()))
            .and_where(Expr::col(rbum_rel::Column::FromRbumId).eq(find_req.from_rbum_id.as_str()))
            .and_where(Expr::col(rbum_rel::Column::ToRbumItemId).eq(find_req.to_rbum_item_id.as_str()))
            .cond_where(
                Cond::all()
                    .add(Cond::any().add(Expr::col(rbum_rel::Column::OwnPaths).eq(cxt.own_paths.as_str())).add(Expr::col(rbum_rel::Column::ToOwnPaths).eq(cxt.own_paths.as_str()))),
            );
        let id = funs.db().get_dto::<IdResp>(&query).await?;
        Ok(id.map(|resp| resp.id))
    }

    pub async fn check_rel(check_req: &RbumRelCheckReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<bool> {
        let rbum_rel_id = Self::find_rel_id(
            &RbumRelFindReq {
                tag: check_req.tag.clone(),
                from_rbum_kind: check_req.from_rbum_kind.clone(),
                from_rbum_id: check_req.from_rbum_id.clone(),
                to_rbum_item_id: check_req.to_rbum_item_id.clone(),
            },
            funs,
            cxt,
        )
        .await?;
        if let Some(rbum_rel_id) = rbum_rel_id {
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
                            return Ok(false);
                        }
                    } else {
                        return Ok(false);
                    }
                } else if let Some(value) = check_req.to_attrs.get(&rbum_rel_attr.name) {
                    if value != rbum_rel_attr.value.as_str() {
                        return Ok(false);
                    }
                } else {
                    return Ok(false);
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
                let kind = RbumRelEnvKind::from_str(rbum_rel_env.kind.as_str())
                    .map_err(|_| TardisError::FormatError(format!("rel env kind convert error {}", rbum_rel_env.kind.as_str())))?;
                match kind {
                    RbumRelEnvKind::DatetimeRange => {
                        if i64::from_str(rbum_rel_env.value1.as_str())? > Utc::now().timestamp() || i64::from_str(rbum_rel_env.value2.as_str())? < Utc::now().timestamp() {
                            return Ok(false);
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
            Ok(true)
        } else {
            Ok(false)
        }
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
            kind: Set(add_req.kind.to_string()),
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
    pub kind: String,
    pub value1: String,
    pub value2: String,
}

#[derive(Debug, FromQueryResult)]
struct NameAndValueResp {
    pub is_from: bool,
    pub name: String,
    pub value: String,
}
