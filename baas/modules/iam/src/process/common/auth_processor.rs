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

use bios::basic::error::{BIOSError, BIOSResult};
use bios::db::reldb_client::SqlBuilderProcess;
use bios::BIOSFuns;

use crate::domain::ident_domain::IamTenantIdent;
use crate::process::common::cache_processor;

lazy_static! {
    static ref AK_SK_CONTAINER: Mutex<HashMap<String, Regex>> = Mutex::new(HashMap::new());
}

pub async fn valid_account_ident(kind: &str, ak: &str, sk: &str, rel_tenant_id: &str) -> BIOSResult<i64> {
    if let Some(tenant_ident_info) = BIOSFuns::reldb()
        .fetch_optional::<TenantIdentInfoResp>(
            &Query::select()
                .columns(vec![IamTenantIdent::ValidAkRule, IamTenantIdent::ValidSkRule, IamTenantIdent::ValidTime])
                .from(IamTenantIdent::Table)
                .and_where(Expr::col(IamTenantIdent::Kind).eq(kind.to_string()))
                .and_where(Expr::col(IamTenantIdent::RelTenantId).eq(rel_tenant_id.to_string()))
                .done(),
            None,
        )
        .await?
    {
        if ak.is_empty() && !tenant_ident_info.valid_ak_rule.is_empty() {
            let mut aksks = AK_SK_CONTAINER.lock().unwrap();
            if !aksks.contains_key(&tenant_ident_info.valid_ak_rule) {
                aksks.insert(tenant_ident_info.valid_ak_rule.clone(), Regex::new(&tenant_ident_info.valid_ak_rule).unwrap()).unwrap();
            }
            if !aksks.get(&tenant_ident_info.valid_ak_rule).unwrap().is_match(ak) {
                return Err(BIOSError::BadRequest("AccountIdent [sk] invalid format".to_string()));
            }
        }
        if sk.is_empty() && !tenant_ident_info.valid_sk_rule.is_empty() {
            let mut aksks = AK_SK_CONTAINER.lock().unwrap();
            if !aksks.contains_key(&tenant_ident_info.valid_sk_rule) {
                aksks.insert(tenant_ident_info.valid_sk_rule.clone(), Regex::new(&tenant_ident_info.valid_sk_rule).unwrap()).unwrap();
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

pub async fn process_sk(kind: &str, ak: &str, sk: &str, rel_tenant_id: &str, rel_app_id: &str) -> BIOSResult<String> {
    match kind {
        "email" | "phone" => {
            if let Some(tmp_sk) = cache_processor::get_vcode(rel_tenant_id, ak).await? {
                if tmp_sk == sk {
                    Ok(sk.to_string())
                } else {
                    Err(BIOSError::Conflict("The verification code doesn't exist or has expired".to_string()))
                }
            } else {
                Err(BIOSError::Conflict("The verification code doesn't exist or has expired".to_string()))
            }
        }
        "username" => {
            if !sk.trim().is_empty() {
                Ok(bios::basic::security::digest::digest(format!("{}{}", ak, sk).as_str(), None, "SHA512"))
            } else {
                Err(BIOSError::Conflict("Password can't be empty".to_string()))
            }
        }
        "wechat_xcx" => {
            if let Some(account_token) = cache_processor::get_account_token(rel_app_id, kind).await? {
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
