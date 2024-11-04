use serde::{Deserialize, Serialize};
use tardis::{
    chrono::{DateTime, Utc},
    db::sea_orm,
    web::poem_openapi,
};

use crate::stats_enumeration::{StatsDataTypeKind, StatsFactColKind};

/// Add Dimension Configuration Request Object
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsConfDimAddReq {
    /// The primary key or encoding passed in from the external system
    ///
    /// 外部系统传入的主键或编码
    #[oai(validator(pattern = r"^[a-z0-9_]+$"))]
    pub key: String,
    /// The name of the dimension
    ///
    /// 维度名称
    #[oai(validator(min_length = "2"))]
    pub show_name: String,

    /// Whether it is a stable dataset,
    /// if true the dimension data is recorded in the corresponding dimension table,
    /// if false the dimension data is recorded in the fact table
    ///
    /// 是否为稳定数据集，
    /// 如果为true，则维度数据记录在对应的维度表中，
    /// 如果为false，则维度数据记录在事实表中
    pub stable_ds: bool,

    /// Dimension data type
    ///
    /// 维度数据类型
    pub data_type: StatsDataTypeKind,

    /// Hierarchy, starting from 0, for up-rolls/down-rolls.
    /// Multiple fields can be defined for each dimension.
    /// e.g. address dimension can be province-city-district, etc.
    ///
    /// 层级，从0开始，用于上卷/下卷。
    /// 每个维度可以定义多个字段。
    /// 例如地址维度可以是省-市-区等
    pub hierarchy: Option<Vec<String>>,
    pub remark: Option<String>,
    pub dynamic_url: Option<String>,

    /// is_tree = true, the dimension is a tree structure
    ///
    /// 该纬度是否是树形结构
    pub is_tree: Option<bool>,
    pub tree_dynamic_url: Option<String>,
    pub rel_attribute_code: Option<Vec<String>>,
    pub rel_attribute_url: Option<String>,
}

/// Modify Dimension Configuration Request Object
///
/// 修改维度配置请求对象
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsConfDimModifyReq {
    /// The name of the dimension
    /// 维度名称
    #[oai(validator(min_length = "2"))]
    pub show_name: Option<String>,
    /// Whether it is a stable dataset,
    /// if true the dimension data is recorded in the corresponding dimension table,
    /// if false the dimension data is recorded in the fact table
    ///
    /// 是否为稳定数据集，
    /// 如果为true，则维度数据记录在对应的维度表中，
    /// 如果为false，则维度数据记录在事实表中
    pub stable_ds: Option<bool>,
    /// Dimension data type
    ///
    /// 维度数据类型
    pub data_type: Option<StatsDataTypeKind>,
    /// 层级，从0开始，用于上卷/下卷。
    /// 每个维度可以定义多个字段。
    /// 例如地址维度可以是省-市-区等
    /// Hierarchy, starting from 0, for up-rolls/down-rolls.
    /// Multiple fields can be defined for each dimension.
    /// e.g. address dimension can be province-city-district, etc.
    pub hierarchy: Option<Vec<String>>,
    pub remark: Option<String>,
    pub dynamic_url: Option<String>,

    /// is_tree = true, the dimension is a tree structure
    ///
    /// 该纬度是否是树形结构
    pub is_tree: Option<bool>,
    pub tree_dynamic_url: Option<String>,
    pub rel_attribute_code: Option<Vec<String>>,
    pub rel_attribute_url: Option<String>,
}

/// Dimension Configuration Response Object
///
/// 维度配置响应对象
#[derive(poem_openapi::Object, sea_orm::FromQueryResult, Serialize, Deserialize, Debug)]
pub struct StatsConfDimInfoResp {
    /// The primary key or encoding passed in from the external system
    /// 外部系统传入的主键或编码
    #[oai(validator(pattern = r"^[a-z0-9_]+$"))]
    pub key: String,
    /// 维度名称
    /// The name of the dimension
    #[oai(validator(min_length = "2"))]
    pub show_name: String,

    /// Whether it is a stable dataset,
    /// if true the dimension data is recorded in the corresponding dimension table,
    /// if false the dimension data is recorded in the fact table
    ///
    /// 是否为稳定数据集，
    /// 如果为true，则维度数据记录在对应的维度表中，
    /// 如果为false，则维度数据记录在事实表中
    pub stable_ds: bool,
    /// Dimension data type
    ///
    /// 维度数据类型
    pub data_type: StatsDataTypeKind,
    /// Hierarchy, starting from 0, for up-rolls/down-rolls.
    /// Multiple fields can be defined for each dimension.
    /// e.g. address dimension can be province-city-district, etc.
    ///
    /// 层级，从0开始，用于上卷/下卷。
    /// 每个维度可以定义多个字段。
    /// 例如地址维度可以是省-市-区等
    pub hierarchy: Vec<String>,
    /// Whether the dimension is enabled
    pub online: bool,
    pub remark: Option<String>,
    pub dynamic_url: Option<String>,

    /// is_tree = true, the dimension is a tree structure
    ///
    /// 该纬度是否是树形结构
    pub is_tree: bool,
    pub tree_dynamic_url: Option<String>,
    pub rel_attribute_code: Option<Vec<String>>,
    pub rel_attribute_url: Option<String>,

    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

/// Add Fact Configuration Request Object
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsConfFactAddReq {
    /// The primary key or encoding passed in from the external system
    ///
    /// 外部系统传入的主键或编码
    #[oai(validator(pattern = r"^[a-z0-9_]+$"))]
    pub key: String,
    /// The name of the fact
    ///
    /// 事实的名称
    #[oai(validator(min_length = "2"))]
    pub show_name: String,
    /// The default maximum number of queries
    ///
    /// 默认最大查询次数
    #[oai(validator(minimum(value = "1", exclusive = "false")))]
    pub query_limit: i32,
    pub remark: Option<String>,
    pub redirect_path: Option<String>,
    /// default value is false
    pub is_online: Option<bool>,
    pub rel_cert_id: Option<String>,
    pub sync_sql: Option<String>,
    pub sync_cron: Option<String>,
    pub sync_on: Option<bool>,
}

/// Modify Fact Configuration Request Object
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Default)]
pub struct StatsConfFactModifyReq {
    /// The name of the fact
    ///
    /// 事实的名称
    #[oai(validator(min_length = "2"))]
    pub show_name: Option<String>,
    /// The default maximum number of queries
    ///
    /// 默认最大查询次数
    #[oai(validator(minimum(value = "1", exclusive = "false")))]
    pub query_limit: Option<i32>,
    pub remark: Option<String>,
    pub redirect_path: Option<String>,
    pub is_online: Option<bool>,
    pub rel_cert_id: Option<String>,
    pub sync_sql: Option<String>,
    pub sync_cron: Option<String>,
    pub sync_on: Option<bool>,
}

/// Fact Configuration Response Object
#[derive(poem_openapi::Object, sea_orm::FromQueryResult, Serialize, Deserialize, Debug)]
pub struct StatsConfFactInfoResp {
    /// The primary key or encoding passed in from the external system
    ///
    /// 外部系统传入的主键或编码
    #[oai(validator(pattern = r"^[a-z0-9_]+$"))]
    pub key: String,
    /// The name of the fact
    ///
    /// 事实的名称
    #[oai(validator(min_length = "2"))]
    pub show_name: String,
    /// The default maximum number of queries
    ///
    /// 默认最大查询次数
    #[oai(validator(minimum(value = "1", exclusive = "false")))]
    pub query_limit: i32,
    /// Whether the dimension is enabled
    pub online: bool,
    pub remark: Option<String>,
    pub is_online: bool,
    pub redirect_path: Option<String>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub rel_cert_id: Option<String>,
    pub sync_sql: Option<String>,
    pub sync_cron: Option<String>,
    pub sync_on: Option<bool>,
}

/// Add Fact Column Configuration Request Object
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsConfFactColAddReq {
    /// The primary key or encoding passed in from the external system
    ///
    /// 外部系统传入的主键或编码
    #[oai(validator(pattern = r"^[a-z0-9_]+$"))]
    pub key: String,
    /// The name of the fact column
    ///
    /// 事实列的名称
    #[oai(validator(min_length = "2"))]
    pub show_name: String,
    /// kind = Dimension, with index
    /// kind = Measure, without index
    /// kind = Ext, for recording data only, character type, without index
    ///
    /// 类型 = 维度，带索引
    /// 类型 = 度量，不带索引
    /// 类型 = 扩展，仅用于记录数据，字符类型，不带索引
    pub kind: StatsFactColKind,
    /// Valid when kind = Dimension, used to specify the associated dimension configuration table
    ///
    /// 当kind = Dimension时有效，用于指定关联的维度配置表
    pub dim_rel_conf_dim_key: Option<String>,
    /// valid when kind = Dimension, whether to allow multiple values.
    /// When true, the corresponding data format is an array type, and uses the gin type index
    ///
    /// 当kind = Dimension时有效，是否允许多值。
    /// 当为true时，对应的数据格式为数组类型，使用gin类型索引
    pub dim_multi_values: Option<bool>,

    /// Valid when kind = Dimension, used to specify the data type
    /// dynamic dimension when valid
    ///
    /// 当kind = Dimension时有效，用于指定数据类型
    /// 且是动态维度时有效
    pub dim_data_type: Option<StatsDataTypeKind>,
    /// Valid when kind = Dimension, Used to specify the dynamic URL
    /// dynamic dimension when valid
    ///
    /// 当kind = Dimension时有效，用于指定动态URL
    /// 且是动态维度时有效
    pub dim_dynamic_url: Option<String>,
    /// Valid when kind = Measure, Whether to carry out weight distinct
    ///
    /// 当kind = 度量时有效，是否进行权重去重
    pub mes_data_distinct: Option<bool>,
    /// Valid when kind = Measure, Used to specify the data type
    ///
    /// 当kind = 度量时有效，用于指定数据类型
    pub mes_data_type: Option<StatsDataTypeKind>,
    /// Valid when kind = Measure, Used to specify the data update frequency.
    /// E.g. RT(Real Time),1H(Hour),1D(Day),1M(Month)
    ///
    /// 当kind = 度量时有效，用于指定数据更新频率。
    /// 例如 RT(实时)，1H(小时)，1D(天)，1M(月)
    pub mes_frequency: Option<String>,
    /// Valid when kind = Measure, Used to specify the measure unit
    ///
    /// 当kind = 度量时有效，用于指定度量单位
    pub mes_unit: Option<String>,
    /// Valid when kind = Measure, Used to specify the measure activation (only active when all specified dimensions are present)
    ///
    /// 当kind = 度量时有效，用于指定度量激活（仅在所有指定维度都存在时激活）
    pub mes_act_by_dim_conf_keys: Option<Vec<String>>,
    /// Associated fact and fact column configuration.
    /// Format: <fact configuration table key>.<fact field configuration table key>
    ///
    /// 关联的事实和事实列配置。
    /// 格式：<事实配置表key>.<事实字段配置表key>
    pub rel_conf_fact_and_col_key: Option<String>,
    /// The primary key or encoding passed in from the external system
    /// Used to extend the fact column of the ext field
    ///
    /// 关联外部系统传入的主键或编码
    /// 用于扩展ext字段的事实列
    pub rel_external_id: Option<String>,
    pub dim_exclusive_rec: Option<bool>,
    pub remark: Option<String>,
    pub rel_field: Option<String>,
    pub rel_sql: Option<String>,
    pub rel_cert_id: Option<String>,
}

/// Modify Fact Column Configuration Request Object
///
/// 修改事实列配置请求对象
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsConfFactColModifyReq {
    /// The name of the fact column
    ///
    /// 事实列的名称
    #[oai(validator(min_length = "2"))]
    pub show_name: Option<String>,
    /// kind = Dimension, with index
    /// kind = Measure, without index
    /// kind = Ext, for recording data only, character type, without index
    ///
    /// 类型 = 维度，带索引
    /// 类型 = 度量，不带索引
    /// 类型 = 扩展，仅用于记录数据，字符类型，不带索引
    pub kind: Option<StatsFactColKind>,
    /// Valid when kind = Dimension, used to specify the associated dimension configuration table
    ///
    /// 当kind = Dimension时有效，用于指定关联的维度配置表
    pub dim_rel_conf_dim_key: Option<String>,
    /// Valid when kind = Measure, Whether to carry out weight distinct
    ///
    /// 当kind = Dimension时有效，是否允许多值。
    pub mes_data_distinct: Option<bool>,
    /// valid when kind = Dimension, whether to allow multiple values.
    /// When true, the corresponding data format is an array type, and uses the gin type index
    ///
    /// 当kind = Measure时有效，用于指定数据类型
    /// 如果为true时，对应的数据格式为数组类型，使用gin类型索引
    pub dim_multi_values: Option<bool>,
    pub dim_exclusive_rec: Option<bool>,
    /// Valid when kind = Dimension, used to specify the data type
    /// dynamic dimension when valid
    ///
    /// 当kind = Dimension时有效，用于指定数据类型
    /// 且是动态维度时有效
    pub dim_data_type: Option<StatsDataTypeKind>,
    /// Valid when kind = Dimension, Used to specify the dynamic URL
    /// dynamic dimension when valid
    ///
    /// 当kind = Dimension时有效，用于指定动态URL
    /// 且是动态维度时有效
    pub dim_dynamic_url: Option<String>,
    /// Valid when kind = Measure, Used to specify the data type
    ///
    /// 当kind = Measure时有效，用于指定数据类型
    pub mes_data_type: Option<StatsDataTypeKind>,
    /// Valid when kind = Measure, Used to specify the data update frequency.
    /// E.g. RT(Real Time),1H(Hour),1D(Day),1M(Month)
    ///
    /// 当kind = Measure时有效，用于指定数据更新频率。
    /// 例如 RT(实时)，1H(小时)，1D(天)，1M(月)
    pub mes_frequency: Option<String>,
    /// Valid when kind = Measure, Used to specify the measure unit
    ///
    /// 当kind = Measure时有效，用于指定度量单位
    pub mes_unit: Option<String>,
    /// Valid when kind = Measure, Used to specify the measure activation (only active when all specified dimensions are present)
    ///
    /// 当kind = Measure时有效，用于指定度量激活（仅在所有指定维度都存在时激活）
    pub mes_act_by_dim_conf_keys: Option<Vec<String>>,
    /// Associated fact and fact column configuration.
    /// Format: <fact configuration table key>.<fact field configuration table key>
    ///
    /// 关联的事实和事实列配置。
    /// 格式：<事实配置表key>.<事实字段配置表key>
    pub rel_conf_fact_and_col_key: Option<String>,
    /// The primary key or encoding passed in from the external system
    /// Used to extend the fact column of the ext field
    ///
    /// 关联外部系统传入的主键或编码
    /// 用于扩展ext字段的事实列
    pub rel_external_id: Option<String>,
    pub remark: Option<String>,
    pub rel_field: Option<String>,
    pub rel_sql: Option<String>,
    pub rel_cert_id: Option<String>,
}

/// Fact Column Configuration Response Object
///
/// 事实列配置响应对象
#[derive(poem_openapi::Object, sea_orm::FromQueryResult, Serialize, Deserialize, Debug)]
pub struct StatsConfFactColInfoResp {
    /// The primary key or encoding passed in from the external system
    ///
    /// 外部系统传入的主键或编码
    #[oai(validator(pattern = r"^[a-z0-9_]+$"))]
    pub key: String,
    /// The name of the fact column
    ///
    /// 事实列的名称
    #[oai(validator(min_length = "2"))]
    pub show_name: String,
    /// kind = Dimension, with index
    /// kind = Measure, without index
    /// kind = Ext, for recording data only, character type, without index
    ///
    /// 类型 = 维度，带索引
    /// 类型 = 度量，不带索引
    /// 类型 = 扩展，仅用于记录数据，字符类型，不带索引
    pub kind: StatsFactColKind,
    /// Valid when kind = Dimension, used to specify the associated dimension configuration table
    ///
    /// 当kind = Dimension时有效，用于指定关联的维度配置表
    pub dim_rel_conf_dim_key: Option<String>,
    /// valid when kind = Dimension, whether to allow multiple values.
    /// When true, the corresponding data format is an array type, and uses the gin type index
    ///
    /// 当kind = Dimension时有效，是否允许多值。
    /// 当为true时，对应的数据格式为数组类型，使用gin类型索引
    pub dim_multi_values: Option<bool>,
    /// Valid when kind = Dimension, used to specify the data type
    /// dynamic dimension when valid
    ///
    /// 当kind = Dimension时有效，用于指定数据类型
    /// 且是动态维度时有效
    pub dim_data_type: Option<StatsDataTypeKind>,
    /// Valid when kind = Dimension, Used to specify the dynamic URL
    /// dynamic dimension when valid
    ///
    /// 当kind = Dimension时有效，用于指定动态URL
    /// 且是动态维度时有效
    pub dim_dynamic_url: Option<String>,
    /// Valid when kind = Measure, Whether to carry out weight distinct
    ///
    /// 当kind = 度量时有效，是否进行权重去重
    pub mes_data_distinct: Option<bool>,
    /// Valid when kind = Measure, Used to specify the data type
    ///
    /// 当kind = 度量时有效，用于指定数据类型
    pub mes_data_type: Option<StatsDataTypeKind>,
    /// Valid when kind = Measure, Used to specify the data update frequency.
    /// E.g. RT(Real Time),1H(Hour),1D(Day),1M(Month)
    ///
    /// 当kind = 度量时有效，用于指定数据更新频率。
    /// 例如 RT(实时)，1H(小时)，1D(天)，1M(月)
    pub mes_frequency: Option<String>,
    /// Valid when kind = Measure, Used to specify the measure unit
    ///
    /// 当kind = 度量时有效，用于指定度量单位
    pub mes_unit: Option<String>,
    /// Valid when kind = Measure, Used to specify the measure activation (only active when all specified dimensions are present)
    ///
    /// 当kind = 度量时有效，用于指定度量激活（仅在所有指定维度都存在时激活）
    pub mes_act_by_dim_conf_keys: Option<Vec<String>>,
    /// Associated fact key
    ///
    /// 关联的事实key
    pub rel_conf_fact_key: Option<String>,
    /// Associated fact and fact column configuration.
    /// Format: <fact configuration table key>.<fact field configuration table key>
    ///
    /// 关联的事实和事实列配置。
    /// 格式：<事实配置表key>.<事实字段配置表key>
    pub rel_conf_fact_and_col_key: Option<String>,
    /// The primary key or encoding passed in from the external system
    /// Used to extend the fact column of the ext field
    ///
    /// 关联外部系统传入的主键或编码
    /// 用于扩展ext字段的事实列
    pub rel_external_id: Option<String>,
    pub dim_exclusive_rec: Option<String>,
    pub remark: Option<String>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub rel_field: Option<String>,
    pub rel_sql: Option<String>,
    pub rel_cert_id: Option<String>,
}

/// Add Sync DateBase Config Request Object
///
/// 添加同步数据库配置请求对象
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsSyncDbConfigAddReq {
    pub db_url: String,
    pub db_user: String,
    pub db_password: String,
    pub max_connections: Option<u32>,
    pub min_connections: Option<u32>,
}

/// Modify Sync DateBase Config Request Object
///
/// 修改同步数据库配置请求对象
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsSyncDbConfigModifyReq {
    pub id: String,
    pub db_url: Option<String>,
    pub db_user: Option<String>,
    pub db_password: Option<String>,
    pub max_connections: Option<u32>,
    pub min_connections: Option<u32>,
}

/// Sync DateBase Config Response Object
///
/// 同步数据库配置响应对象
#[derive(poem_openapi::Object, sea_orm::FromQueryResult, Serialize, Deserialize, Debug)]
pub struct StatsSyncDbConfigInfoResp {
    pub id: String,
    pub db_url: String,
    pub db_user: String,
    pub max_connections: Option<u32>,
    pub min_connections: Option<u32>,
}

/// Sync DateBase Config Response Object
///
/// 同步数据库配置响应对象
#[derive(poem_openapi::Object, sea_orm::FromQueryResult, Serialize, Deserialize, Debug)]
pub struct StatsSyncDbConfigInfoWithSkResp {
    pub id: String,
    pub db_url: String,
    pub db_user: String,
    pub db_password: String,
    pub max_connections: Option<u32>,
    pub min_connections: Option<u32>,
}

/// Sync DateBase Config Extension Object
///
/// 同步数据库配置扩展对象
#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct StatsSyncDbConfigExt {
    pub max_connections: Option<u32>,
    pub min_connections: Option<u32>,
}