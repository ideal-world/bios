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
use sqlx::types::chrono::{DateTime, Utc};
use validator::Validate;

#[derive(Deserialize, Serialize, Validate)]
pub struct AppIdentAddReq {
    // 应用认证用途
    #[validate(length(min = 2, max = 1000))]
    pub note: String,
    // 应用认证有效时间
    pub valid_time: i64,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct AppIdentModifyReq {
    // 应用认证用途
    #[validate(length(min = 2, max = 1000))]
    pub note: Option<String>,
    // 应用认证有效时间
    pub valid_time: Option<i64>,
}

#[derive(sqlx::FromRow, Deserialize, Serialize, Validate)]
pub struct AppIdentDetailResp {
    #[validate(length(max = 32))]
    pub id: String,
    // 应用认证用途
    #[validate(length(max = 1000))]
    pub note: String,
    // 应用认证名称
    #[validate(length(max = 255))]
    pub ak: String,
    // 应用认证有效时间
    pub valid_time: i64,
    // 所属应用Id
    #[validate(length(max = 32))]
    pub rel_app_id: String,
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
