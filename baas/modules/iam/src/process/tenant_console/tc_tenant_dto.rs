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

use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::process::basic_dto::AccountIdentKind;

#[derive(Deserialize, Serialize, Validate)]
pub struct TenantModifyReq {
    // 租户名称
    #[validate(length(min = 2, max = 255))]
    pub name: Option<String>,
    // 租户图标
    #[validate(length(min = 2, max = 1000))]
    pub icon: Option<String>,
    // 是否开放账号注册
    pub allow_account_register: Option<bool>,
    // 租户扩展信息，Json格式
    #[validate(length(min = 2, max = 5000))]
    pub parameters: Option<String>,
}

#[derive(sqlx::FromRow, Deserialize, Serialize, Validate)]
pub struct TenantDetailResp {
    #[validate(length(max = 32))]
    pub id: String,
    // 租户名称
    #[validate(length(max = 255))]
    pub name: String,
    // 租户图标
    #[validate(length(max = 1000))]
    pub icon: String,
    // 是否开放账号注册
    pub allow_account_register: bool,
    // 租户扩展信息，Json格式
    #[validate(length(max = 5000))]
    pub parameters: String,
    // 租户状态
    #[validate(length(max = 255))]
    pub status: String,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct TenantCertAddReq {
    // 凭证类型名称
    #[validate(length(min = 2, max = 255))]
    pub category: String,
    // 凭证保留的版本数量
    pub version: u32,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct TenantCertModifyReq {
    // 凭证类型名称
    #[validate(length(min = 2, max = 255))]
    pub category: Option<String>,
    // 凭证保留的版本数量
    pub version: Option<u32>,
}

#[derive(sqlx::FromRow, Deserialize, Serialize, Validate)]
pub struct TenantCertDetailResp {
    #[validate(length(max = 32))]
    pub id: String,
    // 凭证类型名称
    #[validate(length(max = 255))]
    pub category: String,
    // 凭证保留的版本数量
    pub version: u32,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct TenantIdentAddReq {
    // 租户认证类型名称
    pub kind: AccountIdentKind,
    // 认证AK校验正则规则说明
    #[validate(length(min = 2, max = 2000))]
    pub valid_ak_rule_node: String,
    // 认证AK校验正则规则
    #[validate(length(min = 2, max = 2000))]
    pub valid_ak_rule: String,
    // 认证SK校验正则规则说明
    #[validate(length(min = 2, max = 2000))]
    pub valid_sk_rule_node: String,
    // 认证SK校验正则规则
    #[validate(length(min = 2, max = 2000))]
    pub valid_sk_rule: String,
    // 认证有效时间（秒）
    pub valid_time: u32,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct TenantIdentModifyReq {
    // 租户认证类型名称
    pub kind: Option<AccountIdentKind>,
    // 认证AK校验正则规则说明
    #[validate(length(min = 2, max = 2000))]
    pub valid_ak_rule_node: Option<String>,
    // 认证AK校验正则规则
    #[validate(length(min = 2, max = 2000))]
    pub valid_ak_rule: Option<String>,
    // 认证SK校验正则规则说明
    #[validate(length(min = 2, max = 2000))]
    pub valid_sk_rule_node: Option<String>,
    // 认证SK校验正则规则
    #[validate(length(min = 2, max = 2000))]
    pub valid_sk_rule: Option<String>,
    // 认证有效时间（秒）
    pub valid_time: Option<u32>,
}

#[derive(sqlx::FromRow, Deserialize, Serialize, Validate)]
pub struct TenantIdentDetailResp {
    #[validate(length(max = 32))]
    pub id: String,
    // 租户认证类型名称
    #[validate(length(max = 255))]
    pub kind: String,
    // 认证AK校验正则规则说明
    #[validate(length(max = 2000))]
    pub valid_ak_rule_node: String,
    // 认证AK校验正则规则
    #[validate(length(max = 2000))]
    pub valid_ak_rule: String,
    // 认证SK校验正则规则说明
    #[validate(length(max = 2000))]
    pub valid_sk_rule_node: String,
    // 认证SK校验正则规则
    #[validate(length(max = 2000))]
    pub valid_sk_rule: String,
    // 认证有效时间（秒）
    pub valid_time: i32,
}
