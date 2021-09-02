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

use crate::process::basic_dto::ResourceKind;

#[derive(Deserialize, Validate)]
pub struct ResourceSubjectQueryReq {
    #[validate(length(min = 2, max = 10))]
    pub name: Option<String>,
    pub page_number: u64,
    pub page_size: u64,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct ResourceSubjectAddReq {
    // 资源主体编码后缀
    #[validate(length(min = 2, max = 255))]
    pub code_postfix: String,
    // 资源主体名称
    #[validate(length(min = 2, max = 255))]
    pub name: String,
    // 资源主体显示排序，asc
    pub sort: u32,
    // 资源类型
    pub kind: ResourceKind,
    // 资源主体连接URI
    #[validate(url)]
    pub uri: String,
    // AK，部分类型支持写到URI中
    #[validate(length(min = 2, max = 1000))]
    pub ak: Option<String>,
    // SK，部分类型支持写到URI中
    #[validate(length(min = 2, max = 1000))]
    pub sk: Option<String>,
    // 第三方平台账号名
    #[validate(length(min = 2, max = 1000))]
    pub platform_account: Option<String>,
    // 第三方平台项目名，如华为云的ProjectId
    #[validate(length(min = 2, max = 1000))]
    pub platform_project_id: Option<String>,
    // 执行超时
    pub timeout_ms: Option<u32>,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct ResourceSubjectModifyReq {
    // 资源主体编码后缀
    #[validate(length(min = 2, max = 255))]
    pub code_postfix: Option<String>,
    // 资源主体名称
    #[validate(length(min = 2, max = 255))]
    pub name: Option<String>,
    // 资源主体显示排序，asc
    pub sort: Option<u32>,
    // 资源类型
    pub kind: Option<ResourceKind>,
    // 资源主体连接URI
    #[validate(url)]
    pub uri: Option<String>,
    // AK，部分类型支持写到URI中
    #[validate(length(max = 1000))]
    pub ak: Option<String>,
    // SK，部分类型支持写到URI中
    #[validate(length(max = 1000))]
    pub sk: Option<String>,
    // 第三方平台账号名
    #[validate(length(max = 1000))]
    pub platform_account: Option<String>,
    // 第三方平台项目名，如华为云的ProjectId
    #[validate(length(max = 1000))]
    pub platform_project_id: Option<String>,
    // 执行超时
    pub timeout_ms: Option<u32>,
}

#[derive(sqlx::FromRow, Deserialize, Serialize, Validate)]
pub struct ResourceSubjectDetailResp {
    #[validate(length(max = 64))]
    pub id: String,
    // 资源主体编码
    #[validate(length(min = 2, max = 255))]
    pub code: String,
    // 资源主体名称
    #[validate(length(min = 2, max = 255))]
    pub name: String,
    // 资源主体显示排序，asc
    pub sort: i32,
    // 资源类型
    pub kind: String,
    // 资源主体连接URI
    #[validate(url)]
    pub uri: String,
    // AK，部分类型支持写到URI中
    #[validate(length(max = 1000))]
    pub ak: String,
    // SK，部分类型支持写到URI中
    #[validate(length(max = 1000))]
    pub sk: String,
    // 第三方平台账号名
    #[validate(length(max = 1000))]
    pub platform_account: String,
    // 第三方平台项目名，如华为云的ProjectId
    #[validate(length(max = 1000))]
    pub platform_project_id: String,
    // 执行超时
    pub timeout_ms: i32,
    // 所属应用Id
    #[validate(length(max = 32))]
    pub rel_app_id: String,
    // 所属租户Id
    #[validate(length(max = 32))]
    pub rel_tenant_id: String,
}
