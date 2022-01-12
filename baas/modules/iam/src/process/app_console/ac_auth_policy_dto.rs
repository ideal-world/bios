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

use crate::process::basic_dto::{AuthObjectKind, AuthObjectOperatorKind, AuthResultKind, OptActionKind};
use sqlx::types::chrono::{DateTime, Utc};

#[derive(Deserialize, Validate)]
pub struct AuthPolicyQueryReq {
    #[validate(length(min = 2, max = 255))]
    pub name: Option<String>,
    pub valid_start_time: Option<i64>,
    pub valid_end_time: Option<i64>,
    #[validate(length(max = 32))]
    pub rel_resource_id: Option<String>,
    pub page_number: u64,
    pub page_size: u64,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct AuthPolicyAddReq {
    // 权限策略名称
    #[validate(length(min = 2, max = 255))]
    pub name: String,
    // 有效开始时间
    pub valid_start_time: i64,
    // 有效结束时间
    pub valid_end_time: i64,
    // 关联资源Id
    #[validate(length(max = 32))]
    pub rel_resource_id: String,
    // 操作类型
    pub action_kind: OptActionKind,
    // 操作结果
    pub result_kind: AuthResultKind,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct AuthPolicyModifyReq {
    // 权限策略名称
    #[validate(length(min = 2, max = 255))]
    pub name: Option<String>,
    // 有效开始时间
    pub valid_start_time: Option<i64>,
    // 有效结束时间
    pub valid_end_time: Option<i64>,
}

#[derive(sqlx::FromRow, Deserialize, Serialize, Validate)]
pub struct AuthPolicyDetailResp {
    #[validate(length(max = 32))]
    pub id: String,
    // 权限策略名称
    #[validate(length(max = 255))]
    pub name: String,
    // 有效开始时间
    pub valid_start_time: i64,
    // 有效结束时间
    pub valid_end_time: i64,
    // 关联资源Id
    #[validate(length(max = 32))]
    pub rel_resource_id: String,
    // 操作类型
    #[validate(length(max = 32))]
    pub action_kind: String,
    // 操作结果
    #[validate(length(max = 32))]
    pub result_kind: String,
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

#[derive(Deserialize, Serialize, Validate)]
pub struct AuthPolicyObjectAddReq {
    // 关联权限对象类型
    pub object_kind: AuthObjectKind,
    // 关联权限对象Id
    #[validate(length(min = 2, max = 255))]
    pub object_id: String,
    // 关联权限对象运算类型
    pub object_operator: AuthObjectOperatorKind,
}

#[derive(sqlx::FromRow, Deserialize, Serialize, Validate)]
pub struct AuthPolicyObjectDetailResp {
    #[validate(length(max = 32))]
    pub id: String,
    // 关联权限对象类型
    pub object_kind: String,
    // 关联权限对象Id
    #[validate(length(max = 255))]
    pub object_id: String,
    // 关联权限对象运算类型
    #[validate(length(max = 32))]
    pub object_operator: String,
    // 关联权限策略Id
    #[validate(length(max = 32))]
    pub rel_auth_policy_id: String,
    #[validate(length(max = 255))]
    pub create_user: String,
    #[validate(length(max = 255))]
    pub update_user: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}
