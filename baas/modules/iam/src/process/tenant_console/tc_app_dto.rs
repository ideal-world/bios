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
pub struct AppQueryReq {
    #[validate(length(min = 2, max = 255))]
    pub name: Option<String>,
    pub page_number: u64,
    pub page_size: u64,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct AppAddReq {
    // 应用名称
    #[validate(length(min = 2, max = 255))]
    pub name: String,
    // 应用图标
    #[validate(length(min = 2, max = 1000))]
    pub icon: Option<String>,
    // 应用扩展信息（Json格式）
    #[validate(length(min = 2, max = 2000))]
    pub parameters: Option<String>,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct AppModifyReq {
    // 应用名称
    #[validate(length(min = 2, max = 255))]
    pub name: Option<String>,
    // 应用图标
    #[validate(length(min = 2, max = 1000))]
    pub icon: Option<String>,
    // 应用扩展信息（Json格式）
    #[validate(length(min = 2, max = 2000))]
    pub parameters: Option<String>,
    // 应用状态
    #[validate(length(min = 2, max = 255))]
    pub status: Option<String>,
}

#[derive(sqlx::FromRow, Deserialize, Serialize, Validate)]
pub struct AppDetailResp {
    #[validate(length(max = 32))]
    pub id: String,
    // 应用名称
    #[validate(length(max = 255))]
    pub name: String,
    // 应用图标
    #[validate(length(max = 1000))]
    pub icon: String,
    // 应用扩展信息（Json格式）
    #[validate(length(max = 2000))]
    pub parameters: String,
    // 应用状态
    #[validate(length(max = 255))]
    pub status: String,
}
