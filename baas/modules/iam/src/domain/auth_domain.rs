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

/// 群组
#[derive(Iden, EnumIter, PartialEq, Copy, Clone)]
pub enum IamGroup {
    Table,
    Id,
    CreateUser,
    UpdateUser,
    CreateTime,
    UpdateTime,
    // 群组编码
    Code,
    // 群组类型名称
    Kind,
    // 群组名称
    Name,
    // Icon
    Icon,
    // 显示排序，asc
    Sort,
    // 关联群组Id，用于多树合成
    RelGroupId,
    // 关联群起始组节点Id，用于多树合成
    RelGroupNodeId,
    // 关联应用Id
    RelAppId,
    // 关联租户Id
    RelTenantId,
    // 开放等级类型名称
    ExposeKind,
}

/// 群组节点
#[derive(Iden, EnumIter, PartialEq, Copy, Clone)]
pub enum IamGroupNode {
    Table,
    Id,
    CreateUser,
    UpdateUser,
    CreateTime,
    UpdateTime,
    //群组节点编码
    Code,
    // 显示排序，asc
    Sort,
    // 业务编码
    BusCode,
    // 节点名称
    Name,
    // 节点扩展信息，Json格式
    Parameters,
    // 关联群组Id
    RelGroupId,
}

/// 账号群组
#[derive(Iden, EnumIter, PartialEq, Copy, Clone)]
pub enum IamAccountGroup {
    Table,
    Id,
    CreateUser,
    UpdateUser,
    CreateTime,
    UpdateTime,
    // 关联账号Id
    RelAccountId,
    // 关联群组节点Id
    RelGroupNodeId,
}

/// 角色
#[derive(Iden, EnumIter, PartialEq, Copy, Clone)]
pub enum IamRole {
    Table,
    Id,
    CreateUser,
    UpdateUser,
    CreateTime,
    UpdateTime,
    // 角色编码
    Code,
    // 角色名称
    Name,
    // 显示排序，asc
    Sort,
    // 关联应用Id
    RelAppId,
    // 关联租户Id
    RelTenantId,
}

/// 账号角色
#[derive(Iden, EnumIter, PartialEq, Copy, Clone)]
pub enum IamAccountRole {
    Table,
    Id,
    CreateUser,
    UpdateUser,
    CreateTime,
    UpdateTime,
    // 关联账号Id
    RelAccountId,
    // 关联角色Id
    RelRoleId,
}

/// 资源主体
///
/// 所有三方调用都视为资源，需要配置资源主体，比如微信公众号、华为云等
///
/// ResourceKind#MENU:
/// uri = MENU路径
/// e.g.
/// uri = menu://iam
///
/// ResourceKind#ELEMENT:
/// uri = 元素路径
/// e.g.
/// uri = element://iam
///
/// ResourceKind#API:
/// uri = API路径
/// e.g.
/// uri = http://10.20.0.10:8080/iam
/// uri = https://iam/iam
///
/// ResourceKind#RELDB:
/// uri = 数据库连接地址
/// e.g.
/// uri = mysql://user1:92njc93nt39n@192.168.0.100:3306/test?useUnicode=true&characterEncoding=utf-8&rewriteBatchedStatements=true
/// uri = h2:./xyy.db;AUTO_SERVER=TRUE
///
/// ResourceKind#CACHE:
/// uri = 缓存连接地址
/// e.g.
/// uri = redis://:diwn9234@localhost:6379/1
///
/// ResourceKind#MQ:
/// uri = MQ连接地址
/// e.g.
/// uri = amqp://user1:onsw3223@localhost:10000/vhost1
///
/// ResourceKind#OBJECT:
/// uri = 对象存储连接地址
/// e.g.
/// uri = https://test-bucket.obs.cn-north-4.myhuaweicloud.com/test-object?acl
#[derive(Iden, EnumIter, PartialEq, Copy, Clone)]
pub enum IamResourceSubject {
    Table,
    Id,
    CreateUser,
    UpdateUser,
    CreateTime,
    UpdateTime,
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

/// 资源
///
/// URI格式： resource kind://resource subject code/path?property-key=property-value
///
/// path_and_query 为空时 表示为（整个）该资源主体
///
/// ResourceKind#API:
/// path_and_query = API路径
/// e.g.
/// path_and_query = admin/**
/// path_and_query = admin/user
/// 当 ResourceSubject 的 ResourceSubject#Uri = http://10.20.0.10:8080/iam 时
/// 则 资源的真正URI = http://10.20.0.10:8080/iam/admin/user
///
/// ResourceKind#MENU:
/// path_and_query = 菜单树节点Id
/// e.g.
/// path_and_query = userMgr/batchImport ，表示 用户管理（userMgr）/批量导入（batchImport）
///
/// ResourceKind#ELEMENT:
/// path_and_query = 页面路径/元素Id
/// 或 path_and_query = 页面路径?属性名=属性值
/// e.g.
/// path_and_query = userMgr/userDelete ，表示 用户管理页面（userMgr）的删除按钮（id  = 'userDelete'）
/// path_and_query = userMgr?class=userDelete ，表示 用户管理页面（userMgr）的删除按钮（class = 'userDelete'）
///
/// ResourceKind#RELDB:
/// path_and_query = 表名
/// 或 path_and_query = 表名/fields/字段名
/// 或 path_and_query = 表名/rows/主键值
/// 或 path_and_query = 表名/rows?字段名=字段值
/// e.g.
/// path_and_query = user ，表示 user表
/// path_and_query = user/fields/idcard ，表示 user表idcard字段
/// path_and_query = user/rows/100 ，表示 user表主键为100
/// path_and_query = user/rows?idcard=331xxx ，user表身份证为331xxx
///
/// ResourceKind#CACHE:
/// path_and_query = Key名称
/// e.g.
/// path_and_query = user:ids ，表示 user:ids 的Key
///
/// ResourceKind#MQ:
/// path_and_query = Topic名称
/// e.g.
/// path_and_query = addUser ，表示 addUser 主题
///
/// ResourceKind#OBJECT:
/// path_and_query = Key名称
/// e.g.
/// path_and_query = user/100 ，表示 user/100 的key
#[derive(Iden, EnumIter, PartialEq, Copy, Clone)]
pub enum IamResource {
    Table,
    Id,
    CreateUser,
    UpdateUser,
    CreateTime,
    UpdateTime,
    // path and query
    PathAndQuery,
    // 资源名称
    Name,
    // 资源图标
    Icon,
    // 资源显示排序，asc
    Sort,
    // 触发后的操作，多用于菜单链接
    Action,
    // 是否是资源组
    ResGroup,
    // 资源所属组Id
    ParentId,
    // 关联资源主体Id
    RelResourceSubjectId,
    // 关联应用Id
    RelAppId,
    // 关联租户Id
    RelTenantId,
    // 开放等级类型名称
    ExposeKind,
}

/// 权限策略
///
/// 支持跨应用、租户的权限分配（发布--订阅模式）
/// 仅资源所有者可以分配自己的资源权限
#[derive(Iden, EnumIter, PartialEq, Copy, Clone)]
pub enum IamAuthPolicy {
    Table,
    Id,
    CreateUser,
    UpdateUser,
    CreateTime,
    UpdateTime,
    // 权限策略名称
    Name,
    // 生效时间
    ValidStartTime,
    // 失效时间
    ValidEndTime,
    // 关联资源Id
    RelResourceId,
    // 操作类型名称
    ActionKind,
    // 操作结果名称
    ResultKind,
    // 关联应用Id
    RelAppId,
    // 关联租户Id
    RelTenantId,
}

/// 权限策略关联对象
#[derive(Iden, EnumIter, PartialEq, Copy, Clone)]
pub enum IamAuthPolicyObject {
    Table,
    Id,
    CreateUser,
    UpdateUser,
    CreateTime,
    UpdateTime,
    // 关联权限对象类型名称
    ObjectKind,
    // 关联权限对象Id
    ObjectId,
    // 关联权限对象运算类型名称
    ObjectOperator,
    // 关联权限策略
    RelAuthPolicyId,
}
