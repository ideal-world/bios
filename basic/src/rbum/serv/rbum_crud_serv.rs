use std::collections::HashMap;

use async_trait::async_trait;
use itertools::Itertools;
use lazy_static::lazy_static;
use serde::Serialize;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::chrono::Utc;
use tardis::db::reldb_client::{IdResp, TardisActiveModel};
use tardis::db::sea_orm::sea_query::{Alias, Cond, Expr, Func, IntoValueTuple, JoinType, Order, Query, SelectStatement, Value, ValueTuple};
use tardis::db::sea_orm::{self, Condition, EntityTrait, FromQueryResult, QueryFilter, Select};
use tardis::regex::Regex;
use tardis::web::poem_openapi::types::{ParseFromJSON, ToJSON};
use tardis::web::web_resp::TardisPage;
use tardis::TardisFunsInst;

use crate::process::task_processor::TaskProcessor;
use crate::rbum::domain::rbum_item;
use crate::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use crate::rbum::helper::rbum_event_helper::RbumEventMessage;
use crate::rbum::helper::{rbum_event_helper, rbum_scope_helper};
use crate::rbum::rbum_config::RbumConfigApi;

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
    pub static ref R_URL_PART_CODE: Regex = Regex::new(r"^[a-z0-9-.]+$").expect("Regular parsing error");
}

#[async_trait]
pub trait RbumCrudOperation<E, AddReq, ModifyReq, SummaryResp, DetailResp, FilterReq>
where
    E: TardisActiveModel + Sync + Send,
    AddReq: Sync + Send,
    ModifyReq: Sync + Send,
    SummaryResp: FromQueryResult + ParseFromJSON + ToJSON + Serialize + Send + Sync,
    DetailResp: FromQueryResult + ParseFromJSON + ToJSON + Serialize + Send + Sync,
    FilterReq: Sync + Send,
{
    fn get_table_name() -> &'static str;

    fn get_obj_name() -> String {
        Self::get_obj_name_from(Self::get_table_name())
    }

    fn get_obj_name_from(table_name: &str) -> String {
        table_name.replace("rbum_", "")
    }

    // ----------------------------- Ownership -------------------------------

    async fn check_ownership(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        Self::check_ownership_with_table_name(id, Self::get_table_name(), funs, ctx).await
    }

    async fn check_ownership_with_table_name(id: &str, table_name: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if funs.db().count(&Self::package_ownership_query_with_table_name(id, table_name, ctx)).await? == 0 {
            return Err(funs.err().not_found(
                &Self::get_obj_name_from(table_name),
                "check",
                &format!("ownership {}.{} is illegal by {}", Self::get_obj_name_from(table_name), id, ctx.owner),
                "404-rbum-*-ownership-illegal",
            ));
        }
        Ok(())
    }

    fn package_ownership_query(id: &str, ctx: &TardisContext) -> SelectStatement {
        Self::package_ownership_query_with_table_name(id, Self::get_table_name(), ctx)
    }

    fn package_ownership_query_with_table_name(id: &str, table_name: &str, ctx: &TardisContext) -> SelectStatement {
        let mut query = Query::select();
        query
            .column(ID_FIELD.clone())
            .from(Alias::new(table_name))
            .and_where(Expr::col(ID_FIELD.clone()).eq(id))
            .and_where(Expr::col(OWN_PATHS_FIELD.clone()).like(format!("{}%", ctx.own_paths).as_str()));
        query
    }

    // ----------------------------- Scope -------------------------------

    async fn check_scope(id: &str, table_name: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if funs
            .db()
            .count(
                Query::select()
                    .column((Alias::new(table_name), ID_FIELD.clone()))
                    .from(Alias::new(table_name))
                    .and_where(Expr::tbl(Alias::new(table_name), ID_FIELD.clone()).eq(id))
                    .with_scope(table_name, &ctx.own_paths, false),
            )
            .await?
            == 0
        {
            return Err(funs.err().not_found(
                &Self::get_obj_name_from(table_name),
                "check",
                &format!("scope {}.{} is illegal by {}", Self::get_obj_name_from(table_name), id, ctx.owner),
                "404-rbum-*-scope-illegal",
            ));
        }
        Ok(())
    }

    async fn check_scopes(values: HashMap<String, &Vec<String>>, expect_number: u64, table_name: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let mut query = Query::select();
        let msg = values.iter().map(|(k, v)| format!("{}={:?}", k, v)).join(",");
        query.column((Alias::new(table_name), ID_FIELD.clone())).from(Alias::new(table_name)).with_scope(table_name, &ctx.own_paths, false);
        for (k, v) in values {
            query.and_where(Expr::tbl(Alias::new(table_name), Alias::new(&k)).is_in(v.clone()));
        }
        if funs.db().count(&query).await? != expect_number {
            return Err(funs.err().not_found(
                &Self::get_obj_name_from(table_name),
                "check",
                &format!("scopes {}.{} is illegal by {}", Self::get_obj_name_from(table_name), msg, ctx.owner),
                "404-rbum-*-scope-illegal",
            ));
        }
        Ok(())
    }

    // ----------------------------- Exist -------------------------------

    async fn check_exist_before_delete(id: &str, rel_table_name: &str, rel_field_name: &str, funs: &TardisFunsInst) -> TardisResult<()> {
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
            return Err(funs.err().conflict(
                &Self::get_obj_name(),
                "delete",
                &format!(
                    "can not delete {}.{} when there are associated by {}.{}",
                    Self::get_obj_name(),
                    id,
                    Self::get_obj_name_from(rel_table_name),
                    rel_field_name
                ),
                "409-rbum-*-delete-conflict",
            ));
        }
        Ok(())
    }

    async fn check_exist_with_cond_before_delete(rel_table_name: &str, condition: Condition, funs: &TardisFunsInst) -> TardisResult<()> {
        if funs.db().count(Query::select().column((Alias::new(rel_table_name), ID_FIELD.clone())).from(Alias::new(rel_table_name)).cond_where(condition)).await? > 0 {
            return Err(funs.err().conflict(
                &Self::get_obj_name(),
                "delete",
                &format!(
                    "can not delete {} when there are associated by {}",
                    Self::get_obj_name(),
                    Self::get_obj_name_from(rel_table_name)
                ),
                "409-rbum-*-delete-conflict",
            ));
        }
        Ok(())
    }

    // ----------------------------- Add -------------------------------

    async fn package_add(add_req: &AddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<E>;

    async fn before_add_rbum(_: &mut AddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    async fn after_add_rbum(_: &str, _: &AddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    async fn add_rbum(add_req: &mut AddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        Self::before_add_rbum(add_req, funs, ctx).await?;
        let domain = Self::package_add(add_req, funs, ctx).await?;
        let insert_result = funs.db().insert_one(domain, ctx).await?;
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
            Self::after_add_rbum(&id, add_req, funs, ctx).await?;
            TaskProcessor::add_notify_event(Self::get_table_name(), "c", id.as_str(), ctx)?;
            // rbum_event_helper::try_notify(Self::get_table_name(), "c", &id, funs, ctx).await?;
            Ok(id.to_string())
        } else {
            return Err(funs.err().internal_error(
                &Self::get_obj_name(),
                "add",
                "id data type is invalid, currently only the string is supported",
                "500-rbum-crud-id-type",
            ));
        }
    }

    // ----------------------------- Modify -------------------------------

    async fn package_modify(id: &str, modify_req: &ModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<E>;

    async fn before_modify_rbum(id: &str, _: &mut ModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        Self::check_ownership(id, funs, ctx).await
    }

    async fn after_modify_rbum(_: &str, _: &mut ModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    async fn modify_rbum(id: &str, modify_req: &mut ModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        Self::before_modify_rbum(id, modify_req, funs, ctx).await?;
        let domain = Self::package_modify(id, modify_req, funs, ctx).await?;
        funs.db().update_one(domain, ctx).await?;
        Self::after_modify_rbum(id, modify_req, funs, ctx).await?;
        TaskProcessor::add_notify_event(Self::get_table_name(), "u", id, ctx)?;
        // rbum_event_helper::try_notify(Self::get_table_name(), "u", id, funs, ctx).await?;
        Ok(())
    }

    // ----------------------------- Delete -------------------------------

    async fn package_delete(id: &str, _funs: &TardisFunsInst, _ctx: &TardisContext) -> TardisResult<Select<E::Entity>> {
        Ok(E::Entity::find().filter(Expr::col(ID_FIELD.clone()).eq(id)))
    }

    async fn before_delete_rbum(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<DetailResp>> {
        Self::check_ownership(id, funs, ctx).await?;
        Ok(None)
    }

    async fn after_delete_rbum(_: &str, _: &Option<DetailResp>, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    async fn delete_rbum(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<u64> {
        let deleted_rbum = Self::before_delete_rbum(id, funs, ctx).await?;
        let select = Self::package_delete(id, funs, ctx).await?;
        #[cfg(feature = "with-mq")]
        {
            let delete_records = funs.db().soft_delete_custom(select, "id").await?;
            let mq_topic_entity_deleted = &funs.rbum_conf_mq_topic_entity_deleted();
            let mq_header = std::collections::HashMap::from([(funs.rbum_conf_mq_header_name_operator(), ctx.owner.clone())]);
            for delete_record in &delete_records {
                funs.mq().publish(mq_topic_entity_deleted, tardis::TardisFuns::json.obj_to_string(delete_record)?, &mq_header).await?;
            }
            Self::after_delete_rbum(id, &deleted_rbum, funs, ctx).await?;
            TaskProcessor::add_notify_event(Self::get_table_name(), "d", id, ctx)?;
            // rbum_event_helper::try_notify(Self::get_table_name(), "d", id, funs, ctx).await?;
            Ok(delete_records.len() as u64)
        }
        #[cfg(not(feature = "with-mq"))]
        {
            let delete_records = funs.db().soft_delete(select, &ctx.owner).await?;
            Self::after_delete_rbum(id, &deleted_rbum, funs, ctx).await?;
            TaskProcessor::add_notify_event(Self::get_table_name(), "d", id, ctx)?;
            // rbum_event_helper::try_notify(Self::get_table_name(), "d", &id, funs, ctx).await?;
            Ok(delete_records)
        }
    }

    // ----------------------------- Query -------------------------------

    async fn package_query(is_detail: bool, filter: &FilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<SelectStatement>;

    async fn peek_rbum(id: &str, filter: &FilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<SummaryResp> {
        Self::do_peek_rbum(id, filter, funs, ctx).await
    }

    async fn do_peek_rbum(id: &str, filter: &FilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<SummaryResp> {
        let mut query = Self::package_query(false, filter, funs, ctx).await?;
        query.and_where(Expr::tbl(Alias::new(Self::get_table_name()), ID_FIELD.clone()).eq(id));
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

    async fn get_rbum(id: &str, filter: &FilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<DetailResp> {
        Self::do_get_rbum(id, filter, funs, ctx).await
    }

    async fn do_get_rbum(id: &str, filter: &FilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<DetailResp> {
        let mut query = Self::package_query(true, filter, funs, ctx).await?;
        query.and_where(Expr::tbl(Alias::new(Self::get_table_name()), ID_FIELD.clone()).eq(id));
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

    async fn paginate_id_rbums(
        filter: &FilterReq,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<String>> {
        Self::do_paginate_id_rbums(filter, page_number, page_size, desc_sort_by_create, desc_sort_by_update, funs, ctx).await
    }

    async fn do_paginate_id_rbums(
        filter: &FilterReq,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<String>> {
        let mut query = Self::package_query(false, filter, funs, ctx).await?;
        if let Some(sort) = desc_sort_by_create {
            query.order_by((Alias::new(Self::get_table_name()), CREATE_TIME_FIELD.clone()), if sort { Order::Desc } else { Order::Asc });
        }
        if let Some(sort) = desc_sort_by_update {
            query.order_by((Alias::new(Self::get_table_name()), UPDATE_TIME_FIELD.clone()), if sort { Order::Desc } else { Order::Asc });
        }
        let (records, total_size) = funs.db().paginate_dtos::<IdResp>(&query, page_number, page_size).await?;
        Ok(TardisPage {
            page_size,
            page_number,
            total_size,
            records: records.into_iter().map(|resp| resp.id).collect(),
        })
    }

    async fn paginate_rbums(
        filter: &FilterReq,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<SummaryResp>> {
        Self::do_paginate_rbums(filter, page_number, page_size, desc_sort_by_create, desc_sort_by_update, funs, ctx).await
    }

    async fn do_paginate_rbums(
        filter: &FilterReq,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<SummaryResp>> {
        let mut query = Self::package_query(false, filter, funs, ctx).await?;
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

    async fn paginate_detail_rbums(
        filter: &FilterReq,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<DetailResp>> {
        Self::do_paginate_detail_rbums(filter, page_number, page_size, desc_sort_by_create, desc_sort_by_update, funs, ctx).await
    }

    async fn do_paginate_detail_rbums(
        filter: &FilterReq,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<DetailResp>> {
        let mut query = Self::package_query(true, filter, funs, ctx).await?;
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

    async fn find_one_rbum(filter: &FilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<SummaryResp>> {
        Self::do_find_one_rbum(filter, funs, ctx).await
    }

    async fn do_find_one_rbum(filter: &FilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<SummaryResp>> {
        let result = Self::find_rbums(filter, None, None, funs, ctx).await?;
        if result.len() > 1 {
            Err(funs.err().conflict(&Self::get_obj_name(), "find_one", "found multiple records", "409-rbum-*-obj-multi-exist"))
        } else {
            Ok(result.into_iter().next())
        }
    }

    async fn find_id_rbums(
        filter: &FilterReq,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<String>> {
        Self::do_find_id_rbums(filter, desc_sort_by_create, desc_sort_by_update, funs, ctx).await
    }

    async fn do_find_id_rbums(
        filter: &FilterReq,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<String>> {
        let mut query = Self::package_query(false, filter, funs, ctx).await?;
        if let Some(sort) = desc_sort_by_create {
            query.order_by((Alias::new(Self::get_table_name()), CREATE_TIME_FIELD.clone()), if sort { Order::Desc } else { Order::Asc });
        }
        if let Some(sort) = desc_sort_by_update {
            query.order_by((Alias::new(Self::get_table_name()), UPDATE_TIME_FIELD.clone()), if sort { Order::Desc } else { Order::Asc });
        }
        Ok(funs.db().find_dtos::<IdResp>(&query).await?.into_iter().map(|resp| resp.id).collect())
    }

    async fn find_rbums(
        filter: &FilterReq,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<SummaryResp>> {
        Self::do_find_rbums(filter, desc_sort_by_create, desc_sort_by_update, funs, ctx).await
    }

    async fn do_find_rbums(
        filter: &FilterReq,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<SummaryResp>> {
        let mut query = Self::package_query(false, filter, funs, ctx).await?;
        if let Some(sort) = desc_sort_by_create {
            query.order_by((Alias::new(Self::get_table_name()), CREATE_TIME_FIELD.clone()), if sort { Order::Desc } else { Order::Asc });
        }
        if let Some(sort) = desc_sort_by_update {
            query.order_by((Alias::new(Self::get_table_name()), UPDATE_TIME_FIELD.clone()), if sort { Order::Desc } else { Order::Asc });
        }
        Ok(funs.db().find_dtos(&query).await?)
    }

    async fn find_one_detail_rbum(filter: &FilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<DetailResp>> {
        Self::do_find_one_detail_rbum(filter, funs, ctx).await
    }

    async fn do_find_one_detail_rbum(filter: &FilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<DetailResp>> {
        let result = Self::find_detail_rbums(filter, None, None, funs, ctx).await?;
        if result.len() > 1 {
            Err(funs.err().conflict(&Self::get_obj_name(), "find_one_detail", "found multiple records", "409-rbum-*-obj-multi-exist"))
        } else {
            Ok(result.into_iter().next())
        }
    }

    async fn find_detail_rbums(
        filter: &FilterReq,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<DetailResp>> {
        Self::do_find_detail_rbums(filter, desc_sort_by_create, desc_sort_by_update, funs, ctx).await
    }

    async fn do_find_detail_rbums(
        filter: &FilterReq,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<DetailResp>> {
        let mut query = Self::package_query(true, filter, funs, ctx).await?;
        if let Some(sort) = desc_sort_by_create {
            query.order_by((Alias::new(Self::get_table_name()), CREATE_TIME_FIELD.clone()), if sort { Order::Desc } else { Order::Asc });
        }
        if let Some(sort) = desc_sort_by_update {
            query.order_by((Alias::new(Self::get_table_name()), UPDATE_TIME_FIELD.clone()), if sort { Order::Desc } else { Order::Asc });
        }
        Ok(funs.db().find_dtos(&query).await?)
    }

    async fn count_rbums(filter: &FilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<u64> {
        Self::do_count_rbums(filter, funs, ctx).await
    }

    async fn do_count_rbums(filter: &FilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<u64> {
        let query = Self::package_query(false, filter, funs, ctx).await?;
        funs.db().count(&query).await
    }

    async fn exist_rbum(filter: &FilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
        let query = Self::count_rbums(filter, funs, ctx).await?;
        Ok(query > 0)
    }
}

pub trait RbumCrudQueryPackage {
    fn with_filter(&mut self, table_name: &str, filter: &RbumBasicFilterReq, ignore_owner: bool, has_scope: bool, ctx: &TardisContext) -> &mut Self;
    fn with_scope(&mut self, table_name: &str, filter_own_paths: &str, with_sub_own_paths: bool) -> &mut Self;
}

impl RbumCrudQueryPackage for SelectStatement {
    fn with_filter(&mut self, table_name: &str, filter: &RbumBasicFilterReq, with_owner: bool, has_scope: bool, ctx: &TardisContext) -> &mut Self {
        if filter.rel_ctx_owner {
            self.and_where(Expr::tbl(Alias::new(table_name), OWNER_FIELD.clone()).eq(ctx.owner.as_str()));
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
                JoinType::LeftJoin,
                rbum_item::Entity,
                OWNER_TABLE.clone(),
                Expr::tbl(OWNER_TABLE.clone(), ID_FIELD.clone()).equals(Alias::new(table_name), OWNER_FIELD.clone()),
            );
        }
        let filter_own_paths = if let Some(own_paths) = &filter.own_paths { own_paths.as_str() } else { &ctx.own_paths };
        if has_scope && !filter.ignore_scope {
            self.with_scope(table_name, filter_own_paths, filter.with_sub_own_paths);
        } else if filter.with_sub_own_paths {
            self.and_where(Expr::tbl(Alias::new(table_name), OWN_PATHS_FIELD.clone()).like(format!("{}%", filter_own_paths).as_str()));
        } else {
            self.and_where(Expr::tbl(Alias::new(table_name), OWN_PATHS_FIELD.clone()).eq(filter_own_paths));
        }
        if let Some(desc_by_sort) = filter.desc_by_sort {
            self.order_by((Alias::new(table_name), SORT_FIELD.clone()), if desc_by_sort { Order::Desc } else { Order::Asc });
        }
        self
    }

    fn with_scope(&mut self, table_name: &str, filter_own_paths: &str, with_sub_own_paths: bool) -> &mut Self {
        let mut cond = Cond::any().add(Expr::tbl(Alias::new(table_name), SCOPE_LEVEL_FIELD.clone()).eq(0));

        let own_cond = if with_sub_own_paths {
            Expr::tbl(Alias::new(table_name), OWN_PATHS_FIELD.clone()).like(format!("{}%", filter_own_paths))
        } else {
            Expr::tbl(Alias::new(table_name), OWN_PATHS_FIELD.clone()).eq(filter_own_paths)
        };
        cond = cond.add(own_cond);

        if let Some(p1) = rbum_scope_helper::get_pre_paths(1, filter_own_paths) {
            cond = cond.add(
                Cond::all().add(Expr::tbl(Alias::new(table_name), SCOPE_LEVEL_FIELD.clone()).eq(1)).add(
                    Cond::any()
                        .add(Expr::tbl(Alias::new(table_name), OWN_PATHS_FIELD.clone()).eq(""))
                        .add(Expr::tbl(Alias::new(table_name), OWN_PATHS_FIELD.clone()).like(format!("{}%", p1))),
                ),
            );
            if let Some(p2) = rbum_scope_helper::get_pre_paths(2, filter_own_paths) {
                let node_len = (p2.len() - p1.len() - 1) as u8;
                cond = cond.add(
                    Cond::all().add(Expr::tbl(Alias::new(table_name), SCOPE_LEVEL_FIELD.clone()).eq(2)).add(
                        Cond::any()
                            .add(Expr::tbl(Alias::new(table_name), OWN_PATHS_FIELD.clone()).eq(""))
                            .add(
                                Cond::all()
                                    .add(Expr::expr(Func::char_length(Expr::tbl(Alias::new(table_name), OWN_PATHS_FIELD.clone()))).eq(node_len))
                                    .add(Expr::tbl(Alias::new(table_name), OWN_PATHS_FIELD.clone()).like(format!("{}%", p1))),
                            )
                            .add(Expr::tbl(Alias::new(table_name), OWN_PATHS_FIELD.clone()).like(format!("{}%", p2))),
                    ),
                );
                if let Some(p3) = rbum_scope_helper::get_pre_paths(3, filter_own_paths) {
                    cond = cond.add(
                        Cond::all().add(Expr::tbl(Alias::new(table_name), SCOPE_LEVEL_FIELD.clone()).eq(3)).add(
                            Cond::any()
                                .add(Expr::tbl(Alias::new(table_name), OWN_PATHS_FIELD.clone()).eq(""))
                                .add(
                                    Cond::all()
                                        .add(Expr::expr(Func::char_length(Expr::tbl(Alias::new(table_name), OWN_PATHS_FIELD.clone()))).eq(node_len))
                                        .add(Expr::tbl(Alias::new(table_name), OWN_PATHS_FIELD.clone()).like(format!("{}%", p1))),
                                )
                                .add(
                                    Cond::all()
                                        .add(Expr::expr(Func::char_length(Expr::tbl(Alias::new(table_name), OWN_PATHS_FIELD.clone()))).eq(node_len * 2 + 1))
                                        .add(Expr::tbl(Alias::new(table_name), OWN_PATHS_FIELD.clone()).like(format!("{}%", p2))),
                                )
                                .add(Expr::tbl(Alias::new(table_name), OWN_PATHS_FIELD.clone()).like(format!("{}%", p3))),
                        ),
                    );
                } else if with_sub_own_paths {
                    // System admin (own_paths = "") created Tenant admin (scope_level = 1 & own_paths = "") and App admin (scope_level = 2 & own_paths = "").
                    //
                    // A tenant admin needs to query the roles under that tenant and app, the corresponding condition should be (with_sub_own_paths = true):
                    //
                    // ```sql
                    // scope_level = 0
                    // OR own_paths LIKE '<tenant_id>%'
                    // OR (scope_level = 1 AND (own_paths = '' OR own_paths LIKE '<tenant_id>%'))
                    // OR (scope_level = 2 AND (own_paths = '' OR own_paths LIKE '<tenant_id>%'))
                    // ```
                    cond = cond.add(
                        Cond::all().add(Expr::tbl(Alias::new(table_name), SCOPE_LEVEL_FIELD.clone()).eq(3)).add(
                            Cond::any()
                                .add(Expr::tbl(Alias::new(table_name), OWN_PATHS_FIELD.clone()).eq(""))
                                .add(
                                    Cond::all()
                                        .add(Expr::expr(Func::char_length(Expr::tbl(Alias::new(table_name), OWN_PATHS_FIELD.clone()))).eq(node_len))
                                        .add(Expr::tbl(Alias::new(table_name), OWN_PATHS_FIELD.clone()).like(format!("{}%", p1))),
                                )
                                .add(Expr::tbl(Alias::new(table_name), OWN_PATHS_FIELD.clone()).like(format!("{}%", p2))),
                        ),
                    );
                }
            } else if with_sub_own_paths {
                cond = cond.add(
                    Cond::all().add(Expr::tbl(Alias::new(table_name), SCOPE_LEVEL_FIELD.clone()).eq(2)).add(
                        Cond::any()
                            .add(Expr::tbl(Alias::new(table_name), OWN_PATHS_FIELD.clone()).eq(""))
                            .add(Expr::tbl(Alias::new(table_name), OWN_PATHS_FIELD.clone()).like(format!("{}%", p1))),
                    ),
                );
            }
        };

        self.cond_where(Cond::all().add(cond));
        self
    }
}

#[derive(Debug, sea_orm::FromQueryResult)]
pub struct NameResp {
    pub name: String,
}
