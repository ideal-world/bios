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

use crate::domain::auth_domain::{IamAccountGroup, IamAccountRole, IamAuthPolicy, IamAuthPolicyObject, IamGroup, IamGroupNode, IamResource, IamResourceSubject, IamRole};
use crate::domain::ident_domain::{IamAccount, IamAccountApp, IamApp, IamAppIdent};
use crate::process::basic_dto::CommonStatus;
use crate::process::common::cache_processor;
use crate::process::tenant_console::tc_app_dto::{AppAddReq, AppDetailResp, AppModifyReq, AppQueryReq};

#[post("/console/tenant/app")]
pub async fn add_app(app_add_req: Json<AppAddReq>, req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;
    let id = bios::basic::field::uuid();

    BIOSFuns::reldb()
        .exec(
            &Query::insert()
                .into_table(IamApp::Table)
                .columns(vec![
                    IamApp::Id,
                    IamApp::CreateUser,
                    IamApp::UpdateUser,
                    IamApp::Name,
                    IamApp::Icon,
                    IamApp::Parameters,
                    IamApp::Status,
                    IamApp::RelTenantId,
                ])
                .values_panic(vec![
                    id.as_str().into(),
                    ident_info.account_id.as_str().into(),
                    ident_info.account_id.as_str().into(),
                    app_add_req.name.as_str().into(),
                    app_add_req.icon.as_deref().unwrap_or(&"").into(),
                    app_add_req.parameters.as_deref().unwrap_or(&"").into(),
                    CommonStatus::Enabled.to_string().to_lowercase().into(),
                    ident_info.tenant_id.into(),
                ])
                .done(),
            None,
        )
        .await?;
    BIOSRespHelper::ok(id)
}

#[put("/console/tenant/app/{id}")]
pub async fn modify_app(app_modify_req: Json<AppModifyReq>, req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;
    let id: String = req.match_info().get("id").unwrap().parse()?;

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamApp::Id])
                .from(IamApp::Table)
                .and_where(Expr::col(IamApp::Id).eq(id.as_str()))
                .and_where(Expr::col(IamApp::RelTenantId).eq(ident_info.tenant_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::NotFound("App not exists".to_string()));
    }

    let mut values = Vec::new();
    if let Some(name) = &app_modify_req.name {
        values.push((IamApp::Name, name.to_string().into()));
    }
    if let Some(parameters) = &app_modify_req.parameters {
        values.push((IamApp::Parameters, parameters.to_string().into()));
    }
    if let Some(icon) = &app_modify_req.icon {
        values.push((IamApp::Icon, icon.to_string().into()));
    }
    if let Some(status) = &app_modify_req.status {
        values.push((IamApp::Status, status.to_string().to_lowercase().into()));
    }
    values.push((IamApp::UpdateUser, ident_info.account_id.as_str().into()));

    let mut conn = BIOSFuns::reldb().conn().await;
    let mut tx = conn.begin().await?;

    BIOSFuns::reldb()
        .exec(
            &Query::update()
                .table(IamApp::Table)
                .values(values)
                .and_where(Expr::col(IamApp::Id).eq(id.as_str()))
                .and_where(Expr::col(IamApp::RelTenantId).eq(ident_info.tenant_id.as_str()))
                .done(),
            Some(&mut tx),
        )
        .await?;
    if let Some(status) = &app_modify_req.status {
        let aksks = BIOSFuns::reldb()
            .fetch_all::<AkSkInfoResp>(
                &Query::select()
                    .columns(vec![IamAppIdent::Ak, IamAppIdent::Sk, IamAppIdent::ValidTime])
                    .from(IamAppIdent::Table)
                    .and_where(Expr::col(IamAppIdent::RelAppId).eq(id.as_str()))
                    .done(),
                None,
            )
            .await?;
        match status {
            CommonStatus::Enabled => {
                for aksk_resp in aksks {
                    cache_processor::set_aksk(&ident_info.tenant_id, &id, &aksk_resp.ak, &aksk_resp.sk, aksk_resp.valid_time).await?;
                }
            }
            CommonStatus::Disabled => {
                for aksk_resp in aksks {
                    cache_processor::remove_aksk(&aksk_resp.ak).await?;
                }
            }
        }
    }
    tx.commit().await?;
    BIOSRespHelper::ok("")
}

#[get("/console/tenant/app")]
pub async fn list_app(query: VQuery<AppQueryReq>, req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;

    let create_user_table = Alias::new("create");
    let update_user_table = Alias::new("update");
    let sql_builder = Query::select()
        .columns(vec![
            (IamApp::Table, IamApp::Id),
            (IamApp::Table, IamApp::CreateTime),
            (IamApp::Table, IamApp::UpdateTime),
            (IamApp::Table, IamApp::Name),
            (IamApp::Table, IamApp::Icon),
            (IamApp::Table, IamApp::Parameters),
            (IamApp::Table, IamApp::Status),
            (IamApp::Table, IamApp::RelTenantId),
        ])
        .expr_as(Expr::tbl(create_user_table.clone(), IamAccount::Name), Alias::new("create_user"))
        .expr_as(Expr::tbl(update_user_table.clone(), IamAccount::Name), Alias::new("update_user"))
        .from(IamApp::Table)
        .join_as(
            JoinType::InnerJoin,
            IamAccount::Table,
            create_user_table.clone(),
            Expr::tbl(create_user_table, IamAccount::Id).equals(IamApp::Table, IamApp::CreateUser),
        )
        .join_as(
            JoinType::InnerJoin,
            IamAccount::Table,
            update_user_table.clone(),
            Expr::tbl(update_user_table, IamAccount::Id).equals(IamApp::Table, IamApp::UpdateUser),
        )
        .and_where_option(if let Some(name) = &query.name {
            Some(Expr::tbl(IamApp::Table, IamApp::Name).like(format!("%{}%", name).as_str()))
        } else {
            None
        })
        .and_where(Expr::tbl(IamApp::Table, IamApp::RelTenantId).eq(ident_info.tenant_id))
        .order_by(IamApp::UpdateTime, Order::Desc)
        .done();
    let items = BIOSFuns::reldb().pagination::<AppDetailResp>(&sql_builder, query.page_number, query.page_size, None).await?;
    BIOSRespHelper::ok(items)
}

#[delete("/console/tenant/app/{id}")]
pub async fn delete_app(req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;
    let id: String = req.match_info().get("id").unwrap().parse()?;

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamApp::Id])
                .from(IamApp::Table)
                .and_where(Expr::col(IamApp::Id).eq(id.as_str()))
                .and_where(Expr::col(IamApp::RelTenantId).eq(ident_info.tenant_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::NotFound("App not exists".to_string()));
    }

    let mut conn = BIOSFuns::reldb().conn().await;
    let mut tx = conn.begin().await?;

    let aksks = BIOSFuns::reldb()
        .fetch_all::<AkSkInfoResp>(
            &Query::select()
                .columns(vec![IamAppIdent::Ak, IamAppIdent::Sk, IamAppIdent::ValidTime])
                .from(IamAppIdent::Table)
                .and_where(Expr::col(IamAppIdent::RelAppId).eq(id.as_str()))
                .done(),
            Some(&mut tx),
        )
        .await?;
    // Delete IamAppIdent
    BIOSFuns::reldb()
        .soft_del(
            IamAppIdent::Table,
            IamAppIdent::Id,
            &ident_info.account_id,
            &Query::select()
                .columns(IamAppIdent::iter().filter(|i| *i != IamAppIdent::Table))
                .from(IamAppIdent::Table)
                .and_where(Expr::col(IamAppIdent::RelAppId).eq(id.as_str()))
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
                .and_where(Expr::col(IamAccountApp::RelAppId).eq(id.as_str()))
                .done(),
            &mut tx,
        )
        .await?;
    // Delete IamGroup
    let group_ids = BIOSFuns::reldb()
        .fetch_all::<IdResp>(
            &Query::select().columns(vec![IamGroup::Id]).from(IamGroup::Table).and_where(Expr::col(IamGroup::RelAppId).eq(id.as_str())).done(),
            Some(&mut tx),
        )
        .await?
        .iter()
        .map(|record| record.id.to_string())
        .collect::<Vec<String>>();
    BIOSFuns::reldb()
        .soft_del(
            IamGroup::Table,
            IamGroup::Id,
            &ident_info.account_id,
            &Query::select().columns(IamGroup::iter().filter(|i| *i != IamGroup::Table)).from(IamGroup::Table).and_where(Expr::col(IamGroup::RelAppId).eq(id.as_str())).done(),
            &mut tx,
        )
        .await?;
    // Delete IamGroupNode
    let group_node_ids = BIOSFuns::reldb()
        .fetch_all::<IdResp>(
            &Query::select().columns(vec![IamGroupNode::Id]).from(IamGroupNode::Table).and_where(Expr::col(IamGroupNode::RelGroupId).is_in(group_ids.clone())).done(),
            Some(&mut tx),
        )
        .await?
        .iter()
        .map(|record| record.id.to_string())
        .collect::<Vec<String>>();
    BIOSFuns::reldb()
        .soft_del(
            IamGroupNode::Table,
            IamGroupNode::Id,
            &ident_info.account_id,
            &Query::select()
                .columns(IamGroupNode::iter().filter(|i| *i != IamGroupNode::Table))
                .from(IamGroupNode::Table)
                .and_where(Expr::col(IamGroupNode::RelGroupId).is_in(group_ids))
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
                .and_where(Expr::col(IamAccountGroup::RelGroupNodeId).is_in(group_node_ids))
                .done(),
            &mut tx,
        )
        .await?;
    // Delete IamRole
    let role_ids = BIOSFuns::reldb()
        .fetch_all::<IdResp>(
            &Query::select().columns(vec![IamRole::Id]).from(IamRole::Table).and_where(Expr::col(IamRole::RelAppId).eq(id.as_str())).done(),
            Some(&mut tx),
        )
        .await?
        .iter()
        .map(|record| record.id.to_string())
        .collect::<Vec<String>>();
    BIOSFuns::reldb()
        .soft_del(
            IamRole::Table,
            IamRole::Id,
            &ident_info.account_id,
            &Query::select().columns(IamRole::iter().filter(|i| *i != IamRole::Table)).from(IamRole::Table).and_where(Expr::col(IamRole::RelAppId).eq(id.as_str())).done(),
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
                .and_where(Expr::col(IamAccountRole::RelRoleId).is_in(role_ids))
                .done(),
            &mut tx,
        )
        .await?;
    // Delete IamResourceSubject
    BIOSFuns::reldb()
        .soft_del(
            IamResourceSubject::Table,
            IamResourceSubject::Id,
            &ident_info.account_id,
            &Query::select()
                .columns(IamResourceSubject::iter().filter(|i| *i != IamResourceSubject::Table))
                .from(IamResourceSubject::Table)
                .and_where(Expr::col(IamResourceSubject::RelAppId).eq(id.as_str()))
                .done(),
            &mut tx,
        )
        .await?;
    // Delete IamResource
    BIOSFuns::reldb()
        .soft_del(
            IamResource::Table,
            IamResource::Id,
            &ident_info.account_id,
            &Query::select()
                .columns(IamResource::iter().filter(|i| *i != IamResource::Table))
                .from(IamResource::Table)
                .and_where(Expr::col(IamResource::RelAppId).eq(id.as_str()))
                .done(),
            &mut tx,
        )
        .await?;
    // Delete IamAuthPolicy
    let auth_policy_ids = BIOSFuns::reldb()
        .fetch_all::<IdResp>(
            &Query::select().columns(vec![IamAuthPolicy::Id]).from(IamAuthPolicy::Table).and_where(Expr::col(IamAuthPolicy::RelAppId).eq(id.as_str())).done(),
            Some(&mut tx),
        )
        .await?
        .iter()
        .map(|record| record.id.to_string())
        .collect::<Vec<String>>();
    BIOSFuns::reldb()
        .soft_del(
            IamAuthPolicy::Table,
            IamAuthPolicy::Id,
            &ident_info.account_id,
            &Query::select()
                .columns(IamAuthPolicy::iter().filter(|i| *i != IamAuthPolicy::Table))
                .from(IamAuthPolicy::Table)
                .and_where(Expr::col(IamAuthPolicy::RelAppId).eq(id.as_str()))
                .done(),
            &mut tx,
        )
        .await?;
    // Delete IamAuthPolicySubject
    BIOSFuns::reldb()
        .soft_del(
            IamAuthPolicyObject::Table,
            IamAuthPolicyObject::Id,
            &ident_info.account_id,
            &Query::select()
                .columns(IamAuthPolicyObject::iter().filter(|i| *i != IamAuthPolicyObject::Table))
                .from(IamAuthPolicyObject::Table)
                .and_where(Expr::col(IamAuthPolicyObject::RelAuthPolicyId).is_in(auth_policy_ids))
                .done(),
            &mut tx,
        )
        .await?;
    // Delete IamApp
    let sql_builder = Query::select().columns(IamApp::iter().filter(|i| *i != IamApp::Table)).from(IamApp::Table).and_where(Expr::col(IamApp::Id).eq(id.as_str())).done();
    BIOSFuns::reldb().soft_del(IamApp::Table, IamApp::Id, &ident_info.account_id, &sql_builder, &mut tx).await?;
    // Remove aksk info at redis cache
    for aksk_resp in aksks {
        cache_processor::remove_aksk(&aksk_resp.ak).await?;
    }
    tx.commit().await?;
    BIOSRespHelper::ok("")
}

#[derive(sqlx::FromRow, serde::Deserialize)]
pub struct AkSkInfoResp {
    pub ak: String,
    pub sk: String,
    pub valid_time: i64,
}

#[derive(sqlx::FromRow, serde::Deserialize)]
pub struct IdResp {
    pub id: String,
}
