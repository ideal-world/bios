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

use actix_web::{delete, get, post, put, HttpRequest};
use sea_query::{Alias, Cond, Expr, JoinType, Order, Query};
use sqlx::Connection;
use strum::IntoEnumIterator;

use bios::basic::error::BIOSError;
use bios::db::reldb_client::SqlBuilderProcess;
use bios::web::basic_processor::get_ident_account_info;
use bios::web::resp_handler::{BIOSResp, BIOSRespHelper};
use bios::web::validate::json::Json;
use bios::web::validate::query::Query as VQuery;
use bios::BIOSFuns;

use crate::domain::auth_domain::{IamAccountGroup, IamAccountRole, IamAuthPolicyObject};
use crate::domain::ident_domain::{IamAccount, IamAccountApp, IamAccountBind, IamAccountIdent, IamApp};
use crate::process::basic_dto::{AuthObjectKind, CommonStatus};
use crate::process::common::{auth_processor, cache_processor};
use crate::process::tenant_console::tc_account_dto::{
    AccountAddReq, AccountAppDetailResp, AccountDetailResp, AccountIdentAddReq, AccountIdentDetailResp, AccountIdentModifyReq, AccountModifyReq, AccountQueryReq,
};

#[post("/console/tenant/account")]
pub async fn add_account(account_add_req: Json<AccountAddReq>, req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;
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
                    id.clone().into(),
                    ident_info.account_id.clone().into(),
                    ident_info.account_id.clone().into(),
                    open_id.into(),
                    account_add_req.name.clone().into(),
                    account_add_req.avatar.clone().unwrap_or_default().into(),
                    account_add_req.parameters.clone().unwrap_or_default().into(),
                    account_add_req.parent_id.clone().unwrap_or_default().into(),
                    ident_info.tenant_id.clone().into(),
                    CommonStatus::Enabled.to_string().to_lowercase().into(),
                ])
                .done(),
            None,
        )
        .await?;
    BIOSRespHelper::ok(id)
}

#[put("/console/tenant/account/{id}")]
pub async fn modify_account(account_modify_req: Json<AccountModifyReq>, req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;
    let id: String = req.match_info().get("id").unwrap().parse()?;

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamAccount::Id])
                .from(IamAccount::Table)
                .and_where(Expr::col(IamAccount::Id).eq(id.clone()))
                .and_where(Expr::col(IamAccount::RelTenantId).eq(ident_info.tenant_id.clone()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::NotFound("Account not exists".to_string()));
    }

    let mut values = Vec::new();
    if let Some(name) = &account_modify_req.name {
        values.push((IamAccount::Name, name.to_string().into()));
    }
    if let Some(avatar) = &account_modify_req.avatar {
        values.push((IamAccount::Avatar, avatar.to_string().into()));
    }
    if let Some(parameters) = &account_modify_req.parameters {
        values.push((IamAccount::Parameters, parameters.to_string().into()));
    }
    if let Some(parent_id) = &account_modify_req.parent_id {
        values.push((IamAccount::ParentId, parent_id.to_string().into()));
    }
    if let Some(status) = &account_modify_req.status {
        values.push((IamAccount::Status, status.to_string().to_lowercase().into()));
    }
    values.push((IamAccount::UpdateUser, ident_info.account_id.clone().into()));

    let mut conn = BIOSFuns::reldb().conn().await;
    let mut tx = conn.begin().await?;

    BIOSFuns::reldb()
        .exec(
            &Query::update()
                .table(IamAccount::Table)
                .values(values)
                .and_where(Expr::col(IamAccount::Id).eq(id.clone()))
                .and_where(Expr::col(IamAccount::RelTenantId).eq(ident_info.tenant_id))
                .done(),
            Some(&mut tx),
        )
        .await?;
    if let Some(status) = &account_modify_req.status {
        if status.to_string().to_lowercase() == CommonStatus::Disabled.to_string().to_lowercase() {
            // Remove token
            cache_processor::remove_token_by_account(&id).await?;
        }
    }
    tx.commit().await?;
    BIOSRespHelper::ok("")
}

#[get("/console/tenant/account")]
pub async fn list_account(query: VQuery<AccountQueryReq>, req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;

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
        .and_where(Expr::tbl(IamAccount::Table, IamAccount::RelTenantId).eq(ident_info.tenant_id))
        .order_by(IamAccount::UpdateTime, Order::Desc)
        .done();
    let item = BIOSFuns::reldb().pagination::<AccountDetailResp>(&sql_builder, query.page_number, query.page_size, None).await?;
    BIOSRespHelper::ok(item)
}

#[delete("/console/tenant/account/{id}")]
pub async fn delete_account(req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;
    let id: String = req.match_info().get("id").unwrap().parse()?;

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamAccount::Id])
                .from(IamAccount::Table)
                .and_where(Expr::col(IamAccount::Id).eq(id.clone()))
                .and_where(Expr::col(IamAccount::RelTenantId).eq(ident_info.tenant_id.clone()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::NotFound("Account not exists".to_string()));
    }

    let mut conn = BIOSFuns::reldb().conn().await;
    let mut tx = conn.begin().await?;

    // Delete IamAccountIdent
    BIOSFuns::reldb()
        .soft_del(
            IamAccountIdent::Table,
            IamAccountIdent::Id,
            &ident_info.account_id,
            &Query::select()
                .columns(IamAccountIdent::iter().filter(|i| *i != IamAccountIdent::Table))
                .from(IamAccountIdent::Table)
                .and_where(Expr::col(IamAccountIdent::RelAccountId).eq(id.clone()))
                .done(),
            &mut tx,
        )
        .await?;
    // Delete IamAccountBind
    BIOSFuns::reldb()
        .soft_del(
            IamAccountBind::Table,
            IamAccountBind::Id,
            &ident_info.account_id,
            &Query::select()
                .columns(IamAccountBind::iter().filter(|i| *i != IamAccountBind::Table))
                .from(IamAccountBind::Table)
                .cond_where(Cond::any().add(Expr::col(IamAccountBind::FromAccountId).eq(id.clone())).add(Expr::col(IamAccountBind::ToAccountId).eq(id.clone())))
                .done(),
            &mut tx,
        )
        .await?;
    // Delete IamAccountApp
    BIOSFuns::reldb()
        .soft_del(
            IamAccountApp::Table,
            IamAccountApp::Id,
            &ident_info.account_id,
            &Query::select()
                .columns(IamAccountApp::iter().filter(|i| *i != IamAccountApp::Table))
                .from(IamAccountApp::Table)
                .and_where(Expr::col(IamAccountApp::RelAccountId).eq(id.clone()))
                .done(),
            &mut tx,
        )
        .await?;
    // Delete IamAccountGroup
    BIOSFuns::reldb()
        .soft_del(
            IamAccountGroup::Table,
            IamAccountGroup::Id,
            &ident_info.account_id,
            &Query::select()
                .columns(IamAccountGroup::iter().filter(|i| *i != IamAccountGroup::Table))
                .from(IamAccountGroup::Table)
                .and_where(Expr::col(IamAccountGroup::RelAccountId).eq(id.clone()))
                .done(),
            &mut tx,
        )
        .await?;
    // Delete IamAccountRole
    BIOSFuns::reldb()
        .soft_del(
            IamAccountRole::Table,
            IamAccountRole::Id,
            &ident_info.account_id,
            &Query::select()
                .columns(IamAccountRole::iter().filter(|i| *i != IamAccountRole::Table))
                .from(IamAccountRole::Table)
                .and_where(Expr::col(IamAccountRole::RelAccountId).eq(id.clone()))
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
                .and_where(Expr::col(IamAuthPolicyObject::ObjectId).eq(id.clone()))
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
            &ident_info.account_id,
            &Query::select()
                .columns(IamAuthPolicyObject::iter().filter(|i| *i != IamAuthPolicyObject::Table))
                .from(IamAuthPolicyObject::Table)
                .and_where(Expr::col(IamAuthPolicyObject::ObjectKind).eq(AuthObjectKind::Account.to_string().to_lowercase()))
                .and_where(Expr::col(IamAuthPolicyObject::ObjectId).eq(id.clone()))
                .done(),
            &mut tx,
        )
        .await?;
    // Delete IamAccount
    let sql_builder = Query::select()
        .columns(IamAccount::iter().filter(|i| *i != IamAccount::Table))
        .from(IamAccount::Table)
        .and_where(Expr::col(IamAccount::Id).eq(id.clone()))
        .and_where(Expr::col(IamAccount::RelTenantId).eq(ident_info.tenant_id.clone()))
        .done();
    BIOSFuns::reldb().soft_del(IamAccount::Table, IamAccount::Id, &ident_info.account_id, &sql_builder, &mut tx).await?;

    // Remove auth policy cache
    for auth_policy_id in auth_policy_ids.iter() {
        cache_processor::remove_auth_policy(&auth_policy_id, &mut tx).await?;
    }
    // Remove token
    cache_processor::remove_token_by_account(&id).await?;
    tx.commit().await?;
    BIOSRespHelper::ok("")
}

// ------------------------------------

#[post("/console/tenant/account/{account_id}/ident")]
pub async fn add_account_ident(account_ident_add_req: Json<AccountIdentAddReq>, req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;
    let account_id: String = req.match_info().get("account_id").unwrap().parse()?;
    let id = bios::basic::field::uuid();

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamAccount::Id])
                .from(IamAccount::Table)
                .and_where(Expr::col(IamAccount::Id).eq(account_id.clone()))
                .and_where(Expr::col(IamAccount::RelTenantId).eq(ident_info.tenant_id.clone()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::NotFound("Account not exists".to_string()));
    }
    if BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamAccountIdent::Id])
                .from(IamAccountIdent::Table)
                .and_where(Expr::col(IamAccountIdent::Kind).eq(account_ident_add_req.kind.to_string().to_lowercase()))
                .and_where(Expr::col(IamAccountIdent::Ak).eq(account_ident_add_req.ak.clone()))
                .and_where(Expr::col(IamAccountIdent::RelAccountId).eq(account_id.clone()))
                .done(),
            None,
        )
        .await?
    {
        return Err(BIOSError::Conflict("AccountIdent [kind] and [ak] already exists".to_string()));
    }

    auth_processor::valid_account_ident(
        &account_ident_add_req.kind.to_string().to_lowercase(),
        &account_ident_add_req.ak,
        &account_ident_add_req.sk.clone().unwrap_or_default(),
        &ident_info.tenant_id,
        None,
    )
    .await?;
    let processed_sk = auth_processor::process_sk(
        &account_ident_add_req.kind.to_string().to_lowercase(),
        &account_ident_add_req.ak,
        &account_ident_add_req.sk.clone().unwrap_or_default(),
        &ident_info.tenant_id,
        &ident_info.app_id,
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
                    id.clone().into(),
                    ident_info.account_id.clone().into(),
                    ident_info.account_id.clone().into(),
                    account_ident_add_req.kind.to_string().to_lowercase().into(),
                    account_ident_add_req.ak.clone().into(),
                    processed_sk.into(),
                    account_ident_add_req.valid_start_time.into(),
                    account_ident_add_req.valid_end_time.into(),
                    account_id.into(),
                    ident_info.tenant_id.into(),
                ])
                .done(),
            None,
        )
        .await?;
    BIOSRespHelper::ok(id)
}

#[put("/console/tenant/account/{account_id}/ident/{id}")]
pub async fn modify_account_ident(account_ident_modify_req: Json<AccountIdentModifyReq>, req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;
    let account_id: String = req.match_info().get("account_id").unwrap().parse()?;
    let id: String = req.match_info().get("id").unwrap().parse()?;

    if account_ident_modify_req.ak.is_some() && account_ident_modify_req.ak.is_none() || account_ident_modify_req.sk.is_none() && account_ident_modify_req.sk.is_some() {
        return BIOSRespHelper::bus_error(BIOSError::BadRequest("AccountIdent [ak] and [sk] must exist at the same time".to_string()));
    }

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamAccount::Id])
                .from(IamAccount::Table)
                .and_where(Expr::col(IamAccount::Id).eq(account_id.clone()))
                .and_where(Expr::col(IamAccount::RelTenantId).eq(ident_info.tenant_id.clone()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::NotFound("Account not exists".to_string()));
    }

    let kind = BIOSFuns::reldb()
        .fetch_one_json(
            &Query::select()
                .columns(vec![IamAccountIdent::Kind])
                .from(IamAccountIdent::Table)
                .and_where(Expr::col(IamAccountIdent::Id).eq(id.clone()))
                .and_where(Expr::col(IamAccountIdent::RelAccountId).eq(account_id.clone()))
                .done(),
            None,
        )
        .await?;
    let kind = kind["kind"].as_str().unwrap_or_default();

    auth_processor::valid_account_ident(
        kind,
        account_ident_modify_req.ak.clone().unwrap_or_default().as_str(),
        account_ident_modify_req.sk.clone().unwrap_or_default().as_str(),
        &ident_info.tenant_id,
        None,
    )
    .await?;
    let mut values = Vec::new();
    if let Some(ak) = &account_ident_modify_req.ak {
        values.push((IamAccountIdent::Ak, ak.to_string().clone().into()));
    }
    if let Some(sk) = &account_ident_modify_req.sk {
        let processed_sk = auth_processor::process_sk(
            kind,
            account_ident_modify_req.ak.clone().unwrap_or_default().as_str(),
            sk,
            &ident_info.tenant_id,
            &ident_info.app_id,
        )
        .await?;
        values.push((IamAccountIdent::Sk, processed_sk.into()));
    }
    if let Some(valid_start_time) = account_ident_modify_req.valid_start_time {
        values.push((IamAccountIdent::ValidStartTime, valid_start_time.to_string().clone().into()));
    }
    if let Some(valid_end_time) = account_ident_modify_req.valid_end_time {
        values.push((IamAccountIdent::ValidEndTime, valid_end_time.to_string().clone().into()));
    }
    values.push((IamAccountIdent::UpdateUser, ident_info.account_id.clone().into()));
    BIOSFuns::reldb()
        .exec(
            &Query::update()
                .table(IamAccountIdent::Table)
                .values(values)
                .and_where(Expr::col(IamAccountIdent::Id).eq(id.clone()))
                .and_where(Expr::col(IamAccountIdent::RelAccountId).eq(account_id))
                .done(),
            None,
        )
        .await?;
    BIOSRespHelper::ok("")
}

#[get("/console/tenant/account/{account_id}/ident")]
pub async fn list_account_ident(req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;
    let account_id: String = req.match_info().get("account_id").unwrap().parse()?;

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamAccount::Id])
                .from(IamAccount::Table)
                .and_where(Expr::col(IamAccount::Id).eq(account_id.clone()))
                .and_where(Expr::col(IamAccount::RelTenantId).eq(ident_info.tenant_id.clone()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::NotFound("Account not exists".to_string()));
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
    BIOSRespHelper::ok(items)
}

#[delete("/console/tenant/account/{account_id}/ident/{id}")]
pub async fn delete_account_ident(req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;
    let account_id: String = req.match_info().get("account_id").unwrap().parse()?;
    let id: String = req.match_info().get("id").unwrap().parse()?;

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamAccount::Id])
                .from(IamAccount::Table)
                .and_where(Expr::col(IamAccount::Id).eq(account_id.clone()))
                .and_where(Expr::col(IamAccount::RelTenantId).eq(ident_info.tenant_id.clone()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::NotFound("Account not exists".to_string()));
    }

    let mut conn = BIOSFuns::reldb().conn().await;
    let mut tx = conn.begin().await?;

    let sql_builder = Query::select()
        .columns(IamAccountIdent::iter().filter(|i| *i != IamAccountIdent::Table))
        .from(IamAccountIdent::Table)
        .and_where(Expr::col(IamAccountIdent::Id).eq(id.clone()))
        .and_where(Expr::col(IamAccountIdent::RelAccountId).eq(account_id))
        .done();
    BIOSFuns::reldb().soft_del(IamAccountIdent::Table, IamAccountIdent::Id, &ident_info.account_id, &sql_builder, &mut tx).await?;
    tx.commit().await?;
    BIOSRespHelper::ok("")
}

// ------------------------------------

#[post("/console/tenant/account/{account_id}/app/{app_id}")]
pub async fn add_account_app(req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;
    let account_id: String = req.match_info().get("account_id").unwrap().parse()?;
    let app_id: String = req.match_info().get("app_id").unwrap().parse()?;
    let id = bios::basic::field::uuid();

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamAccount::Id])
                .from(IamAccount::Table)
                .and_where(Expr::col(IamAccount::Id).eq(account_id.clone()))
                .and_where(Expr::col(IamAccount::RelTenantId).eq(ident_info.tenant_id.clone()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::NotFound("Account not exists".to_string()));
    }
    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamApp::Id])
                .from(IamApp::Table)
                .and_where(Expr::col(IamApp::Id).eq(app_id.clone()))
                .and_where(Expr::col(IamApp::RelTenantId).eq(ident_info.tenant_id.clone()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::NotFound("App not exists".to_string()));
    }

    if BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamAccountApp::Id])
                .from(IamAccountApp::Table)
                .and_where(Expr::col(IamAccountApp::RelAppId).eq(app_id.clone()))
                .and_where(Expr::col(IamAccountApp::RelAccountId).eq(account_id.clone()))
                .done(),
            None,
        )
        .await?
    {
        return Err(BIOSError::Conflict("IamAccountApp [rel_app_id] and [rel_account_id] already exists".to_string()));
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
                    id.clone().into(),
                    ident_info.account_id.clone().into(),
                    ident_info.account_id.clone().into(),
                    account_id.into(),
                    app_id.into(),
                ])
                .done(),
            None,
        )
        .await?;
    BIOSRespHelper::ok(id)
}

#[get("/console/tenant/account/{account_id}/app")]
pub async fn list_account_app(req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;
    let account_id: String = req.match_info().get("account_id").unwrap().parse()?;

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamAccount::Id])
                .from(IamAccount::Table)
                .and_where(Expr::col(IamAccount::Id).eq(account_id.clone()))
                .and_where(Expr::col(IamAccount::RelTenantId).eq(ident_info.tenant_id.clone()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::NotFound("Account not exists".to_string()));
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
    BIOSRespHelper::ok(items)
}

#[delete("/console/tenant/account/{account_id}/app/{app_id}")]
pub async fn delete_account_app(req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;
    let account_id: String = req.match_info().get("account_id").unwrap().parse()?;
    let app_id: String = req.match_info().get("app_id").unwrap().parse()?;

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamAccount::Id])
                .from(IamAccount::Table)
                .and_where(Expr::col(IamAccount::Id).eq(account_id.clone()))
                .and_where(Expr::col(IamAccount::RelTenantId).eq(ident_info.tenant_id.clone().clone()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::NotFound("Account not exists".to_string()));
    }
    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamApp::Id])
                .from(IamApp::Table)
                .and_where(Expr::col(IamApp::Id).eq(app_id.clone()))
                .and_where(Expr::col(IamApp::RelTenantId).eq(ident_info.tenant_id.clone().clone()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::NotFound("App not exists".to_string()));
    }

    let mut conn = BIOSFuns::reldb().conn().await;
    let mut tx = conn.begin().await?;

    let sql_builder = Query::select()
        .columns(IamAccountApp::iter().filter(|i| *i != IamAccountApp::Table))
        .from(IamAccountApp::Table)
        .and_where(Expr::col(IamAccountApp::RelAccountId).eq(account_id.clone()))
        .and_where(Expr::col(IamAccountApp::RelAppId).eq(app_id))
        .done();
    BIOSFuns::reldb().soft_del(IamAccountApp::Table, IamAccountApp::Id, &ident_info.account_id, &sql_builder, &mut tx).await?;
    // Remove token
    cache_processor::remove_token_by_account(&account_id).await?;
    tx.commit().await?;
    BIOSRespHelper::ok("")
}

#[derive(sqlx::FromRow, serde::Deserialize)]
pub struct IdResp {
    pub id: String,
}
