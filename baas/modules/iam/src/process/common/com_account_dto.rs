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

use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::process::basic_dto::AccountIdentKind;

#[derive(Deserialize, Serialize, Validate)]
pub struct AccountChangeReq {
    // 账号名称
    #[validate(length(min = 2, max = 255))]
    pub name: Option<String>,
    // 账号头像
    #[validate(length(min = 2, max = 1000))]
    pub avatar: Option<String>,
    // 账号扩展信息，Json格式
    #[validate(length(min = 2, max = 2000))]
    pub parameters: Option<String>,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct AccountIdentChangeReq {
    // 账号认证类型
    pub kind: AccountIdentKind,
    // 账号认证名称
    #[validate(length(min = 2, max = 255), regex = "bios::basic::field::R_CODE_CS")]
    pub ak: String,
    // 账号认证密钥
    #[validate(length(min = 2, max = 255))]
    pub sk: String,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct AccountLoginReq {
    // 账号认证类型
    pub kind: AccountIdentKind,
    // 账号认证名称
    #[validate(length(min = 2, max = 255), regex = "bios::basic::field::R_CODE_CS")]
    pub ak: String,
    // 账号认证密钥
    #[validate(length(min = 2, max = 255))]
    pub sk: String,
    // 凭证分类
    #[validate(length(min = 2, max = 255))]
    pub cert_category: Option<String>,
    // 关联应用Id
    #[validate(length(max = 32))]
    pub rel_app_id: String,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct AccountOAuthLoginReq {
    // 认证类型，只能是OAuth类型的认证
    pub kind: AccountIdentKind,
    // 授权码
    #[validate(length(min = 2, max = 255))]
    pub code: String,
    // 关联应用Id
    #[validate(length(max = 32))]
    pub rel_app_id: String,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct AccountRegisterReq {
    // 账号名称
    #[validate(length(min = 2, max = 255))]
    pub name: String,
    // 账号头像
    #[validate(length(min = 2, max = 1000))]
    pub avatar: Option<String>,
    // 账号扩展信息，Json格式
    #[validate(length(min = 2, max = 2000))]
    pub parameters: Option<String>,
    // 账号认证类型
    pub kind: AccountIdentKind,
    // 账号认证名称
    #[validate(length(min = 2, max = 255), regex = "bios::basic::field::R_CODE_CS")]
    pub ak: String,
    // 账号认证密钥
    #[validate(length(min = 2, max = 255))]
    pub sk: String,
    // 关联应用Id
    #[validate(length(max = 32))]
    pub rel_app_id: String,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct AccountInfoDetailResp {
    #[validate(length(max = 32))]
    pub app_id: String,
    #[validate(length(max = 255))]
    pub app_name: String,
    #[validate(length(max = 32))]
    pub tenant_id: String,
    #[validate(length(max = 255))]
    pub tenant_name: String,
    #[validate(length(max = 32))]
    pub account_id: String,
    #[validate(length(max = 255))]
    pub account_name: String,
    #[validate(length(max = 255))]
    pub token: String,
    #[validate(length(max = 255))]
    pub token_kind: String,
    pub roles: Vec<(String, String)>,
    pub groups: Vec<(String, String)>,
}
