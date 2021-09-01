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

#[derive(Iden)]
pub enum IamResourceSubject {
    Table,
    Id,
    CreateUser,
    UpdateUser,
    CreateTime,
    UpdateTime,
    // 资源主体编码
    Code,
    // 资源类型名称
    Kind,
    // 资源主体连接URI
    Uri,
    // 资源主体名称
    Name,
    // 资源主体显示排序，asc
    Sort,
    // AK，部分类型支持写到URI中
    Ak,
    // SK，部分类型支持写到URI中
    Sk,
    // 第三方平台账号名
    PlatformAccount,
    // 第三方平台项目名，如华为云的ProjectId
    PlatformProjectId,
    // 执行超时
    TimeoutMs,
    // 关联应用Id
    RelAppId,
    // 关联租户Id
    RelTenantId,
}

// 通用配置信息
#[derive(Iden)]
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
#[derive(Iden)]
pub enum BiosDelRecord {
    Table,
    Id,
    CreateUser,
    CreateTime,
    // 对象名称
    EntityName,
    // 记录Id
    Content,
    // 删除内容，Json格式
    RecordId,
}
