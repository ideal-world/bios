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
use sqlx::types::chrono::{DateTime, Utc};
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct RoleQueryReq {
    #[validate(length(min = 2, max = 255))]
    pub code: Option<String>,
    #[validate(length(min = 2, max = 255))]
    pub name: Option<String>,
    pub page_number: u64,
    pub page_size: u64,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct RoleAddReq {
    // 角色编码
    #[validate(length(min = 2, max = 255), regex = "bios::basic::field::R_CODE_CS")]
    pub code: String,
    // 角色名称
    #[validate(length(min = 2, max = 255))]
    pub name: String,
    // 角色显示排序，asc
    pub sort: i32,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct RoleModifyReq {
    // 角色编码
    #[validate(length(min = 2, max = 255), regex = "bios::basic::field::R_CODE_CS")]
    pub code: Option<String>,
    // 资源主体名称
    #[validate(length(min = 2, max = 255))]
    pub name: Option<String>,
    // 资源主体显示排序，asc
    pub sort: Option<i32>,
}

#[derive(sqlx::FromRow, Deserialize, Serialize, Validate)]
pub struct RoleDetailResp {
    #[validate(length(max = 32))]
    pub id: String,
    // 角色编码
    #[validate(length(max = 255))]
    pub code: String,
    // 角色名称
    #[validate(length(max = 255))]
    pub name: String,
    // 角色显示排序，asc
    pub sort: i32,
    // 所属应用Id
    #[validate(length(max = 32))]
    pub rel_app_code: String,
    // 所属租户Id
    #[validate(length(max = 32))]
    pub rel_tenant_id: String,
    #[validate(length(max = 255))]
    pub create_user: String,
    #[validate(length(max = 255))]
    pub update_user: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}
