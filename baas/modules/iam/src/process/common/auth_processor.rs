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

use std::collections::HashMap;
use std::sync::Mutex;

use chrono::Utc;
use regex::Regex;
use sea_query::{Expr, Query};
use sqlx::{MySql, Transaction};

use bios::basic::dto::BIOSContext;
use bios::basic::error::BIOSError;
use bios::basic::result::{output, BIOSResult};
use bios::db::reldb_client::SqlBuilderProcess;
use bios::BIOSFuns;

use crate::domain::auth_domain::{IamAccountRole, IamAuthPolicy, IamAuthPolicyObject, IamResource, IamResourceSubject, IamRole};
use crate::domain::ident_domain::IamTenantIdent;
use crate::iam_config::WorkSpaceConfig;
use crate::iam_constant::IamOutput;
use crate::process::basic_dto::{AccountIdentKind, AuthObjectKind, AuthObjectOperatorKind, AuthResultKind, ExposeKind, OptActionKind, ResourceKind};
use crate::process::common::cache_processor;

lazy_static! {
    static ref AK_SK_CONTAINER: Mutex<HashMap<String, Regex>> = Mutex::new(HashMap::new());
}

pub async fn valid_account_ident<'c>(kind: &AccountIdentKind, ak: &str, sk: &str, tx: Option<&mut Transaction<'c, MySql>>, context: &BIOSContext) -> BIOSResult<i64> {
    if let Some(tenant_ident_info) = BIOSFuns::reldb()
        .fetch_optional::<TenantIdentInfoResp>(
            &Query::select()
                .columns(vec![IamTenantIdent::ValidAkRule, IamTenantIdent::ValidSkRule, IamTenantIdent::ValidTime])
                .from(IamTenantIdent::Table)
                .and_where(Expr::col(IamTenantIdent::Kind).eq(kind.to_string().to_lowercase()))
                .and_where(Expr::col(IamTenantIdent::RelTenantId).eq(context.ident.tenant_id.as_str()))
                .done(),
            tx,
        )
        .await?
    {
        if !ak.is_empty() && !tenant_ident_info.valid_ak_rule.is_empty() {
            let mut aksks = AK_SK_CONTAINER.lock().unwrap();
            if !aksks.contains_key(&tenant_ident_info.valid_ak_rule) {
                aksks.insert(tenant_ident_info.valid_ak_rule.clone(), Regex::new(&tenant_ident_info.valid_ak_rule).unwrap());
            }
            if !aksks.get(&tenant_ident_info.valid_ak_rule).unwrap().is_match(ak) {
                return BIOSError::err(IamOutput::CommonAccountIdentValidCheckInvalidFormat("ak"));
            }
        }
        if !sk.is_empty() && !tenant_ident_info.valid_sk_rule.is_empty() {
            let mut aksks = AK_SK_CONTAINER.lock().unwrap();
            if !aksks.contains_key(&tenant_ident_info.valid_sk_rule) {
                aksks.insert(tenant_ident_info.valid_sk_rule.clone(), Regex::new(&tenant_ident_info.valid_sk_rule).unwrap());
            }
            if !aksks.get(&tenant_ident_info.valid_sk_rule).unwrap().is_match(sk) {
                return BIOSError::err(IamOutput::CommonAccountIdentValidCheckInvalidFormat("sk"));
            }
        }
        Ok(Utc::now().timestamp() + tenant_ident_info.valid_time)
    } else {
        return BIOSError::err(IamOutput::CommonAccountIdentValidCheckNotFound());
    }
}

pub async fn validate_sk(kind: &AccountIdentKind, ak: &str, request_sk: &str, stored_sk: &str, context: &BIOSContext) -> BIOSResult<()> {
    match kind {
        AccountIdentKind::Phone | AccountIdentKind::Email => {
            if let Some(tmp_sk) = cache_processor::get_vcode(ak, &context).await? {
                if tmp_sk == request_sk {
                    cache_processor::remove_vcode(ak, &context).await?;
                    cache_processor::remove_vcode_error_times(ak, &context).await?;
                    Ok(())
                } else {
                    let error_times = cache_processor::incr_vcode_error_times(ak, &context).await?;
                    if error_times >= BIOSFuns::ws_config::<WorkSpaceConfig>().iam.security.account_vcode_max_error_times as usize {
                        cache_processor::remove_vcode(ak, &context).await?;
                        cache_processor::remove_vcode_error_times(ak, &context).await?;
                        log::warn!("{}", output(IamOutput::CommonAccountIdentValidCheckVCodeOverMaxTimes(ak.to_string()), &context).to_log());
                        BIOSError::err(IamOutput::CommonAccountIdentValidCheckVCodeOverMaxTimes(ak.to_string()))
                    } else {
                        log::warn!(
                            "{}",
                            output(IamOutput::CommonAccountIdentValidCheckInvalidVCodeNotFoundOrExpired(ak.to_string()), &context).to_log()
                        );
                        BIOSError::err(IamOutput::CommonAccountIdentValidCheckInvalidVCodeNotFoundOrExpired(ak.to_string()))
                    }
                }
            } else {
                log::warn!(
                    "{}",
                    output(IamOutput::CommonAccountIdentValidCheckInvalidVCodeNotFoundOrExpired(ak.to_string()), &context).to_log()
                );
                BIOSError::err(IamOutput::CommonAccountIdentValidCheckInvalidVCodeNotFoundOrExpired(ak.to_string()))
            }
        }
        AccountIdentKind::Username => {
            if !request_sk.trim().is_empty() {
                if bios::basic::security::digest::digest(format!("{}{}", ak, request_sk).as_str(), None, "SHA512") == stored_sk {
                    Ok(())
                } else {
                    log::warn!("{}", output(IamOutput::CommonAccountIdentValidCheckUserOrPasswordError(ak.to_string()), &context).to_log());
                    BIOSError::err(IamOutput::CommonAccountIdentValidCheckUserOrPasswordError(ak.to_string()))
                }
            } else {
                log::warn!("{}", output(IamOutput::CommonAccountIdentValidCheckPasswordNotEmpty(), &context).to_log());
                BIOSError::err(IamOutput::CommonAccountIdentValidCheckPasswordNotEmpty())
            }
        }
        AccountIdentKind::WechatXcx => {
            if let Some(account_token) = cache_processor::get_account_token(kind.to_string().to_lowercase().as_str(), &context).await? {
                if account_token == request_sk {
                    Ok(())
                } else {
                    log::warn!(
                        "{}",
                        output(IamOutput::CommonAccountIdentValidCheckInvalidAccessTokenNotFoundOrExpired(ak.to_string()), &context).to_log()
                    );
                    BIOSError::err(IamOutput::CommonAccountIdentValidCheckInvalidAccessTokenNotFoundOrExpired(ak.to_string()))
                }
            } else {
                log::warn!(
                    "{}",
                    output(IamOutput::CommonAccountIdentValidCheckInvalidAccessTokenNotFoundOrExpired(ak.to_string()), &context).to_log()
                );
                BIOSError::err(IamOutput::CommonAccountIdentValidCheckInvalidAccessTokenNotFoundOrExpired(ak.to_string()))
            }
        }
        _ => {
            log::warn!("{}", output(IamOutput::CommonAccountIdentValidCheckUnsupportedAuthKind(ak.to_string()), &context).to_log());
            BIOSError::err(IamOutput::CommonAccountIdentValidCheckUnsupportedAuthKind(ak.to_string()))
        }
    }
}

pub async fn process_sk(kind: &AccountIdentKind, ak: &str, sk: &str, context: &BIOSContext) -> BIOSResult<String> {
    match kind {
        AccountIdentKind::Phone | AccountIdentKind::Email => {
            if let Some(tmp_sk) = cache_processor::get_vcode(ak, &context).await? {
                if tmp_sk == sk {
                    Ok(sk.to_string())
                } else {
                    log::warn!("{}", output(IamOutput::CommonAccountIdentValidCheckVCodeOverMaxTimes(ak.to_string()), &context).to_log());
                    BIOSError::err(IamOutput::CommonAccountIdentValidCheckVCodeOverMaxTimes(ak.to_string()))
                }
            } else {
                log::warn!("{}", output(IamOutput::CommonAccountIdentValidCheckVCodeOverMaxTimes(ak.to_string()), &context).to_log());
                BIOSError::err(IamOutput::CommonAccountIdentValidCheckVCodeOverMaxTimes(ak.to_string()))
            }
        }
        AccountIdentKind::Username => {
            if !sk.trim().is_empty() {
                Ok(bios::basic::security::digest::digest(format!("{}{}", ak, sk).as_str(), None, "SHA512"))
            } else {
                log::warn!("{}", output(IamOutput::CommonAccountIdentValidCheckPasswordNotEmpty(), &context).to_log());
                BIOSError::err(IamOutput::CommonAccountIdentValidCheckPasswordNotEmpty())
            }
        }
        AccountIdentKind::WechatXcx => {
            if let Some(account_token) = cache_processor::get_account_token(kind.to_string().to_lowercase().as_str(), &context).await? {
                if account_token == sk {
                    Ok("".to_string())
                } else {
                    log::warn!("{}", output(IamOutput::CommonAccountIdentValidCheckUserOrPasswordError(ak.to_string()), &context).to_log());
                    BIOSError::err(IamOutput::CommonAccountIdentValidCheckUserOrPasswordError(ak.to_string()))
                }
            } else {
                log::warn!("{}", output(IamOutput::CommonAccountIdentValidCheckUserOrPasswordError(ak.to_string()), &context).to_log());
                BIOSError::err(IamOutput::CommonAccountIdentValidCheckUserOrPasswordError(ak.to_string()))
            }
        }
        _ => {
            // TODO
            Ok("".to_string())
        }
    }
}

pub async fn init_account_role<'c>(role_code: &str, role_name: &str, tx: &mut Transaction<'c, MySql>, context: &BIOSContext) -> BIOSResult<String> {
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
                    context.ident.account_id.as_str().into(),
                    context.ident.account_id.as_str().into(),
                    role_code.into(),
                    role_name.into(),
                    0.into(),
                    context.ident.app_id.as_str().into(),
                    context.ident.tenant_id.as_str().into(),
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
                    context.ident.account_id.as_str().into(),
                    context.ident.account_id.as_str().into(),
                    context.ident.account_id.as_str().into(),
                    role_id.as_str().into(),
                ])
                .done(),
            Some(tx),
        )
        .await?;
    Ok(role_id)
}

pub async fn init_resource_subject<'c>(kind: &ResourceKind, ident_uri: &str, name: &str, tx: &mut Transaction<'c, MySql>, context: &BIOSContext) -> BIOSResult<String> {
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
                    IamResourceSubject::IdentUri,
                    IamResourceSubject::Name,
                    IamResourceSubject::Sort,
                    IamResourceSubject::Uri,
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
                    context.ident.account_id.as_str().into(),
                    context.ident.account_id.as_str().into(),
                    kind.to_string().to_lowercase().into(),
                    ident_uri.into(),
                    name.into(),
                    0.into(),
                    "".into(),
                    "".into(),
                    "".into(),
                    "".into(),
                    "".into(),
                    0.into(),
                    context.ident.app_id.as_str().into(),
                    context.ident.tenant_id.as_str().into(),
                ])
                .done(),
            Some(tx),
        )
        .await?;
    Ok(resource_subject_id)
}

pub async fn init_resource<'c>(
    path_and_query: &str,
    name: &str,
    resource_subject_id: &str,
    resource_expose_kind: &ExposeKind,
    tx: &mut Transaction<'c, MySql>,
    context: &BIOSContext,
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
                    context.ident.account_id.as_str().into(),
                    context.ident.account_id.as_str().into(),
                    path_and_query.into(),
                    name.into(),
                    "".into(),
                    0.into(),
                    "".into(),
                    false.into(),
                    "".into(),
                    resource_subject_id.into(),
                    context.ident.app_id.as_str().into(),
                    context.ident.tenant_id.as_str().into(),
                    resource_expose_kind.to_string().to_lowercase().into(),
                ])
                .done(),
            Some(tx),
        )
        .await?;
    Ok(resource_id)
}

pub async fn init_auth<'c>(
    auth_info: Vec<(&str, Vec<&OptActionKind>)>,
    name: &str,
    object_kind: &AuthObjectKind,
    object_id: &str,
    tx: &mut Transaction<'c, MySql>,
    context: &BIOSContext,
) -> BIOSResult<()> {
    // Init AuthPolicy
    async fn init_auth_policy<'c>(
        name: &str,
        action: &OptActionKind,
        resource_id: &str,
        result: &AuthResultKind,
        tx: &mut Transaction<'c, MySql>,
        context: &BIOSContext,
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
                        context.ident.account_id.as_str().into(),
                        context.ident.account_id.as_str().into(),
                        name.into(),
                        valid_start_time.into(),
                        valid_end_time.into(),
                        action.to_string().to_lowercase().into(),
                        resource_id.into(),
                        result.to_string().to_lowercase().into(),
                        context.ident.app_id.as_str().into(),
                        context.ident.tenant_id.as_str().into(),
                    ])
                    .done(),
                Some(tx),
            )
            .await?;
        Ok(auth_policy_id)
    }
    // Init AuthPolicyObject
    async fn init_auth_policy_object<'c>(
        object_kind: &AuthObjectKind,
        object_id: &str,
        auth_policy_id: &str,
        tx: &mut Transaction<'c, MySql>,
        context: &BIOSContext,
    ) -> BIOSResult<()> {
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
                        context.ident.account_id.as_str().into(),
                        context.ident.account_id.as_str().into(),
                        object_kind.to_string().to_lowercase().into(),
                        object_id.into(),
                        AuthObjectOperatorKind::Eq.to_string().to_lowercase().into(),
                        auth_policy_id.into(),
                    ])
                    .done(),
                Some(tx),
            )
            .await?;
        Ok(())
    }
    let auth_info = auth_info.iter().cloned().collect::<HashMap<&str, Vec<&OptActionKind>>>();
    for (resource_id, actions) in auth_info {
        for action in actions {
            let auth_policy_id = init_auth_policy(format!("{}权限", name).as_str(), action, &resource_id, &AuthResultKind::Accept, tx, context).await?;
            init_auth_policy_object(object_kind, object_id, &auth_policy_id, tx, context).await?;
        }
    }
    Ok(())
}

#[derive(sqlx::FromRow, serde::Deserialize)]
pub struct TenantIdentInfoResp {
    pub valid_ak_rule: String,
    pub valid_sk_rule: String,
    pub valid_time: i64,
}
