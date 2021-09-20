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
use bios::db::reldb_client::SqlBuilderProcess;
use bios::web::basic_processor::extract_context_with_account;
use bios::web::resp_handler::BIOSResponse;
use bios::web::validate::json::Json;
use bios::BIOSFuns;

use crate::domain::ident_domain::{IamAccount, IamAppIdent};
use crate::iam_constant::{IamOutput, ObjectKind};
use crate::process::app_console::ac_app_dto::{AppIdentAddReq, AppIdentDetailResp, AppIdentModifyReq};
use crate::process::common::cache_processor;

#[post("/console/app/app/ident")]
pub async fn add_app_ident(app_ident_add_req: Json<AppIdentAddReq>, req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;
    let id = bios::basic::field::uuid();
    let ak = bios::basic::security::key::generate_ak();
    let sk = bios::basic::security::key::generate_sk(&ak);

    let mut conn = BIOSFuns::reldb().conn().await;
    let mut tx = conn.begin().await?;

    BIOSFuns::reldb()
        .exec(
            &Query::insert()
                .into_table(IamAppIdent::Table)
                .columns(vec![
                    IamAppIdent::Id,
                    IamAppIdent::CreateUser,
                    IamAppIdent::UpdateUser,
                    IamAppIdent::Note,
                    IamAppIdent::Ak,
                    IamAppIdent::Sk,
                    IamAppIdent::ValidTime,
                    IamAppIdent::RelAppId,
                    IamAppIdent::RelTenantId,
                ])
                .values_panic(vec![
                    id.as_str().into(),
                    context.ident.account_id.as_str().into(),
                    context.ident.account_id.as_str().into(),
                    app_ident_add_req.note.as_str().into(),
                    ak.as_str().into(),
                    sk.as_str().into(),
                    app_ident_add_req.valid_time.into(),
                    context.ident.app_id.as_str().into(),
                    context.ident.tenant_id.as_str().into(),
                ])
                .done(),
            Some(&mut tx),
        )
        .await?;
    cache_processor::set_aksk(&context.ident.tenant_id, &context.ident.app_id, &ak, &sk, app_ident_add_req.valid_time, &context).await?;
    tx.commit().await?;
    BIOSResp::ok(id, Some(&context))
}

#[put("/console/app/app/ident/{id}")]
pub async fn modify_app_ident(app_ident_modify_req: Json<AppIdentModifyReq>, req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;
    let id: String = req.match_info().get("id").unwrap().parse()?;

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamAppIdent::Id])
                .from(IamAppIdent::Table)
                .and_where(Expr::col(IamAppIdent::Id).eq(id.as_str()))
                .and_where(Expr::col(IamAppIdent::RelAppId).eq(context.ident.app_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSResp::err(IamOutput::AppConsoleEntityModifyCheckNotFound(ObjectKind::AppIdent, ObjectKind::AppIdent), Some(&context));
    }

    let mut values = Vec::new();
    if let Some(note) = &app_ident_modify_req.note {
        values.push((IamAppIdent::Note, note.as_str().into()));
    }
    if let Some(valid_time) = app_ident_modify_req.valid_time {
        values.push((IamAppIdent::ValidTime, valid_time.into()));
    }
    values.push((IamAppIdent::UpdateUser, context.ident.account_id.as_str().into()));

    let mut conn = BIOSFuns::reldb().conn().await;
    let mut tx = conn.begin().await?;

    BIOSFuns::reldb()
        .exec(
            &Query::update()
                .table(IamAppIdent::Table)
                .values(values)
                .and_where(Expr::col(IamAppIdent::Id).eq(id.as_str()))
                .and_where(Expr::col(IamAppIdent::RelAppId).eq(context.ident.app_id.as_str()))
                .done(),
            Some(&mut tx),
        )
        .await?;
    if let Some(valid_time) = app_ident_modify_req.valid_time {
        let sql_builder = Query::select().columns(vec![IamAppIdent::Ak, IamAppIdent::Sk]).from(IamAppIdent::Table).and_where(Expr::col(IamAppIdent::Id).eq(id.as_str())).done();
        let aksk_resp = BIOSFuns::reldb().fetch_one::<AkSkResp>(&sql_builder, Some(&mut tx)).await?;
        cache_processor::set_aksk(&context.ident.tenant_id, &context.ident.app_id, &aksk_resp.ak, &aksk_resp.sk, valid_time, &context).await?;
    }
    tx.commit().await?;
    BIOSResp::ok("", Some(&context))
}

#[get("/console/app/app/ident")]
pub async fn list_app_ident(req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;

    let create_user_table = Alias::new("create");
    let update_user_table = Alias::new("update");
    let sql_builder = Query::select()
        .columns(vec![
            (IamAppIdent::Table, IamAppIdent::Id),
            (IamAppIdent::Table, IamAppIdent::CreateTime),
            (IamAppIdent::Table, IamAppIdent::UpdateTime),
            (IamAppIdent::Table, IamAppIdent::Note),
            (IamAppIdent::Table, IamAppIdent::Ak),
            (IamAppIdent::Table, IamAppIdent::ValidTime),
            (IamAppIdent::Table, IamAppIdent::RelAppId),
            (IamAppIdent::Table, IamAppIdent::RelTenantId),
        ])
        .expr_as(Expr::tbl(create_user_table.clone(), IamAccount::Name), Alias::new("create_user"))
        .expr_as(Expr::tbl(update_user_table.clone(), IamAccount::Name), Alias::new("update_user"))
        .from(IamAppIdent::Table)
        .join_as(
            JoinType::InnerJoin,
            IamAccount::Table,
            create_user_table.clone(),
            Expr::tbl(create_user_table, IamAccount::Id).equals(IamAppIdent::Table, IamAppIdent::CreateUser),
        )
        .join_as(
            JoinType::InnerJoin,
            IamAccount::Table,
            update_user_table.clone(),
            Expr::tbl(update_user_table, IamAccount::Id).equals(IamAppIdent::Table, IamAppIdent::UpdateUser),
        )
        .and_where(Expr::tbl(IamAppIdent::Table, IamAppIdent::RelAppId).eq(context.ident.app_id.as_str()))
        .order_by(IamAppIdent::UpdateTime, Order::Desc)
        .done();
    let items = BIOSFuns::reldb().fetch_all::<AppIdentDetailResp>(&sql_builder, None).await?;
    BIOSResp::ok(items, Some(&context))
}

#[delete("/console/app/app/ident/{id}")]
pub async fn delete_app_ident(req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;
    let id: String = req.match_info().get("id").unwrap().parse()?;

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamAppIdent::Id])
                .from(IamAppIdent::Table)
                .and_where(Expr::col(IamAppIdent::Id).eq(id.as_str()))
                .and_where(Expr::col(IamAppIdent::RelAppId).eq(context.ident.app_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSResp::err(IamOutput::AppConsoleEntityDeleteCheckNotFound(ObjectKind::AppIdent, ObjectKind::AppIdent), Some(&context));
    }

    let sql_builder = Query::select().columns(vec![IamAppIdent::Ak, IamAppIdent::Sk]).from(IamAppIdent::Table).and_where(Expr::col(IamAppIdent::Id).eq(id.as_str())).done();
    let aksk_resp = BIOSFuns::reldb().fetch_one::<AkSkResp>(&sql_builder, None).await?;

    let mut conn = BIOSFuns::reldb().conn().await;
    let mut tx = conn.begin().await?;

    let sql_builder = Query::select()
        .columns(IamAppIdent::iter().filter(|i| *i != IamAppIdent::Table))
        .from(IamAppIdent::Table)
        .and_where(Expr::col(IamAppIdent::Id).eq(id.as_str()))
        .and_where(Expr::col(IamAppIdent::RelAppId).eq(context.ident.app_id.as_str()))
        .done();
    BIOSFuns::reldb().soft_del(IamAppIdent::Table, IamAppIdent::Id, &context.ident.account_id, &sql_builder, &mut tx).await?;
    cache_processor::remove_aksk(&aksk_resp.ak, &context).await?;
    tx.commit().await?;
    BIOSResp::ok("", Some(&context))
}

#[get("/console/app/app/ident/{id}/sk")]
pub async fn get_app_ident_sk(req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;
    let id: String = req.match_info().get("id").unwrap().parse()?;

    let sql_builder = Query::select()
        .columns(vec![IamAppIdent::Ak, IamAppIdent::Sk])
        .from(IamAppIdent::Table)
        .and_where(Expr::col(IamAppIdent::Id).eq(id.as_str()))
        .and_where(Expr::col(IamAppIdent::RelAppId).eq(context.ident.app_id.as_str()))
        .done();
    let aksk_resp = BIOSFuns::reldb().fetch_one::<AkSkResp>(&sql_builder, None).await?;
    BIOSResp::ok(aksk_resp.sk, Some(&context))
}

#[derive(sqlx::FromRow, serde::Deserialize)]
pub struct AkSkResp {
    pub ak: String,
    pub sk: String,
}
