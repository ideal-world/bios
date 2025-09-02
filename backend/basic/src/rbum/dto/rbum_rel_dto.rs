use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::default::Default;
use tardis::chrono::{DateTime, Utc};

use tardis::db::sea_orm;

use tardis::web::poem_openapi;

use crate::rbum::rbum_enumeration::{RbumRelEnvKind, RbumRelFromKind};

/// Add request for resource relationship
///
/// 资源关联添加请求
#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct RbumRelAddReq {
    /// Relationship tag
    ///
    /// 关联标签
    ///
    /// Used to distinguish different relationships.
    ///
    /// 用于区分不同的关联关系。
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub tag: String,
    /// Relationship note
    ///
    /// 关联备注
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub note: Option<String>,
    /// Relationship source type
    ///
    /// 关联来源方的类型
    pub from_rbum_kind: RbumRelFromKind,
    /// Relationship source id
    ///
    /// 关联来源方的id
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub from_rbum_id: String,
    /// Relationship target id
    ///
    /// 关联目标方的id
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub to_rbum_item_id: String,
    /// Relationship target ownership path
    ///
    /// 关联目标方的所有权路径
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub to_own_paths: String,
    /// Relationship extension information
    ///
    /// 关联扩展信息
    ///
    /// E.g. the record from or to is in another service, to avoid remote calls,
    /// you can redundantly add the required information to this field.
    ///
    /// 例如：记录来源或目标在另一个服务中，为避免远程调用，可以将所需信息冗余添加到此字段。
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub ext: Option<String>,
    /// Whether the target is an external object
    ///
    /// 关联目标方是否是外部对象
    ///
    /// If ``true``, the validity of the associated target will not be verified.
    ///
    /// 当为 ``true`` 不会校验关联目标方的合法性。
    pub to_is_outside: bool,

    /// Whether it is disabled
    ///
    /// 是否禁用
    ///
    /// A disabled relationship is not considered in business logic, but the data is still retained.
    ///
    /// 禁用的关联在业务逻辑中不予考虑，但数据仍然保留。
    pub disabled: Option<bool>,
}

/// Modify request for resource relationship
///
/// 资源关联修改请求
#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct RbumRelModifyReq {
    /// Relationship tag
    ///
    /// 关联标签
    ///
    /// Used to distinguish different relationships.
    ///
    /// 用于区分不同的关联关系。
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub tag: Option<String>,
    /// Relationship note
    ///
    /// 关联备注
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub note: Option<String>,
    /// Relationship extension information
    ///
    /// 关联扩展信息
    ///
    /// E.g. the record from or to is in another service, to avoid remote calls,
    /// you can redundantly add the required information to this field.
    ///
    /// 例如：记录来源或目标在另一个服务中，为避免远程调用，可以将所需信息冗余添加到此字段。
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub ext: Option<String>,

    /// Whether it is disabled
    ///
    /// 是否禁用
    ///
    /// A disabled relationship is not considered in business logic, but the data is still retained.
    ///
    /// 禁用的关联在业务逻辑中不予考虑，但数据仍然保留。
    pub disabled: Option<bool>,
}

/// Simple find request for resource relationship
///
/// 资源关联简单查找请求
#[derive(Serialize, Deserialize, Debug, Clone, Default, poem_openapi::Object)]
#[serde(default)]
pub struct RbumRelSimpleFindReq {
    /// Relationship tag
    ///
    /// 关联标签
    ///
    /// Used to distinguish different relationships.
    ///
    /// 用于区分不同的关联关系。
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub tag: Option<String>,
    /// Relationship source type
    ///
    /// 关联来源方的类型
    pub from_rbum_kind: Option<RbumRelFromKind>,
    /// Relationship source id
    ///
    /// 关联来源方的id
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub from_rbum_id: Option<String>,
    /// Relationship target id
    ///
    /// 关联目标方的id
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub to_rbum_item_id: Option<String>,
    /// Relationship source ownership path
    ///
    /// 关联来源方的所有权路径
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub from_own_paths: Option<String>,
    /// Relationship target ownership path
    ///
    /// 关联目标方的所有权路径
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub to_rbum_own_paths: Option<String>,
}

/// Check request for resource relationship
///
/// 资源关联检查请求
#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct RbumRelCheckReq {
    /// Relationship tag
    ///
    /// 关联标签
    ///
    /// Used to distinguish different relationships.
    ///
    /// 用于区分不同的关联关系。
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub tag: String,
    /// Relationship source type
    ///
    /// 关联来源方的类型
    pub from_rbum_kind: RbumRelFromKind,
    /// Relationship source id
    ///
    /// 关联来源方的id
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub from_rbum_id: String,
    /// Relationship target id
    ///
    /// 关联目标方的id
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub to_rbum_item_id: String,
    /// Limit the attributes of the relationship source
    ///
    /// 关联来源方的限定属性集合
    ///
    /// Format: ``{"Attribute name": "Input value"}``
    ///
    /// 格式: ``{"属性名称": "传入的值"}``
    pub from_attrs: HashMap<String, String>,
    /// Limit the attributes of the relationship target
    ///
    /// 关联目标方的限定属性集合
    ///
    /// Format: ``{"Attribute name": "Input value"}``
    ///
    /// 格式: ``{"属性名称": "传入的值"}``
    pub to_attrs: HashMap<String, String>,
    /// Limit the environment of the relationship
    ///
    /// 关联目标方的限定环境集合
    pub envs: Vec<RbumRelEnvCheckReq>,
}

/// Check request for resource relationship environment
///
/// 资源关联环境检查请求
#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct RbumRelEnvCheckReq {
    /// Relationship environment type
    ///
    /// 关联的环境类型
    pub kind: RbumRelEnvKind,
    /// Input value
    ///
    /// 传入的关联环境值
    pub value: String,
}

/// Resource relationship bone information
///
/// 资源关联骨干信息
#[derive(Serialize, Deserialize, Debug, Clone, poem_openapi::Object, sea_orm::FromQueryResult)]
pub struct RbumRelBoneResp {
    /// Relationship tag
    ///
    /// 关联标签
    pub tag: String,
    /// Relationship note
    ///
    /// 关联备注
    pub note: String,
    /// Relationship source type
    ///
    /// 关联来源方的类型
    pub from_rbum_kind: RbumRelFromKind,
    /// Relationship source or target id
    ///
    /// 关联来源方或目标方的id
    pub rel_id: String,
    /// Relationship source or target name
    ///
    /// 关联来源方或目标方的名称
    ///
    /// When `from_rbum_kind` is [`crate::rbum::rbum_enumeration::RbumRelFromKind::Item`] is the name of the resource item,
    /// When `from_rbum_kind` is [`crate::rbum::rbum_enumeration::RbumRelFromKind::Set`] is the name of the resource set,
    /// When `from_rbum_kind` is [`crate::rbum::rbum_enumeration::RbumRelFromKind::SetCate`] is the name of the resource set category(node).
    ///
    /// 当 `from_rbum_kind` 为 [`crate::rbum::rbum_enumeration::RbumRelFromKind::Item`] 时是资源项的名称，
    /// 当 `from_rbum_kind` 为 [`crate::rbum::rbum_enumeration::RbumRelFromKind::Set`] 时是资源集的名称，
    /// 当 `from_rbum_kind` 为 [`crate::rbum::rbum_enumeration::RbumRelFromKind::SetCate`] 时是资源集分类（节点）的名称。
    pub rel_name: String,
    /// Relationship source or target ownership path
    ///
    /// 关联来源方或目标方的所有权路径
    pub rel_own_paths: String,
    /// Relationship extension information
    ///
    /// 关联扩展信息
    ///
    /// E.g. the record from or to is in another service, to avoid remote calls,
    /// you can redundantly add the required information to this field.
    ///
    /// 例如：记录来源或目标在另一个服务中，为避免远程调用，可以将所需信息冗余添加到此字段。
    pub ext: String,
}

impl RbumRelBoneResp {
    /// According to the relationship detail information, generate the relationship summary information
    ///
    /// 根据关联详细信息生成关联概要信息
    ///
    /// # Arguments
    ///
    /// * `detail` - Relationship detail information
    /// * `package_to_info` - If ``true``, generate the summary information of the relationship source side, if ``false``, generate the summary information of the relationship target side
    pub fn new(detail: RbumRelDetailResp, package_to_info: bool) -> RbumRelBoneResp {
        if package_to_info {
            RbumRelBoneResp {
                tag: detail.tag,
                note: detail.note,
                from_rbum_kind: detail.from_rbum_kind,
                rel_id: detail.to_rbum_item_id,
                rel_name: detail.to_rbum_item_name,
                rel_own_paths: detail.to_own_paths,
                ext: detail.ext,
            }
        } else {
            RbumRelBoneResp {
                rel_name: match &detail.from_rbum_kind {
                    RbumRelFromKind::Item => detail.from_rbum_item_name,
                    RbumRelFromKind::Set => detail.from_rbum_set_name,
                    RbumRelFromKind::SetCate => detail.from_rbum_set_cate_name,
                    RbumRelFromKind::Cert => "".to_string(),
                    RbumRelFromKind::Other => "".to_string(),
                },
                tag: detail.tag,
                note: detail.note,
                from_rbum_kind: detail.from_rbum_kind,
                rel_id: detail.from_rbum_id,
                rel_own_paths: detail.own_paths,
                ext: detail.ext,
            }
        }
    }
}

/// Resource relationship detail information
///
/// 资源关联详细信息
#[derive(Serialize, Deserialize, Clone, Debug, poem_openapi::Object, sea_orm::FromQueryResult)]
pub struct RbumRelDetailResp {
    /// Relationship id
    ///
    /// 关联id
    pub id: String,
    /// Relationship tag
    ///
    /// 关联标签
    pub tag: String,
    /// Relationship note
    ///
    /// 关联备注
    pub note: String,
    /// Relationship source type
    ///
    /// 关联来源方的类型
    pub from_rbum_kind: RbumRelFromKind,
    /// Relationship source id
    ///
    /// 关联来源方的id
    pub from_rbum_id: String,
    /// Relationship source resource item name
    ///
    /// 关联来源方的资源项名称
    ///
    /// Only valid when `from_rbum_kind` is [`crate::rbum::rbum_enumeration::RbumRelFromKind::Item`].
    ///
    /// 仅当 `from_rbum_kind` 为 [`crate::rbum::rbum_enumeration::RbumRelFromKind::Item`] 时有值。
    pub from_rbum_item_name: String,
    /// Relationship source resource set name
    ///
    /// 关联来源方的资源集名称
    ///
    /// Only valid when `from_rbum_kind` is [`crate::rbum::rbum_enumeration::RbumRelFromKind::Set`].
    ///
    /// 仅当 `from_rbum_kind` 为 [`crate::rbum::rbum_enumeration::RbumRelFromKind::Set`] 时有值。
    pub from_rbum_set_name: String,
    /// Relationship source resource set category(node) name
    ///
    /// 关联来源方的资源集分类（节点）名称
    ///
    /// Only valid when `from_rbum_kind` is [`crate::rbum::rbum_enumeration::RbumRelFromKind::SetCate`].
    ///
    /// 仅当 `from_rbum_kind` 为 [`crate::rbum::rbum_enumeration::RbumRelFromKind::SetCate`] 时有值。
    pub from_rbum_set_cate_name: String,
    /// Relationship target id
    ///
    /// 关联目标方的id
    pub to_rbum_item_id: String,
    /// Relationship target name
    ///
    /// 关联目标方的name
    pub to_rbum_item_name: String,
    /// Relationship target ownership path
    ///
    /// 关联目标方的所有权路径
    pub to_own_paths: String,
    /// Relationship extension information
    ///
    /// 关联扩展信息
    pub ext: String,

    pub own_paths: String,
    pub owner: String,
    pub owner_name: Option<String>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}
