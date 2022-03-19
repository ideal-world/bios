use async_trait::async_trait;
use lazy_static::lazy_static;
use serde::Serialize;
use tardis::basic::dto::TardisContext;
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::{TardisActiveModel, TardisRelDBlConnection};
use tardis::db::sea_orm::*;
use tardis::db::sea_query::{Alias, Cond, Expr, IntoValueTuple, JoinType, Order, Query, SelectStatement, Value, ValueTuple};
use tardis::web::poem_openapi::types::{ParseFromJSON, ToJSON};
use tardis::web::web_resp::TardisPage;

use crate::rbum::domain::rbum_item;
use crate::rbum::dto::filer_dto::RbumBasicFilterReq;
use crate::rbum::enumeration::RbumScopeKind;

lazy_static! {
    static ref UPDATER_TABLE: Alias = Alias::new("updater");
    static ref REL_APP_TABLE: Alias = Alias::new("relApp");
    static ref ID_FIELD: Alias = Alias::new("id");
    static ref CODE_FIELD: Alias = Alias::new("code");
    static ref NAME_FIELD: Alias = Alias::new("name");
    static ref UPDATER_CODE_FIELD: Alias = Alias::new("updater_code");
    static ref REL_APP_CODE_FIELD: Alias = Alias::new("rel_app_code");
    static ref UPDATE_TIME_FIELD: Alias = Alias::new("update_time");
    static ref SCOPE_KIND_FIELD: Alias = Alias::new("scope_kind");
    static ref REL_KIND_ID_FIELD: Alias = Alias::new("rel_kind_id");
    static ref REL_DOMAIN_ID_FIELD: Alias = Alias::new("rel_domain_id");
    static ref DISABLED_FIELD: Alias = Alias::new("disabled");
}

pub trait RbumCrudQueryPackage {
    fn query_with_filter(&mut self, table_name: &str, filter: &RbumBasicFilterReq, cxt: &TardisContext) -> &mut Self;

    fn query_with_safe(&mut self, table_name: &str) -> &mut Self;

    fn query_with_scope(&mut self, table_name: &str, cxt: &TardisContext) -> &mut Self;
}

impl RbumCrudQueryPackage for SelectStatement {
    fn query_with_filter(&mut self, table_name: &str, filter: &RbumBasicFilterReq, cxt: &TardisContext) -> &mut Self {
        if filter.rel_cxt_app {
            self.and_where(Expr::tbl(Alias::new(table_name), REL_APP_CODE_FIELD.clone()).eq(cxt.app_code.as_str()));
        }
        if filter.rel_cxt_updater {
            self.and_where(Expr::tbl(Alias::new(table_name), UPDATER_CODE_FIELD.clone()).eq(cxt.account_code.as_str()));
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
        if let Some(name) = &filter.name {
            self.and_where(Expr::tbl(Alias::new(table_name), NAME_FIELD.clone()).like(name));
        }
        if let Some(ids) = &filter.ids {
            self.and_where(Expr::tbl(Alias::new(table_name), ID_FIELD.clone()).is_in(ids.clone()));
        }
        self
    }

    fn query_with_safe(&mut self, table_name: &str) -> &mut Self {
        self.expr_as(Expr::tbl(REL_APP_TABLE.clone(), NAME_FIELD.clone()), Alias::new("rel_app_name"))
            .expr_as(Expr::tbl(UPDATER_TABLE.clone(), NAME_FIELD.clone()), Alias::new("updater_name"));
        self.join_as(
            JoinType::InnerJoin,
            rbum_item::Entity,
            REL_APP_TABLE.clone(),
            Expr::tbl(REL_APP_TABLE.clone(), CODE_FIELD.clone()).equals(Alias::new(table_name), REL_APP_CODE_FIELD.clone()),
        )
        .join_as(
            JoinType::InnerJoin,
            rbum_item::Entity,
            UPDATER_TABLE.clone(),
            Expr::tbl(UPDATER_TABLE.clone(), CODE_FIELD.clone()).equals(Alias::new(table_name), UPDATER_CODE_FIELD.clone()),
        );
        self
    }

    fn query_with_scope(&mut self, table_name: &str, cxt: &TardisContext) -> &mut Self {
        self.cond_where(
            Cond::any()
                .add(Expr::tbl(Alias::new(table_name), SCOPE_KIND_FIELD.clone()).eq(RbumScopeKind::Global.to_string()))
                .add(
                    Cond::all()
                        .add(Expr::tbl(Alias::new(table_name), SCOPE_KIND_FIELD.clone()).eq(RbumScopeKind::Tenant.to_string()))
                        .add(Expr::tbl(Alias::new(table_name), REL_APP_CODE_FIELD.clone()).like(format!("{}%", cxt.tenant_code).as_str())),
                )
                .add(
                    Cond::all()
                        .add(Expr::tbl(Alias::new(table_name), SCOPE_KIND_FIELD.clone()).eq(RbumScopeKind::App.to_string()))
                        .add(Expr::tbl(Alias::new(table_name), REL_APP_CODE_FIELD.clone()).eq(cxt.app_code.as_str())),
                ),
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

    fn package_ownership_query(id: &str, cxt: &TardisContext) -> SelectStatement {
        Self::package_ownership_query_with_table_name(id, Self::get_table_name(), cxt)
    }

    fn package_ownership_query_with_table_name(id: &str, table_name: &str, cxt: &TardisContext) -> SelectStatement {
        let mut query = Query::select();
        query
            .column(ID_FIELD.clone())
            .from(Alias::new(table_name))
            .and_where(Expr::col(ID_FIELD.clone()).eq(id))
            .and_where(Expr::col(REL_APP_CODE_FIELD.clone()).eq(cxt.app_code.as_str()));
        query
    }

    fn package_scope_query(id: &str, table_name: &str, cxt: &TardisContext) -> SelectStatement {
        let mut query = Query::select();
        query
            .column((Alias::new(table_name), ID_FIELD.clone()))
            .from(Alias::new(table_name))
            .and_where(Expr::tbl(Alias::new(table_name), ID_FIELD.clone()).eq(id))
            .query_with_scope(table_name, cxt);
        query
    }

    async fn package_add(add_req: &AddReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<E>;

    async fn package_modify(id: &str, modify_req: &ModifyReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<E>;

    async fn package_delete(id: &str, _db: &TardisRelDBlConnection<'a>, _cxt: &TardisContext) -> TardisResult<Select<E::Entity>> {
        Ok(E::Entity::find().filter(Expr::col(ID_FIELD.clone()).eq(id)))
    }

    async fn package_query(is_detail: bool, filter: &RbumBasicFilterReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<SelectStatement>;

    async fn check_ownership(id: &str, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Self::check_ownership_with_table_name(id, Self::get_table_name(), db, cxt).await
    }

    async fn check_ownership_with_table_name(id: &str, table_name: &str, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        if db.count(&Self::package_ownership_query_with_table_name(id, table_name, cxt)).await? == 0 {
            return Err(TardisError::NotFound(format!("The ownership of {}.{} is illegal", Self::get_table_name(), id)));
        }
        Ok(())
    }

    async fn check_scope(id: &str, table_name: &str, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        if db.count(&Self::package_scope_query(id, table_name, cxt)).await? == 0 {
            return Err(TardisError::NotFound(format!("The scope of {}.{} is illegal", Self::get_table_name(), id)));
        }
        Ok(())
    }

    async fn before_add_rbum(_: &mut AddReq, _: &TardisRelDBlConnection<'a>, _: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    async fn after_add_rbum(_: &str, _: &TardisRelDBlConnection<'a>, _: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    async fn add_rbum(add_req: &mut AddReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<String> {
        Self::before_add_rbum(add_req, db, cxt).await?;
        let domain = Self::package_add(add_req, db, cxt).await?;
        let insert_result = db.insert_one(domain, cxt).await?;
        let id_value = insert_result.last_insert_id.into_value_tuple();
        let id = match id_value {
            ValueTuple::One(v) => {
                if let Value::String(s) = v {
                    s.map(|id| id.to_string())
                } else {
                    None
                }
            }
            _ => None,
        };
        if let Some(id) = id {
            Self::after_add_rbum(&id, db, cxt).await?;
            Ok(id)
        } else {
            return Err(TardisError::InternalError(
                "The id data type is invalid, currently only the string is supported".to_string(),
            ));
        }
    }

    async fn before_modify_rbum(id: &str, _: &mut ModifyReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Self::check_ownership(id, db, cxt).await
    }

    async fn after_modify_rbum(_: &str, _: &mut ModifyReq, _: &TardisRelDBlConnection<'a>, _: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    async fn modify_rbum(id: &str, modify_req: &mut ModifyReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Self::before_modify_rbum(id, modify_req, db, cxt).await?;
        let domain = Self::package_modify(id, modify_req, db, cxt).await?;
        db.update_one(domain, cxt).await?;
        Self::after_modify_rbum(id, modify_req, db, cxt).await
    }

    async fn before_delete_rbum(id: &str, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Self::check_ownership(id, db, cxt).await
    }

    async fn after_delete_rbum(_: &str, _: &TardisRelDBlConnection<'a>, _: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    async fn delete_rbum(id: &str, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        Self::before_delete_rbum(id, db, cxt).await?;
        let select = Self::package_delete(id, db, cxt).await?;
        let delete_records = db.soft_delete(select, &cxt.account_code).await?;
        Self::after_delete_rbum(id, db, cxt).await?;
        Ok(delete_records)
    }

    async fn get_rbum(id: &str, filter: &RbumBasicFilterReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<DetailResp> {
        let mut query = Self::package_query(true, filter, db, cxt).await?;
        query.and_where(Expr::tbl(Alias::new(Self::get_table_name()), ID_FIELD.clone()).eq(id));
        let query = db.get_dto(&query).await?;
        match query {
            Some(resp) => Ok(resp),
            // TODO
            None => Err(TardisError::NotFound("".to_string())),
        }
    }

    async fn paginate_rbums(
        filter: &RbumBasicFilterReq,
        page_number: u64,
        page_size: u64,
        desc_sort_by_update: Option<bool>,
        db: &TardisRelDBlConnection<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<SummaryResp>> {
        let mut query = Self::package_query(false, filter, db, cxt).await?;
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

    async fn find_rbums(filter: &RbumBasicFilterReq, desc_sort_by_update: Option<bool>, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<Vec<SummaryResp>> {
        let mut query = Self::package_query(false, filter, db, cxt).await?;
        if let Some(sort) = desc_sort_by_update {
            query.order_by((Alias::new(Self::get_table_name()), UPDATE_TIME_FIELD.clone()), if sort { Order::Desc } else { Order::Asc });
        }
        Ok(db.find_dtos(&query).await?)
    }
}

#[derive(Debug, FromQueryResult)]
pub struct NameResp {
    pub name: String,
}
