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

#[derive(Deserialize, Validate)]
pub struct AccountQueryReq {
    #[validate(length(min = 2, max = 255))]
    pub name: Option<String>,
    pub page_number: u64,
    pub page_size: u64,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct AccountAddReq {
    // 账号名称
    #[validate(length(min = 2, max = 255))]
    pub name: String,
    // 账号头像
    #[validate(length(min = 2, max = 1000))]
    pub avatar: Option<String>,
    // 账号扩展信息（Json格式）
    #[validate(length(min = 2, max = 2000))]
    pub parameters: Option<String>,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct AccountModifyReq {
    // 账号名称
    #[validate(length(min = 2, max = 255))]
    pub name: Option<String>,
    // 账号头像
    #[validate(length(min = 2, max = 1000))]
    pub avatar: Option<String>,
    // 账号扩展信息（Json格式）
    #[validate(length(min = 2, max = 2000))]
    pub parameters: Option<String>,
    // 父账号Id
    #[validate(length(max = 32))]
    pub parent_id: Option<String>,
    // 账号状态
    #[validate(length(min = 2, max = 255))]
    pub status: String,
}

#[derive(sqlx::FromRow, Deserialize, Serialize, Validate)]
pub struct AccountDetailResp {
    #[validate(length(max = 32))]
    pub id: String,
    // OpenId
    #[validate(length(max = 32))]
    pub open_id: String,
    // 账号名称
    #[validate(length(max = 255))]
    pub name: String,
    // 账号头像
    #[validate(length(max = 1000))]
    pub avatar: String,
    // 账号扩展信息（Json格式）
    #[validate(length(max = 2000))]
    pub parameters: String,
    // 父账号Id
    #[validate(length(max = 32))]
    pub parent_id: String,
    // 账号状态
    #[validate(length(max = 255))]
    pub status: String,
    #[validate(length(max = 255))]
    pub create_user: String,
    #[validate(length(max = 255))]
    pub update_user: String,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct AccountIdentAddReq {
    // 账号认证类型
    pub kind: AccountIdentKind,
    // 账号认证名称
    #[validate(length(min = 2, max = 255))]
    pub ak: String,
    // 账号认证密钥
    #[validate(length(min = 2, max = 255))]
    pub sk: Option<String>,
    // 账号认证有效开始时间
    pub valid_start_time: u64,
    // 账号认证有效结束时间
    pub valid_end_time: u64,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct AccountIdentModifyReq {
    // 账号认证名称
    #[validate(length(min = 2, max = 255))]
    pub ak: Option<String>,
    // 账号认证密钥
    #[validate(length(min = 2, max = 255))]
    pub sk: Option<String>,
    // 账号认证有效开始时间
    pub valid_start_time: Option<u64>,
    // 账号认证有效结束时间
    pub valid_end_time: Option<u64>,
}

#[derive(sqlx::FromRow, Deserialize, Serialize, Validate)]
pub struct AccountIdentDetailResp {
    #[validate(length(max = 32))]
    pub id: String,
    // 账号认证类型
    #[validate(length(max = 255))]
    pub kind: String,
    // 账号认证名称
    #[validate(length(max = 255))]
    pub ak: String,
    // 账号认证有效开始时间
    pub valid_start_time: i64,
    // 账号认证有效结束时间
    pub valid_end_time: i64,
    // 关联账号Id
    #[validate(length(max = 32))]
    pub rel_account_id: String,
    #[validate(length(max = 255))]
    pub create_user: String,
    #[validate(length(max = 255))]
    pub update_user: String,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct AccountBindAddReq {
    // 源租户Id
    #[validate(length(max = 32))]
    pub from_tenant_id: String,
    // 源租户账号Id
    #[validate(length(max = 32))]
    pub from_account_id: String,
    // 目标租户Id
    #[validate(length(max = 32))]
    pub to_tenant_id: String,
    // 目标租户账号Id
    #[validate(length(max = 32))]
    pub to_account_id: String,
    // 绑定使用的账号认证类型名称
    pub kind: AccountIdentKind,
}

#[derive(sqlx::FromRow, Deserialize, Serialize, Validate)]
pub struct AccountBindDetailResp {
    #[validate(length(max = 32))]
    pub id: String,
    // 源租户Id
    #[validate(length(max = 32))]
    pub from_tenant_id: String,
    // 源租户账号Id
    #[validate(length(max = 32))]
    pub from_account_id: String,
    // 目标租户Id
    #[validate(length(max = 32))]
    pub to_tenant_id: String,
    // 目标租户账号Id
    #[validate(length(max = 32))]
    pub to_account_id: String,
    // 绑定使用的账号认证类型名称
    #[validate(length(max = 255))]
    pub kind: String,
    #[validate(length(max = 255))]
    pub create_user: String,
    #[validate(length(max = 255))]
    pub update_user: String,
}
