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

use bios::basic::dto::BIOSResp;
use bios::basic::error::BIOSError;
use bios::db::reldb_client::SqlBuilderProcess;
use bios::web::basic_processor::extract_context_with_account;
use bios::web::resp_handler::BIOSResponse;
use bios::web::validate::json::Json;
use bios::web::validate::query::Query as VQuery;
use bios::BIOSFuns;

use crate::domain::ident_domain::{IamAccount, IamApp, IamAppIdent, IamTenant};
use crate::process::basic_dto::CommonStatus;
use crate::process::common::cache_processor;
use crate::process::system_console::sc_tenant_dto::{TenantAddReq, TenantDetailResp, TenantModifyReq, TenantQueryReq};

#[post("/console/system/tenant")]
pub async fn add_tenant(tenant_add_req: Json<TenantAddReq>, req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;
    let id = bios::basic::field::uuid();

    BIOSFuns::reldb()
        .exec(
            &Query::insert()
                .into_table(IamTenant::Table)
                .columns(vec![
                    IamTenant::Id,
                    IamTenant::CreateUser,
                    IamTenant::UpdateUser,
                    IamTenant::Name,
                    IamTenant::Icon,
                    IamTenant::AllowAccountRegister,
                    IamTenant::Parameters,
                    IamTenant::Status,
                ])
                .values_panic(vec![
                    id.as_str().into(),
                    context.ident.account_id.as_str().into(),
                    context.ident.account_id.as_str().into(),
                    tenant_add_req.name.as_str().into(),
                    tenant_add_req.icon.as_deref().unwrap_or_default().into(),
                    tenant_add_req.allow_account_register.into(),
                    tenant_add_req.parameters.as_deref().unwrap_or_default().into(),
                    CommonStatus::Enabled.to_string().to_lowercase().into(),
                ])
                .done(),
            None,
        )
        .await?;
    BIOSResp::ok(id, Some(&context))
}

#[put("/console/system/tenant/{id}")]
pub async fn modify_tenant(tenant_modify_req: Json<TenantModifyReq>, req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;
    let id: String = req.match_info().get("id").unwrap().parse()?;

    let mut values = Vec::new();
    if let Some(name) = &tenant_modify_req.name {
        values.push((IamTenant::Name, name.as_str().into()));
    }
    if let Some(icon) = &tenant_modify_req.icon {
        values.push((IamTenant::Icon, icon.as_str().into()));
    }
    if let Some(allow_account_register) = tenant_modify_req.allow_account_register {
        values.push((IamTenant::AllowAccountRegister, allow_account_register.into()));
    }
    if let Some(parameters) = &tenant_modify_req.parameters {
        values.push((IamTenant::Parameters, parameters.as_str().into()));
    }
    if let Some(status) = &tenant_modify_req.status {
        values.push((IamTenant::Status, status.to_string().to_lowercase().into()));
    }
    values.push((IamTenant::UpdateUser, context.ident.account_id.as_str().into()));

    let mut conn = BIOSFuns::reldb().conn().await;
    let mut tx = conn.begin().await?;

    BIOSFuns::reldb()
        .exec(
            &Query::update().table(IamTenant::Table).values(values).and_where(Expr::col(IamTenant::Id).eq(id.as_str())).done(),
            Some(&mut tx),
        )
        .await?;
    if let Some(status) = &tenant_modify_req.status {
        let enabled_aksks = BIOSFuns::reldb()
            .fetch_all::<AkSkInfoResp>(
                &Query::select()
                    .column((IamApp::Table, IamApp::Id))
                    .column((IamAppIdent::Table, IamAppIdent::Ak))
                    .column((IamAppIdent::Table, IamAppIdent::Sk))
                    .column((IamAppIdent::Table, IamAppIdent::ValidTime))
                    .from(IamApp::Table)
                    .inner_join(IamAppIdent::Table, Expr::tbl(IamAppIdent::Table, IamAppIdent::RelAppId).equals(IamApp::Table, IamApp::Id))
                    .and_where(Expr::tbl(IamApp::Table, IamApp::Status).eq(CommonStatus::Enabled.to_string().to_lowercase()))
                    .and_where(Expr::tbl(IamApp::Table, IamApp::RelTenantId).eq(id.as_str()))
                    .done(),
                None,
            )
            .await?;
        match status {
            CommonStatus::Enabled => {
                for aksk_resp in enabled_aksks {
                    cache_processor::set_aksk(&id, &aksk_resp.app_id, &aksk_resp.ak, &aksk_resp.sk, aksk_resp.valid_time, &context).await?;
                }
            }
            CommonStatus::Disabled => {
                for aksk_resp in enabled_aksks {
                    cache_processor::remove_aksk(&aksk_resp.ak, &context).await?;
                }
            }
        }
    }
    tx.commit().await?;

    BIOSResp::ok("", Some(&context))
}

#[get("/console/system/tenant")]
pub async fn list_tenant(query: VQuery<TenantQueryReq>, req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;

    let create_user_table = Alias::new("create");
    let update_user_table = Alias::new("update");
    let sql_builder = Query::select()
        .columns(vec![
            (IamTenant::Table, IamTenant::Id),
            (IamTenant::Table, IamTenant::CreateTime),
            (IamTenant::Table, IamTenant::UpdateTime),
            (IamTenant::Table, IamTenant::Name),
            (IamTenant::Table, IamTenant::Icon),
            (IamTenant::Table, IamTenant::AllowAccountRegister),
            (IamTenant::Table, IamTenant::Parameters),
            (IamTenant::Table, IamTenant::Status),
        ])
        .expr_as(Expr::tbl(create_user_table.clone(), IamAccount::Name), Alias::new("create_user"))
        .expr_as(Expr::tbl(update_user_table.clone(), IamAccount::Name), Alias::new("update_user"))
        .from(IamTenant::Table)
        .join_as(
            JoinType::InnerJoin,
            IamAccount::Table,
            create_user_table.clone(),
            Expr::tbl(create_user_table, IamAccount::Id).equals(IamTenant::Table, IamTenant::CreateUser),
        )
        .join_as(
            JoinType::InnerJoin,
            IamAccount::Table,
            update_user_table.clone(),
            Expr::tbl(update_user_table, IamAccount::Id).equals(IamTenant::Table, IamTenant::UpdateUser),
        )
        .and_where_option(if let Some(name) = &query.name {
            Some(Expr::tbl(IamTenant::Table, IamTenant::Name).like(format!("%{}%", name).as_str()))
        } else {
            None
        })
        .order_by(IamTenant::UpdateTime, Order::Desc)
        .done();
    let items = BIOSFuns::reldb().pagination::<TenantDetailResp>(&sql_builder, query.page_number, query.page_size, None).await?;
    BIOSResp::ok(items, Some(&context))
}

#[delete("/console/system/tenant/{id}")]
pub async fn delete_tenant(req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;
    let id: String = req.match_info().get("id").unwrap().parse()?;

    if BIOSFuns::reldb()
        .exists(
            &Query::select().columns(vec![IamApp::Id]).from(IamApp::Table).and_where(Expr::col(IamApp::RelTenantId).eq(id.as_str())).done(),
            None,
        )
        .await?
    {
        return BIOSResp::err(BIOSError::Conflict("Please delete the associated [app] data first".to_owned()), Some(&context));
    }

    let mut conn = BIOSFuns::reldb().conn().await;
    let mut tx = conn.begin().await?;

    let sql_builder =
        Query::select().columns(IamTenant::iter().filter(|i| *i != IamTenant::Table)).from(IamTenant::Table).and_where(Expr::col(IamTenant::Id).eq(id.as_str())).done();
    BIOSFuns::reldb().soft_del(IamTenant::Table, IamTenant::Id, &context.ident.account_id, &sql_builder, &mut tx).await?;

    tx.commit().await?;
    BIOSResp::ok("", Some(&context))
}

#[derive(sqlx::FromRow, serde::Deserialize)]
pub struct AkSkInfoResp {
    pub ak: String,
    pub sk: String,
    pub valid_time: i64,
    pub app_id: String,
}
