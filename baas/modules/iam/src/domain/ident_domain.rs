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

/// 租户
///
/// 数据隔离单位，不同租户间的数据不能直接共享。
/// 一个租户可以有多个应用。
#[derive(Iden, EnumIter, PartialEq, Copy, Clone)]
pub enum IamTenant {
    Table,
    Id,
    CreateUser,
    UpdateUser,
    CreateTime,
    UpdateTime,
    // 租户名称
    Name,
    // 租户图标
    Icon,
    // 是否开放账号注册
    AllowAccountRegister,
    // 租户扩展信息，Json格式
    Parameters,
    // 租户状态
    Status,
}

/// 租户凭证配置
///
/// 用于指定当前租户凭证类型及个性化配置。
/// 一个账号可以有一个或多个凭证，每个凭证类型有各自可保留的版本数量。
/// 此模型用于处理单点、单终端、多终端同时登录的问题。
#[derive(Iden, EnumIter, PartialEq, Copy, Clone)]
pub enum IamTenantCert {
    Table,
    Id,
    CreateUser,
    UpdateUser,
    CreateTime,
    UpdateTime,
    // 凭证类型名称
    Category,
    // 凭证保留的版本数量
    Version,
    // 关联租户Id
    RelTenantId,
}

/// 租户认证配置
///
/// 用于指定当前租户可使用的认证类型及个性化配置。
/// AccountIdentKind#USERNAME 为基础认证，所有租户都必须包含此认证类型。
/// 所有认证类型都支持使用自己的AK及基础认证的SK(密码)作为认证方式。
#[derive(Iden, EnumIter, PartialEq, Copy, Clone)]
pub enum IamTenantIdent {
    Table,
    Id,
    CreateUser,
    UpdateUser,
    CreateTime,
    UpdateTime,
    // 租户认证类型名称
    Kind,
    // 认证AK校验正则规则说明
    ValidAkRuleNote,
    // 认证AK校验正则规则
    ValidAkRule,
    // 认证SK校验正则规则说明
    ValidSkRuleNote,
    // 认证SK校验正则规则
    ValidSkRule,
    // 认证有效时间（秒）
    ValidTime,
    // 关联租户Id
    RelTenantId,
}

/// 应用
///
/// 面向业务系统，一般而言，一个应用对应于一个业务系统，但并不强制这种一对一的关系。
#[derive(Iden, EnumIter, PartialEq, Copy, Clone)]
pub enum IamApp {
    Table,
    Id,
    CreateUser,
    UpdateUser,
    CreateTime,
    UpdateTime,
    // 应用名称
    Name,
    // 应用图标
    Icon,
    // 应用扩展信息，Json格式
    Parameters,
    // 关联租户Id
    RelTenantId,
    // 应用状态
    Status,
}

/// 应用认证
#[derive(Iden, EnumIter, PartialEq, Copy, Clone)]
pub enum IamAppIdent {
    Table,
    Id,
    CreateUser,
    UpdateUser,
    CreateTime,
    UpdateTime,
    // 应用认证用途
    Note,
    // 应用认证名称（Access Key Id）
    Ak,
    // 应用认证密钥（Secret Access Key）
    Sk,
    // 认证有效时间（秒）
    ValidTime,
    // 关联应用Id
    RelAppId,
    // 关联租户Id
    RelTenantId,
}

/// 账号
///
/// 用户身份单位。
/// 隶属于租户，即便是同一个自然人在不同租户间也会有各自的账号，同一租户的不同应用共享账号信息。
#[derive(Iden, EnumIter, PartialEq, Copy, Clone)]
pub enum IamAccount {
    Table,
    Id,
    CreateUser,
    UpdateUser,
    CreateTime,
    UpdateTime,
    // OpenId是账号对外提供的主键，给业务方使用，用于标识账号唯一性的字段。
    OpenId,
    // 账号名称
    Name,
    // 账号头像
    Avatar,
    // 账号扩展信息，Json格式
    Parameters,
    // 父子账号可用于支持RAM（ Resource Access Management）用户功能。
    // 父子账号间OpenId相同。
    ParentId,
    // 关联租户Id
    RelTenantId,
    // 账号状态
    Status,
}

/// 账号认证
#[derive(Iden, EnumIter, PartialEq, Copy, Clone)]
pub enum IamAccountIdent {
    Table,
    Id,
    CreateUser,
    UpdateUser,
    CreateTime,
    UpdateTime,
    // 账号认证类型名称
    Kind,
    // 账号认证名称
    Ak,
    // 账号认证密钥
    Sk,
    // 账号认证有效开始时间
    ValidStartTime,
    // 账号认证有效结束时间
    ValidEndTime,
    // 关联账号Id
    RelAccountId,
    // 关联租户Id
    RelTenantId,
}

/// 账号应用关联
#[derive(Iden, EnumIter, PartialEq, Copy, Clone)]
pub enum IamAccountApp {
    Table,
    Id,
    CreateUser,
    UpdateUser,
    CreateTime,
    UpdateTime,
    // 关联应用Id
    RelAppId,
    // 关联账号Id
    RelAccountId,
}

/// 账号绑定
#[derive(Iden, EnumIter, PartialEq, Copy, Clone)]
pub enum IamAccountBind {
    Table,
    Id,
    CreateUser,
    UpdateUser,
    CreateTime,
    UpdateTime,
    // 源租户Id
    FromTenantId,
    // 源租户账号Id
    FromAccountId,
    // 目标租户Id
    ToTenantId,
    // 目标户账号Id
    ToAccountId,
    // 绑定使用的账号认证类型名称
    IdentKind,
}
