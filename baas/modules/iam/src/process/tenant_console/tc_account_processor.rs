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

use std::collections::HashSet;
use std::str::FromStr;

use actix_web::{delete, get, post, put, HttpRequest};
use sea_query::{Alias, Cond, Expr, JoinType, Order, Query};
use sqlx::Connection;
use strum::IntoEnumIterator;

use bios::basic::dto::BIOSResp;
use bios::basic::error::BIOSError;
use bios::db::reldb_client::SqlBuilderProcess;
use bios::web::basic_processor::extract_context_with_account;
use bios::web::resp_handler::BIOSResponse;
use bios::web::validate::json::Json;
use bios::web::validate::query::Query as VQuery;
use bios::BIOSFuns;

use crate::domain::auth_domain::{IamAccountGroup, IamAccountRole, IamAuthPolicyObject};
use crate::domain::ident_domain::{IamAccount, IamAccountApp, IamAccountBind, IamAccountIdent, IamApp};
use crate::process::basic_dto::{AccountIdentKind, AuthObjectKind, CommonStatus};
use crate::process::common::{auth_processor, cache_processor};
use crate::process::tenant_console::tc_account_dto::{
    AccountAddReq, AccountAppDetailResp, AccountDetailResp, AccountIdentAddReq, AccountIdentDetailResp, AccountIdentModifyReq, AccountModifyReq, AccountQueryReq,
};

#[post("/console/tenant/account")]
pub async fn add_account(account_add_req: Json<AccountAddReq>, req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;
    let id = bios::basic::field::uuid();
    let open_id = format!("ao_{}", bios::basic::field::uuid());

    BIOSFuns::reldb()
        .exec(
            &Query::insert()
                .into_table(IamAccount::Table)
                .columns(vec![
                    IamAccount::Id,
                    IamAccount::CreateUser,
                    IamAccount::UpdateUser,
                    IamAccount::OpenId,
                    IamAccount::Name,
                    IamAccount::Avatar,
                    IamAccount::Parameters,
                    IamAccount::ParentId,
                    IamAccount::RelTenantId,
                    IamAccount::Status,
                ])
                .values_panic(vec![
                    id.as_str().into(),
                    context.ident.account_id.as_str().into(),
                    context.ident.account_id.as_str().into(),
                    open_id.into(),
                    account_add_req.name.as_str().into(),
                    account_add_req.avatar.as_deref().unwrap_or_default().into(),
                    account_add_req.parameters.as_deref().unwrap_or_default().into(),
                    account_add_req.parent_id.as_deref().unwrap_or_default().into(),
                    context.ident.tenant_id.as_str().into(),
                    CommonStatus::Enabled.to_string().to_lowercase().into(),
                ])
                .done(),
            None,
        )
        .await?;
    BIOSResp::ok(id, Some(&context))
}

#[put("/console/tenant/account/{id}")]
pub async fn modify_account(account_modify_req: Json<AccountModifyReq>, req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;
    let id: String = req.match_info().get("id").unwrap().parse()?;

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamAccount::Id])
                .from(IamAccount::Table)
                .and_where(Expr::col(IamAccount::Id).eq(id.as_str()))
                .and_where(Expr::col(IamAccount::RelTenantId).eq(context.ident.tenant_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSResp::err(BIOSError::NotFound("Account not exists".to_string()), Some(&context));
    }

    let mut values = Vec::new();
    if let Some(name) = &account_modify_req.name {
        values.push((IamAccount::Name, name.as_str().into()));
    }
    if let Some(avatar) = &account_modify_req.avatar {
        values.push((IamAccount::Avatar, avatar.as_str().into()));
    }
    if let Some(parameters) = &account_modify_req.parameters {
        values.push((IamAccount::Parameters, parameters.as_str().into()));
    }
    if let Some(parent_id) = &account_modify_req.parent_id {
        values.push((IamAccount::ParentId, parent_id.as_str().into()));
    }
    if let Some(status) = &account_modify_req.status {
        values.push((IamAccount::Status, status.to_string().to_lowercase().into()));
    }
    values.push((IamAccount::UpdateUser, context.ident.account_id.as_str().into()));

    let mut conn = BIOSFuns::reldb().conn().await;
    let mut tx = conn.begin().await?;

    BIOSFuns::reldb()
        .exec(
            &Query::update()
                .table(IamAccount::Table)
                .values(values)
                .and_where(Expr::col(IamAccount::Id).eq(id.as_str()))
                .and_where(Expr::col(IamAccount::RelTenantId).eq(context.ident.tenant_id.as_str()))
                .done(),
            Some(&mut tx),
        )
        .await?;
    if let Some(status) = &account_modify_req.status {
        if status.to_string().to_lowercase() == CommonStatus::Disabled.to_string().to_lowercase() {
            // Remove token
            cache_processor::remove_token_by_account(&id, &context).await?;
        }
    }
    tx.commit().await?;
    BIOSResp::ok("", Some(&context))
}

#[get("/console/tenant/account")]
pub async fn list_account(query: VQuery<AccountQueryReq>, req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;

    let create_user_table = Alias::new("create");
    let update_user_table = Alias::new("update");
    let sql_builder = Query::select()
        .columns(vec![
            (IamAccount::Table, IamAccount::Id),
            (IamAccount::Table, IamAccount::CreateTime),
            (IamAccount::Table, IamAccount::UpdateTime),
            (IamAccount::Table, IamAccount::Name),
            (IamAccount::Table, IamAccount::Avatar),
            (IamAccount::Table, IamAccount::Parameters),
            (IamAccount::Table, IamAccount::OpenId),
            (IamAccount::Table, IamAccount::ParentId),
            (IamAccount::Table, IamAccount::RelTenantId),
            (IamAccount::Table, IamAccount::Status),
        ])
        .expr_as(Expr::tbl(create_user_table.clone(), IamAccount::Name), Alias::new("create_user"))
        .expr_as(Expr::tbl(update_user_table.clone(), IamAccount::Name), Alias::new("update_user"))
        .from(IamAccount::Table)
        .join_as(
            JoinType::InnerJoin,
            IamAccount::Table,
            create_user_table.clone(),
            Expr::tbl(create_user_table, IamAccount::Id).equals(IamAccount::Table, IamAccount::CreateUser),
        )
        .join_as(
            JoinType::InnerJoin,
            IamAccount::Table,
            update_user_table.clone(),
            Expr::tbl(update_user_table, IamAccount::Id).equals(IamAccount::Table, IamAccount::UpdateUser),
        )
        .and_where_option(if let Some(name) = &query.name {
            Some(Expr::tbl(IamAccount::Table, IamAccount::Name).like(format!("%{}%", name).as_str()))
        } else {
            None
        })
        .and_where(Expr::tbl(IamAccount::Table, IamAccount::RelTenantId).eq(context.ident.tenant_id.as_str()))
        .order_by(IamAccount::UpdateTime, Order::Desc)
        .done();
    let item = BIOSFuns::reldb().pagination::<AccountDetailResp>(&sql_builder, query.page_number, query.page_size, None).await?;
    BIOSResp::ok(item, Some(&context))
}

#[delete("/console/tenant/account/{id}")]
pub async fn delete_account(req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;
    let id: String = req.match_info().get("id").unwrap().parse()?;

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamAccount::Id])
                .from(IamAccount::Table)
                .and_where(Expr::col(IamAccount::Id).eq(id.as_str()))
                .and_where(Expr::col(IamAccount::RelTenantId).eq(context.ident.tenant_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSResp::err(BIOSError::NotFound("Account not exists".to_string()), Some(&context));
    }

    let mut conn = BIOSFuns::reldb().conn().await;
    let mut tx = conn.begin().await?;

    // Delete IamAccountIdent
    BIOSFuns::reldb()
        .soft_del(
            IamAccountIdent::Table,
            IamAccountIdent::Id,
            &context.ident.account_id,
            &Query::select()
                .columns(IamAccountIdent::iter().filter(|i| *i != IamAccountIdent::Table))
                .from(IamAccountIdent::Table)
                .and_where(Expr::col(IamAccountIdent::RelAccountId).eq(id.as_str()))
                .done(),
            &mut tx,
        )
        .await?;
    // Delete IamAccountBind
    BIOSFuns::reldb()
        .soft_del(
            IamAccountBind::Table,
            IamAccountBind::Id,
            &context.ident.account_id,
            &Query::select()
                .columns(IamAccountBind::iter().filter(|i| *i != IamAccountBind::Table))
                .from(IamAccountBind::Table)
                .cond_where(Cond::any().add(Expr::col(IamAccountBind::FromAccountId).eq(id.as_str())).add(Expr::col(IamAccountBind::ToAccountId).eq(id.as_str())))
                .done(),
            &mut tx,
        )
        .await?;
    // Delete IamAccountApp
    BIOSFuns::reldb()
        .soft_del(
            IamAccountApp::Table,
            IamAccountApp::Id,
            &context.ident.account_id,
            &Query::select()
                .columns(IamAccountApp::iter().filter(|i| *i != IamAccountApp::Table))
                .from(IamAccountApp::Table)
                .and_where(Expr::col(IamAccountApp::RelAccountId).eq(id.as_str()))
                .done(),
            &mut tx,
        )
        .await?;
    // Delete IamAccountGroup
    BIOSFuns::reldb()
        .soft_del(
            IamAccountGroup::Table,
            IamAccountGroup::Id,
            &context.ident.account_id,
            &Query::select()
                .columns(IamAccountGroup::iter().filter(|i| *i != IamAccountGroup::Table))
                .from(IamAccountGroup::Table)
                .and_where(Expr::col(IamAccountGroup::RelAccountId).eq(id.as_str()))
                .done(),
            &mut tx,
        )
        .await?;
    // Delete IamAccountRole
    BIOSFuns::reldb()
        .soft_del(
            IamAccountRole::Table,
            IamAccountRole::Id,
            &context.ident.account_id,
            &Query::select()
                .columns(IamAccountRole::iter().filter(|i| *i != IamAccountRole::Table))
                .from(IamAccountRole::Table)
                .and_where(Expr::col(IamAccountRole::RelAccountId).eq(id.as_str()))
                .done(),
            &mut tx,
        )
        .await?;
    // Delete IamAuthPolicySubject
    let auth_policy_ids = BIOSFuns::reldb()
        .fetch_all::<IdResp>(
            &Query::select()
                .columns(vec![IamAuthPolicyObject::RelAuthPolicyId])
                .from(IamAuthPolicyObject::Table)
                .and_where(Expr::col(IamAuthPolicyObject::ObjectKind).eq(AuthObjectKind::Account.to_string().to_lowercase()))
                .and_where(Expr::col(IamAuthPolicyObject::ObjectId).eq(id.as_str()))
                .done(),
            Some(&mut tx),
        )
        .await?
        .iter()
        .map(|record| record.id.to_string())
        .collect::<HashSet<String>>();
    BIOSFuns::reldb()
        .soft_del(
            IamAuthPolicyObject::Table,
            IamAuthPolicyObject::Id,
            &context.ident.account_id,
            &Query::select()
                .columns(IamAuthPolicyObject::iter().filter(|i| *i != IamAuthPolicyObject::Table))
                .from(IamAuthPolicyObject::Table)
                .and_where(Expr::col(IamAuthPolicyObject::ObjectKind).eq(AuthObjectKind::Account.to_string().to_lowercase()))
                .and_where(Expr::col(IamAuthPolicyObject::ObjectId).eq(id.as_str()))
                .done(),
            &mut tx,
        )
        .await?;
    // Delete IamAccount
    let sql_builder = Query::select()
        .columns(IamAccount::iter().filter(|i| *i != IamAccount::Table))
        .from(IamAccount::Table)
        .and_where(Expr::col(IamAccount::Id).eq(id.as_str()))
        .and_where(Expr::col(IamAccount::RelTenantId).eq(context.ident.tenant_id.as_str()))
        .done();
    BIOSFuns::reldb().soft_del(IamAccount::Table, IamAccount::Id, &context.ident.account_id, &sql_builder, &mut tx).await?;

    // Remove auth policy cache
    for auth_policy_id in auth_policy_ids.iter() {
        cache_processor::remove_auth_policy(&auth_policy_id, &mut tx, &context).await?;
    }
    // Remove token
    cache_processor::remove_token_by_account(&id, &context).await?;
    tx.commit().await?;
    BIOSResp::ok("", Some(&context))
}

// ------------------------------------

#[post("/console/tenant/account/{account_id}/ident")]
pub async fn add_account_ident(account_ident_add_req: Json<AccountIdentAddReq>, req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;
    let account_id: String = req.match_info().get("account_id").unwrap().parse()?;
    let id = bios::basic::field::uuid();

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamAccount::Id])
                .from(IamAccount::Table)
                .and_where(Expr::col(IamAccount::Id).eq(account_id.as_str()))
                .and_where(Expr::col(IamAccount::RelTenantId).eq(context.ident.tenant_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSResp::err(BIOSError::NotFound("Account not exists".to_string()), Some(&context));
    }
    if BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamAccountIdent::Id])
                .from(IamAccountIdent::Table)
                .and_where(Expr::col(IamAccountIdent::Kind).eq(account_ident_add_req.kind.to_string().to_lowercase()))
                .and_where(Expr::col(IamAccountIdent::Ak).eq(account_ident_add_req.ak.as_str()))
                .and_where(Expr::col(IamAccountIdent::RelTenantId).eq(context.ident.tenant_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSResp::err(BIOSError::Conflict("AccountIdent [kind] and [ak] already exists".to_string()), Some(&context));
    }

    auth_processor::valid_account_ident(
        &account_ident_add_req.kind,
        &account_ident_add_req.ak,
        &account_ident_add_req.sk.as_deref().unwrap_or_default(),
        None,
        &context,
    )
    .await?;
    let processed_sk = auth_processor::process_sk(
        &account_ident_add_req.kind,
        &account_ident_add_req.ak,
        &account_ident_add_req.sk.as_deref().unwrap_or_default(),
        &context,
    )
    .await?;

    BIOSFuns::reldb()
        .exec(
            &Query::insert()
                .into_table(IamAccountIdent::Table)
                .columns(vec![
                    IamAccountIdent::Id,
                    IamAccountIdent::CreateUser,
                    IamAccountIdent::UpdateUser,
                    IamAccountIdent::Kind,
                    IamAccountIdent::Ak,
                    IamAccountIdent::Sk,
                    IamAccountIdent::ValidStartTime,
                    IamAccountIdent::ValidEndTime,
                    IamAccountIdent::RelAccountId,
                    IamAccountIdent::RelTenantId,
                ])
                .values_panic(vec![
                    id.as_str().into(),
                    context.ident.account_id.as_str().into(),
                    context.ident.account_id.as_str().into(),
                    account_ident_add_req.kind.to_string().to_lowercase().into(),
                    account_ident_add_req.ak.as_str().into(),
                    processed_sk.into(),
                    account_ident_add_req.valid_start_time.into(),
                    account_ident_add_req.valid_end_time.into(),
                    account_id.into(),
                    context.ident.tenant_id.as_str().into(),
                ])
                .done(),
            None,
        )
        .await?;
    BIOSResp::ok(id, Some(&context))
}

#[put("/console/tenant/account/{account_id}/ident/{id}")]
pub async fn modify_account_ident(account_ident_modify_req: Json<AccountIdentModifyReq>, req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;
    let account_id: String = req.match_info().get("account_id").unwrap().parse()?;
    let id: String = req.match_info().get("id").unwrap().parse()?;

    if account_ident_modify_req.ak.is_some() && account_ident_modify_req.ak.is_none() || account_ident_modify_req.sk.is_none() && account_ident_modify_req.sk.is_some() {
        return BIOSResp::err(BIOSError::BadRequest("AccountIdent [ak] and [sk] must exist at the same time".to_string()), Some(&context));
    }

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamAccount::Id])
                .from(IamAccount::Table)
                .and_where(Expr::col(IamAccount::Id).eq(account_id.as_str()))
                .and_where(Expr::col(IamAccount::RelTenantId).eq(context.ident.tenant_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSResp::err(BIOSError::NotFound("Account not exists".to_string()), Some(&context));
    }

    let kind = BIOSFuns::reldb()
        .fetch_one_json(
            &Query::select()
                .columns(vec![IamAccountIdent::Kind])
                .from(IamAccountIdent::Table)
                .and_where(Expr::col(IamAccountIdent::Id).eq(id.as_str()))
                .and_where(Expr::col(IamAccountIdent::RelAccountId).eq(account_id.as_str()))
                .done(),
            None,
        )
        .await?;
    let kind = kind["kind"].as_str().unwrap_or_default();

    auth_processor::valid_account_ident(
        &AccountIdentKind::from_str(kind).unwrap(),
        account_ident_modify_req.ak.as_deref().unwrap_or_default(),
        account_ident_modify_req.sk.as_deref().unwrap_or_default(),
        None,
        &context,
    )
    .await?;
    let mut values = Vec::new();
    if let Some(ak) = &account_ident_modify_req.ak {
        if BIOSFuns::reldb()
            .exists(
                &Query::select()
                    .columns(vec![IamAccountIdent::Id])
                    .from(IamAccountIdent::Table)
                    .and_where(Expr::col(IamAccountIdent::Kind).eq(kind))
                    .and_where(Expr::col(IamAccountIdent::Ak).eq(ak.as_str()))
                    .and_where(Expr::col(IamAccountIdent::RelTenantId).eq(context.ident.tenant_id.as_str()))
                    .and_where(Expr::col(IamAccountIdent::RelAccountId).ne(id.as_str()))
                    .done(),
                None,
            )
            .await?
        {
            return BIOSResp::err(BIOSError::Conflict("AccountIdent [kind] and [ak] already exists".to_string()), Some(&context));
        }
        values.push((IamAccountIdent::Ak, ak.to_string().as_str().into()));
    }
    if let Some(sk) = &account_ident_modify_req.sk {
        let processed_sk = auth_processor::process_sk(
            &AccountIdentKind::from_str(kind).unwrap(),
            account_ident_modify_req.ak.as_deref().unwrap_or_default(),
            sk,
            &context,
        )
        .await?;
        values.push((IamAccountIdent::Sk, processed_sk.into()));
    }
    if let Some(valid_start_time) = account_ident_modify_req.valid_start_time {
        values.push((IamAccountIdent::ValidStartTime, valid_start_time.to_string().as_str().into()));
    }
    if let Some(valid_end_time) = account_ident_modify_req.valid_end_time {
        values.push((IamAccountIdent::ValidEndTime, valid_end_time.to_string().as_str().into()));
    }
    values.push((IamAccountIdent::UpdateUser, context.ident.account_id.as_str().into()));
    BIOSFuns::reldb()
        .exec(
            &Query::update()
                .table(IamAccountIdent::Table)
                .values(values)
                .and_where(Expr::col(IamAccountIdent::Id).eq(id.as_str()))
                .and_where(Expr::col(IamAccountIdent::RelAccountId).eq(account_id))
                .done(),
            None,
        )
        .await?;
    BIOSResp::ok("", Some(&context))
}

#[get("/console/tenant/account/{account_id}/ident")]
pub async fn list_account_ident(req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;
    let account_id: String = req.match_info().get("account_id").unwrap().parse()?;

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamAccount::Id])
                .from(IamAccount::Table)
                .and_where(Expr::col(IamAccount::Id).eq(account_id.as_str()))
                .and_where(Expr::col(IamAccount::RelTenantId).eq(context.ident.tenant_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSResp::err(BIOSError::NotFound("Account not exists".to_string()), Some(&context));
    }

    let create_user_table = Alias::new("create");
    let update_user_table = Alias::new("update");
    let sql_builder = Query::select()
        .columns(vec![
            (IamAccountIdent::Table, IamAccountIdent::Id),
            (IamAccountIdent::Table, IamAccountIdent::CreateTime),
            (IamAccountIdent::Table, IamAccountIdent::UpdateTime),
            (IamAccountIdent::Table, IamAccountIdent::Kind),
            (IamAccountIdent::Table, IamAccountIdent::Ak),
            (IamAccountIdent::Table, IamAccountIdent::ValidStartTime),
            (IamAccountIdent::Table, IamAccountIdent::ValidEndTime),
            (IamAccountIdent::Table, IamAccountIdent::RelAccountId),
        ])
        .expr_as(Expr::tbl(create_user_table.clone(), IamAccount::Name), Alias::new("create_user"))
        .expr_as(Expr::tbl(update_user_table.clone(), IamAccount::Name), Alias::new("update_user"))
        .from(IamAccountIdent::Table)
        .join_as(
            JoinType::InnerJoin,
            IamAccount::Table,
            create_user_table.clone(),
            Expr::tbl(create_user_table, IamAccount::Id).equals(IamAccountIdent::Table, IamAccountIdent::CreateUser),
        )
        .join_as(
            JoinType::InnerJoin,
            IamAccount::Table,
            update_user_table.clone(),
            Expr::tbl(update_user_table, IamAccount::Id).equals(IamAccountIdent::Table, IamAccountIdent::UpdateUser),
        )
        .and_where(Expr::tbl(IamAccountIdent::Table, IamAccountIdent::RelAccountId).eq(account_id))
        .order_by(IamAccountIdent::UpdateTime, Order::Desc)
        .done();
    let items = BIOSFuns::reldb().fetch_all::<AccountIdentDetailResp>(&sql_builder, None).await?;
    BIOSResp::ok(items, Some(&context))
}

#[delete("/console/tenant/account/{account_id}/ident/{id}")]
pub async fn delete_account_ident(req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;
    let account_id: String = req.match_info().get("account_id").unwrap().parse()?;
    let id: String = req.match_info().get("id").unwrap().parse()?;

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamAccount::Id])
                .from(IamAccount::Table)
                .and_where(Expr::col(IamAccount::Id).eq(account_id.as_str()))
                .and_where(Expr::col(IamAccount::RelTenantId).eq(context.ident.tenant_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSResp::err(BIOSError::NotFound("Account not exists".to_string()), Some(&context));
    }

    let mut conn = BIOSFuns::reldb().conn().await;
    let mut tx = conn.begin().await?;

    let sql_builder = Query::select()
        .columns(IamAccountIdent::iter().filter(|i| *i != IamAccountIdent::Table))
        .from(IamAccountIdent::Table)
        .and_where(Expr::col(IamAccountIdent::Id).eq(id.as_str()))
        .and_where(Expr::col(IamAccountIdent::RelAccountId).eq(account_id))
        .done();
    BIOSFuns::reldb().soft_del(IamAccountIdent::Table, IamAccountIdent::Id, &context.ident.account_id, &sql_builder, &mut tx).await?;
    tx.commit().await?;
    BIOSResp::ok("", Some(&context))
}

// ------------------------------------

#[post("/console/tenant/account/{account_id}/app/{app_id}")]
pub async fn add_account_app(req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;
    let account_id: String = req.match_info().get("account_id").unwrap().parse()?;
    let app_id: String = req.match_info().get("app_id").unwrap().parse()?;
    let id = bios::basic::field::uuid();

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamAccount::Id])
                .from(IamAccount::Table)
                .and_where(Expr::col(IamAccount::Id).eq(account_id.as_str()))
                .and_where(Expr::col(IamAccount::RelTenantId).eq(context.ident.tenant_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSResp::err(BIOSError::NotFound("Account not exists".to_string()), Some(&context));
    }
    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamApp::Id])
                .from(IamApp::Table)
                .and_where(Expr::col(IamApp::Id).eq(app_id.as_str()))
                .and_where(Expr::col(IamApp::RelTenantId).eq(context.ident.tenant_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSResp::err(BIOSError::NotFound("App not exists".to_string()), Some(&context));
    }

    if BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamAccountApp::Id])
                .from(IamAccountApp::Table)
                .and_where(Expr::col(IamAccountApp::RelAppId).eq(app_id.as_str()))
                .and_where(Expr::col(IamAccountApp::RelAccountId).eq(account_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSResp::err(
            BIOSError::Conflict("IamAccountApp [rel_app_id] and [rel_account_id] already exists".to_string()),
            Some(&context),
        );
    }

    BIOSFuns::reldb()
        .exec(
            &Query::insert()
                .into_table(IamAccountApp::Table)
                .columns(vec![
                    IamAccountApp::Id,
                    IamAccountApp::CreateUser,
                    IamAccountApp::UpdateUser,
                    IamAccountApp::RelAccountId,
                    IamAccountApp::RelAppId,
                ])
                .values_panic(vec![
                    id.as_str().into(),
                    context.ident.account_id.as_str().into(),
                    context.ident.account_id.as_str().into(),
                    account_id.into(),
                    app_id.into(),
                ])
                .done(),
            None,
        )
        .await?;
    BIOSResp::ok(id, Some(&context))
}

#[get("/console/tenant/account/{account_id}/app")]
pub async fn list_account_app(req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;
    let account_id: String = req.match_info().get("account_id").unwrap().parse()?;

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamAccount::Id])
                .from(IamAccount::Table)
                .and_where(Expr::col(IamAccount::Id).eq(account_id.as_str()))
                .and_where(Expr::col(IamAccount::RelTenantId).eq(context.ident.tenant_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSResp::err(BIOSError::NotFound("Account not exists".to_string()), Some(&context));
    }

    let create_user_table = Alias::new("create");
    let update_user_table = Alias::new("update");
    let sql_builder = Query::select()
        .columns(vec![
            (IamAccountApp::Table, IamAccountApp::Id),
            (IamAccountApp::Table, IamAccountApp::CreateTime),
            (IamAccountApp::Table, IamAccountApp::UpdateTime),
            (IamAccountApp::Table, IamAccountApp::RelAccountId),
            (IamAccountApp::Table, IamAccountApp::RelAppId),
        ])
        .expr_as(Expr::tbl(create_user_table.clone(), IamAccount::Name), Alias::new("create_user"))
        .expr_as(Expr::tbl(update_user_table.clone(), IamAccount::Name), Alias::new("update_user"))
        .from(IamAccountApp::Table)
        .join_as(
            JoinType::InnerJoin,
            IamAccount::Table,
            create_user_table.clone(),
            Expr::tbl(create_user_table, IamAccount::Id).equals(IamAccountApp::Table, IamAccountApp::CreateUser),
        )
        .join_as(
            JoinType::InnerJoin,
            IamAccount::Table,
            update_user_table.clone(),
            Expr::tbl(update_user_table, IamAccount::Id).equals(IamAccountApp::Table, IamAccountApp::UpdateUser),
        )
        .and_where(Expr::tbl(IamAccountApp::Table, IamAccountApp::RelAccountId).eq(account_id))
        .order_by(IamAccountApp::UpdateTime, Order::Desc)
        .done();
    let items = BIOSFuns::reldb().fetch_all::<AccountAppDetailResp>(&sql_builder, None).await?;
    BIOSResp::ok(items, Some(&context))
}

#[delete("/console/tenant/account/{account_id}/app/{app_id}")]
pub async fn delete_account_app(req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;
    let account_id: String = req.match_info().get("account_id").unwrap().parse()?;
    let app_id: String = req.match_info().get("app_id").unwrap().parse()?;

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamAccount::Id])
                .from(IamAccount::Table)
                .and_where(Expr::col(IamAccount::Id).eq(account_id.as_str()))
                .and_where(Expr::col(IamAccount::RelTenantId).eq(context.ident.tenant_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSResp::err(BIOSError::NotFound("Account not exists".to_string()), Some(&context));
    }
    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamApp::Id])
                .from(IamApp::Table)
                .and_where(Expr::col(IamApp::Id).eq(app_id.as_str()))
                .and_where(Expr::col(IamApp::RelTenantId).eq(context.ident.tenant_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSResp::err(BIOSError::NotFound("App not exists".to_string()), Some(&context));
    }

    let mut conn = BIOSFuns::reldb().conn().await;
    let mut tx = conn.begin().await?;

    let sql_builder = Query::select()
        .columns(IamAccountApp::iter().filter(|i| *i != IamAccountApp::Table))
        .from(IamAccountApp::Table)
        .and_where(Expr::col(IamAccountApp::RelAccountId).eq(account_id.as_str()))
        .and_where(Expr::col(IamAccountApp::RelAppId).eq(app_id))
        .done();
    BIOSFuns::reldb().soft_del(IamAccountApp::Table, IamAccountApp::Id, &context.ident.account_id, &sql_builder, &mut tx).await?;
    // Remove token
    cache_processor::remove_token_by_account(&account_id, &context).await?;
    tx.commit().await?;
    BIOSResp::ok("", Some(&context))
}

#[derive(sqlx::FromRow, serde::Deserialize)]
pub struct IdResp {
    pub id: String,
}
