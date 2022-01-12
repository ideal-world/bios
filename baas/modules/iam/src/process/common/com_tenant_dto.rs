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

use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Deserialize, Serialize, Validate)]
pub struct TenantRegisterReq {
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
    // 应用名称
    #[validate(length(min = 2, max = 255))]
    pub app_name: String,
    // 租户管理员登录用户名
    #[validate(length(min = 2, max = 255))]
    pub account_username: String,
    // 租户管理员登录密钥
    #[validate(length(min = 2, max = 255))]
    pub account_password: String,
}
