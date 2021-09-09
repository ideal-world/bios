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
use chrono::Utc;
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

use crate::domain::ident_domain::{IamAccount, IamApp, IamAppIdent, IamTenant};
use crate::iam_config::WorkSpaceConfig;
use crate::process::basic_dto::CommonStatus;
use crate::process::system_console::sc_tenant_dto::{TenantAddReq, TenantDetailResp, TenantModifyReq, TenantQueryReq};

#[post("/console/system/tenant")]
pub async fn add_tenant(tenant_add_req: Json<TenantAddReq>, req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;
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
                    id.clone().into(),
                    ident_info.account_id.clone().into(),
                    ident_info.account_id.clone().into(),
                    tenant_add_req.name.clone().into(),
                    tenant_add_req.icon.clone().unwrap_or_default().into(),
                    tenant_add_req.allow_account_register.into(),
                    tenant_add_req.parameters.clone().unwrap_or_default().into(),
                    CommonStatus::Enabled.to_string().to_lowercase().into(),
                ])
                .done(),
            None,
        )
        .await?;
    BIOSRespHelper::ok(id)
}

#[put("/console/system/tenant/{id}")]
pub async fn modify_tenant(tenant_modify_req: Json<TenantModifyReq>, req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;
    let id: String = req.match_info().get("id").unwrap().parse()?;

    let mut values = Vec::new();
    if let Some(name) = &tenant_modify_req.name {
        values.push((IamTenant::Name, name.to_string().into()));
    }
    if let Some(icon) = &tenant_modify_req.icon {
        values.push((IamTenant::Icon, icon.to_string().into()));
    }
    if let Some(allow_account_register) = tenant_modify_req.allow_account_register {
        values.push((IamTenant::AllowAccountRegister, allow_account_register.into()));
    }
    if let Some(parameters) = &tenant_modify_req.parameters {
        values.push((IamTenant::Parameters, parameters.to_string().into()));
    }
    if let Some(status) = &tenant_modify_req.status {
        values.push((IamTenant::Status, status.to_string().to_lowercase().into()));
    }
    values.push((IamTenant::UpdateUser, ident_info.account_id.clone().into()));

    let mut conn = BIOSFuns::reldb().conn().await;
    let mut tx = conn.begin().await?;

    BIOSFuns::reldb()
        .exec(
            &Query::update().table(IamTenant::Table).values(values).and_where(Expr::col(IamTenant::Id).eq(id.clone())).done(),
            Some(&mut tx),
        )
        .await?;
    if let Some(status) = &tenant_modify_req.status {
        let enabled_aksks = BIOSFuns::reldb()
            .fetch_all::<AkSkInfoResp>(
                &Query::select()
                    .columns(vec![(IamApp::Table, IamApp::Id)])
                    .columns(vec![(IamAppIdent::Table, IamAppIdent::Ak)])
                    .columns(vec![(IamAppIdent::Table, IamAppIdent::Sk)])
                    .columns(vec![(IamAppIdent::Table, IamAppIdent::ValidTime)])
                    .from(IamApp::Table)
                    .inner_join(IamAppIdent::Table, Expr::tbl(IamAppIdent::Table, IamAppIdent::RelAppId).equals(IamApp::Table, IamApp::Id))
                    .and_where(Expr::tbl(IamApp::Table, IamApp::Status).eq(CommonStatus::Enabled.to_string().to_lowercase()))
                    .and_where(Expr::tbl(IamApp::Table, IamApp::RelTenantId).eq(id.clone()))
                    .done(),
                None,
            )
            .await?;
        match status {
            CommonStatus::Enabled => {
                for aksk_resp in enabled_aksks {
                    BIOSFuns::cache()
                        .set_ex(
                            format!("{}{}", &BIOSFuns::ws_config::<WorkSpaceConfig>().iam.cache_aksk, aksk_resp.ak).as_str(),
                            format!("{}:{}:{}", aksk_resp.sk, id.clone(), aksk_resp.app_id).as_str(),
                            (aksk_resp.valid_time - Utc::now().timestamp()) as usize,
                        )
                        .await?;
                }
            }
            CommonStatus::Disabled => {
                for aksk_resp in enabled_aksks {
                    BIOSFuns::cache().del(format!("{}{}", &BIOSFuns::ws_config::<WorkSpaceConfig>().iam.cache_aksk, aksk_resp.ak).as_str()).await?;
                }
            }
        }
    }
    tx.commit().await?;

    BIOSRespHelper::ok("")
}

#[get("/console/system/tenant")]
pub async fn list_tenant(query: VQuery<TenantQueryReq>) -> BIOSResp {
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
    BIOSRespHelper::ok(items)
}

#[delete("/console/system/tenant/{id}")]
pub async fn delete_tenant(req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;
    let id: String = req.match_info().get("id").unwrap().parse()?;

    if BIOSFuns::reldb()
        .exists(
            &Query::select().columns(vec![IamApp::Id]).from(IamApp::Table).and_where(Expr::col(IamApp::RelTenantId).eq(id.clone())).done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::Conflict("Please delete the associated [app] data first".to_owned()));
    }

    let mut conn = BIOSFuns::reldb().conn().await;
    let mut tx = conn.begin().await?;

    let sql_builder =
        Query::select().columns(IamTenant::iter().filter(|i| *i != IamTenant::Table)).from(IamTenant::Table).and_where(Expr::col(IamTenant::Id).eq(id.clone())).done();
    BIOSFuns::reldb().soft_del::<TenantDetailResp, _, _>(IamTenant::Table, IamTenant::Id, &ident_info.account_id, &sql_builder, &mut tx).await?;

    tx.commit().await?;
    BIOSRespHelper::ok("")
}

#[derive(sqlx::FromRow, serde::Deserialize)]
pub struct AkSkInfoResp {
    pub ak: String,
    pub sk: String,
    pub valid_time: i64,
    pub app_id: String,
}
