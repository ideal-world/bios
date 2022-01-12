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

use chrono::Utc;
use sea_query::Query;
use sqlx::Connection;

use bios::basic::dto::{BIOSContext, IdentInfo};
use bios::basic::result::BIOSResult;
use bios::db::reldb_client::SqlBuilderProcess;
use bios::BIOSFuns;

use crate::domain::auth_domain::IamAuthPolicy;
use crate::domain::ident_domain::{IamAccount, IamAccountApp, IamAccountIdent, IamApp, IamAppIdent, IamTenant, IamTenantCert, IamTenantIdent};
use crate::iam_config::WorkSpaceConfig;
use crate::process::basic_dto::{AccountIdentKind, AuthObjectKind, CommonStatus, ExposeKind, OptActionKind, ResourceKind};
use crate::process::common::{auth_processor, cache_processor};

pub async fn init() -> BIOSResult<()> {
    log::info!("[Startup]Initializing IAM application");
    let iam_config = &BIOSFuns::ws_config::<WorkSpaceConfig>().iam;
    let mut conn = BIOSFuns::reldb().conn().await;
    let mut tx = conn.begin().await?;

    let context = BIOSContext {
        ident: IdentInfo {
            account_id: bios::basic::field::uuid(),
            token: "".to_string(),
            token_kind: "".to_string(),
            roles: vec![],
            app_id: bios::basic::field::uuid(),
            tenant_id: bios::basic::field::uuid(),
            ak: "".to_string(),
            groups: vec![],
        },
        trace: Default::default(),
        lang: "en_US".to_string(),
    };

    // Init Tenant
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
                    iam_config.app.tenant_name.as_str().into(),
                    "".into(),
                    false.into(),
                    "".into(),
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
                    iam_config.app.app_name.as_str().into(),
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
                    iam_config.app.admin_name.as_str().into(),
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
        &iam_config.app.admin_name,
        &iam_config.app.admin_password,
        Some(&mut tx),
        &context,
    )
    .await?;
    let processed_sk = auth_processor::process_sk(&AccountIdentKind::Username, &iam_config.app.admin_name, &iam_config.app.admin_password, &context).await?;
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
                    iam_config.app.admin_name.as_str().into(),
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
    let system_role_id = auth_processor::init_account_role(
        &iam_config.security.system_admin_role_code.as_str(),
        &iam_config.security.system_admin_role_name.as_str(),
        &mut tx,
        &context,
    )
    .await?;
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
    let resource_subject_api_id = auth_processor::init_resource_subject(
        &ResourceKind::Api,
        format!("https://{}", iam_config.service_name).as_str(),
        format!("{}接口", iam_config.app.app_name).as_str(),
        &mut tx,
        &context,
    )
    .await?;
    auth_processor::init_resource_subject(
        &ResourceKind::Menu,
        format!("https://{}/common/resource/menu/{}", iam_config.service_name, context.ident.app_id).as_str(),
        format!("{}菜单", iam_config.app.app_name).as_str(),
        &mut tx,
        &context,
    )
    .await?;
    auth_processor::init_resource_subject(
        &ResourceKind::Element,
        format!("https://{}/common/resource/element/{}", iam_config.service_name, context.ident.app_id).as_str(),
        format!("{}元素", iam_config.app.app_name).as_str(),
        &mut tx,
        &context,
    )
    .await?;

    // Init Resource
    let resource_console_system_id = auth_processor::init_resource("/console/system/**", "系统控制台", &resource_subject_api_id, &ExposeKind::App, &mut tx, &context).await?;
    let resource_console_tenant_id = auth_processor::init_resource("/console/tenant/**", "租户控制台", &resource_subject_api_id, &ExposeKind::Global, &mut tx, &context).await?;
    let resource_console_app_id = auth_processor::init_resource("/console/app/**", "应用控制台", &resource_subject_api_id, &ExposeKind::Global, &mut tx, &context).await?;

    // Init Auth
    auth_processor::init_auth(
        vec![(
            &resource_console_system_id,
            vec![
                &OptActionKind::Get,
                &OptActionKind::Post,
                &OptActionKind::Put,
                &OptActionKind::Delete,
                &OptActionKind::Patch,
            ],
        )],
        "系统控制台",
        &AuthObjectKind::Role,
        &system_role_id,
        &mut tx,
        &context,
    )
    .await?;
    auth_processor::init_auth(
        vec![(
            &resource_console_tenant_id,
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
            &resource_console_app_id,
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

    let auth_policy_ids = BIOSFuns::reldb().fetch_all_json(&Query::select().column(IamAuthPolicy::Id).from(IamAuthPolicy::Table).done(), Some(&mut tx)).await?;
    for id in auth_policy_ids {
        cache_processor::rebuild_auth_policy(id["id"].as_str().unwrap(), &mut tx, &context).await?;
    }
    cache_processor::set_aksk(&context.ident.tenant_id, &context.ident.app_id, &ak, &sk, i64::MAX, &context).await?;
    tx.commit().await?;
    Ok(())
}
