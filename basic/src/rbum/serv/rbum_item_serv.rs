use async_trait::async_trait;
use serde::Serialize;
use tardis::basic::dto::TardisContext;
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::{TardisActiveModel, TardisRelDBlConnection};
use tardis::db::sea_orm::*;
use tardis::db::sea_query::*;
use tardis::web::poem_openapi::types::{ParseFromJSON, ToJSON};
use tardis::web::web_resp::TardisPage;
use tardis::TardisFuns;

use crate::rbum::domain::{rbum_domain, rbum_item, rbum_item_attr, rbum_kind, rbum_kind_attr, rbum_set_item};
use crate::rbum::dto::filer_dto::{RbumBasicFilterReq, RbumItemFilterReq};
use crate::rbum::dto::rbum_item_attr_dto::{RbumItemAttrAddReq, RbumItemAttrDetailResp, RbumItemAttrModifyReq, RbumItemAttrSummaryResp};
use crate::rbum::dto::rbum_item_dto::{RbumItemAddReq, RbumItemDetailResp, RbumItemModifyReq, RbumItemSummaryResp};
use crate::rbum::enumeration::RbumScopeKind;
use crate::rbum::serv::rbum_crud_serv::{RbumCrudOperation, RbumCrudQueryPackage, CREATE_TIME_FIELD, ID_FIELD, UPDATE_TIME_FIELD};
use crate::rbum::serv::rbum_domain_serv::RbumDomainServ;
use crate::rbum::serv::rbum_kind_serv::{RbumKindAttrServ, RbumKindServ};

pub struct RbumItemServ;
pub struct RbumItemAttrServ;

#[async_trait]
impl<'a> RbumCrudOperation<'a, rbum_item::ActiveModel, RbumItemAddReq, RbumItemModifyReq, RbumItemSummaryResp, RbumItemDetailResp> for RbumItemServ {
    fn get_table_name() -> &'static str {
        rbum_item::Entity.table_name()
    }

    async fn package_add(add_req: &RbumItemAddReq, _: &TardisRelDBlConnection<'a>, _: &TardisContext) -> TardisResult<rbum_item::ActiveModel> {
        let id = format!("{}{}{}", add_req.rel_rbum_kind_id, add_req.rel_rbum_domain_id, TardisFuns::field.nanoid());
        let code = if let Some(code) = &add_req.code { code.0.clone() } else { id.clone() };
        let uri_path = if let Some(uri_path) = &add_req.uri_path { uri_path.0.clone() } else { code.clone() };
        Ok(rbum_item::ActiveModel {
            id: Set(id),
            code: Set(code),
            uri_path: Set(uri_path),
            name: Set(add_req.name.to_string()),
            icon: Set(add_req.icon.as_ref().unwrap_or(&"".to_string()).to_string()),
            sort: Set(add_req.sort.unwrap_or(0)),
            rel_rbum_kind_id: Set(add_req.rel_rbum_kind_id.to_string()),
            rel_rbum_domain_id: Set(add_req.rel_rbum_domain_id.to_string()),
            scope_kind: Set(add_req.scope_kind.as_ref().unwrap_or(&RbumScopeKind::App).to_string()),
            disabled: Set(add_req.disabled.unwrap_or(false)),
            ..Default::default()
        })
    }

    async fn package_modify(id: &str, modify_req: &RbumItemModifyReq, _: &TardisRelDBlConnection<'a>, _: &TardisContext) -> TardisResult<rbum_item::ActiveModel> {
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
        Ok(rbum_item)
    }

    async fn package_query(is_detail: bool, filter: &RbumBasicFilterReq, _: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<SelectStatement> {
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
                (rbum_item::Entity, rbum_item::Column::RelAppCode),
                (rbum_item::Entity, rbum_item::Column::UpdaterCode),
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

        Ok(query)
    }

    async fn before_add_rbum(add_req: &mut RbumItemAddReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Self::check_scope(&add_req.rel_rbum_kind_id, RbumKindServ::get_table_name(), db, cxt).await?;
        Self::check_scope(&add_req.rel_rbum_domain_id, RbumDomainServ::get_table_name(), db, cxt).await?;
        if let Some(code) = &add_req.code {
            if db
                .count(
                    Query::select()
                        .column(rbum_item::Column::Id)
                        .from(rbum_item::Entity)
                        .and_where(Expr::col(rbum_item::Column::RelAppCode).eq(cxt.app_code.as_str()))
                        .and_where(Expr::col(rbum_item::Column::Code).eq(code.0.as_str())),
                )
                .await?
                > 0
            {
                return Err(TardisError::BadRequest(format!("Code {} already exists", code.0)));
            }
        }
        Ok(())
    }

    async fn before_delete_rbum(id: &str, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Self::check_ownership(id, db, cxt).await?;
        if db.count(Query::select().column(rbum_item_attr::Column::Id).from(rbum_item_attr::Entity).and_where(Expr::col(rbum_item_attr::Column::RelRbumItemId).eq(id))).await? > 0 {
            return Err(TardisError::BadRequest("can not delete rbum item when there are rbum item attr".to_string()));
        }
        if db.count(Query::select().column(rbum_set_item::Column::Id).from(rbum_set_item::Entity).and_where(Expr::col(rbum_set_item::Column::RelRbumItemId).eq(id))).await? > 0 {
            return Err(TardisError::BadRequest("can not delete rbum item when there are rbum set item".to_string()));
        }
        Ok(())
    }
}

#[async_trait]
pub trait RbumItemCrudOperation<'a, EXT, AddReq, ModifyReq, SummaryResp, DetailResp>
where
    EXT: TardisActiveModel + Sync + Send,
    AddReq: Sync + Send,
    ModifyReq: Sync + Send,
    SummaryResp: FromQueryResult + ParseFromJSON + ToJSON + Serialize + Send + Sync,
    DetailResp: FromQueryResult + ParseFromJSON + ToJSON + Serialize + Send + Sync,
{
    fn get_ext_table_name() -> &'static str;
    fn get_rbum_kind_id() -> String;
    fn get_rbum_domain_id() -> String;

    async fn package_item_add(add_req: &AddReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<RbumItemAddReq>;

    async fn package_ext_add(id: &str, add_req: &AddReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<EXT>;

    async fn package_item_modify(id: &str, modify_req: &ModifyReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<Option<RbumItemModifyReq>>;

    async fn package_ext_modify(id: &str, modify_req: &ModifyReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<Option<EXT>>;

    async fn package_delete(id: &str, _db: &TardisRelDBlConnection<'a>, _cxt: &TardisContext) -> TardisResult<Select<EXT::Entity>> {
        Ok(EXT::Entity::find().filter(Expr::col(ID_FIELD.clone()).eq(id)))
    }

    async fn package_item_query(query: &mut SelectStatement, is_detail: bool, filter: &RbumItemFilterReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext)
        -> TardisResult<()>;

    async fn before_add_item(_: &mut AddReq, _: &TardisRelDBlConnection<'a>, _: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    async fn after_add_item(_: &str, _: &TardisRelDBlConnection<'a>, _: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    async fn add_item(add_req: &mut AddReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<String> {
        Self::before_add_item(add_req, db, cxt).await?;
        let mut item_add_resp = Self::package_item_add(add_req, db, cxt).await?;
        item_add_resp.rel_rbum_kind_id = Self::get_rbum_kind_id();
        item_add_resp.rel_rbum_domain_id = Self::get_rbum_domain_id();
        let id = RbumItemServ::add_rbum(&mut item_add_resp, db, cxt).await?;
        let ext_domain = Self::package_ext_add(&id, add_req, db, cxt).await?;
        db.insert_one(ext_domain, cxt).await?;
        Self::after_add_item(&id, db, cxt).await?;
        Ok(id)
    }

    async fn before_modify_item(_: &str, _: &mut ModifyReq, _: &TardisRelDBlConnection<'a>, _: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    async fn after_modify_item(_: &str, _: &mut ModifyReq, _: &TardisRelDBlConnection<'a>, _: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    async fn modify_item(id: &str, modify_req: &mut ModifyReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Self::before_modify_item(id, modify_req, db, cxt).await?;
        let item_modify_resp = Self::package_item_modify(id, modify_req, db, cxt).await?;
        if let Some(mut item_modify_resp) = item_modify_resp {
            RbumItemServ::modify_rbum(id, &mut item_modify_resp, db, cxt).await?;
        }
        let ext_domain = Self::package_ext_modify(id, modify_req, db, cxt).await?;
        if let Some(ext_domain) = ext_domain {
            db.update_one(ext_domain, cxt).await?;
        }
        Self::after_modify_item(id, modify_req, db, cxt).await
    }

    async fn before_delete_item(_: &str, _: &TardisRelDBlConnection<'a>, _: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    async fn after_delete_item(_: &str, _: &TardisRelDBlConnection<'a>, _: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    async fn delete_item(id: &str, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        Self::before_delete_item(id, db, cxt).await?;
        RbumItemServ::delete_rbum(id, db, cxt).await?;
        let select = Self::package_delete(id, db, cxt).await?;
        let delete_records = db.soft_delete(select, &cxt.account_code).await?;
        Self::after_delete_item(id, db, cxt).await?;
        Ok(delete_records)
    }

    async fn get_item(id: &str, filter: &RbumItemFilterReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<DetailResp> {
        let mut query = RbumItemServ::package_query(
            true,
            &RbumBasicFilterReq {
                rel_cxt_app: filter.rel_cxt_app,
                rel_cxt_updater: filter.rel_cxt_updater,
                scope_kind: filter.scope_kind.clone(),
                rel_tenant_code: filter.iam_tenant_code.clone(),
                rel_app_code: filter.iam_app_code.clone(),
                rbum_kind_id: Some(Self::get_rbum_kind_id()),
                rbum_domain_id: Some(Self::get_rbum_domain_id()),
                enabled: filter.enabled,
                name: filter.name.clone(),
                code: filter.code.clone(),
                ..Default::default()
            },
            db,
            cxt,
        )
        .await?;
        query.inner_join(
            Alias::new(Self::get_ext_table_name()),
            Expr::tbl(Alias::new(Self::get_ext_table_name()), ID_FIELD.clone()).equals(rbum_item::Entity, rbum_item::Column::Id),
        );
        Self::package_item_query(&mut query, true, filter, db, cxt).await?;
        query.and_where(Expr::tbl(rbum_item::Entity, rbum_item::Column::Id).eq(id));
        let query = db.get_dto(&query).await?;
        match query {
            Some(resp) => Ok(resp),
            // TODO
            None => Err(TardisError::NotFound("".to_string())),
        }
    }

    async fn paginate_items(
        filter: &RbumItemFilterReq,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        db: &TardisRelDBlConnection<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<SummaryResp>> {
        let mut query = RbumItemServ::package_query(
            false,
            &RbumBasicFilterReq {
                rel_cxt_app: filter.rel_cxt_app,
                rel_cxt_updater: filter.rel_cxt_updater,
                scope_kind: filter.scope_kind.clone(),
                rel_tenant_code: filter.iam_tenant_code.clone(),
                rel_app_code: filter.iam_app_code.clone(),
                rbum_kind_id: Some(Self::get_rbum_kind_id()),
                rbum_domain_id: Some(Self::get_rbum_domain_id()),
                enabled: filter.enabled,
                name: filter.name.clone(),
                code: filter.code.clone(),
                ..Default::default()
            },
            db,
            cxt,
        )
        .await?;
        query.inner_join(
            Alias::new(Self::get_ext_table_name()),
            Expr::tbl(Alias::new(Self::get_ext_table_name()), ID_FIELD.clone()).equals(rbum_item::Entity, rbum_item::Column::Id),
        );
        Self::package_item_query(&mut query, false, filter, db, cxt).await?;
        if let Some(sort) = desc_sort_by_create {
            query.order_by((rbum_item::Entity, CREATE_TIME_FIELD.clone()), if sort { Order::Desc } else { Order::Asc });
        }
        if let Some(sort) = desc_sort_by_update {
            query.order_by((rbum_item::Entity, UPDATE_TIME_FIELD.clone()), if sort { Order::Desc } else { Order::Asc });
        }
        let (records, total_size) = db.paginate_dtos(&query, page_number, page_size).await?;
        Ok(TardisPage {
            page_size,
            page_number,
            total_size,
            records,
        })
    }

    async fn find_items(
        filter: &RbumItemFilterReq,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        db: &TardisRelDBlConnection<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<Vec<SummaryResp>> {
        let mut query = RbumItemServ::package_query(
            false,
            &RbumBasicFilterReq {
                rel_cxt_app: filter.rel_cxt_app,
                rel_cxt_updater: filter.rel_cxt_updater,
                scope_kind: filter.scope_kind.clone(),
                rel_tenant_code: filter.iam_tenant_code.clone(),
                rel_app_code: filter.iam_app_code.clone(),
                rbum_kind_id: Some(Self::get_rbum_kind_id()),
                rbum_domain_id: Some(Self::get_rbum_domain_id()),
                enabled: filter.enabled,
                name: filter.name.clone(),
                code: filter.code.clone(),
                ..Default::default()
            },
            db,
            cxt,
        )
        .await?;
        query.inner_join(
            Alias::new(Self::get_ext_table_name()),
            Expr::tbl(Alias::new(Self::get_ext_table_name()), ID_FIELD.clone()).equals(rbum_item::Entity, rbum_item::Column::Id),
        );
        Self::package_item_query(&mut query, false, filter, db, cxt).await?;
        if let Some(sort) = desc_sort_by_create {
            query.order_by((rbum_item::Entity, CREATE_TIME_FIELD.clone()), if sort { Order::Desc } else { Order::Asc });
        }
        if let Some(sort) = desc_sort_by_update {
            query.order_by((rbum_item::Entity, UPDATE_TIME_FIELD.clone()), if sort { Order::Desc } else { Order::Asc });
        }
        Ok(db.find_dtos(&query).await?)
    }

    async fn count_items(filter: &RbumItemFilterReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        let query = RbumItemServ::package_query(
            false,
            &RbumBasicFilterReq {
                rel_cxt_app: filter.rel_cxt_app,
                rel_cxt_updater: filter.rel_cxt_updater,
                scope_kind: filter.scope_kind.clone(),
                rbum_kind_id: Some(Self::get_rbum_kind_id()),
                rbum_domain_id: Some(Self::get_rbum_domain_id()),
                enabled: filter.enabled,
                name: filter.name.clone(),
                code: filter.code.clone(),
                ..Default::default()
            },
            db,
            cxt,
        )
        .await?;
        db.count(&query).await
    }
}

#[async_trait]
impl<'a> RbumCrudOperation<'a, rbum_item_attr::ActiveModel, RbumItemAttrAddReq, RbumItemAttrModifyReq, RbumItemAttrSummaryResp, RbumItemAttrDetailResp> for RbumItemAttrServ {
    fn get_table_name() -> &'static str {
        rbum_item_attr::Entity.table_name()
    }

    async fn package_add(add_req: &RbumItemAttrAddReq, _: &TardisRelDBlConnection<'a>, _: &TardisContext) -> TardisResult<rbum_item_attr::ActiveModel> {
        Ok(rbum_item_attr::ActiveModel {
            id: Set(TardisFuns::field.nanoid()),
            value: Set(add_req.value.to_string()),
            rel_rbum_item_id: Set(add_req.rel_rbum_item_id.to_string()),
            rel_rbum_kind_attr_id: Set(add_req.rel_rbum_kind_attr_id.to_string()),
            ..Default::default()
        })
    }

    async fn package_modify(id: &str, modify_req: &RbumItemAttrModifyReq, _: &TardisRelDBlConnection<'a>, _: &TardisContext) -> TardisResult<rbum_item_attr::ActiveModel> {
        Ok(rbum_item_attr::ActiveModel {
            id: Set(id.to_string()),
            value: Set(modify_req.value.to_string()),
            ..Default::default()
        })
    }

    async fn package_query(is_detail: bool, filter: &RbumBasicFilterReq, _: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<SelectStatement> {
        let mut query = Query::select();
        query
            .columns(vec![
                (rbum_item_attr::Entity, rbum_item_attr::Column::Id),
                (rbum_item_attr::Entity, rbum_item_attr::Column::Value),
                (rbum_item_attr::Entity, rbum_item_attr::Column::RelRbumItemId),
                (rbum_item_attr::Entity, rbum_item_attr::Column::RelRbumKindAttrId),
                (rbum_item_attr::Entity, rbum_item_attr::Column::RelAppCode),
                (rbum_item_attr::Entity, rbum_item_attr::Column::UpdaterCode),
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

        Ok(query)
    }

    async fn before_add_rbum(add_req: &mut RbumItemAttrAddReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Self::check_scope(&add_req.rel_rbum_item_id, RbumItemServ::get_table_name(), db, cxt).await?;
        Self::check_scope(&add_req.rel_rbum_kind_attr_id, RbumKindAttrServ::get_table_name(), db, cxt).await
    }
}
