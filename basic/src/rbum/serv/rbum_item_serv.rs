use std::collections::HashMap;

use async_trait::async_trait;
use serde::Serialize;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::{IdResp, TardisActiveModel};
use tardis::db::sea_orm;
use tardis::db::sea_orm::sea_query::*;
use tardis::db::sea_orm::*;
use tardis::web::poem_openapi::types::{ParseFromJSON, ToJSON};
use tardis::web::web_resp::TardisPage;
use tardis::{TardisFuns, TardisFunsInst};

use crate::rbum::domain::{rbum_cert, rbum_cert_conf, rbum_domain, rbum_item, rbum_item_attr, rbum_kind, rbum_kind_attr, rbum_rel, rbum_set_item};
use crate::rbum::dto::rbum_filer_dto::{
    RbumBasicFilterReq, RbumCertConfFilterReq, RbumCertFilterReq, RbumItemAttrFilterReq, RbumItemFilterFetcher, RbumItemRelFilterReq, RbumKindAttrFilterReq, RbumSetItemFilterReq,
};
use crate::rbum::dto::rbum_item_attr_dto::{RbumItemAttrAddReq, RbumItemAttrDetailResp, RbumItemAttrModifyReq, RbumItemAttrSummaryResp, RbumItemAttrsAddOrModifyReq};
use crate::rbum::dto::rbum_item_dto::{RbumItemAddReq, RbumItemDetailResp, RbumItemKernelAddReq, RbumItemModifyReq, RbumItemSummaryResp};
use crate::rbum::dto::rbum_kind_attr_dto::RbumKindAttrSummaryResp;
use crate::rbum::dto::rbum_rel_dto::{RbumRelAddReq, RbumRelFindReq};
use crate::rbum::helper::rbum_event_helper;
use crate::rbum::rbum_config::RbumConfigApi;
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
impl RbumCrudOperation<rbum_item::ActiveModel, RbumItemAddReq, RbumItemModifyReq, RbumItemSummaryResp, RbumItemDetailResp, RbumBasicFilterReq> for RbumItemServ {
    fn get_table_name() -> &'static str {
        rbum_item::Entity.table_name()
    }

    async fn package_add(add_req: &RbumItemAddReq, funs: &TardisFunsInst, _: &TardisContext) -> TardisResult<rbum_item::ActiveModel> {
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
                return Err(funs.err().conflict(&Self::get_obj_name(), "add", &format!("code {} already exists", code), "409-rbum-*-code-exist"));
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

    async fn before_add_rbum(add_req: &mut RbumItemAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        Self::check_scope(&add_req.rel_rbum_kind_id, RbumKindServ::get_table_name(), funs, ctx).await?;
        Self::check_scope(&add_req.rel_rbum_domain_id, RbumDomainServ::get_table_name(), funs, ctx).await?;
        Ok(())
    }

    async fn package_modify(id: &str, modify_req: &RbumItemModifyReq, funs: &TardisFunsInst, _: &TardisContext) -> TardisResult<rbum_item::ActiveModel> {
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
                return Err(funs.err().conflict(&Self::get_obj_name(), "modify", &format!("code {} already exists", code), "409-rbum-*-code-exist"));
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

    async fn before_delete_rbum(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<RbumItemDetailResp>> {
        Self::check_ownership(id, funs, ctx).await?;
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
        Ok(None)
    }

    async fn package_query(is_detail: bool, filter: &RbumBasicFilterReq, _: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<SelectStatement> {
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
        query.with_filter(Self::get_table_name(), filter, is_detail, true, ctx);
        Ok(query)
    }
}

#[async_trait]
pub trait RbumItemCrudOperation<EXT, AddReq, ModifyReq, SummaryResp, DetailResp, ItemFilterReq>
where
    EXT: TardisActiveModel + Sync + Send,
    AddReq: Sync + Send,
    ModifyReq: Sync + Send,
    SummaryResp: FromQueryResult + ParseFromJSON + ToJSON + Serialize + Send + Sync,
    DetailResp: FromQueryResult + ParseFromJSON + ToJSON + Serialize + Send + Sync,
    ItemFilterReq: Sync + Send + RbumItemFilterFetcher,
{
    fn get_ext_table_name() -> &'static str;

    fn get_obj_name() -> String {
        Self::get_ext_table_name().to_string()
    }

    fn get_rbum_kind_id() -> String;

    fn get_rbum_domain_id() -> String;

    // ----------------------------- Add -------------------------------

    async fn package_item_add(add_req: &AddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<RbumItemKernelAddReq>;

    async fn package_ext_add(id: &str, add_req: &AddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<EXT>;

    async fn before_add_item(_: &mut AddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    async fn after_add_item(_: &str, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    async fn add_item(add_req: &mut AddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        Self::before_add_item(add_req, funs, ctx).await?;
        let item_add_req = Self::package_item_add(add_req, funs, ctx).await?;
        let mut item_add_req = RbumItemAddReq {
            id: item_add_req.id.clone(),
            code: item_add_req.code.clone(),
            name: item_add_req.name.clone(),
            rel_rbum_kind_id: Self::get_rbum_kind_id(),
            rel_rbum_domain_id: Self::get_rbum_domain_id(),
            scope_level: item_add_req.scope_level.clone(),
            disabled: item_add_req.disabled,
        };
        let id = RbumItemServ::add_rbum(&mut item_add_req, funs, ctx).await?;
        let ext_domain = Self::package_ext_add(&id, add_req, funs, ctx).await?;
        funs.db().insert_one(ext_domain, ctx).await?;
        Self::after_add_item(&id, funs, ctx).await?;
        rbum_event_helper::try_notify(Self::get_ext_table_name(), "c", &id, funs, ctx).await?;
        Ok(id)
    }

    async fn add_item_with_simple_rel_by_from(add_req: &mut AddReq, tag: &str, to_rbum_item_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let id = Self::add_item(add_req, funs, ctx).await?;
        RbumRelServ::add_rbum(
            &mut RbumRelAddReq {
                tag: tag.to_string(),
                note: None,
                from_rbum_kind: RbumRelFromKind::Item,
                from_rbum_id: id.to_string(),
                to_rbum_item_id: to_rbum_item_id.to_string(),
                to_own_paths: ctx.own_paths.to_string(),
                to_is_outside: false,
                ext: None,
            },
            funs,
            ctx,
        )
        .await?;
        Ok(id)
    }

    async fn add_item_with_simple_rel_by_to(add_req: &mut AddReq, tag: &str, from_rbum_item_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let id = Self::add_item(add_req, funs, ctx).await?;
        RbumRelServ::add_rbum(
            &mut RbumRelAddReq {
                tag: tag.to_string(),
                note: None,
                from_rbum_kind: RbumRelFromKind::Item,
                from_rbum_id: from_rbum_item_id.to_string(),
                to_rbum_item_id: id.to_string(),
                to_own_paths: ctx.own_paths.to_string(),
                to_is_outside: false,
                ext: None,
            },
            funs,
            ctx,
        )
        .await?;
        Ok(id)
    }

    // ----------------------------- Modify -------------------------------

    async fn package_item_modify(id: &str, modify_req: &ModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<RbumItemModifyReq>>;

    async fn package_ext_modify(id: &str, modify_req: &ModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<EXT>>;

    async fn before_modify_item(_: &str, _: &mut ModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    async fn after_modify_item(_: &str, _: &mut ModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    async fn modify_item(id: &str, modify_req: &mut ModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        Self::before_modify_item(id, modify_req, funs, ctx).await?;
        let item_modify_req = Self::package_item_modify(id, modify_req, funs, ctx).await?;
        if let Some(mut item_modify_req) = item_modify_req {
            RbumItemServ::modify_rbum(id, &mut item_modify_req, funs, ctx).await?;
        } else {
            RbumItemServ::check_ownership(id, funs, ctx).await?;
        }
        let ext_domain = Self::package_ext_modify(id, modify_req, funs, ctx).await?;
        if let Some(ext_domain) = ext_domain {
            funs.db().update_one(ext_domain, ctx).await?;
        }
        Self::after_modify_item(id, modify_req, funs, ctx).await?;
        rbum_event_helper::try_notify(Self::get_ext_table_name(), "u", id, funs, ctx).await?;
        Ok(())
    }

    // ----------------------------- Delete -------------------------------

    async fn package_delete(id: &str, _funs: &TardisFunsInst, _ctx: &TardisContext) -> TardisResult<Select<EXT::Entity>> {
        Ok(EXT::Entity::find().filter(Expr::col(ID_FIELD.clone()).eq(id)))
    }

    async fn before_delete_item(_: &str, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<Option<DetailResp>> {
        Ok(None)
    }

    async fn after_delete_item(_: &str, _: &Option<DetailResp>, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    async fn delete_item(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<u64> {
        let deleted_item = Self::before_delete_item(id, funs, ctx).await?;
        let select = Self::package_delete(id, funs, ctx).await?;
        #[cfg(feature = "with-mq")]
        {
            let delete_records = funs.db().soft_delete_custom(select, "id").await?;
            RbumItemServ::delete_rbum(id, funs, ctx).await?;
            let mq_topic_entity_deleted = &funs.rbum_conf_mq_topic_entity_deleted();
            let mq_header = std::collections::HashMap::from([(funs.rbum_conf_mq_header_name_operator(), ctx.owner.clone())]);
            for delete_record in &delete_records {
                funs.mq().request(mq_topic_entity_deleted, TardisFuns::json.obj_to_string(delete_record)?, &mq_header).await?;
            }
            Self::after_delete_item(id, &deleted_item, funs, ctx).await?;
            rbum_event_helper::try_notify(Self::get_ext_table_name(), "d", id, funs, ctx).await?;
            Ok(delete_records.len() as u64)
        }
        #[cfg(not(feature = "with-mq"))]
        {
            let delete_records = funs.db().soft_delete(select, &ctx.owner).await?;
            RbumItemServ::delete_rbum(id, funs, ctx).await?;
            Self::after_delete_item(id, &deleted_item, funs, ctx).await?;
            rbum_event_helper::try_notify(Self::get_ext_table_name(), "d", &id, funs, ctx).await?;
            Ok(delete_records)
        }
    }

    async fn delete_item_with_all_rels(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<u64> {
        // Delete rels
        let rel_ids = RbumRelServ::find_rel_ids(
            &RbumRelFindReq {
                tag: None,
                from_rbum_kind: Some(RbumRelFromKind::Item),
                from_rbum_id: Some(id.to_string()),
                to_rbum_item_id: None,
            },
            funs,
            ctx,
        )
        .await?;
        for rel_id in rel_ids {
            RbumRelServ::delete_rel_with_ext(&rel_id, funs, ctx).await?;
        }
        let rel_ids = RbumRelServ::find_rel_ids(
            &RbumRelFindReq {
                tag: None,
                from_rbum_kind: Some(RbumRelFromKind::Item),
                from_rbum_id: None,
                to_rbum_item_id: Some(id.to_string()),
            },
            funs,
            ctx,
        )
        .await?;
        for rel_id in rel_ids {
            RbumRelServ::delete_rel_with_ext(&rel_id, funs, ctx).await?;
        }

        // Delete set items
        let set_item_ids = RbumSetItemServ::find_id_rbums(
            &RbumSetItemFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                rel_rbum_item_ids: Some(vec![id.to_string()]),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        for set_item_id in set_item_ids {
            RbumSetItemServ::delete_rbum(&set_item_id, funs, ctx).await?;
        }

        // Delete Certs
        let cert_ids = RbumCertServ::find_id_rbums(
            &RbumCertFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                rel_rbum_kind: Some(RbumCertRelKind::Item),
                rel_rbum_id: Some(id.to_string()),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        for cert_id in cert_ids {
            RbumCertServ::delete_rbum(&cert_id, funs, ctx).await?;
        }

        // Delete Cert Conf
        let cert_conf_ids = RbumCertConfServ::find_id_rbums(
            &RbumCertConfFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                rel_rbum_item_id: Some(id.to_string()),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        for cert_conf_id in cert_conf_ids {
            RbumCertConfServ::delete_rbum(&cert_conf_id, funs, ctx).await?;
        }

        Self::delete_item(id, funs, ctx).await
    }

    // ----------------------------- Query -------------------------------

    async fn package_item_query(is_detail: bool, filter: &ItemFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<SelectStatement> {
        let mut query = RbumItemServ::package_query(
            is_detail,
            &RbumBasicFilterReq {
                ignore_scope: filter.basic().ignore_scope,
                rel_ctx_owner: filter.basic().rel_ctx_owner,
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
            ctx,
        )
        .await?;
        fn package_rel(query: &mut SelectStatement, rel_table: Alias, rbum_item_rel_filter_req: &RbumItemRelFilterReq) {
            if rbum_item_rel_filter_req.rel_by_from {
                if rbum_item_rel_filter_req.is_left {
                    query.join_as(
                        JoinType::LeftJoin,
                        rbum_rel::Entity,
                        rel_table.clone(),
                        Expr::tbl(rel_table.clone(), rbum_rel::Column::FromRbumId).equals(rbum_item::Entity, rbum_item::Column::Id),
                    );
                } else {
                    query.join_as(
                        JoinType::InnerJoin,
                        rbum_rel::Entity,
                        rel_table.clone(),
                        Expr::tbl(rel_table.clone(), rbum_rel::Column::FromRbumId).equals(rbum_item::Entity, rbum_item::Column::Id),
                    );
                }
                if let Some(rel_item_id) = &rbum_item_rel_filter_req.rel_item_id {
                    query.and_where(Expr::tbl(rel_table.clone(), rbum_rel::Column::ToRbumItemId).eq(rel_item_id.to_string()));
                }
            } else {
                if rbum_item_rel_filter_req.is_left {
                    query.join_as(
                        JoinType::LeftJoin,
                        rbum_rel::Entity,
                        rel_table.clone(),
                        Expr::tbl(rel_table.clone(), rbum_rel::Column::ToRbumItemId).equals(rbum_item::Entity, rbum_item::Column::Id),
                    );
                } else {
                    query.join_as(
                        JoinType::InnerJoin,
                        rbum_rel::Entity,
                        rel_table.clone(),
                        Expr::tbl(rel_table.clone(), rbum_rel::Column::ToRbumItemId).equals(rbum_item::Entity, rbum_item::Column::Id),
                    );
                }
                if let Some(rel_item_id) = &rbum_item_rel_filter_req.rel_item_id {
                    query.and_where(Expr::tbl(rel_table.clone(), rbum_rel::Column::FromRbumId).eq(rel_item_id.to_string()));
                }
            }
            if let Some(tag) = &rbum_item_rel_filter_req.tag {
                query.and_where(Expr::tbl(rel_table.clone(), rbum_rel::Column::Tag).eq(tag.to_string()));
            }
            if let Some(from_rbum_kind) = &rbum_item_rel_filter_req.from_rbum_kind {
                query.and_where(Expr::tbl(rel_table.clone(), rbum_rel::Column::FromRbumKind).eq(from_rbum_kind.to_int()));
            }
            if let Some(ext_eq) = &rbum_item_rel_filter_req.ext_eq {
                query.and_where(Expr::tbl(rel_table.clone(), rbum_rel::Column::Ext).eq(ext_eq.to_string()));
            }
            if let Some(ext_like) = &rbum_item_rel_filter_req.ext_like {
                query.and_where(Expr::tbl(rel_table, rbum_rel::Column::Ext).like(format!("%{}%", ext_like).as_str()));
            }
        }
        if let Some(rbum_item_rel_filter_req) = &filter.rel() {
            package_rel(&mut query, Alias::new("rbum_rel1"), rbum_item_rel_filter_req);
        }
        if let Some(rbum_item_rel_filter_req) = &filter.rel2() {
            package_rel(&mut query, Alias::new("rbum_rel2"), rbum_item_rel_filter_req);
        }
        Ok(query)
    }

    async fn package_ext_query(query: &mut SelectStatement, is_detail: bool, filter: &ItemFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()>;

    async fn peek_item(id: &str, filter: &ItemFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<SummaryResp> {
        Self::do_peek_item(id, filter, funs, ctx).await
    }

    async fn do_peek_item(id: &str, filter: &ItemFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<SummaryResp> {
        let mut query = Self::package_item_query(false, filter, funs, ctx).await?;
        query.inner_join(
            Alias::new(Self::get_ext_table_name()),
            Expr::tbl(Alias::new(Self::get_ext_table_name()), ID_FIELD.clone()).equals(rbum_item::Entity, rbum_item::Column::Id),
        );
        Self::package_ext_query(&mut query, false, filter, funs, ctx).await?;
        query.and_where(Expr::tbl(rbum_item::Entity, rbum_item::Column::Id).eq(id));
        let query = funs.db().get_dto(&query).await?;
        match query {
            Some(resp) => Ok(resp),
            None => Err(funs.err().not_found(
                &Self::get_obj_name(),
                "peek",
                &format!("not found {}.{} by {}", Self::get_obj_name(), id, ctx.owner),
                "找不到指定的数据",
            )),
        }
    }

    async fn get_item(id: &str, filter: &ItemFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<DetailResp> {
        Self::do_get_item(id, filter, funs, ctx).await
    }

    async fn do_get_item(id: &str, filter: &ItemFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<DetailResp> {
        let mut query = Self::package_item_query(true, filter, funs, ctx).await?;
        query.inner_join(
            Alias::new(Self::get_ext_table_name()),
            Expr::tbl(Alias::new(Self::get_ext_table_name()), ID_FIELD.clone()).equals(rbum_item::Entity, rbum_item::Column::Id),
        );
        Self::package_ext_query(&mut query, true, filter, funs, ctx).await?;
        query.and_where(Expr::tbl(rbum_item::Entity, rbum_item::Column::Id).eq(id));
        let query = funs.db().get_dto(&query).await?;
        match query {
            Some(resp) => Ok(resp),
            None => Err(funs.err().not_found(
                &Self::get_obj_name(),
                "get",
                &format!("not found {}.{} by {}", Self::get_obj_name(), id, ctx.owner),
                "找不到指定的数据",
            )),
        }
    }

    async fn paginate_id_items(
        filter: &ItemFilterReq,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<String>> {
        Self::do_paginate_id_items(filter, page_number, page_size, desc_sort_by_create, desc_sort_by_update, funs, ctx).await
    }

    async fn do_paginate_id_items(
        filter: &ItemFilterReq,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<String>> {
        let mut query = Self::package_item_query(false, filter, funs, ctx).await?;
        query.inner_join(
            Alias::new(Self::get_ext_table_name()),
            Expr::tbl(Alias::new(Self::get_ext_table_name()), ID_FIELD.clone()).equals(rbum_item::Entity, rbum_item::Column::Id),
        );
        Self::package_ext_query(&mut query, false, filter, funs, ctx).await?;
        if let Some(sort) = desc_sort_by_create {
            query.order_by((rbum_item::Entity, CREATE_TIME_FIELD.clone()), if sort { Order::Desc } else { Order::Asc });
        }
        if let Some(sort) = desc_sort_by_update {
            query.order_by((rbum_item::Entity, UPDATE_TIME_FIELD.clone()), if sort { Order::Desc } else { Order::Asc });
        }
        let (records, total_size) = funs.db().paginate_dtos::<IdResp>(&query, page_number, page_size).await?;
        Ok(TardisPage {
            page_size,
            page_number,
            total_size,
            records: records.into_iter().map(|resp| resp.id).collect(),
        })
    }

    async fn paginate_items(
        filter: &ItemFilterReq,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<SummaryResp>> {
        Self::do_paginate_items(filter, page_number, page_size, desc_sort_by_create, desc_sort_by_update, funs, ctx).await
    }

    async fn do_paginate_items(
        filter: &ItemFilterReq,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<SummaryResp>> {
        let mut query = Self::package_item_query(false, filter, funs, ctx).await?;
        query.inner_join(
            Alias::new(Self::get_ext_table_name()),
            Expr::tbl(Alias::new(Self::get_ext_table_name()), ID_FIELD.clone()).equals(rbum_item::Entity, rbum_item::Column::Id),
        );
        Self::package_ext_query(&mut query, false, filter, funs, ctx).await?;
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

    async fn paginate_detail_items(
        filter: &ItemFilterReq,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<DetailResp>> {
        Self::do_paginate_detail_items(filter, page_number, page_size, desc_sort_by_create, desc_sort_by_update, funs, ctx).await
    }

    async fn do_paginate_detail_items(
        filter: &ItemFilterReq,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<DetailResp>> {
        let mut query = Self::package_item_query(true, filter, funs, ctx).await?;
        query.inner_join(
            Alias::new(Self::get_ext_table_name()),
            Expr::tbl(Alias::new(Self::get_ext_table_name()), ID_FIELD.clone()).equals(rbum_item::Entity, rbum_item::Column::Id),
        );
        Self::package_ext_query(&mut query, true, filter, funs, ctx).await?;
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

    async fn find_one_item(filter: &ItemFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<SummaryResp>> {
        Self::do_find_one_item(filter, funs, ctx).await
    }

    async fn do_find_one_item(filter: &ItemFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<SummaryResp>> {
        let result = Self::find_items(filter, None, None, funs, ctx).await?;
        if result.len() > 1 {
            Err(funs.err().conflict(&Self::get_obj_name(), "find_one", "found multiple records", "409-rbum-*-obj-multi-exist"))
        } else {
            Ok(result.into_iter().next())
        }
    }

    async fn find_id_items(
        filter: &ItemFilterReq,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<String>> {
        Self::do_find_id_items(filter, desc_sort_by_create, desc_sort_by_update, funs, ctx).await
    }

    async fn do_find_id_items(
        filter: &ItemFilterReq,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<String>> {
        let mut query = Self::package_item_query(false, filter, funs, ctx).await?;
        query.inner_join(
            Alias::new(Self::get_ext_table_name()),
            Expr::tbl(Alias::new(Self::get_ext_table_name()), ID_FIELD.clone()).equals(rbum_item::Entity, rbum_item::Column::Id),
        );
        Self::package_ext_query(&mut query, false, filter, funs, ctx).await?;
        if let Some(sort) = desc_sort_by_create {
            query.order_by((rbum_item::Entity, CREATE_TIME_FIELD.clone()), if sort { Order::Desc } else { Order::Asc });
        }
        if let Some(sort) = desc_sort_by_update {
            query.order_by((rbum_item::Entity, UPDATE_TIME_FIELD.clone()), if sort { Order::Desc } else { Order::Asc });
        }
        Ok(funs.db().find_dtos::<IdResp>(&query).await?.into_iter().map(|resp| resp.id).collect())
    }

    async fn find_items(
        filter: &ItemFilterReq,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<SummaryResp>> {
        Self::do_find_items(filter, desc_sort_by_create, desc_sort_by_update, funs, ctx).await
    }

    async fn do_find_items(
        filter: &ItemFilterReq,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<SummaryResp>> {
        let mut query = Self::package_item_query(false, filter, funs, ctx).await?;
        query.inner_join(
            Alias::new(Self::get_ext_table_name()),
            Expr::tbl(Alias::new(Self::get_ext_table_name()), ID_FIELD.clone()).equals(rbum_item::Entity, rbum_item::Column::Id),
        );
        Self::package_ext_query(&mut query, false, filter, funs, ctx).await?;
        if let Some(sort) = desc_sort_by_create {
            query.order_by((rbum_item::Entity, CREATE_TIME_FIELD.clone()), if sort { Order::Desc } else { Order::Asc });
        }
        if let Some(sort) = desc_sort_by_update {
            query.order_by((rbum_item::Entity, UPDATE_TIME_FIELD.clone()), if sort { Order::Desc } else { Order::Asc });
        }
        Ok(funs.db().find_dtos(&query).await?)
    }

    async fn find_one_detail_item(filter: &ItemFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<DetailResp>> {
        Self::do_find_one_detail_item(filter, funs, ctx).await
    }

    async fn do_find_one_detail_item(filter: &ItemFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<DetailResp>> {
        let result = Self::find_detail_items(filter, None, None, funs, ctx).await?;
        if result.len() > 1 {
            Err(funs.err().conflict(&Self::get_obj_name(), "find_one_detail", "found multiple records", "409-rbum-*-obj-multi-exist"))
        } else {
            Ok(result.into_iter().next())
        }
    }

    async fn find_detail_items(
        filter: &ItemFilterReq,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<DetailResp>> {
        Self::do_find_detail_items(filter, desc_sort_by_create, desc_sort_by_update, funs, ctx).await
    }

    async fn do_find_detail_items(
        filter: &ItemFilterReq,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<DetailResp>> {
        let mut query = Self::package_item_query(true, filter, funs, ctx).await?;
        query.inner_join(
            Alias::new(Self::get_ext_table_name()),
            Expr::tbl(Alias::new(Self::get_ext_table_name()), ID_FIELD.clone()).equals(rbum_item::Entity, rbum_item::Column::Id),
        );
        Self::package_ext_query(&mut query, true, filter, funs, ctx).await?;
        if let Some(sort) = desc_sort_by_create {
            query.order_by((rbum_item::Entity, CREATE_TIME_FIELD.clone()), if sort { Order::Desc } else { Order::Asc });
        }
        if let Some(sort) = desc_sort_by_update {
            query.order_by((rbum_item::Entity, UPDATE_TIME_FIELD.clone()), if sort { Order::Desc } else { Order::Asc });
        }
        Ok(funs.db().find_dtos(&query).await?)
    }

    async fn count_items(filter: &ItemFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<u64> {
        Self::do_count_items(filter, funs, ctx).await
    }

    async fn do_count_items(filter: &ItemFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<u64> {
        let mut query = Self::package_item_query(false, filter, funs, ctx).await?;
        query.inner_join(
            Alias::new(Self::get_ext_table_name()),
            Expr::tbl(Alias::new(Self::get_ext_table_name()), ID_FIELD.clone()).equals(rbum_item::Entity, rbum_item::Column::Id),
        );
        Self::package_ext_query(&mut query, false, filter, funs, ctx).await?;
        funs.db().count(&query).await
    }

    async fn is_disabled(id: &str, funs: &TardisFunsInst) -> TardisResult<bool> {
        #[derive(Debug, sea_orm::FromQueryResult)]
        pub struct StatusResp {
            pub disabled: bool,
        }
        let result =
            funs.db().get_dto::<StatusResp>(Query::select().column(rbum_item::Column::Disabled).from(rbum_item::Entity).and_where(Expr::col(rbum_item::Column::Id).eq(id))).await?;
        if let Some(result) = result {
            Ok(result.disabled)
        } else {
            Err(funs.err().not_found(
                &Self::get_obj_name(),
                "is_disabled",
                &format!("not found {}.{}", Self::get_obj_name(), id),
                "404-rbum-*-obj-not-exist",
            ))
        }
    }
}

#[async_trait]
impl RbumCrudOperation<rbum_item_attr::ActiveModel, RbumItemAttrAddReq, RbumItemAttrModifyReq, RbumItemAttrSummaryResp, RbumItemAttrDetailResp, RbumItemAttrFilterReq>
    for RbumItemAttrServ
{
    fn get_table_name() -> &'static str {
        rbum_item_attr::Entity.table_name()
    }

    async fn package_add(add_req: &RbumItemAttrAddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<rbum_item_attr::ActiveModel> {
        Ok(rbum_item_attr::ActiveModel {
            id: Set(TardisFuns::field.nanoid()),
            value: Set(add_req.value.to_string()),
            rel_rbum_item_id: Set(add_req.rel_rbum_item_id.to_string()),
            rel_rbum_kind_attr_id: Set(add_req.rel_rbum_kind_attr_id.to_string()),
            ..Default::default()
        })
    }

    async fn before_add_rbum(add_req: &mut RbumItemAttrAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        Self::check_scope(&add_req.rel_rbum_item_id, RbumItemServ::get_table_name(), funs, ctx).await?;
        Self::check_scope(&add_req.rel_rbum_kind_attr_id, RbumKindAttrServ::get_table_name(), funs, ctx).await?;
        let rbum_kind_attr = RbumKindAttrServ::peek_rbum(&add_req.rel_rbum_kind_attr_id, &RbumKindAttrFilterReq::default(), funs, ctx).await?;
        if rbum_kind_attr.main_column {
            return Err(funs.err().bad_request(
                &Self::get_obj_name(),
                "add",
                "extension fields located in main table cannot be added using this function",
                "400-rbum-kind-attr-main-illegal",
            ));
        }
        if rbum_kind_attr.idx {
            return Err(funs.err().bad_request(
                &Self::get_obj_name(),
                "add",
                "index extension fields cannot be added using this function",
                "400-rbum-kind-attr-idx-illegal",
            ));
        }
        Ok(())
    }

    async fn package_modify(id: &str, modify_req: &RbumItemAttrModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<rbum_item_attr::ActiveModel> {
        Ok(rbum_item_attr::ActiveModel {
            id: Set(id.to_string()),
            value: Set(modify_req.value.to_string()),
            ..Default::default()
        })
    }

    async fn package_query(is_detail: bool, filter: &RbumItemAttrFilterReq, _: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<SelectStatement> {
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
        query.with_filter(Self::get_table_name(), &filter.basic, is_detail, false, ctx);
        Ok(query)
    }
}

impl RbumItemAttrServ {
    pub async fn find_item_attr_defs_by_item_id(rbum_item_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<RbumKindAttrSummaryResp>> {
        let rel_rbum_kind_id = RbumItemServ::peek_rbum(
            rbum_item_id,
            &RbumBasicFilterReq {
                with_sub_own_paths: true,
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
        .rel_rbum_kind_id;
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
            ctx,
        )
        .await
    }

    pub async fn add_or_modify_item_attrs(add_req: &RbumItemAttrsAddOrModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        // Implicit rel_rbum_kind_attr scope check
        let rbum_kind_attrs = Self::find_item_attr_defs_by_item_id(&add_req.rel_rbum_item_id, funs, ctx).await?;
        let in_main_table_attrs = rbum_kind_attrs.iter().filter(|i| add_req.values.contains_key(&i.name) && i.main_column).collect::<Vec<&RbumKindAttrSummaryResp>>();
        let in_ext_table_attrs = rbum_kind_attrs.iter().filter(|i| add_req.values.contains_key(&i.name) && !i.main_column).collect::<Vec<&RbumKindAttrSummaryResp>>();

        if !in_main_table_attrs.is_empty() {
            // Implicit rel_rbum_item scope check
            let rel_rbum_kind_id = RbumItemServ::peek_rbum(&add_req.rel_rbum_item_id, &RbumBasicFilterReq::default(), funs, ctx).await?.rel_rbum_kind_id;
            let main_table_name = RbumKindServ::peek_rbum(&rel_rbum_kind_id, &RbumBasicFilterReq::default(), funs, ctx).await?.ext_table_name;

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
                let exist_item_attr_ids = Self::find_id_rbums(
                    &RbumItemAttrFilterReq {
                        basic: Default::default(),
                        rel_rbum_item_id: Some(add_req.rel_rbum_item_id.to_string()),
                        rel_rbum_kind_attr_id: Some(in_ext_table_attr.id.to_string()),
                    },
                    None,
                    None,
                    funs,
                    ctx,
                )
                .await?;
                if exist_item_attr_ids.is_empty() {
                    Self::add_rbum(
                        &mut RbumItemAttrAddReq {
                            value: column_val,
                            rel_rbum_item_id: add_req.rel_rbum_item_id.to_string(),
                            rel_rbum_kind_attr_id: in_ext_table_attr.id.to_string(),
                        },
                        funs,
                        ctx,
                    )
                    .await?;
                } else {
                    Self::modify_rbum(exist_item_attr_ids.get(0).unwrap(), &mut RbumItemAttrModifyReq { value: column_val }, funs, ctx).await?;
                }
            }
        }

        Ok(())
    }

    pub async fn find_item_attr_values(rbum_item_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<HashMap<String, String>> {
        let rbum_kind_attrs = Self::find_item_attr_defs_by_item_id(rbum_item_id, funs, ctx).await?;
        let in_main_table_attrs = rbum_kind_attrs.iter().filter(|i| i.main_column).collect::<Vec<&RbumKindAttrSummaryResp>>();
        let has_in_ext_table_attrs = rbum_kind_attrs.iter().any(|i| !i.main_column);

        let mut values: HashMap<String, String> = HashMap::new();
        if !in_main_table_attrs.is_empty() {
            let rel_rbum_kind_id = RbumItemServ::peek_rbum(rbum_item_id, &RbumBasicFilterReq::default(), funs, ctx).await?.rel_rbum_kind_id;
            let ext_table_name = RbumKindServ::peek_rbum(&rel_rbum_kind_id, &RbumBasicFilterReq::default(), funs, ctx).await?.ext_table_name;

            let mut select_statement = Query::select();
            select_statement.from(Alias::new(&ext_table_name));
            for in_main_table_attr in &in_main_table_attrs {
                let column_name = Alias::new(&in_main_table_attr.name);
                select_statement.column(column_name);
            }
            select_statement.and_where(Expr::col(ID_FIELD.clone()).eq(rbum_item_id));
            let select_statement = funs.db().raw_conn().get_database_backend().build(&select_statement);
            if let Some(row) = funs.db().raw_conn().query_one(select_statement).await? {
                for in_main_table_attr in &in_main_table_attrs {
                    let value: String = row.try_get("", &in_main_table_attr.name)?;
                    values.insert(in_main_table_attr.name.clone(), value);
                }
            }
        }

        if has_in_ext_table_attrs {
            let attr_values = Self::find_rbums(
                &RbumItemAttrFilterReq {
                    rel_rbum_item_id: Some(rbum_item_id.to_string()),
                    ..Default::default()
                },
                None,
                None,
                funs,
                ctx,
            )
            .await?;
            for attr_value in attr_values {
                values.insert(attr_value.rel_rbum_kind_attr_name, attr_value.value);
            }
        }
        Ok(values)
    }
}

#[derive(Debug, sea_orm::FromQueryResult)]
pub struct CodeResp {
    pub code: String,
}
