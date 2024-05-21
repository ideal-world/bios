use std::collections::HashMap;

use async_trait::async_trait;
use serde::Serialize;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::{IdResp, TardisActiveModel};
use tardis::db::sea_orm::sea_query::*;
use tardis::db::sea_orm::*;
use tardis::db::sea_orm::{self, IdenStatic};
use tardis::web::poem_openapi::types::{ParseFromJSON, ToJSON};
use tardis::web::web_resp::TardisPage;
use tardis::{TardisFuns, TardisFunsInst};

use super::rbum_crud_serv::{IdNameResp, CREATE_TIME_FIELD, ID_FIELD, UPDATE_TIME_FIELD};
use crate::rbum::domain::{rbum_cert, rbum_cert_conf, rbum_domain, rbum_item, rbum_item_attr, rbum_kind, rbum_kind_attr, rbum_rel, rbum_set_item};
use crate::rbum::dto::rbum_filer_dto::{
    RbumBasicFilterReq, RbumCertConfFilterReq, RbumCertFilterReq, RbumItemAttrFilterReq, RbumItemFilterFetcher, RbumItemRelFilterReq, RbumKindAttrFilterReq, RbumKindFilterReq,
    RbumSetItemFilterReq, RbumSetItemRelFilterReq,
};
use crate::rbum::dto::rbum_item_attr_dto::{RbumItemAttrAddReq, RbumItemAttrDetailResp, RbumItemAttrModifyReq, RbumItemAttrSummaryResp, RbumItemAttrsAddOrModifyReq};
use crate::rbum::dto::rbum_item_dto::{RbumItemAddReq, RbumItemDetailResp, RbumItemKernelAddReq, RbumItemKernelModifyReq, RbumItemSummaryResp};
use crate::rbum::dto::rbum_kind_attr_dto::RbumKindAttrSummaryResp;
use crate::rbum::dto::rbum_rel_dto::{RbumRelAddReq, RbumRelSimpleFindReq};
use crate::rbum::helper::rbum_event_helper;
#[cfg(feature = "with-mq")]
use crate::rbum::rbum_config::RbumConfigApi;
use crate::rbum::rbum_enumeration::{RbumCertRelKind, RbumRelFromKind, RbumScopeLevelKind};
use crate::rbum::serv::rbum_cert_serv::{RbumCertConfServ, RbumCertServ};
#[cfg(feature = "with-mq")]
use crate::rbum::serv::rbum_crud_serv::ID_FIELD_NAME;
use crate::rbum::serv::rbum_crud_serv::{RbumCrudOperation, RbumCrudQueryPackage};
use crate::rbum::serv::rbum_domain_serv::RbumDomainServ;
use crate::rbum::serv::rbum_kind_serv::{RbumKindAttrServ, RbumKindServ};
use crate::rbum::serv::rbum_rel_serv::RbumRelServ;
use crate::rbum::serv::rbum_set_serv::RbumSetItemServ;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref RBUM_ITEM_TABLE: Alias = Alias::new("rbum_item");
}

pub struct RbumItemServ;

pub struct RbumItemAttrServ;

#[async_trait]
impl RbumCrudOperation<rbum_item::ActiveModel, RbumItemAddReq, RbumItemKernelModifyReq, RbumItemSummaryResp, RbumItemDetailResp, RbumBasicFilterReq> for RbumItemServ {
    fn get_table_name() -> &'static str {
        rbum_item::Entity.table_name()
    }

    async fn before_add_rbum(add_req: &mut RbumItemAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        Self::check_scope(&add_req.rel_rbum_kind_id, RbumKindServ::get_table_name(), funs, ctx).await?;
        Self::check_scope(&add_req.rel_rbum_domain_id, RbumDomainServ::get_table_name(), funs, ctx).await?;
        Ok(())
    }

    async fn package_add(add_req: &RbumItemAddReq, funs: &TardisFunsInst, _: &TardisContext) -> TardisResult<rbum_item::ActiveModel> {
        let id = if let Some(id) = &add_req.id { id.to_string() } else { TardisFuns::field.nanoid() };
        let code = if let Some(code) = &add_req.code {
            if funs
                .db()
                .count(
                    Query::select()
                        .column((rbum_item::Entity, rbum_item::Column::Id))
                        .from(rbum_item::Entity)
                        .inner_join(
                            rbum_domain::Entity,
                            Expr::col((rbum_domain::Entity, rbum_domain::Column::Id)).equals((rbum_item::Entity, rbum_item::Column::RelRbumDomainId)),
                        )
                        .inner_join(
                            rbum_kind::Entity,
                            Expr::col((rbum_kind::Entity, rbum_kind::Column::Id)).equals((rbum_item::Entity, rbum_item::Column::RelRbumKindId)),
                        )
                        .and_where(Expr::col((rbum_item::Entity, rbum_item::Column::Code)).eq(code.to_string())),
                )
                .await?
                > 0
            {
                return Err(funs.err().conflict(&Self::get_obj_name(), "add", &format!("code {code} already exists"), "409-rbum-*-code-exist"));
            }
            code.to_string()
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

    async fn package_modify(id: &str, modify_req: &RbumItemKernelModifyReq, funs: &TardisFunsInst, _: &TardisContext) -> TardisResult<rbum_item::ActiveModel> {
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
                            Expr::col((rbum_domain::Entity, rbum_domain::Column::Id)).equals((rbum_item::Entity, rbum_item::Column::RelRbumDomainId)),
                        )
                        .inner_join(
                            rbum_kind::Entity,
                            Expr::col((rbum_kind::Entity, rbum_kind::Column::Id)).equals((rbum_item::Entity, rbum_item::Column::RelRbumKindId)),
                        )
                        .and_where(Expr::col((rbum_item::Entity, rbum_item::Column::Code)).eq(code.to_string()))
                        .and_where(Expr::col((rbum_item::Entity, rbum_item::Column::Id)).ne(id)),
                )
                .await?
                > 0
            {
                return Err(funs.err().conflict(&Self::get_obj_name(), "modify", &format!("code {code} already exists"), "409-rbum-*-code-exist"));
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
            any![
                all![
                    Expr::col(rbum_rel::Column::FromRbumKind).eq(RbumRelFromKind::Item.to_int()),
                    Expr::col(rbum_rel::Column::FromRbumId).eq(id)
                ],
                Expr::col(rbum_rel::Column::ToRbumItemId).eq(id)
            ],
            funs,
        )
        .await?;
        Self::check_exist_before_delete(id, RbumSetItemServ::get_table_name(), rbum_set_item::Column::RelRbumItemId.as_str(), funs).await?;
        Self::check_exist_before_delete(id, RbumCertConfServ::get_table_name(), rbum_cert_conf::Column::RelRbumItemId.as_str(), funs).await?;
        Self::check_exist_with_cond_before_delete(
            RbumCertServ::get_table_name(),
            all![
                Expr::col(rbum_cert::Column::RelRbumKind).eq(RbumCertRelKind::Item.to_int()),
                Expr::col(rbum_cert::Column::RelRbumId).eq(id)
            ],
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
                .expr_as(Expr::col((rbum_kind::Entity, rbum_kind::Column::Name)), Alias::new("rel_rbum_kind_name"))
                .expr_as(Expr::col((rbum_domain::Entity, rbum_domain::Column::Name)), Alias::new("rel_rbum_domain_name"))
                .inner_join(
                    rbum_kind::Entity,
                    Expr::col((rbum_kind::Entity, rbum_kind::Column::Id)).equals((rbum_item::Entity, rbum_item::Column::RelRbumKindId)),
                )
                .inner_join(
                    rbum_domain::Entity,
                    Expr::col((rbum_domain::Entity, rbum_domain::Column::Id)).equals((rbum_item::Entity, rbum_item::Column::RelRbumDomainId)),
                );
        }
        query.with_filter(Self::get_table_name(), filter, is_detail, true, ctx);
        Ok(query)
    }
}

/// Resource item extended common operation
///
/// 资源项扩展公共操作
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
    /// Get the name of the extended table
    ///
    /// 获取扩展表的名称
    fn get_ext_table_name() -> &'static str;

    /// Get the name of the extended object
    ///
    /// 获取扩展对象的名称
    ///
    /// Mostly used for printing log identifiers.
    ///
    /// 多用于打印日志的标识。
    fn get_obj_name() -> String {
        Self::get_ext_table_name().to_string()
    }

    /// Get default resource kind
    ///
    /// 获取默认的资源类型
    fn get_rbum_kind_id() -> Option<String>;

    /// Get default resource domain
    ///
    /// 获取默认的资源域
    fn get_rbum_domain_id() -> Option<String>;

    // ----------------------------- Add -------------------------------

    /// Pre-processing of the add request
    ///
    /// 添加请求的前置处理
    async fn before_add_item(_: &mut AddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    /// Package add request of the kernel part of the resource item
    ///
    /// 组装资源项核心部分的添加请求
    ///
    /// # Examples:
    ///
    /// ```
    /// async fn package_item_add(add_req: &IamAppAddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<RbumItemKernelAddReq> {
    ///     Ok(RbumItemKernelAddReq {
    ///         id: add_req.id.clone(),
    ///         name: add_req.name.clone(),
    ///         disabled: add_req.disabled,
    ///         scope_level: add_req.scope_level.clone(),
    ///         ..Default::default()
    ///     })
    /// }
    /// ```
    async fn package_item_add(add_req: &AddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<RbumItemKernelAddReq>;

    /// Package add request of the extended part of the resource item
    ///
    /// 组装资源项扩展部分的添加请求
    ///
    /// # Examples:
    ///
    /// ```
    /// async fn package_ext_add(id: &str, add_req: &IamAppAddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<iam_app::ActiveModel> {
    ///     Ok(iam_app::ActiveModel {
    ///         id: Set(id.to_string()),
    ///         icon: Set(add_req.icon.as_ref().unwrap_or(&"".to_string()).to_string()),
    ///         sort: Set(add_req.sort.unwrap_or(0)),
    ///         contact_phone: Set(add_req.contact_phone.as_ref().unwrap_or(&"".to_string()).to_string()),
    ///         ..Default::default()
    ///     })
    /// }
    /// ```
    async fn package_ext_add(id: &str, add_req: &AddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<EXT>;

    /// Post-processing of the add request
    ///
    /// 添加请求的后置处理
    async fn after_add_item(_: &str, _: &mut AddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    /// Add resource item
    ///
    /// 添加资源项
    async fn add_item(add_req: &mut AddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        Self::before_add_item(add_req, funs, ctx).await?;
        let add_kernel_req = Self::package_item_add(add_req, funs, ctx).await?;
        let mut item_add_req = RbumItemAddReq {
            id: add_kernel_req.id.clone(),
            code: add_kernel_req.code.clone(),
            name: add_kernel_req.name.clone(),
            rel_rbum_kind_id: if let Some(rel_rbum_kind_id) = &add_kernel_req.rel_rbum_kind_id {
                rel_rbum_kind_id.to_string()
            } else {
                Self::get_rbum_kind_id().ok_or_else(|| funs.err().bad_request(&Self::get_obj_name(), "add_item", "kind is required", "400-rbum-kind-require"))?
            },
            rel_rbum_domain_id: if let Some(rel_rbum_domain_id) = &add_kernel_req.rel_rbum_domain_id {
                rel_rbum_domain_id.to_string()
            } else {
                Self::get_rbum_domain_id().ok_or_else(|| funs.err().bad_request(&Self::get_obj_name(), "add_item", "domain is required", "400-rbum-domain-require"))?
            },
            scope_level: add_kernel_req.scope_level.clone(),
            disabled: add_kernel_req.disabled,
        };
        let id = RbumItemServ::add_rbum(&mut item_add_req, funs, ctx).await?;
        let add_ext_req = Self::package_ext_add(&id, add_req, funs, ctx).await?;
        funs.db().insert_one(add_ext_req, ctx).await?;
        Self::after_add_item(&id, add_req, funs, ctx).await?;
        rbum_event_helper::add_notify_event(Self::get_ext_table_name(), "c", id.as_str(), ctx).await?;
        Ok(id)
    }

    /// Add resource item with simple relationship (the added resource item is the source party)
    ///
    /// 添加资源项及其简单关系（添加的资源项为来源方）
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

    /// Add resource item with simple relationship (the added resource item is the target party)
    ///
    /// 添加资源项及其简单关系（添加的资源项为目标方）
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

    ///  Pre-processing of the modify request
    ///
    /// 修改请求的前置处理
    async fn before_modify_item(_: &str, _: &mut ModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    /// Package modify request of the kernel part of the resource item
    ///
    /// 组装资源项核心部分的修改请求
    ///
    /// # Examples:
    ///
    /// ```
    /// async fn package_item_modify(_: &str, modify_req: &IamAppModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<Option<RbumItemKernelModifyReq>> {
    ///     if modify_req.name.is_none() && modify_req.scope_level.is_none() && modify_req.disabled.is_none() {
    ///         return Ok(None);
    ///     }
    ///     Ok(Some(RbumItemKernelModifyReq {
    ///         code: None,
    ///         name: modify_req.name.clone(),
    ///         scope_level: modify_req.scope_level.clone(),
    ///         disabled: modify_req.disabled,
    ///     }))
    /// }
    /// ```
    async fn package_item_modify(id: &str, modify_req: &ModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<RbumItemKernelModifyReq>>;

    /// Package modify request of the extended part of the resource item
    ///
    /// 组装资源项扩展部分的修改请求
    ///
    /// # Examples:
    ///
    /// ```
    /// async fn package_ext_modify(id: &str, modify_req: &IamAppModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<Option<iam_app::ActiveModel>> {
    ///     if modify_req.icon.is_none() && modify_req.sort.is_none() && modify_req.contact_phone.is_none() {
    ///         return Ok(None);
    ///     }
    ///     let mut iam_app = iam_app::ActiveModel {
    ///         id: Set(id.to_string()),
    ///         ..Default::default()
    ///     };
    ///     if let Some(icon) = &modify_req.icon {
    ///         iam_app.icon = Set(icon.to_string());
    ///     }
    ///     if let Some(sort) = modify_req.sort {
    ///         iam_app.sort = Set(sort);
    ///     }
    ///     if let Some(contact_phone) = &modify_req.contact_phone {
    ///         iam_app.contact_phone = Set(contact_phone.to_string());
    ///     }
    ///     Ok(Some(iam_app))
    /// }
    /// ```
    async fn package_ext_modify(id: &str, modify_req: &ModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<EXT>>;

    /// Post-processing of the modify request
    ///
    /// 修改请求的后置处理
    async fn after_modify_item(_: &str, _: &mut ModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    /// Modify resource item
    ///
    /// 修改资源项
    async fn modify_item(id: &str, modify_req: &mut ModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        Self::before_modify_item(id, modify_req, funs, ctx).await?;
        let modify_kernel_req = Self::package_item_modify(id, modify_req, funs, ctx).await?;
        if let Some(mut item_modify_req) = modify_kernel_req {
            RbumItemServ::modify_rbum(id, &mut item_modify_req, funs, ctx).await?;
        } else {
            RbumItemServ::check_ownership(id, funs, ctx).await?;
        }
        let modify_ext_req = Self::package_ext_modify(id, modify_req, funs, ctx).await?;
        if let Some(ext_domain) = modify_ext_req {
            funs.db().update_one(ext_domain, ctx).await?;
        }
        Self::after_modify_item(id, modify_req, funs, ctx).await?;
        rbum_event_helper::add_notify_event(Self::get_ext_table_name(), "u", id, ctx).await?;
        Ok(())
    }

    // ----------------------------- Delete -------------------------------

    /// Pre-processing of the delete request
    ///
    /// 删除请求的前置处理
    async fn before_delete_item(_: &str, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<Option<DetailResp>> {
        Ok(None)
    }

    /// Post-processing of the delete request
    ///
    /// 删除请求的后置处理
    async fn after_delete_item(_: &str, _: &Option<DetailResp>, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    /// Delete resource item
    ///
    /// 删除资源项
    ///
    /// TODO remove mq and send detail data to event.
    async fn delete_item(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<u64> {
        let deleted_item = Self::before_delete_item(id, funs, ctx).await?;
        let item_select_req = <EXT::Entity as EntityTrait>::find().filter(Expr::col(ID_FIELD.clone()).eq(id));
        #[cfg(feature = "with-mq")]
        {
            let deleted_ext_records = funs.db().soft_delete_custom(item_select_req, ID_FIELD_NAME).await?;
            RbumItemServ::delete_rbum(id, funs, ctx).await?;
            let mq_topic_entity_deleted = &funs.rbum_conf_mq_topic_entity_deleted();
            let mq_header = std::collections::HashMap::from([(funs.rbum_conf_mq_header_name_operator(), ctx.owner.clone())]);
            for delete_record in &deleted_ext_records {
                funs.mq().publish(mq_topic_entity_deleted, TardisFuns::json.obj_to_string(delete_record)?, &mq_header).await?;
            }
            Self::after_delete_item(id, &deleted_item, funs, ctx).await?;
            rbum_event_helper::add_notify_event(Self::get_ext_table_name(), "d", id, ctx).await?;
            Ok(deleted_ext_records.len() as u64)
        }
        #[cfg(not(feature = "with-mq"))]
        {
            let deleted_ext_records = funs.db().soft_delete(item_select_req, &ctx.owner).await?;
            RbumItemServ::delete_rbum(id, funs, ctx).await?;
            Self::after_delete_item(id, &deleted_item, funs, ctx).await?;
            rbum_event_helper::add_notify_event(Self::get_ext_table_name(), "d", id, ctx).await?;
            Ok(deleted_ext_records)
        }
    }

    /// Delete resource item with all relationships
    ///
    /// 删除资源项及其所有关系
    async fn delete_item_with_all_rels(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<u64> {
        // Delete rels
        let rel_ids = RbumRelServ::find_rel_ids(
            &RbumRelSimpleFindReq {
                tag: None,
                from_rbum_kind: Some(RbumRelFromKind::Item),
                from_rbum_id: Some(id.to_string()),
                to_rbum_item_id: None,
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        for rel_id in rel_ids {
            RbumRelServ::delete_rel_with_ext(&rel_id, funs, ctx).await?;
        }
        let rel_ids = RbumRelServ::find_rel_ids(
            &RbumRelSimpleFindReq {
                tag: None,
                from_rbum_kind: None,
                from_rbum_id: None,
                to_rbum_item_id: Some(id.to_string()),
                ..Default::default()
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

    /// Package query request of the kernel part of the resource item
    ///
    /// 组装资源项核心部分的查询请求
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
                names: filter.basic().names.clone(),
                code: filter.basic().code.clone(),
                codes: filter.basic().codes.clone(),
                rbum_kind_id: if filter.basic().rbum_kind_id.is_some() {
                    filter.basic().rbum_kind_id.clone()
                } else {
                    Self::get_rbum_kind_id()
                },
                rbum_domain_id: if filter.basic().rbum_domain_id.is_some() {
                    filter.basic().rbum_domain_id.clone()
                } else {
                    Self::get_rbum_domain_id()
                },
            },
            funs,
            ctx,
        )
        .await?;

        if let Some(rbum_item_rel_filter_req) = &filter.rel() {
            Self::package_rel(&mut query, Alias::new("rbum_rel1"), rbum_item_rel_filter_req);
        }
        if let Some(rbum_item_rel_filter_req) = &filter.rel2() {
            Self::package_rel(&mut query, Alias::new("rbum_rel2"), rbum_item_rel_filter_req);
        }
        Ok(query)
    }

    /// Package condition of the relationship
    ///
    /// 组装关联关系的查询条件
    fn package_rel(query: &mut SelectStatement, rel_table: Alias, rbum_item_rel_filter_req: &RbumItemRelFilterReq) {
        let mut binding = Query::select();
        let sub_query = binding.from(rbum_rel::Entity);
        if let Some(tag) = &rbum_item_rel_filter_req.tag {
            sub_query.and_where(Expr::col((rbum_rel::Entity, rbum_rel::Column::Tag)).eq(tag.to_string()));
        }
        if let Some(from_rbum_kind) = &rbum_item_rel_filter_req.from_rbum_kind {
            sub_query.and_where(Expr::col((rbum_rel::Entity, rbum_rel::Column::FromRbumKind)).eq(from_rbum_kind.to_int()));
        }
        if let Some(ext_eq) = &rbum_item_rel_filter_req.ext_eq {
            sub_query.and_where(Expr::col((rbum_rel::Entity, rbum_rel::Column::Ext)).eq(ext_eq.to_string()));
        }
        if let Some(ext_like) = &rbum_item_rel_filter_req.ext_like {
            sub_query.and_where(Expr::col((rbum_rel::Entity, rbum_rel::Column::Ext)).like(format!("%{ext_like}%").as_str()));
        }
        if let Some(own_paths) = &rbum_item_rel_filter_req.own_paths {
            sub_query.and_where(Expr::col((rbum_rel::Entity, rbum_rel::Column::OwnPaths)).eq(own_paths.to_string()));
        }
        if rbum_item_rel_filter_req.rel_by_from {
            if let Some(rel_item_id) = &rbum_item_rel_filter_req.rel_item_id {
                sub_query.and_where(Expr::col((rbum_rel::Entity, rbum_rel::Column::ToRbumItemId)).eq(rel_item_id.to_string()));
            }
            if let Some(rel_item_ids) = &rbum_item_rel_filter_req.rel_item_ids {
                if rel_item_ids.len() == 1 {
                    sub_query.and_where(Expr::col((rbum_rel::Entity, rbum_rel::Column::ToRbumItemId)).eq(rel_item_ids.first().expect("ignore").to_string()));
                } else if !rel_item_ids.is_empty() {
                    sub_query.and_where(Expr::col((rbum_rel::Entity, rbum_rel::Column::ToRbumItemId)).is_in(rel_item_ids));
                }
            }
            sub_query.column((rbum_rel::Entity, rbum_rel::Column::FromRbumId));
            sub_query.group_by_col((rbum_rel::Entity, rbum_rel::Column::FromRbumId));

            if rbum_item_rel_filter_req.optional {
                query.join_subquery(
                    JoinType::LeftJoin,
                    sub_query.take(),
                    rel_table,
                    Expr::col((rbum_rel::Entity, rbum_rel::Column::FromRbumId)).equals((rbum_item::Entity, rbum_item::Column::Id)),
                );
            } else {
                query.join_subquery(
                    JoinType::InnerJoin,
                    sub_query.take(),
                    rel_table.clone(),
                    Expr::col((rel_table, rbum_rel::Column::FromRbumId)).equals((rbum_item::Entity, rbum_item::Column::Id)),
                );
            }
        } else {
            if let Some(rel_item_id) = &rbum_item_rel_filter_req.rel_item_id {
                sub_query.and_where(Expr::col((rbum_rel::Entity, rbum_rel::Column::FromRbumId)).eq(rel_item_id.to_string()));
            }
            if let Some(rel_item_ids) = &rbum_item_rel_filter_req.rel_item_ids {
                if rel_item_ids.len() == 1 {
                    sub_query.and_where(Expr::col((rbum_rel::Entity, rbum_rel::Column::FromRbumId)).eq(rel_item_ids.first().expect("ignore").to_string()));
                } else if !rel_item_ids.is_empty() {
                    sub_query.and_where(Expr::col((rbum_rel::Entity, rbum_rel::Column::FromRbumId)).is_in(rel_item_ids));
                }
            }
            sub_query.column((rbum_rel::Entity, rbum_rel::Column::ToRbumItemId));
            sub_query.group_by_col((rbum_rel::Entity, rbum_rel::Column::ToRbumItemId));

            if rbum_item_rel_filter_req.optional {
                query.join_subquery(
                    JoinType::LeftJoin,
                    sub_query.take(),
                    rel_table.clone(),
                    Expr::col((rel_table, rbum_rel::Column::ToRbumItemId)).equals((rbum_item::Entity, rbum_item::Column::Id)),
                );
            } else {
                query.join_subquery(
                    JoinType::InnerJoin,
                    sub_query.take(),
                    rel_table.clone(),
                    Expr::col((rel_table, rbum_rel::Column::ToRbumItemId)).equals((rbum_item::Entity, rbum_item::Column::Id)),
                );
            }
        }
    }

    /// Package condition of the resource set
    ///
    /// 组装资源集的查询条件
    fn package_set_rel(query: &mut SelectStatement, rel_table: Alias, rbum_set_rel_filter_req: &RbumSetItemRelFilterReq) {
        let mut binding = Query::select();
        let sub_query = binding.from(rbum_set_item::Entity);
        if let Some(set_ids_and_cate_codes) = rbum_set_rel_filter_req.set_ids_and_cate_codes.clone() {
            let mut cond_by_sets = Condition::any();
            for set_id in set_ids_and_cate_codes.keys() {
                let mut cond_by_a_set = Condition::all();
                cond_by_a_set = cond_by_a_set.add(Expr::col((rbum_set_item::Entity, rbum_set_item::Column::RelRbumSetId)).eq(set_id));
                if rbum_set_rel_filter_req.with_sub_set_cate_codes {
                    if let Some(cate_codes) = set_ids_and_cate_codes.get(set_id) {
                        let mut cond = Condition::any();
                        for cate_code in cate_codes {
                            cond = cond.add(Expr::col((rbum_set_item::Entity, rbum_set_item::Column::RelRbumSetCateCode)).like(format!("{}%", cate_code)));
                        }
                        cond_by_a_set = cond_by_a_set.add(Cond::all().add(cond));
                    } else {
                        // 包含所有子集
                    }
                } else {
                    cond_by_a_set = cond_by_a_set.add(
                        Expr::col((rbum_set_item::Entity, rbum_set_item::Column::RelRbumSetCateCode)).is_in(set_ids_and_cate_codes.get(set_id).unwrap_or(&Vec::<String>::new())),
                    );
                }
                cond_by_sets = cond_by_sets.add(cond_by_a_set);
            }
            sub_query.cond_where(cond_by_sets);
        }
        sub_query.column((rbum_set_item::Entity, rbum_set_item::Column::RelRbumItemId));
        sub_query.group_by_col((rbum_set_item::Entity, rbum_set_item::Column::RelRbumItemId));

        query.join_subquery(
            JoinType::InnerJoin,
            sub_query.take(),
            rel_table.clone(),
            Expr::col((rel_table, rbum_set_item::Column::RelRbumItemId)).equals((rbum_item::Entity, rbum_item::Column::Id)),
        );
    }

    /// Package query request of the extended part of the resource item
    ///
    /// 组装资源项扩展部分的查询请求
    ///
    /// # Examples:
    ///
    /// ```
    /// async fn package_ext_query(query: &mut SelectStatement, _: bool, filter: &IamAppFilterReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<()> {
    ///     query.column((iam_app::Entity, iam_app::Column::ContactPhone));
    ///     query.column((iam_app::Entity, iam_app::Column::Icon));
    ///     query.column((iam_app::Entity, iam_app::Column::Sort));
    ///     if let Some(contact_phone) = &filter.contact_phone {
    ///         query.and_where(Expr::col(iam_app::Column::ContactPhone).eq(contact_phone.as_str()));
    /// }
    /// Ok(())
    /// ```
    async fn package_ext_query(query: &mut SelectStatement, is_detail: bool, filter: &ItemFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()>;

    /// Query and get a resource item summary
    ///
    /// 查询并获取一条资源项概要信息
    async fn peek_item(id: &str, filter: &ItemFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<SummaryResp> {
        Self::do_peek_item(id, filter, funs, ctx).await
    }

    /// Query and get a resource item summary
    ///
    /// 查询并获取一条资源项概要信息
    ///
    /// NOTE: Internal method, not recommended to override.
    ///
    /// NOTE： 内部方法，不建议重写。
    async fn do_peek_item(id: &str, filter: &ItemFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<SummaryResp> {
        let mut query = Self::package_item_query(false, filter, funs, ctx).await?;
        query.inner_join(
            Alias::new(Self::get_ext_table_name()),
            Expr::col((Alias::new(Self::get_ext_table_name()), ID_FIELD.clone())).equals((rbum_item::Entity, rbum_item::Column::Id)),
        );
        Self::package_ext_query(&mut query, false, filter, funs, ctx).await?;
        query.and_where(Expr::col((rbum_item::Entity, rbum_item::Column::Id)).eq(id));
        let query = funs.db().get_dto(&query).await?;
        match query {
            Some(resp) => Ok(resp),
            None => Err(funs.err().not_found(
                &Self::get_obj_name(),
                "peek",
                &format!("not found {}.{} by {}", Self::get_obj_name(), id, ctx.owner),
                "404-rbum-*-obj-not-exist",
            )),
        }
    }

    /// Query and get a resource item detail
    ///
    /// 查询并获取一条资源项详细信息
    async fn get_item(id: &str, filter: &ItemFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<DetailResp> {
        Self::do_get_item(id, filter, funs, ctx).await
    }

    /// Query and get a resource item detail
    ///
    /// 查询并获取一条资源项详细信息
    ///
    /// NOTE: Internal method, not recommended to override.
    ///
    /// NOTE： 内部方法，不建议重写。
    async fn do_get_item(id: &str, filter: &ItemFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<DetailResp> {
        let mut query = Self::package_item_query(true, filter, funs, ctx).await?;
        query.inner_join(
            Alias::new(Self::get_ext_table_name()),
            Expr::col((Alias::new(Self::get_ext_table_name()), ID_FIELD.clone())).equals((rbum_item::Entity, rbum_item::Column::Id)),
        );
        Self::package_ext_query(&mut query, true, filter, funs, ctx).await?;
        query.and_where(Expr::col((rbum_item::Entity, rbum_item::Column::Id)).eq(id));
        let query = funs.db().get_dto(&query).await?;
        match query {
            Some(resp) => Ok(resp),
            None => Err(funs.err().not_found(
                &Self::get_obj_name(),
                "get",
                &format!("not found {}.{} by {}", Self::get_obj_name(), id, ctx.owner),
                "404-rbum-*-obj-not-exist",
            )),
        }
    }

    /// Query and page to get the resource item id set
    ///
    /// 查询并分页获取资源项id集合
    async fn paginate_id_items(
        filter: &ItemFilterReq,
        page_number: u32,
        page_size: u32,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<String>> {
        Self::do_paginate_id_items(filter, page_number, page_size, desc_sort_by_create, desc_sort_by_update, funs, ctx).await
    }

    /// Query and page to get the resource item id set
    ///
    /// 查询并分页获取资源项id集合
    ///
    /// NOTE: Internal method, not recommended to override.
    ///
    /// NOTE： 内部方法，不建议重写。
    async fn do_paginate_id_items(
        filter: &ItemFilterReq,
        page_number: u32,
        page_size: u32,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<String>> {
        let mut query = Self::package_item_query(false, filter, funs, ctx).await?;
        query.inner_join(
            Alias::new(Self::get_ext_table_name()),
            Expr::col((Alias::new(Self::get_ext_table_name()), ID_FIELD.clone())).equals((rbum_item::Entity, rbum_item::Column::Id)),
        );
        Self::package_ext_query(&mut query, false, filter, funs, ctx).await?;
        query.clear_selects();
        query.column((rbum_item::Entity, rbum_item::Column::Id));
        if let Some(sort) = desc_sort_by_create {
            query.order_by((rbum_item::Entity, CREATE_TIME_FIELD.clone()), if sort { Order::Desc } else { Order::Asc });
        }
        if let Some(sort) = desc_sort_by_update {
            query.order_by((rbum_item::Entity, UPDATE_TIME_FIELD.clone()), if sort { Order::Desc } else { Order::Asc });
        }
        let (records, total_size) = funs.db().paginate_dtos::<IdResp>(&query, page_number as u64, page_size as u64).await?;
        Ok(TardisPage {
            page_size: page_size as u64,
            page_number: page_number as u64,
            total_size,
            records: records.into_iter().map(|resp| resp.id).collect(),
        })
    }

    /// Query and page to get the resource item id and name set
    ///
    /// 查询并分页获取资源项id和名称集合
    async fn paginate_id_name_items(
        filter: &ItemFilterReq,
        page_number: u32,
        page_size: u32,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<IdNameResp>> {
        Self::do_paginate_id_name_items(filter, page_number, page_size, desc_sort_by_create, desc_sort_by_update, funs, ctx).await
    }

    /// Query and page to get the resource item id and name set
    ///
    /// 查询并分页获取资源项id和名称集合
    ///
    /// NOTE: Internal method, not recommended to override.
    ///
    /// NOTE： 内部方法，不建议重写。
    async fn do_paginate_id_name_items(
        filter: &ItemFilterReq,
        page_number: u32,
        page_size: u32,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<IdNameResp>> {
        let mut query = Self::package_item_query(false, filter, funs, ctx).await?;
        query.inner_join(
            Alias::new(Self::get_ext_table_name()),
            Expr::col((Alias::new(Self::get_ext_table_name()), ID_FIELD.clone())).equals((rbum_item::Entity, rbum_item::Column::Id)),
        );
        Self::package_ext_query(&mut query, false, filter, funs, ctx).await?;
        query.clear_selects();
        query.columns([(rbum_item::Entity, rbum_item::Column::Id), (rbum_item::Entity, rbum_item::Column::Name)]);
        if let Some(sort) = desc_sort_by_create {
            query.order_by((rbum_item::Entity, CREATE_TIME_FIELD.clone()), if sort { Order::Desc } else { Order::Asc });
        }
        if let Some(sort) = desc_sort_by_update {
            query.order_by((rbum_item::Entity, UPDATE_TIME_FIELD.clone()), if sort { Order::Desc } else { Order::Asc });
        }
        let (records, total_size) = funs.db().paginate_dtos::<IdNameResp>(&query, page_number as u64, page_size as u64).await?;
        Ok(TardisPage {
            page_size: page_size as u64,
            page_number: page_number as u64,
            total_size,
            records,
        })
    }

    /// Query and page to get the resource item summary set
    ///
    /// 查询并分页获取资源项概要信息集合
    async fn paginate_items(
        filter: &ItemFilterReq,
        page_number: u32,
        page_size: u32,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<SummaryResp>> {
        Self::do_paginate_items(filter, page_number, page_size, desc_sort_by_create, desc_sort_by_update, funs, ctx).await
    }

    /// Query and page to get the resource item summary set
    ///
    /// 查询并分页获取资源项概要信息集合
    ///
    /// NOTE: Internal method, not recommended to override.
    ///
    /// NOTE： 内部方法，不建议重写。
    async fn do_paginate_items(
        filter: &ItemFilterReq,
        page_number: u32,
        page_size: u32,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<SummaryResp>> {
        let mut query = Self::package_item_query(false, filter, funs, ctx).await?;
        query.inner_join(
            Alias::new(Self::get_ext_table_name()),
            Expr::col((Alias::new(Self::get_ext_table_name()), ID_FIELD.clone())).equals((rbum_item::Entity, rbum_item::Column::Id)),
        );
        Self::package_ext_query(&mut query, false, filter, funs, ctx).await?;
        if let Some(sort) = desc_sort_by_create {
            query.order_by((rbum_item::Entity, CREATE_TIME_FIELD.clone()), if sort { Order::Desc } else { Order::Asc });
        }
        if let Some(sort) = desc_sort_by_update {
            query.order_by((rbum_item::Entity, UPDATE_TIME_FIELD.clone()), if sort { Order::Desc } else { Order::Asc });
        }
        let (records, total_size) = funs.db().paginate_dtos(&query, page_number as u64, page_size as u64).await?;
        Ok(TardisPage {
            page_size: page_size as u64,
            page_number: page_number as u64,
            total_size,
            records,
        })
    }

    /// Query and page to get the resource item detail set
    ///
    /// 查询并分页获取资源项详细信息集合
    async fn paginate_detail_items(
        filter: &ItemFilterReq,
        page_number: u32,
        page_size: u32,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<DetailResp>> {
        Self::do_paginate_detail_items(filter, page_number, page_size, desc_sort_by_create, desc_sort_by_update, funs, ctx).await
    }

    /// Query and page to get the resource item detail set
    ///
    /// 查询并分页获取资源项详细信息集合
    ///
    /// NOTE: Internal method, not recommended to override.
    ///
    /// NOTE： 内部方法，不建议重写。
    async fn do_paginate_detail_items(
        filter: &ItemFilterReq,
        page_number: u32,
        page_size: u32,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<DetailResp>> {
        let mut query = Self::package_item_query(true, filter, funs, ctx).await?;
        query.inner_join(
            Alias::new(Self::get_ext_table_name()),
            Expr::col((Alias::new(Self::get_ext_table_name()), ID_FIELD.clone())).equals((rbum_item::Entity, rbum_item::Column::Id)),
        );
        Self::package_ext_query(&mut query, true, filter, funs, ctx).await?;
        if let Some(sort) = desc_sort_by_create {
            query.order_by((rbum_item::Entity, CREATE_TIME_FIELD.clone()), if sort { Order::Desc } else { Order::Asc });
        }
        if let Some(sort) = desc_sort_by_update {
            query.order_by((rbum_item::Entity, UPDATE_TIME_FIELD.clone()), if sort { Order::Desc } else { Order::Asc });
        }
        let (records, total_size) = funs.db().paginate_dtos(&query, page_number as u64, page_size as u64).await?;
        Ok(TardisPage {
            page_size: page_size as u64,
            page_number: page_number as u64,
            total_size,
            records,
        })
    }

    /// Query and get a resource item summary
    ///
    /// 查询并获取一条资源项概要信息
    async fn find_one_item(filter: &ItemFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<SummaryResp>> {
        Self::do_find_one_item(filter, funs, ctx).await
    }

    /// Query and get a resource item summary
    ///
    /// 查询并获取一条资源项概要信息
    ///
    /// NOTE: Internal method, not recommended to override.
    ///
    /// NOTE： 内部方法，不建议重写。
    async fn do_find_one_item(filter: &ItemFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<SummaryResp>> {
        let result = Self::find_items(filter, None, None, funs, ctx).await?;
        if result.len() > 1 {
            Err(funs.err().conflict(&Self::get_obj_name(), "find_one", "found multiple records", "409-rbum-*-obj-multi-exist"))
        } else {
            Ok(result.into_iter().next())
        }
    }

    /// Query and get the resource item id set
    ///
    /// 查询并获取资源项id集合
    async fn find_id_items(
        filter: &ItemFilterReq,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<String>> {
        Self::do_find_id_items(filter, desc_sort_by_create, desc_sort_by_update, funs, ctx).await
    }

    /// Query and get the resource item id set
    ///
    /// 查询并获取资源项id集合
    ///
    /// NOTE: Internal method, not recommended to override.
    ///
    /// NOTE： 内部方法，不建议重写。
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
            Expr::col((Alias::new(Self::get_ext_table_name()), ID_FIELD.clone())).equals((rbum_item::Entity, rbum_item::Column::Id)),
        );
        Self::package_ext_query(&mut query, false, filter, funs, ctx).await?;
        query.clear_selects();
        query.column((rbum_item::Entity, rbum_item::Column::Id));
        if let Some(sort) = desc_sort_by_create {
            query.order_by((rbum_item::Entity, CREATE_TIME_FIELD.clone()), if sort { Order::Desc } else { Order::Asc });
        }
        if let Some(sort) = desc_sort_by_update {
            query.order_by((rbum_item::Entity, UPDATE_TIME_FIELD.clone()), if sort { Order::Desc } else { Order::Asc });
        }
        Ok(funs.db().find_dtos::<IdResp>(&query).await?.into_iter().map(|resp| resp.id).collect())
    }

    /// Query and get the resource item id and name set
    ///
    /// 查询并获取资源项id和名称集合
    async fn find_id_name_items(
        filter: &ItemFilterReq,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<HashMap<String, String>> {
        Self::do_find_id_name_items(filter, desc_sort_by_create, desc_sort_by_update, funs, ctx).await
    }

    /// Query and get the resource item id and name set
    ///
    /// 查询并获取资源项id和名称集合
    ///
    /// NOTE: Internal method, not recommended to override.
    ///
    /// NOTE： 内部方法，不建议重写。
    async fn do_find_id_name_items(
        filter: &ItemFilterReq,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<HashMap<String, String>> {
        let mut query = Self::package_item_query(false, filter, funs, ctx).await?;
        query.inner_join(
            Alias::new(Self::get_ext_table_name()),
            Expr::col((Alias::new(Self::get_ext_table_name()), ID_FIELD.clone())).equals((rbum_item::Entity, rbum_item::Column::Id)),
        );
        Self::package_ext_query(&mut query, false, filter, funs, ctx).await?;
        query.clear_selects();
        query.columns([(rbum_item::Entity, rbum_item::Column::Id), (rbum_item::Entity, rbum_item::Column::Name)]);
        if let Some(sort) = desc_sort_by_create {
            query.order_by((rbum_item::Entity, CREATE_TIME_FIELD.clone()), if sort { Order::Desc } else { Order::Asc });
        }
        if let Some(sort) = desc_sort_by_update {
            query.order_by((rbum_item::Entity, UPDATE_TIME_FIELD.clone()), if sort { Order::Desc } else { Order::Asc });
        }
        Ok(funs.db().find_dtos::<IdNameResp>(&query).await?.into_iter().map(|resp| (resp.id, resp.name)).collect())
    }

    /// Query and get the resource item summary set
    ///
    /// 查询并获取资源项概要信息集合
    async fn find_items(
        filter: &ItemFilterReq,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<SummaryResp>> {
        Self::do_find_items(filter, desc_sort_by_create, desc_sort_by_update, funs, ctx).await
    }

    /// Query and get the resource item summary set
    ///
    /// 查询并获取资源项概要信息集合
    ///
    /// NOTE: Internal method, not recommended to override.
    ///
    /// NOTE： 内部方法，不建议重写。
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
            Expr::col((Alias::new(Self::get_ext_table_name()), ID_FIELD.clone())).equals((rbum_item::Entity, rbum_item::Column::Id)),
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

    /// Query and get a resource item detail
    ///
    /// 查询并获取一条资源项详细信息
    async fn find_one_detail_item(filter: &ItemFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<DetailResp>> {
        Self::do_find_one_detail_item(filter, funs, ctx).await
    }

    /// Query and get a resource item detail
    ///
    /// 查询并获取一条资源项详细信息
    ///
    /// NOTE: Internal method, not recommended to override.
    ///
    /// NOTE： 内部方法，不建议重写。
    async fn do_find_one_detail_item(filter: &ItemFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<DetailResp>> {
        let result = Self::find_detail_items(filter, None, None, funs, ctx).await?;
        if result.len() > 1 {
            Err(funs.err().conflict(&Self::get_obj_name(), "find_one_detail", "found multiple records", "409-rbum-*-obj-multi-exist"))
        } else {
            Ok(result.into_iter().next())
        }
    }

    /// Query and get the resource item detail set
    ///
    /// 查询并获取资源项详细信息集合
    async fn find_detail_items(
        filter: &ItemFilterReq,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<DetailResp>> {
        Self::do_find_detail_items(filter, desc_sort_by_create, desc_sort_by_update, funs, ctx).await
    }

    /// Query and get the resource item detail set
    ///
    /// 查询并获取资源项详细信息集合
    ///
    /// NOTE: Internal method, not recommended to override.
    ///
    /// NOTE： 内部方法，不建议重写。
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
            Expr::col((Alias::new(Self::get_ext_table_name()), ID_FIELD.clone())).equals((rbum_item::Entity, rbum_item::Column::Id)),
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

    /// Query and count the number of resource items
    ///
    /// 查询并统计资源项数量
    async fn count_items(filter: &ItemFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<u64> {
        Self::do_count_items(filter, funs, ctx).await
    }

    /// Query and count the number of resource items
    ///
    /// 查询并统计资源项数量
    ///
    /// NOTE: Internal method, not recommended to override.
    ///
    /// NOTE： 内部方法，不建议重写。
    async fn do_count_items(filter: &ItemFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<u64> {
        let mut query = Self::package_item_query(false, filter, funs, ctx).await?;
        query.inner_join(
            Alias::new(Self::get_ext_table_name()),
            Expr::col((Alias::new(Self::get_ext_table_name()), ID_FIELD.clone())).equals((rbum_item::Entity, rbum_item::Column::Id)),
        );
        Self::package_ext_query(&mut query, false, filter, funs, ctx).await?;
        funs.db().count(&query).await
    }

    /// Whether the resource item is disabled
    ///
    /// 判断资源项是否被禁用
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

    async fn package_add(add_req: &RbumItemAttrAddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<rbum_item_attr::ActiveModel> {
        Ok(rbum_item_attr::ActiveModel {
            id: Set(TardisFuns::field.nanoid()),
            value: Set(add_req.value.to_string()),
            rel_rbum_item_id: Set(add_req.rel_rbum_item_id.to_string()),
            rel_rbum_kind_attr_id: Set(add_req.rel_rbum_kind_attr_id.to_string()),
            ..Default::default()
        })
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
            .expr_as(Expr::col((rbum_kind_attr::Entity, rbum_kind_attr::Column::Name)), Alias::new("rel_rbum_kind_attr_name"))
            .from(rbum_item_attr::Entity)
            .inner_join(
                rbum_kind_attr::Entity,
                Expr::col((rbum_kind_attr::Entity, rbum_kind_attr::Column::Id)).equals((rbum_item_attr::Entity, rbum_item_attr::Column::RelRbumKindAttrId)),
            );
        if let Some(rel_rbum_item_id) = &filter.rel_rbum_item_id {
            query.and_where(Expr::col((rbum_item_attr::Entity, rbum_item_attr::Column::RelRbumItemId)).eq(rel_rbum_item_id.to_string()));
        }
        if let Some(rel_rbum_kind_attr_id) = &filter.rel_rbum_kind_attr_id {
            query.and_where(Expr::col((rbum_item_attr::Entity, rbum_item_attr::Column::RelRbumKindAttrId)).eq(rel_rbum_kind_attr_id.to_string()));
        }
        query.with_filter(Self::get_table_name(), &filter.basic, is_detail, false, ctx);
        Ok(query)
    }
}

impl RbumItemAttrServ {
    /// Get resource kind id and resource kind attribute definitions corresponding to resource item id
    ///
    /// 获取资源项对应资源类型id及资源类型所有属性定义
    async fn find_res_kind_id_and_res_kind_attrs_by_item_id(
        rbum_item_id: &str,
        secret: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<(String, Vec<RbumKindAttrSummaryResp>)> {
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
        let rbum_kind_attrs = RbumKindAttrServ::find_rbums(
            &RbumKindAttrFilterReq {
                basic: RbumBasicFilterReq {
                    rbum_kind_id: Some(rel_rbum_kind_id.clone()),
                    ..Default::default()
                },
                secret,
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        Ok((rel_rbum_kind_id, rbum_kind_attrs))
    }

    /// Add or modify resource item extended attributes
    ///
    /// 添加或修改资源项扩展属性
    pub async fn add_or_modify_item_attrs(add_req: &RbumItemAttrsAddOrModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        // Implicit rel_rbum_kind_attr scope check
        let (rel_rbum_kind_id, rbum_kind_attrs) = Self::find_res_kind_id_and_res_kind_attrs_by_item_id(&add_req.rel_rbum_item_id, None, funs, ctx).await?;
        let in_main_table_attrs = rbum_kind_attrs.iter().filter(|i| add_req.values.contains_key(&i.name) && i.main_column).collect::<Vec<&RbumKindAttrSummaryResp>>();
        let in_ext_table_attrs = rbum_kind_attrs.iter().filter(|i| add_req.values.contains_key(&i.name) && !i.main_column).collect::<Vec<&RbumKindAttrSummaryResp>>();
        if !in_main_table_attrs.is_empty() {
            // Implicit rel_rbum_item scope check
            let main_table_name = RbumKindServ::peek_rbum(&rel_rbum_kind_id, &RbumKindFilterReq::default(), funs, ctx).await?.ext_table_name;

            let mut update_statement = Query::update();
            update_statement.table(Alias::new(&main_table_name));

            for in_main_table_attr in in_main_table_attrs {
                let column_val = if in_main_table_attr.secret && !in_main_table_attr.dyn_default_value.is_empty() {
                    Self::replace_url_placeholder(&in_main_table_attr.dyn_default_value, &add_req.values, funs).await?
                } else if in_main_table_attr.secret {
                    in_main_table_attr.default_value.clone()
                } else {
                    add_req.values.get(&in_main_table_attr.name).expect("ignore").clone()
                };

                let column_name = Alias::new(&in_main_table_attr.name);
                update_statement.value(column_name, Value::from(column_val));
            }
            update_statement.and_where(Expr::col(ID_FIELD.clone()).eq(add_req.rel_rbum_item_id.as_str()));
            funs.db().execute(&update_statement).await?;
        }

        if !in_ext_table_attrs.is_empty() {
            for in_ext_table_attr in in_ext_table_attrs {
                let column_val = if in_ext_table_attr.secret && !in_ext_table_attr.dyn_default_value.is_empty() {
                    Self::replace_url_placeholder(&in_ext_table_attr.dyn_default_value, &add_req.values, funs).await?
                } else if in_ext_table_attr.secret {
                    in_ext_table_attr.default_value.clone()
                } else {
                    add_req.values.get(&in_ext_table_attr.name).expect("ignore").clone()
                };

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
                    Self::modify_rbum(exist_item_attr_ids.first().expect("ignore"), &mut RbumItemAttrModifyReq { value: column_val }, funs, ctx).await?;
                }
            }
        }

        Ok(())
    }

    /// Get resource item extended attributes
    ///
    /// 获取资源项扩展属性集合
    ///
    /// # Returns
    ///
    /// The key is the attribute name, and the value is the attribute value.
    pub async fn find_item_attr_values(rbum_item_id: &str, secret: Option<bool>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<HashMap<String, String>> {
        let (rel_rbum_kind_id, rbum_kind_attrs) = Self::find_res_kind_id_and_res_kind_attrs_by_item_id(rbum_item_id, secret, funs, ctx).await?;
        let in_main_table_attrs = rbum_kind_attrs.iter().filter(|i| i.main_column).collect::<Vec<&RbumKindAttrSummaryResp>>();
        let has_in_ext_table_attrs = rbum_kind_attrs.iter().any(|i| !i.main_column);

        let mut values: HashMap<String, String> = HashMap::new();
        if !in_main_table_attrs.is_empty() {
            let ext_table_name = RbumKindServ::peek_rbum(&rel_rbum_kind_id, &RbumKindFilterReq::default(), funs, ctx).await?.ext_table_name;

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

    async fn replace_url_placeholder(url: &str, values: &HashMap<String, String>, funs: &TardisFunsInst) -> TardisResult<String> {
        let resp = if RbumKindAttrServ::url_has_placeholder(url)? {
            let url: String = RbumKindAttrServ::url_replace(url, values)?;
            if RbumKindAttrServ::url_has_placeholder(&url)? {
                return Err(funs.err().bad_request(
                    &Self::get_obj_name(),
                    "replace_url_placeholder",
                    "url processing failure",
                    "400-rbum-kind-attr-dyn-url-illegal",
                ));
            }
            funs.web_client().get_to_str(&url, None).await
        } else {
            funs.web_client().get_to_str(url, None).await
        };
        match resp {
            Ok(resp) => Ok(resp.body.unwrap_or_else(|| "".to_string())),
            Err(e) => {
                return Err(funs.err().bad_request(
                    &Self::get_obj_name(),
                    "replace_url_placeholder",
                    &format!("url processing failure: {}", e),
                    "400-rbum-kind-attr-dyn-url-illegal",
                ));
            }
        }
    }
}

#[derive(Debug, sea_orm::FromQueryResult)]
pub struct CodeResp {
    pub code: String,
}
