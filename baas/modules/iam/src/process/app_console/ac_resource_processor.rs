/*
 * Copyright 2022. the original author or authors.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use actix_web::{delete, get, post, put, HttpRequest};
use sea_query::{Alias, Cond, Expr, JoinType, Order, Query};
use sqlx::Connection;
use strum::IntoEnumIterator;

use bios::basic::dto::BIOSResp;
use bios::db::reldb_client::SqlBuilderProcess;
use bios::web::basic_processor::extract_context_with_account;
use bios::web::resp_handler::BIOSResponse;
use bios::web::validate::json::Json;
use bios::web::validate::query::Query as VQuery;
use bios::BIOSFuns;

use crate::domain::auth_domain::{IamAuthPolicy, IamResource, IamResourceSubject};
use crate::domain::ident_domain::IamAccount;
use crate::iam_constant::{IamOutput, ObjectKind};
use crate::process::app_console::ac_resource_dto::{
    ResourceAddReq, ResourceDetailResp, ResourceModifyReq, ResourceQueryReq, ResourceSubjectAddReq, ResourceSubjectDetailResp, ResourceSubjectModifyReq, ResourceSubjectQueryReq,
};

#[post("/console/app/resource/subject")]
pub async fn add_resource_subject(resource_subject_add_req: Json<ResourceSubjectAddReq>, req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;
    let ident_uri = bios::basic::uri::format(&resource_subject_add_req.ident_uri)?;
    if BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamResourceSubject::Id])
                .from(IamResourceSubject::Table)
                .and_where(Expr::col(IamResourceSubject::Kind).eq(resource_subject_add_req.kind.to_string().to_lowercase()))
                .and_where(Expr::col(IamResourceSubject::IdentUri).eq(ident_uri.as_str()))
                .and_where(Expr::col(IamResourceSubject::RelAppId).eq(context.ident.app_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSResp::err(IamOutput::AppConsoleEntityCreateCheckExists(ObjectKind::ResourceSubject, "ResourceSubject"), Some(&context));
    }
    let id = bios::basic::field::uuid();
    BIOSFuns::reldb()
        .exec(
            &Query::insert()
                .into_table(IamResourceSubject::Table)
                .columns(vec![
                    IamResourceSubject::Id,
                    IamResourceSubject::CreateUser,
                    IamResourceSubject::UpdateUser,
                    IamResourceSubject::Kind,
                    IamResourceSubject::IdentUri,
                    IamResourceSubject::Name,
                    IamResourceSubject::Sort,
                    IamResourceSubject::Uri,
                    IamResourceSubject::Ak,
                    IamResourceSubject::Sk,
                    IamResourceSubject::PlatformAccount,
                    IamResourceSubject::PlatformProjectId,
                    IamResourceSubject::TimeoutMs,
                    IamResourceSubject::RelAppId,
                    IamResourceSubject::RelTenantId,
                ])
                .values_panic(vec![
                    id.as_str().into(),
                    context.ident.account_id.as_str().into(),
                    context.ident.account_id.as_str().into(),
                    resource_subject_add_req.kind.to_string().to_lowercase().into(),
                    ident_uri.into(),
                    resource_subject_add_req.name.as_str().into(),
                    resource_subject_add_req.sort.into(),
                    resource_subject_add_req.uri.as_deref().unwrap_or_default().into(),
                    resource_subject_add_req.ak.as_deref().unwrap_or_default().into(),
                    resource_subject_add_req.sk.as_deref().unwrap_or_default().into(),
                    resource_subject_add_req.platform_account.as_deref().unwrap_or_default().into(),
                    resource_subject_add_req.platform_project_id.as_deref().unwrap_or_default().into(),
                    resource_subject_add_req.timeout_ms.unwrap_or_default().into(),
                    context.ident.app_id.as_str().into(),
                    context.ident.tenant_id.as_str().into(),
                ])
                .done(),
            None,
        )
        .await?;
    BIOSResp::ok(id, Some(&context))
}

#[put("/console/app/resource/subject/{id}")]
pub async fn modify_resource_subject(resource_subject_modify_req: Json<ResourceSubjectModifyReq>, req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;
    let id: String = req.match_info().get("id").unwrap().parse()?;

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamResourceSubject::Id])
                .from(IamResourceSubject::Table)
                .and_where(Expr::col(IamResourceSubject::Id).eq(id.as_str()))
                .and_where(Expr::col(IamResourceSubject::RelAppId).eq(context.ident.app_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSResp::err(
            IamOutput::AppConsoleEntityModifyCheckNotFound(ObjectKind::ResourceSubject, "ResourceSubject"),
            Some(&context),
        );
    }

    if resource_subject_modify_req.kind.is_some() && resource_subject_modify_req.ident_uri.is_none()
        || resource_subject_modify_req.kind.is_none() && resource_subject_modify_req.ident_uri.is_some()
    {
        return BIOSResp::err(
            IamOutput::AppConsoleEntityModifyCheckExistFieldsAtSomeTime(ObjectKind::ResourceSubject, "[kin] and [ident_uri]"),
            Some(&context),
        );
    }

    let mut values = Vec::new();
    if resource_subject_modify_req.kind.is_some() && resource_subject_modify_req.ident_uri.is_some() {
        let kind = resource_subject_modify_req.kind.as_ref().unwrap().to_string().to_lowercase();
        let ident_uri = bios::basic::uri::format(resource_subject_modify_req.ident_uri.as_ref().unwrap())?;
        if BIOSFuns::reldb()
            .exists(
                &Query::select()
                    .columns(vec![IamResourceSubject::Id])
                    .from(IamResourceSubject::Table)
                    .and_where(Expr::col(IamResourceSubject::Kind).eq(kind.as_str()))
                    .and_where(Expr::col(IamResourceSubject::IdentUri).eq(ident_uri.as_str()))
                    .and_where(Expr::col(IamResourceSubject::RelAppId).eq(context.ident.app_id.as_str()))
                    .and_where(Expr::col(IamResourceSubject::Id).ne(id.as_str()))
                    .done(),
                None,
            )
            .await?
        {
            return BIOSResp::err(IamOutput::AppConsoleEntityModifyCheckExists(ObjectKind::ResourceSubject, "ResourceSubject"), Some(&context));
        }
        values.push((IamResourceSubject::Kind, kind.into()));
        values.push((IamResourceSubject::Uri, ident_uri.into()));
    }
    if let Some(name) = &resource_subject_modify_req.name {
        values.push((IamResourceSubject::Name, name.as_str().into()));
    }
    if let Some(sort) = resource_subject_modify_req.sort {
        values.push((IamResourceSubject::Sort, sort.into()));
    }
    if let Some(uri) = &resource_subject_modify_req.uri {
        values.push((IamResourceSubject::Uri, uri.as_str().into()));
    }
    if let Some(ak) = &resource_subject_modify_req.ak {
        values.push((IamResourceSubject::Ak, ak.as_str().into()));
    }
    if let Some(sk) = &resource_subject_modify_req.sk {
        values.push((IamResourceSubject::Sk, sk.as_str().into()));
    }
    if let Some(platform_project_id) = &resource_subject_modify_req.platform_project_id {
        values.push((IamResourceSubject::PlatformProjectId, platform_project_id.as_str().into()));
    }
    if let Some(platform_account) = &resource_subject_modify_req.platform_account {
        values.push((IamResourceSubject::PlatformAccount, platform_account.as_str().into()));
    }
    if let Some(platform_project_id) = &resource_subject_modify_req.platform_project_id {
        values.push((IamResourceSubject::PlatformProjectId, platform_project_id.as_str().into()));
    }
    if let Some(timeout_ms) = resource_subject_modify_req.timeout_ms {
        values.push((IamResourceSubject::TimeoutMs, timeout_ms.into()));
    }
    values.push((IamResourceSubject::UpdateUser, context.ident.account_id.as_str().into()));
    let sql_builder = Query::update()
        .table(IamResourceSubject::Table)
        .values(values)
        .and_where(Expr::col(IamResourceSubject::Id).eq(id.as_str()))
        .and_where(Expr::col(IamResourceSubject::RelAppId).eq(context.ident.app_id.as_str()))
        .done();
    BIOSFuns::reldb().exec(&sql_builder, None).await?;
    BIOSResp::ok("", Some(&context))
}

#[get("/console/app/resource/subject")]
pub async fn list_resource_subject(query: VQuery<ResourceSubjectQueryReq>, req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;

    let create_user_table = Alias::new("create");
    let update_user_table = Alias::new("update");
    let sql_builder = Query::select()
        .columns(vec![
            (IamResourceSubject::Table, IamResourceSubject::Id),
            (IamResourceSubject::Table, IamResourceSubject::CreateTime),
            (IamResourceSubject::Table, IamResourceSubject::UpdateTime),
            (IamResourceSubject::Table, IamResourceSubject::Kind),
            (IamResourceSubject::Table, IamResourceSubject::IdentUri),
            (IamResourceSubject::Table, IamResourceSubject::Name),
            (IamResourceSubject::Table, IamResourceSubject::Sort),
            (IamResourceSubject::Table, IamResourceSubject::Uri),
            (IamResourceSubject::Table, IamResourceSubject::Ak),
            (IamResourceSubject::Table, IamResourceSubject::Sk),
            (IamResourceSubject::Table, IamResourceSubject::PlatformAccount),
            (IamResourceSubject::Table, IamResourceSubject::PlatformProjectId),
            (IamResourceSubject::Table, IamResourceSubject::TimeoutMs),
            (IamResourceSubject::Table, IamResourceSubject::RelAppId),
            (IamResourceSubject::Table, IamResourceSubject::RelTenantId),
        ])
        .expr_as(Expr::tbl(create_user_table.clone(), IamAccount::Name), Alias::new("create_user"))
        .expr_as(Expr::tbl(update_user_table.clone(), IamAccount::Name), Alias::new("update_user"))
        .from(IamResourceSubject::Table)
        .join_as(
            JoinType::InnerJoin,
            IamAccount::Table,
            create_user_table.clone(),
            Expr::tbl(create_user_table, IamAccount::Id).equals(IamResourceSubject::Table, IamResourceSubject::CreateUser),
        )
        .join_as(
            JoinType::InnerJoin,
            IamAccount::Table,
            update_user_table.clone(),
            Expr::tbl(update_user_table, IamAccount::Id).equals(IamResourceSubject::Table, IamResourceSubject::UpdateUser),
        )
        .and_where_option(if query.name.as_ref().is_some() {
            Some(Expr::tbl(IamResourceSubject::Table, IamResourceSubject::Name).like(format!("%{}%", query.name.as_ref().unwrap()).as_str()))
        } else {
            None
        })
        .and_where(Expr::tbl(IamResourceSubject::Table, IamResourceSubject::RelAppId).eq(context.ident.app_id.as_str()))
        .order_by(IamResourceSubject::UpdateTime, Order::Desc)
        .done();
    let items = BIOSFuns::reldb().pagination::<ResourceSubjectDetailResp>(&sql_builder, query.page_number, query.page_size, None).await?;
    BIOSResp::ok(items, Some(&context))
}

#[delete("/console/app/resource/subject/{id}")]
pub async fn delete_resource_subject(req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;
    let id: String = req.match_info().get("id").unwrap().parse()?;

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamResourceSubject::Id])
                .from(IamResourceSubject::Table)
                .and_where(Expr::col(IamResourceSubject::Id).eq(id.as_str()))
                .and_where(Expr::col(IamResourceSubject::RelAppId).eq(context.ident.app_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSResp::err(
            IamOutput::AppConsoleEntityDeleteCheckNotFound(ObjectKind::ResourceSubject, "ResourceSubject"),
            Some(&context),
        );
    }
    if BIOSFuns::reldb()
        .exists(
            &Query::select().columns(vec![IamResource::Id]).from(IamResource::Table).and_where(Expr::col(IamResource::RelResourceSubjectId).eq(id.as_str())).done(),
            None,
        )
        .await?
    {
        return BIOSResp::err(
            IamOutput::AppConsoleEntityDeleteCheckExistAssociatedData(ObjectKind::ResourceSubject, "Resource"),
            Some(&context),
        );
    }
    let mut conn = BIOSFuns::reldb().conn().await;
    let mut tx = conn.begin().await?;

    let sql_builder = Query::select()
        .columns(IamResourceSubject::iter().filter(|i| *i != IamResourceSubject::Table))
        .from(IamResourceSubject::Table)
        .and_where(Expr::col(IamResourceSubject::Id).eq(id.as_str()))
        .and_where(Expr::col(IamResourceSubject::RelAppId).eq(context.ident.app_id.as_str()))
        .done();
    BIOSFuns::reldb().soft_del(IamResourceSubject::Table, IamResourceSubject::Id, &context.ident.account_id, &sql_builder, &mut tx).await?;

    tx.commit().await?;
    BIOSResp::ok("", Some(&context))
}

// ------------------------------------

#[post("/console/app/resource")]
pub async fn add_resource(resource_add_req: Json<ResourceAddReq>, req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;
    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamResourceSubject::Id])
                .from(IamResourceSubject::Table)
                .and_where(Expr::col(IamResourceSubject::Id).eq(resource_add_req.rel_resource_subject_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSResp::err(IamOutput::AppConsoleEntityCreateCheckNotFound(ObjectKind::Resource, "ResourceSubject"), Some(&context));
    }
    if BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamResource::Id])
                .from(IamResource::Table)
                .and_where(Expr::col(IamResource::PathAndQuery).eq(resource_add_req.path_and_query.as_str()))
                .and_where(Expr::col(IamResource::RelResourceSubjectId).eq(resource_add_req.rel_resource_subject_id.as_str()))
                .and_where(Expr::col(IamResource::RelAppId).eq(context.ident.app_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSResp::err(IamOutput::AppConsoleEntityCreateCheckExists(ObjectKind::Resource, "Resource"), Some(&context));
    }
    let id = bios::basic::field::uuid();
    BIOSFuns::reldb()
        .exec(
            &Query::insert()
                .into_table(IamResource::Table)
                .columns(vec![
                    IamResource::Id,
                    IamResource::CreateUser,
                    IamResource::UpdateUser,
                    IamResource::PathAndQuery,
                    IamResource::Name,
                    IamResource::Icon,
                    IamResource::Sort,
                    IamResource::Action,
                    IamResource::ResGroup,
                    IamResource::ParentId,
                    IamResource::RelResourceSubjectId,
                    IamResource::RelAppId,
                    IamResource::RelTenantId,
                    IamResource::ExposeKind,
                ])
                .values_panic(vec![
                    id.as_str().into(),
                    context.ident.account_id.as_str().into(),
                    context.ident.account_id.as_str().into(),
                    resource_add_req.path_and_query.as_str().to_lowercase().into(),
                    resource_add_req.name.as_str().into(),
                    resource_add_req.icon.as_str().into(),
                    resource_add_req.sort.into(),
                    resource_add_req.action.as_deref().unwrap_or_default().into(),
                    resource_add_req.res_group.into(),
                    resource_add_req.parent_id.as_deref().unwrap_or_default().into(),
                    resource_add_req.rel_resource_subject_id.as_str().into(),
                    context.ident.app_id.as_str().into(),
                    context.ident.tenant_id.as_str().into(),
                    resource_add_req.expose_kind.as_ref().unwrap_or(&crate::process::basic_dto::ExposeKind::App).to_string().to_lowercase().into(),
                ])
                .done(),
            None,
        )
        .await?;
    BIOSResp::ok(id, Some(&context))
}

#[put("/console/app/resource/{id}")]
pub async fn modify_resource(resource_modify_req: Json<ResourceModifyReq>, req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;
    let id: String = req.match_info().get("id").unwrap().parse()?;

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamResource::Id])
                .from(IamResource::Table)
                .and_where(Expr::col(IamResource::Id).eq(id.as_str()))
                .and_where(Expr::col(IamResource::RelAppId).eq(context.ident.app_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSResp::err(IamOutput::AppConsoleEntityModifyCheckNotFound(ObjectKind::Resource, "Resource"), Some(&context));
    }
    if let Some(path_and_query) = &resource_modify_req.path_and_query {
        let resource_subject_id_info = BIOSFuns::reldb()
            .fetch_one_json(
                &Query::select()
                    .column((IamResourceSubject::Table, IamResourceSubject::Id))
                    .from(IamResource::Table)
                    .inner_join(
                        IamResourceSubject::Table,
                        Expr::tbl(IamResourceSubject::Table, IamResourceSubject::Id).equals(IamResource::Table, IamResource::RelResourceSubjectId),
                    )
                    .and_where(Expr::tbl(IamResourceSubject::Table, IamResourceSubject::RelAppId).eq(context.ident.app_id.as_str()))
                    .and_where(Expr::tbl(IamResource::Table, IamResource::RelAppId).eq(context.ident.app_id.as_str()))
                    .and_where(Expr::tbl(IamResource::Table, IamResource::Id).eq(id.as_str()))
                    .done(),
                None,
            )
            .await?;
        if BIOSFuns::reldb()
            .exists(
                &Query::select()
                    .columns(vec![IamResource::Id])
                    .from(IamResource::Table)
                    .and_where(Expr::col(IamResource::Id).ne(id.as_str()))
                    .and_where(Expr::col(IamResource::PathAndQuery).eq(path_and_query.to_string().to_lowercase()))
                    .and_where(Expr::col(IamResource::RelResourceSubjectId).eq(resource_subject_id_info["id"].as_str().unwrap()))
                    .and_where(Expr::col(IamResource::RelAppId).eq(context.ident.app_id.as_str()))
                    .done(),
                None,
            )
            .await?
        {
            return BIOSResp::err(IamOutput::AppConsoleEntityModifyCheckExists(ObjectKind::Resource, "Resource"), Some(&context));
        }
    }
    if let Some(parent_id) = &resource_modify_req.parent_id {
        if !BIOSFuns::reldb()
            .exists(
                &Query::select()
                    .columns(vec![IamResource::Id])
                    .from(IamResource::Table)
                    .and_where(Expr::col(IamResource::Id).ne(parent_id.to_string().as_str()))
                    .and_where(Expr::col(IamResource::RelAppId).eq(context.ident.app_id.as_str()))
                    .done(),
                None,
            )
            .await?
        {
            return BIOSResp::err(IamOutput::AppConsoleEntityModifyCheckExists(ObjectKind::Resource, "Resource"), Some(&context));
        }
    }
    let mut values = Vec::new();
    if let Some(path_and_query) = &resource_modify_req.path_and_query {
        values.push((IamResource::PathAndQuery, path_and_query.to_string().to_lowercase().into()));
    }
    if let Some(name) = &resource_modify_req.name {
        values.push((IamResource::Name, name.as_str().into()));
    }
    if let Some(icon) = &resource_modify_req.icon {
        values.push((IamResource::Icon, icon.as_str().into()));
    }
    if let Some(action) = &resource_modify_req.action {
        values.push((IamResource::Action, action.as_str().into()));
    }
    if let Some(sort) = resource_modify_req.sort {
        values.push((IamResource::Sort, sort.into()));
    }
    if let Some(res_group) = resource_modify_req.res_group {
        values.push((IamResource::ResGroup, res_group.into()));
    }
    if let Some(parent_id) = &resource_modify_req.parent_id {
        values.push((IamResource::ParentId, parent_id.as_str().into()));
    }
    if let Some(expose_kind) = &resource_modify_req.expose_kind {
        values.push((IamResource::ExposeKind, expose_kind.to_string().to_lowercase().into()));
    }
    values.push((IamResource::UpdateUser, context.ident.account_id.as_str().into()));
    BIOSFuns::reldb()
        .exec(
            &Query::update()
                .table(IamResource::Table)
                .values(values)
                .and_where(Expr::col(IamResource::Id).eq(id.as_str()))
                .and_where(Expr::col(IamResource::RelAppId).eq(context.ident.app_id.as_str()))
                .done(),
            None,
        )
        .await?;
    BIOSResp::ok("", Some(&context))
}

#[get("/console/app/resource")]
pub async fn list_resource(query: VQuery<ResourceQueryReq>, req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;

    let create_user_table = Alias::new("create");
    let update_user_table = Alias::new("update");
    let sql_builder = Query::select()
        .columns(vec![
            (IamResource::Table, IamResource::Id),
            (IamResource::Table, IamResource::CreateTime),
            (IamResource::Table, IamResource::UpdateTime),
            (IamResource::Table, IamResource::PathAndQuery),
            (IamResource::Table, IamResource::Name),
            (IamResource::Table, IamResource::Icon),
            (IamResource::Table, IamResource::Sort),
            (IamResource::Table, IamResource::Action),
            (IamResource::Table, IamResource::ResGroup),
            (IamResource::Table, IamResource::ParentId),
            (IamResource::Table, IamResource::RelResourceSubjectId),
            (IamResource::Table, IamResource::RelAppId),
            (IamResource::Table, IamResource::RelTenantId),
            (IamResource::Table, IamResource::ExposeKind),
        ])
        .expr_as(Expr::tbl(create_user_table.clone(), IamAccount::Name), Alias::new("create_user"))
        .expr_as(Expr::tbl(update_user_table.clone(), IamAccount::Name), Alias::new("update_user"))
        .from(IamResource::Table)
        .join_as(
            JoinType::InnerJoin,
            IamAccount::Table,
            create_user_table.clone(),
            Expr::tbl(create_user_table, IamAccount::Id).equals(IamResource::Table, IamResource::CreateUser),
        )
        .join_as(
            JoinType::InnerJoin,
            IamAccount::Table,
            update_user_table.clone(),
            Expr::tbl(update_user_table, IamAccount::Id).equals(IamResource::Table, IamResource::UpdateUser),
        )
        .and_where_option(if let Some(name) = &query.name {
            Some(Expr::tbl(IamResource::Table, IamResource::Name).like(format!("%{}%", name).as_str()))
        } else {
            None
        })
        .and_where_option(if let Some(path_and_query) = &query.path_and_query {
            Some(Expr::tbl(IamResource::Table, IamResource::PathAndQuery).like(format!("%{}%", path_and_query).as_str()))
        } else {
            None
        })
        .conditions(
            !query.expose,
            |x| {
                x.and_where(Expr::tbl(IamResource::Table, IamResource::RelAppId).eq(context.ident.app_id.as_str()));
            },
            |x| {
                x.cond_where(
                    Cond::any().add(Expr::tbl(IamResource::Table, IamResource::ExposeKind).eq(crate::process::basic_dto::ExposeKind::Global.to_string().to_lowercase())).add(
                        Cond::all()
                            .add(Expr::tbl(IamResource::Table, IamResource::RelTenantId).eq(context.ident.tenant_id.as_str()))
                            .add(Expr::tbl(IamResource::Table, IamResource::ExposeKind).eq(crate::process::basic_dto::ExposeKind::Tenant.to_string().to_lowercase())),
                    ),
                );
            },
        )
        .order_by(IamResource::UpdateTime, Order::Desc)
        .done();
    let items = BIOSFuns::reldb().pagination::<ResourceDetailResp>(&sql_builder, query.page_number, query.page_size, None).await?;
    BIOSResp::ok(items, Some(&context))
}

#[delete("/console/app/resource/{id}")]
pub async fn delete_resource(req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;
    let id: String = req.match_info().get("id").unwrap().parse()?;

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamResource::Id])
                .from(IamResource::Table)
                .and_where(Expr::col(IamResource::Id).eq(id.as_str()))
                .and_where(Expr::col(IamResource::RelAppId).eq(context.ident.app_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSResp::err(IamOutput::AppConsoleEntityDeleteCheckNotFound(ObjectKind::Resource, "Resource"), Some(&context));
    }
    if BIOSFuns::reldb()
        .exists(
            &Query::select().columns(vec![IamAuthPolicy::Id]).from(IamAuthPolicy::Table).and_where(Expr::col(IamAuthPolicy::RelResourceId).eq(id.as_str())).done(),
            None,
        )
        .await?
    {
        return BIOSResp::err(
            IamOutput::AppConsoleEntityDeleteCheckExistAssociatedData(ObjectKind::Resource, "AuthPolicyObject"),
            Some(&context),
        );
    }

    if BIOSFuns::reldb()
        .exists(
            &Query::select().columns(vec![IamResource::Id]).from(IamResource::Table).and_where(Expr::col(IamResource::ParentId).eq(id.as_str())).done(),
            None,
        )
        .await?
    {
        return BIOSResp::err(IamOutput::AppConsoleEntityDeleteCheckExistAssociatedData(ObjectKind::Resource, "Resource"), Some(&context));
    }

    let mut conn = BIOSFuns::reldb().conn().await;
    let mut tx = conn.begin().await?;

    let sql_builder = Query::select()
        .columns(IamResource::iter().filter(|i| *i != IamResource::Table))
        .from(IamResource::Table)
        .and_where(Expr::col(IamResource::Id).eq(id.as_str()))
        .and_where(Expr::col(IamResource::RelAppId).eq(context.ident.app_id.as_str()))
        .done();
    BIOSFuns::reldb().soft_del(IamResource::Table, IamResource::Id, &context.ident.account_id, &sql_builder, &mut tx).await?;

    tx.commit().await?;
    BIOSResp::ok("", Some(&context))
}
