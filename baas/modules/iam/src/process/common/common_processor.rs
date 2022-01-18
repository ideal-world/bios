/*
 * Copyright 2022. the original author or authors.
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
use itertools::Itertools;
use sea_query::{Alias, Cond, Expr, Order, Query};
use sqlx::Connection;

use bios::basic::dto::BIOSResp;
use bios::basic::dto::{BIOSContext, IdentInfo, Trace};
use bios::basic::error::BIOSError;
use bios::basic::result::BIOSResult;
use bios::db::reldb_client::SqlBuilderProcess;
use bios::web::basic_processor::{extract_context, extract_context_with_account};
use bios::web::resp_handler::BIOSResponse;
use bios::web::validate::json::Json;
use bios::BIOSFuns;

use crate::domain::auth_domain::{IamAccountGroup, IamAccountRole, IamAuthPolicy, IamAuthPolicyObject, IamGroup, IamGroupNode, IamResource, IamResourceSubject, IamRole};
use crate::domain::ident_domain::{IamAccount, IamAccountApp, IamAccountIdent, IamApp, IamAppIdent, IamTenant, IamTenantCert, IamTenantIdent};
use crate::iam_config::WorkSpaceConfig;
use crate::iam_constant::{IamOutput, ObjectKind};
use crate::process::basic_dto::{AccountIdentKind, AuthObjectKind, AuthObjectOperatorKind, CommonStatus, ExposeKind, OptActionKind, ResourceKind};
use crate::process::common::com_account_dto::{AccountChangeReq, AccountIdentChangeReq, AccountInfoDetailResp, AccountLoginReq, AccountRegisterReq};
use crate::process::common::com_resource_dto::ResourceDetailResp;
use crate::process::common::com_tenant_dto::TenantRegisterReq;
use crate::process::common::{auth_processor, cache_processor};

#[post("/common/tenant")]
pub async fn register_tenant(tenant_register_req: Json<TenantRegisterReq>, req: HttpRequest) -> BIOSResponse {
    let context = extract_context(&req)?;
    let iam_config = &BIOSFuns::ws_config::<WorkSpaceConfig>().iam;
    if !iam_config.allow_tenant_register {
        return BIOSResp::err(BIOSError::Conflict("Current settings don't allow self-registration of tenants".to_string()), Some(&context));
    }
    let mut conn = BIOSFuns::reldb().conn().await;
    let mut tx = conn.begin().await?;

    // Init Tenant

    let context = BIOSContext {
        trace: Trace { ..context.trace },
        ident: IdentInfo {
            account_id: bios::basic::field::uuid(),
            app_id: bios::basic::field::uuid(),
            tenant_id: bios::basic::field::uuid(),
            ..context.ident
        },
        lang: context.lang,
    };

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
                    context.ident.tenant_id.as_str().into(),
                    context.ident.account_id.as_str().into(),
                    context.ident.account_id.as_str().into(),
                    tenant_register_req.name.as_str().into(),
                    tenant_register_req.icon.as_deref().unwrap_or_default().into(),
                    tenant_register_req.allow_account_register.into(),
                    tenant_register_req.parameters.as_deref().unwrap_or_default().into(),
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
                    context.ident.account_id.as_str().into(),
                    context.ident.account_id.as_str().into(),
                    AccountIdentKind::Username.to_string().to_lowercase().into(),
                    iam_config.security.default_valid_ak_rule_note.as_str().into(),
                    iam_config.security.default_valid_ak_rule.as_str().into(),
                    iam_config.security.default_valid_sk_rule_note.as_str().into(),
                    iam_config.security.default_valid_sk_rule.as_str().into(),
                    iam_config.security.default_valid_time_sec.into(),
                    context.ident.tenant_id.as_str().into(),
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
                    context.ident.account_id.as_str().into(),
                    context.ident.account_id.as_str().into(),
                    "".into(),
                    1.into(),
                    context.ident.tenant_id.as_str().into(),
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
                    context.ident.app_id.as_str().into(),
                    context.ident.account_id.as_str().into(),
                    context.ident.account_id.as_str().into(),
                    tenant_register_req.app_name.as_str().into(),
                    "".into(),
                    "".into(),
                    CommonStatus::Enabled.to_string().to_lowercase().into(),
                    context.ident.tenant_id.as_str().into(),
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
                    context.ident.account_id.as_str().into(),
                    context.ident.account_id.as_str().into(),
                    "".into(),
                    ak.as_str().into(),
                    sk.as_str().into(),
                    i64::MAX.into(),
                    context.ident.app_id.as_str().into(),
                    context.ident.tenant_id.as_str().into(),
                ])
                .done(),
            Some(&mut tx),
        )
        .await?;

    // Init Account
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
                    context.ident.account_id.as_str().into(),
                    context.ident.account_id.as_str().into(),
                    context.ident.account_id.as_str().into(),
                    format!("ao_{}", bios::basic::field::uuid()).into(),
                    format!("{}管理员", tenant_register_req.name).into(),
                    "".into(),
                    "".into(),
                    "".into(),
                    context.ident.tenant_id.as_str().into(),
                    CommonStatus::Enabled.to_string().to_lowercase().into(),
                ])
                .done(),
            Some(&mut tx),
        )
        .await?;

    // Init AccountIdent
    let valid_end_time = auth_processor::valid_account_ident(
        &AccountIdentKind::Username,
        &tenant_register_req.account_username,
        &tenant_register_req.account_password,
        Some(&mut tx),
        &context,
    )
    .await?;
    let processed_sk = auth_processor::process_sk(
        &AccountIdentKind::Username,
        &tenant_register_req.account_username,
        &tenant_register_req.account_password,
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
                    bios::basic::field::uuid().into(),
                    context.ident.account_id.as_str().into(),
                    context.ident.account_id.as_str().into(),
                    AccountIdentKind::Username.to_string().to_lowercase().into(),
                    tenant_register_req.account_username.as_str().into(),
                    processed_sk.into(),
                    Utc::now().timestamp().into(),
                    valid_end_time.into(),
                    context.ident.account_id.as_str().into(),
                    context.ident.tenant_id.as_str().into(),
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
                    context.ident.account_id.as_str().into(),
                    context.ident.account_id.as_str().into(),
                    context.ident.account_id.as_str().into(),
                    context.ident.app_id.as_str().into(),
                ])
                .done(),
            Some(&mut tx),
        )
        .await?;

    // Init Role And AccountRole
    let tenant_role_id = auth_processor::init_account_role(
        &iam_config.security.tenant_admin_role_code.as_str(),
        &iam_config.security.tenant_admin_role_name.as_str(),
        &mut tx,
        &context,
    )
    .await?;
    let app_role_id = auth_processor::init_account_role(
        &iam_config.security.app_admin_role_code.as_str(),
        &iam_config.security.app_admin_role_name.as_str(),
        &mut tx,
        &context,
    )
    .await?;

    // Init ResourceSubject
    let resource_subject_menu_id = auth_processor::init_resource_subject(
        &ResourceKind::Menu,
        format!("https://{}/common/resource/menu/{}", iam_config.service_name, context.ident.app_id).as_str(),
        format!("{}菜单", tenant_register_req.app_name).as_str(),
        &mut tx,
        &context,
    )
    .await?;
    let resource_subject_element_id = auth_processor::init_resource_subject(
        &ResourceKind::Element,
        format!("https://{}/common/resource/element/{}", iam_config.service_name, context.ident.app_id).as_str(),
        format!("{}元素", tenant_register_req.app_name).as_str(),
        &mut tx,
        &context,
    )
    .await?;

    // Init Resource
    let resource_pub_menu_id = auth_processor::init_resource("/pub/**", "租户共享菜单", &resource_subject_menu_id, &ExposeKind::Tenant, &mut tx, &context).await?;
    let resource_pub_element_id = auth_processor::init_resource("/pub/**", "租户共享元素", &resource_subject_element_id, &ExposeKind::Tenant, &mut tx, &context).await?;

    // Fetch IAM console resources
    let resource_console_tenant_id = BIOSFuns::reldb()
        .fetch_one_json(
            &Query::select()
                .column(IamResource::Id)
                .from(IamResource::Table)
                .and_where(Expr::col(IamResource::PathAndQuery).eq("/console/tenant/**"))
                .order_by(IamResource::CreateTime, Order::Asc)
                .done(),
            Some(&mut tx),
        )
        .await?;
    let resource_console_tenant_id = resource_console_tenant_id["id"].as_str().unwrap();

    let resource_console_app_id = BIOSFuns::reldb()
        .fetch_one_json(
            &Query::select()
                .column(IamResource::Id)
                .from(IamResource::Table)
                .and_where(Expr::col(IamResource::PathAndQuery).eq("/console/app/**"))
                .order_by(IamResource::CreateTime, Order::Asc)
                .done(),
            Some(&mut tx),
        )
        .await?;
    let resource_console_app_id = resource_console_app_id["id"].as_str().unwrap();

    // Init Auth
    auth_processor::init_auth(
        vec![(
            resource_console_tenant_id,
            vec![
                &OptActionKind::Get,
                &OptActionKind::Post,
                &OptActionKind::Put,
                &OptActionKind::Delete,
                &OptActionKind::Patch,
            ],
        )],
        "租户控制台",
        &AuthObjectKind::Role,
        &tenant_role_id,
        &mut tx,
        &context,
    )
    .await?;
    auth_processor::init_auth(
        vec![(
            resource_console_app_id,
            vec![
                &OptActionKind::Get,
                &OptActionKind::Post,
                &OptActionKind::Put,
                &OptActionKind::Delete,
                &OptActionKind::Patch,
            ],
        )],
        "应用控制台",
        &AuthObjectKind::Role,
        &app_role_id,
        &mut tx,
        &context,
    )
    .await?;

    auth_processor::init_auth(
        vec![(&resource_pub_menu_id, vec![&OptActionKind::Get])],
        "租户共享菜单",
        &AuthObjectKind::Tenant,
        &context.ident.tenant_id,
        &mut tx,
        &context,
    )
    .await?;
    auth_processor::init_auth(
        vec![(&resource_pub_element_id, vec![&OptActionKind::Get])],
        "租户共享元素",
        &AuthObjectKind::Tenant,
        &context.ident.tenant_id,
        &mut tx,
        &context,
    )
    .await?;
    let auth_policy_ids = BIOSFuns::reldb().fetch_all_json(&Query::select().column(IamAuthPolicy::Id).from(IamAuthPolicy::Table).done(), Some(&mut tx)).await?;
    for id in auth_policy_ids {
        cache_processor::rebuild_auth_policy(id["id"].as_str().unwrap(), &mut tx, &context).await?;
    }
    cache_processor::set_aksk(&context.ident.tenant_id, &context.ident.app_id, &ak, &sk, i64::MAX, &context).await?;
    tx.commit().await?;

    // Login
    let ident_info = do_login(tenant_register_req.account_username.as_str(), valid_end_time, "", &context).await?;
    BIOSResp::ok(ident_info, Some(&context))
}

#[post("/common/account")]
pub async fn register_account_normal(account_register_req: Json<AccountRegisterReq>, req: HttpRequest) -> BIOSResponse {
    let context = extract_context(&req)?;
    let ident_info = register_account(&account_register_req, &context).await?;
    BIOSResp::ok(ident_info, Some(&context))
}

pub async fn register_account(account_register_req: &AccountRegisterReq, context: &BIOSContext) -> BIOSResult<IdentInfo> {
    let tenant_id = BIOSFuns::reldb()
        .fetch_optional_json(
            &Query::select()
                .column((IamTenant::Table, IamTenant::Id))
                .from(IamApp::Table)
                .inner_join(IamTenant::Table, Expr::tbl(IamTenant::Table, IamTenant::Id).equals(IamApp::Table, IamApp::RelTenantId))
                .and_where(Expr::tbl(IamApp::Table, IamApp::Id).eq(account_register_req.rel_app_id.as_str()))
                .and_where(Expr::tbl(IamApp::Table, IamApp::Status).eq(CommonStatus::Enabled.to_string().to_lowercase()))
                .and_where(Expr::tbl(IamTenant::Table, IamTenant::Status).eq(CommonStatus::Enabled.to_string().to_lowercase()))
                .done(),
            None,
        )
        .await?;
    if tenant_id.is_none() {
        return IamOutput::CommonEntityCreateCheckNotFound(ObjectKind::Account, "App")?;
    }
    let tenant_id = tenant_id.unwrap();
    let tenant_id = tenant_id["id"].as_str().unwrap();
    let account_id = bios::basic::field::uuid();

    let context = BIOSContext {
        trace: Trace { ..context.trace.clone() },
        ident: IdentInfo {
            account_id,
            app_id: account_register_req.rel_app_id.to_string(),
            tenant_id: tenant_id.to_string(),
            ..context.ident.clone()
        },
        lang: context.lang.clone(),
    };

    if BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamAccountIdent::Id])
                .from(IamAccountIdent::Table)
                .and_where(Expr::col(IamAccountIdent::Kind).eq(account_register_req.kind.to_string().to_lowercase()))
                .and_where(Expr::col(IamAccountIdent::Ak).eq(account_register_req.ak.as_str()))
                .and_where(Expr::col(IamAccountIdent::RelTenantId).eq(context.ident.tenant_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return IamOutput::CommonEntityCreateCheckExists(ObjectKind::AccountIdent, "AccountIdent")?;
    }

    let mut conn = BIOSFuns::reldb().conn().await;
    let mut tx = conn.begin().await?;

    // Init Account
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
                    context.ident.account_id.as_str().into(),
                    context.ident.account_id.as_str().into(),
                    context.ident.account_id.as_str().into(),
                    format!("ao_{}", bios::basic::field::uuid()).into(),
                    account_register_req.name.as_str().into(),
                    account_register_req.avatar.as_deref().unwrap_or_default().into(),
                    account_register_req.parameters.as_deref().unwrap_or_default().into(),
                    "".into(),
                    context.ident.tenant_id.as_str().into(),
                    CommonStatus::Enabled.to_string().to_lowercase().into(),
                ])
                .done(),
            Some(&mut tx),
        )
        .await?;
    // Init AccountIdent
    let valid_end_time = auth_processor::valid_account_ident(&AccountIdentKind::Username, &account_register_req.ak, &account_register_req.sk, Some(&mut tx), &context).await?;
    let processed_sk = auth_processor::process_sk(&AccountIdentKind::Username, &account_register_req.ak, &account_register_req.sk, &context).await?;
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
                    context.ident.account_id.as_str().into(),
                    context.ident.account_id.as_str().into(),
                    AccountIdentKind::Username.to_string().to_lowercase().into(),
                    account_register_req.ak.as_str().into(),
                    processed_sk.into(),
                    Utc::now().timestamp().into(),
                    valid_end_time.into(),
                    context.ident.account_id.as_str().into(),
                    context.ident.tenant_id.as_str().into(),
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
                    context.ident.account_id.as_str().into(),
                    context.ident.account_id.as_str().into(),
                    context.ident.account_id.as_str().into(),
                    context.ident.app_id.as_str().into(),
                ])
                .done(),
            Some(&mut tx),
        )
        .await?;
    tx.commit().await?;
    // Login
    let ident_info = do_login(account_register_req.ak.as_str(), valid_end_time, "", &context).await?;
    Ok(ident_info)
}

#[post("/common/login")]
pub async fn login_normal(account_login_req: Json<AccountLoginReq>, req: HttpRequest) -> BIOSResponse {
    let context = extract_context(&req)?;
    let ident_info = login(&account_login_req, &context).await?;
    BIOSResp::ok(ident_info, Some(&context))
}

pub async fn login(account_login_req: &AccountLoginReq, context: &BIOSContext) -> BIOSResult<IdentInfo> {
    log::info!(
        "Login : [{}] kind = {}, ak = {}",
        account_login_req.rel_app_id,
        account_login_req.kind.to_string().to_lowercase(),
        account_login_req.ak
    );
    let now = Utc::now().timestamp();
    let account_info = BIOSFuns::reldb()
        .fetch_optional_json(
            &Query::select()
                .column((IamAccountIdent::Table, IamAccountIdent::Sk))
                .column((IamAccountIdent::Table, IamAccountIdent::ValidEndTime))
                .column((IamAccount::Table, IamAccount::Id))
                .expr_as(Expr::tbl(IamApp::Table, IamApp::Id), Alias::new("app_id"))
                .expr_as(Expr::tbl(IamTenant::Table, IamTenant::Id), Alias::new("tenant_id"))
                .from(IamAccountIdent::Table)
                .inner_join(
                    IamAccount::Table,
                    Expr::tbl(IamAccount::Table, IamAccount::Id).equals(IamAccountIdent::Table, IamAccountIdent::RelAccountId),
                )
                .inner_join(
                    IamAccountApp::Table,
                    Expr::tbl(IamAccountApp::Table, IamAccountApp::RelAccountId).equals(IamAccount::Table, IamAccount::Id),
                )
                .inner_join(IamApp::Table, Expr::tbl(IamApp::Table, IamApp::Id).equals(IamAccountApp::Table, IamAccountApp::RelAppId))
                .inner_join(IamTenant::Table, Expr::tbl(IamTenant::Table, IamTenant::Id).equals(IamApp::Table, IamApp::RelTenantId))
                .inner_join(
                    IamTenantCert::Table,
                    Expr::tbl(IamTenantCert::Table, IamTenantCert::RelTenantId).equals(IamTenant::Table, IamTenant::Id),
                )
                .and_where(Expr::tbl(IamAccountIdent::Table, IamAccountIdent::Kind).eq(account_login_req.kind.to_string().to_lowercase()))
                .and_where(Expr::tbl(IamAccountIdent::Table, IamAccountIdent::Ak).eq(account_login_req.ak.as_str()))
                .and_where(Expr::tbl(IamAccountIdent::Table, IamAccountIdent::ValidStartTime).lte(now))
                .and_where(Expr::tbl(IamAccountIdent::Table, IamAccountIdent::ValidEndTime).gte(now))
                .and_where(Expr::tbl(IamApp::Table, IamApp::Id).eq(account_login_req.rel_app_id.as_str()))
                .and_where(Expr::tbl(IamTenantCert::Table, IamTenantCert::Category).eq(account_login_req.cert_category.as_deref().unwrap_or_default()))
                .and_where(Expr::tbl(IamAccount::Table, IamAccount::Status).eq(CommonStatus::Enabled.to_string().to_lowercase()))
                .and_where(Expr::tbl(IamApp::Table, IamApp::Status).eq(CommonStatus::Enabled.to_string().to_lowercase()))
                .and_where(Expr::tbl(IamTenant::Table, IamTenant::Status).eq(CommonStatus::Enabled.to_string().to_lowercase()))
                .done(),
            None,
        )
        .await?;
    if account_info.is_none() {
        log::warn!(
            "Login Fail: [{}] kind = {}, ak = {} doesn't exist or has expired",
            account_login_req.rel_app_id,
            account_login_req.kind.to_string().to_lowercase(),
            account_login_req.ak
        );
        return IamOutput::CommonLoginCheckAccountNotFoundOrExpired(account_login_req.ak.to_string())?;
    }
    let account_info = account_info.unwrap();
    let stored_sk = account_info["sk"].as_str().unwrap();
    let valid_end_time = account_info["valid_end_time"].as_i64().unwrap();

    let context = BIOSContext {
        trace: Trace { ..context.trace.clone() },
        ident: IdentInfo {
            account_id: account_info["id"].as_str().unwrap().to_string(),
            app_id: account_info["app_id"].as_str().unwrap().to_string(),
            tenant_id: account_info["tenant_id"].as_str().unwrap().to_string(),
            ..context.ident.clone()
        },
        lang: context.lang.to_string(),
    };

    auth_processor::validate_sk(&account_login_req.kind, account_login_req.ak.as_str(), account_login_req.sk.as_str(), stored_sk, &context).await?;
    log::info!(
        "Login Success:  [{}-{}] ak {}",
        context.ident.tenant_id,
        context.ident.app_id,
        account_login_req.ak.as_str()
    );
    let ident_info = do_login(
        account_login_req.ak.as_str(),
        valid_end_time,
        account_login_req.cert_category.as_deref().unwrap_or_default(),
        &context,
    )
    .await?;
    Ok(ident_info)
}

async fn do_login<'c>(ak: &str, valid_end_time: i64, cert_category: &str, context: &BIOSContext) -> BIOSResult<IdentInfo> {
    let role_info = BIOSFuns::reldb()
        .fetch_all_json(
            &Query::select()
                .column(IamAccountRole::RelRoleId)
                .from(IamAccountRole::Table)
                .and_where(Expr::col(IamAccountRole::RelAccountId).eq(context.ident.account_id.as_str()))
                .done(),
            None,
        )
        .await?;
    let role_info = role_info.into_iter().map(|x| x["rel_role_id"].as_str().unwrap().to_string()).collect::<Vec<String>>();
    let group_node_info = BIOSFuns::reldb()
        .fetch_all_json(
            &Query::select()
                .column(IamAccountGroup::RelGroupNodeId)
                .from(IamAccountGroup::Table)
                .and_where(Expr::col(IamAccountGroup::RelAccountId).eq(context.ident.account_id.as_str()))
                .done(),
            None,
        )
        .await?;
    let group_node_info = group_node_info.into_iter().map(|x| x["rel_group_node_id"].as_str().unwrap().to_string()).collect::<Vec<String>>();

    let token = bios::basic::security::key::generate_token();

    let context = BIOSContext {
        trace: Trace {
            id: context.trace.id.to_string(),
            app: context.trace.app.to_string(),
            inst: context.trace.inst.to_string(),
        },
        ident: IdentInfo {
            app_id: context.ident.app_id.to_string(),
            tenant_id: context.ident.tenant_id.to_string(),
            account_id: context.ident.account_id.to_string(),
            token,
            token_kind: cert_category.to_string(),
            ak: ak.to_string(),
            roles: role_info,
            groups: group_node_info,
        },
        lang: context.lang.to_string(),
    };

    cache_processor::set_token(valid_end_time, &context).await?;
    Ok(context.ident)
}

#[get("/common/login")]
pub async fn fetch_login_info(req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;

    let account_name = BIOSFuns::reldb()
        .fetch_one_json(
            &Query::select().column(IamAccount::Name).from(IamAccount::Table).and_where(Expr::col(IamAccount::Id).eq(context.ident.account_id.as_str())).done(),
            None,
        )
        .await?;
    let account_name = account_name["name"].as_str().unwrap();

    let app_name = BIOSFuns::reldb()
        .fetch_one_json(
            &Query::select().column(IamApp::Name).from(IamApp::Table).and_where(Expr::col(IamApp::Id).eq(context.ident.app_id.as_str())).done(),
            None,
        )
        .await?;
    let app_name = app_name["name"].as_str().unwrap();

    let tenant_name = BIOSFuns::reldb()
        .fetch_one_json(
            &Query::select().column(IamTenant::Name).from(IamTenant::Table).and_where(Expr::col(IamTenant::Id).eq(context.ident.tenant_id.as_str())).done(),
            None,
        )
        .await?;
    let tenant_name = tenant_name["name"].as_str().unwrap();

    let role_info = BIOSFuns::reldb()
        .fetch_all_json(
            &Query::select().columns(vec![IamRole::Id, IamRole::Name]).from(IamRole::Table).and_where(Expr::col(IamRole::Id).is_in(context.ident.roles.clone())).done(),
            None,
        )
        .await?;
    let role_info = role_info.into_iter().map(|x| (x["id"].as_str().unwrap().to_string(), x["name"].as_str().unwrap().to_string())).collect::<Vec<(String, String)>>();

    let group_node_info = BIOSFuns::reldb()
        .fetch_all_json(
            &Query::select()
                .column((IamGroupNode::Table, IamGroupNode::Id))
                .column((IamGroupNode::Table, IamGroupNode::Name))
                .expr_as(Expr::col((IamGroup::Table, IamGroup::Name)), Alias::new("group_name"))
                .from(IamGroupNode::Table)
                .inner_join(
                    IamGroup::Table,
                    Expr::tbl(IamGroup::Table, IamGroup::Id).equals(IamGroupNode::Table, IamGroupNode::RelGroupId),
                )
                .and_where(Expr::tbl(IamGroupNode::Table, IamGroupNode::Id).is_in(context.ident.groups.clone()))
                .done(),
            None,
        )
        .await?;
    let group_node_info = group_node_info
        .into_iter()
        .map(|x| {
            (
                x["id"].as_str().unwrap().to_string(),
                format!("{} - {}", x["group_name"].as_str().unwrap().to_string(), x["name"].as_str().unwrap().to_string()),
            )
        })
        .collect::<Vec<(String, String)>>();

    BIOSResp::ok(
        AccountInfoDetailResp {
            app_id: context.ident.app_id.to_string(),
            app_name: app_name.to_string(),
            tenant_id: context.ident.tenant_id.to_string(),
            tenant_name: tenant_name.to_string(),
            account_id: context.ident.account_id.to_string(),
            account_name: account_name.to_string(),
            token: context.ident.token.to_string(),
            token_kind: context.ident.token_kind.to_string(),
            roles: role_info,
            groups: group_node_info,
        },
        Some(&context),
    )
}

#[delete("/common/logout")]
pub async fn logout(req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;
    cache_processor::remove_token(&context).await?;
    BIOSResp::ok("", Some(&context))
}

#[put("/common/account")]
pub async fn change_account(account_change_req: Json<AccountChangeReq>, req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;

    let mut values = Vec::new();
    if let Some(name) = &account_change_req.name {
        values.push((IamAccount::Name, name.as_str().into()));
    }
    if let Some(avatar) = &account_change_req.avatar {
        values.push((IamAccount::Avatar, avatar.as_str().into()));
    }
    if let Some(parameters) = &account_change_req.parameters {
        values.push((IamAccount::Parameters, parameters.as_str().into()));
    }
    values.push((IamAccount::UpdateUser, context.ident.account_id.as_str().into()));

    BIOSFuns::reldb()
        .exec(
            &Query::update()
                .table(IamAccount::Table)
                .values(values)
                .and_where(Expr::col(IamAccount::Id).eq(context.ident.account_id.as_str()))
                .and_where(Expr::col(IamAccount::RelTenantId).eq(context.ident.tenant_id.as_str()))
                .done(),
            None,
        )
        .await?;

    BIOSResp::ok("", Some(&context))
}

#[put("/common/account/ident")]
pub async fn change_account_ident(account_ident_change_req: Json<AccountIdentChangeReq>, req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;

    let id = BIOSFuns::reldb()
        .fetch_one_json(
            &Query::select()
                .columns(vec![IamAccountIdent::Id])
                .from(IamAccountIdent::Table)
                .and_where(Expr::col(IamAccountIdent::Kind).eq(account_ident_change_req.kind.to_string().to_lowercase()))
                .and_where(Expr::col(IamAccountIdent::RelAccountId).eq(context.ident.account_id.as_str()))
                .done(),
            None,
        )
        .await?;
    let id = id["id"].as_str().unwrap();

    if BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamAccountIdent::Id])
                .from(IamAccountIdent::Table)
                .and_where(Expr::col(IamAccountIdent::Kind).eq(account_ident_change_req.kind.to_string().to_lowercase()))
                .and_where(Expr::col(IamAccountIdent::Ak).eq(account_ident_change_req.ak.as_str()))
                .and_where(Expr::col(IamAccountIdent::RelTenantId).eq(context.ident.tenant_id.as_str()))
                .and_where(Expr::col(IamAccountIdent::Id).ne(id))
                .done(),
            None,
        )
        .await?
    {
        return BIOSResp::err(IamOutput::CommonEntityModifyCheckExists(ObjectKind::AccountIdent, "AccountIdent"), Some(&context));
    }

    auth_processor::valid_account_ident(&account_ident_change_req.kind, &account_ident_change_req.ak, &account_ident_change_req.sk, None, &context).await?;

    let processed_sk = auth_processor::process_sk(&account_ident_change_req.kind, &account_ident_change_req.ak, &account_ident_change_req.sk, &context).await?;

    let mut values = Vec::new();
    values.push((IamAccountIdent::Ak, account_ident_change_req.ak.as_str().into()));
    values.push((IamAccountIdent::Sk, processed_sk.into()));
    values.push((IamAccountIdent::UpdateUser, context.ident.account_id.as_str().into()));
    BIOSFuns::reldb()
        .exec(
            &Query::update()
                .table(IamAccountIdent::Table)
                .values(values)
                .and_where(Expr::col(IamAccountIdent::Id).eq(id))
                .and_where(Expr::col(IamAccountIdent::RelAccountId).eq(context.ident.account_id.as_str()))
                .done(),
            None,
        )
        .await?;
    cache_processor::remove_token(&context).await?;
    BIOSResp::ok("", Some(&context))
}

#[get("/common/resource/menu/{app_id}")]
pub async fn fetch_menu_resources(req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;
    BIOSResp::ok(fetch_resources(ResourceKind::Menu.to_string().to_lowercase(), &context).await?, Some(&context))
}

#[get("/common/resource/element/{app_id}")]
pub async fn fetch_element_resources(req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;
    BIOSResp::ok(fetch_resources(ResourceKind::Element.to_string().to_lowercase(), &context).await?, Some(&context))
}

async fn fetch_resources(kind: String, context: &BIOSContext) -> BIOSResult<Vec<ResourceDetailResp>> {
    let resource_info = BIOSFuns::reldb()
        .fetch_all_json(
            &Query::select()
                .column((IamResource::Table, IamResource::Name))
                .column((IamResource::Table, IamResource::PathAndQuery))
                .column((IamResource::Table, IamResource::Icon))
                .column((IamResource::Table, IamResource::Action))
                .column((IamResource::Table, IamResource::Sort))
                .column((IamResource::Table, IamResource::ResGroup))
                .column((IamResource::Table, IamResource::ParentId))
                .column((IamResourceSubject::Table, IamResourceSubject::IdentUri))
                .expr_as(Expr::col((IamResourceSubject::Table, IamResourceSubject::Name)), Alias::new("subject_name"))
                .from(IamAuthPolicyObject::Table)
                .inner_join(
                    IamAuthPolicy::Table,
                    Expr::tbl(IamAuthPolicy::Table, IamAuthPolicy::Id).equals(IamAuthPolicyObject::Table, IamAuthPolicyObject::RelAuthPolicyId),
                )
                .inner_join(
                    IamResource::Table,
                    Expr::tbl(IamResource::Table, IamResource::Id).equals(IamAuthPolicy::Table, IamAuthPolicy::RelResourceId),
                )
                .inner_join(
                    IamResourceSubject::Table,
                    Expr::tbl(IamResourceSubject::Table, IamResourceSubject::Id).equals(IamResource::Table, IamResource::RelResourceSubjectId),
                )
                .and_where(Expr::tbl(IamResourceSubject::Table, IamResourceSubject::Kind).eq(kind.as_str()))
                .cond_where(
                    // TODO support more operators
                    Cond::any()
                        .add(
                            Cond::all()
                                .add(Expr::tbl(IamAuthPolicyObject::Table, IamAuthPolicyObject::ObjectKind).eq(AuthObjectKind::Account.to_string().to_lowercase()))
                                .add(Expr::tbl(IamAuthPolicyObject::Table, IamAuthPolicyObject::ObjectId).eq(context.ident.account_id.as_str()))
                                .add(Expr::tbl(IamAuthPolicyObject::Table, IamAuthPolicyObject::ObjectOperator).eq(AuthObjectOperatorKind::Eq.to_string().to_lowercase())),
                        )
                        .add(
                            Cond::all()
                                .add(Expr::tbl(IamAuthPolicyObject::Table, IamAuthPolicyObject::ObjectKind).eq(AuthObjectKind::App.to_string().to_lowercase()))
                                .add(Expr::tbl(IamAuthPolicyObject::Table, IamAuthPolicyObject::ObjectId).eq(context.ident.app_id.as_str()))
                                .add(Expr::tbl(IamAuthPolicyObject::Table, IamAuthPolicyObject::ObjectOperator).eq(AuthObjectOperatorKind::Eq.to_string().to_lowercase())),
                        )
                        .add(
                            Cond::all()
                                .add(Expr::tbl(IamAuthPolicyObject::Table, IamAuthPolicyObject::ObjectKind).eq(AuthObjectKind::Tenant.to_string().to_lowercase()))
                                .add(Expr::tbl(IamAuthPolicyObject::Table, IamAuthPolicyObject::ObjectId).eq(context.ident.tenant_id.as_str()))
                                .add(Expr::tbl(IamAuthPolicyObject::Table, IamAuthPolicyObject::ObjectOperator).eq(AuthObjectOperatorKind::Eq.to_string().to_lowercase())),
                        )
                        .add(
                            Cond::all()
                                .add(Expr::tbl(IamAuthPolicyObject::Table, IamAuthPolicyObject::ObjectKind).eq(AuthObjectKind::Role.to_string().to_lowercase()))
                                .add(Expr::tbl(IamAuthPolicyObject::Table, IamAuthPolicyObject::ObjectId).is_in(context.ident.roles.clone()))
                                .add(Expr::tbl(IamAuthPolicyObject::Table, IamAuthPolicyObject::ObjectOperator).eq(AuthObjectOperatorKind::Eq.to_string().to_lowercase())),
                        )
                        .add(
                            Cond::all()
                                .add(Expr::tbl(IamAuthPolicyObject::Table, IamAuthPolicyObject::ObjectKind).eq(AuthObjectKind::GroupNode.to_string().to_lowercase()))
                                .add(Expr::tbl(IamAuthPolicyObject::Table, IamAuthPolicyObject::ObjectId).is_in(context.ident.groups.clone()))
                                .add(Expr::tbl(IamAuthPolicyObject::Table, IamAuthPolicyObject::ObjectOperator).eq(AuthObjectOperatorKind::Eq.to_string().to_lowercase())),
                        ),
                )
                .done(),
            None,
        )
        .await?;
    let result = resource_info
        .iter()
        .map(|item| {
            let res_name = item["name"].as_str().unwrap();
            let path_and_query = item["path_and_query"].as_str().unwrap();
            let icon = item["icon"].as_str().unwrap();
            let action = item["action"].as_str().unwrap();
            let sort = item["sort"].as_i64().unwrap();
            let res_group = item["res_group"].as_bool().unwrap();
            let parent_id = item["parent_id"].as_str().unwrap();
            let ident_uri = item["ident_uri"].as_str().unwrap();
            let subject_name = item["subject_name"].as_str().unwrap();
            ResourceDetailResp {
                name: format!("{}-{}", subject_name, res_name),
                ident_uri: bios::basic::uri::format_with_item(ident_uri, path_and_query).unwrap(),
                icon: icon.to_string(),
                action: action.to_string(),
                sort: sort as i32,
                res_group,
                parent_id: parent_id.to_string(),
            }
        })
        .collect_vec();
    Ok(result)
}
