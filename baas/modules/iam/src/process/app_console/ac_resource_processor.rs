/*
 * Copyright 2021. gudaoxuri
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
use sea_query::{Alias, Cond, Expr, JoinType, Query};
use sqlx::Connection;
use strum::IntoEnumIterator;

use bios::basic::error::BIOSError;
use bios::db::basic_dto::IdResp;
use bios::db::reldb_client::SqlBuilderProcess;
use bios::web::basic_processor::get_ident_account_info;
use bios::web::resp_handler::{BIOSResp, BIOSRespHelper};
use bios::web::validate::json::Json;
use bios::web::validate::query::Query as VQuery;
use bios::BIOSFuns;

use crate::constant::RESOURCE_SUBJECT_DEFAULT_CODE_SPLIT;
use crate::domain::auth_domain::{IamAuthPolicy, IamResource, IamResourceSubject};
use crate::domain::ident_domain::IamAccount;
use crate::process::app_console::ac_resource_dto::{
    ResourceAddReq, ResourceDetailResp, ResourceModifyReq, ResourceQueryReq, ResourceSubjectAddReq, ResourceSubjectDetailResp, ResourceSubjectModifyReq, ResourceSubjectQueryReq,
};

#[post("/console/app/resource/subject")]
pub async fn add_resource_subject(resource_subject_add_req: Json<ResourceSubjectAddReq>, req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;
    if resource_subject_add_req.code_postfix.contains(&RESOURCE_SUBJECT_DEFAULT_CODE_SPLIT) {
        return BIOSRespHelper::bus_error(BIOSError::BadRequest(
            format!("ResourceSubject [code_postfix] can't contain [{}]", &RESOURCE_SUBJECT_DEFAULT_CODE_SPLIT).to_owned(),
        ));
    }
    let resource_subject_code = format!(
        "{}{}{}{}{}",
        &ident_info.app_id,
        &RESOURCE_SUBJECT_DEFAULT_CODE_SPLIT,
        &resource_subject_add_req.kind.to_string().to_lowercase(),
        &RESOURCE_SUBJECT_DEFAULT_CODE_SPLIT,
        &resource_subject_add_req.code_postfix
    )
    .to_lowercase();
    if BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamResourceSubject::Id])
                .from(IamResourceSubject::Table)
                .and_where(Expr::col(IamResourceSubject::Code).eq(resource_subject_code.clone()))
                .and_where(Expr::col(IamResourceSubject::RelAppId).eq(ident_info.app_id.clone()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::BadRequest("ResourceSubject [code] already exists".to_owned()));
    }
    let id = bios::basic::field::uuid();
    let sql_builder = Query::insert()
        .into_table(IamResourceSubject::Table)
        .columns(vec![
            IamResourceSubject::Id,
            IamResourceSubject::CreateUser,
            IamResourceSubject::UpdateUser,
            IamResourceSubject::Code,
            IamResourceSubject::Kind,
            IamResourceSubject::Uri,
            IamResourceSubject::Name,
            IamResourceSubject::Sort,
            IamResourceSubject::Ak,
            IamResourceSubject::Sk,
            IamResourceSubject::PlatformAccount,
            IamResourceSubject::PlatformProjectId,
            IamResourceSubject::TimeoutMs,
            IamResourceSubject::RelAppId,
            IamResourceSubject::RelTenantId,
        ])
        .values_panic(vec![
            id.clone().into(),
            ident_info.account_id.clone().into(),
            ident_info.account_id.clone().into(),
            resource_subject_code.clone().into(),
            resource_subject_add_req.kind.to_string().to_lowercase().into(),
            bios::basic::uri::format(&resource_subject_add_req.uri).expect("Uri parse error").into(),
            resource_subject_add_req.name.clone().into(),
            resource_subject_add_req.sort.into(),
            resource_subject_add_req.ak.as_ref().unwrap_or(&"".to_string()).to_string().into(),
            resource_subject_add_req.sk.as_ref().unwrap_or(&"".to_string()).to_string().into(),
            resource_subject_add_req.platform_account.as_ref().unwrap_or(&"".to_string()).to_string().into(),
            resource_subject_add_req.platform_project_id.as_ref().unwrap_or(&"".to_string()).to_string().into(),
            resource_subject_add_req.timeout_ms.unwrap_or(0).into(),
            ident_info.app_id.clone().into(),
            ident_info.tenant_id.clone().into(),
        ])
        .done();
    BIOSFuns::reldb().exec(&sql_builder, None).await?;
    BIOSRespHelper::ok(id)
}

#[put("/console/app/resource/subject/{id}")]
pub async fn modify_resource_subject(resource_subject_modify_req: Json<ResourceSubjectModifyReq>, req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;
    let id: String = req.match_info().get("id").unwrap().parse()?;

    if resource_subject_modify_req.code_postfix.is_some() && resource_subject_modify_req.kind.is_none() {
        return BIOSRespHelper::bus_error(BIOSError::BadRequest("ResourceSubject [code_postfix] and [kind] must both exist".to_owned()));
    }
    let mut values = Vec::new();
    if let Some(code_postfix) = &resource_subject_modify_req.code_postfix {
        if resource_subject_modify_req.code_postfix.as_ref().unwrap().contains(&RESOURCE_SUBJECT_DEFAULT_CODE_SPLIT) {
            return BIOSRespHelper::bus_error(BIOSError::BadRequest(
                format!("ResourceSubject [code_postfix] can't contain [{}]", &RESOURCE_SUBJECT_DEFAULT_CODE_SPLIT).to_owned(),
            ));
        }
        let resource_subject_code = format!(
            "{}{}{}{}{}",
            &ident_info.app_id,
            &RESOURCE_SUBJECT_DEFAULT_CODE_SPLIT,
            &resource_subject_modify_req.kind.as_ref().unwrap().to_string().to_lowercase(),
            &RESOURCE_SUBJECT_DEFAULT_CODE_SPLIT,
            code_postfix
        )
        .to_lowercase();
        if BIOSFuns::reldb()
            .exists(
                &Query::select()
                    .columns(vec![IamResourceSubject::Id])
                    .from(IamResourceSubject::Table)
                    .and_where(Expr::col(IamResourceSubject::Id).ne(id.clone()))
                    .and_where(Expr::col(IamResourceSubject::Code).eq(resource_subject_code.clone()))
                    .and_where(Expr::col(IamResourceSubject::RelAppId).eq(ident_info.app_id.clone()))
                    .done(),
                None,
            )
            .await?
        {
            return BIOSRespHelper::bus_error(BIOSError::BadRequest("ResourceSubject [code] already exists".to_owned()));
        }
        values.push((IamResourceSubject::Code, resource_subject_code.into()));
    }
    if let Some(kind) = &resource_subject_modify_req.kind {
        values.push((IamResourceSubject::Kind, kind.to_string().into()));
    }
    if let Some(uri) = &resource_subject_modify_req.uri {
        values.push((IamResourceSubject::Uri, bios::basic::uri::format(uri)?.into()));
    }
    if let Some(name) = &resource_subject_modify_req.name {
        values.push((IamResourceSubject::Name, name.to_string().into()));
    }
    if let Some(sort) = resource_subject_modify_req.sort {
        values.push((IamResourceSubject::Sort, sort.into()));
    }
    if let Some(ak) = &resource_subject_modify_req.ak {
        values.push((IamResourceSubject::Ak, ak.to_string().into()));
    }
    if let Some(sk) = &resource_subject_modify_req.sk {
        values.push((IamResourceSubject::Sk, sk.to_string().into()));
    }
    if let Some(platform_project_id) = &resource_subject_modify_req.platform_project_id {
        values.push((IamResourceSubject::PlatformProjectId, platform_project_id.to_string().into()));
    }
    if let Some(platform_account) = &resource_subject_modify_req.platform_account {
        values.push((IamResourceSubject::PlatformAccount, platform_account.to_string().into()));
    }
    if let Some(platform_project_id) = &resource_subject_modify_req.platform_project_id {
        values.push((IamResourceSubject::PlatformProjectId, platform_project_id.to_string().into()));
    }
    if let Some(timeout_ms) = &resource_subject_modify_req.timeout_ms {
        values.push((IamResourceSubject::TimeoutMs, timeout_ms.to_string().into()));
    }
    values.push((IamResourceSubject::UpdateUser, ident_info.account_id.clone().into()));
    let sql_builder = Query::update()
        .table(IamResourceSubject::Table)
        .values(values)
        .and_where(Expr::col(IamResourceSubject::Id).eq(id.clone()))
        .and_where(Expr::col(IamResourceSubject::RelAppId).eq(ident_info.app_id.clone()))
        .done();
    BIOSFuns::reldb().exec(&sql_builder, None).await?;
    BIOSRespHelper::ok("")
}

#[get("/console/app/resource/subject")]
pub async fn list_resource_subject(query: VQuery<ResourceSubjectQueryReq>, req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;

    let create_user_table = Alias::new("create");
    let update_user_table = Alias::new("update");
    let sql_builder = Query::select()
        .columns(vec![
            (IamResourceSubject::Table, IamResourceSubject::Id),
            (IamResourceSubject::Table, IamResourceSubject::CreateTime),
            (IamResourceSubject::Table, IamResourceSubject::UpdateTime),
            (IamResourceSubject::Table, IamResourceSubject::Code),
            (IamResourceSubject::Table, IamResourceSubject::Kind),
            (IamResourceSubject::Table, IamResourceSubject::Uri),
            (IamResourceSubject::Table, IamResourceSubject::Name),
            (IamResourceSubject::Table, IamResourceSubject::Sort),
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
        .and_where(Expr::tbl(IamResourceSubject::Table, IamResourceSubject::RelAppId).eq(ident_info.app_id.clone()))
        .done();
    let items = BIOSFuns::reldb()
        .pagination::<ResourceSubjectDetailResp>(&sql_builder, query.page_number, query.page_size, None)
        .await?;
    BIOSRespHelper::ok(items)
}

#[delete("/console/app/resource/subject/{id}")]
pub async fn delete_resource_subject(req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;
    let id: String = req.match_info().get("id").unwrap().parse()?;

    if BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamResource::Id])
                .from(IamResource::Table)
                .and_where(Expr::col(IamResource::RelResourceSubjectId).eq(id.clone()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::Conflict("Please delete the associated [resource] data first".to_owned()));
    }
    let mut conn = BIOSFuns::reldb().conn().await;
    let mut tx = conn.begin().await?;

    let sql_builder = Query::select()
        .columns(IamResourceSubject::iter().filter(|i| *i != IamResourceSubject::Table))
        .from(IamResourceSubject::Table)
        .and_where(Expr::col(IamResourceSubject::Id).eq(id.clone()))
        .and_where(Expr::col(IamResourceSubject::RelAppId).eq(ident_info.app_id.clone()))
        .done();
    BIOSFuns::reldb()
        .soft_del::<ResourceSubjectDetailResp, _, _>(IamResourceSubject::Table, IamResourceSubject::Id, &ident_info.account_id, &sql_builder, &mut tx)
        .await?;

    tx.commit().await?;
    BIOSRespHelper::ok("")
}

// ------------------------------------

#[post("/console/app/resource")]
pub async fn add_resource(resource_add_req: Json<ResourceAddReq>, req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;
    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamResourceSubject::Id])
                .from(IamResourceSubject::Table)
                .and_where(Expr::col(IamResourceSubject::Id).eq(resource_add_req.rel_resource_subject_id.clone()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::NotFound("Resource [rel_resource_subject_id] not exists".to_string()));
    }
    if BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamResource::Id])
                .from(IamResource::Table)
                .and_where(Expr::col(IamResource::PathAndQuery).eq(resource_add_req.path_and_query.clone()))
                .and_where(Expr::col(IamResource::RelResourceSubjectId).eq(resource_add_req.rel_resource_subject_id.clone()))
                .and_where(Expr::col(IamResource::RelAppId).eq(ident_info.app_id.clone()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::Conflict("Resource [path_and_query] already exists".to_string()));
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
                    id.clone().into(),
                    ident_info.account_id.clone().into(),
                    ident_info.account_id.clone().into(),
                    resource_add_req.path_and_query.clone().to_lowercase().into(),
                    resource_add_req.name.clone().into(),
                    resource_add_req.icon.clone().into(),
                    resource_add_req.sort.clone().into(),
                    resource_add_req.action.as_ref().unwrap_or(&"".to_string()).to_string().into(),
                    resource_add_req.res_group.into(),
                    resource_add_req.parent_id.as_ref().unwrap_or(&"".to_string()).to_string().into(),
                    resource_add_req.rel_resource_subject_id.clone().into(),
                    ident_info.app_id.clone().into(),
                    ident_info.tenant_id.clone().into(),
                    resource_add_req
                        .expose_kind
                        .as_ref()
                        .unwrap_or(&crate::process::basic_dto::ExposeKind::App)
                        .to_string()
                        .to_lowercase()
                        .into(),
                ])
                .done(),
            None,
        )
        .await?;
    BIOSRespHelper::ok(id)
}

#[put("/console/app/resource/{id}")]
pub async fn modify_resource(resource_modify_req: Json<ResourceModifyReq>, req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;
    let id: String = req.match_info().get("id").unwrap().parse()?;

    if let Some(path_and_query) = &resource_modify_req.path_and_query {
        let resource_subject_id_info = BIOSFuns::reldb()
            .fetch_one::<IdResp>(
                &Query::select()
                    .columns(vec![(IamResourceSubject::Table, IamResourceSubject::Id)])
                    .from(IamResource::Table)
                    .inner_join(
                        IamResourceSubject::Table,
                        Expr::tbl(IamResourceSubject::Table, IamResourceSubject::Id).equals(IamResource::Table, IamResource::RelResourceSubjectId),
                    )
                    .and_where(Expr::tbl(IamResourceSubject::Table, IamResourceSubject::RelAppId).eq(ident_info.app_id.clone()))
                    .and_where(Expr::tbl(IamResource::Table, IamResource::RelAppId).eq(ident_info.app_id.clone()))
                    .and_where(Expr::tbl(IamResource::Table, IamResource::Id).eq(id.clone()))
                    .done(),
                None,
            )
            .await?;
        if BIOSFuns::reldb()
            .exists(
                &Query::select()
                    .columns(vec![IamResource::Id])
                    .from(IamResource::Table)
                    .and_where(Expr::col(IamResource::Id).ne(id.clone()))
                    .and_where(Expr::col(IamResource::PathAndQuery).eq(path_and_query.to_string().to_lowercase()))
                    .and_where(Expr::col(IamResource::RelResourceSubjectId).eq(resource_subject_id_info.id))
                    .and_where(Expr::col(IamResource::RelAppId).eq(ident_info.app_id.clone()))
                    .done(),
                None,
            )
            .await?
        {
            return BIOSRespHelper::bus_error(BIOSError::Conflict("Resource [path_and_query] already exists".to_string()));
        }
    }
    if let Some(parent_id) = &resource_modify_req.parent_id {
        if !BIOSFuns::reldb()
            .exists(
                &Query::select()
                    .columns(vec![IamResource::Id])
                    .from(IamResource::Table)
                    .and_where(Expr::col(IamResource::Id).ne(parent_id.to_string().clone()))
                    .and_where(Expr::col(IamResource::RelAppId).eq(ident_info.app_id.clone()))
                    .done(),
                None,
            )
            .await?
        {
            return BIOSRespHelper::bus_error(BIOSError::Conflict("Resource [parent_id] not found".to_string()));
        }
    }
    let mut values = Vec::new();
    if let Some(path_and_query) = &resource_modify_req.path_and_query {
        values.push((IamResource::PathAndQuery, path_and_query.to_string().to_lowercase().into()));
    }
    if let Some(name) = &resource_modify_req.name {
        values.push((IamResource::Name, name.to_string().into()));
    }
    if let Some(icon) = &resource_modify_req.icon {
        values.push((IamResource::Icon, icon.to_string().into()));
    }
    if let Some(action) = &resource_modify_req.action {
        values.push((IamResource::Action, action.to_string().into()));
    }
    if let Some(sort) = resource_modify_req.sort {
        values.push((IamResource::Sort, sort.into()));
    }
    if let Some(res_group) = resource_modify_req.res_group {
        values.push((IamResource::ResGroup, res_group.into()));
    }
    if let Some(parent_id) = &resource_modify_req.parent_id {
        values.push((IamResource::ParentId, parent_id.to_string().into()));
    }
    if let Some(expose_kind) = &resource_modify_req.expose_kind {
        values.push((IamResource::ExposeKind, expose_kind.to_string().to_lowercase().into()));
    }
    values.push((IamResource::UpdateUser, ident_info.account_id.clone().into()));
    BIOSFuns::reldb()
        .exec(
            &Query::update()
                .table(IamResource::Table)
                .values(values)
                .and_where(Expr::col(IamResource::Id).eq(id.clone()))
                .and_where(Expr::col(IamResource::RelAppId).eq(ident_info.app_id.clone()))
                .done(),
            None,
        )
        .await?;
    BIOSRespHelper::ok("")
}

#[get("/console/app/resource")]
pub async fn list_resource(query: VQuery<ResourceQueryReq>, req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;

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
                x.and_where(Expr::tbl(IamResource::Table, IamResource::RelAppId).eq(ident_info.app_id.clone()));
            },
            |x| {
                x.cond_where(
                    Cond::any()
                        .add(Expr::tbl(IamResource::Table, IamResource::ExposeKind).eq(crate::process::basic_dto::ExposeKind::Global.to_string().to_lowercase()))
                        .add(
                            Cond::all()
                                .add(Expr::tbl(IamResource::Table, IamResource::RelTenantId).eq(ident_info.tenant_id.clone()))
                                .add(Expr::tbl(IamResource::Table, IamResource::ExposeKind).eq(crate::process::basic_dto::ExposeKind::Tenant.to_string().to_lowercase())),
                        ),
                );
            },
        )
        .done();
    let items = BIOSFuns::reldb()
        .pagination::<ResourceDetailResp>(&sql_builder, query.page_number, query.page_size, None)
        .await?;
    BIOSRespHelper::ok(items)
}

#[delete("/console/app/resource/{id}")]
pub async fn delete_resource(req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;
    let id: String = req.match_info().get("id").unwrap().parse()?;

    if BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamAuthPolicy::Id])
                .from(IamAuthPolicy::Table)
                .and_where(Expr::col(IamAuthPolicy::RelResourceId).eq(id.clone()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::Conflict("Please delete the associated [auth_policy] data first".to_owned()));
    }

    if BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamResource::Id])
                .from(IamResource::Table)
                .and_where(Expr::col(IamResource::ParentId).eq(id.clone()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::Conflict("Please delete the associated [resource] data first".to_owned()));
    }

    let mut conn = BIOSFuns::reldb().conn().await;
    let mut tx = conn.begin().await?;

    let sql_builder = Query::select()
        .columns(IamResource::iter().filter(|i| *i != IamResource::Table))
        .from(IamResource::Table)
        .and_where(Expr::col(IamResource::Id).eq(id.clone()))
        .and_where(Expr::col(IamResource::RelAppId).eq(ident_info.app_id.clone()))
        .done();
    BIOSFuns::reldb()
        .soft_del::<ResourceDetailResp, _, _>(IamResource::Table, IamResource::Id, &ident_info.account_id, &sql_builder, &mut tx)
        .await?;

    tx.commit().await?;
    BIOSRespHelper::ok("")
}
