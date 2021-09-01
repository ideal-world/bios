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

use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct IdentInfo {
    pub app_id: Option<String>,
    pub tenant_id: Option<String>,
    pub account_id: Option<String>,
    pub token: Option<String>,
    pub token_kind: Option<String>,
    pub ak: Option<String>,
    pub roles: Option<Vec<String>>,
    pub groups: Option<Vec<String>>,
}

#[derive(Display, Debug, Deserialize, Serialize)]
pub enum ResourceKind {
    // API
    API,
    // 菜单
    MENU,
    // 页面元素
    ELEMENT,
    // OAuth
    OAUTH,
    // 关系数据库
    RELDB,
    // 缓存
    CACHE,
    // MQ
    MQ,
    // 对象存储
    OBJECT,
    // Task
    TASK,
}

#[derive(Display, Debug, Deserialize, Serialize)]
pub enum ExposeKind {
    // 应用级
    APP,
    // 租户级
    TENANT,
    // 系统级
    GLOBAL,
}
