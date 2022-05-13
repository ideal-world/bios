use async_trait::async_trait;
use serde::Serialize;
use tardis::basic::dto::{TardisContext, TardisFunsInst};
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm::*;
use tardis::db::sea_query::*;
use tardis::web::poem_openapi::types::{ParseFromJSON, ToJSON};
use tardis::web::web_resp::TardisPage;
use tardis::TardisFuns;

use crate::rbum::domain::{rbum_cert, rbum_cert_conf, rbum_domain, rbum_item, rbum_item_attr, rbum_kind, rbum_kind_attr, rbum_rel, rbum_set_item};
use crate::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumItemAttrFilterReq, RbumItemFilterFetcher, RbumKindAttrFilterReq};
use crate::rbum::dto::rbum_item_attr_dto::{RbumItemAttrAddReq, RbumItemAttrDetailResp, RbumItemAttrModifyReq, RbumItemAttrSummaryResp, RbumItemAttrsAddOrModifyReq};
use crate::rbum::dto::rbum_item_dto::{RbumItemAddReq, RbumItemDetailResp, RbumItemKernelAddReq, RbumItemModifyReq, RbumItemSummaryResp};
use crate::rbum::dto::rbum_kind_attr_dto::RbumKindAttrSummaryResp;
use crate::rbum::dto::rbum_rel_dto::{RbumRelAddReq, RbumRelFindReq};
use crate::rbum::rbum_enumeration::{RbumCertRelKind, RbumRelFromKind, RbumScopeLevelKind};
use crate::rbum::serv::rbum_cert_serv::{RbumCertConfServ, RbumCertServ};
use crate::rbum::serv::rbum_crud_serv::{RbumCrudOperation, RbumCrudQueryPackage, CREATE_TIME_FIELD, ID_FIELD, UPDATE_TIME_FIELD};
use crate::rbum::serv::rbum_domain_serv::RbumDomainServ;
use crate::rbum::serv::rbum_kind_serv::{RbumKindAttrServ, RbumKindServ};
use crate::rbum::serv::rbum_rel_serv::RbumRelServ;
use crate::rbum::serv::rbum_set_serv::RbumSetItemServ;

pub struct RbumItemServ;
pub struct RbumItemAttrServ;

#[async_trait]
impl<'a> RbumCrudOperation<'a, rbum_item::ActiveModel, RbumItemAddReq, RbumItemModifyReq, RbumItemSummaryResp, RbumItemDetailResp, RbumBasicFilterReq> for RbumItemServ {
    fn get_table_name() -> &'static str {
        rbum_item::Entity.table_name()
    }

    async fn package_add(add_req: &RbumItemAddReq, funs: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<rbum_item::ActiveModel> {
        let id = if let Some(id) = &add_req.id { id.0.clone() } else { TardisFuns::field.nanoid() };
        let code = if let Some(code) = &add_req.code {
            if funs
                .db()
                .count(
                    Query::select()
                        .column((rbum_item::Entity, rbum_item::Column::Id))
                        .from(rbum_item::Entity)
                        .inner_join(
                            rbum_domain::Entity,
                            Expr::tbl(rbum_domain::Entity, rbum_domain::Column::Id).equals(rbum_item::Entity, rbum_item::Column::RelRbumDomainId),
                        )
                        .inner_join(
                            rbum_kind::Entity,
                            Expr::tbl(rbum_kind::Entity, rbum_kind::Column::Id).equals(rbum_item::Entity, rbum_item::Column::RelRbumKindId),
                        )
                        .and_where(Expr::tbl(rbum_item::Entity, rbum_item::Column::Code).eq(code.0.as_str())),
                )
                .await?
                > 0
            {
                return Err(TardisError::BadRequest(format!("code {} already exists", code)));
            }
            code.0.clone()
        } else {
            id.clone()
        };
        Ok(rbum_item::ActiveModel {
            id: Set(id),
            code: Set(code),
            name: Set(add_req.name.to_string()),
            rel_rbum_kind_id: Set(add_req.rel_rbum_kind_id.to_string()),
            rel_rbum_domain_id: Set(add_req.rel_rbum_domain_id.to_string()),
            scope_level: Set(add_req.scope_level.as_ref().unwrap_or(&RbumScopeLevelKind::Private).to_int()),
            disabled: Set(add_req.disabled.unwrap_or(false)),
            ..Default::default()
        })
    }

    async fn before_add_rbum(add_req: &mut RbumItemAddReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Self::check_scope(&add_req.rel_rbum_kind_id, RbumKindServ::get_table_name(), funs, cxt).await?;
        Self::check_scope(&add_req.rel_rbum_domain_id, RbumDomainServ::get_table_name(), funs, cxt).await?;
        Ok(())
    }

    async fn package_modify(id: &str, modify_req: &RbumItemModifyReq, funs: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<rbum_item::ActiveModel> {
        let mut rbum_item = rbum_item::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(code) = &modify_req.code {
            if funs
                .db()
                .count(
                    Query::select()
                        .column((rbum_item::Entity, rbum_item::Column::Id))
                        .from(rbum_item::Entity)
                        .inner_join(
                            rbum_domain::Entity,
                            Expr::tbl(rbum_domain::Entity, rbum_domain::Column::Id).equals(rbum_item::Entity, rbum_item::Column::RelRbumDomainId),
                        )
                        .inner_join(
                            rbum_kind::Entity,
                            Expr::tbl(rbum_kind::Entity, rbum_kind::Column::Id).equals(rbum_item::Entity, rbum_item::Column::RelRbumKindId),
                        )
                        .and_where(Expr::tbl(rbum_item::Entity, rbum_item::Column::Code).eq(code.0.as_str()))
                        .and_where(Expr::tbl(rbum_item::Entity, rbum_item::Column::Id).ne(id)),
                )
                .await?
                > 0
            {
                return Err(TardisError::BadRequest(format!("code {} already exists", code)));
            }
            rbum_item.code = Set(code.to_string());
        }
        if let Some(name) = &modify_req.name {
            rbum_item.name = Set(name.to_string());
        }
        if let Some(scope_level) = &modify_req.scope_level {
            rbum_item.scope_level = Set(scope_level.to_int());
        }
        if let Some(disabled) = modify_req.disabled {
            rbum_item.disabled = Set(disabled);
        }
        Ok(rbum_item)
    }

    async fn before_delete_rbum(id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Self::check_ownership(id, funs, cxt).await?;
        Self::check_exist_before_delete(id, RbumItemAttrServ::get_table_name(), rbum_item_attr::Column::RelRbumItemId.as_str(), funs).await?;
        Self::check_exist_with_cond_before_delete(
            RbumRelServ::get_table_name(),
            Cond::any()
                .add(Cond::all().add(Expr::col(rbum_rel::Column::FromRbumKind).eq(RbumRelFromKind::Item.to_int())).add(Expr::col(rbum_rel::Column::FromRbumId).eq(id)))
                .add(Expr::col(rbum_rel::Column::ToRbumItemId).eq(id)),
            funs,
        )
        .await?;
        Self::check_exist_before_delete(id, RbumSetItemServ::get_table_name(), rbum_set_item::Column::RelRbumItemId.as_str(), funs).await?;
        Self::check_exist_before_delete(id, RbumCertConfServ::get_table_name(), rbum_cert_conf::Column::RelRbumItemId.as_str(), funs).await?;
        Self::check_exist_with_cond_before_delete(
            RbumCertServ::get_table_name(),
            Cond::all().add(Expr::col(rbum_cert::Column::RelRbumKind).eq(RbumCertRelKind::Item.to_int())).add(Expr::col(rbum_cert::Column::RelRbumId).eq(id)),
            funs,
        )
        .await?;
        Ok(())
    }

    async fn package_query(is_detail: bool, filter: &RbumBasicFilterReq, _: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<SelectStatement> {
        let mut query = Query::select();
        query
            .columns(vec![
                (rbum_item::Entity, rbum_item::Column::Id),
                (rbum_item::Entity, rbum_item::Column::Code),
                (rbum_item::Entity, rbum_item::Column::Name),
                (rbum_item::Entity, rbum_item::Column::RelRbumKindId),
                (rbum_item::Entity, rbum_item::Column::RelRbumDomainId),
                (rbum_item::Entity, rbum_item::Column::OwnPaths),
                (rbum_item::Entity, rbum_item::Column::Owner),
                (rbum_item::Entity, rbum_item::Column::CreateTime),
                (rbum_item::Entity, rbum_item::Column::UpdateTime),
                (rbum_item::Entity, rbum_item::Column::ScopeLevel),
                (rbum_item::Entity, rbum_item::Column::Disabled),
            ])
            .from(rbum_item::Entity);

        if is_detail {
            query
                .expr_as(Expr::tbl(rbum_kind::Entity, rbum_kind::Column::Name), Alias::new("rel_rbum_kind_name"))
                .expr_as(Expr::tbl(rbum_domain::Entity, rbum_domain::Column::Name), Alias::new("rel_rbum_domain_name"))
                .inner_join(
                    rbum_kind::Entity,
                    Expr::tbl(rbum_kind::Entity, rbum_kind::Column::Id).equals(rbum_item::Entity, rbum_item::Column::RelRbumKindId),
                )
                .inner_join(
                    rbum_domain::Entity,
                    Expr::tbl(rbum_domain::Entity, rbum_domain::Column::Id).equals(rbum_item::Entity, rbum_item::Column::RelRbumDomainId),
                );
        }
        query.with_filter(Self::get_table_name(), filter, is_detail, true, cxt);
        Ok(query)
    }
}

#[async_trait]
pub trait RbumItemCrudOperation<'a, EXT, AddReq, ModifyReq, SummaryResp, DetailResp, ItemFilterReq>
where
    EXT: TardisActiveModel + Sync + Send,
    AddReq: Sync + Send,
    ModifyReq: Sync + Send,
    SummaryResp: FromQueryResult + ParseFromJSON + ToJSON + Serialize + Send + Sync,
    DetailResp: FromQueryResult + ParseFromJSON + ToJSON + Serialize + Send + Sync,
    ItemFilterReq: Sync + Send + RbumItemFilterFetcher,
{
    fn get_ext_table_name() -> &'static str;
    fn get_rbum_kind_id() -> String;
    fn get_rbum_domain_id() -> String;

    // ----------------------------- Add -------------------------------

    async fn package_item_add(add_req: &AddReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<RbumItemKernelAddReq>;

    async fn package_ext_add(id: &str, add_req: &AddReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<EXT>;

    async fn before_add_item(_: &mut AddReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    async fn after_add_item(_: &str, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    async fn add_item(add_req: &mut AddReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<String> {
        Self::before_add_item(add_req, funs, cxt).await?;
        let item_add_req = Self::package_item_add(add_req, funs, cxt).await?;
        let mut item_add_req = RbumItemAddReq {
            id: item_add_req.id.clone(),
            code: item_add_req.code.clone(),
            name: item_add_req.name.clone(),
            rel_rbum_kind_id: Self::get_rbum_kind_id(),
            rel_rbum_domain_id: Self::get_rbum_domain_id(),
            scope_level: item_add_req.scope_level.clone(),
            disabled: item_add_req.disabled,
        };
        let id = RbumItemServ::add_rbum(&mut item_add_req, funs, cxt).await?;
        let ext_domain = Self::package_ext_add(&id, add_req, funs, cxt).await?;
        funs.db().insert_one(ext_domain, cxt).await?;
        Self::after_add_item(&id, funs, cxt).await?;
        Ok(id)
    }

    async fn add_item_with_simple_rel_by_from(add_req: &mut AddReq, tag: &str, to_rbum_item_id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<String> {
        let id = Self::add_item(add_req, funs, cxt).await?;
        RbumRelServ::add_rbum(
            &mut RbumRelAddReq {
                tag: tag.to_string(),
                note: None,
                from_rbum_kind: RbumRelFromKind::Item,
                from_rbum_id: id.to_string(),
                to_rbum_item_id: to_rbum_item_id.to_string(),
                to_own_paths: cxt.own_paths.to_string(),
                to_is_outside: false,
                ext: None,
            },
            funs,
            cxt,
        )
        .await?;
        Ok(id)
    }

    async fn add_item_with_simple_rel_by_to(add_req: &mut AddReq, tag: &str, from_rbum_item_id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<String> {
        let id = Self::add_item(add_req, funs, cxt).await?;
        RbumRelServ::add_rbum(
            &mut RbumRelAddReq {
                tag: tag.to_string(),
                note: None,
                from_rbum_kind: RbumRelFromKind::Item,
                from_rbum_id: from_rbum_item_id.to_string(),
                to_rbum_item_id: id.to_string(),
                to_own_paths: cxt.own_paths.to_string(),
                to_is_outside: false,
                ext: None,
            },
            funs,
            cxt,
        )
        .await?;
        Ok(id)
    }

    // ----------------------------- Modify -------------------------------

    async fn package_item_modify(id: &str, modify_req: &ModifyReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<Option<RbumItemModifyReq>>;

    async fn package_ext_modify(id: &str, modify_req: &ModifyReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<Option<EXT>>;

    async fn before_modify_item(_: &str, _: &mut ModifyReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    async fn after_modify_item(_: &str, _: &mut ModifyReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    async fn modify_item(id: &str, modify_req: &mut ModifyReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Self::before_modify_item(id, modify_req, funs, cxt).await?;
        let item_modify_req = Self::package_item_modify(id, modify_req, funs, cxt).await?;
        if let Some(mut item_modify_req) = item_modify_req {
            RbumItemServ::modify_rbum(id, &mut item_modify_req, funs, cxt).await?;
        } else {
            RbumItemServ::check_ownership(id, funs, cxt).await?;
        }
        let ext_domain = Self::package_ext_modify(id, modify_req, funs, cxt).await?;
        if let Some(ext_domain) = ext_domain {
            funs.db().update_one(ext_domain, cxt).await?;
        }
        Self::after_modify_item(id, modify_req, funs, cxt).await
    }

    // ----------------------------- Delete -------------------------------

    async fn package_delete(id: &str, _funs: &TardisFunsInst<'a>, _cxt: &TardisContext) -> TardisResult<Select<EXT::Entity>> {
        Ok(EXT::Entity::find().filter(Expr::col(ID_FIELD.clone()).eq(id)))
    }

    async fn before_delete_item(_: &str, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    async fn after_delete_item(_: &str, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    async fn delete_item(id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        Self::before_delete_item(id, funs, cxt).await?;
        RbumItemServ::delete_rbum(id, funs, cxt).await?;
        let select = Self::package_delete(id, funs, cxt).await?;
        #[cfg(feature = "with-mq")]
        {
            let delete_records = funs.db().soft_delete_custom(select, "id").await?;
            let mq_topic_entity_deleted = &crate::rbum::rbum_config::RbumConfigManager::get(funs.module_code())?.mq_topic_entity_deleted;
            let mq_header = std::collections::HashMap::from([(
                crate::rbum::rbum_config::RbumConfigManager::get(funs.module_code())?.mq_header_name_operator,
                cxt.owner.clone(),
            )]);
            for delete_record in &delete_records {
                funs.mq().request(mq_topic_entity_deleted, TardisFuns::json.obj_to_string(delete_record)?, &mq_header).await?;
            }
            Self::after_delete_item(id, funs, cxt).await?;
            Ok(delete_records.len() as u64)
        }
        #[cfg(not(feature = "with-mq"))]
        {
            let delete_records = funs.db().soft_delete(select, &cxt.owner).await?;
            Ok(delete_records)
        }
    }

    async fn delete_item_with_all_rels(id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        let rel_ids = RbumRelServ::find_rel_ids(
            &RbumRelFindReq {
                tag: None,
                from_rbum_kind: Some(RbumRelFromKind::Item),
                from_rbum_id: Some(id.to_string()),
                to_rbum_item_id: None,
            },
            funs,
            cxt,
        )
        .await?;
        for rel_id in rel_ids {
            RbumRelServ::delete_rel_with_ext(&rel_id, funs, cxt).await?;
        }
        let rel_ids = RbumRelServ::find_rel_ids(
            &RbumRelFindReq {
                tag: None,
                from_rbum_kind: Some(RbumRelFromKind::Item),
                from_rbum_id: None,
                to_rbum_item_id: Some(id.to_string()),
            },
            funs,
            cxt,
        )
        .await?;
        for rel_id in rel_ids {
            RbumRelServ::delete_rel_with_ext(&rel_id, funs, cxt).await?;
        }
        Self::delete_item(id, funs, cxt).await
    }

    // ----------------------------- Query -------------------------------

    async fn package_item_query(is_detail: bool, filter: &ItemFilterReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<SelectStatement> {
        let mut query = RbumItemServ::package_query(
            is_detail,
            &RbumBasicFilterReq {
                ignore_scope: filter.basic().ignore_scope,
                rel_cxt_owner: filter.basic().rel_cxt_owner,
                own_paths: filter.basic().own_paths.clone(),
                with_sub_own_paths: filter.basic().with_sub_own_paths,
                ids: filter.basic().ids.clone(),
                scope_level: filter.basic().scope_level.clone(),
                enabled: filter.basic().enabled,
                name: filter.basic().name.clone(),
                code: filter.basic().code.clone(),
                rbum_kind_id: if filter.basic().rbum_kind_id.is_some() {
                    filter.basic().rbum_kind_id.clone()
                } else {
                    Some(Self::get_rbum_kind_id())
                },
                rbum_domain_id: if filter.basic().rbum_domain_id.is_some() {
                    filter.basic().rbum_domain_id.clone()
                } else {
                    Some(Self::get_rbum_domain_id())
                },
                desc_by_sort: filter.basic().desc_by_sort,
            },
            funs,
            cxt,
        )
        .await?;
        if let Some(rbum_item_rel_filter_req) = &filter.rel() {
            if rbum_item_rel_filter_req.rel_by_from {
                query.inner_join(
                    rbum_rel::Entity,
                    Expr::tbl(rbum_rel::Entity, rbum_rel::Column::FromRbumId).equals(rbum_item::Entity, rbum_item::Column::Id),
                );
                if let Some(rel_item_id) = &rbum_item_rel_filter_req.rel_item_id {
                    query.and_where(Expr::tbl(rbum_rel::Entity, rbum_rel::Column::ToRbumItemId).eq(rel_item_id.to_string()));
                }
            } else {
                query.inner_join(
                    rbum_rel::Entity,
                    Expr::tbl(rbum_rel::Entity, rbum_rel::Column::ToRbumItemId).equals(rbum_item::Entity, rbum_item::Column::Id),
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
        Ok(query)
    }

    async fn package_ext_query(query: &mut SelectStatement, is_detail: bool, filter: &ItemFilterReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()>;

    async fn get_item(id: &str, filter: &ItemFilterReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<DetailResp> {
        let mut query = Self::package_item_query(true, filter, funs, cxt).await?;
        query.inner_join(
            Alias::new(Self::get_ext_table_name()),
            Expr::tbl(Alias::new(Self::get_ext_table_name()), ID_FIELD.clone()).equals(rbum_item::Entity, rbum_item::Column::Id),
        );
        Self::package_ext_query(&mut query, true, filter, funs, cxt).await?;
        query.and_where(Expr::tbl(rbum_item::Entity, rbum_item::Column::Id).eq(id));
        let query = funs.db().get_dto(&query).await?;
        match query {
            Some(resp) => Ok(resp),
            // TODO
            None => Err(TardisError::NotFound("".to_string())),
        }
    }

    async fn paginate_items(
        filter: &ItemFilterReq,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<SummaryResp>> {
        let mut query = Self::package_item_query(false, filter, funs, cxt).await?;
        query.inner_join(
            Alias::new(Self::get_ext_table_name()),
            Expr::tbl(Alias::new(Self::get_ext_table_name()), ID_FIELD.clone()).equals(rbum_item::Entity, rbum_item::Column::Id),
        );
        Self::package_ext_query(&mut query, false, filter, funs, cxt).await?;
        if let Some(sort) = desc_sort_by_create {
            query.order_by((rbum_item::Entity, CREATE_TIME_FIELD.clone()), if sort { Order::Desc } else { Order::Asc });
        }
        if let Some(sort) = desc_sort_by_update {
            query.order_by((rbum_item::Entity, UPDATE_TIME_FIELD.clone()), if sort { Order::Desc } else { Order::Asc });
        }
        let (records, total_size) = funs.db().paginate_dtos(&query, page_number, page_size).await?;
        Ok(TardisPage {
            page_size,
            page_number,
            total_size,
            records,
        })
    }

    async fn find_items(
        filter: &ItemFilterReq,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<Vec<SummaryResp>> {
        let mut query = Self::package_item_query(false, filter, funs, cxt).await?;
        query.inner_join(
            Alias::new(Self::get_ext_table_name()),
            Expr::tbl(Alias::new(Self::get_ext_table_name()), ID_FIELD.clone()).equals(rbum_item::Entity, rbum_item::Column::Id),
        );
        Self::package_ext_query(&mut query, false, filter, funs, cxt).await?;
        if let Some(sort) = desc_sort_by_create {
            query.order_by((rbum_item::Entity, CREATE_TIME_FIELD.clone()), if sort { Order::Desc } else { Order::Asc });
        }
        if let Some(sort) = desc_sort_by_update {
            query.order_by((rbum_item::Entity, UPDATE_TIME_FIELD.clone()), if sort { Order::Desc } else { Order::Asc });
        }
        Ok(funs.db().find_dtos(&query).await?)
    }

    async fn count_items(filter: &ItemFilterReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        let mut query = Self::package_item_query(false, filter, funs, cxt).await?;
        query.inner_join(
            Alias::new(Self::get_ext_table_name()),
            Expr::tbl(Alias::new(Self::get_ext_table_name()), ID_FIELD.clone()).equals(rbum_item::Entity, rbum_item::Column::Id),
        );
        Self::package_ext_query(&mut query, false, filter, funs, cxt).await?;
        funs.db().count(&query).await
    }
}

#[async_trait]
impl<'a> RbumCrudOperation<'a, rbum_item_attr::ActiveModel, RbumItemAttrAddReq, RbumItemAttrModifyReq, RbumItemAttrSummaryResp, RbumItemAttrDetailResp, RbumItemAttrFilterReq>
    for RbumItemAttrServ
{
    fn get_table_name() -> &'static str {
        rbum_item_attr::Entity.table_name()
    }

    async fn package_add(add_req: &RbumItemAttrAddReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<rbum_item_attr::ActiveModel> {
        Ok(rbum_item_attr::ActiveModel {
            id: Set(TardisFuns::field.nanoid()),
            value: Set(add_req.value.to_string()),
            rel_rbum_item_id: Set(add_req.rel_rbum_item_id.to_string()),
            rel_rbum_kind_attr_id: Set(add_req.rel_rbum_kind_attr_id.to_string()),
            ..Default::default()
        })
    }

    async fn before_add_rbum(add_req: &mut RbumItemAttrAddReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Self::check_scope(&add_req.rel_rbum_item_id, RbumItemServ::get_table_name(), funs, cxt).await?;
        Self::check_scope(&add_req.rel_rbum_kind_attr_id, RbumKindAttrServ::get_table_name(), funs, cxt).await?;
        let rbum_kind_attr = RbumKindAttrServ::peek_rbum(&add_req.rel_rbum_kind_attr_id, &RbumKindAttrFilterReq::default(), funs, cxt).await?;
        if rbum_kind_attr.main_column {
            return Err(TardisError::BadRequest(
                "Extension fields located in main table cannot be added using this function".to_string(),
            ));
        }
        if rbum_kind_attr.idx {
            return Err(TardisError::BadRequest("Indexed extension fields cannot be added using this function".to_string()));
        }
        Ok(())
    }

    async fn package_modify(id: &str, modify_req: &RbumItemAttrModifyReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<rbum_item_attr::ActiveModel> {
        Ok(rbum_item_attr::ActiveModel {
            id: Set(id.to_string()),
            value: Set(modify_req.value.to_string()),
            ..Default::default()
        })
    }

    async fn package_query(is_detail: bool, filter: &RbumItemAttrFilterReq, _: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<SelectStatement> {
        let mut query = Query::select();
        query
            .columns(vec![
                (rbum_item_attr::Entity, rbum_item_attr::Column::Id),
                (rbum_item_attr::Entity, rbum_item_attr::Column::Value),
                (rbum_item_attr::Entity, rbum_item_attr::Column::RelRbumItemId),
                (rbum_item_attr::Entity, rbum_item_attr::Column::RelRbumKindAttrId),
                (rbum_item_attr::Entity, rbum_item_attr::Column::OwnPaths),
                (rbum_item_attr::Entity, rbum_item_attr::Column::Owner),
                (rbum_item_attr::Entity, rbum_item_attr::Column::CreateTime),
                (rbum_item_attr::Entity, rbum_item_attr::Column::UpdateTime),
            ])
            .expr_as(Expr::tbl(rbum_item::Entity, rbum_item::Column::Name), Alias::new("rel_rbum_item_name"))
            .expr_as(Expr::tbl(rbum_kind_attr::Entity, rbum_kind_attr::Column::Name), Alias::new("rel_rbum_kind_attr_name"))
            .from(rbum_item_attr::Entity)
            .inner_join(
                rbum_item::Entity,
                Expr::tbl(rbum_item::Entity, rbum_item::Column::Id).equals(rbum_item_attr::Entity, rbum_item_attr::Column::RelRbumItemId),
            )
            .inner_join(
                rbum_kind_attr::Entity,
                Expr::tbl(rbum_kind_attr::Entity, rbum_kind_attr::Column::Id).equals(rbum_item_attr::Entity, rbum_item_attr::Column::RelRbumKindAttrId),
            );
        if let Some(rel_rbum_item_id) = &filter.rel_rbum_item_id {
            query.and_where(Expr::tbl(rbum_item_attr::Entity, rbum_item_attr::Column::RelRbumItemId).eq(rel_rbum_item_id.to_string()));
        }
        if let Some(rel_rbum_kind_attr_id) = &filter.rel_rbum_kind_attr_id {
            query.and_where(Expr::tbl(rbum_item_attr::Entity, rbum_item_attr::Column::RelRbumKindAttrId).eq(rel_rbum_kind_attr_id.to_string()));
        }
        query.with_filter(Self::get_table_name(), &filter.basic, is_detail, false, cxt);
        Ok(query)
    }
}

impl<'a> RbumItemAttrServ {
    pub async fn find_item_attr_defs_by_item_id(rbum_item_id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<Vec<RbumKindAttrSummaryResp>> {
        let rel_rbum_kind_id = RbumItemServ::peek_rbum(rbum_item_id, &RbumBasicFilterReq::default(), funs, cxt).await?.rel_rbum_kind_id;
        RbumKindAttrServ::find_rbums(
            &RbumKindAttrFilterReq {
                basic: RbumBasicFilterReq {
                    rbum_kind_id: Some(rel_rbum_kind_id),
                    ..Default::default()
                },
            },
            None,
            None,
            funs,
            cxt,
        )
        .await
    }

    pub async fn add_or_modify_item_attrs(add_req: &RbumItemAttrsAddOrModifyReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        // Implicit rel_rbum_kind_attr scope check
        let rbum_kind_attrs = Self::find_item_attr_defs_by_item_id(&add_req.rel_rbum_item_id, funs, cxt).await?;
        let in_main_table_attrs = rbum_kind_attrs.iter().filter(|i| add_req.values.contains_key(&i.name) && i.main_column).collect::<Vec<&RbumKindAttrSummaryResp>>();
        let in_ext_table_attrs = rbum_kind_attrs.iter().filter(|i| add_req.values.contains_key(&i.name) && !i.main_column).collect::<Vec<&RbumKindAttrSummaryResp>>();

        if !in_main_table_attrs.is_empty() {
            // Implicit rel_rbum_item scope check
            let rel_rbum_kind_id = RbumItemServ::peek_rbum(&add_req.rel_rbum_item_id, &RbumBasicFilterReq::default(), funs, cxt).await?.rel_rbum_kind_id;
            let main_table_name = RbumKindServ::peek_rbum(&rel_rbum_kind_id, &RbumBasicFilterReq::default(), funs, cxt).await?.ext_table_name;

            let mut update_statement = Query::update();
            update_statement.table(Alias::new(&main_table_name));

            for in_main_table_attr in in_main_table_attrs {
                let column_name = Alias::new(&in_main_table_attr.name);
                let column_val = add_req.values.get(&in_main_table_attr.name).unwrap().clone();
                update_statement.value(column_name, column_val.into());
            }
            update_statement.and_where(Expr::col(ID_FIELD.clone()).eq(add_req.rel_rbum_item_id.as_str()));
            funs.db().execute(&update_statement).await?;
        }

        if !in_ext_table_attrs.is_empty() {
            for in_ext_table_attr in in_ext_table_attrs {
                let column_val = add_req.values.get(&in_ext_table_attr.name).unwrap().clone();
                let exist_item_attr = Self::find_rbums(
                    &RbumItemAttrFilterReq {
                        basic: Default::default(),
                        rel_rbum_item_id: Some(add_req.rel_rbum_item_id.to_string()),
                        rel_rbum_kind_attr_id: Some(in_ext_table_attr.id.to_string()),
                    },
                    None,
                    None,
                    funs,
                    cxt,
                )
                .await?;
                if exist_item_attr.is_empty() {
                    Self::add_rbum(
                        &mut RbumItemAttrAddReq {
                            value: column_val,
                            rel_rbum_item_id: add_req.rel_rbum_item_id.to_string(),
                            rel_rbum_kind_attr_id: in_ext_table_attr.id.to_string(),
                        },
                        funs,
                        cxt,
                    )
                    .await?;
                } else {
                    Self::modify_rbum(&exist_item_attr.get(0).unwrap().id, &mut RbumItemAttrModifyReq { value: column_val }, funs, cxt).await?;
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, FromQueryResult)]
pub struct CodeResp {
    pub code: String,
}
