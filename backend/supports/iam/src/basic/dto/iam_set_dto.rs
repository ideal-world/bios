use std::collections::HashMap;

use bios_basic::rbum::dto::rbum_domain_dto::RbumDomainSummaryResp;
use bios_basic::rbum::dto::rbum_kind_dto::RbumKindSummaryResp;
use bios_basic::rbum::dto::rbum_set_dto::{RbumSetTreeNodeResp, RbumSetTreeResp};
use bios_basic::rbum::dto::rbum_set_item_dto::RbumSetItemRelInfoResp;
use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::web::poem_openapi;

use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamSetCateAddReq {
    // #[oai(validator(min_length = "1", max_length = "255"))]
    pub name: TrimString,
    pub scope_level: Option<RbumScopeLevelKind>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub bus_code: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub icon: Option<String>,
    pub sort: Option<i64>,
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub ext: Option<String>,
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub rbum_parent_cate_id: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamSetCateModifyReq {
    // #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: Option<TrimString>,
    pub scope_level: Option<RbumScopeLevelKind>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub bus_code: Option<TrimString>,
    pub icon: Option<String>,
    pub sort: Option<i64>,
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub ext: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct IamSetItemAggAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub set_cate_id: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamSetItemWithDefaultSetAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub set_cate_id: Option<String>,
    pub sort: i64,

    #[oai(validator(min_length = "2"))]
    pub rel_rbum_item_id: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamSetItemAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub set_id: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub set_cate_id: String,
    pub sort: i64,

    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub rel_rbum_item_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct IamSetTreeResp {
    pub main: Vec<RbumSetTreeNodeResp>,
    pub ext: Option<IamSetTreeExtResp>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct IamSetTreeExtResp {
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
    /// 资源项与数据权限的关联信息
    ///
    /// Resource item and data guard association information
    ///
    /// Format: ``item.id -> data guard items``
    pub item_data_guards: HashMap<String, Vec<RbumSetItemRelInfoResp>>,
}

impl From<RbumSetTreeResp> for IamSetTreeResp {
    fn from(value: RbumSetTreeResp) -> Self {
        let ext = if let Some(value_ext) = value.ext {
            Some(IamSetTreeExtResp {
                items: value_ext.items,
                item_number_agg: value_ext.item_number_agg,
                item_kinds: value_ext.item_kinds,
                item_domains: value_ext.item_domains,
                item_data_guards: HashMap::new(),
            })
        } else {
            None
        };
        Self {
            main: value.main,
            ext,
        }
    }
}
