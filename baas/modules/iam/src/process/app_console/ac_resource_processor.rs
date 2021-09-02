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
use sea_query::{Expr, Query};
use sqlx::Connection;
use strum::IntoEnumIterator;

use bios::basic::error::BIOSError;
use bios::db::reldb_client::SqlBuilderProcess;
use bios::web::resp_handler::{BIOSResp, BIOSRespHelper};
use bios::web::validate::json::Json;
use bios::web::validate::query::Query as VQuery;
use bios::BIOSFuns;

use crate::constant::RESOURCE_SUBJECT_DEFAULT_CODE_SPLIT;
use crate::domain::auth_domain::{IamResource, IamResourceSubject};
use crate::process::app_console::ac_resource_dto::{ResourceSubjectAddReq, ResourceSubjectDetailResp, ResourceSubjectModifyReq, ResourceSubjectQueryReq};
use crate::process::basic_processor::get_ident_info_by_account;

#[post("/console/app/resource/subject")]
pub async fn add_resource_subject(resource_subject_add_req: Json<ResourceSubjectAddReq>, req: HttpRequest) -> BIOSResp {
    let ident_info_result = get_ident_info_by_account(&req);
    if ident_info_result.is_err() {
        return BIOSRespHelper::bus_error(ident_info_result.err().unwrap());
    }
    let ident_info = ident_info_result.unwrap();

    if resource_subject_add_req.code_postfix.contains(&RESOURCE_SUBJECT_DEFAULT_CODE_SPLIT) {
        return BIOSRespHelper::bus_error(BIOSError::BadRequest(
            format!("ResourceSubject [code_postfix] can't contain [{}]", &RESOURCE_SUBJECT_DEFAULT_CODE_SPLIT).to_owned(),
        ));
    }
    let resource_subject_code = format!(
        "{}{}{}{}{}",
        ident_info.app_id.as_ref().unwrap(),
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
                .and_where(Expr::col(IamResourceSubject::RelAppId).eq(ident_info.app_id.as_ref().unwrap().to_string()))
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
            ident_info.account_id.as_ref().unwrap().to_string().into(),
            ident_info.account_id.as_ref().unwrap().to_string().into(),
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
            ident_info.app_id.as_ref().unwrap().to_string().into(),
            ident_info.tenant_id.as_ref().unwrap().to_string().into(),
        ])
        .done();
    BIOSFuns::reldb().exec(&sql_builder, None).await?;
    BIOSRespHelper::ok(id)
}

#[put("/console/app/resource/subject/{id}")]
pub async fn modify_resource_subject(resource_subject_modify_req: Json<ResourceSubjectModifyReq>, req: HttpRequest) -> BIOSResp {
    let ident_info_result = get_ident_info_by_account(&req);
    if ident_info_result.is_err() {
        return BIOSRespHelper::bus_error(ident_info_result.err().unwrap());
    }
    let ident_info = ident_info_result.unwrap();

    let id: String = req.match_info().get("id").unwrap().parse()?;
    if resource_subject_modify_req.code_postfix.is_some() && resource_subject_modify_req.kind.is_none() {
        return BIOSRespHelper::bus_error(BIOSError::BadRequest("The resourceSubject [code_postfix] and [kind] must both exist".to_owned()));
    }
    let mut values = Vec::new();
    if resource_subject_modify_req.code_postfix.is_some() {
        if resource_subject_modify_req.code_postfix.as_ref().unwrap().contains(&RESOURCE_SUBJECT_DEFAULT_CODE_SPLIT) {
            return BIOSRespHelper::bus_error(BIOSError::BadRequest(
                format!("ResourceSubject [code_postfix] can't contain [{}]", &RESOURCE_SUBJECT_DEFAULT_CODE_SPLIT).to_owned(),
            ));
        }
        let resource_subject_code = format!(
            "{}{}{}{}{}",
            ident_info.app_id.as_ref().unwrap(),
            &RESOURCE_SUBJECT_DEFAULT_CODE_SPLIT,
            &resource_subject_modify_req.kind.as_ref().unwrap().to_string().to_lowercase(),
            &RESOURCE_SUBJECT_DEFAULT_CODE_SPLIT,
            &resource_subject_modify_req.code_postfix.as_ref().unwrap().to_string()
        )
        .to_lowercase();
        if BIOSFuns::reldb()
            .exists(
                &Query::select()
                    .columns(vec![IamResourceSubject::Id])
                    .from(IamResourceSubject::Table)
                    .and_where(Expr::col(IamResourceSubject::Id).ne(id.clone()))
                    .and_where(Expr::col(IamResourceSubject::Code).eq(resource_subject_code.clone()))
                    .and_where(Expr::col(IamResourceSubject::RelAppId).eq(ident_info.app_id.as_ref().unwrap().to_string()))
                    .done(),
                None,
            )
            .await?
        {
            return BIOSRespHelper::bus_error(BIOSError::BadRequest("ResourceSubject [code] already exists".to_owned()));
        }
        values.push((IamResourceSubject::Code, resource_subject_code.into()));
    }
    if resource_subject_modify_req.kind.as_ref().is_some() {
        values.push((IamResourceSubject::Kind, resource_subject_modify_req.kind.as_ref().unwrap().to_string().into()));
    }
    if resource_subject_modify_req.uri.as_ref().is_some() {
        values.push((IamResourceSubject::Uri, bios::basic::uri::format(resource_subject_modify_req.uri.as_ref().unwrap())?.into()));
    }
    if resource_subject_modify_req.name.is_some() {
        values.push((IamResourceSubject::Name, resource_subject_modify_req.name.as_ref().unwrap().to_string().into()));
    }
    if resource_subject_modify_req.sort.is_some() {
        values.push((IamResourceSubject::Sort, resource_subject_modify_req.sort.as_ref().unwrap().to_string().into()));
    }
    if resource_subject_modify_req.ak.is_some() {
        values.push((IamResourceSubject::Ak, resource_subject_modify_req.ak.as_ref().unwrap().to_string().into()));
    }
    if resource_subject_modify_req.ak.is_some() {
        values.push((IamResourceSubject::Ak, resource_subject_modify_req.ak.as_ref().unwrap().to_string().into()));
    }
    if resource_subject_modify_req.sk.is_some() {
        values.push((IamResourceSubject::Sk, resource_subject_modify_req.sk.as_ref().unwrap().to_string().into()));
    }
    if resource_subject_modify_req.platform_project_id.is_some() {
        values.push((
            IamResourceSubject::PlatformProjectId,
            resource_subject_modify_req.platform_project_id.as_ref().unwrap().to_string().into(),
        ));
    }
    if resource_subject_modify_req.platform_account.is_some() {
        values.push((
            IamResourceSubject::PlatformProjectId,
            resource_subject_modify_req.platform_account.as_ref().unwrap().to_string().into(),
        ));
    }
    if resource_subject_modify_req.timeout_ms.is_some() {
        values.push((IamResourceSubject::TimeoutMs, resource_subject_modify_req.timeout_ms.unwrap().into()));
    }
    values.push((IamResourceSubject::UpdateUser, ident_info.account_id.as_ref().unwrap().to_string().into()));
    let sql_builder = Query::update()
        .table(IamResourceSubject::Table)
        .values(values)
        .and_where(Expr::col(IamResourceSubject::Id).eq(id.clone()))
        .and_where(Expr::col(IamResourceSubject::RelAppId).eq(ident_info.app_id.as_ref().unwrap().to_string()))
        .done();
    BIOSFuns::reldb().exec(&sql_builder, None).await?;
    BIOSRespHelper::ok("")
}

#[get("/console/app/resource/subject")]
pub async fn list_resource_subject(query: VQuery<ResourceSubjectQueryReq>, req: HttpRequest) -> BIOSResp {
    let ident_info_result = get_ident_info_by_account(&req);
    if ident_info_result.is_err() {
        return BIOSRespHelper::bus_error(ident_info_result.err().unwrap());
    }
    let ident_info = ident_info_result.unwrap();

    let sql_builder = Query::select()
        .columns(IamResourceSubject::iter().filter(|i| *i != IamResourceSubject::Table))
        .from(IamResourceSubject::Table)
        .and_where_option(if query.name.as_ref().is_some() {
            Some(Expr::col(IamResourceSubject::Name).like(format!("%{}%", query.name.as_ref().unwrap()).as_str()))
        } else {
            None
        })
        .and_where(Expr::col(IamResourceSubject::RelAppId).eq(ident_info.app_id.as_ref().unwrap().to_string()))
        .done();
    let items = BIOSFuns::reldb()
        .pagination::<ResourceSubjectDetailResp>(&sql_builder, query.page_number, query.page_size, None)
        .await?;
    BIOSRespHelper::ok(items)
}

#[delete("/console/app/resource/subject/{id}")]
pub async fn delete_resource_subject(req: HttpRequest) -> BIOSResp {
    let ident_info_result = get_ident_info_by_account(&req);
    if ident_info_result.is_err() {
        return BIOSRespHelper::bus_error(ident_info_result.err().unwrap());
    }
    let ident_info = ident_info_result.unwrap();

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
        .and_where(Expr::col(IamResourceSubject::RelAppId).eq(ident_info.app_id.as_ref().unwrap().to_string()))
        .done();
    BIOSFuns::reldb()
        .soft_del::<ResourceSubjectDetailResp, _, _>(
            IamResourceSubject::Table,
            IamResourceSubject::Id,
            ident_info.account_id.as_ref().unwrap(),
            &sql_builder,
            &mut tx,
        )
        .await?;

    tx.commit().await?;
    BIOSRespHelper::ok("")
}
