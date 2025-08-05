use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::chrono::{DateTime, Utc};

use tardis::db::sea_orm;

use tardis::web::poem_openapi;

use crate::rbum::rbum_enumeration::RbumScopeLevelKind;

/// Add request for resource kind
///
/// 资源类型添加请求
#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct RbumKindAddReq {
    /// Resource kind module
    ///
    /// 资源类型模块
    ///
    /// Default is ``empty``
    ///
    /// 默认为 ``空``
    ///
    /// Used to further divide the resource  kind. For example, there are multiple resource  kinds under the ``cmdb compute`` module, such as ``ecs, ec2, k8s``.
    ///
    /// 用于对资源类型做简单的分类。比如 ``cmdb计算`` 模块下可以有 ``ecs、ec2、k8s`` 等多个资源类型。
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub module: Option<String>,
    /// Resource kind code
    ///
    /// 资源类型编码
    ///
    /// Resource kind code, which is required to conform to the scheme specification in the uri, matching the regular: ``^[a-z0-9-.]+$`` .
    ///
    /// 资源类型编码，需要符合uri中的scheme规范，匹配正则：``^[a-z0-9-.]+$`` 。
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub code: TrimString,
    /// Resource kind name
    ///
    /// 资源类型名称
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: TrimString,
    /// Resource kind note
    ///
    /// 资源类型备注
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub note: Option<String>,
    /// Resource kind icon
    ///
    /// 资源类型图标
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub icon: Option<String>,
    /// Resource kind sort
    ///
    /// 资源类型排序
    pub sort: Option<i64>,
    /// Extension table name
    ///
    /// 扩展表名
    ///
    /// Each resource kind can specify an extension table for storing customized data.
    ///
    /// 每个资源类型可以指定一个扩展表用于存储自定义数据。
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub ext_table_name: Option<String>,
    /// Parent kind id
    ///
    /// 资源类型父id
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub parent_id: Option<String>,

    pub scope_level: Option<RbumScopeLevelKind>,
}

/// Modify request for resource kind
///
/// 资源类型修改请求
#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct RbumKindModifyReq {
    /// Resource kind module
    ///
    /// 资源类型模块
    ///
    /// Used to further divide the resource  kind. For example, there are multiple resource  kinds under the ``cmdb compute`` module, such as ``ecs, ec2, k8s``.
    ///
    /// 用于对资源类型做简单的分类。比如 ``cmdb计算`` 模块下可以有 ``ecs、ec2、k8s`` 等多个资源类型。
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub module: Option<String>,
    /// Resource kind name
    ///
    /// 资源类型名称
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: Option<TrimString>,
    /// Resource kind note
    ///
    /// 资源类型备注
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub note: Option<String>,
    /// Resource kind icon
    ///
    /// 资源类型图标
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub icon: Option<String>,
    /// Resource kind sort
    ///
    /// 资源类型排序
    pub sort: Option<i64>,
    /// Extension table name
    ///
    /// 扩展表名
    ///
    /// Each resource kind can specify an extension table for storing customized data.
    ///
    /// 每个资源类型可以指定一个扩展表用于存储自定义数据。
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub ext_table_name: Option<String>,
    /// Parent kind id
    ///
    /// 资源类型父id
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub parent_id: Option<String>,

    pub scope_level: Option<RbumScopeLevelKind>,
}

/// Resource kind summary information
///
/// 资源类型概要信息
#[derive(Serialize, Deserialize, Debug, Clone, poem_openapi::Object, sea_orm::FromQueryResult)]
pub struct RbumKindSummaryResp {
    /// Resource kind id
    ///
    /// 资源类型id
    pub id: String,
    /// Resource kind module
    ///
    /// 资源类型模块
    pub module: String,
    /// Resource kind code
    ///
    /// 资源类型编码
    pub code: String,
    /// Resource kind name
    ///
    /// 资源类型名称
    pub name: String,
    /// Resource kind icon
    ///
    /// 资源类型图标
    pub icon: String,
    /// Resource kind sort
    ///
    /// 资源类型排序
    pub sort: i64,
    /// Extension table name
    ///
    /// 扩展表名
    pub ext_table_name: String,
    /// Parent kind id
    ///
    /// 资源类型父id
    pub parent_id: String,

    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
}

/// Resource kind detail information
///
/// 资源类型详细信息
#[derive(Serialize, Deserialize, Debug, Clone, poem_openapi::Object, sea_orm::FromQueryResult)]
pub struct RbumKindDetailResp {
    /// Resource kind id
    ///
    /// 资源类型id
    pub id: String,
    /// Resource kind module
    ///
    /// 资源类型模块
    pub module: String,
    /// Resource kind code
    ///
    /// 资源类型编码
    pub code: String,
    /// Resource kind name
    ///
    /// 资源类型名称
    pub name: String,
    /// Resource kind note
    ///
    /// 资源类型备注
    pub note: String,
    /// Resource kind icon
    ///
    /// 资源类型图标
    pub icon: String,
    /// Resource kind sort
    ///
    /// 资源类型排序
    pub sort: i64,
    /// Extension table name
    ///
    /// 扩展表名
    pub ext_table_name: String,
    /// Parent kind id
    ///
    /// 资源类型父id
    pub parent_id: String,

    pub parent_name: Option<String>,

    pub own_paths: String,
    pub owner: String,
    pub owner_name: Option<String>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
}
