use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::chrono::{DateTime, Utc};
#[cfg(feature = "default")]
use tardis::db::sea_orm;
#[cfg(feature = "default")]
use tardis::web::poem_openapi;

use crate::rbum::rbum_enumeration::{RbumCertRelKind, RbumCertStatusKind};

/// Add request for certificate
///
/// 凭证添加请求
#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct RbumCertAddReq {
    /// Certification access key
    ///
    /// 凭证名
    ///
    /// see [`crate::rbum::dto::rbum_cert_conf_dto::RbumCertConfAddReq::ak_rule`]
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "2000")))]
    pub ak: TrimString,
    /// Certification secret key
    ///
    /// 凭证密钥
    ///
    /// see [`crate::rbum::dto::rbum_cert_conf_dto::RbumCertConfAddReq::sk_rule`]
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "10000")))]
    pub sk: Option<TrimString>,
    /// Whether to hide the sk
    ///
    /// 是否隐藏密钥
    ///
    /// Default is ``false``
    ///
    /// 默认为 ``false``
    ///
    /// In some scenarios with high security requirements, you can choose to hide the key, such as: display as "******".
    ///
    /// 在一些安全性要求较高的场景下，可以选择隐藏密钥，如：显示为“******”。
    pub sk_invisible: Option<bool>,
    /// Whether to ignore the key check
    ///
    /// 是否忽略密钥校验
    ///
    /// WARNING: This field is only for special scenarios, please use it with caution.
    ///
    /// 警告：此字段仅用于特殊场景，请谨慎使用。
    pub ignore_check_sk: bool,
    /// Certificate type
    ///
    /// 凭证类型
    ///
    /// Different from [`crate::rbum::dto::rbum_cert_conf_dto::RbumCertConfAddReq::kind`], when this data exists, it indicates that the certificate does not need to be associated with the certificate configuration.
    ///
    /// 与 [`crate::rbum::dto::rbum_cert_conf_dto::RbumCertConfAddReq::kind`] 不同，当存在此数据时表明该凭证不用关联凭证配置。
    pub kind: Option<String>,
    /// Certificate supplier
    ///
    /// 凭证供应商
    ///
    /// Different from [`crate::rbum::dto::rbum_cert_conf_dto::RbumCertConfAddReq::supplier`], when this data exists, it indicates that the certificate does not need to be associated with the certificate configuration.
    ///
    /// 与 [`crate::rbum::dto::rbum_cert_conf_dto::RbumCertConfAddReq::supplier`] 不同，当存在此数据时表明该凭证不用关联凭证配置。
    pub supplier: Option<String>,
    /// Certificate extension information
    ///
    /// 凭证扩展信息
    ///
    /// Such as database connection pool configuration.
    ///
    /// 比如数据库连接池配置。
    ///
    /// Different from [`crate::rbum::dto::rbum_cert_conf_dto::RbumCertConfAddReq::ext`], this field is used to identify the specific extension information of the certificate.
    ///
    /// 与 [`crate::rbum::dto::rbum_cert_conf_dto::RbumCertConfAddReq::ext`] 不同，此字段用于标识该条凭证的特有的扩展信息。
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "2000")))]
    pub ext: Option<String>,
    /// Certificate effective time
    ///
    /// Default is ``Current time``
    ///
    /// 默认为 ``当前时间``
    ///
    /// 凭证的生效时间
    pub start_time: Option<DateTime<Utc>>,
    /// Certificate expiration time
    ///
    /// When associated with [certificate configuration](crate::rbum::dto::rbum_cert_conf_dto::RbumCertConfDetailResp), it defaults to ``start_time + expiration time of the certificate configuration``,
    /// otherwise it defaults to ``start_time + 100 years``.
    ///
    /// 当关联了 [凭证配置](crate::rbum::dto::rbum_cert_conf_dto::RbumCertConfDetailResp) 时，默认为 ``start_time + 凭证配置的过期时间``, 否则默认为 ``start_time + 100年``。
    ///
    /// NOTE: When associated with [certificate configuration](crate::rbum::dto::rbum_cert_conf_dto::RbumCertConfDetailResp) and ``is dynamic sk``, it defaults to ``start_time + 100 years``.
    ///
    /// NOTE: 当关联了 [凭证配置](crate::rbum::dto::rbum_cert_conf_dto::RbumCertConfDetailResp) 且 ``为动态sk时`` 默认为 ``start_time + 100年``
    pub end_time: Option<DateTime<Utc>>,
    /// Certificate connection address
    ///
    /// 凭证连接地址
    ///
    /// Different from [`crate::rbum::dto::rbum_cert_conf_dto::RbumCertConfAddReq::conn_uri`], this field is used to identify the specific connection address of the certificate.
    ///
    /// 与 [`crate::rbum::dto::rbum_cert_conf_dto::RbumCertConfAddReq::conn_uri`] 不同，此字段用于标识该条凭证的特有的连接地址。
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "2000")))]
    pub conn_uri: Option<String>,
    /// Credential status
    ///
    /// 凭证的状态
    pub status: RbumCertStatusKind,

    /// Dynamic sk(verification code)
    ///
    /// 动态sk（验证码）
    ///
    /// NOTE: Only valid when [`crate::rbum::dto::rbum_cert_conf_dto::RbumCertConfAddReq::sk_dynamic`] is ``true``.
    ///
    /// NOTE: This field cannot exist with the ``sk`` field at the same time.
    ///
    /// NOTE: 仅当  [`crate::rbum::dto::rbum_cert_conf_dto::RbumCertConfAddReq::sk_dynamic`] 为 ``true`` 时有效。
    ///
    /// NOTE: 此字段不可与 ``sk`` 字段同时存在。
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "2000")))]
    pub vcode: Option<TrimString>,

    /// Associated [cert configuration](crate::rbum::dto::rbum_cert_conf_dto::RbumCertConfDetailResp) id
    ///
    /// 关联的[凭证配置](crate::rbum::dto::rbum_cert_conf_dto::RbumCertConfDetailResp)id
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub rel_rbum_cert_conf_id: Option<String>,
    /// Associated [resource kind](crate::rbum::rbum_enumeration::RbumCertRelKind)
    ///
    /// 关联的[资源类型](crate::rbum::rbum_enumeration::RbumCertRelKind)
    pub rel_rbum_kind: RbumCertRelKind,
    /// Associated resource id
    ///
    /// 关联的资源id
    ///
    /// # examples:
    ///
    /// * if rel_rbum_kind == Item
    ///   - rel_rbum_id same as the rel_rbum_item_id of cert configuration：E.g. Gitlab token
    ///   - rel_rbum_id different as the rel_rbum_item_id of cert configuration：E.g. User password (the cert configuration is bound to the tenant, and the cert instance corresponds to the user)
    ///
    /// * if rel_rbum_kind == Set
    ///   - E.g. In the Plug-in service, it can be bound to the plug-in instance library
    ///
    /// * if rel_rbum_kind == Rel
    ///  - In the CMDB service, a resource can be sliced (E.g. DB instance), we can specify slice information of association
    ///
    /// # 使用示例：
    ///
    /// * 如果 rel_rbum_kind == Item
    ///  - rel_rbum_id 与 cert configuration 的 rel_rbum_item_id 相同：比如 Gitlab token
    ///  - rel_rbum_id 与 cert configuration 的 rel_rbum_item_id 不同：比如 用户密码（cert configuration 绑定租户，cert 实例对应用户）
    ///
    /// * 如果 rel_rbum_kind == Set
    /// - 比如在插件服务中，可以绑定到插件实例库
    ///
    /// * 如果 rel_rbum_kind == Rel
    /// - 在 CMDB 服务中，一个资源可以被切片（比如 DB 实例），我们可以指定关联的切片信息
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub rel_rbum_id: String,
    /// Whether ``rel_rbum_id`` is an external value
    ///
    /// ``rel_rbum_id`` 是否是外部值
    ///
    /// If ``true``, ignore the scope check for ``rel_rbum_id``.
    ///
    /// 当为 ``true`` 时忽略对 ``rel_rbum_id`` 的作用域检查.
    pub is_outside: bool,
}

/// Modify request for certificate
///
/// 凭证修改请求
#[derive(Serialize, Deserialize, Debug, Default)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct RbumCertModifyReq {
    /// Certification access key
    ///
    /// 凭证名
    ///
    /// see [`crate::rbum::dto::rbum_cert_conf_dto::RbumCertConfModifyReq::ak_rule`]
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "2000")))]
    pub ak: Option<TrimString>,
    /// Certification secret key
    ///
    /// 凭证密钥
    ///
    /// see [`crate::rbum::dto::rbum_cert_conf_dto::RbumCertConfAddReq::sk_rule`]
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "10000")))]
    pub sk: Option<TrimString>,
    /// Whether to hide the sk
    ///
    /// 是否隐藏密钥
    ///
    /// In some scenarios with high security requirements, you can choose to hide the key, such as: display as "******".
    ///
    /// 在一些安全性要求较高的场景下，可以选择隐藏密钥，如：显示为“******”。
    pub sk_invisible: Option<bool>,
    /// Whether to ignore the key check
    ///
    /// 是否忽略密钥校验
    ///
    /// WARNING: This field is only for special scenarios, please use it with caution.
    ///
    /// 警告：此字段仅用于特殊场景，请谨慎使用。
    pub ignore_check_sk: bool,
    /// Certificate extension information
    ///
    /// 凭证扩展信息
    ///
    /// Such as database connection pool configuration.
    ///
    /// 比如数据库连接池配置。
    ///
    /// Different from [`crate::rbum::dto::rbum_cert_conf_dto::RbumCertConfModifyReq::ext`], this field is used to identify the specific extension information of the certificate.
    ///
    /// 与 [`crate::rbum::dto::rbum_cert_conf_dto::RbumCertConfModifyReq::ext`] 不同，此字段用于标识该条凭证的特有的扩展信息。
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "2000")))]
    pub ext: Option<String>,
    /// Certificate effective time
    ///
    /// 凭证的生效时间
    pub start_time: Option<DateTime<Utc>>,
    /// Certificate expiration time
    ///
    /// 凭证的失效时间
    pub end_time: Option<DateTime<Utc>>,
    /// Certificate connection address
    ///
    /// 凭证连接地址
    ///
    /// Different from [`crate::rbum::dto::rbum_cert_conf_dto::RbumCertConfModifyReq::conn_uri`], this field is used to identify the specific connection address of the certificate.
    ///
    /// 与 [`crate::rbum::dto::rbum_cert_conf_dto::RbumCertConfModifyReq::conn_uri`] 不同，此字段用于标识该条凭证的特有的连接地址。
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "2000")))]
    pub conn_uri: Option<String>,
    /// Credential status
    ///
    /// 凭证的状态
    pub status: Option<RbumCertStatusKind>,
}

/// Certificate summary information
///
/// 凭证概要信息
#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object, sea_orm::FromQueryResult))]
pub struct RbumCertSummaryResp {
    /// Certification id
    ///
    /// 凭证id
    pub id: String,
    /// Certification access key
    ///
    /// 凭证名
    pub ak: String,
    /// Certificate type
    ///
    /// 凭证类型
    pub kind: String,
    /// Certificate supplier
    ///
    /// 凭证供应商
    pub supplier: String,
    /// Certificate extension information
    ///
    /// 凭证扩展信息
    pub ext: String,
    /// Certificate effective time
    ///
    /// 凭证的生效时间
    pub start_time: DateTime<Utc>,
    /// Certificate expiration time
    ///
    /// 凭证的失效时间
    pub end_time: DateTime<Utc>,
    /// Credential status
    ///
    /// 凭证的状态
    pub status: RbumCertStatusKind,

    /// Associated [cert configuration](crate::rbum::dto::rbum_cert_conf_dto::RbumCertConfDetailResp) id
    ///
    /// 关联的[凭证配置](crate::rbum::dto::rbum_cert_conf_dto::RbumCertConfDetailResp)id
    pub rel_rbum_cert_conf_id: Option<String>,
    /// Associated [cert configuration](crate::rbum::dto::rbum_cert_conf_dto::RbumCertConfDetailResp) name
    ///
    /// 关联的[凭证配置](crate::rbum::dto::rbum_cert_conf_dto::RbumCertConfDetailResp)名称
    pub rel_rbum_cert_conf_name: Option<String>,
    // TODO
    pub rel_rbum_cert_conf_code: Option<String>,
    /// Associated [resource kind](crate::rbum::rbum_enumeration::RbumCertRelKind)
    ///
    /// 关联的[资源类型](crate::rbum::rbum_enumeration::RbumCertRelKind)
    pub rel_rbum_kind: RbumCertRelKind,
    /// Associated resource id
    ///
    /// 关联的资源id
    pub rel_rbum_id: String,

    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

/// Certificate summary information with secret key
///
/// 带有密钥的凭证概要信息
#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object, sea_orm::FromQueryResult))]
pub struct RbumCertSummaryWithSkResp {
    /// Certification id
    ///
    /// 凭证id
    pub id: String,
    /// Certification access key
    ///
    /// 凭证名
    pub ak: String,
    /// Certification secret key
    ///
    /// 凭证密钥
    pub sk: String,
    /// Whether to hide the sk
    ///
    /// 是否隐藏密钥
    pub sk_invisible: bool,
    /// Certificate type
    ///
    /// 凭证类型
    pub kind: String,
    /// Certificate supplier
    ///
    /// 凭证供应商
    pub supplier: String,
    /// Certificate extension information
    ///
    /// 凭证扩展信息
    pub ext: String,
    /// Certificate effective time
    ///
    /// 凭证的生效时间
    pub start_time: DateTime<Utc>,
    /// Certificate expiration time
    ///
    /// 凭证的失效时间
    pub end_time: DateTime<Utc>,
    /// Certificate connection address
    ///
    /// 凭证连接地址
    pub conn_uri: String,
    /// Credential status
    ///
    /// 凭证的状态
    pub status: RbumCertStatusKind,

    /// Associated [cert configuration](crate::rbum::dto::rbum_cert_conf_dto::RbumCertConfDetailResp) id
    ///
    /// 关联的[凭证配置](crate::rbum::dto::rbum_cert_conf_dto::RbumCertConfDetailResp)id
    pub rel_rbum_cert_conf_id: Option<String>,
    /// Associated [cert configuration](crate::rbum::dto::rbum_cert_conf_dto::RbumCertConfDetailResp) name
    ///
    /// 关联的[凭证配置](crate::rbum::dto::rbum_cert_conf_dto::RbumCertConfDetailResp)名称
    pub rel_rbum_cert_conf_name: Option<String>,
    // TODO
    pub rel_rbum_cert_conf_code: Option<String>,
    /// Associated [resource kind](crate::rbum::rbum_enumeration::RbumCertRelKind)
    ///
    /// 关联的[资源类型](crate::rbum::rbum_enumeration::RbumCertRelKind)
    pub rel_rbum_kind: RbumCertRelKind,
    /// Associated resource id
    ///
    /// 关联的资源id
    pub rel_rbum_id: String,

    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

/// Certificate detail information
///
/// 凭证详细信息
#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object, sea_orm::FromQueryResult))]
pub struct RbumCertDetailResp {
    /// Certification id
    ///
    /// 凭证id
    pub id: String,
    /// Certification access key
    ///
    /// 凭证名
    pub ak: String,
    /// Whether to hide the sk
    ///
    /// 是否隐藏密钥
    pub sk_invisible: bool,
    /// Certificate type
    ///
    /// 凭证类型
    pub kind: String,
    /// Certificate supplier
    ///
    /// 凭证供应商
    pub supplier: String,
    /// Certificate extension information
    ///
    /// 凭证扩展信息
    pub ext: String,
    /// Certificate effective time
    ///
    /// 凭证的生效时间
    pub start_time: DateTime<Utc>,
    /// Certificate expiration time
    ///
    /// 凭证的失效时间
    pub end_time: DateTime<Utc>,
    /// Certificate connection address
    ///
    /// 凭证连接地址
    pub conn_uri: String,
    /// Credential status
    ///
    /// 凭证的状态
    pub status: RbumCertStatusKind,

    /// Associated [cert configuration](crate::rbum::dto::rbum_cert_conf_dto::RbumCertConfDetailResp) id
    ///
    /// 关联的[凭证配置](crate::rbum::dto::rbum_cert_conf_dto::RbumCertConfDetailResp)id
    pub rel_rbum_cert_conf_id: Option<String>,
    /// Associated [cert configuration](crate::rbum::dto::rbum_cert_conf_dto::RbumCertConfDetailResp) name
    ///
    /// 关联的[凭证配置](crate::rbum::dto::rbum_cert_conf_dto::RbumCertConfDetailResp)名称
    pub rel_rbum_cert_conf_name: Option<String>,
    // TODO
    pub rel_rbum_cert_conf_code: Option<String>,
    /// Associated [resource kind](crate::rbum::rbum_enumeration::RbumCertRelKind)
    ///
    /// 关联的[资源类型](crate::rbum::rbum_enumeration::RbumCertRelKind)
    pub rel_rbum_kind: RbumCertRelKind,
    /// Associated resource id
    ///
    /// 关联的资源id
    pub rel_rbum_id: String,

    pub own_paths: String,
    pub owner: String,
    pub owner_name: Option<String>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}
