use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::chrono::{DateTime, Utc};

use tardis::db::sea_orm;

use tardis::web::poem_openapi;

use crate::rbum::rbum_enumeration::RbumScopeLevelKind;

/// Add request for resource set category(node)
///
/// 资源集分类（节点）添加请求
#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct RbumSetCateAddReq {
    /// Business code for custom
    ///
    /// 自定义业务编码
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub bus_code: TrimString,
    /// Node name
    ///
    /// 节点名称
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: TrimString,
    /// Node icon
    ///
    /// 节点图标
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub icon: Option<String>,
    /// Node sort
    ///
    /// 节点排序
    pub sort: Option<i64>,
    /// Node extension information
    ///
    /// 节点扩展信息
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub ext: Option<String>,
    /// Associated [resource set](crate::rbum::dto::rbum_set_dto::RbumSetDetailResp) id
    ///
    /// 关联[资源集](crate::rbum::dto::rbum_set_dto::RbumSetDetailResp) id
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub rel_rbum_set_id: String,
    /// Parent node id
    ///
    /// 父节点id
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub rbum_parent_cate_id: Option<String>,

    pub scope_level: Option<RbumScopeLevelKind>,
}

/// Modify request for resource set category(node)
///
/// 资源集分类（节点）修改请求
#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct RbumSetCateModifyReq {
    /// Business code for custom
    ///
    /// 自定义业务编码
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub bus_code: Option<TrimString>,
    /// Node name
    ///
    /// 节点名称
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: Option<TrimString>,
    /// Node icon
    ///
    /// 节点图标
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub icon: Option<String>,
    /// Node sort
    ///
    /// 节点排序
    pub sort: Option<i64>,
    /// Node extension information
    ///
    /// 节点扩展信息
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub ext: Option<String>,

    /// Parent node id
    ///
    /// 父节点id
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub rbum_parent_cate_id: Option<String>,

    pub scope_level: Option<RbumScopeLevelKind>,
}

/// Resource set category(node) summary information
///
/// 资源集分类（节点）概要信息
#[derive(Serialize, Deserialize, Debug, poem_openapi::Object, sea_orm::FromQueryResult)]
pub struct RbumSetCateSummaryResp {
    /// Node id
    ///
    /// 节点id
    pub id: String,
    /// System (internal) code
    ///
    /// 系统（内部）编码
    ///
    /// using regular hierarchical code to avoid recursive tree queries.
    ///
    /// 使用规则的层级编码，避免递归树查询。
    pub sys_code: String,
    /// Business code for custom
    ///
    /// 自定义业务编码
    pub bus_code: String,
    /// Node name
    ///
    /// 节点名称
    pub name: String,
    /// Node icon
    ///
    /// 节点图标
    pub icon: String,
    /// Node sort
    ///
    /// 节点排序
    pub sort: i64,
    /// Node extension information
    ///
    /// 节点扩展信息
    pub ext: String,
    /// Associated [resource set](crate::rbum::dto::rbum_set_dto::RbumSetSummaryResp) id
    ///
    /// 关联[资源集](crate::rbum::dto::rbum_set_dto::RbumSetSummaryResp) id
    pub rel_rbum_set_id: String,

    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
}

/// Resource set category(node) detail information
///
/// 资源集分类（节点）详细信息
#[derive(Serialize, Deserialize, Debug, poem_openapi::Object, sea_orm::FromQueryResult)]
pub struct RbumSetCateDetailResp {
    /// Node id
    ///
    /// 节点id
    pub id: String,
    /// System (internal) code
    ///
    /// 系统（内部）编码
    ///
    /// using regular hierarchical code to avoid recursive tree queries.
    ///
    /// 使用规则的层级编码，避免递归树查询。
    pub sys_code: String,
    /// Business code for custom
    ///
    /// 自定义业务编码
    pub bus_code: String,
    /// Node name
    ///
    /// 节点名称
    pub name: String,
    /// Node icon
    ///
    /// 节点图标
    pub icon: String,
    /// Node sort
    ///
    /// 节点排序
    pub sort: i64,
    /// Node extension information
    ///
    /// 节点扩展信息
    pub ext: String,
    /// Associated [resource set](crate::rbum::dto::rbum_set_dto::RbumSetDetailResp) id
    ///
    /// 关联[资源集](crate::rbum::dto::rbum_set_dto::RbumSetDetailResp) id
    pub rel_rbum_set_id: String,

    pub own_paths: String,
    pub owner: String,
    pub owner_name: Option<String>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
}
