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

use crate::process::basic_dto::{ExposeKind, ResourceKind};

#[derive(Deserialize, Validate)]
pub struct ResourceSubjectQueryReq {
    #[validate(length(min = 2, max = 255))]
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
    #[validate(length(max = 32))]
    pub id: String,
    // 资源主体编码
    #[validate(length(max = 255))]
    pub code: String,
    // 资源主体名称
    #[validate(length(max = 255))]
    pub name: String,
    // 资源主体显示排序，asc
    pub sort: i32,
    // 资源类型
    #[validate(length(max = 255))]
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
    #[validate(length(max = 255))]
    pub create_user: String,
    #[validate(length(max = 255))]
    pub update_user: String,
}

#[derive(Deserialize, Validate)]
pub struct ResourceQueryReq {
    #[validate(length(min = 2, max = 255))]
    pub name: Option<String>,
    #[validate(length(min = 2, max = 5000))]
    pub path_and_query: Option<String>,
    pub expose: bool,
    pub page_number: u64,
    pub page_size: u64,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct ResourceAddReq {
    // 资源名称
    #[validate(length(min = 2, max = 255))]
    pub name: String,
    // 资源路径
    #[validate(length(max = 5000))]
    pub path_and_query: String,
    // 资源图标
    #[validate(length(max = 1000))]
    pub icon: String,
    // 触发后的操作，多用于菜单链接
    #[validate(length(max = 5000))]
    pub action: Option<String>,
    // 资源显示排序，asc
    pub sort: u32,
    // 是否是资源组
    pub res_group: bool,
    // 资源所属组Id
    #[validate(length(max = 32))]
    pub parent_id: Option<String>,
    // 关联资源主体Id
    #[validate(length(max = 32))]
    pub rel_resource_subject_id: String,
    // 开放等级类型
    pub expose_kind: Option<ExposeKind>,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct ResourceModifyReq {
    // 资源名称
    #[validate(length(min = 2, max = 255))]
    pub name: Option<String>,
    // 资源路径
    #[validate(length(max = 5000))]
    pub path_and_query: Option<String>,
    // 资源图标
    #[validate(length(max = 1000))]
    pub icon: Option<String>,
    // 触发后的操作，多用于菜单链接
    #[validate(length(max = 5000))]
    pub action: Option<String>,
    // 资源显示排序，asc
    pub sort: Option<u32>,
    // 是否是资源组
    pub res_group: Option<bool>,
    // 资源所属组Id
    #[validate(length(max = 32))]
    pub parent_id: Option<String>,
    // 开放等级类型
    pub expose_kind: Option<ExposeKind>,
}

#[derive(sqlx::FromRow, Deserialize, Serialize, Validate)]
pub struct ResourceDetailResp {
    #[validate(length(max = 32))]
    pub id: String,
    // 资源名称
    #[validate(length(max = 255))]
    pub name: String,
    // 资源路径
    #[validate(length(max = 5000))]
    pub path_and_query: String,
    // 资源图标
    #[validate(length(max = 1000))]
    pub icon: String,
    // 触发后的操作，多用于菜单链接
    #[validate(length(max = 5000))]
    pub action: String,
    // 资源显示排序，asc
    pub sort: i32,
    // 是否是资源组
    pub res_group: bool,
    // 资源所属组Id
    #[validate(length(max = 32))]
    pub parent_id: String,
    // 关联资源主体Id
    #[validate(length(max = 32))]
    pub rel_resource_subject_id: String,
    // 开放等级类型
    #[validate(length(min = 2, max = 255))]
    pub expose_kind: String,
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
}
