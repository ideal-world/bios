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

use sea_query::Iden;
use strum::EnumIter;

// 通用配置信息
#[derive(Iden, EnumIter, PartialEq, Copy, Clone)]
pub enum BiosConfig {
    Table,
    Id,
    CreateUser,
    UpdateUser,
    CreateTime,
    UpdateTime,
    K,
    V,
}

// 记录删除信息
#[derive(Iden, EnumIter, PartialEq, Copy, Clone)]
pub enum BiosDelRecord {
    Table,
    Id,
    CreateUser,
    CreateTime,
    // 对象名称
    EntityName,
    // 记录Id
    RecordId,
    // 删除内容，Json格式
    Content,
}
