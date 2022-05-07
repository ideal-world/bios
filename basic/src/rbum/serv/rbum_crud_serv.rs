use async_trait::async_trait;
use lazy_static::lazy_static;
use serde::Serialize;
use tardis::basic::dto::{TardisContext, TardisFunsInst};
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm::*;
use tardis::db::sea_query::{Alias, Cond, Expr, IntoValueTuple, JoinType, Order, Query, SelectStatement, Value, ValueTuple};
use tardis::web::poem_openapi::types::{ParseFromJSON, ToJSON};
use tardis::web::web_resp::TardisPage;

use crate::rbum::domain::rbum_item;
use crate::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use crate::rbum::helper::rbum_scope_helper;

lazy_static! {
    pub static ref OWNER_TABLE: Alias = Alias::new("t_owner");
    pub static ref ID_FIELD: Alias = Alias::new("id");
    pub static ref OWNER_FIELD: Alias = Alias::new("owner");
    pub static ref OWN_PATHS_FIELD: Alias = Alias::new("own_paths");
    pub static ref CREATE_TIME_FIELD: Alias = Alias::new("create_time");
    pub static ref UPDATE_TIME_FIELD: Alias = Alias::new("update_time");
    pub static ref CODE_FIELD: Alias = Alias::new("code");
    pub static ref NAME_FIELD: Alias = Alias::new("name");
    pub static ref SORT_FIELD: Alias = Alias::new("sort");
    pub static ref SCOPE_LEVEL_FIELD: Alias = Alias::new("scope_level");
    pub static ref REL_KIND_ID_FIELD: Alias = Alias::new("rel_rbum_kind_id");
    pub static ref REL_DOMAIN_ID_FIELD: Alias = Alias::new("rel_rbum_domain_id");
    pub static ref DISABLED_FIELD: Alias = Alias::new("disabled");
}

#[async_trait]
pub trait RbumCrudOperation<'a, E, AddReq, ModifyReq, SummaryResp, DetailResp, FilterReq>
where
    E: TardisActiveModel + Sync + Send,
    AddReq: Sync + Send,
    ModifyReq: Sync + Send,
    SummaryResp: FromQueryResult + ParseFromJSON + ToJSON + Serialize + Send + Sync,
    DetailResp: FromQueryResult + ParseFromJSON + ToJSON + Serialize + Send + Sync,
    FilterReq: Sync + Send,
{
    fn get_table_name() -> &'static str;

    // ----------------------------- Ownership -------------------------------

    async fn check_ownership(id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Self::check_ownership_with_table_name(id, Self::get_table_name(), funs, cxt).await
    }

    async fn check_ownership_with_table_name(id: &str, table_name: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        if funs.db().count(&Self::package_ownership_query_with_table_name(id, table_name, cxt)).await? == 0 {
            return Err(TardisError::NotFound(format!("The ownership of {}.{} is illegal", Self::get_table_name(), id)));
        }
        Ok(())
    }

    fn package_ownership_query(id: &str, cxt: &TardisContext) -> SelectStatement {
        Self::package_ownership_query_with_table_name(id, Self::get_table_name(), cxt)
    }

    fn package_ownership_query_with_table_name(id: &str, table_name: &str, cxt: &TardisContext) -> SelectStatement {
        let mut query = Query::select();
        query
            .column(ID_FIELD.clone())
            .from(Alias::new(table_name))
            .and_where(Expr::col(ID_FIELD.clone()).eq(id))
            .and_where(Expr::col(OWN_PATHS_FIELD.clone()).like(format!("{}%", cxt.own_paths).as_str()));
        query
    }

    // ----------------------------- Scope -------------------------------

    async fn check_scope(id: &str, table_name: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        if funs
            .db()
            .count(
                &Query::select()
                    .column((Alias::new(table_name), ID_FIELD.clone()))
                    .from(Alias::new(table_name))
                    .and_where(Expr::tbl(Alias::new(table_name), ID_FIELD.clone()).eq(id))
                    .with_scope(table_name, cxt),
            )
            .await?
            == 0
        {
            return Err(TardisError::NotFound(format!("The scope of {}.{} is illegal", Self::get_table_name(), id)));
        }
        Ok(())
    }

    // ----------------------------- Exist -------------------------------

    async fn check_exist_before_delete(id: &str, rel_table_name: &str, rel_field_name: &str, funs: &TardisFunsInst<'a>) -> TardisResult<()> {
        if funs
            .db()
            .count(
                Query::select()
                    .column((Alias::new(rel_table_name), ID_FIELD.clone()))
                    .from(Alias::new(rel_table_name))
                    .and_where(Expr::tbl(Alias::new(rel_table_name), Alias::new(rel_field_name)).eq(id)),
            )
            .await?
            > 0
        {
            return Err(TardisError::BadRequest(format!(
                "Can not delete {} when there are {}",
                Self::get_table_name(),
                rel_table_name
            )));
        }
        Ok(())
    }

    async fn check_exist_with_cond_before_delete(rel_table_name: &str, condition: Condition, funs: &TardisFunsInst<'a>) -> TardisResult<()> {
        if funs.db().count(Query::select().column((Alias::new(rel_table_name), ID_FIELD.clone())).from(Alias::new(rel_table_name)).cond_where(condition)).await? > 0 {
            return Err(TardisError::BadRequest(format!(
                "Can not delete {} when there are {}",
                Self::get_table_name(),
                rel_table_name
            )));
        }
        Ok(())
    }

    // ----------------------------- Add -------------------------------

    async fn package_add(add_req: &AddReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<E>;

    async fn before_add_rbum(_: &mut AddReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    async fn after_add_rbum(_: &str, _: &AddReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    async fn add_rbum(add_req: &mut AddReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<String> {
        Self::before_add_rbum(add_req, funs, cxt).await?;
        let domain = Self::package_add(add_req, funs, cxt).await?;
        let insert_result = funs.db().insert_one(domain, cxt).await?;
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
            Self::after_add_rbum(&id, add_req, funs, cxt).await?;
            Ok(id)
        } else {
            return Err(TardisError::InternalError(
                "The id data type is invalid, currently only the string is supported".to_string(),
            ));
        }
    }

    // ----------------------------- Modify -------------------------------

    async fn package_modify(id: &str, modify_req: &ModifyReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<E>;

    async fn before_modify_rbum(id: &str, _: &mut ModifyReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Self::check_ownership(id, funs, cxt).await
    }

    async fn after_modify_rbum(_: &str, _: &mut ModifyReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    async fn modify_rbum(id: &str, modify_req: &mut ModifyReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Self::before_modify_rbum(id, modify_req, funs, cxt).await?;
        let domain = Self::package_modify(id, modify_req, funs, cxt).await?;
        funs.db().update_one(domain, cxt).await?;
        Self::after_modify_rbum(id, modify_req, funs, cxt).await
    }

    // ----------------------------- Delete -------------------------------

    async fn package_delete(id: &str, _funs: &TardisFunsInst<'a>, _cxt: &TardisContext) -> TardisResult<Select<E::Entity>> {
        Ok(E::Entity::find().filter(Expr::col(ID_FIELD.clone()).eq(id)))
    }

    async fn before_delete_rbum(id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Self::check_ownership(id, funs, cxt).await
    }

    async fn after_delete_rbum(_: &str, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    async fn delete_rbum(id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        Self::before_delete_rbum(id, funs, cxt).await?;
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
                funs.mq().request(mq_topic_entity_deleted, tardis::TardisFuns::json.obj_to_string(delete_record)?, &mq_header).await?;
            }
            Self::after_delete_rbum(id, funs, cxt).await?;
            Ok(delete_records.len() as u64)
        }
        #[cfg(not(feature = "with-mq"))]
        {
            let delete_records = funs.db().soft_delete(select, &cxt.owner).await?;
            Ok(delete_records)
        }
    }

    // ----------------------------- Query -------------------------------

    async fn package_query(is_detail: bool, filter: &FilterReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<SelectStatement>;

    async fn peek_rbum(id: &str, filter: &FilterReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<SummaryResp> {
        let mut query = Self::package_query(false, filter, funs, cxt).await?;
        query.and_where(Expr::tbl(Alias::new(Self::get_table_name()), ID_FIELD.clone()).eq(id));
        let query = funs.db().get_dto(&query).await?;
        match query {
            Some(resp) => Ok(resp),
            // TODO
            None => Err(TardisError::NotFound("".to_string())),
        }
    }

    async fn get_rbum(id: &str, filter: &FilterReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<DetailResp> {
        let mut query = Self::package_query(true, filter, funs, cxt).await?;
        query.and_where(Expr::tbl(Alias::new(Self::get_table_name()), ID_FIELD.clone()).eq(id));
        let query = funs.db().get_dto(&query).await?;
        match query {
            Some(resp) => Ok(resp),
            // TODO
            None => Err(TardisError::NotFound("".to_string())),
        }
    }

    async fn paginate_rbums(
        filter: &FilterReq,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<SummaryResp>> {
        let mut query = Self::package_query(false, filter, funs, cxt).await?;
        if let Some(sort) = desc_sort_by_create {
            query.order_by((Alias::new(Self::get_table_name()), CREATE_TIME_FIELD.clone()), if sort { Order::Desc } else { Order::Asc });
        }
        if let Some(sort) = desc_sort_by_update {
            query.order_by((Alias::new(Self::get_table_name()), UPDATE_TIME_FIELD.clone()), if sort { Order::Desc } else { Order::Asc });
        }
        let (records, total_size) = funs.db().paginate_dtos(&query, page_number, page_size).await?;
        Ok(TardisPage {
            page_size,
            page_number,
            total_size,
            records,
        })
    }

    async fn find_rbums(
        filter: &FilterReq,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<Vec<SummaryResp>> {
        let mut query = Self::package_query(false, filter, funs, cxt).await?;
        if let Some(sort) = desc_sort_by_create {
            query.order_by((Alias::new(Self::get_table_name()), CREATE_TIME_FIELD.clone()), if sort { Order::Desc } else { Order::Asc });
        }
        if let Some(sort) = desc_sort_by_update {
            query.order_by((Alias::new(Self::get_table_name()), UPDATE_TIME_FIELD.clone()), if sort { Order::Desc } else { Order::Asc });
        }
        Ok(funs.db().find_dtos(&query).await?)
    }

    async fn count_rbums(filter: &FilterReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        let query = Self::package_query(false, filter, funs, cxt).await?;
        funs.db().count(&query).await
    }

    async fn exist_rbum(filter: &FilterReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<bool> {
        let query = Self::count_rbums(filter, funs, cxt).await?;
        Ok(query > 0)
    }
}

pub trait RbumCrudQueryPackage {
    fn with_filter(&mut self, table_name: &str, filter: &RbumBasicFilterReq, ignore_owner: bool, has_scope: bool, cxt: &TardisContext) -> &mut Self;
    fn with_scope(&mut self, table_name: &str, cxt: &TardisContext) -> &mut Self;
}

impl RbumCrudQueryPackage for SelectStatement {
    fn with_filter(&mut self, table_name: &str, filter: &RbumBasicFilterReq, with_owner: bool, has_scope: bool, cxt: &TardisContext) -> &mut Self {
        if filter.rel_cxt_owner {
            self.and_where(Expr::tbl(Alias::new(table_name), OWNER_FIELD.clone()).eq(cxt.owner.as_str()));
        }
        if let Some(ids) = &filter.ids {
            self.and_where(Expr::tbl(Alias::new(table_name), ID_FIELD.clone()).is_in(ids.clone()));
        }

        if let Some(scope_level) = &filter.scope_level {
            self.and_where(Expr::tbl(Alias::new(table_name), SCOPE_LEVEL_FIELD.clone()).eq(scope_level.to_int()));
        }
        if let Some(enabled) = filter.enabled {
            self.and_where(Expr::tbl(Alias::new(table_name), DISABLED_FIELD.clone()).eq(!enabled));
        }

        if let Some(name) = &filter.name {
            self.and_where(Expr::tbl(Alias::new(table_name), NAME_FIELD.clone()).like(format!("%{}%", name).as_str()));
        }
        if let Some(code) = &filter.code {
            self.and_where(Expr::tbl(Alias::new(table_name), CODE_FIELD.clone()).like(format!("{}%", code).as_str()));
        }

        if let Some(rbum_kind_id) = &filter.rbum_kind_id {
            self.and_where(Expr::tbl(Alias::new(table_name), REL_KIND_ID_FIELD.clone()).eq(rbum_kind_id.to_string()));
        }
        if let Some(rbum_domain_id) = &filter.rbum_domain_id {
            self.and_where(Expr::tbl(Alias::new(table_name), REL_DOMAIN_ID_FIELD.clone()).eq(rbum_domain_id.to_string()));
        }
        if with_owner {
            self.expr_as(Expr::tbl(OWNER_TABLE.clone(), NAME_FIELD.clone()), Alias::new("owner_name")).join_as(
                JoinType::InnerJoin,
                rbum_item::Entity,
                OWNER_TABLE.clone(),
                Expr::tbl(OWNER_TABLE.clone(), ID_FIELD.clone()).equals(Alias::new(table_name), OWNER_FIELD.clone()),
            );
        }
        if let Some(own_paths) = &filter.own_paths {
            if filter.with_sub_own_paths {
                self.and_where(Expr::tbl(Alias::new(table_name), OWN_PATHS_FIELD.clone()).like(format!("{}%", own_paths).as_str()));
            } else {
                self.and_where(Expr::tbl(Alias::new(table_name), OWN_PATHS_FIELD.clone()).eq(own_paths.to_string()));
            }
        } else if has_scope && !filter.ignore_scope {
            self.with_scope(table_name, cxt);
        } else if !has_scope && !filter.ignore_scope {
            if filter.with_sub_own_paths {
                self.and_where(Expr::tbl(Alias::new(table_name), OWN_PATHS_FIELD.clone()).like(format!("{}%", cxt.own_paths).as_str()));
            } else {
                self.and_where(Expr::tbl(Alias::new(table_name), OWN_PATHS_FIELD.clone()).eq(cxt.own_paths.as_str()));
            }
        }
        if let Some(desc_by_sort) = filter.desc_by_sort {
            self.order_by((Alias::new(table_name), SORT_FIELD.clone()), if desc_by_sort { Order::Desc } else { Order::Asc });
        }
        self
    }

    fn with_scope(&mut self, table_name: &str, cxt: &TardisContext) -> &mut Self {
        self.cond_where(
            Cond::all().add(
                Cond::any()
                    .add(
                        Cond::all()
                            .add(Expr::tbl(Alias::new(table_name), SCOPE_LEVEL_FIELD.clone()).eq(-1))
                            .add(Expr::tbl(Alias::new(table_name), OWN_PATHS_FIELD.clone()).eq(cxt.own_paths.as_str())),
                    )
                    .add(Expr::tbl(Alias::new(table_name), SCOPE_LEVEL_FIELD.clone()).eq(0))
                    .add(
                        Cond::all()
                            .add(Expr::tbl(Alias::new(table_name), SCOPE_LEVEL_FIELD.clone()).eq(1))
                            .add(Expr::tbl(Alias::new(table_name), OWN_PATHS_FIELD.clone()).like(format!("{}%", rbum_scope_helper::get_pre_paths(1, &cxt.own_paths)).as_str())),
                    )
                    .add(
                        Cond::all()
                            .add(Expr::tbl(Alias::new(table_name), SCOPE_LEVEL_FIELD.clone()).eq(2))
                            .add(Expr::tbl(Alias::new(table_name), OWN_PATHS_FIELD.clone()).like(format!("{}%", rbum_scope_helper::get_pre_paths(2, &cxt.own_paths)).as_str())),
                    )
                    .add(
                        Cond::all()
                            .add(Expr::tbl(Alias::new(table_name), SCOPE_LEVEL_FIELD.clone()).eq(3))
                            .add(Expr::tbl(Alias::new(table_name), OWN_PATHS_FIELD.clone()).like(format!("{}%", rbum_scope_helper::get_pre_paths(3, &cxt.own_paths)).as_str())),
                    ),
            ),
        );
        self
    }
}

#[derive(Debug, FromQueryResult)]
pub struct NameResp {
    pub name: String,
}
