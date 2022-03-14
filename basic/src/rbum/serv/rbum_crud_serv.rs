use std::process::id;
use std::ptr::eq;

use async_trait::async_trait;
use lazy_static::lazy_static;
use poem_openapi::types::{ParseFromJSON, ToJSON};
use sea_orm::sea_query::{Alias, Cond};
use sea_orm::{FromQueryResult, InsertResult, Select};
use serde::Serialize;
use tardis::basic::dto::TardisContext;
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::{TardisActiveModel, TardisRelDBlConnection};
use tardis::db::sea_query::{Expr, JoinType, Order, Query, SelectStatement};
use tardis::web::web_resp::TardisPage;

use crate::rbum::domain::{rbum_domain, rbum_item};
use crate::rbum::dto::filer_dto::RbumBasicFilterReq;
use crate::rbum::enumeration::RbumScopeKind;

lazy_static! {
    static ref UPDATER_TABLE: Alias = Alias::new("updater");
    static ref REL_APP_TABLE: Alias = Alias::new("relApp");
    static ref REL_TENANT_TABLE: Alias = Alias::new("relTenant");
    static ref ID_FIELD: Alias = Alias::new("id");
    static ref NAME_FIELD: Alias = Alias::new("name");
    static ref UPDATER_ID_FIELD: Alias = Alias::new("updater_id");
    static ref REL_APP_ID_FIELD: Alias = Alias::new("rel_app_id");
    static ref REL_TENANT_ID_FIELD: Alias = Alias::new("rel_tenant_id");
    static ref UPDATE_TIME_FIELD: Alias = Alias::new("update_time");
    static ref SCOPE_KIND_FIELD: Alias = Alias::new("scope_kind");
    static ref REL_KIND_ID_FIELD: Alias = Alias::new("rel_kind_id");
    static ref REL_DOMAIN_ID_FIELD: Alias = Alias::new("rel_domain_id");
    static ref DISABLED_FIELD: Alias = Alias::new("disabled");
}

pub trait RbumCrudQueryPackage {
    fn query_with_filter(&mut self, table_name: &str, filter: &RbumBasicFilterReq, cxt: &TardisContext) -> &mut Self;

    fn query_with_safe(&mut self, table_name: &str) -> &mut Self;
}

impl RbumCrudQueryPackage for SelectStatement {
    fn query_with_filter(&mut self, table_name: &str, filter: &RbumBasicFilterReq, cxt: &TardisContext) -> &mut Self {
        if filter.rel_cxt_app {
            self.and_where(Expr::tbl(Alias::new(table_name), REL_APP_ID_FIELD.clone()).eq(cxt.app_id.as_str()));
        }
        if filter.rel_cxt_tenant {
            self.and_where(Expr::tbl(Alias::new(table_name), REL_TENANT_ID_FIELD.clone()).eq(cxt.tenant_id.as_str()));
        }
        if filter.rel_cxt_updater {
            self.and_where(Expr::tbl(Alias::new(table_name), UPDATER_ID_FIELD.clone()).eq(cxt.account_id.as_str()));
        }
        if let Some(scope_kind) = &filter.scope_kind {
            self.and_where(Expr::tbl(Alias::new(table_name), SCOPE_KIND_FIELD.clone()).eq(scope_kind.to_string()));
        }
        if let Some(kind_id) = &filter.kind_id {
            self.and_where(Expr::tbl(Alias::new(table_name), REL_KIND_ID_FIELD.clone()).eq(kind_id.to_string()));
        }
        if let Some(domain_id) = &filter.domain_id {
            self.and_where(Expr::tbl(Alias::new(table_name), REL_DOMAIN_ID_FIELD.clone()).eq(domain_id.to_string()));
        }
        if let Some(enabled) = filter.enabled {
            self.and_where(Expr::tbl(Alias::new(table_name), DISABLED_FIELD.clone()).eq(!enabled));
        }
        self
    }

    fn query_with_safe(&mut self, table_name: &str) -> &mut Self {
        self.expr_as(Expr::tbl(REL_APP_TABLE.clone(), NAME_FIELD.clone()), Alias::new("rel_app_name"))
            .expr_as(Expr::tbl(REL_TENANT_TABLE.clone(), NAME_FIELD.clone()), Alias::new("rel_tenant_name"))
            .expr_as(Expr::tbl(UPDATER_TABLE.clone(), NAME_FIELD.clone()), Alias::new("updater_name"));
        self.join_as(
            JoinType::InnerJoin,
            rbum_item::Entity,
            REL_APP_TABLE.clone(),
            Expr::tbl(REL_APP_TABLE.clone(), ID_FIELD.clone()).equals(Alias::new(table_name), REL_APP_ID_FIELD.clone()),
        )
        .join_as(
            JoinType::InnerJoin,
            rbum_item::Entity,
            REL_TENANT_TABLE.clone(),
            Expr::tbl(REL_TENANT_TABLE.clone(), ID_FIELD.clone()).equals(Alias::new(table_name), REL_TENANT_ID_FIELD.clone()),
        )
        .join_as(
            JoinType::InnerJoin,
            rbum_item::Entity,
            UPDATER_TABLE.clone(),
            Expr::tbl(UPDATER_TABLE.clone(), ID_FIELD.clone()).equals(Alias::new(table_name), UPDATER_ID_FIELD.clone()),
        );
        self
    }
}

#[async_trait]
pub trait RbumCrudOperation<'a, E, AddReq, ModifyReq, SummaryResp, DetailResp>
where
    E: TardisActiveModel + Sync + Send,
    AddReq: Sync + Send,
    ModifyReq: Sync + Send,
    SummaryResp: FromQueryResult + ParseFromJSON + ToJSON + Serialize + Send + Sync,
    DetailResp: FromQueryResult + ParseFromJSON + ToJSON + Serialize + Send + Sync,
{
    fn get_table_name() -> &'static str;

    fn has_scope() -> bool;

    fn package_ownership_query(id: &str, cxt: &TardisContext) -> SelectStatement {
        let mut query = Query::select();
        query
            .column(ID_FIELD.clone())
            .from(Alias::new(Self::get_table_name()))
            .and_where(Expr::col(ID_FIELD.clone()).eq(id))
            .and_where(Expr::col(REL_APP_ID_FIELD.clone()).eq(cxt.app_id.as_str()))
            .and_where(Expr::col(REL_TENANT_ID_FIELD.clone()).eq(cxt.tenant_id.as_str()));
        query
    }

    fn package_scope_query(id: &str, cxt: &TardisContext) -> SelectStatement {
        let mut query = Query::select();
        query.column(ID_FIELD.clone()).from(Alias::new(Self::get_table_name())).and_where(Expr::col(ID_FIELD.clone()).eq(id)).cond_where(
            Cond::any()
                .add(Expr::col(SCOPE_KIND_FIELD.clone()).eq(RbumScopeKind::Global.to_string()))
                .add(
                    Cond::all()
                        .add(Expr::col(SCOPE_KIND_FIELD.clone()).eq(RbumScopeKind::Tenant.to_string()))
                        .add(Expr::col(REL_TENANT_ID_FIELD.clone()).eq(cxt.tenant_id.as_str())),
                )
                .add(Cond::all().add(Expr::col(SCOPE_KIND_FIELD.clone()).eq(RbumScopeKind::App.to_string())).add(Expr::col(REL_APP_ID_FIELD.clone()).eq(cxt.app_id.as_str()))),
        );
        query
    }

    fn package_add(add_req: &AddReq, cxt: &TardisContext) -> E;

    fn package_modify(id: &str, modify_req: &ModifyReq, cxt: &TardisContext) -> E;

    fn package_delete(id: &str, cxt: &TardisContext) -> Select<E::Entity>;

    fn package_query(is_detail: bool, filter: &RbumBasicFilterReq, cxt: &TardisContext) -> SelectStatement;

    async fn add_rbum(add_req: &AddReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<InsertResult<E>> {
        let domain = Self::package_add(add_req, cxt);
        Ok(db.insert_one(domain, cxt).await?)
    }

    async fn modify_rbum(id: &str, modify_req: &ModifyReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        if db.count(&Self::package_ownership_query(id, cxt)).await? == 0 {
            return Err(TardisError::NotFound(format!("The ownership of {}.{} is illegal", Self::get_table_name(), id)));
        }
        let domain = Self::package_modify(id, modify_req, cxt);
        db.update_one(domain, cxt).await?;
        Ok(())
    }

    async fn delete_rbum(id: &str, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        if db.count(&Self::package_ownership_query(id, cxt)).await? == 0 {
            return Err(TardisError::NotFound(format!("The ownership of {}.{} is illegal", Self::get_table_name(), id)));
        }
        let select = Self::package_delete(id, cxt);
        db.soft_delete(select, &cxt.account_id).await
    }

    async fn get_rbum(id: &str, filter: &RbumBasicFilterReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<DetailResp> {
        if Self::has_scope() && db.count(&Self::package_scope_query(id, cxt)).await? == 0 {
            return Err(TardisError::NotFound(format!("The scope of {}.{} is illegal", Self::get_table_name(), id)));
        }
        if !Self::has_scope() && db.count(&Self::package_ownership_query(id, cxt)).await? == 0 {
            return Err(TardisError::NotFound(format!("The ownership of {}.{} is illegal", Self::get_table_name(), id)));
        }
        let mut query = Self::package_query(true, filter, cxt);
        query.and_where(Expr::tbl(Alias::new(Self::get_table_name()), ID_FIELD.clone()).eq(id));
        let query = db.get_dto(&query).await?;
        match query {
            Some(resp) => Ok(resp),
            // TODO
            None => Err(TardisError::NotFound("".to_string())),
        }
    }

    async fn find_rbums(
        filter: &RbumBasicFilterReq,
        page_number: u64,
        page_size: u64,
        desc_sort_by_update: Option<bool>,
        db: &TardisRelDBlConnection<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<SummaryResp>> {
        let mut query = Self::package_query(false, filter, cxt);
        if let Some(sort) = desc_sort_by_update {
            query.order_by((Alias::new(Self::get_table_name()), UPDATE_TIME_FIELD.clone()), if sort { Order::Desc } else { Order::Asc });
        }
        let (records, total_size) = db.paginate_dtos(&query, page_number, page_size).await?;
        Ok(TardisPage {
            page_size,
            page_number,
            total_size,
            records,
        })
    }
}
