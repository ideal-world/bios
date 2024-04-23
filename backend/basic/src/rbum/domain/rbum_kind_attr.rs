use tardis::basic::dto::TardisContext;
use tardis::chrono::{self, Utc};
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm;
use tardis::db::sea_orm::prelude::*;
use tardis::db::sea_orm::sea_query::{ColumnDef, IndexCreateStatement, Table, TableCreateStatement};
use tardis::db::sea_orm::*;
use tardis::TardisCreateIndex;

/// Resource kind attribute definition model
///
/// 资源类型属性定义模型
///
/// General logic for dynamic request processing:
///
/// 1. dynamic values take precedence over static values
/// 1. supports calling http to get data with GET request
/// 1. request url supports attribute variable substitution, format is: ``{attribute name}``
/// 1. if no attribute variable substitution exists and ``secret = false`` , the url is called directly and the corresponding value is returned
/// 1. if attribute variable substitution exists, then：
///     1. extract all attribute variables to be replaced
///     1. monitor changes of these attributes
///     1. substitute attribute variables with values into the url
///     1. if no longer an attribute variable substitution in the url and ``secret = false`` , call the url and return the corresponding value
/// 1. before the resource object is saved, if ``secret = true`` and an attribute variable substitution in the url, call the url and return the corresponding value
///
/// For security reasons, the last step must be completed by the server.
///
/// 动态请求的通用逻辑处理：
///
/// 1. 动态值优先于静态值
/// 1. 支持调用http获取数据，请求方式为GET
/// 1. 请求url支持属性变量替换，格式为：``{属性名}``
/// 1. 如果没有属性变量替换存在且 ``secret = false`` ，则直接调用url，返回对应值
/// 1. 如果存在属性变量替换，则：
///     1. 提取所有需要替换的属性变量
///     1. 监听这些属性的变化
///     1. 将属性变量替换为值后，替换到url中
///     1. 如果url中不再存在属性变量替换且 ``secret = false`` ，则调用url，返回对应值
/// 1. 在保存资源对象之前，如果 ``secret = true`` 且url中存在属性变量替换，则调用url，返回对应值
///
/// 为了安全起见，最后一步须由服务端完成。
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, TardisCreateIndex)]
#[sea_orm(table_name = "rbum_kind_attr")]
pub struct Model {
    /// Attribute definition id
    ///
    /// 属性定义id
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    /// Attribute definition module
    ///
    /// 属性定义模块
    /// 
    /// Used to distinguish different instances of the same resource kind. 
    /// For example, the ``user`` kind resource, different tenants can have different Attribute definitions.
    ///
    /// 用于区别使用同一资源类型的不同实例。比如 ``用户`` 类型的资源，不同的租户可以有不同的属性定义。
    #[index(index_id = "id")]
    pub module: String,
    /// Attribute definition name
    ///
    /// 属性定义名称
    ///
    /// Corresponds to the field name, such as ``<input name=$name />`` .
    ///
    /// 多对应于字段名，如 ``<input name=$name />`` 。
    #[index(index_id = "id")]
    pub name: String,
    /// Attribute definition label
    ///
    /// 属性定义标签
    ///
    /// Corresponds to the field label, such as ``<label for=$name>$label</label>`` .
    ///
    /// 多对应于字段标签，如 ``<label for="$name">$label</label>`` 。
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
    ///
    /// Positioning kind extension attributes are used to assist in locating resources other than ID,
    /// such as ``ID card`` and ``work number``. Generally, an index will be added.
    ///
    /// 定位类型的属性用于除ID外辅助定位资源，比如 ``身份证`` ``工号`` 。 一般而言会加索引。
    pub position: bool,
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
    pub capacity: bool,
    /// Whether overload is allowed
    ///
    /// 是否允许超载
    ///
    /// This attribute is only valid when ``capacity = true``, and is used to indicate whether the capacity is allowed to be overloaded.
    ///
    /// 此属性仅当 ``capacity = true``` 时有效，用于表示容量是否允许超载。
    pub overload: bool,
    /// Whether it is a secret
    ///
    /// 是否是秘密
    ///
    /// If ``true``, the value corresponding to this attribute will not be returned to the front end, but will only be processed on the server.
    /// This attribute can ensure the security of attribute data.
    ///
    /// 当为 ``true`` 时属性对应的值不会返回给前端，仅在服务端处理。此属性可以保证属性数据安全。
    pub secret: bool,
    /// Whether it is the main column
    ///
    /// 是否是主列
    ///
    /// If ``true``, it means that the attribute is the field corresponding to [`crate::rbum::domain::rbum_kind::Model::ext_table_name`] table,
    /// otherwise the attribute value will be stored in the [`crate::rbum::domain::rbum_item_attr::Model`] table.
    ///
    /// 当为 ``true`` 时表示该属性是 [`crate::rbum::domain::rbum_kind::Model::ext_table_name`] 表对应的字段，
    /// 否则会将该属性值存储在 [`crate::rbum::domain::rbum_item_attr::Model`] 表中。
    pub main_column: bool,
    /// Whether indexing is needed
    ///
    /// 是否需要索引
    ///
    /// This attribute is only valid when ``main_column = true``, used to indicate whether the attribute needs to be indexed.
    ///
    /// 此属性仅当 ``main_column = true`` 时有效，用于表示是否需要对该属性进行索引。
    pub idx: bool,
    /// Data kind
    ///
    /// 数据类型
    ///
    /// Associated [resource kind](crate::rbum::rbum_enumeration::RbumDataTypeKind)
    pub data_type: String,
    /// Show widget kind
    ///
    /// 显示控件类型
    ///
    /// Associated [resource kind](crate::rbum::rbum_enumeration::RbumWidgetTypeKind)
    pub widget_type: String,
    /// Number of columns occupied by the widget
    ///
    /// 控件占用列数
    pub widget_columns: i16,
    /// Whether to hide by default
    ///
    /// 默认是否隐藏
    ///
    /// If ``true``, the value corresponding to this attribute will be returned to the front end, but will not be displayed.
    /// This attribute is often used for internal processing on the front end.
    /// This attribute cannot guarantee the security of attribute data.
    ///
    /// 当为 ``true`` 时该属性对应的值会返给前端，但不会显示。多用于前端内部处理。此属性不能保证属性数据安全。
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
    ///
    /// Used to implement multi-level attributes.
    ///
    /// 用于实现多级属性。
    pub parent_attr_name: String,
    /// Custom behavior
    ///
    /// 自定义行为
    ///
    /// For example: user selection function, role selection function, etc.
    /// Custom behavior needs to be bound to the corresponding function code.
    ///
    /// 例如：用户选择函数、角色选择函数等。
    /// 自定义行为需要绑定到对应的函数代码。
    pub action: String,
    /// Extension information
    ///
    /// 扩展信息
    pub ext: String,
    /// Associated [resource kind](crate::rbum::domain::rbum_kind::Model) id
    ///
    /// 关联的 [资源类型](crate::rbum::domain::rbum_kind::Model) id
    #[index(index_id = "id")]
    pub rel_rbum_kind_id: String,

    pub scope_level: i16,

    pub own_paths: String,
    pub owner: String,
    pub create_time: chrono::DateTime<Utc>,
    pub update_time: chrono::DateTime<Utc>,
    pub create_by: String,
    pub update_by: String,
}

impl TardisActiveModel for ActiveModel {
    fn fill_ctx(&mut self, ctx: &TardisContext, is_insert: bool) {
        if is_insert {
            self.own_paths = Set(ctx.own_paths.to_string());
            self.owner = Set(ctx.owner.to_string());
            self.create_by = Set(ctx.owner.to_string());
        }
        self.update_by = Set(ctx.owner.to_string());
    }

    fn create_table_statement(db: DbBackend) -> TableCreateStatement {
        let mut builder = Table::create();
        builder
            .table(Entity.table_ref())
            .if_not_exists()
            .col(ColumnDef::new(Column::Id).not_null().string().primary_key())
            // Specific
            .col(ColumnDef::new(Column::Name).not_null().string())
            .col(ColumnDef::new(Column::Module).not_null().string())
            .col(ColumnDef::new(Column::Label).not_null().string())
            .col(ColumnDef::new(Column::Note).not_null().string().default(""))
            .col(ColumnDef::new(Column::Sort).not_null().big_integer())
            .col(ColumnDef::new(Column::MainColumn).not_null().boolean().default(false))
            .col(ColumnDef::new(Column::Position).not_null().boolean().default(false))
            .col(ColumnDef::new(Column::Capacity).not_null().boolean().default(false))
            .col(ColumnDef::new(Column::Hide).not_null().boolean().default(false))
            .col(ColumnDef::new(Column::Secret).not_null().boolean().default(false))
            .col(ColumnDef::new(Column::ShowByConds).not_null().string().default(""))
            .col(ColumnDef::new(Column::Overload).not_null().boolean().default(false))
            .col(ColumnDef::new(Column::Idx).not_null().boolean().default(false))
            .col(ColumnDef::new(Column::DataType).not_null().string())
            .col(ColumnDef::new(Column::WidgetType).not_null().string())
            .col(ColumnDef::new(Column::WidgetColumns).not_null().small_integer().default(1))
            .col(ColumnDef::new(Column::DefaultValue).not_null().string().default(""))
            .col(ColumnDef::new(Column::DynDefaultValue).not_null().string().default(""))
            .col(ColumnDef::new(Column::Options).not_null().text())
            .col(ColumnDef::new(Column::DynOptions).not_null().string().default(""))
            .col(ColumnDef::new(Column::Required).not_null().boolean().default(false))
            .col(ColumnDef::new(Column::MinLength).not_null().integer())
            .col(ColumnDef::new(Column::MaxLength).not_null().integer())
            .col(ColumnDef::new(Column::ParentAttrName).not_null().string().default(""))
            .col(ColumnDef::new(Column::Action).not_null().string().default(""))
            .col(ColumnDef::new(Column::Ext).not_null().string().default(""))
            .col(ColumnDef::new(Column::RelRbumKindId).not_null().string())
            // Basic
            .col(ColumnDef::new(Column::OwnPaths).not_null().string())
            .col(ColumnDef::new(Column::Owner).not_null().string())
            // With Scope
            .col(ColumnDef::new(Column::ScopeLevel).not_null().small_integer())
            .col(ColumnDef::new(Column::CreateBy).not_null().string())
            .col(ColumnDef::new(Column::UpdateBy).not_null().string());
        if db == DatabaseBackend::Postgres {
            builder
                .col(ColumnDef::new(Column::CreateTime).extra("DEFAULT CURRENT_TIMESTAMP".to_string()).timestamp_with_time_zone())
                .col(ColumnDef::new(Column::UpdateTime).extra("DEFAULT CURRENT_TIMESTAMP".to_string()).timestamp_with_time_zone());
        } else {
            builder
                .engine("InnoDB")
                .character_set("utf8mb4")
                .collate("utf8mb4_0900_as_cs")
                .col(ColumnDef::new(Column::CreateTime).extra("DEFAULT CURRENT_TIMESTAMP".to_string()).timestamp())
                .col(ColumnDef::new(Column::UpdateTime).extra("DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP".to_string()).timestamp());
        }
        builder.to_owned()
    }

    fn create_index_statement() -> Vec<IndexCreateStatement> {
        tardis_create_index_statement()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
