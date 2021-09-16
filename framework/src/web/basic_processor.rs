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

use actix_web::HttpRequest;

use crate::basic::dto::{IdentAccountInfo, IdentAppInfo, IdentInfo, IdentPubInfo};
use crate::basic::error::{BIOSError, BIOSResult};
use crate::BIOSFuns;

pub fn get_ident_app_info(req: &HttpRequest) -> BIOSResult<IdentAppInfo> {
    match get_ident_info(req) {
        Ok(ident_info) => {
            if ident_info.tenant_id.is_none() || ident_info.app_id.is_none() || ident_info.ak.is_none() {
                Err(BIOSError::Unauthorized("Ident Info [tenant_id] or [app_id] or [ak] doesn't exists".to_owned()))
            } else {
                Ok(IdentAppInfo {
                    app_id: ident_info.app_id.unwrap(),
                    tenant_id: ident_info.tenant_id.unwrap(),
                    ak: ident_info.ak.unwrap(),
                })
            }
        }
        Err(e) => Err(e),
    }
}

pub fn get_ident_account_info(req: &HttpRequest) -> BIOSResult<IdentAccountInfo> {
    match get_ident_info(req) {
        Ok(ident_info) => {
            if ident_info.tenant_id.is_none() || ident_info.app_id.is_none() || ident_info.ak.is_none() || ident_info.account_id.is_none() {
                Err(BIOSError::Unauthorized(
                    "Ident Info [tenant_id] or [app_id] or [ak] or [account_id] doesn't exists".to_owned(),
                ))
            } else {
                Ok(IdentAccountInfo {
                    app_id: ident_info.app_id.unwrap(),
                    tenant_id: ident_info.tenant_id.unwrap(),
                    ak: ident_info.ak.unwrap(),
                    account_id: ident_info.account_id.unwrap(),
                    token: ident_info.token.unwrap(),
                    token_kind: ident_info.token_kind.unwrap_or_default(),
                    roles: ident_info.roles.unwrap_or_default(),
                    groups: ident_info.groups.unwrap_or_default(),
                })
            }
        }
        Err(e) => Err(e),
    }
}

pub fn get_ident_pub_info(req: &HttpRequest) -> BIOSResult<IdentPubInfo> {
    match get_ident_info(req) {
        Ok(_) => Ok(IdentPubInfo {}),
        Err(e) => Err(e),
    }
}

pub fn get_ident_info(req: &HttpRequest) -> BIOSResult<IdentInfo> {
    match extract_ident_info(req) {
        Some(ident_info) => Ok(ident_info),
        None => Err(BIOSError::Unauthorized("Ident Info doesn't exists".to_owned())),
    }
}

pub fn extract_ident_info(req: &HttpRequest) -> Option<IdentInfo> {
    req.headers()
        .get(&BIOSFuns::fw_config().web.ident_info_flag)
        .filter(|ident_info_str| !ident_info_str.is_empty())
        .map(|ident_info_str| {
            let ident_info_str = &ident_info_str.to_str().expect("Ident Info convert to string error");
            let ident_info = crate::basic::security::digest::base64::decode(ident_info_str).expect("Ident Info decode base64 error");
            crate::basic::json::str_to_obj(&ident_info).expect("Ident Info deserialize error")
        })
}
