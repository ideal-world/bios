/*
 * Copyright 2022. gudaoxuri
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

use crate::domain::ident_domain::{IamAccount, IamAccountIdent, IamTenant, IamTenantCert, IamTenantIdent};
use crate::iam_constant::{IamOutput, ObjectKind};
use crate::process::tenant_console::tc_tenant_dto::{
    TenantCertAddReq, TenantCertDetailResp, TenantCertModifyReq, TenantDetailResp, TenantIdentAddReq, TenantIdentDetailResp, TenantIdentModifyReq, TenantModifyReq,
};

#[put("/console/tenant/tenant")]
pub async fn modify_tenant(tenant_modify_req: Json<TenantModifyReq>, req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;

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
    values.push((IamTenant::UpdateUser, context.ident.account_id.as_str().into()));

    BIOSFuns::reldb()
        .exec(
            &Query::update().table(IamTenant::Table).values(values).and_where(Expr::col(IamTenant::Id).eq(context.ident.tenant_id.as_str())).done(),
            None,
        )
        .await?;
    BIOSResp::ok("", Some(&context))
}

#[get("/console/tenant/tenant")]
pub async fn get_tenant(req: HttpRequest) -> BIOSResponse {
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
        .and_where(Expr::tbl(IamTenant::Table, IamTenant::Id).eq(context.ident.tenant_id.as_str()))
        .done();
    let item = BIOSFuns::reldb().fetch_one::<TenantDetailResp>(&sql_builder, None).await?;
    BIOSResp::ok(item, Some(&context))
}

// ------------------------------------

#[post("/console/tenant/tenant/cert")]
pub async fn add_tenant_cert(tenant_cert_add_req: Json<TenantCertAddReq>, req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;
    let id = bios::basic::field::uuid();

    if BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamTenantCert::Id])
                .from(IamTenantCert::Table)
                .and_where(Expr::col(IamTenantCert::Category).eq(tenant_cert_add_req.category.as_str()))
                .and_where(Expr::col(IamTenantCert::RelTenantId).eq(context.ident.tenant_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSResp::err(IamOutput::TenantConsoleEntityCreateCheckExists(ObjectKind::TenantCert, "TenantCert"), Some(&context));
    }

    BIOSFuns::reldb()
        .exec(
            &Query::insert()
                .into_table(IamTenantCert::Table)
                .columns(vec![
                    IamTenantCert::Id,
                    IamTenantCert::CreateUser,
                    IamTenantCert::UpdateUser,
                    IamTenantCert::Category,
                    IamTenantCert::Version,
                    IamTenantCert::RelTenantId,
                ])
                .values_panic(vec![
                    id.as_str().into(),
                    context.ident.account_id.as_str().into(),
                    context.ident.account_id.as_str().into(),
                    tenant_cert_add_req.category.as_str().into(),
                    tenant_cert_add_req.version.into(),
                    context.ident.tenant_id.as_str().into(),
                ])
                .done(),
            None,
        )
        .await?;
    BIOSResp::ok(id, Some(&context))
}

#[put("/console/tenant/tenant/cert/{id}")]
pub async fn modify_tenant_cert(tenant_cert_modify_req: Json<TenantCertModifyReq>, req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;
    let id: String = req.match_info().get("id").unwrap().parse()?;

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamTenantCert::Id])
                .from(IamTenantCert::Table)
                .and_where(Expr::col(IamTenantCert::Id).eq(id.as_str()))
                .and_where(Expr::col(IamTenantCert::RelTenantId).eq(context.ident.tenant_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSResp::err(IamOutput::TenantConsoleEntityModifyCheckNotFound(ObjectKind::TenantCert, "TenantCert"), Some(&context));
    }

    let mut values = Vec::new();
    if let Some(version) = tenant_cert_modify_req.version {
        values.push((IamTenantCert::Version, version.into()));
    }
    values.push((IamTenantCert::UpdateUser, context.ident.account_id.as_str().into()));

    BIOSFuns::reldb()
        .exec(
            &Query::update()
                .table(IamTenantCert::Table)
                .values(values)
                .and_where(Expr::col(IamTenantCert::Id).eq(id.as_str()))
                .and_where(Expr::col(IamTenantCert::RelTenantId).eq(context.ident.tenant_id.as_str()))
                .done(),
            None,
        )
        .await?;
    BIOSResp::ok("", Some(&context))
}

#[get("/console/tenant/tenant/cert")]
pub async fn list_tenant_cert(req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;

    let create_user_table = Alias::new("create");
    let update_user_table = Alias::new("update");
    let sql_builder = Query::select()
        .columns(vec![
            (IamTenantCert::Table, IamTenantCert::Id),
            (IamTenantCert::Table, IamTenantCert::CreateTime),
            (IamTenantCert::Table, IamTenantCert::UpdateTime),
            (IamTenantCert::Table, IamTenantCert::Category),
            (IamTenantCert::Table, IamTenantCert::Version),
            (IamTenantCert::Table, IamTenantCert::RelTenantId),
        ])
        .expr_as(Expr::tbl(create_user_table.clone(), IamAccount::Name), Alias::new("create_user"))
        .expr_as(Expr::tbl(update_user_table.clone(), IamAccount::Name), Alias::new("update_user"))
        .from(IamTenantCert::Table)
        .join_as(
            JoinType::InnerJoin,
            IamAccount::Table,
            create_user_table.clone(),
            Expr::tbl(create_user_table, IamAccount::Id).equals(IamTenantCert::Table, IamTenantCert::CreateUser),
        )
        .join_as(
            JoinType::InnerJoin,
            IamAccount::Table,
            update_user_table.clone(),
            Expr::tbl(update_user_table, IamAccount::Id).equals(IamTenantCert::Table, IamTenantCert::UpdateUser),
        )
        .and_where(Expr::tbl(IamTenantCert::Table, IamTenantCert::RelTenantId).eq(context.ident.tenant_id.as_str()))
        .order_by(IamTenantCert::UpdateTime, Order::Desc)
        .done();
    let items = BIOSFuns::reldb().fetch_all::<TenantCertDetailResp>(&sql_builder, None).await?;
    BIOSResp::ok(items, Some(&context))
}

#[delete("/console/tenant/tenant/cert/{id}")]
pub async fn delete_tenant_cert(req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;
    let id: String = req.match_info().get("id").unwrap().parse()?;

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamTenantCert::Id])
                .from(IamTenantCert::Table)
                .and_where(Expr::col(IamTenantCert::Id).eq(id.as_str()))
                .and_where(Expr::col(IamTenantCert::RelTenantId).eq(context.ident.tenant_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSResp::err(IamOutput::TenantConsoleEntityDeleteCheckNotFound(ObjectKind::TenantCert, "TenantCert"), Some(&context));
    }

    let mut conn = BIOSFuns::reldb().conn().await;
    let mut tx = conn.begin().await?;

    let sql_builder = Query::select()
        .columns(IamTenantCert::iter().filter(|i| *i != IamTenantCert::Table))
        .from(IamTenantCert::Table)
        .and_where(Expr::col(IamTenantCert::Id).eq(id.as_str()))
        .and_where(Expr::col(IamTenantCert::RelTenantId).eq(context.ident.tenant_id.as_str()))
        .done();
    BIOSFuns::reldb().soft_del(IamTenantCert::Table, IamTenantCert::Id, &context.ident.account_id, &sql_builder, &mut tx).await?;
    tx.commit().await?;
    BIOSResp::ok("", Some(&context))
}

// ------------------------------------

#[post("/console/tenant/tenant/ident")]
pub async fn add_tenant_ident(tenant_ident_add_req: Json<TenantIdentAddReq>, req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;
    let id = bios::basic::field::uuid();

    if BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamTenantIdent::Id])
                .from(IamTenantIdent::Table)
                .and_where(Expr::col(IamTenantIdent::Kind).eq(tenant_ident_add_req.kind.to_string().to_lowercase()))
                .and_where(Expr::col(IamTenantIdent::RelTenantId).eq(context.ident.tenant_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSResp::err(IamOutput::TenantConsoleEntityCreateCheckExists(ObjectKind::TenantIdent, "TenantIdent"), Some(&context));
    }

    BIOSFuns::reldb()
        .exec(
            &Query::insert()
                .into_table(IamTenantIdent::Table)
                .columns(vec![
                    IamTenantIdent::Id,
                    IamTenantIdent::CreateUser,
                    IamTenantIdent::UpdateUser,
                    IamTenantIdent::Kind,
                    IamTenantIdent::ValidAkRuleNote,
                    IamTenantIdent::ValidAkRule,
                    IamTenantIdent::ValidSkRuleNote,
                    IamTenantIdent::ValidSkRule,
                    IamTenantIdent::ValidTime,
                    IamTenantIdent::RelTenantId,
                ])
                .values_panic(vec![
                    id.as_str().into(),
                    context.ident.account_id.as_str().into(),
                    context.ident.account_id.as_str().into(),
                    tenant_ident_add_req.kind.to_string().to_lowercase().into(),
                    tenant_ident_add_req.valid_ak_rule_note.as_deref().unwrap_or_default().into(),
                    tenant_ident_add_req.valid_ak_rule.as_deref().unwrap_or_default().into(),
                    tenant_ident_add_req.valid_sk_rule_note.as_deref().unwrap_or_default().into(),
                    tenant_ident_add_req.valid_sk_rule.as_deref().unwrap_or_default().into(),
                    tenant_ident_add_req.valid_time.into(),
                    context.ident.tenant_id.as_str().into(),
                ])
                .done(),
            None,
        )
        .await?;
    BIOSResp::ok(id, Some(&context))
}

#[put("/console/tenant/tenant/ident/{id}")]
pub async fn modify_tenant_ident(tenant_ident_modify_req: Json<TenantIdentModifyReq>, req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;
    let id: String = req.match_info().get("id").unwrap().parse()?;

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamTenantIdent::Id])
                .from(IamTenantIdent::Table)
                .and_where(Expr::col(IamTenantIdent::Id).eq(id.as_str()))
                .and_where(Expr::col(IamTenantIdent::RelTenantId).eq(context.ident.tenant_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSResp::err(IamOutput::TenantConsoleEntityModifyCheckNotFound(ObjectKind::TenantIdent, "TenantIdent"), Some(&context));
    }

    let mut values = Vec::new();
    if let Some(valid_ak_rule_note) = &tenant_ident_modify_req.valid_ak_rule_note {
        values.push((IamTenantIdent::ValidAkRuleNote, valid_ak_rule_note.to_string().as_str().into()));
    }
    if let Some(valid_ak_rule) = &tenant_ident_modify_req.valid_ak_rule {
        values.push((IamTenantIdent::ValidAkRule, valid_ak_rule.to_string().as_str().into()));
    }
    if let Some(valid_sk_rule_note) = &tenant_ident_modify_req.valid_sk_rule_note {
        values.push((IamTenantIdent::ValidSkRuleNote, valid_sk_rule_note.to_string().as_str().into()));
    }
    if let Some(valid_sk_rule) = &tenant_ident_modify_req.valid_sk_rule {
        values.push((IamTenantIdent::ValidSkRule, valid_sk_rule.to_string().as_str().into()));
    }
    if let Some(valid_time) = tenant_ident_modify_req.valid_time {
        values.push((IamTenantIdent::ValidTime, valid_time.into()));
    }
    values.push((IamTenantIdent::UpdateUser, context.ident.account_id.as_str().into()));

    BIOSFuns::reldb()
        .exec(
            &Query::update()
                .table(IamTenantIdent::Table)
                .values(values)
                .and_where(Expr::col(IamTenantIdent::Id).eq(id.as_str()))
                .and_where(Expr::col(IamTenantIdent::RelTenantId).eq(context.ident.tenant_id.as_str()))
                .done(),
            None,
        )
        .await?;
    BIOSResp::ok("", Some(&context))
}

#[get("/console/tenant/tenant/ident")]
pub async fn list_tenant_ident(req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;

    let create_user_table = Alias::new("create");
    let update_user_table = Alias::new("update");
    let sql_builder = Query::select()
        .columns(vec![
            (IamTenantIdent::Table, IamTenantIdent::Id),
            (IamTenantIdent::Table, IamTenantIdent::CreateTime),
            (IamTenantIdent::Table, IamTenantIdent::UpdateTime),
            (IamTenantIdent::Table, IamTenantIdent::Kind),
            (IamTenantIdent::Table, IamTenantIdent::ValidAkRuleNote),
            (IamTenantIdent::Table, IamTenantIdent::ValidAkRule),
            (IamTenantIdent::Table, IamTenantIdent::ValidSkRuleNote),
            (IamTenantIdent::Table, IamTenantIdent::ValidSkRule),
            (IamTenantIdent::Table, IamTenantIdent::ValidTime),
            (IamTenantIdent::Table, IamTenantIdent::RelTenantId),
        ])
        .expr_as(Expr::tbl(create_user_table.clone(), IamAccount::Name), Alias::new("create_user"))
        .expr_as(Expr::tbl(update_user_table.clone(), IamAccount::Name), Alias::new("update_user"))
        .from(IamTenantIdent::Table)
        .join_as(
            JoinType::InnerJoin,
            IamAccount::Table,
            create_user_table.clone(),
            Expr::tbl(create_user_table, IamAccount::Id).equals(IamTenantIdent::Table, IamTenantIdent::CreateUser),
        )
        .join_as(
            JoinType::InnerJoin,
            IamAccount::Table,
            update_user_table.clone(),
            Expr::tbl(update_user_table, IamAccount::Id).equals(IamTenantIdent::Table, IamTenantIdent::UpdateUser),
        )
        .and_where(Expr::tbl(IamTenantIdent::Table, IamTenantIdent::RelTenantId).eq(context.ident.tenant_id.as_str()))
        .order_by(IamTenantIdent::UpdateTime, Order::Desc)
        .done();
    let items = BIOSFuns::reldb().fetch_all::<TenantIdentDetailResp>(&sql_builder, None).await?;
    BIOSResp::ok(items, Some(&context))
}

#[delete("/console/tenant/tenant/ident/{id}")]
pub async fn delete_tenant_ident(req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;
    let id: String = req.match_info().get("id").unwrap().parse()?;

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamTenantIdent::Id])
                .from(IamTenantIdent::Table)
                .and_where(Expr::col(IamTenantIdent::Id).eq(id.as_str()))
                .and_where(Expr::col(IamTenantIdent::RelTenantId).eq(context.ident.tenant_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSResp::err(IamOutput::TenantConsoleEntityDeleteCheckNotFound(ObjectKind::TenantIdent, "TenantIdent"), Some(&context));
    }
    if BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![(IamAccountIdent::Table, IamAccountIdent::Id)])
                .from(IamAccountIdent::Table)
                .inner_join(
                    IamTenantIdent::Table,
                    Expr::tbl(IamTenantIdent::Table, IamTenantIdent::Kind).equals(IamAccountIdent::Table, IamAccountIdent::Kind),
                )
                .and_where(Expr::tbl(IamTenantIdent::Table, IamTenantIdent::Id).eq(id.as_str()))
                .and_where(Expr::tbl(IamTenantIdent::Table, IamTenantIdent::RelTenantId).eq(context.ident.tenant_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSResp::err(
            IamOutput::TenantConsoleEntityDeleteCheckExistAssociatedData(ObjectKind::TenantIdent, "AccountIdent"),
            Some(&context),
        );
    }

    let mut conn = BIOSFuns::reldb().conn().await;
    let mut tx = conn.begin().await?;

    let sql_builder = Query::select()
        .columns(IamTenantIdent::iter().filter(|i| *i != IamTenantIdent::Table))
        .from(IamTenantIdent::Table)
        .and_where(Expr::col(IamTenantIdent::Id).eq(id.as_str()))
        .and_where(Expr::col(IamTenantIdent::RelTenantId).eq(context.ident.tenant_id.as_str()))
        .done();
    BIOSFuns::reldb().soft_del(IamTenantIdent::Table, IamTenantIdent::Id, &context.ident.account_id, &sql_builder, &mut tx).await?;
    tx.commit().await?;
    BIOSResp::ok("", Some(&context))
}
