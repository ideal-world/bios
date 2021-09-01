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

use bios::basic::error::{BIOSError, BIOSResult};
use bios::BIOSFuns;

use crate::iam_config::WorkSpaceConfig;
use crate::process::basic_dto::IdentInfo;

pub fn get_ident_info_by_app(req: &HttpRequest) -> BIOSResult<IdentInfo> {
    match get_ident_info(req) {
        Ok(ident_info) => {
            if ident_info.tenant_id.is_none() || ident_info.app_id.is_none() {
                Err(BIOSError::Unauthorized(
                    "[BIOS.BAAS.IAM] Ident Info [tenant_id] or [app_id] doesn't exist".to_owned(),
                ))
            } else {
                Ok(ident_info)
            }
        }
        Err(e) => Err(e),
    }
}

pub fn get_ident_info_by_account(req: &HttpRequest) -> BIOSResult<IdentInfo> {
    match get_ident_info(req) {
        Ok(ident_info) => {
            if ident_info.tenant_id.is_none()
                || ident_info.app_id.is_none()
                || ident_info.account_id.is_none()
            {
                Err(BIOSError::Unauthorized(
                    "[BIOS.BAAS.IAM] Ident Info [tenant_id] or [app_id] or [account_id] doesn't exist"
                        .to_owned(),
                ))
            } else {
                Ok(ident_info)
            }
        }
        Err(e) => Err(e),
    }
}

pub fn get_ident_info(req: &HttpRequest) -> BIOSResult<IdentInfo> {
    match extract_ident_info(req) {
        Some(ident_info) => Ok(ident_info),
        None => Err(BIOSError::Unauthorized(
            "[BIOS.BAAS.IAM] Ident Info doesn't exist".to_owned(),
        )),
    }
}

pub fn extract_ident_info(req: &HttpRequest) -> Option<IdentInfo> {
    req.headers()
        .get(&BIOSFuns::config::<WorkSpaceConfig>().ws.iam.ident_info_flag)
        .filter(|ident_info_str| !ident_info_str.is_empty())
        .map(|ident_info_str| {
            let ident_info_str = &ident_info_str
                .to_str()
                .expect("Ident Info convert to string error");
            let ident_info = bios::basic::security::digest::base64::decode(ident_info_str)
                .expect("Ident Info decode base64 error");
            bios::basic::json::str_to_obj(&ident_info).expect("Ident Info deserialize error")
        })
}
