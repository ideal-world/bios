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

use actix_web::{post, put, HttpRequest};
use chrono::Utc;
use sea_query::{JoinType, Order, Query};
use sqlx::Connection;

use bios::basic::error::BIOSError;
use bios::db::reldb_client::SqlBuilderProcess;
use bios::web::resp_handler::{BIOSResp, BIOSRespHelper};
use bios::web::validate::json::Json;
use bios::BIOSFuns;

use crate::domain::auth_domain::{IamAuthPolicy, IamAuthPolicyObject, IamGroup, IamGroupNode, IamResource, IamResourceSubject, IamRole};
use crate::domain::ident_domain::{IamAccount, IamAccountApp, IamAccountIdent, IamApp, IamAppIdent, IamTenant, IamTenantCert, IamTenantIdent};
use crate::iam_config::WorkSpaceConfig;
use crate::process::basic_dto::{AccountIdentKind, CommonStatus};
use crate::process::common::auth_processor;
use crate::process::common::com_tenant_dto::TenantRegisterReq;
use crate::process::tenant_console::tc_app_dto::{AppModifyReq, AppQueryReq};

#[post("/common/tenant")]
pub async fn registerTenant(tenant_register_req: Json<TenantRegisterReq>, req: HttpRequest) -> BIOSResp {
    let iam_config = &BIOSFuns::ws_config::<WorkSpaceConfig>().iam;
    if !iam_config.allow_tenant_register {
        return BIOSRespHelper::bus_error(BIOSError::Conflict("Current settings don't allow self-registration of tenants".to_string()));
    }
    let mut conn = BIOSFuns::reldb().conn().await;
    let mut tx = conn.begin().await?;

    // Init Tenant
    let tenant_id = bios::basic::field::uuid();
    let app_id = bios::basic::field::uuid();
    let account_id = bios::basic::field::uuid();
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
                    tenant_id.clone().into(),
                    account_id.clone().into(),
                    account_id.clone().into(),
                    tenant_register_req.name.clone().into(),
                    tenant_register_req.icon.as_deref().unwrap_or(&"").into(),
                    tenant_register_req.allow_account_register.into(),
                    tenant_register_req.parameters.as_deref().unwrap_or(&"").into(),
                    CommonStatus::Enabled.to_string().to_lowercase().into(),
                ])
                .done(),
            Some(&mut tx),
        )
        .await?;
    // Init TenantIdent
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
                    bios::basic::field::uuid().into(),
                    account_id.clone().into(),
                    account_id.clone().into(),
                    AccountIdentKind::Username.to_string().to_lowercase().into(),
                    iam_config.security.default_valid_ak_rule_note.to_string().into(),
                    iam_config.security.default_valid_ak_rule.to_string().into(),
                    iam_config.security.default_valid_sk_rule_note.to_string().into(),
                    iam_config.security.default_valid_sk_rule.to_string().into(),
                    iam_config.security.default_valid_time_sec.into(),
                    tenant_id.clone().into(),
                ])
                .done(),
            Some(&mut tx),
        )
        .await?;
    // Init TenantCert
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
                    bios::basic::field::uuid().into(),
                    account_id.clone().into(),
                    account_id.clone().into(),
                    "".clone().into(),
                    0.into(),
                    tenant_id.clone().into(),
                ])
                .done(),
            Some(&mut tx),
        )
        .await?;
    // Init App
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
                    app_id.clone().into(),
                    account_id.clone().into(),
                    account_id.clone().into(),
                    tenant_register_req.app_name.clone().into(),
                    "".into(),
                    "".into(),
                    CommonStatus::Enabled.to_string().to_lowercase().into(),
                    tenant_id.clone().into(),
                ])
                .done(),
            Some(&mut tx),
        )
        .await?;
    // Init AppIdent
    let ak = bios::basic::security::key::generate_ak();
    let sk = bios::basic::security::key::generate_sk(&ak);
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
                    bios::basic::field::uuid().into(),
                    account_id.clone().into(),
                    account_id.clone().into(),
                    "".clone().into(),
                    ak.clone().into(),
                    sk.clone().into(),
                    0.into(),
                    app_id.clone().into(),
                    tenant_id.clone().into(),
                ])
                .done(),
            Some(&mut tx),
        )
        .await?;
    // Init Account
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
                    account_id.as_str().into(),
                    account_id.as_str().into(),
                    account_id.as_str().into(),
                    open_id.into(),
                    format!("{} 管理员", tenant_register_req.name).into(),
                    "".into(),
                    "".into(),
                    "".into(),
                    tenant_id.as_str().into(),
                    CommonStatus::Enabled.to_string().to_lowercase().into(),
                ])
                .done(),
            Some(&mut tx),
        )
        .await?;
    // Init AccountIdent
    let valid_end_time = auth_processor::valid_account_ident(
        &AccountIdentKind::Username.to_string().to_lowercase(),
        &tenant_register_req.account_username,
        &tenant_register_req.account_password,
        &tenant_id,
        Some(&mut tx),
    )
    .await?;
    let processed_sk = auth_processor::process_sk(
        &AccountIdentKind::Username.to_string().to_lowercase(),
        &tenant_register_req.account_username,
        &tenant_register_req.account_password,
        &tenant_id,
        &app_id,
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
                    bios::basic::field::uuid().into(),
                    account_id.clone().into(),
                    account_id.clone().into(),
                    AccountIdentKind::Username.to_string().to_lowercase().into(),
                    tenant_register_req.account_username.clone().into(),
                    processed_sk.into(),
                    Utc::now().timestamp().into(),
                    valid_end_time.into(),
                    account_id.clone().into(),
                    tenant_id.clone().into(),
                ])
                .done(),
            Some(&mut tx),
        )
        .await?;
    // Init AccountApp
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
                    bios::basic::field::uuid().into(),
                    account_id.clone().into(),
                    account_id.clone().into(),
                    account_id.clone().into(),
                    app_id.clone().into(),
                ])
                .done(),
            Some(&mut tx),
        )
        .await?;
    // Init AccountRole

    // Login

    // cache_processor::set_aksk(&ident_info.tenant_id, &ident_info.app_id, &ak, &sk, app_ident_add_req.valid_time).await?;

    tx.commit().await?;
    BIOSRespHelper::ok(tenant_id)
}
