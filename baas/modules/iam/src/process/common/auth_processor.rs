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

use bios::basic::error::{BIOSError, BIOSResult};
use bios::db::reldb_client::SqlBuilderProcess;
use bios::BIOSFuns;

use crate::domain::ident_domain::IamTenantIdent;
use crate::iam_config::WorkSpaceConfig;
use crate::process::basic_dto::AccountIdentKind;
use crate::process::common::cache_processor;

lazy_static! {
    static ref AK_SK_CONTAINER: Mutex<HashMap<String, Regex>> = Mutex::new(HashMap::new());
}

pub async fn valid_account_ident<'c>(kind: &AccountIdentKind, ak: &str, sk: &str, rel_tenant_id: &str, tx: Option<&mut Transaction<'c, MySql>>) -> BIOSResult<i64> {
    if let Some(tenant_ident_info) = BIOSFuns::reldb()
        .fetch_optional::<TenantIdentInfoResp>(
            &Query::select()
                .columns(vec![IamTenantIdent::ValidAkRule, IamTenantIdent::ValidSkRule, IamTenantIdent::ValidTime])
                .from(IamTenantIdent::Table)
                .and_where(Expr::col(IamTenantIdent::Kind).eq(kind.to_string().to_lowercase()))
                .and_where(Expr::col(IamTenantIdent::RelTenantId).eq(rel_tenant_id.to_string()))
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
                return Err(BIOSError::BadRequest("AccountIdent [ak] invalid format".to_string()));
            }
        }
        if !sk.is_empty() && !tenant_ident_info.valid_sk_rule.is_empty() {
            let mut aksks = AK_SK_CONTAINER.lock().unwrap();
            if !aksks.contains_key(&tenant_ident_info.valid_sk_rule) {
                aksks.insert(tenant_ident_info.valid_sk_rule.clone(), Regex::new(&tenant_ident_info.valid_sk_rule).unwrap());
            }
            if !aksks.get(&tenant_ident_info.valid_sk_rule).unwrap().is_match(sk) {
                return Err(BIOSError::BadRequest("AccountIdent [sk] invalid format".to_string()));
            }
        }
        Ok(Utc::now().timestamp() + tenant_ident_info.valid_time)
    } else {
        Err(BIOSError::NotFound("AccountIdent [kind] not exists".to_string()))
    }
}

pub async fn validate_sk(kind: &AccountIdentKind, ak: &str, request_sk: &str, stored_sk: &str, tenant_id: &str, app_id: &str) -> BIOSResult<()> {
    match kind {
        AccountIdentKind::Phone | AccountIdentKind::Email => {
            if let Some(tmp_sk) = cache_processor::get_vcode(tenant_id, ak).await? {
                if tmp_sk == request_sk {
                    cache_processor::remove_vcode(tenant_id, ak).await?;
                    cache_processor::remove_vcode_error_times(tenant_id, ak).await?;
                    Ok(())
                } else {
                    let error_times = cache_processor::incr_vcode_error_times(tenant_id, ak).await?;
                    if error_times >= BIOSFuns::ws_config::<WorkSpaceConfig>().iam.security.account_vcode_max_error_times as usize {
                        cache_processor::remove_vcode(tenant_id, ak).await?;
                        cache_processor::remove_vcode_error_times(tenant_id, ak).await?;
                        log::warn!("Verification code [{}] in tenant [{}] over the maximum times", ak, tenant_id);
                        Err(BIOSError::Conflict("Verification code over the maximum times".to_string()))
                    } else {
                        log::warn!("Verification code [{}] in tenant [{}] doesn't match", ak, tenant_id);
                        Err(BIOSError::Conflict("Verification code doesn't exist or has expired".to_string()))
                    }
                }
            } else {
                log::warn!("Verification code [{}] in tenant [{}] doesn't exist or has expired", ak, tenant_id);
                Err(BIOSError::Conflict("Verification code doesn't exist or has expired".to_string()))
            }
        }
        AccountIdentKind::Username => {
            if !request_sk.trim().is_empty() {
                if bios::basic::security::digest::digest(format!("{}{}", ak, request_sk).as_str(), None, "SHA512") == stored_sk {
                    Ok(())
                } else {
                    log::warn!("Username [{}] or Password [{}] in tenant [{}] error", ak, request_sk, tenant_id);
                    Err(BIOSError::Conflict("Username or Password error".to_string()))
                }
            } else {
                Err(BIOSError::BadRequest("Password can't be empty".to_string()))
            }
        }
        AccountIdentKind::WechatXcx => {
            if let Some(account_token) = cache_processor::get_account_token(app_id, kind.to_string().to_lowercase().as_str()).await? {
                if account_token == request_sk {
                    Ok(())
                } else {
                    log::warn!("Account token [{}] in tenant [{}] doesn't match", account_token, tenant_id);
                    Err(BIOSError::Conflict("Account Token doesn't exist or has expired".to_string()))
                }
            } else {
                log::warn!("Account token in tenant [{}] doesn't exist or has expired", tenant_id);
                Err(BIOSError::Conflict("Account Token doesn't exist or has expired".to_string()))
            }
        }
        _ => Err(BIOSError::BadRequest("Unsupported authentication kind".to_string())),
    }
}

pub async fn process_sk(kind: &AccountIdentKind, ak: &str, sk: &str, tenant_id: &str, app_id: &str) -> BIOSResult<String> {
    match kind {
        AccountIdentKind::Phone | AccountIdentKind::Email => {
            if let Some(tmp_sk) = cache_processor::get_vcode(tenant_id, ak).await? {
                if tmp_sk == sk {
                    Ok(sk.to_string())
                } else {
                    Err(BIOSError::Conflict("Verification code doesn't exist or has expired".to_string()))
                }
            } else {
                Err(BIOSError::Conflict("Verification code doesn't exist or has expired".to_string()))
            }
        }
        AccountIdentKind::Username => {
            if !sk.trim().is_empty() {
                Ok(bios::basic::security::digest::digest(format!("{}{}", ak, sk).as_str(), None, "SHA512"))
            } else {
                Err(BIOSError::Conflict("Password can't be empty".to_string()))
            }
        }
        AccountIdentKind::WechatXcx => {
            if let Some(account_token) = cache_processor::get_account_token(app_id, kind.to_string().to_lowercase().as_str()).await? {
                if account_token == sk {
                    Ok("".to_string())
                } else {
                    Err(BIOSError::Conflict("Account Token doesn't exist or has expired".to_string()))
                }
            } else {
                Err(BIOSError::Conflict("Account Token doesn't exist or has expired".to_string()))
            }
        }
        _ => {
            // TODO
            Ok("".to_string())
        }
    }
}

#[derive(sqlx::FromRow, serde::Deserialize)]
pub struct TenantIdentInfoResp {
    pub valid_ak_rule: String,
    pub valid_sk_rule: String,
    pub valid_time: i64,
}
