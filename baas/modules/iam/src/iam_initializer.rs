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

use chrono::Utc;
use sea_query::Query;
use sqlx::{Connection, MySql, Transaction};

use bios::basic::error::BIOSResult;
use bios::db::reldb_client::SqlBuilderProcess;
use bios::BIOSFuns;

use crate::domain::auth_domain::{IamAccountRole, IamAuthPolicy, IamAuthPolicyObject, IamResource, IamResourceSubject, IamRole};
use crate::domain::ident_domain::{IamAccount, IamAccountApp, IamAccountIdent, IamApp, IamAppIdent, IamTenant, IamTenantCert, IamTenantIdent};
use crate::iam_config::WorkSpaceConfig;
use crate::process::basic_dto::{AccountIdentKind, AuthObjectKind, AuthObjectOperatorKind, AuthResultKind, CommonStatus, ExposeKind, OptActionKind, ResourceKind};
use crate::process::common::{auth_processor, cache_processor};

pub async fn init() -> BIOSResult<()> {
    log::info!("[Startup]Initializing IAM application");
    let iam_config = &BIOSFuns::ws_config::<WorkSpaceConfig>().iam;
    let mut conn = BIOSFuns::reldb().conn().await;
    let mut tx = conn.begin().await?;

    let tenant_id = bios::basic::field::uuid();
    let app_id = bios::basic::field::uuid();
    let account_id = bios::basic::field::uuid();
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
                    tenant_id.as_str().into(),
                    account_id.as_str().into(),
                    account_id.as_str().into(),
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
                    account_id.as_str().into(),
                    account_id.as_str().into(),
                    AccountIdentKind::Username.to_string().to_lowercase().into(),
                    iam_config.security.default_valid_ak_rule_note.to_string().into(),
                    iam_config.security.default_valid_ak_rule.to_string().into(),
                    iam_config.security.default_valid_sk_rule_note.to_string().into(),
                    iam_config.security.default_valid_sk_rule.to_string().into(),
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
                    0.into(),
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
                    iam_config.app.app_name.as_str().into(),
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
                    iam_config.app.admin_name.as_str().into(),
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
        &iam_config.app.admin_name,
        &iam_config.app.admin_password,
        &tenant_id,
        Some(&mut tx),
    )
    .await?;
    let processed_sk = auth_processor::process_sk(
        &AccountIdentKind::Username.to_string().to_lowercase(),
        &iam_config.app.admin_name,
        &iam_config.app.admin_password,
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
                    iam_config.app.admin_name.as_str().into(),
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
    // Init Role And AccountRole
    async fn init_account_role<'c>(role_code: &str, role_name: &str, account_id: &str, app_id: &str, tenant_id: &str, tx: &mut Transaction<'c, MySql>) -> BIOSResult<String> {
        let role_id = bios::basic::field::uuid();
        BIOSFuns::reldb()
            .exec(
                &Query::insert()
                    .into_table(IamRole::Table)
                    .columns(vec![
                        IamRole::Id,
                        IamRole::CreateUser,
                        IamRole::UpdateUser,
                        IamRole::Code,
                        IamRole::Name,
                        IamRole::Sort,
                        IamRole::RelAppId,
                        IamRole::RelTenantId,
                    ])
                    .values_panic(vec![
                        role_id.as_str().into(),
                        account_id.into(),
                        account_id.into(),
                        role_code.into(),
                        role_name.into(),
                        0.into(),
                        app_id.into(),
                        tenant_id.into(),
                    ])
                    .done(),
                Some(tx),
            )
            .await?;
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
                        account_id.into(),
                        account_id.into(),
                        account_id.into(),
                        role_id.as_str().into(),
                    ])
                    .done(),
                Some(tx),
            )
            .await?;
        Ok(role_id)
    }
    let system_role_id = init_account_role(
        &iam_config.security.system_admin_role_code.as_str(),
        &iam_config.security.system_admin_role_name.as_str(),
        &account_id,
        &app_id,
        &tenant_id,
        &mut tx,
    )
    .await?;
    let tenant_role_id = init_account_role(
        &iam_config.security.tenant_admin_role_code.as_str(),
        &iam_config.security.tenant_admin_role_name.as_str(),
        &account_id,
        &app_id,
        &tenant_id,
        &mut tx,
    )
    .await?;
    let app_role_id = init_account_role(
        &iam_config.security.app_admin_role_code.as_str(),
        &iam_config.security.app_admin_role_name.as_str(),
        &account_id,
        &app_id,
        &tenant_id,
        &mut tx,
    )
    .await?;
    // Init ResourceSubject
    async fn init_resource_subject<'c>(
        kind: &ResourceKind,
        uri: &str,
        name: &str,
        account_id: &str,
        app_id: &str,
        tenant_id: &str,
        tx: &mut Transaction<'c, MySql>,
    ) -> BIOSResult<String> {
        let resource_subject_id = bios::basic::field::uuid();
        BIOSFuns::reldb()
            .exec(
                &Query::insert()
                    .into_table(IamResourceSubject::Table)
                    .columns(vec![
                        IamResourceSubject::Id,
                        IamResourceSubject::CreateUser,
                        IamResourceSubject::UpdateUser,
                        IamResourceSubject::Kind,
                        IamResourceSubject::Uri,
                        IamResourceSubject::Name,
                        IamResourceSubject::Sort,
                        IamResourceSubject::Ak,
                        IamResourceSubject::Sk,
                        IamResourceSubject::PlatformAccount,
                        IamResourceSubject::PlatformProjectId,
                        IamResourceSubject::TimeoutMs,
                        IamResourceSubject::RelAppId,
                        IamResourceSubject::RelTenantId,
                    ])
                    .values_panic(vec![
                        resource_subject_id.as_str().into(),
                        account_id.into(),
                        account_id.into(),
                        kind.to_string().to_lowercase().into(),
                        uri.into(),
                        name.into(),
                        0.into(),
                        "".into(),
                        "".into(),
                        "".into(),
                        "".into(),
                        0.into(),
                        app_id.into(),
                        tenant_id.into(),
                    ])
                    .done(),
                Some(tx),
            )
            .await?;
        Ok(resource_subject_id)
    }
    // Init ResourceSubject
    let resource_subject_api_id = init_resource_subject(
        &ResourceKind::Api,
        format!("api://{}", iam_config.service_name).as_str(),
        format!("{} APIs", iam_config.app.app_name).as_str(),
        &account_id,
        &app_id,
        &tenant_id,
        &mut tx,
    )
    .await?;
    // Init Auth
    async fn init_auth<'c>(
        path_and_query: &str,
        name: &str,
        resource_subject_id: &str,
        role_id: &str,
        account_id: &str,
        app_id: &str,
        tenant_id: &str,
        tx: &mut Transaction<'c, MySql>,
    ) -> BIOSResult<()> {
        // Init Resource
        async fn init_resource<'c>(
            path_and_query: &str,
            name: &str,
            resource_subject_id: &str,
            account_id: &str,
            app_id: &str,
            tenant_id: &str,
            tx: &mut Transaction<'c, MySql>,
        ) -> BIOSResult<String> {
            let resource_id = bios::basic::field::uuid();
            BIOSFuns::reldb()
                .exec(
                    &Query::insert()
                        .into_table(IamResource::Table)
                        .columns(vec![
                            IamResource::Id,
                            IamResource::CreateUser,
                            IamResource::UpdateUser,
                            IamResource::PathAndQuery,
                            IamResource::Name,
                            IamResource::Icon,
                            IamResource::Sort,
                            IamResource::Action,
                            IamResource::ResGroup,
                            IamResource::ParentId,
                            IamResource::RelResourceSubjectId,
                            IamResource::RelAppId,
                            IamResource::RelTenantId,
                            IamResource::ExposeKind,
                        ])
                        .values_panic(vec![
                            resource_id.as_str().into(),
                            account_id.into(),
                            account_id.into(),
                            path_and_query.into(),
                            name.into(),
                            "".into(),
                            0.into(),
                            "".into(),
                            false.into(),
                            "".into(),
                            resource_subject_id.into(),
                            app_id.into(),
                            tenant_id.into(),
                            ExposeKind::Global.to_string().to_lowercase().into(),
                        ])
                        .done(),
                    Some(tx),
                )
                .await?;
            Ok(resource_id)
        }
        // Init AuthPolicy
        async fn init_auth_policy<'c>(
            name: &str,
            action: &OptActionKind,
            resource_id: &str,
            result: &AuthResultKind,
            account_id: &str,
            app_id: &str,
            tenant_id: &str,
            tx: &mut Transaction<'c, MySql>,
        ) -> BIOSResult<String> {
            let auth_policy_id = bios::basic::field::uuid();
            let valid_start_time = Utc::now().timestamp();
            let valid_end_time = i64::MAX;
            BIOSFuns::reldb()
                .exec(
                    &Query::insert()
                        .into_table(IamAuthPolicy::Table)
                        .columns(vec![
                            IamAuthPolicy::Id,
                            IamAuthPolicy::CreateUser,
                            IamAuthPolicy::UpdateUser,
                            IamAuthPolicy::Name,
                            IamAuthPolicy::ValidStartTime,
                            IamAuthPolicy::ValidEndTime,
                            IamAuthPolicy::ActionKind,
                            IamAuthPolicy::RelResourceId,
                            IamAuthPolicy::ResultKind,
                            IamAuthPolicy::RelAppId,
                            IamAuthPolicy::RelTenantId,
                        ])
                        .values_panic(vec![
                            auth_policy_id.as_str().into(),
                            account_id.into(),
                            account_id.into(),
                            name.into(),
                            valid_start_time.into(),
                            valid_end_time.into(),
                            action.to_string().to_lowercase().into(),
                            resource_id.into(),
                            result.to_string().to_lowercase().into(),
                            app_id.into(),
                            tenant_id.into(),
                        ])
                        .done(),
                    Some(tx),
                )
                .await?;
            Ok(auth_policy_id)
        }
        // Init AuthPolicyObject
        async fn init_auth_policy_object<'c>(role_id: &str, auth_policy_id: &str, account_id: &str, tx: &mut Transaction<'c, MySql>) -> BIOSResult<()> {
            BIOSFuns::reldb()
                .exec(
                    &Query::insert()
                        .into_table(IamAuthPolicyObject::Table)
                        .columns(vec![
                            IamAuthPolicyObject::Id,
                            IamAuthPolicyObject::CreateUser,
                            IamAuthPolicyObject::UpdateUser,
                            IamAuthPolicyObject::ObjectKind,
                            IamAuthPolicyObject::ObjectId,
                            IamAuthPolicyObject::ObjectOperator,
                            IamAuthPolicyObject::RelAuthPolicyId,
                        ])
                        .values_panic(vec![
                            bios::basic::field::uuid().into(),
                            account_id.into(),
                            account_id.into(),
                            AuthObjectKind::Role.to_string().to_lowercase().into(),
                            role_id.into(),
                            AuthObjectOperatorKind::Eq.to_string().to_lowercase().into(),
                            auth_policy_id.into(),
                        ])
                        .done(),
                    Some(tx),
                )
                .await?;
            Ok(())
        }

        let resource_id = init_resource(path_and_query, format!("{}资源", name).as_str(), resource_subject_id, account_id, app_id, tenant_id, tx).await?;
        let auth_policy_id = init_auth_policy(
            format!("{}权限", name).as_str(),
            &OptActionKind::Post,
            &resource_id,
            &AuthResultKind::Accept,
            account_id,
            app_id,
            tenant_id,
            tx,
        )
        .await?;
        init_auth_policy_object(role_id, &auth_policy_id, account_id, tx).await?;
        let auth_policy_id = init_auth_policy(
            format!("{}权限", name).as_str(),
            &OptActionKind::Put,
            &resource_id,
            &AuthResultKind::Accept,
            account_id,
            app_id,
            tenant_id,
            tx,
        )
        .await?;
        init_auth_policy_object(role_id, &auth_policy_id, account_id, tx).await?;
        let auth_policy_id = init_auth_policy(
            format!("{}权限", name).as_str(),
            &OptActionKind::Get,
            &resource_id,
            &AuthResultKind::Accept,
            account_id,
            app_id,
            tenant_id,
            tx,
        )
        .await?;
        init_auth_policy_object(role_id, &auth_policy_id, account_id, tx).await?;
        let auth_policy_id = init_auth_policy(
            format!("{}权限", name).as_str(),
            &OptActionKind::Delete,
            &resource_id,
            &AuthResultKind::Accept,
            account_id,
            app_id,
            tenant_id,
            tx,
        )
        .await?;
        init_auth_policy_object(role_id, &auth_policy_id, account_id, tx).await?;
        Ok(())
    }

    init_auth(
        "/console/system/**",
        "系统控制台",
        &resource_subject_api_id,
        &system_role_id,
        &account_id,
        &app_id,
        &tenant_id,
        &mut tx,
    )
    .await?;
    init_auth(
        "/console/tenant/**",
        "租户控制台",
        &resource_subject_api_id,
        &tenant_role_id,
        &account_id,
        &app_id,
        &tenant_id,
        &mut tx,
    )
    .await?;
    init_auth(
        "/console/app/**",
        "应用控制台",
        &resource_subject_api_id,
        &app_role_id,
        &account_id,
        &app_id,
        &tenant_id,
        &mut tx,
    )
    .await?;

    let auth_policy_ids = BIOSFuns::reldb().fetch_all_json(&Query::select().column(IamAuthPolicy::Id).from(IamAuthPolicy::Table).done(), Some(&mut tx)).await?;
    for id in auth_policy_ids {
        cache_processor::rebuild_auth_policy(id["id"].as_str().unwrap(), &mut tx).await?;
    }
    cache_processor::set_aksk(&tenant_id, &app_id, &ak, &sk, i64::MAX).await?;
    tx.commit().await?;
    Ok(())
}
