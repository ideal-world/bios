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

use std::str::FromStr;

use derive_more::Display;
use serde::{Deserialize, Serialize};

/// 公共状态枚举
#[derive(Display, Debug, Deserialize, Serialize)]
pub enum CommonStatus {
    // 禁用
    Disabled,
    // 启用
    Enabled,
}

/// 账号认证类型枚举
///
/// 编码属性URI的Host部分，不能带下划线
#[derive(Display, Debug, Deserialize, Serialize)]
pub enum AccountIdentKind {
    // 用户名 + 密码
    Username,
    // 租户间授权认证
    AuthIdent,
    // 手机号 + 验证码
    Phone,
    // 邮箱 + 密码
    Email,
    // 微信小程序OAuth
    WechatXcx,
}

impl FromStr for AccountIdentKind {
    type Err = ();
    fn from_str(input: &str) -> Result<AccountIdentKind, Self::Err> {
        match input {
            "username" => Ok(AccountIdentKind::Username),
            "auth_ident" => Ok(AccountIdentKind::AuthIdent),
            "phone" => Ok(AccountIdentKind::Phone),
            "email" => Ok(AccountIdentKind::Email),
            "wechat_xcx" => Ok(AccountIdentKind::WechatXcx),
            _ => Err(()),
        }
    }
}

/// 资源类型枚举
#[derive(Display, Debug, Deserialize, Serialize)]
pub enum ResourceKind {
    // API
    Api,
    // 菜单
    Menu,
    // 页面元素
    Element,
    // OAuth
    Oauth,
    // 关系数据库
    Reldb,
    // 缓存
    Cache,
    // MQ
    Mq,
    // 对象存储
    Object,
    // Task
    Task,
}

/// 群组类型枚举
#[derive(Display, Debug, Deserialize, Serialize)]
pub enum GroupKind {
    // 行政
    Administration,
    // 虚拟
    Virtual,
}

/// 开放级别枚举
#[derive(Display, Debug, Deserialize, Serialize)]
pub enum ExposeKind {
    // 租户级
    Tenant,
    // 应用级
    App,
    // 系统级
    Global,
}

/// 权限对象类型枚举
#[derive(Display, Debug, Deserialize, Serialize)]
pub enum AuthObjectKind {
    // 租户
    Tenant,
    // 应用
    App,
    // 角色
    Role,
    // 群组节点
    GroupNode,
    // 账户
    Account,
}

/// 权限对象运算类型枚举
#[derive(Display, Debug, Deserialize, Serialize)]
pub enum AuthObjectOperatorKind {
    // 等于
    Eq,
    // 不等于
    Neq,
    // 包含，可用于群组当前及祖父节点
    Include,
    // LIKE，用于群组当前及子孙节点
    Like,
}

/// 操作类型枚举
///
/// 借用HTTP Method，但不严格与其语义对等
#[derive(Display, Debug, Deserialize, Serialize)]
pub enum OptActionKind {
    // 获取
    Get,
    // 创建
    Post,
    // 更新
    Put,
    // 局部更新
    Patch,
    // 删除
    Delete,
}

/// 权限结果类型枚举
#[derive(Display, Debug, Deserialize, Serialize)]
pub enum AuthResultKind {
    // 接受
    Accept,
    // 拒绝
    Reject,
}
