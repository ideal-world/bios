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
use validator::Validate;

#[derive(Deserialize, Serialize, Validate)]
pub struct ResourceDetailResp {
    // 资源名称
    #[validate(length(max = 255))]
    pub name: TrimString,
    // 资源uri
    #[validate(length(max = 5000))]
    pub ident_uri: String,
    // 资源图标
    #[validate(length(max = 1000))]
    pub icon: String,
    // 触发后的操作，多用于菜单链接
    #[validate(length(max = 5000))]
    pub action: String,
    // 资源显示排序，asc
    pub sort: u32,
    // 是否是资源组
    pub res_group: bool,
    // 资源所属组Id
    #[validate(length(max = 32))]
    pub parent_id: String,
}
