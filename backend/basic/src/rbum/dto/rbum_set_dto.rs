use crate::rbum::dto::rbum_domain_dto::RbumDomainSummaryResp;
use crate::rbum::dto::rbum_kind_dto::RbumKindSummaryResp;
use crate::rbum::dto::rbum_set_item_dto::RbumSetItemRelInfoResp;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tardis::basic::field::TrimString;
use tardis::chrono::{DateTime, Utc};
#[cfg(feature = "default")]
use tardis::db::sea_orm;
#[cfg(feature = "default")]
use tardis::web::poem_openapi;

use crate::rbum::rbum_enumeration::RbumScopeLevelKind;

/// Add request for resource set
///
/// 资源集添加请求
#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct RbumSetAddReq {
    /// Resource set code
    ///
    /// 资源集编码
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub code: TrimString,
    /// Resource set kind
    ///
    /// 资源集类型
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub kind: TrimString,
    /// Resource set name
    ///
    /// 资源集名称
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub name: TrimString,
    /// Resource set note
    ///
    /// 资源集备注
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "2000")))]
    pub note: Option<String>,
    /// Resource set icon
    ///
    /// 资源集图标
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "1000")))]
    pub icon: Option<String>,
    /// Resource set sort
    ///
    /// 资源集排序
    pub sort: Option<i64>,
    /// Resource set extension information
    ///
    /// 资源集扩展信息
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "1000")))]
    pub ext: Option<String>,

    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,
}

/// Modify request for resource set
///
/// 资源集修改请求
#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct RbumSetModifyReq {
    /// Resource set name
    ///
    /// 资源集名称
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub name: Option<TrimString>,
    /// Resource set note
    ///
    /// 资源集备注
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "2000")))]
    pub note: Option<String>,
    /// Resource set icon
    ///
    /// 资源集图标
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "1000")))]
    pub icon: Option<String>,
    /// Resource set sort
    ///
    /// 资源集排序
    pub sort: Option<i64>,
    /// Resource set extension information
    ///
    /// 资源集扩展信息
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "1000")))]
    pub ext: Option<String>,

    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,
}

/// Resource set summary information
///
/// 资源集概要信息
#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object, sea_orm::FromQueryResult))]
pub struct RbumSetSummaryResp {
    /// Resource set id
    ///
    /// 资源集id
    pub id: String,
    /// Resource set code
    ///
    /// 资源集编码
    pub code: String,
    /// Resource set kind
    ///
    /// 资源集类型
    pub kind: String,
    /// Resource set name
    ///
    /// 资源集名称
    pub name: String,
    /// Resource set icon
    ///
    /// 资源集图标
    pub icon: String,
    /// Resource set sort
    ///
    /// 资源集排序
    pub sort: i64,
    /// Resource set extension information
    ///
    /// 资源集扩展信息
    pub ext: String,

    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
    pub disabled: bool,
}

/// Resource set detail information
///
/// 资源集详细信息
#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object, sea_orm::FromQueryResult))]
pub struct RbumSetDetailResp {
    /// Resource set id
    ///
    /// 资源集id
    pub id: String,
    /// Resource set code
    ///
    /// 资源集编码
    pub code: String,
    /// Resource set kind
    ///
    /// 资源集类型
    pub kind: String,
    /// Resource set name
    ///
    /// 资源集名称
    pub name: String,
    /// Resource set note
    ///
    /// 资源集备注
    pub note: String,
    /// Resource set icon
    ///
    /// 资源集图标
    pub icon: String,
    /// Resource set sort
    ///
    /// 资源集排序
    pub sort: i64,
    /// Resource set extension information
    ///
    /// 资源集扩展信息
    pub ext: String,

    pub own_paths: String,
    pub owner: String,
    pub owner_name: Option<String>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
    pub disabled: bool,
}

/// Resource set path information
///
/// 资源集路径信息
#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object, sea_orm::FromQueryResult))]
pub struct RbumSetPathResp {
    /// Node id
    ///
    /// 节点id
    pub id: String,
    /// Node name
    ///
    /// 节点名称
    pub name: String,

    pub own_paths: String,
}

/// Resource tree information
///
/// 资源树信息
#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct RbumSetTreeResp {
    /// Resource tree node information
    ///
    /// 资源树节点信息
    pub main: Vec<RbumSetTreeNodeResp>,
    /// Resource tree extension information
    ///
    /// 资源树扩展信息
    pub ext: Option<RbumSetTreeExtResp>,
}

impl RbumSetTreeResp {
    pub fn to_trees(&self) -> RbumSetTreeCateResp {
        let mut nodes_map: HashMap<String, RbumSetTreeCateNodeResp> = HashMap::new();
        let mut child_map: HashMap<String, Vec<String>> = HashMap::new();
        let mut roots: Vec<String> = Vec::new();

        // 初始化所有节点
        for node in &self.main {
            nodes_map.insert(
                node.id.clone(),
                RbumSetTreeCateNodeResp {
                    id: node.id.clone(),
                    sys_code: node.sys_code.clone(),
                    bus_code: node.bus_code.clone(),
                    name: node.name.clone(),
                    icon: node.icon.clone(),
                    sort: node.sort,
                    ext: node.ext.clone(),
                    node: vec![],
                    own_paths: node.own_paths.clone(),
                    owner: node.owner.clone(),
                    create_time: node.create_time,
                    update_time: node.update_time,
                    scope_level: node.scope_level.clone(),
                },
            );
            if let Some(parent_id) = node.pid.as_ref() {
                child_map.entry(parent_id.to_string()).or_default().push(node.id.clone());
            } else {
                roots.push(node.id.clone());
            }
        }
        // 递归方法构建树
        fn build_tree(node_id: String, nodes_map: &mut HashMap<String, RbumSetTreeCateNodeResp>, child_map: &HashMap<String, Vec<String>>) -> Option<RbumSetTreeCateNodeResp> {
            if let Some(mut node) = nodes_map.remove(&node_id) {
                if let Some(children) = child_map.get(&node_id) {
                    for child_id in children.clone() {
                        let child_node = build_tree(child_id, nodes_map, child_map);
                        if let Some(child_node) = child_node {
                            node.node.push(child_node);
                        }
                    }
                }
                Some(node)
            } else {
                None
            }
        }
        // 构建根节点的树结构
        let mut trees: Vec<RbumSetTreeCateNodeResp> = Vec::new();
        for root_id in roots {
            if let Some(root_node) = build_tree(root_id, &mut nodes_map, &child_map) {
                trees.push(root_node);
            }
        }
        RbumSetTreeCateResp {
            cate_tree: trees,
            ext: self.ext.clone(),
        }
    }
}

/// Resource tree node information
///
/// 资源树节点信息
#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct RbumSetTreeNodeResp {
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
    /// Parent node id
    ///
    /// 父节点Id
    pub pid: Option<String>,
    /// Associated object id
    ///
    /// 关联对象Id
    ///
    /// This association is set by the business layer, and the rbum model will not assign a value to it.
    ///
    /// 此关联由上层的业务设置，rbum模型不会为其赋值。
    pub rel: Option<String>,

    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
}

/// Resource tree extension information
///
/// 资源树扩展信息
#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct RbumSetTreeExtResp {
    /// 节点与资源项的关联信息
    ///
    /// Node and resource item association information
    ///
    /// Format: ``node.id -> resource items``
    pub items: HashMap<String, Vec<RbumSetItemRelInfoResp>>,
    /// 节点关联资源项统计信息
    ///
    /// Node associated resource item statistics information
    ///
    /// Format: ``node.id -> [`crate::rbum::dto::rbum_set_item_dto::RbumSetItemInfoResp::rel_rbum_item_kind_id`] ->  resource item number``
    pub item_number_agg: HashMap<String, HashMap<String, u64>>,
    /// Resource kind information
    ///
    /// 资源类型信息
    ///
    /// Format: ``kind.id -> kind summary information``
    pub item_kinds: HashMap<String, RbumKindSummaryResp>,
    /// Resource domain information
    ///
    /// 资源域信息
    ///
    /// Format: ``domain.id -> domain summary info``
    pub item_domains: HashMap<String, RbumDomainSummaryResp>,
}

/// Resource tree information
///
/// 资源目录树信息
#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct RbumSetTreeCateResp {
    /// Resource cate tree node information
    ///
    /// 资源目录树节点信息
    pub cate_tree: Vec<RbumSetTreeCateNodeResp>,
    /// Resource tree extension information
    ///
    /// 资源树扩展信息
    pub ext: Option<RbumSetTreeExtResp>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
/// Resource tree cate structure information
///
/// 资源树目录结构信息
pub struct RbumSetTreeCateNodeResp {
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

    pub node: Vec<RbumSetTreeCateNodeResp>,

    pub own_paths: String,

    pub owner: String,

    pub create_time: DateTime<Utc>,

    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
}
