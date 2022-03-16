use async_trait::async_trait;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisRelDBlConnection;
use tardis::db::sea_orm::*;
use tardis::db::sea_query::*;

use crate::rbum::domain::{rbum_domain, rbum_item, rbum_item_attr, rbum_kind, rbum_kind_attr};
use crate::rbum::dto::filer_dto::RbumBasicFilterReq;
use crate::rbum::dto::rbum_item_attr_dto::{RbumItemAttrAddReq, RbumItemAttrDetailResp, RbumItemAttrModifyReq, RbumItemAttrSummaryResp};
use crate::rbum::dto::rbum_item_dto::{RbumItemAddReq, RbumItemDetailResp, RbumItemModifyReq, RbumItemSummaryResp};
use crate::rbum::enumeration::RbumScopeKind;
use crate::rbum::serv::rbum_crud_serv::{RbumCrudOperation, RbumCrudQueryPackage};
use crate::rbum::serv::rbum_domain_serv::RbumDomainServ;
use crate::rbum::serv::rbum_kind_serv::{RbumKindAttrServ, RbumKindServ};

pub struct RbumItemServ;
pub struct RbumItemAttrServ;

#[async_trait]
impl<'a> RbumCrudOperation<'a, rbum_item::ActiveModel, RbumItemAddReq, RbumItemModifyReq, RbumItemSummaryResp, RbumItemDetailResp> for RbumItemServ {
    fn get_table_name() -> &'static str {
        rbum_item::Entity.table_name()
    }

    fn package_add(add_req: &RbumItemAddReq, _: &TardisContext) -> rbum_item::ActiveModel {
        rbum_item::ActiveModel {
            code: Set(add_req.code.to_string()),
            uri_path: Set(add_req.uri_path.to_string()),
            name: Set(add_req.name.to_string()),
            icon: Set(add_req.icon.as_ref().unwrap_or(&"".to_string()).to_string()),
            sort: Set(add_req.sort.unwrap_or(0)),
            rel_rbum_kind_id: Set(add_req.rel_rbum_kind_id.to_string()),
            rel_rbum_domain_id: Set(add_req.rel_rbum_domain_id.to_string()),
            scope_kind: Set(add_req.scope_kind.as_ref().unwrap_or(&RbumScopeKind::App).to_string()),
            disabled: Set(add_req.disabled.unwrap_or(false)),
            ..Default::default()
        }
    }

    fn package_modify(id: &str, modify_req: &RbumItemModifyReq, _: &TardisContext) -> rbum_item::ActiveModel {
        let mut rbum_item = rbum_item::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(code) = &modify_req.code {
            rbum_item.code = Set(code.to_string());
        }
        if let Some(uri_path) = &modify_req.uri_path {
            rbum_item.uri_path = Set(uri_path.to_string());
        }
        if let Some(name) = &modify_req.name {
            rbum_item.name = Set(name.to_string());
        }
        if let Some(icon) = &modify_req.icon {
            rbum_item.icon = Set(icon.to_string());
        }
        if let Some(sort) = modify_req.sort {
            rbum_item.sort = Set(sort);
        }
        if let Some(scope_kind) = &modify_req.scope_kind {
            rbum_item.scope_kind = Set(scope_kind.to_string());
        }
        if let Some(disabled) = modify_req.disabled {
            rbum_item.disabled = Set(disabled);
        }
        rbum_item
    }

    fn package_query(is_detail: bool, filter: &RbumBasicFilterReq, cxt: &TardisContext) -> SelectStatement {
        let mut query = Query::select();
        query
            .columns(vec![
                (rbum_item::Entity, rbum_item::Column::Id),
                (rbum_item::Entity, rbum_item::Column::Code),
                (rbum_item::Entity, rbum_item::Column::UriPath),
                (rbum_item::Entity, rbum_item::Column::Name),
                (rbum_item::Entity, rbum_item::Column::Icon),
                (rbum_item::Entity, rbum_item::Column::Sort),
                (rbum_item::Entity, rbum_item::Column::RelRbumKindId),
                (rbum_item::Entity, rbum_item::Column::RelRbumDomainId),
                (rbum_item::Entity, rbum_item::Column::RelAppId),
                (rbum_item::Entity, rbum_item::Column::RelTenantId),
                (rbum_item::Entity, rbum_item::Column::UpdaterId),
                (rbum_item::Entity, rbum_item::Column::CreateTime),
                (rbum_item::Entity, rbum_item::Column::UpdateTime),
                (rbum_item::Entity, rbum_item::Column::ScopeKind),
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

            query.query_with_safe(Self::get_table_name());
        }

        query.query_with_filter(Self::get_table_name(), filter, cxt);
        query.query_with_scope(Self::get_table_name(), cxt);

        query
    }

    async fn before_add_rbum(add_req: &mut RbumItemAddReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Self::check_scope(&add_req.rel_rbum_kind_id, RbumKindServ::get_table_name(), db, cxt).await?;
        Self::check_scope(&add_req.rel_rbum_domain_id, RbumDomainServ::get_table_name(), db, cxt).await
    }
}

#[async_trait]
impl<'a> RbumCrudOperation<'a, rbum_item_attr::ActiveModel, RbumItemAttrAddReq, RbumItemAttrModifyReq, RbumItemAttrSummaryResp, RbumItemAttrDetailResp> for RbumItemAttrServ {
    fn get_table_name() -> &'static str {
        rbum_item_attr::Entity.table_name()
    }

    fn package_add(add_req: &RbumItemAttrAddReq, _: &TardisContext) -> rbum_item_attr::ActiveModel {
        rbum_item_attr::ActiveModel {
            value: Set(add_req.value.to_string()),
            rel_rbum_item_id: Set(add_req.rel_rbum_item_id.to_string()),
            rel_rbum_kind_attr_id: Set(add_req.rel_rbum_kind_attr_id.to_string()),
            ..Default::default()
        }
    }

    fn package_modify(id: &str, modify_req: &RbumItemAttrModifyReq, _: &TardisContext) -> rbum_item_attr::ActiveModel {
        rbum_item_attr::ActiveModel {
            id: Set(id.to_string()),
            value: Set(modify_req.value.to_string()),
            ..Default::default()
        }
    }

    fn package_query(is_detail: bool, filter: &RbumBasicFilterReq, cxt: &TardisContext) -> SelectStatement {
        let mut query = Query::select();
        query
            .columns(vec![
                (rbum_item_attr::Entity, rbum_item_attr::Column::Id),
                (rbum_item_attr::Entity, rbum_item_attr::Column::Value),
                (rbum_item_attr::Entity, rbum_item_attr::Column::RelRbumItemId),
                (rbum_item_attr::Entity, rbum_item_attr::Column::RelRbumKindAttrId),
                (rbum_item_attr::Entity, rbum_item_attr::Column::RelAppId),
                (rbum_item_attr::Entity, rbum_item_attr::Column::RelTenantId),
                (rbum_item_attr::Entity, rbum_item_attr::Column::UpdaterId),
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

        if is_detail {
            query.query_with_safe(Self::get_table_name());
        }

        query.query_with_filter(Self::get_table_name(), filter, cxt);

        query
    }

    async fn before_add_rbum(add_req: &mut RbumItemAttrAddReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Self::check_scope(&add_req.rel_rbum_item_id, RbumItemServ::get_table_name(), db, cxt).await?;
        Self::check_scope(&add_req.rel_rbum_kind_attr_id, RbumKindAttrServ::get_table_name(), db, cxt).await
    }
}
