use std::collections::HashMap;
use std::default::Default;

use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
#[cfg(feature = "default")]
use tardis::web::poem_openapi;

use crate::rbum::rbum_enumeration::{RbumCertConfStatusKind, RbumCertRelKind, RbumCertStatusKind, RbumRelFromKind, RbumScopeLevelKind, RbumSetCateLevelQueryKind};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
#[serde(default)]
pub struct RbumBasicFilterReq {
    pub ignore_scope: bool,
    pub rel_ctx_owner: bool,

    pub own_paths: Option<String>,
    pub with_sub_own_paths: bool,
    pub ids: Option<Vec<String>>,
    pub scope_level: Option<RbumScopeLevelKind>,
    pub enabled: Option<bool>,
    pub name: Option<String>,
    pub names: Option<Vec<String>>,
    pub code: Option<String>,
    pub codes: Option<Vec<String>>,
    pub rbum_kind_id: Option<String>,
    pub rbum_domain_id: Option<String>,

    pub desc_by_sort: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
#[serde(default)]
pub struct RbumCertConfFilterReq {
    pub basic: RbumBasicFilterReq,
    pub kind: Option<TrimString>,
    pub supplier: Option<String>,
    pub status: Option<RbumCertConfStatusKind>,
    pub rel_rbum_domain_id: Option<String>,
    pub rel_rbum_item_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
#[serde(default)]
pub struct RbumCertFilterReq {
    pub basic: RbumBasicFilterReq,
    pub id: Option<String>,
    pub ak: Option<String>,
    /// ak like "ak%"
    pub ak_like: Option<String>,
    pub kind: Option<String>,
    pub suppliers: Option<Vec<String>>,
    pub status: Option<RbumCertStatusKind>,
    pub rel: Option<RbumItemRelFilterReq>,
    pub rel_rbum_kind: Option<RbumCertRelKind>,
    pub rel_rbum_id: Option<String>,
    pub rel_rbum_ids: Option<Vec<String>>,
    pub rel_rbum_cert_conf_ids: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
#[serde(default)]
pub struct RbumKindFilterReq {
    pub basic: RbumBasicFilterReq,
    pub module: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
#[serde(default)]
pub struct RbumKindAttrFilterReq {
    pub basic: RbumBasicFilterReq,
    pub secret: Option<bool>,
    pub parent_attr_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
#[serde(default)]
pub struct RbumItemAttrFilterReq {
    pub basic: RbumBasicFilterReq,
    pub rel_rbum_item_id: Option<String>,
    pub rel_rbum_kind_attr_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
#[serde(default)]
pub struct RbumRelFilterReq {
    pub basic: RbumBasicFilterReq,
    pub tag: Option<String>,
    pub from_rbum_kind: Option<RbumRelFromKind>,
    pub from_rbum_id: Option<String>,
    pub from_rbum_scope_levels: Option<Vec<i16>>,
    pub to_rbum_item_id: Option<String>,
    pub to_rbum_item_scope_levels: Option<Vec<i16>>,
    pub to_own_paths: Option<String>,
    pub ext_eq: Option<String>,
    pub ext_like: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
#[serde(default)]
pub struct RbumRelExtFilterReq {
    pub basic: RbumBasicFilterReq,
    pub rel_rbum_rel_id: Option<String>,
}

/// Resource set filter
///
/// 资源集过滤器
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct RbumSetFilterReq {
    /// Basic filter
    ///
    /// 基础过滤
    pub basic: RbumBasicFilterReq,
    /// Resource relation filter
    ///
    /// 资源关联过滤
    pub rel: Option<RbumItemRelFilterReq>,
    /// Include resource kind id
    ///
    /// 包含的资源类型id
    pub kind: Option<String>,
}

/// Resource set category(node) filter
///
/// 资源集分类（节点）过滤器
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct RbumSetCateFilterReq {
    /// Basic filter
    ///
    /// 基础过滤
    pub basic: RbumBasicFilterReq,
    /// Resource relation filter
    ///
    /// 资源关联过滤
    pub rel: Option<RbumItemRelFilterReq>,
    /// Include resource set id
    ///
    /// 包含的资源集id
    pub rel_rbum_set_id: Option<String>,
    /// Include resource category (node) sys_codes
    ///
    /// 包含的资源分类（节点）sys_code 列表
    pub sys_codes: Option<Vec<String>>,
    /// Resource set category(node) query kind
    ///
    /// 资源集分类（节点）的查询类型
    ///
    /// Only valid when ``sys_codes`` exists.
    ///
    /// 仅当 ``sys_codes`` 存在时有效。
    pub sys_code_query_kind: Option<RbumSetCateLevelQueryKind>,
    /// Resource set category(node) query depth
    ///
    /// 资源集分类（节点）查询深度
    ///
    /// Only valid when ``sys_codes`` exists and ``sys_code_query_kind = CurrentAndSub or Sub``.
    ///
    /// 仅当 ``sys_codes`` 存在并且 ``sys_code_query_kind = CurrentAndSub or Sub`` 时有效。
    pub sys_code_query_depth: Option<i16>,
    /// Include resource category (node) extension information
    ///
    /// 包含的资源分类（节点）扩展信息
    pub cate_exts: Option<Vec<String>>,
}

/// Resource set category(node) mount resource item filter
///
/// 资源集分类（节点）挂载资源项的过滤器
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct RbumSetItemFilterReq {
    /// Basic filter
    ///
    /// 基础过滤
    pub basic: RbumBasicFilterReq,
    /// Include resource set id
    ///
    /// 包含的资源集id
    pub rel_rbum_set_id: Option<String>,
    /// Resource set category(node) query kind
    ///
    /// 资源集分类（节点）的查询类型
    ///
    /// Only valid when ``sys_codes`` exists.
    ///
    /// 仅当 ``sys_codes`` 存在时有效。
    pub sys_code_query_kind: Option<RbumSetCateLevelQueryKind>,
    /// Resource set category(node) query depth
    ///
    /// 资源集分类（节点）查询深度
    ///
    /// Only valid when ``sys_codes`` exists and ``sys_code_query_kind = CurrentAndSub or Sub``.
    ///
    /// 仅当 ``sys_codes`` 存在并且 ``sys_code_query_kind = CurrentAndSub or Sub`` 时有效。
    pub sys_code_query_depth: Option<i16>,
    /// Include resource category (node) sys_codes
    ///
    /// 包含的资源分类（节点）sys_code 列表
    pub rel_rbum_set_cate_sys_codes: Option<Vec<String>>,
    /// Include resource category (node) ids
    ///
    /// 包含的资源分类（节点）id 列表
    pub rel_rbum_set_cate_ids: Option<Vec<String>>,
    /// Include resource category (node) code
    ///
    /// 包含的资源分类（节点）code
    pub rel_rbum_set_item_cate_code: Option<String>,
    /// Whether the associated resource item can not exist
    ///
    /// 关联的资源项是否可以不存在
    ///
    /// Default is ``true``
    ///
    /// 默认为 ``true``
    pub rel_rbum_item_can_not_exist: Option<bool>,
    /// Include the associated resource item ids
    ///
    /// 包含关联的资源项id列表
    pub rel_rbum_item_ids: Option<Vec<String>>,
    /// Include the associated resource item scope level
    ///
    /// 包含关联的资源项作用域级别
    pub rel_rbum_item_scope_level: Option<RbumScopeLevelKind>,
    /// Include the associated resource item kind ids
    ///
    /// 包含关联的资源项类型id列表
    pub rel_rbum_item_kind_ids: Option<Vec<String>>,
    /// Include the associated resource item domain ids
    ///
    /// 包含关联的资源项域id列表
    pub rel_rbum_item_domain_ids: Option<Vec<String>>,
    /// Whether the associated resource item is disabled
    ///
    /// 关联的资源项是否已禁用
    pub rel_rbum_item_disabled: Option<bool>,
}

/// Resource set filter
///
/// 资源集过滤器
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct RbumSetTreeFilterReq {
    /// Whether to get the associated resource items
    ///
    /// 是否获取关联的资源项
    pub fetch_cate_item: bool,
    /// Whether to not get the associated resource items and the disabled ones
    ///
    /// 是否不获取包含关联的且已禁用的资源项
    ///
    /// Only valid when ``fetch_cate_item = true``.
    ///
    /// 仅当 ``fetch_cate_item = true`` 时有效。
    pub hide_item_with_disabled: bool,
    /// Whether to filter out nodes that do not have associated resource items
    ///
    /// 返回的树是否过滤掉没有关联资源项的节点
    ///
    /// Only valid when ``fetch_cate_item = true``.
    ///
    /// 仅当 ``fetch_cate_item = true`` 时有效。
    pub hide_cate_with_empty_item: bool,
    /// Include resource category (node) sys_codes
    ///
    /// 包含的资源分类（节点）sys_code 列表
    pub sys_codes: Option<Vec<String>>,
    /// Resource set category(node) query kind
    ///
    /// 资源集分类（节点）的查询类型
    ///
    /// Only valid when ``sys_codes`` exists.
    ///
    /// 仅当 ``sys_codes`` 存在时有效。
    pub sys_code_query_kind: Option<RbumSetCateLevelQueryKind>,
    /// Resource set category(node) query depth
    ///
    /// 资源集分类（节点）查询深度
    ///
    /// Only valid when ``sys_codes`` exists and ``sys_code_query_kind = CurrentAndSub or Sub``.
    ///
    /// 仅当 ``sys_codes`` 存在并且 ``sys_code_query_kind = CurrentAndSub or Sub`` 时有效。
    pub sys_code_query_depth: Option<i16>,
    /// Include resource category (node) extension information
    ///
    /// 包含的资源分类（节点）扩展信息
    pub cate_exts: Option<Vec<String>>,
    /// Include the associated resource item ids
    ///
    /// 包含关联的资源项id列表
    pub rel_rbum_item_ids: Option<Vec<String>>,
    /// Include the associated resource item kind ids
    ///
    /// 包含关联的资源项类型id列表
    pub rel_rbum_item_kind_ids: Option<Vec<String>>,
    /// Include the associated resource item domain ids
    ///
    /// 包含关联的资源项域id列表
    pub rel_rbum_item_domain_ids: Option<Vec<String>>,
}

/// Resource relation filter
///
/// 资源关联过滤器
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
#[serde(default)]
pub struct RbumItemRelFilterReq {
    pub rel_by_from: bool,
    pub is_left: bool,
    pub tag: Option<String>,
    pub from_rbum_kind: Option<RbumRelFromKind>,
    pub rel_item_id: Option<String>,
    pub rel_item_ids: Option<Vec<String>>,
    pub ext_eq: Option<String>,
    pub ext_like: Option<String>,
    pub own_paths: Option<String>,
}

pub trait RbumItemFilterFetcher {
    fn basic(&self) -> &RbumBasicFilterReq;
    fn rel(&self) -> &Option<RbumItemRelFilterReq>;
    fn rel2(&self) -> &Option<RbumItemRelFilterReq>;
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct RbumSetItemRelFilterReq {
    //同时根据set_id cate_code 二元组限制
    pub set_ids_and_cate_codes: Option<HashMap<String, Vec<String>>>,
    pub with_sub_set_cate_codes: bool,
    pub rel_item_ids: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
#[serde(default)]
pub struct RbumItemBasicFilterReq {
    pub basic: RbumBasicFilterReq,
    pub rel: Option<RbumItemRelFilterReq>,
}

impl RbumItemFilterFetcher for RbumItemBasicFilterReq {
    fn basic(&self) -> &RbumBasicFilterReq {
        &self.basic
    }
    fn rel(&self) -> &Option<RbumItemRelFilterReq> {
        &self.rel
    }
    fn rel2(&self) -> &Option<RbumItemRelFilterReq> {
        &self.rel
    }
}
