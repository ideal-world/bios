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


create schema if not exists iam collate utf8mb4_0900_ai_ci;

use iam;

create table if not exists iam_account
(
    id            varchar(64)
    primary key,
    open_id       varchar(100)                        not null comment 'Open Id',
    name          varchar(255)                        not null comment '账号名称',
    avatar        varchar(1000)                       not null comment '账号头像',
    parameters    varchar(2000)                       not null comment '账号扩展信息，Json格式',
    parent_id     varchar(64)                         not null comment '父账号Id，不存在时为空',
    status        varchar(50)                         not null comment '账号状态',
    rel_tenant_id varchar(64)                         not null comment '关联租户Id',
    create_time   timestamp default CURRENT_TIMESTAMP null comment '创建时间',
    create_user   varchar(64)                         not null comment '创建者Id',
    update_time   timestamp default CURRENT_TIMESTAMP null on update CURRENT_TIMESTAMP comment '最后一次修改时间',
    update_user   varchar(64)                         not null comment '最后一次修改者Id',
    constraint u_open_id_parent_id
    unique (open_id, parent_id)
    )
    comment '账号';

create index i_tenant_status
    on iam_account (rel_tenant_id, status);

create table if not exists iam_account_app
(
    id             varchar(64)
    primary key,
    rel_account_id varchar(64)                         not null comment '关联账号Id',
    rel_app_id     varchar(64)                         not null comment '关联应用Id',
    create_time    timestamp default CURRENT_TIMESTAMP null comment '创建时间',
    create_user    varchar(64)                         not null comment '创建者Id',
    update_time    timestamp default CURRENT_TIMESTAMP null on update CURRENT_TIMESTAMP comment '最后一次修改时间',
    update_user    varchar(64)                         not null comment '最后一次修改者Id',
    constraint u_account_app
    unique (rel_app_id, rel_account_id)
    )
    comment '账号应用关联';

create index i_app
    on iam_account_app (rel_app_id);

create table if not exists iam_account_bind
(
    id              varchar(64)
    primary key,
    from_account_id varchar(64)                         not null comment '源租户账号Id',
    from_tenant_id  varchar(64)                         not null comment '源租户Id',
    ident_kind      varchar(255)                        not null comment '绑定使用的账号认证类型名称',
    to_account_id   varchar(64)                         not null comment '目标户账号Id',
    to_tenant_id    varchar(64)                         not null comment '目标租户Id',
    create_time     timestamp default CURRENT_TIMESTAMP null comment '创建时间',
    create_user     varchar(64)                         not null comment '创建者Id',
    update_time     timestamp default CURRENT_TIMESTAMP null on update CURRENT_TIMESTAMP comment '最后一次修改时间',
    update_user     varchar(64)                         not null comment '最后一次修改者Id',
    constraint u_from_to_account
    unique (from_account_id, to_account_id)
    )
    comment '账号绑定';

create index i_to_tenant
    on iam_account_bind (to_tenant_id);

create index i_from_tenant
    on iam_account_bind (from_tenant_id);

create table if not exists iam_account_group
(
    id                varchar(64)
    primary key,
    rel_account_id    varchar(64)                         not null comment '关联账号Id',
    rel_group_node_id varchar(64)                         not null comment '关联群组节点Id',
    create_time       timestamp default CURRENT_TIMESTAMP null comment '创建时间',
    create_user       varchar(64)                         not null comment '创建者Id',
    update_time       timestamp default CURRENT_TIMESTAMP null on update CURRENT_TIMESTAMP comment '最后一次修改时间',
    update_user       varchar(64)                         not null comment '最后一次修改者Id',
    constraint u_account_group
    unique (rel_account_id, rel_group_node_id)
    )
    comment '账号群组关联';

create index i_group
    on iam_account_group (rel_group_node_id);

create table if not exists iam_account_ident
(
    id               varchar(64)
    primary key,
    kind             varchar(100)                        not null comment '账号认证类型名称',
    ak               varchar(255)                        not null comment '账号认证名称',
    sk               varchar(255)                        not null comment '账号认证密钥',
    valid_end_time   bigint                              not null comment '账号认证有效结束时间',
    valid_start_time bigint                              not null comment '账号认证有效开始时间',
    rel_account_id   varchar(64)                         not null comment '关联账号Id',
    rel_tenant_id    varchar(64)                         not null comment '关联租户Id',
    create_time      timestamp default CURRENT_TIMESTAMP null comment '创建时间',
    create_user      varchar(64)                         not null comment '创建者Id',
    update_time      timestamp default CURRENT_TIMESTAMP null on update CURRENT_TIMESTAMP comment '最后一次修改时间',
    update_user      varchar(64)                         not null comment '最后一次修改者Id',
    constraint u_tenant_kind_ak
    unique (rel_tenant_id, kind, ak)
    )
    comment '账号认证';

create index i_valid1
    on iam_account_ident (rel_tenant_id, rel_account_id, kind, ak, valid_start_time, valid_end_time);

create index i_valid2
    on iam_account_ident (rel_account_id, kind, valid_start_time, valid_end_time);

create table if not exists iam_account_role
(
    id             varchar(64)
    primary key,
    rel_account_id varchar(64)                         not null comment '关联账号Id',
    rel_role_id    varchar(64)                         not null comment '关联角色Id',
    create_time    timestamp default CURRENT_TIMESTAMP null comment '创建时间',
    create_user    varchar(64)                         not null comment '创建者Id',
    update_time    timestamp default CURRENT_TIMESTAMP null on update CURRENT_TIMESTAMP comment '最后一次修改时间',
    update_user    varchar(64)                         not null comment '最后一次修改者Id',
    constraint u_account_role
    unique (rel_account_id, rel_role_id)
    )
    comment '账号角色关联';

create index i_role
    on iam_account_role (rel_role_id);

create table if not exists iam_app
(
    id            varchar(64)
    primary key,
    name          varchar(255)                        not null comment '应用名称',
    icon          varchar(1000)                       not null comment '应用图标',
    parameters    varchar(2000)                       not null comment '应用扩展信息，Json格式',
    status        varchar(50)                         not null comment '应用状态',
    rel_tenant_id varchar(64)                         not null comment '关联租户Id',
    create_time   timestamp default CURRENT_TIMESTAMP null comment '创建时间',
    create_user   varchar(64)                         not null comment '创建者Id',
    update_time   timestamp default CURRENT_TIMESTAMP null on update CURRENT_TIMESTAMP comment '最后一次修改时间',
    update_user   varchar(64)                         not null comment '最后一次修改者Id',
    constraint u_tenant_name
    unique (rel_tenant_id, name)
    )
    comment '应用';

create index i_status
    on iam_app (status);

create table if not exists iam_app_ident
(
    id            varchar(64)
    primary key,
    ak            varchar(255)                        not null comment '应用认证名称（Access Key Id）',
    sk            varchar(1000)                       not null comment '应用认证密钥（Secret Access Key）',
    valid_time    bigint                              not null comment '应用认证有效时间',
    note          varchar(1000)                       not null comment '应用认证用途',
    rel_app_id    varchar(64)                         not null comment '关联应用Id',
    rel_tenant_id varchar(64)                         not null comment '关联租户Id',
    create_time   timestamp default CURRENT_TIMESTAMP null comment '创建时间',
    create_user   varchar(64)                         not null comment '创建者Id',
    update_time   timestamp default CURRENT_TIMESTAMP null on update CURRENT_TIMESTAMP comment '最后一次修改时间',
    update_user   varchar(64)                         not null comment '最后一次修改者Id',
    constraint u_ak
    unique (ak)
    )
    comment '应用认证';

create index i_app_valid
    on iam_app_ident (rel_app_id, valid_time);

create table if not exists iam_auth_policy
(
    id                           varchar(64)
    primary key,
    name                         varchar(255)                        not null comment '权限策略名称',
    action_kind                  varchar(100)                        not null comment '操作类型名称',
    rel_resource_id              varchar(64)                         not null comment '关联资源Id',
    result_kind                  varchar(100)                        not null comment '操作结果名称',
    valid_start_time             bigint                              not null comment '生效时间',
    valid_end_time               bigint                              not null comment '失效时间',
    rel_app_id                   varchar(64)                         not null comment '关联应用Id',
    rel_tenant_id                varchar(64)                         not null comment '关联租户Id',
    create_time                  timestamp default CURRENT_TIMESTAMP null comment '创建时间',
    create_user                  varchar(64)                         not null comment '创建者Id',
    update_time                  timestamp default CURRENT_TIMESTAMP null on update CURRENT_TIMESTAMP comment '最后一次修改时间',
    update_user                  varchar(64)                         not null comment '最后一次修改者Id'
    )
    comment '权限策略';

create table if not exists iam_auth_policy_object
(
    id                  varchar(64)
    primary key,
    object_kind         varchar(20)                         not null comment '关联权限对象类型名称',
    object_id           varchar(255)                        not null comment '关联权限对象Id',
    object_operator     varchar(20)                         not null comment '关联权限对象运算类型名称',
    rel_auth_policy_id  varchar(64)                         not null comment '关联权限策略Id',
    create_time         timestamp default CURRENT_TIMESTAMP null comment '创建时间',
    create_user         varchar(64)                         not null comment '创建者Id',
    update_time         timestamp default CURRENT_TIMESTAMP null on update CURRENT_TIMESTAMP comment '最后一次修改时间',
    update_user         varchar(64)                         not null comment '最后一次修改者Id'
    )
    comment '权限策略关联对象';

create table if not exists iam_group
(
    id                varchar(64)
    primary key,
    kind              varchar(100)                        not null comment '群组类型名称',
    code              varchar(255)                        not null comment '群组编码',
    name              varchar(255)                        not null comment '群组名称',
    icon              varchar(1000)                       not null comment '群组图标',
    expose_kind       varchar(100)                        not null comment '开放等级类型名称',
    rel_group_id      varchar(64)                         not null comment '关联群组Id，用于多树合成',
    rel_group_node_id varchar(64)                         not null comment '关联群起始组节点Id，用于多树合成',
    sort              int                                 not null comment '显示排序，asc',
    rel_app_id        varchar(64)                         not null comment '关联应用Id',
    rel_tenant_id     varchar(64)                         not null comment '关联租户Id',
    create_time       timestamp default CURRENT_TIMESTAMP null comment '创建时间',
    create_user       varchar(64)                         not null comment '创建者Id',
    update_time       timestamp default CURRENT_TIMESTAMP null on update CURRENT_TIMESTAMP comment '最后一次修改时间',
    update_user       varchar(64)                         not null comment '最后一次修改者Id',
    constraint u_tenant_app_id
    unique (rel_tenant_id, rel_app_id, code)
    )
    comment '群组';

create index i_expose
    on iam_group (expose_kind);

create table if not exists iam_group_node
(
    id           varchar(64)
    primary key,
    code         varchar(1000)                       not null comment '节点编码',
    sort         int                                 not null comment '显示排序，asc',
    bus_code     varchar(500)                        not null comment '业务编码',
    name         varchar(255)                        not null comment '节点名称',
    parameters   varchar(2000)                       not null comment '节点扩展信息，Json格式',
    rel_group_id varchar(64)                         not null comment '关联群组Id',
    create_time  timestamp default CURRENT_TIMESTAMP null comment '创建时间',
    create_user  varchar(64)                         not null comment '创建者Id',
    update_time  timestamp default CURRENT_TIMESTAMP null on update CURRENT_TIMESTAMP comment '最后一次修改时间',
    update_user  varchar(64)                         not null comment '最后一次修改者Id'
    )
    comment '群组节点';

create table if not exists iam_resource
(
    id                      varchar(64)
    primary key,
    path_and_query          varchar(5000)                       not null comment 'Path and Query',
    name                    varchar(255)                        not null comment '资源名称',
    icon                    varchar(1000)                       not null comment '资源图标',
    expose_kind             varchar(100)                        not null comment '开放等级类型名称',
    parent_id               varchar(64)                         not null comment '资源所属组Id',
    res_group               tinyint(1)                          not null comment '是否是资源组',
    action                  varchar(1000)                       not null comment '触发后的操作，多用于菜单链接',
    sort                    int                                 not null comment '资源显示排序，asc',
    rel_resource_subject_id varchar(64)                         not null comment '关联资源主体Id',
    rel_app_id              varchar(64)                         not null comment '关联应用Id',
    rel_tenant_id           varchar(64)                         not null comment '关联租户Id',
    create_time             timestamp default CURRENT_TIMESTAMP null comment '创建时间',
    create_user             varchar(64)                         not null comment '创建者Id',
    update_time             timestamp default CURRENT_TIMESTAMP null on update CURRENT_TIMESTAMP comment '最后一次修改时间',
    update_user             varchar(64)                         not null comment '最后一次修改者Id'
    )
    comment '资源';

create index i_expose
    on iam_resource (expose_kind);

create index i_parent
    on iam_resource (parent_id);

create table if not exists iam_resource_subject
(
    id                  varchar(64)
    primary key,
    kind                varchar(100)                        not null comment '资源类型名称',
    ident_uri           varchar(5000)                       not null comment '资源主体标识URI',
    uri                 varchar(5000)                       not null comment '资源主体连接URI',
    name                varchar(255)                        not null comment '资源主体名称',
    ak                  varchar(1000)                       not null comment 'AK，部分类型支持写到URI中',
    sk                  varchar(1000)                       not null comment 'SK，部分类型支持写到URI中',
    platform_account    varchar(1000)                       not null comment '第三方平台账号名',
    platform_project_id varchar(1000)                       not null comment '第三方平台项目名，如华为云的ProjectId',
    sort                int                                 not null comment '资源主体显示排序，asc',
    timeout_ms          int                                 not null comment '执行超时',
    rel_app_id          varchar(64)                         not null comment '关联应用Id',
    rel_tenant_id       varchar(64)                         not null comment '关联租户Id',
    create_time         timestamp default CURRENT_TIMESTAMP null comment '创建时间',
    create_user         varchar(64)                         not null comment '创建者Id',
    update_time         timestamp default CURRENT_TIMESTAMP null on update CURRENT_TIMESTAMP comment '最后一次修改时间',
    update_user         varchar(64)                         not null comment '最后一次修改者Id'
    )
    comment '资源主体';

create index i_tenant_app_kind
    on iam_resource_subject (rel_tenant_id, rel_app_id, kind);

create table if not exists iam_role
(
    id            varchar(64)
    primary key,
    code          varchar(255)                        not null comment '角色编码',
    name          varchar(255)                        not null comment '角色名称',
    sort          int                                 not null comment '显示排序，asc',
    rel_app_id    varchar(64)                         not null comment '关联应用Id',
    rel_tenant_id varchar(64)                         not null comment '关联租户Id',
    create_time   timestamp default CURRENT_TIMESTAMP null comment '创建时间',
    create_user   varchar(64)                         not null comment '创建者Id',
    update_time   timestamp default CURRENT_TIMESTAMP null on update CURRENT_TIMESTAMP comment '最后一次修改时间',
    update_user   varchar(64)                         not null comment '最后一次修改者Id',
    constraint u_tenant_app_id
    unique (rel_tenant_id, rel_app_id, code)
    )
    comment '角色';

create table if not exists iam_tenant
(
    id                     varchar(64)
    primary key,
    name                   varchar(255)                        not null comment '租户名称',
    icon                   varchar(1000)                       not null comment '租户图标',
    parameters             varchar(5000)                       not null comment '租户扩展信息，Json格式',
    allow_account_register tinyint(1)                          not null comment '是否开放账号注册',
    status                 varchar(50)                         not null comment '租户状态',
    create_time            timestamp default CURRENT_TIMESTAMP null comment '创建时间',
    create_user            varchar(64)                         not null comment '创建者Id',
    update_time            timestamp default CURRENT_TIMESTAMP null on update CURRENT_TIMESTAMP comment '最后一次修改时间',
    update_user            varchar(64)                         not null comment '最后一次修改者Id'
    )
    comment '租户';

create index u_status
    on iam_tenant (status);

create table if not exists iam_tenant_ident
(
    id                 varchar(64)
    primary key,
    kind               varchar(100)                        not null comment '租户认证类型名称',
    valid_ak_rule      varchar(2000)                       not null comment '认证AK校验正则规则',
    valid_ak_rule_note varchar(2000)                       not null comment '认证AK校验正则规则说明',
    valid_sk_rule      varchar(2000)                       not null comment '认证SK校验正则规则',
    valid_sk_rule_note varchar(2000)                       not null comment '认证SK校验正则规则说明',
    valid_time         bigint                              not null comment '认证有效时间（秒）',
    rel_tenant_id      varchar(64)                         not null comment '关联租户Id',
    create_time        timestamp default CURRENT_TIMESTAMP null comment '创建时间',
    create_user        varchar(64)                         not null comment '创建者Id',
    update_time        timestamp default CURRENT_TIMESTAMP null on update CURRENT_TIMESTAMP comment '最后一次修改时间',
    update_user        varchar(64)                         not null comment '最后一次修改者Id',
    constraint u_tenant_kind
    unique (rel_tenant_id, kind)
    )
    comment '租户认证配置';

create table if not exists iam_tenant_cert
(
    id            varchar(64)
    primary key,
    category      varchar(255)                        not null comment '凭证类型名称',
    version       tinyint                             not null comment '凭证保留的版本数量',
    rel_tenant_id varchar(64)                         not null comment '关联租户Id',
    create_time   timestamp default CURRENT_TIMESTAMP null comment '创建时间',
    create_user   varchar(64)                         not null comment '创建者Id',
    update_time   timestamp default CURRENT_TIMESTAMP null on update CURRENT_TIMESTAMP comment '最后一次修改时间',
    update_user   varchar(64)                         not null comment '最后一次修改者Id',
    constraint u_tenant_category
    unique (rel_tenant_id, category)
    )
    comment '租户凭证配置';


