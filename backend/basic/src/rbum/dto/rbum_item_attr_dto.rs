use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tardis::chrono::{DateTime, Utc};
#[cfg(feature = "default")]
use tardis::db::sea_orm;
#[cfg(feature = "default")]
use tardis::web::poem_openapi;

/// Add request for resource item extended attribute value
///
/// 资源项扩展属性值添加请求
#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct RbumItemAttrAddReq {
    /// Extended attribute value
    ///
    /// 扩展属性值
    #[cfg_attr(feature = "default", oai(validator(min_length = "1", max_length = "2000")))]
    pub value: String,
    /// Associated [resource item](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) id
    ///
    /// 关联的[资源项](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) id
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub rel_rbum_item_id: String,
    /// Associated [resource kind attribute definition](crate::rbum::dto::rbum_kind_attr_dto::RbumKindAttrDetailResp) id
    ///
    /// 关联的[资源类型属性定义](crate::rbum::dto::rbum_kind_attr_dto::RbumKindAttrDetailResp) id
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub rel_rbum_kind_attr_id: String,
}

/// Modify request for resource item extended attribute value
///
/// 资源项扩展属性值修改请求
#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct RbumItemAttrModifyReq {
    /// Extended attribute value
    ///
    /// 扩展属性值
    #[cfg_attr(feature = "default", oai(validator(min_length = "1", max_length = "2000")))]
    pub value: String,
}

/// Batch add or modify request for resource item extended attribute values
///
/// 批量添加或修改资源项扩展属性值请求
#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct RbumItemAttrsAddOrModifyReq {
    /// Add or modify value collection
    ///
    /// 添加或修改的值集合
    ///
    /// Format: ``{ "field name": "field value" }``
    ///
    /// ``field name``: [`crate::rbum::dto::rbum_kind_attr_dto::RbumKindAttrDetailResp::name`]
    pub values: HashMap<String, String>,
    /// Associated [resource item](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) id
    ///
    /// 关联的[资源项](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) id
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub rel_rbum_item_id: String,
}

/// Resource item extended attribute value summary information
///
/// 源项扩展属性值概要信息
#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object, sea_orm::FromQueryResult))]
pub struct RbumItemAttrSummaryResp {
    /// Extended attribute value id
    ///
    /// 扩展属性值id
    pub id: String,
    /// Extended attribute value
    ///
    /// 扩展属性值
    pub value: String,
    /// Associated [resource item](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) id
    ///
    /// 关联的[资源项](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) id
    pub rel_rbum_item_id: String,
    /// Associated [resource kind attribute definition](crate::rbum::dto::rbum_kind_attr_dto::RbumKindAttrDetailResp) id
    ///
    /// 关联的[资源类型属性定义](crate::rbum::dto::rbum_kind_attr_dto::RbumKindAttrDetailResp) id
    pub rel_rbum_kind_attr_id: String,
    /// Associated [resource kind attribute definition](crate::rbum::dto::rbum_kind_attr_dto::RbumKindAttrDetailResp) name
    ///
    /// 关联的[资源类型属性定义](crate::rbum::dto::rbum_kind_attr_dto::RbumKindAttrDetailResp) 名称
    pub rel_rbum_kind_attr_name: String,

    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

/// Resource item extended attribute value detail information
///
/// 源项扩展属性值详细信息
#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object, sea_orm::FromQueryResult))]
pub struct RbumItemAttrDetailResp {
    /// Extended attribute value id
    ///
    /// 扩展属性值id
    pub id: String,
    /// Extended attribute value
    ///
    /// 扩展属性值
    pub value: String,
    /// Associated [resource item](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) id
    ///
    /// 关联的[资源项](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) id
    pub rel_rbum_item_id: String,
    /// Associated [resource kind attribute definition](crate::rbum::dto::rbum_kind_attr_dto::RbumKindAttrDetailResp) id
    ///
    /// 关联的[资源类型属性定义](crate::rbum::dto::rbum_kind_attr_dto::RbumKindAttrDetailResp) id
    pub rel_rbum_kind_attr_id: String,
    /// Associated [resource kind attribute definition](crate::rbum::dto::rbum_kind_attr_dto::RbumKindAttrDetailResp) name
    ///
    /// 关联的[资源类型属性定义](crate::rbum::dto::rbum_kind_attr_dto::RbumKindAttrDetailResp) 名称
    pub rel_rbum_kind_attr_name: String,

    pub own_paths: String,
    pub owner: String,
    pub owner_name: Option<String>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}
