use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::chrono::{DateTime, Utc};

use tardis::db::sea_orm;

use tardis::web::poem_openapi;

use crate::rbum::rbum_enumeration::{RbumDataTypeKind, RbumScopeLevelKind, RbumWidgetTypeKind};

/// Add request for resource kind attribute definition
///
/// 资源类型属性定义添加请求
#[derive(Serialize, Deserialize, Debug)]
#[derive(poem_openapi::Object)]
pub struct RbumKindAttrAddReq {
    /// Attribute definition module
    ///
    /// 属性定义模块
    ///
    /// Default is ``empty``
    ///
    /// 默认为 ``空``
    ///
    /// Used to distinguish different instances of the same resource kind.
    /// For example, the ``user`` kind resource, different tenants can have different Attribute definitions.
    ///
    /// 用于区别使用同一资源类型的不同实例。比如 ``用户`` 类型的资源，不同的租户可以有不同的属性定义。
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub module: Option<TrimString>,
    /// Attribute definition name
    ///
    /// 属性定义名称
    ///
    /// Corresponds to the field name, such as ``<input name=$name />`` .
    ///
    /// 多对应于字段名，如 ``<input name=$name />`` 。
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: TrimString,
    /// Attribute definition label
    ///
    /// 属性定义标签
    ///
    /// Corresponds to the field label, such as ``<label for=$name>$label</label>`` .
    ///
    /// 多对应于字段标签，如 ``<label for="$name">$label</label>`` 。
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub label: String,
    /// Attribute definition note
    ///
    /// 属性定义备注
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub note: Option<String>,
    /// Attribute definition sort
    ///
    /// 属性定义排序
    pub sort: Option<i64>,
    /// 是否定位类型的属性定义
    ///
    /// Whether the Attribute is a positioning kind
    ///
    /// Default is ``false``
    ///
    /// 默认为 ``false``
    ///
    /// Positioning kind extension attributes are used to assist in locating resources other than ID,
    /// such as ``ID card`` and ``work number``. Generally, an index will be added.
    ///
    /// 定位类型的属性用于除ID外辅助定位资源，比如 ``身份证`` ``工号`` 。 一般而言会加索引。
    pub position: Option<bool>,
    /// Whether the Attribute definition is a capacity kind
    ///
    /// 是否容量类型的属性定义
    ///
    /// Default is ``false``
    ///
    /// 默认为 ``false``
    ///
    /// Capacity kind extension attributes are used to store capacity information, such as ``disk capacity`` and ``memory capacity``.
    /// These attributes can be "consumed" and are often used in conjunction with [`crate::rbum::domain::rbum_rel::Model`] and [`crate::rbum::domain::rbum_rel_attr::Model`],
    /// the latter of which can record the consumed capacity.
    ///
    /// 容量类型的属性用于存储容量信息，如 ``磁盘容量`` ``内存容量`` 。
    /// 这些属性是可以被“消耗”的，多与 [`crate::rbum::domain::rbum_rel::Model`] 及 [`crate::rbum::domain::rbum_rel_attr::Model`] 配合使用，后者可以记录消耗的容量。
    pub capacity: Option<bool>,
    /// Whether overload is allowed
    ///
    /// 是否允许超载
    ///
    /// Default is ``false``
    ///
    /// 默认为 ``false``
    ///
    /// This attribute is only valid when ``capacity = true``, and is used to indicate whether the capacity is allowed to be overloaded.
    ///
    /// 此属性仅当 ``capacity = true``` 时有效，用于表示容量是否允许超载。
    pub overload: Option<bool>,
    /// Whether it is a secret
    ///
    /// 是否是秘密
    ///
    /// Default is ``false``
    ///
    /// 默认为 ``false``
    ///
    /// If ``true``, the value corresponding to this attribute will not be returned to the front end, but will only be processed on the server.
    /// This attribute can ensure the security of attribute data.
    ///
    /// 当为 ``true`` 时属性对应的值不会返回给前端，仅在服务端处理。此属性可以保证属性数据安全。
    pub secret: Option<bool>,
    /// Whether it is the main column
    ///
    /// 是否是主列
    ///
    /// Default is ``false``
    ///
    /// 默认为 ``false``
    ///
    /// If ``true``, it means that the attribute is the field corresponding to [`crate::rbum::domain::rbum_kind::Model::ext_table_name`] table,
    /// otherwise the attribute value will be stored in the [`crate::rbum::domain::rbum_item_attr::Model`] table.
    ///
    /// 当为 ``true`` 时表示该属性是 [`crate::rbum::domain::rbum_kind::Model::ext_table_name`] 表对应的字段，
    /// 否则会将该属性值存储在 [`crate::rbum::domain::rbum_item_attr::Model`] 表中。
    pub main_column: Option<bool>,
    /// Whether indexing is needed
    ///
    /// 是否需要索引
    ///
    /// Default is ``false``
    ///
    /// 默认为 ``false``
    ///
    /// This attribute is only valid when ``main_column = true``, used to indicate whether the attribute needs to be indexed.
    ///
    /// 此属性仅当 ``main_column = true`` 时有效，用于表示是否需要对该属性进行索引。
    pub idx: Option<bool>,
    /// Data kind
    ///
    /// 数据类型
    pub data_type: RbumDataTypeKind,
    /// Show widget kind
    ///
    /// 显示控件类型
    pub widget_type: RbumWidgetTypeKind,
    /// Number of columns occupied by the widget
    ///
    /// 控件占用列数
    ///
    /// Default is ``0``, indicating self-adaptation
    ///
    /// 默认为 ``0`` ， 表示自适应
    #[oai(validator(minimum(value = "0", exclusive = "false")))]
    pub widget_columns: Option<i16>,
    /// Whether to hide by default
    ///
    /// 默认是否隐藏
    ///
    /// Default is ``false``
    ///
    /// 默认为 ``false``
    ///
    /// If ``true``, the value corresponding to this attribute will be returned to the front end, but will not be displayed.
    /// This attribute is often used for internal processing on the front end.
    /// This attribute cannot guarantee the security of attribute data.
    ///
    /// 当为 ``true`` 时该属性对应的值会返给前端，但不会显示。多用于前端内部处理。此属性不能保证属性数据安全。
    pub hide: Option<bool>,
    /// Show conditions
    ///
    /// 显示条件
    ///
    /// Json format: ``{<attribute name>:<attribute value>}``, currently only supports ``and`` operations.
    ///
    /// Json格式：``{<属性名>:<属性值>}``，目前仅支持``and``操作。
    pub show_by_conds: Option<String>,
    /// Fixed default value
    ///
    /// 固定默认值
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub default_value: Option<String>,
    /// Dynamic default value
    ///
    /// 动态默认值
    ///
    /// It can be a URL (with placeholders) or a set of placeholders.
    /// Placeholders are wrapped in ``{...}``, and the corresponding values can come from other current attribute values or incoming context variables.
    ///
    /// 可以是一个URL（允许有占位符）或是一组占位符。占位符使用 ``{...}``包裹，对应的值可以来自当前的其它属性值、传入的上下文变量。
    ///
    /// The return format is the same as ``default_value``
    /// or ``json`` when ``data_type = Json``` and ``widget_type = Control``
    /// or ``array`` when ``data_type = Array`` and ``widget_type = Group``.
    ///
    /// 返回格式与 ``default_value`` 一致，
    /// 或当 ``data_type = Json`` 且 ``widget_type = Control`` 时为 ``json`` ，
    /// 或当 ``data_type = Array`` 且 ``widget_type = Group`` 时为 ``array``。
    pub dyn_default_value: Option<String>,
    /// Fixed options
    ///
    /// 固定选项
    ///
    /// Json array format: ``[{name:<display name>:value:<corresponding value>}]``.
    ///
    /// Json数组格式：``[{name:<显示名称>:value:<对应值>}]``。
    pub options: Option<String>,
    /// Dynamic options
    ///
    /// 动态选项
    ///
    /// It can be a URL (with placeholders) or a set of placeholders.
    /// Placeholders are wrapped in ``{...}``, and the corresponding values can come from other current attribute values or incoming context variables.
    ///
    /// 可以是一个URL（允许有占位符）或是一组占位符。占位符使用 ``{...}``包裹，对应的值可以来自当前的其它属性值、传入的上下文变量。
    ///
    /// the return format is the same as `options`.
    ///
    /// 返回格式与 `options` 一致。
    pub dyn_options: Option<String>,
    /// Whether it is required
    ///
    /// 是否必填
    ///
    /// Default is ``false``
    ///
    /// 默认为 ``false``
    pub required: Option<bool>,
    /// Minimum length
    ///
    /// 最小长度
    ///
    /// Default is ``0``, indicating no limit
    ///
    /// 默认为 ``0`` ， 表示不限制
    #[oai(validator(minimum(value = "0", exclusive = "false")))]
    pub min_length: Option<i32>,
    /// Maximum length
    ///
    /// 最大长度
    ///
    /// Default is ``0``, indicating no limit
    ///
    /// 默认为 ``0`` ， 表示不限制
    #[oai(validator(minimum(value = "0", exclusive = "false")))]
    pub max_length: Option<i32>,
    /// Parent attribute name
    ///
    /// 父属性名称
    ///
    /// Used to implement multi-level attributes.
    ///
    /// 用于实现多级属性。
    pub parent_attr_name: Option<TrimString>,
    /// Custom behavior
    ///
    /// 自定义行为
    ///
    /// For example: user selection function, role selection function, etc.
    /// Custom behavior needs to be bound to the corresponding function code.
    ///
    /// 例如：用户选择函数、角色选择函数等。
    /// 自定义行为需要绑定到对应的函数代码。
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub action: Option<String>,
    /// Extension information
    ///
    /// 扩展信息
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub ext: Option<String>,
    /// Associated [resource kind](crate::rbum::dto::rbum_kind_dto::RbumKindDetailResp) id
    ///
    /// 关联的 [资源类型](crate::rbum::dto::rbum_kind_dto::RbumKindDetailResp) id
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub rel_rbum_kind_id: String,

    pub scope_level: Option<RbumScopeLevelKind>,
}

/// Modify request for resource kind attribute definition
///
/// 资源类型属性定义修改请求
#[derive(Serialize, Deserialize, Debug)]
#[derive(poem_openapi::Object)]
pub struct RbumKindAttrModifyReq {
    /// Attribute definition label
    ///
    /// 属性定义标签
    ///
    /// Corresponds to the field label, such as ``<label for=$name>$label</label>`` .
    ///
    /// 多对应于字段标签，如 ``<label for="$name">$label</label>`` 。
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub label: Option<String>,
    /// Attribute definition note
    ///
    /// 属性定义备注
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub note: Option<String>,
    /// Attribute definition sort
    ///
    /// 属性定义排序
    pub sort: Option<i64>,
    /// 是否定位类型的属性定义
    ///
    /// Whether the Attribute is a positioning kind
    ///
    /// Positioning kind extension attributes are used to assist in locating resources other than ID,
    /// such as ``ID card`` and ``work number``. Generally, an index will be added.
    ///
    /// 定位类型的属性用于除ID外辅助定位资源，比如 ``身份证`` ``工号`` 。 一般而言会加索引。
    pub position: Option<bool>,
    /// Whether the Attribute definition is a capacity kind
    ///
    /// 是否容量类型的属性定义
    ///
    /// Capacity kind extension attributes are used to store capacity information, such as ``disk capacity`` and ``memory capacity``.
    /// These attributes can be "consumed" and are often used in conjunction with [`crate::rbum::domain::rbum_rel::Model`] and [`crate::rbum::domain::rbum_rel_attr::Model`],
    /// the latter of which can record the consumed capacity.
    ///
    /// 容量类型的属性用于存储容量信息，如 ``磁盘容量`` ``内存容量`` 。
    /// 这些属性是可以被“消耗”的，多与 [`crate::rbum::domain::rbum_rel::Model`] 及 [`crate::rbum::domain::rbum_rel_attr::Model`] 配合使用，后者可以记录消耗的容量。
    pub capacity: Option<bool>,
    /// Whether overload is allowed
    ///
    /// 是否允许超载
    ///
    /// This attribute is only valid when ``capacity = true``, and is used to indicate whether the capacity is allowed to be overloaded.
    ///
    /// 此属性仅当 ``capacity = true``` 时有效，用于表示容量是否允许超载。
    pub overload: Option<bool>,
    /// Whether it is a secret
    ///
    /// 是否是秘密
    ///
    /// If ``true``, the value corresponding to this attribute will not be returned to the front end, but will only be processed on the server.
    /// This attribute can ensure the security of attribute data.
    ///
    /// 当为 ``true`` 时属性对应的值不会返回给前端，仅在服务端处理。此属性可以保证属性数据安全。
    pub secret: Option<bool>,
    /// Whether it is the main column
    ///
    /// 是否是主列
    ///
    /// If ``true``, it means that the attribute is the field corresponding to [`crate::rbum::domain::rbum_kind::Model::ext_table_name`] table,
    /// otherwise the attribute value will be stored in the [`crate::rbum::domain::rbum_item_attr::Model`] table.
    ///
    /// 当为 ``true`` 时表示该属性是 [`crate::rbum::domain::rbum_kind::Model::ext_table_name`] 表对应的字段，
    /// 否则会将该属性值存储在 [`crate::rbum::domain::rbum_item_attr::Model`] 表中。
    pub main_column: Option<bool>,
    /// Whether indexing is needed
    ///
    /// 是否需要索引
    ///
    /// This attribute is only valid when ``main_column = true``, used to indicate whether the attribute needs to be indexed.
    ///
    /// 此属性仅当 ``main_column = true`` 时有效，用于表示是否需要对该属性进行索引。
    pub idx: Option<bool>,
    /// Data kind
    ///
    /// 数据类型
    pub data_type: Option<RbumDataTypeKind>,
    /// Show widget kind
    ///
    /// 显示控件类型
    pub widget_type: Option<RbumWidgetTypeKind>,
    /// Number of columns occupied by the widget
    ///
    /// 控件占用列数
    ///
    /// Default is ``0``, indicating self-adaptation
    ///
    /// 默认为 ``0`` ， 表示自适应
    #[oai(validator(minimum(value = "0", exclusive = "false")))]
    pub widget_columns: Option<i16>,
    /// Whether to hide by default
    ///
    /// 默认是否隐藏
    ///
    /// If ``true``, the value corresponding to this attribute will be returned to the front end, but will not be displayed.
    /// This attribute is often used for internal processing on the front end.
    /// This attribute cannot guarantee the security of attribute data.
    ///
    /// 当为 ``true`` 时该属性对应的值会返给前端，但不会显示。多用于前端内部处理。此属性不能保证属性数据安全。
    pub hide: Option<bool>,
    /// Show conditions
    ///
    /// 显示条件
    ///
    /// Json format: ``{<attribute name>:<attribute value>}``, currently only supports ``and`` operations.
    ///
    /// Json格式：``{<属性名>:<属性值>}``，目前仅支持``and``操作。
    pub show_by_conds: Option<String>,
    /// Fixed default value
    ///
    /// 固定默认值
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub default_value: Option<String>,
    /// Dynamic default value
    ///
    /// 动态默认值
    ///
    /// It can be a URL (with placeholders) or a set of placeholders.
    /// Placeholders are wrapped in ``{...}``, and the corresponding values can come from other current attribute values or incoming context variables.
    ///
    /// 可以是一个URL（允许有占位符）或是一组占位符。占位符使用 ``{...}``包裹，对应的值可以来自当前的其它属性值、传入的上下文变量。
    ///
    /// The return format is the same as ``default_value``
    /// or ``json`` when ``data_type = Json``` and ``widget_type = Control``
    /// or ``array`` when ``data_type = Array`` and ``widget_type = Group``.
    ///
    /// 返回格式与 ``default_value`` 一致，
    /// 或当 ``data_type = Json`` 且 ``widget_type = Control`` 时为 ``json`` ，
    /// 或当 ``data_type = Array`` 且 ``widget_type = Group`` 时为 ``array``。
    pub dyn_default_value: Option<String>,
    /// Fixed options
    ///
    /// 固定选项
    ///
    /// Json array format: ``[{name:<display name>:value:<corresponding value>}]``.
    ///
    /// Json数组格式：``[{name:<显示名称>:value:<对应值>}]``。
    pub options: Option<String>,
    /// Dynamic options
    ///
    /// 动态选项
    ///
    /// It can be a URL (with placeholders) or a set of placeholders.
    /// Placeholders are wrapped in ``{...}``, and the corresponding values can come from other current attribute values or incoming context variables.
    ///
    /// 可以是一个URL（允许有占位符）或是一组占位符。占位符使用 ``{...}``包裹，对应的值可以来自当前的其它属性值、传入的上下文变量。
    ///
    /// the return format is the same as `options`.
    ///
    /// 返回格式与 `options` 一致。
    pub dyn_options: Option<String>,
    /// Whether it is required
    ///
    /// 是否必填
    pub required: Option<bool>,
    /// Minimum length
    ///
    /// 最小长度
    #[oai(validator(minimum(value = "0", exclusive = "false")))]
    pub min_length: Option<i32>,
    /// Maximum length
    ///
    /// 最大长度
    #[oai(validator(minimum(value = "0", exclusive = "false")))]
    pub max_length: Option<i32>,
    /// Parent attribute name
    ///
    /// 父属性名称
    ///
    /// Used to implement multi-level attributes.
    ///
    /// 用于实现多级属性。
    pub parent_attr_name: Option<TrimString>,
    /// Custom behavior
    ///
    /// 自定义行为
    ///
    /// For example: user selection function, role selection function, etc.
    /// Custom behavior needs to be bound to the corresponding function code.
    ///
    /// 例如：用户选择函数、角色选择函数等。
    /// 自定义行为需要绑定到对应的函数代码。
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub action: Option<String>,
    /// Extension information
    ///
    /// 扩展信息
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub ext: Option<String>,

    pub scope_level: Option<RbumScopeLevelKind>,
}

/// Resource kind attribute definition summary information
///
/// 资源类型属性定义概要信息
#[derive(Serialize, Deserialize, Debug)]
#[derive(poem_openapi::Object, sea_orm::FromQueryResult)]
pub struct RbumKindAttrSummaryResp {
    /// Attribute definition id
    ///
    /// 属性定义id
    pub id: String,
    /// Attribute definition module
    ///
    /// 属性定义模块
    pub module: String,
    /// Attribute definition name
    ///
    /// 属性定义名称
    pub name: String,
    /// Attribute definition label
    ///
    /// 属性定义标签
    pub label: String,
    /// Attribute definition note
    ///
    /// 属性定义备注
    pub note: String,
    /// Attribute definition sort
    ///
    /// 属性定义排序
    pub sort: i64,
    /// 是否定位类型的属性定义
    ///
    /// Whether the Attribute is a positioning kind
    pub position: bool,
    /// Whether the Attribute definition is a capacity kind
    ///
    /// 是否容量类型的属性定义
    pub capacity: bool,
    /// Whether overload is allowed
    ///
    /// 是否允许超载
    pub overload: bool,
    /// Whether it is a secret
    ///
    /// 是否是秘密
    pub secret: bool,
    /// Whether it is the main column
    ///
    /// 是否是主列
    pub main_column: bool,
    /// Whether indexing is needed
    ///
    /// 是否需要索引
    pub idx: bool,
    /// Data kind
    ///
    /// 数据类型
    pub data_type: RbumDataTypeKind,
    /// Show widget kind
    ///
    /// 显示控件类型
    pub widget_type: RbumWidgetTypeKind,
    /// Number of columns occupied by the widget
    ///
    /// 控件占用列数
    pub widget_columns: i16,
    /// Whether to hide by default
    ///
    /// 默认是否隐藏
    pub hide: bool,
    /// Show conditions
    ///
    /// 显示条件
    ///
    /// Json format: ``{<attribute name>:<attribute value>}``, currently only supports ``and`` operations.
    ///
    /// Json格式：``{<属性名>:<属性值>}``，目前仅支持``and``操作。
    pub show_by_conds: String,
    /// Fixed default value
    ///
    /// 固定默认值
    pub default_value: String,
    /// Dynamic default value
    ///
    /// 动态默认值
    ///
    /// Placeholders are wrapped in ``{...}``, and the corresponding values can come from other current attribute values or incoming context variables.
    ///
    /// 可以是一个URL（允许有占位符）或是一组占位符。占位符使用 ``{...}``包裹，对应的值可以来自当前的其它属性值、传入的上下文变量。
    ///
    /// The return format is the same as ``default_value``
    /// or ``json`` when ``data_type = Json``` and ``widget_type = Control``
    /// or ``array`` when ``data_type = Array`` and ``widget_type = Group``.
    ///
    /// 返回格式与 ``default_value`` 一致，
    /// 或当 ``data_type = Json`` 且 ``widget_type = Control`` 时为 ``json`` ，
    /// 或当 ``data_type = Array`` 且 ``widget_type = Group`` 时为 ``array``。
    pub dyn_default_value: String,
    /// Fixed options
    ///
    /// 固定选项
    ///
    /// Json array format: ``[{name:<display name>:value:<corresponding value>}]``.
    ///
    /// Json数组格式：``[{name:<显示名称>:value:<对应值>}]``。
    pub options: String,
    /// Dynamic options
    ///
    /// 动态选项
    ///
    /// It can be a URL (with placeholders) or a set of placeholders.
    /// Placeholders are wrapped in ``{...}``, and the corresponding values can come from other current attribute values or incoming context variables.
    ///
    /// 可以是一个URL（允许有占位符）或是一组占位符。占位符使用 ``{...}``包裹，对应的值可以来自当前的其它属性值、传入的上下文变量。
    ///
    /// the return format is the same as `options`.
    ///
    /// 返回格式与 `options` 一致。
    pub dyn_options: String,
    /// Whether it is required
    ///
    /// 是否必填
    pub required: bool,
    /// Minimum length
    ///
    /// 最小长度
    pub min_length: i32,
    /// Maximum length
    ///
    /// 最大长度
    pub max_length: i32,
    /// Parent attribute name
    ///
    /// 父属性名称
    pub parent_attr_name: String,
    /// Custom behavior
    ///
    /// 自定义行为
    pub action: String,
    /// Extension information
    ///
    /// 扩展信息
    pub ext: String,
    /// Associated [resource kind](crate::rbum::dto::rbum_kind_dto::RbumKindDetailResp) id
    ///
    /// 关联的 [资源类型](crate::rbum::dto::rbum_kind_dto::RbumKindDetailResp) id
    pub rel_rbum_kind_id: String,

    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
}

/// Resource kind attribute definition detail information
///
/// 资源类型属性定义详细信息
#[derive(Serialize, Deserialize, Debug)]
#[derive(poem_openapi::Object, sea_orm::FromQueryResult)]
pub struct RbumKindAttrDetailResp {
    /// Attribute definition id
    ///
    /// 属性定义id
    pub id: String,
    /// Attribute definition module
    ///
    /// 属性定义模块
    pub module: String,
    /// Attribute definition module
    ///
    /// 属性定义模块
    pub name: String,
    /// Attribute definition label
    ///
    /// 属性定义标签
    pub label: String,
    /// Attribute definition note
    ///
    /// 属性定义备注
    pub note: String,
    /// Attribute definition sort
    ///
    /// 属性定义排序
    pub sort: i64,
    /// 是否定位类型的属性定义
    ///
    /// Whether the Attribute is a positioning kind
    pub position: bool,
    /// Whether the Attribute definition is a capacity kind
    ///
    /// 是否容量类型的属性定义
    pub capacity: bool,
    /// Whether overload is allowed
    ///
    /// 是否允许超载
    pub overload: bool,
    /// Whether it is a secret
    ///
    /// 是否是秘密
    pub secret: bool,
    /// Whether it is the main column
    ///
    /// 是否是主列
    pub main_column: bool,
    /// Whether indexing is needed
    ///
    /// 是否需要索引
    pub idx: bool,
    /// Data kind
    ///
    /// 数据类型
    pub data_type: RbumDataTypeKind,
    /// Show widget kind
    ///
    /// 显示控件类型
    pub widget_type: RbumWidgetTypeKind,
    /// Number of columns occupied by the widget
    ///
    /// 控件占用列数
    pub widget_columns: i16,
    /// Whether to hide by default
    ///
    /// 默认是否隐藏
    pub hide: bool,
    /// Show conditions
    ///
    /// 显示条件
    ///
    /// Json format: ``{<attribute name>:<attribute value>}``, currently only supports ``and`` operations.
    ///
    /// Json格式：``{<属性名>:<属性值>}``，目前仅支持``and``操作。
    pub show_by_conds: String,
    /// Fixed default value
    ///
    /// 固定默认值
    pub default_value: String,
    /// Dynamic default value
    ///
    /// 动态默认值
    ///
    /// Placeholders are wrapped in ``{...}``, and the corresponding values can come from other current attribute values or incoming context variables.
    ///
    /// 可以是一个URL（允许有占位符）或是一组占位符。占位符使用 ``{...}``包裹，对应的值可以来自当前的其它属性值、传入的上下文变量。
    ///
    /// The return format is the same as ``default_value``
    /// or ``json`` when ``data_type = Json``` and ``widget_type = Control``
    /// or ``array`` when ``data_type = Array`` and ``widget_type = Group``.
    ///
    /// 返回格式与 ``default_value`` 一致，
    /// 或当 ``data_type = Json`` 且 ``widget_type = Control`` 时为 ``json`` ，
    /// 或当 ``data_type = Array`` 且 ``widget_type = Group`` 时为 ``array``。
    pub dyn_default_value: String,
    /// Fixed options
    ///
    /// 固定选项
    ///
    /// Json array format: ``[{name:<display name>:value:<corresponding value>}]``.
    ///
    /// Json数组格式：``[{name:<显示名称>:value:<对应值>}]``。
    pub options: String,
    /// Dynamic options
    ///
    /// 动态选项
    ///
    /// It can be a URL (with placeholders) or a set of placeholders.
    /// Placeholders are wrapped in ``{...}``, and the corresponding values can come from other current attribute values or incoming context variables.
    ///
    /// 可以是一个URL（允许有占位符）或是一组占位符。占位符使用 ``{...}``包裹，对应的值可以来自当前的其它属性值、传入的上下文变量。
    ///
    /// the return format is the same as `options`.
    ///
    /// 返回格式与 `options` 一致。
    pub dyn_options: String,
    /// Whether it is required
    ///
    /// 是否必填
    pub required: bool,
    /// Minimum length
    ///
    /// 最小长度
    pub min_length: i32,
    /// Maximum length
    ///
    /// 最大长度
    pub max_length: i32,
    /// Parent attribute name
    ///
    /// 父属性名称
    pub parent_attr_name: String,
    /// Custom behavior
    ///
    /// 自定义行为
    pub action: String,
    /// Extension information
    ///
    /// 扩展信息
    pub ext: String,
    /// Associated [resource kind](crate::rbum::dto::rbum_kind_dto::RbumKindDetailResp) id
    ///
    /// 关联的 [资源类型](crate::rbum::dto::rbum_kind_dto::RbumKindDetailResp) id
    pub rel_rbum_kind_id: String,
    /// Associated [resource kind](crate::rbum::dto::rbum_kind_dto::RbumKindDetailResp) name
    ///
    /// 关联的 [资源类型](crate::rbum::dto::rbum_kind_dto::RbumKindDetailResp) name
    pub rel_rbum_kind_name: String,

    pub own_paths: String,
    pub owner: String,
    pub owner_name: Option<String>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
}
