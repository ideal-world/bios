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

use crate::basic::dto::BIOSContext;
use crate::basic::error::BIOSError;
use crate::basic::result::BIOSResult;
use crate::BIOSFuns;

pub fn extract_context_with_account(req: &HttpRequest) -> BIOSResult<BIOSContext> {
    match extract_context(req) {
        Ok(context) => {
            if context.ident.tenant_id.is_empty() || context.ident.app_id.is_empty() || context.ident.ak.is_empty() || context.ident.account_id.is_empty() {
                Err(BIOSError::Unauthorized(
                    "BIOS Context [tenant_id] or [app_id] or [ak] or [account_id] doesn't exists".to_string(),
                ))
            } else {
                Ok(context)
            }
        }
        Err(e) => Err(e),
    }
}

pub fn extract_context(req: &HttpRequest) -> BIOSResult<BIOSContext> {
    if !req.headers().contains_key(&BIOSFuns::fw_config().web.context_flag) {
        return Err(BIOSError::BadRequest("BIOS Context doesn't exists".to_string()));
    }
    let context = req.headers().get(&BIOSFuns::fw_config().web.context_flag).expect("BIOS Context doesn't exists");
    let context = context.to_str().expect("BIOS Context convert to string error");
    let context = BIOSFuns::security.base64.decode(context)?;
    let mut context = BIOSFuns::json.str_to_obj::<BIOSContext>(&context)?;
    context.trace.app = BIOSFuns::fw_config().app.id.to_string();
    context.trace.inst = BIOSFuns::fw_config().app.inst.to_string();
    if req.headers().contains_key(&BIOSFuns::fw_config().web.lang_flag) {
        context.lang = req.headers().get(&BIOSFuns::fw_config().web.lang_flag).unwrap().to_str().unwrap().to_string();
    }
    Ok(context)
}
