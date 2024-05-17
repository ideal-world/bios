use std::collections::HashMap;
use std::default::Default;

use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
#[cfg(feature = "default")]
use tardis::web::poem_openapi;

use crate::rbum::rbum_enumeration::{RbumCertConfStatusKind, RbumCertRelKind, RbumCertStatusKind, RbumRelFromKind, RbumScopeLevelKind, RbumSetCateLevelQueryKind};

/// Resource basic filter
///
/// 资源基础过滤器
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
#[serde(default)]
pub struct RbumBasicFilterReq {
    /// Whether to ignore the scope
    ///
    /// 是否忽略作用域
    ///
    /// If ``true``, it means that only the ownership path is related, and the scope is not considered.
    ///
    /// 为 ``true`` 时表示只与所有权路径有关，不考虑作用域。
    pub ignore_scope: bool,
    /// Scope
    ///
    /// 作用域
    ///
    /// Only valid when ``ignore_scope = true``.
    ///
    /// 仅当 ``ignore_scope = true`` 有效。
    pub scope_level: Option<RbumScopeLevelKind>,
    /// Whether to include sub-ownership paths
    ///
    /// 是否包含子所有权路径
    pub with_sub_own_paths: bool,
    /// Whether to include the owner of the context
    ///
    /// 是否包含上下文所有者
    pub rel_ctx_owner: bool,
    /// Ownership path
    ///
    /// 所有权路径
    pub own_paths: Option<String>,
    /// Object id set
    ///
    /// 对象id集合
    pub ids: Option<Vec<String>>,
    /// Object name
    ///
    /// 对象名称
    pub name: Option<String>,
    /// Object names
    ///
    /// 对象名称集合
    pub names: Option<Vec<String>>,
    /// Object code
    ///
    /// 对象编码
    pub code: Option<String>,
    /// Object codes
    ///
    /// 对象编码集合
    pub codes: Option<Vec<String>>,
    /// Whether to include only enabled objects
    ///
    /// 是否仅包含启用的对象
    pub enabled: Option<bool>,
    /// Resource kind id
    ///
    /// 资源类型id
    pub rbum_kind_id: Option<String>,
    /// Resource domain id
    ///
    /// 资源域id
    pub rbum_domain_id: Option<String>,
}

/// Resource certificate configuration filter
///
/// 资源凭证配置过滤器
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
#[serde(default)]
pub struct RbumCertConfFilterReq {
    /// Basic filter
    ///
    /// 基础过滤
    pub basic: RbumBasicFilterReq,
    /// Certificate configuration type
    ///
    /// 凭证配置类型
    pub kind: Option<TrimString>,
    /// Certificate configuration supplier
    ///
    /// 凭证配置供应商
    pub supplier: Option<String>,
    /// Certificate configuration status
    ///
    /// 凭证配置状态
    pub status: Option<RbumCertConfStatusKind>,
    /// Associated resource domain id
    ///
    /// 关联的资源域id
    pub rel_rbum_domain_id: Option<String>,
    /// Associated resource item id
    ///
    /// 关联的资源项id
    pub rel_rbum_item_id: Option<String>,
}

/// Resource certificate filter
///
/// 资源凭证过滤器
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
#[serde(default)]
pub struct RbumCertFilterReq {
    /// Basic filter
    ///
    /// 基础过滤
    pub basic: RbumBasicFilterReq,
    /// Certificate id
    ///
    /// 凭证id
    pub id: Option<String>,
    /// Certificate ak
    ///
    /// 凭证ak
    pub ak: Option<String>,
    /// Certificate ak(left like)
    ///
    /// 凭证id（左包含）
    pub ak_like: Option<String>,
    /// Certificate type
    ///
    /// 凭证类型
    pub kind: Option<String>,
    /// Certificate supplier
    ///
    /// 凭证供应商
    pub suppliers: Option<Vec<String>>,
    /// Certificate status
    ///
    /// 凭证状态
    pub status: Option<RbumCertStatusKind>,
    /// Certificate extension information
    ///
    /// 凭证扩展信息
    pub ext: Option<String>,
    /// Association type
    ///
    /// 关联类型
    pub rel: Option<RbumItemRelFilterReq>,
    /// Associated resource kind id
    ///
    /// 关联的资源类型id
    pub rel_rbum_kind: Option<RbumCertRelKind>,
    /// Associated object id
    ///
    /// 关联的对象id
    pub rel_rbum_id: Option<String>,
    /// Associated object id set
    ///
    /// 关联的对象id集合
    pub rel_rbum_ids: Option<Vec<String>>,
    /// Associated resource certificate configuration id set
    ///
    /// 关联的凭证配置id集合
    pub rel_rbum_cert_conf_ids: Option<Vec<String>>,
}

/// Resource kind filter
///
/// 资源类型过滤器
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
#[serde(default)]
pub struct RbumKindFilterReq {
    /// Basic filter
    ///
    /// 基础过滤
    pub basic: RbumBasicFilterReq,
    /// Module
    ///
    /// 模块
    pub module: Option<String>,
}

/// Resource kind attribute filter
///
/// 资源类型属性过滤器
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
#[serde(default)]
pub struct RbumKindAttrFilterReq {
    /// Basic filter
    ///
    /// 基础过滤
    pub basic: RbumBasicFilterReq,
    /// Whether it is a secret
    ///
    /// 是否是秘密
    pub secret: Option<bool>,
    /// Parent attribute name
    ///
    /// 父属性名称
    pub parent_attr_name: Option<String>,
}

/// Resource item attribute filter
///
/// 资源项属性过滤器
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
#[serde(default)]
pub struct RbumItemAttrFilterReq {
    /// Basic filter
    ///
    /// 基础过滤
    pub basic: RbumBasicFilterReq,
    /// Associated resource item id
    ///
    /// 关联的资源项id
    pub rel_rbum_item_id: Option<String>,
    /// Associated resource kind attribute id
    ///
    /// 关联的资源类型属性id
    pub rel_rbum_kind_attr_id: Option<String>,
}

/// Resource relation filter
///
/// 资源关联过滤器
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
#[serde(default)]
pub struct RbumRelFilterReq {
    /// Basic filter
    ///
    /// 基础过滤
    pub basic: RbumBasicFilterReq,
    /// Associated tag
    ///
    /// 关联的标签
    pub tag: Option<String>,
    /// ``from`` party kind
    ///
    /// ``from`` 方的类型
    pub from_rbum_kind: Option<RbumRelFromKind>,
    /// ``from`` party object id
    ///
    /// ``from`` 方的对象id
    pub from_rbum_id: Option<String>,
    /// ``from`` party scope levels
    ///
    /// ``from`` 方的作用域集合
    pub from_rbum_scope_levels: Option<Vec<i16>>,
    /// ``to`` party object id
    ///
    /// ``to`` 方的对象id
    pub to_rbum_item_id: Option<String>,
    /// ``to`` party scope levels
    ///
    /// ``to`` 方的作用域集合
    pub to_rbum_item_scope_levels: Option<Vec<i16>>,
    /// ``to`` party ownership
    ///
    /// ``to`` 方的所有权
    pub to_own_paths: Option<String>,
    /// Extension information(equal)
    ///
    /// 扩展信息（相等）
    pub ext_eq: Option<String>,
    /// Extension information(like)
    ///
    /// 扩展信息（包含）
    pub ext_like: Option<String>,
}

/// Resource relation extension filter
///
/// 资源关联扩展过滤器
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
#[serde(default)]
pub struct RbumRelExtFilterReq {
    /// Basic filter
    ///
    /// 基础过滤
    pub basic: RbumBasicFilterReq,
    /// Associated resource relation id
    ///
    /// 关联的资源关联id
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
    pub rel: Option<RbumItemRelSimpleFilterReq>,
    /// Resource kind id
    ///
    /// 资源类型id
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
    pub rel: Option<RbumItemRelSimpleFilterReq>,
    /// Resource set id
    ///
    /// 资源集id
    pub rel_rbum_set_id: Option<String>,
    /// Resource category (node) sys_codes
    ///
    /// 资源分类（节点）sys_code 列表
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
    /// Resource category (node) extension information
    ///
    /// 资源分类（节点）扩展信息
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
    /// Resource set id
    ///
    /// 资源集id
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
    /// Resource category (node) sys_codes
    ///
    /// 资源分类（节点）sys_code 列表
    pub rel_rbum_set_cate_sys_codes: Option<Vec<String>>,
    /// Resource category (node) id set
    ///
    /// 资源分类（节点）id 列表
    pub rel_rbum_set_cate_ids: Option<Vec<String>>,
    /// Resource category (node) code
    ///
    /// 资源分类（节点）code
    pub rel_rbum_set_item_cate_code: Option<String>,
    /// Whether the associated resource item can not exist
    ///
    /// 关联的资源项是否可以不存在
    ///
    /// Default is ``true``
    ///
    /// 默认为 ``true``
    pub rel_rbum_item_can_not_exist: Option<bool>,
    /// Associated resource item id set
    /// 关联的资源项id列表
    pub rel_rbum_item_ids: Option<Vec<String>>,
    /// Associated resource item scope level
    ///
    /// 关联的资源项作用域级别
    pub rel_rbum_item_scope_level: Option<RbumScopeLevelKind>,
    /// Associated resource item kind id set
    ///
    /// 关联的资源项类型id列表
    pub rel_rbum_item_kind_ids: Option<Vec<String>>,
    /// Associated resource item domain id set
    ///
    /// 关联的资源项域id列表
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
    /// Resource category (node) sys_codes
    ///
    /// 资源分类（节点）sys_code 列表
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
    /// Resource category (node) extension information
    ///
    /// 资源分类（节点）扩展信息
    pub cate_exts: Option<Vec<String>>,
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
    /// Associated resource item id set
    ///
    /// 关联的资源项id列表
    ///
    /// Only valid when ``fetch_cate_item = true``.
    ///
    /// 仅当 ``fetch_cate_item = true`` 时有效。
    pub rel_rbum_item_ids: Option<Vec<String>>,
    /// Associated resource item kind id set
    ///
    /// 关联的资源项类型id列表
    ///
    /// Only valid when ``fetch_cate_item = true``.
    ///
    /// 仅当 ``fetch_cate_item = true`` 时有效。
    pub rel_rbum_item_kind_ids: Option<Vec<String>>,
    /// Associated resource item domain id set
    ///
    /// 关联的资源项域id列表
    ///
    /// Only valid when ``fetch_cate_item = true``.
    ///
    /// 仅当 ``fetch_cate_item = true`` 时有效。
    pub rel_rbum_item_domain_ids: Option<Vec<String>>,
}

/// Resource set category (node) associated filter for mounted resource items
///
/// 资源集分类（节点）挂载资源项的关联过滤器
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct RbumSetItemRelFilterReq {
    /// Resource set id and resource category (node) id set
    ///
    /// 资源集Id与资源分类（节点）Id集合
    ///
    /// There is an ``or`` relationship between different resource sets, and the Id set of each resource category (node) in the same resource set is an ``or`` relationship.
    ///
    /// 不同资源集之间为 ``or`` 关系，同一资源集的每个资源分类（节点）Id集合为 ``or`` 关系。
    pub set_ids_and_cate_codes: Option<HashMap<String, Vec<String>>>,
    /// Whether the resource category (node) is associated with descendants
    ///
    /// 资源分类（节点）是否包含子孙级
    pub with_sub_set_cate_codes: bool,
    /// Associated object id set
    ///
    /// 关联的对象id集合
    pub rel_item_ids: Option<Vec<String>>,
}

/// Simple Resource item relation filter
///
/// 简单的资源项关联过滤器
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
#[serde(default)]
pub struct RbumItemRelSimpleFilterReq {
    /// Whether the related party is a ``from`` party
    ///
    /// 关联方是否是 ``from`` 方
    pub rel_by_from: bool,
    /// Associated tag
    ///
    /// 关联的标签
    pub tag: Option<String>,
    /// ``from`` party kind
    ///
    /// ``from`` 方的类型
    pub from_rbum_kind: Option<RbumRelFromKind>,
    /// Associated object id
    ///
    /// 关联的对象id
    pub rel_item_id: Option<String>,
}

/// Resource item relation filter
///
/// 资源项关联过滤器
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
#[serde(default)]
pub struct RbumItemRelFilterReq {
    /// Is it optional
    ///
    /// 是否可选
    ///
    /// When it is ``true``, it means that records with empty associations can be returned (corresponding to left join),
    /// otherwise records that fully satisfy the association filtering (corresponding to inner join) are returned.
    ///
    /// 为 ``true`` 时表示可返回关联为空的记录（对应于left join），反之返回完全满足关联过滤的记录（对应于inner join）。
    pub optional: bool,
    /// Whether the related party is a ``from`` party
    ///
    /// 关联方是否是 ``from`` 方
    pub rel_by_from: bool,
    /// Associated tag
    ///
    /// 关联的标签
    pub tag: Option<String>,
    /// ``from`` party kind
    ///
    /// ``from`` 方的类型
    pub from_rbum_kind: Option<RbumRelFromKind>,
    /// Associated object id
    ///
    /// 关联的对象id
    pub rel_item_id: Option<String>,
    /// Associated object id set
    ///
    /// 关联的对象id集合
    pub rel_item_ids: Option<Vec<String>>,
    /// Extension information(equal)
    ///
    /// 扩展信息（相等）
    pub ext_eq: Option<String>,
    /// Extension information(like)
    ///
    /// 扩展信息（包含）
    pub ext_like: Option<String>,
    pub own_paths: Option<String>,
}

/// Resource item filter fetcher
///
/// 资源项过滤获取器
pub trait RbumItemFilterFetcher {
    /// Basic filter
    ///
    /// 基础过滤
    fn basic(&self) -> &RbumBasicFilterReq;
    /// Resource item relation filter 1
    ///
    /// 资源项关联过滤1
    fn rel(&self) -> &Option<RbumItemRelFilterReq>;
    /// Resource item relation filter 2
    ///
    /// 资源项关联过滤2
    fn rel2(&self) -> &Option<RbumItemRelFilterReq>;
}

/// Resource item basic filter
///
/// 资源项基础过滤器
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
#[serde(default)]
pub struct RbumItemBasicFilterReq {
    /// Basic filter
    ///
    /// 基础过滤
    pub basic: RbumBasicFilterReq,
    /// Resource item relation filter 1
    ///
    /// 资源项关联过滤1
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
