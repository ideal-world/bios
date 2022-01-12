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

use async_trait::async_trait;

use bios::basic::dto::BIOSContext;
use bios::basic::error::BIOSError;
use bios::basic::result::{output, BIOSResult};
use bios::web::web_client::BIOSWebClient;
use bios::BIOSFuns;

use crate::iam_constant::IamOutput;
use crate::process::basic_dto::AccountIdentKind;
use crate::process::common::oauth::platform_api::{PlatformAPI, UserInfoResp};

#[derive(serde::Deserialize)]
pub struct WechatXcx {}

#[async_trait(?Send)]
impl PlatformAPI for WechatXcx {
    fn get_platform_flag(&self) -> String {
        AccountIdentKind::WechatXcx.to_string().to_lowercase()
    }

    async fn do_get_user_info(&self, code: &str, ak: &str, sk: &str, context: &BIOSContext) -> BIOSResult<UserInfoResp> {
        let mut response = BIOSFuns::web_client()
            .raw()
            .post(format!(
                "https://api.weixin.qq.com/sns/jscode2session?appid={}&secret={}&js_code={}&grant_type=authorization_code",
                ak, sk, code
            ))
            .send()
            .await?;
        let status_code = response.status();
        let body = BIOSWebClient::body_as_str(&mut response).await?;
        if status_code.as_u16() != 200 {
            log::warn!(
                "{}",
                output(
                    IamOutput::CommonOAuthFetchOAuthInfoError(self.get_platform_flag(), format!("[{}]{}", status_code.as_u16(), body)),
                    context
                )
                .to_log()
            );
            return IamOutput::CommonOAuthFetchOAuthInfoError(self.get_platform_flag(), "WeChat interface call exception".to_string())?;
        }
        log::trace!(
            "{}",
            output(
                IamOutput::CommonOAuthFetchOAuthInfoTrace(self.get_platform_flag(), format!("[{}]{}", status_code.as_u16(), body)),
                context
            )
            .to_log()
        );
        let user_info = bios::basic::json::str_to_json(&body)?;
        if user_info.get("errcode").is_some() && user_info["errcode"].as_str().unwrap_or_default() != "0" {
            return IamOutput::CommonOAuthFetchOAuthInfoError(
                self.get_platform_flag(),
                format!(
                    "[{}]{}",
                    user_info["errcode"].as_str().unwrap_or_default(),
                    user_info["errmsg"].as_str().unwrap_or_default()
                ),
            )?;
        }
        Ok(bios::basic::json::json_to_obj(user_info)?)
    }

    async fn do_get_access_token(&self, ak: &str, sk: &str, context: &BIOSContext) -> BIOSResult<(String, i64)> {
        let mut response =
            BIOSFuns::web_client().raw().get(format!("https://api.weixin.qq.com/cgi-bin/token?grant_type=client_credential&appid={}&secret={}", ak, sk)).send().await?;
        let status_code = response.status();
        let body = BIOSWebClient::body_as_str(&mut response).await?;
        if status_code.as_u16() != 200 {
            log::warn!(
                "{}",
                output(
                    IamOutput::CommonOAuthFetchAccessTokenError(self.get_platform_flag(), format!("[{}]{}", status_code.as_u16(), body)),
                    context
                )
                .to_log()
            );
            return IamOutput::CommonOAuthFetchAccessTokenError(self.get_platform_flag(), "WeChat interface call exception".to_string())?;
        }
        let access_token_info = bios::basic::json::str_to_json(&body)?;
        if access_token_info.get("access_token").is_none() {
            return IamOutput::CommonOAuthFetchAccessTokenTrace(
                self.get_platform_flag(),
                format!("[{}]{}", access_token_info["errcode"].as_str().unwrap_or_default(), "WeChat interface call exception"),
            )?;
        }
        let access_token = access_token_info["access_token"].as_str().unwrap();
        let expire = access_token_info["expires_in"].as_i64().unwrap();
        Ok((access_token.to_string(), expire))
    }
}
