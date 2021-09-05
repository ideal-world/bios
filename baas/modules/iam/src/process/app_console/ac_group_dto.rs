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

use crate::process::basic_dto::{ExposeKind, GroupKind};

#[derive(Deserialize, Validate)]
pub struct GroupQueryReq {
    #[validate(length(min = 2, max = 255))]
    pub code: Option<String>,
    #[validate(length(min = 2, max = 255))]
    pub name: Option<String>,
    pub page_number: u64,
    pub page_size: u64,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct GroupAddReq {
    // 群组编码
    #[validate(length(min = 2, max = 255))]
    pub code: String,
    // 群组名称
    #[validate(length(min = 2, max = 255))]
    pub name: String,
    // 群组类型
    pub kind: GroupKind,
    // 群组显示排序，asc
    pub sort: u32,
    // 群组图标
    #[validate(length(max = 1000))]
    pub icon: String,
    // 关联群组Id，用于多树合成
    #[validate(length(max = 32))]
    pub rel_group_id: Option<String>,
    // 关联群起始组节点Id，用于多树合成
    #[validate(length(max = 32))]
    pub rel_group_node_id: Option<String>,
    // 开放等级类型
    pub expose_kind: Option<ExposeKind>,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct GroupModifyReq {
    // 群组编码
    #[validate(length(min = 2, max = 255))]
    pub code: Option<String>,
    // 群组名称
    #[validate(length(min = 2, max = 255))]
    pub name: Option<String>,
    // 群组类型
    pub kind: Option<GroupKind>,
    // 群组显示排序，asc
    pub sort: Option<u32>,
    // 群组图标
    #[validate(length(max = 1000))]
    pub icon: Option<String>,
    // 关联群组Id，用于多树合成
    #[validate(length(max = 32))]
    pub rel_group_id: Option<String>,
    // 关联群起始组节点Id，用于多树合成
    #[validate(length(max = 32))]
    pub rel_group_node_id: Option<String>,
    // 开放等级类型
    pub expose_kind: Option<ExposeKind>,
}

#[derive(sqlx::FromRow, Deserialize, Serialize, Validate)]
pub struct GroupDetailResp {
    #[validate(length(max = 32))]
    pub id: String,
    // 群组编码
    #[validate(length(max = 255))]
    pub code: String,
    // 群组名称
    #[validate(length(max = 255))]
    pub name: String,
    // 群组类型
    #[validate(length(max = 255))]
    pub kind: String,
    // 群组显示排序，asc
    pub sort: i32,
    // 群组图标
    #[validate(length(max = 1000))]
    pub icon: String,
    // 关联群组Id，用于多树合成
    #[validate(length(max = 32))]
    pub rel_group_id: String,
    // 关联群起始组节点Id，用于多树合成
    #[validate(length(max = 32))]
    pub rel_group_node_id: String,
    // 开放等级类型
    #[validate(length(max = 255))]
    pub expose_kind: String,
    // 所属应用Id
    #[validate(length(max = 32))]
    pub rel_app_id: String,
    // 所属租户Id
    #[validate(length(max = 32))]
    pub rel_tenant_id: String,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct GroupNodeAddReq {
    // 业务编码
    #[validate(length(min = 2, max = 1000))]
    pub bus_code: String,
    // 节点名称
    #[validate(length(min = 2, max = 255))]
    pub name: String,
    // 节点扩展信息，Json格式
    #[validate(length(max = 2000))]
    pub parameters: String,
    // 上级节点Id
    #[validate(length(max = 32))]
    pub parent_id: Option<String>,
    // 同级上一个节点Id
    #[validate(length(max = 32))]
    pub sibling_id: Option<String>,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct GroupNodeModifyReq {
    // 业务编码
    #[validate(length(min = 2, max = 1000))]
    pub bus_code: Option<String>,
    // 节点名称
    #[validate(length(min = 2, max = 255))]
    pub name: Option<String>,
    // 节点扩展信息，Json格式
    #[validate(length(max = 2000))]
    pub parameters: Option<String>,
    // 上级节点Id
    #[validate(length(max = 32))]
    pub parent_id: Option<String>,
    // 同级上一个节点Id
    #[validate(length(max = 32))]
    pub sibling_id: Option<String>,
}

#[derive(sqlx::FromRow, Deserialize, Serialize, Validate)]
pub struct GroupNodeDetailResp {
    #[validate(length(max = 32))]
    pub id: String,
    // 业务编码
    #[validate(length(max = 1000))]
    pub bus_code: String,
    // 节点名称
    #[validate(length(max = 255))]
    pub name: String,
    // 节点扩展信息，Json格式
    #[validate(length(max = 2000))]
    pub parameters: String,
    // 上级节点Id
    #[validate(length(max = 32))]
    pub parent_id: String,
    // 同级上一个节点Id
    #[validate(length(max = 32))]
    pub sibling_id: String,
    // 关联群组Id
    #[validate(length(max = 32))]
    pub rel_group_id: String,
}
