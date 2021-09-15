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

use actix_web::post;
use chrono::Utc;
use sea_query::{Alias, Expr, Query};
use sqlx::{Connection, MySql, Transaction};

use bios::basic::dto::IdentAccountInfo;
use bios::basic::error::{BIOSError, BIOSResult};
use bios::db::reldb_client::SqlBuilderProcess;
use bios::web::resp_handler::{BIOSResp, BIOSRespHelper};
use bios::web::validate::json::Json;
use bios::BIOSFuns;

use crate::domain::auth_domain::{IamAccountGroup, IamAccountRole, IamRole};
use crate::domain::ident_domain::{IamAccount, IamAccountApp, IamAccountIdent, IamApp, IamAppIdent, IamTenant, IamTenantCert, IamTenantIdent};
use crate::iam_config::WorkSpaceConfig;
use crate::process::basic_dto::{AccountIdentKind, CommonStatus};
use crate::process::common::com_account_dto::{AccountLoginReq, AccountRegisterReq};
use crate::process::common::com_tenant_dto::TenantRegisterReq;
use crate::process::common::{auth_processor, cache_processor};

#[post("/common/tenant")]
pub async fn register_tenant(tenant_register_req: Json<TenantRegisterReq>) -> BIOSResp {
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
                    tenant_id.as_str().into(),
                    account_id.as_str().into(),
                    account_id.as_str().into(),
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
                    account_id.as_str().into(),
                    account_id.as_str().into(),
                    AccountIdentKind::Username.to_string().to_lowercase().into(),
                    iam_config.security.default_valid_ak_rule_note.as_str().into(),
                    iam_config.security.default_valid_ak_rule.as_str().into(),
                    iam_config.security.default_valid_sk_rule_note.as_str().into(),
                    iam_config.security.default_valid_sk_rule.as_str().into(),
                    iam_config.security.default_valid_time_sec.into(),
                    tenant_id.as_str().into(),
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
                    account_id.as_str().into(),
                    account_id.as_str().into(),
                    "".into(),
                    1.into(),
                    tenant_id.as_str().into(),
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
                    app_id.as_str().into(),
                    account_id.as_str().into(),
                    account_id.as_str().into(),
                    tenant_register_req.app_name.as_str().into(),
                    "".into(),
                    "".into(),
                    CommonStatus::Enabled.to_string().to_lowercase().into(),
                    tenant_id.as_str().into(),
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
                    account_id.as_str().into(),
                    account_id.as_str().into(),
                    "".into(),
                    ak.as_str().into(),
                    sk.as_str().into(),
                    i64::MAX.into(),
                    app_id.as_str().into(),
                    tenant_id.as_str().into(),
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
                    account_id.as_str().into(),
                    account_id.as_str().into(),
                    account_id.as_str().into(),
                    format!("ao_{}", bios::basic::field::uuid()).into(),
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
        &AccountIdentKind::Username,
        &tenant_register_req.account_username,
        &tenant_register_req.account_password,
        &tenant_id,
        Some(&mut tx),
    )
    .await?;
    let processed_sk = auth_processor::process_sk(
        &AccountIdentKind::Username,
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
                    account_id.as_str().into(),
                    account_id.as_str().into(),
                    AccountIdentKind::Username.to_string().to_lowercase().into(),
                    tenant_register_req.account_username.as_str().into(),
                    processed_sk.into(),
                    Utc::now().timestamp().into(),
                    valid_end_time.into(),
                    account_id.as_str().into(),
                    tenant_id.as_str().into(),
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
                    account_id.as_str().into(),
                    account_id.as_str().into(),
                    account_id.as_str().into(),
                    app_id.as_str().into(),
                ])
                .done(),
            Some(&mut tx),
        )
        .await?;
    // Init AccountRole
    let role_id = BIOSFuns::reldb()
        .fetch_one_json(
            &Query::select().column(IamRole::Id).from(IamRole::Table).and_where(Expr::col(IamRole::Code).eq(iam_config.security.tenant_admin_role_code.as_str())).done(),
            Some(&mut tx),
        )
        .await?;
    let role_id = role_id["id"].as_str().unwrap();
    BIOSFuns::reldb()
        .exec(
            &Query::insert()
                .into_table(IamAccountRole::Table)
                .columns(vec![
                    IamAccountRole::Id,
                    IamAccountRole::CreateUser,
                    IamAccountRole::UpdateUser,
                    IamAccountRole::RelAccountId,
                    IamAccountRole::RelRoleId,
                ])
                .values_panic(vec![
                    bios::basic::field::uuid().into(),
                    account_id.as_str().into(),
                    account_id.as_str().into(),
                    account_id.as_str().into(),
                    role_id.into(),
                ])
                .done(),
            Some(&mut tx),
        )
        .await?;
    let role_id = BIOSFuns::reldb()
        .fetch_one_json(
            &Query::select().column(IamRole::Id).from(IamRole::Table).and_where(Expr::col(IamRole::Code).eq(iam_config.security.app_admin_role_code.as_str())).done(),
            Some(&mut tx),
        )
        .await?;
    let role_id = role_id["id"].as_str().unwrap();
    BIOSFuns::reldb()
        .exec(
            &Query::insert()
                .into_table(IamAccountRole::Table)
                .columns(vec![
                    IamAccountRole::Id,
                    IamAccountRole::CreateUser,
                    IamAccountRole::UpdateUser,
                    IamAccountRole::RelAccountId,
                    IamAccountRole::RelRoleId,
                ])
                .values_panic(vec![
                    bios::basic::field::uuid().into(),
                    account_id.as_str().into(),
                    account_id.as_str().into(),
                    account_id.as_str().into(),
                    role_id.into(),
                ])
                .done(),
            Some(&mut tx),
        )
        .await?;

    cache_processor::set_aksk(&tenant_id, &app_id, &ak, &sk, i64::MAX).await?;
    // Login
    let ident_info = do_login(
        account_id.as_str(),
        tenant_register_req.account_username.as_str(),
        valid_end_time,
        tenant_id.as_str(),
        app_id.as_str(),
        Some(&mut tx),
    )
    .await?;

    tx.commit().await?;

    BIOSRespHelper::ok(ident_info)
}

#[post("/common/account")]
pub async fn register_account(account_register_req: Json<AccountRegisterReq>) -> BIOSResp {
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
        return BIOSRespHelper::bus_error(BIOSError::NotFound("App not exists".to_string()));
    }
    let tenant_id = tenant_id.unwrap();
    let tenant_id = tenant_id["id"].as_str().unwrap();

    if BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamAccountIdent::Id])
                .from(IamAccountIdent::Table)
                .and_where(Expr::col(IamAccountIdent::Kind).eq(account_register_req.kind.to_string().to_lowercase()))
                .and_where(Expr::col(IamAccountIdent::Ak).eq(account_register_req.ak.as_str()))
                .and_where(Expr::col(IamAccountIdent::RelTenantId).eq(tenant_id))
                .done(),
            None,
        )
        .await?
    {
        return Err(BIOSError::Conflict("AccountIdent [kind] and [ak] already exists".to_string()));
    }

    let mut conn = BIOSFuns::reldb().conn().await;
    let mut tx = conn.begin().await?;

    // Init Account
    let account_id = bios::basic::field::uuid();
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
                    format!("ao_{}", bios::basic::field::uuid()).into(),
                    account_register_req.name.as_str().into(),
                    account_register_req.avatar.as_deref().unwrap_or_default().into(),
                    account_register_req.parameters.as_deref().unwrap_or_default().into(),
                    "".into(),
                    tenant_id.into(),
                    CommonStatus::Enabled.to_string().to_lowercase().into(),
                ])
                .done(),
            Some(&mut tx),
        )
        .await?;
    // Init AccountIdent
    let valid_end_time = auth_processor::valid_account_ident(&AccountIdentKind::Username, &account_register_req.ak, &account_register_req.sk, &tenant_id, Some(&mut tx)).await?;
    let processed_sk = auth_processor::process_sk(
        &AccountIdentKind::Username,
        &account_register_req.ak,
        &account_register_req.sk,
        &tenant_id,
        &account_register_req.rel_app_id,
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
                    account_id.as_str().into(),
                    account_id.as_str().into(),
                    AccountIdentKind::Username.to_string().to_lowercase().into(),
                    account_register_req.name.as_str().into(),
                    processed_sk.into(),
                    Utc::now().timestamp().into(),
                    valid_end_time.into(),
                    account_id.as_str().into(),
                    tenant_id.into(),
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
                    account_id.as_str().into(),
                    account_id.as_str().into(),
                    account_id.as_str().into(),
                    account_register_req.rel_app_id.as_str().into(),
                ])
                .done(),
            Some(&mut tx),
        )
        .await?;
    // Login
    let ident_info = do_login(
        account_id.as_str(),
        account_register_req.ak.as_str(),
        valid_end_time,
        tenant_id,
        account_register_req.rel_app_id.as_str(),
        Some(&mut tx),
    )
    .await?;
    tx.commit().await?;
    BIOSRespHelper::ok(ident_info)
}

#[post("/common/login")]
pub async fn login(account_login_req: Json<AccountLoginReq>) -> BIOSResp {
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
                .and_where(Expr::tbl(IamAccountIdent::Table, IamAccountIdent::Kind).eq(account_login_req.kind.to_string().to_lowercase()))
                .and_where(Expr::tbl(IamAccountIdent::Table, IamAccountIdent::Ak).eq(account_login_req.ak.as_str()))
                .and_where(Expr::tbl(IamAccountIdent::Table, IamAccountIdent::ValidStartTime).lte(now))
                .and_where(Expr::tbl(IamAccountIdent::Table, IamAccountIdent::ValidEndTime).gte(now))
                .and_where(Expr::tbl(IamApp::Table, IamApp::Id).eq(account_login_req.rel_app_id.as_str()))
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
        return BIOSRespHelper::bus_error(BIOSError::NotFound("Account not exists".to_string()));
    }
    let account_info = account_info.unwrap();
    let tenant_id = account_info["tenant_id"].as_str().unwrap();
    let app_id = account_info["app_id"].as_str().unwrap();
    let account_id = account_info["id"].as_str().unwrap();
    let stored_sk = account_info["sk"].as_str().unwrap();
    let valid_end_time = account_info["valid_end_time"].as_i64().unwrap();
    auth_processor::validate_sk(
        &account_login_req.kind,
        account_login_req.ak.as_str(),
        account_login_req.sk.as_str(),
        stored_sk,
        tenant_id,
        app_id,
    )
    .await?;
    log::info!("Login Success:  [{}-{}] ak {}", tenant_id, app_id, account_login_req.ak.as_str());
    let ident_info = do_login(account_id, account_login_req.ak.as_str(), valid_end_time, tenant_id, app_id, None).await?;
    BIOSRespHelper::ok(ident_info)
}

async fn do_login<'c>(account_id: &str, ak: &str, valid_end_time: i64, tenant_id: &str, app_id: &str, tx: Option<&mut Transaction<'c, MySql>>) -> BIOSResult<IdentAccountInfo> {
    let role_info = BIOSFuns::reldb()
        .fetch_all_json(
            &Query::select().column(IamAccountRole::RelRoleId).from(IamAccountRole::Table).and_where(Expr::col(IamAccountRole::RelAccountId).eq(account_id)).done(),
            None,
        )
        .await?;
    let role_info = role_info.into_iter().map(|x| x["rel_role_id"].as_str().unwrap().to_string()).collect::<Vec<String>>();
    let group_node_info = BIOSFuns::reldb()
        .fetch_all_json(
            &Query::select().column(IamAccountGroup::RelGroupNodeId).from(IamAccountGroup::Table).and_where(Expr::col(IamAccountGroup::RelAccountId).eq(account_id)).done(),
            None,
        )
        .await?;
    let group_node_info = group_node_info.into_iter().map(|x| x["rel_group_node_id"].as_str().unwrap().to_string()).collect::<Vec<String>>();

    let token = bios::basic::security::key::generate_token();
    let ident_info = IdentAccountInfo {
        app_id: app_id.to_string(),
        tenant_id: tenant_id.to_string(),
        account_id: account_id.to_string(),
        token,
        // TODO
        token_kind: "".to_string(),
        ak: ak.to_string(),
        roles: role_info,
        groups: group_node_info,
    };
    cache_processor::set_token(&ident_info, valid_end_time, tx).await?;
    Ok(ident_info)
}
