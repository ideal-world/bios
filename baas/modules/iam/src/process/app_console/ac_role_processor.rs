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
use sea_query::{Alias, Expr, JoinType, Order, Query};
use sqlx::Connection;
use strum::IntoEnumIterator;

use bios::basic::error::BIOSError;
use bios::db::reldb_client::SqlBuilderProcess;
use bios::web::basic_processor::get_ident_account_info;
use bios::web::resp_handler::{BIOSResp, BIOSRespHelper};
use bios::web::validate::json::Json;
use bios::web::validate::query::Query as VQuery;
use bios::BIOSFuns;

use crate::domain::auth_domain::{IamAccountRole, IamAuthPolicySubject, IamRole};
use crate::domain::ident_domain::IamAccount;
use crate::process::app_console::ac_role_dto::{RoleAddReq, RoleDetailResp, RoleModifyReq, RoleQueryReq};
use crate::process::basic_dto::AuthSubjectKind;

#[post("/console/app/role")]
pub async fn add_role(role_add_req: Json<RoleAddReq>, req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;

    if BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamRole::Id])
                .from(IamRole::Table)
                .and_where(Expr::col(IamRole::Code).eq(role_add_req.code.clone().to_lowercase()))
                .and_where(Expr::col(IamRole::RelAppId).eq(ident_info.app_id.clone()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::Conflict("Role [code] already exists".to_string()));
    }
    let id = bios::basic::field::uuid();
    BIOSFuns::reldb()
        .exec(
            &Query::insert()
                .into_table(IamRole::Table)
                .columns(vec![
                    IamRole::Id,
                    IamRole::CreateUser,
                    IamRole::UpdateUser,
                    IamRole::Code,
                    IamRole::Name,
                    IamRole::Sort,
                    IamRole::RelAppId,
                    IamRole::RelTenantId,
                ])
                .values_panic(vec![
                    id.clone().into(),
                    ident_info.account_id.clone().into(),
                    ident_info.account_id.clone().into(),
                    role_add_req.code.clone().to_lowercase().into(),
                    role_add_req.name.clone().into(),
                    role_add_req.sort.clone().into(),
                    ident_info.app_id.clone().into(),
                    ident_info.tenant_id.clone().into(),
                ])
                .done(),
            None,
        )
        .await?;
    BIOSRespHelper::ok(id)
}

#[put("/console/app/role/{id}")]
pub async fn modify_role(role_modify_req: Json<RoleModifyReq>, req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;
    let id: String = req.match_info().get("id").unwrap().parse()?;

    if let Some(code) = &role_modify_req.code {
        if BIOSFuns::reldb()
            .exists(
                &Query::select()
                    .columns(vec![IamRole::Id])
                    .from(IamRole::Table)
                    .and_where(Expr::col(IamRole::Id).ne(id.clone()))
                    .and_where(Expr::col(IamRole::Code).eq(code.to_string().to_lowercase()))
                    .and_where(Expr::col(IamRole::RelAppId).eq(ident_info.app_id.clone().to_lowercase()))
                    .done(),
                None,
            )
            .await?
        {
            return BIOSRespHelper::bus_error(BIOSError::Conflict("Role [code] already exists".to_string()));
        }
    }
    let mut values = Vec::new();
    if let Some(code) = &role_modify_req.code {
        values.push((IamRole::Code, code.to_string().to_lowercase().into()));
    }
    if let Some(name) = &role_modify_req.name {
        values.push((IamRole::Name, name.to_string().into()));
    }
    if let Some(sort) = role_modify_req.sort {
        values.push((IamRole::Sort, sort.into()));
    }
    values.push((IamRole::UpdateUser, ident_info.account_id.clone().into()));
    BIOSFuns::reldb()
        .exec(
            &Query::update()
                .table(IamRole::Table)
                .values(values)
                .and_where(Expr::col(IamRole::Id).eq(id.clone()))
                .and_where(Expr::col(IamRole::RelAppId).eq(ident_info.app_id.clone()))
                .done(),
            None,
        )
        .await?;
    BIOSRespHelper::ok("")
}

#[get("/console/app/role")]
pub async fn list_role(query: VQuery<RoleQueryReq>, req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;

    let create_user_table = Alias::new("create");
    let update_user_table = Alias::new("update");
    let sql_builder = Query::select()
        .columns(vec![
            (IamRole::Table, IamRole::Id),
            (IamRole::Table, IamRole::CreateTime),
            (IamRole::Table, IamRole::UpdateTime),
            (IamRole::Table, IamRole::Code),
            (IamRole::Table, IamRole::Name),
            (IamRole::Table, IamRole::Sort),
            (IamRole::Table, IamRole::RelAppId),
            (IamRole::Table, IamRole::RelTenantId),
        ])
        .expr_as(Expr::tbl(create_user_table.clone(), IamAccount::Name), Alias::new("create_user"))
        .expr_as(Expr::tbl(update_user_table.clone(), IamAccount::Name), Alias::new("update_user"))
        .from(IamRole::Table)
        .join_as(
            JoinType::InnerJoin,
            IamAccount::Table,
            create_user_table.clone(),
            Expr::tbl(create_user_table, IamAccount::Id).equals(IamRole::Table, IamRole::CreateUser),
        )
        .join_as(
            JoinType::InnerJoin,
            IamAccount::Table,
            update_user_table.clone(),
            Expr::tbl(update_user_table, IamAccount::Id).equals(IamRole::Table, IamRole::UpdateUser),
        )
        .and_where_option(if let Some(code) = &query.code {
            Some(Expr::tbl(IamRole::Table, IamRole::Code).like(format!("%{}%", code).as_str()))
        } else {
            None
        })
        .and_where_option(if let Some(name) = &query.name {
            Some(Expr::tbl(IamRole::Table, IamRole::Name).like(format!("%{}%", name).as_str()))
        } else {
            None
        })
        .and_where(Expr::tbl(IamRole::Table, IamRole::RelAppId).eq(ident_info.app_id.clone()))
        .order_by(IamRole::UpdateTime, Order::Desc)
        .done();
    let items = BIOSFuns::reldb()
        .pagination::<RoleDetailResp>(&sql_builder, query.page_number, query.page_size, None)
        .await?;
    BIOSRespHelper::ok(items)
}

#[delete("/console/app/role/{id}")]
pub async fn delete_role(req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;
    let id: String = req.match_info().get("id").unwrap().parse()?;

    if BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamAuthPolicySubject::Id])
                .from(IamAuthPolicySubject::Table)
                .and_where(Expr::col(IamAuthPolicySubject::SubjectKind).eq(AuthSubjectKind::Role.to_string().to_lowercase()))
                .and_where(Expr::col(IamAuthPolicySubject::SubjectId).eq(id.clone()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::Conflict("Please delete the associated [auth_policy_subject] data first".to_owned()));
    }
    if BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamAccountRole::Id])
                .from(IamAccountRole::Table)
                .and_where(Expr::col(IamAccountRole::RelRoleId).eq(id.clone()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::Conflict("Please delete the associated [account_role] data first".to_owned()));
    }

    let mut conn = BIOSFuns::reldb().conn().await;
    let mut tx = conn.begin().await?;

    let sql_builder = Query::select()
        .columns(IamRole::iter().filter(|i| *i != IamRole::Table))
        .from(IamRole::Table)
        .and_where(Expr::col(IamRole::Id).eq(id.clone()))
        .and_where(Expr::col(IamRole::RelAppId).eq(ident_info.app_id.clone()))
        .done();
    BIOSFuns::reldb()
        .soft_del::<RoleDetailResp, _, _>(IamRole::Table, IamRole::Id, &ident_info.account_id, &sql_builder, &mut tx)
        .await?;

    tx.commit().await?;
    BIOSRespHelper::ok("")
}
