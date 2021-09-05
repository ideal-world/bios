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

#[derive(Deserialize, Validate)]
pub struct TenantQueryReq {
    #[validate(length(min = 2, max = 255))]
    pub name: Option<String>,
    pub page_number: u64,
    pub page_size: u64,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct TenantAddReq {
    // 租户名称
    #[validate(length(min = 2, max = 255))]
    pub name: String,
    // 租户图标
    #[validate(length(min = 2, max = 1000))]
    pub icon: Option<String>,
    // 是否开放账号注册
    pub allow_account_register: bool,
    // 租户扩展信息，Json格式
    #[validate(length(min = 2, max = 5000))]
    pub parameters: Option<String>,
}

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
