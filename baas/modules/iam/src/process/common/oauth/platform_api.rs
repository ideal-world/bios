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

use async_trait::async_trait;

use bios::basic::dto::BIOSContext;
use bios::basic::result::BIOSResult;
use bios::BIOSFuns;

use crate::iam_config::WorkSpaceConfig;

#[async_trait(?Send)]
pub trait PlatformAPI {
    fn get_platform_flag(&self) -> String;

    async fn do_get_user_info(&self, code: &str, ak: &str, sk: &str, context: &BIOSContext) -> BIOSResult<UserInfoResp>;

    async fn do_get_access_token(&self, ak: &str, sk: &str, context: &BIOSContext) -> BIOSResult<(String, i64)>;

    async fn get_user_info(&self, code: &str, ak: &str, sk: &str, context: &BIOSContext) -> BIOSResult<(String, UserInfoResp)> {
        let user_info = self.do_get_user_info(code, ak, sk, context).await?;
        let access_token = self.do_get_access_token(ak, sk, context).await?;
        Ok((access_token.0, user_info))
    }

    async fn get_account_token(&self, ak: &str, sk: &str, context: &BIOSContext) -> BIOSResult<String> {
        let cache_account_token_info = BIOSFuns::cache()
            .get(
                format!(
                    "{}{}:{}",
                    BIOSFuns::ws_config::<WorkSpaceConfig>().iam.cache.access_token,
                    context.ident.app_id,
                    self.get_platform_flag()
                )
                .as_str(),
            )
            .await?;
        if cache_account_token_info.is_some() {
            return Ok(cache_account_token_info.unwrap());
        } else {
            let account_token_info = self.do_get_access_token(ak, sk, context).await?;
            BIOSFuns::cache()
                .set_ex(
                    format!(
                        "{}{}:{}",
                        BIOSFuns::ws_config::<WorkSpaceConfig>().iam.cache.access_token,
                        context.ident.app_id,
                        self.get_platform_flag()
                    )
                    .as_str(),
                    account_token_info.0.as_str(),
                    account_token_info.1 as usize,
                )
                .await?;
            return Ok(account_token_info.0);
        }
    }
}

#[derive(serde::Deserialize)]
pub struct UserInfoResp {
    // 用户唯一标识
    pub account_open_id: String,
    // 同一平台下的多个用户共用一个标识
    pub account_union_id: String,
    pub session_key: i64,
}
