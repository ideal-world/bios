use serde::{Deserialize, Serialize};
use tardis::chrono::{DateTime, Utc};

use tardis::db::sea_orm;

use tardis::web::poem_openapi;

/// Add request for resource relationship attribute condition
///
/// 资源关联属性条件添加请求
#[derive(Serialize, Deserialize, Debug)]
#[derive(poem_openapi::Object)]
pub struct RbumRelAttrAddReq {
    /// Condition qualifier
    ///
    /// 条件限定符
    ///
    /// if true, it means the limitation of the relationship source,
    /// otherwise it is the limitation of the relationship target resource.
    ///
    /// 如果为true，表示关联来源方的限定，否则为关联目标方资源的限定。
    pub is_from: bool,
    /// Relationship attribute name
    ///
    /// 关联属性名称
    ///
    /// When ``rel_rbum_kind_attr_id`` exists, use the corresponding [`crate::rbum::dto::rbum_kind_attr_dto::RbumKindAttrDetailResp::name`], otherwise this field is not empty.
    ///
    /// 当 ``rel_rbum_kind_attr_id`` 存在时使用其对应的 [`crate::rbum::dto::rbum_kind_attr_dto::RbumKindAttrDetailResp::name`]，否则此字段不为空。
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: Option<String>,
    /// Relationship attribute value
    ///
    /// 关联属性值
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub value: String,
    /// Whether to only record
    ///
    /// 是否仅记录
    ///
    /// If true, this condition is only used for records and does not participate in the judgment of whether the relationship is established.
    ///
    /// 如果为true，该条件仅用于记录，不参与判断关联关系是否建立。
    pub record_only: bool,
    /// Associated [resource kind attribute](crate::rbum::dto::rbum_kind_attr_dto::RbumKindAttrDetailResp) id
    ///
    /// 关联的[资源类型属性](crate::rbum::dto::rbum_kind_attr_dto::RbumKindAttrDetailResp) id
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub rel_rbum_kind_attr_id: Option<String>,
    /// Associated [relationship](crate::rbum::dto::rbum_rel_dto::RbumRelDetailResp) id
    ///
    /// 关联的[资源关联](crate::rbum::dto::rbum_rel_dto::RbumRelDetailResp) id
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub rel_rbum_rel_id: String,
}

/// Modify request for resource relationship attribute condition
///
/// 资源关联属性条件修改请求
#[derive(Serialize, Deserialize, Debug)]
#[derive(poem_openapi::Object)]
pub struct RbumRelAttrModifyReq {
    /// Relationship attribute value
    ///
    /// 关联属性值
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub value: String,
}

/// Resource relationship attribute condition detail information
///
/// 资源关联属性条件详细信息
#[derive(Serialize, Deserialize, Clone, Debug)]
#[derive(poem_openapi::Object, sea_orm::FromQueryResult)]
pub struct RbumRelAttrDetailResp {
    /// Relationship attribute id
    ///
    /// 关联属性id
    pub id: String,
    /// Condition qualifier
    ///
    /// 条件限定符
    pub is_from: bool,
    /// Relationship attribute name
    ///
    /// 关联属性名称
    pub name: String,
    /// Relationship attribute value
    ///
    /// 关联属性值
    pub value: String,
    /// Whether to only record
    ///
    /// 是否仅记录
    pub record_only: bool,
    /// Associated [resource kind attribute](crate::rbum::dto::rbum_kind_attr_dto::RbumKindAttrDetailResp) id
    ///
    /// 关联的[资源类型属性](crate::rbum::dto::rbum_kind_attr_dto::RbumKindAttrDetailResp) id
    pub rel_rbum_kind_attr_id: String,
    /// Associated [resource kind attribute](crate::rbum::dto::rbum_kind_attr_dto::RbumKindAttrDetailResp) name
    ///
    /// 关联的[资源类型属性](crate::rbum::dto::rbum_kind_attr_dto::RbumKindAttrDetailResp) 名称
    pub rel_rbum_kind_attr_name: String,
    /// Associated [relationship](crate::rbum::dto::rbum_rel_dto::RbumRelDetailResp) id
    ///
    /// 关联的[资源关联](crate::rbum::dto::rbum_rel_dto::RbumRelDetailResp) id
    pub rel_rbum_rel_id: String,

    pub own_paths: String,
    pub owner: String,
    pub owner_name: Option<String>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}
