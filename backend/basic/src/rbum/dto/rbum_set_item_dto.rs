use serde::{Deserialize, Serialize};
use tardis::chrono::{DateTime, Utc};

use tardis::db::sea_orm;

use tardis::web::poem_openapi;

use crate::rbum::rbum_enumeration::RbumScopeLevelKind;

/// Add request for association between resource set category(node) and resource item
///
/// 添加资源集分类（节点）挂载资源项的关联的请求
#[derive(Serialize, Deserialize, Debug)]
#[derive(poem_openapi::Object)]
pub struct RbumSetItemAddReq {
    /// Association sort
    ///
    /// 关联排序
    pub sort: i64,
    /// Associated [resource set](crate::rbum::dto::rbum_set_dto::RbumSetDetailResp) id
    ///
    /// 关联[资源集](crate::rbum::dto::rbum_set_dto::RbumSetDetailResp) id
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub rel_rbum_set_id: String,
    /// Associated [resource set category(node)](crate::rbum::dto::rbum_set_cate_dto::RbumSetCateDetailResp) id
    ///
    /// 关联[资源集分类（节点）](crate::rbum::dto::rbum_set_cate_dto::RbumSetCateDetailResp) id
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub rel_rbum_set_cate_id: String,
    /// Associated [resource item](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) id
    ///
    /// 关联[资源项](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) id
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub rel_rbum_item_id: String,
}

/// Modify request for association between resource set category(node) and resource item
///
/// 修改资源集分类（节点）挂载资源项的关联的请求
#[derive(Serialize, Deserialize, Debug)]
#[derive(poem_openapi::Object)]
pub struct RbumSetItemModifyReq {
    /// Associated [resource set category(node)](crate::rbum::dto::rbum_set_cate_dto::RbumSetCateDetailResp) id
    ///
    /// 关联[资源集分类（节点）](crate::rbum::dto::rbum_set_cate_dto::RbumSetCateDetailResp) id
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub rel_rbum_set_cate_id: Option<String>,
    /// Association sort
    ///
    /// 关联排序
    pub sort: Option<i64>,
}

/// Summary information of the association between resource set category(node) and resource item
///
/// 资源集分类（节点）挂载资源项的关联的概要信息
#[derive(Serialize, Deserialize, Debug)]
#[derive(poem_openapi::Object, sea_orm::FromQueryResult)]
pub struct RbumSetItemSummaryResp {
    /// Association id
    ///
    /// 关联id
    pub id: String,
    /// Association sort
    ///
    /// 关联排序
    pub sort: i64,
    /// Associated [resource set](crate::rbum::dto::rbum_set_dto::RbumSetDetailResp) id
    ///
    /// 关联[资源集](crate::rbum::dto::rbum_set_dto::RbumSetDetailResp) id
    pub rel_rbum_set_id: String,
    /// Associated [resource set category(node)](crate::rbum::dto::rbum_set_cate_dto::RbumSetCateDetailResp) id
    ///
    /// 关联[资源集分类（节点）](crate::rbum::dto::rbum_set_cate_dto::RbumSetCateDetailResp) id
    pub rel_rbum_set_cate_id: Option<String>,
    /// Associated [resource set category(node)](crate::rbum::dto::rbum_set_cate_dto::RbumSetCateDetailResp) sys_code
    ///
    /// 关联[资源集分类（节点）](crate::rbum::dto::rbum_set_cate_dto::RbumSetCateDetailResp) sys_code
    pub rel_rbum_set_cate_sys_code: Option<String>,
    /// Associated [resource set category(node)](crate::rbum::dto::rbum_set_cate_dto::RbumSetCateDetailResp) name
    ///
    /// 关联[资源集分类（节点）](crate::rbum::dto::rbum_set_cate_dto::RbumSetCateDetailResp) name
    pub rel_rbum_set_cate_name: Option<String>,
    /// Associated [resource item](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) id
    ///
    /// 关联[资源项](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) id
    pub rel_rbum_item_id: String,
    /// Associated [resource item](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) name
    ///
    /// 关联[资源项](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) name
    pub rel_rbum_item_name: String,

    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

/// Resource item information of the association between resource set category(node) and resource item
///
/// 资源集分类（节点）挂载资源项的关联的资源项信息
#[derive(Serialize, Deserialize, Debug, Clone)]
#[derive(poem_openapi::Object, sea_orm::FromQueryResult)]
pub struct RbumSetItemRelInfoResp {
    /// Association id
    ///
    /// 关联id
    pub id: String,
    /// Association sort
    ///
    /// 关联排序
    pub sort: i64,
    /// Associated [resource item](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) id
    ///
    /// 关联[资源项](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) id
    pub rel_rbum_item_id: String,
    /// Associated [resource item](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) code
    ///
    /// 关联[资源项](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) code
    pub rel_rbum_item_code: String,
    /// Associated [resource item](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) name
    ///
    /// 关联[资源项](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) name
    pub rel_rbum_item_name: String,
    /// Associated [resource item](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) rel_rbum_kind_id
    ///
    /// 关联[资源项](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) rel_rbum_kind_id
    pub rel_rbum_item_kind_id: String,
    /// Associated [resource item](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) rel_rbum_domain_id
    ///
    /// 关联[资源项](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) rel_rbum_domain_id
    pub rel_rbum_item_domain_id: String,
    /// Associated [resource item](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) owner
    ///
    /// 关联[资源项](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) owner
    pub rel_rbum_item_owner: String,
    /// Associated [resource item](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) create_time
    ///
    /// 关联[资源项](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) create_time
    pub rel_rbum_item_create_time: DateTime<Utc>,
    /// Associated [resource item](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) update_time
    ///
    /// 关联[资源项](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) update_time
    pub rel_rbum_item_update_time: DateTime<Utc>,
    /// Associated [resource item](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) disabled
    ///
    /// 关联[资源项](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) disabled
    pub rel_rbum_item_disabled: bool,
    /// Associated [resource item](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) scope_level
    ///
    /// 关联[资源项](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) scope_level
    pub rel_rbum_item_scope_level: RbumScopeLevelKind,

    pub own_paths: String,
    pub owner: String,
}

/// Detail information of the association between resource set category(node) and resource item
///
/// 资源集分类（节点）挂载资源项的关联的详细信息
#[derive(Serialize, Deserialize, Debug)]
#[derive(poem_openapi::Object, sea_orm::FromQueryResult)]
pub struct RbumSetItemDetailResp {
    /// Association id
    ///
    /// 关联id
    pub id: String,
    /// Association sort
    ///
    /// 关联排序
    pub sort: i64,
    /// Associated [resource set](crate::rbum::dto::rbum_set_dto::RbumSetDetailResp) id
    ///
    /// 关联[资源集](crate::rbum::dto::rbum_set_dto::RbumSetDetailResp) id
    pub rel_rbum_set_id: String,
    /// Associated [resource set category(node)](crate::rbum::dto::rbum_set_cate_dto::RbumSetCateDetailResp) id
    ///
    /// 关联[资源集分类（节点）](crate::rbum::dto::rbum_set_cate_dto::RbumSetCateDetailResp) id
    pub rel_rbum_set_cate_id: Option<String>,
    /// Associated [resource set category(node)](crate::rbum::dto::rbum_set_cate_dto::RbumSetCateDetailResp) sys_code
    ///
    /// 关联[资源集分类（节点）](crate::rbum::dto::rbum_set_cate_dto::RbumSetCateDetailResp) sys_code
    pub rel_rbum_set_cate_sys_code: Option<String>,
    /// Associated [resource set category(node)](crate::rbum::dto::rbum_set_cate_dto::RbumSetCateDetailResp) name
    ///
    /// 关联[资源集分类（节点）](crate::rbum::dto::rbum_set_cate_dto::RbumSetCateDetailResp) name
    pub rel_rbum_set_cate_name: Option<String>,
    /// Associated [resource item](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) id
    ///
    /// 关联[资源项](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) id
    pub rel_rbum_item_id: String,
    /// Associated [resource item](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) code
    ///
    /// 关联[资源项](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) code
    pub rel_rbum_item_code: String,
    /// Associated [resource item](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) name
    ///
    /// 关联[资源项](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) name
    pub rel_rbum_item_name: String,
    /// Associated [resource item](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) rel_rbum_kind_id
    ///
    /// 关联[资源项](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) rel_rbum_kind_id
    pub rel_rbum_item_kind_id: String,
    /// Associated [resource item](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) rel_rbum_domain_id
    ///
    /// 关联[资源项](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) rel_rbum_domain_id
    pub rel_rbum_item_domain_id: String,
    /// Associated [resource item](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) owner
    ///
    /// 关联[资源项](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) owner
    pub rel_rbum_item_owner: String,
    /// Associated [resource item](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) create_time
    ///
    /// 关联[资源项](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) create_time
    pub rel_rbum_item_create_time: DateTime<Utc>,
    /// Associated [resource item](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) update_time
    ///
    /// 关联[资源项](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) update_time
    pub rel_rbum_item_update_time: DateTime<Utc>,
    /// Associated [resource item](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) disabled
    ///
    /// 关联[资源项](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) disabled
    pub rel_rbum_item_disabled: bool,
    /// Associated [resource item](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) scope_level
    ///
    /// 关联[资源项](crate::rbum::dto::rbum_item_dto::RbumItemDetailResp) scope_level
    pub rel_rbum_item_scope_level: RbumScopeLevelKind,

    pub own_paths: String,
    pub owner: String,
    pub owner_name: Option<String>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}
