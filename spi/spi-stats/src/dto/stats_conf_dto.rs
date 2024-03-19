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
    /// 外部系统传入的主键或编码
    /// The primary key or encoding passed in from the external system
    #[oai(validator(pattern = r"^[a-z0-9_]+$"))]
    pub key: String,
    /// 维度名称
    /// The name of the dimension
    #[oai(validator(min_length = "2"))]
    pub show_name: String,
    /// 是否为稳定数据集，
    /// 如果为true，则维度数据记录在对应的维度表中，
    /// 如果为false，则维度数据记录在事实表中
    /// Whether it is a stable dataset,
    /// if true the dimension data is recorded in the corresponding dimension table,
    /// if false the dimension data is recorded in the fact table
    pub stable_ds: bool,
    /// 维度数据类型
    /// Dimension data type
    pub data_type: StatsDataTypeKind,
    /// 层级，从0开始，用于上卷/下卷。
    /// 每个维度可以定义多个字段。
    /// 例如地址维度可以是省-市-区等
    /// Hierarchy, starting from 0, for up-rolls/down-rolls.
    /// Multiple fields can be defined for each dimension.
    /// e.g. address dimension can be province-city-district, etc.
    pub hierarchy: Option<Vec<String>>,
    pub remark: Option<String>,
    pub dynamic_url: Option<String>,
}

/// Modify Dimension Configuration Request Object
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsConfDimModifyReq {
    /// 维度名称
    /// The name of the dimension
    #[oai(validator(min_length = "2"))]
    pub show_name: Option<String>,
    /// 是否为稳定数据集，
    /// 如果为true，则维度数据记录在对应的维度表中，
    /// 如果为false，则维度数据记录在事实表中
    /// Whether it is a stable dataset,
    /// if true the dimension data is recorded in the corresponding dimension table,
    /// if false the dimension data is recorded in the fact table
    pub stable_ds: Option<bool>,
    /// 维度数据类型
    /// Dimension data type
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
}

/// Dimension Configuration Response Object
#[derive(poem_openapi::Object, sea_orm::FromQueryResult, Serialize, Deserialize, Debug)]
pub struct StatsConfDimInfoResp {
    /// 外部系统传入的主键或编码
    /// The primary key or encoding passed in from the external system
    #[oai(validator(pattern = r"^[a-z0-9_]+$"))]
    pub key: String,
    /// 维度名称
    /// The name of the dimension
    #[oai(validator(min_length = "2"))]
    pub show_name: String,
    /// 是否为稳定数据集，
    /// 如果为true，则维度数据记录在对应的维度表中，
    /// 如果为false，则维度数据记录在事实表中
    /// Whether it is a stable dataset,
    /// if true the dimension data is recorded in the corresponding dimension table,
    /// if false the dimension data is recorded in the fact table
    pub stable_ds: bool,
    /// 维度数据类型
    /// Dimension data type
    pub data_type: StatsDataTypeKind,
    /// 层级，从0开始，用于上卷/下卷。
    /// 每个维度可以定义多个字段。
    /// 例如地址维度可以是省-市-区等
    /// Hierarchy, starting from 0, for up-rolls/down-rolls.
    /// Multiple fields can be defined for each dimension.
    /// e.g. address dimension can be province-city-district, etc.
    pub hierarchy: Vec<String>,
    /// Whether the dimension is enabled
    pub online: bool,
    pub remark: Option<String>,
    pub dynamic_url: Option<String>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

/// Add Fact Configuration Request Object
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsConfFactAddReq {
    /// 外部系统传入的主键或编码
    /// The primary key or encoding passed in from the external system
    #[oai(validator(pattern = r"^[a-z0-9_]+$"))]
    pub key: String,
    /// 事实的名称
    /// The name of the fact
    #[oai(validator(min_length = "2"))]
    pub show_name: String,
    /// 默认最大查询次数
    /// The default maximum number of queries
    #[oai(validator(minimum(value = "1", exclusive = "false")))]
    pub query_limit: i32,
    pub remark: Option<String>,
    pub redirect_path: Option<String>,
    /// default value is false
    pub is_online: Option<bool>,
}

/// Modify Fact Configuration Request Object
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Default)]
pub struct StatsConfFactModifyReq {
    /// 事实的名称
    /// The name of the fact
    #[oai(validator(min_length = "2"))]
    pub show_name: Option<String>,
    /// 默认最大查询次数
    /// The default maximum number of queries
    #[oai(validator(minimum(value = "1", exclusive = "false")))]
    pub query_limit: Option<i32>,
    pub remark: Option<String>,
    pub redirect_path: Option<String>,
    pub is_online: Option<bool>,
}

/// Fact Configuration Response Object
#[derive(poem_openapi::Object, sea_orm::FromQueryResult, Serialize, Deserialize, Debug)]
pub struct StatsConfFactInfoResp {
    /// 外部系统传入的主键或编码
    /// The primary key or encoding passed in from the external system
    #[oai(validator(pattern = r"^[a-z0-9_]+$"))]
    pub key: String,
    /// 事实的名称
    /// The name of the fact
    #[oai(validator(min_length = "2"))]
    pub show_name: String,
    /// 默认最大查询次数
    /// The default maximum number of queries
    #[oai(validator(minimum(value = "1", exclusive = "false")))]
    pub query_limit: i32,
    /// Whether the dimension is enabled
    pub online: bool,
    pub remark: Option<String>,
    pub is_online: bool,
    pub redirect_path: Option<String>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

/// Add Fact Column Configuration Request Object
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsConfFactColAddReq {
    /// 外部系统传入的主键或编码
    /// The primary key or encoding passed in from the external system
    #[oai(validator(pattern = r"^[a-z0-9_]+$"))]
    pub key: String,
    /// 事实列的名称
    /// The name of the fact column
    #[oai(validator(min_length = "2"))]
    pub show_name: String,
    /// 类型 = 维度，带索引
    /// 类型 = 度量，不带索引
    /// 类型 = 扩展，仅用于记录数据，字符类型，不带索引
    /// kind = Dimension, with index
    /// kind = Measure, without index
    /// kind = Ext, for recording data only, character type, without index
    pub kind: StatsFactColKind,
    /// 当kind = Dimension时有效，用于指定关联的维度配置表
    /// Valid when kind = Dimension, used to specify the associated dimension configuration table
    pub dim_rel_conf_dim_key: Option<String>,
    /// 当kind = Dimension时有效，是否允许多值。
    /// 当为true时，对应的数据格式为数组类型，使用gin类型索引
    /// valid when kind = Dimension, whether to allow multiple values.
    /// When true, the corresponding data format is an array type, and uses the gin type index
    pub dim_multi_values: Option<bool>,
    /// 当kind = 度量时有效，是否进行权重去重
    /// Valid when kind = Measure, Whether to carry out weight distinct
    pub mes_data_distinct: Option<bool>,
    /// 当kind = 度量时有效，用于指定数据类型
    /// Valid when kind = Measure, Used to specify the data type
    pub mes_data_type: Option<StatsDataTypeKind>,
    /// 当kind = 度量时有效，用于指定数据更新频率。
    /// 例如 RT(实时)，1H(小时)，1D(天)，1M(月)
    /// Valid when kind = Measure, Used to specify the data update frequency.
    /// E.g. RT(Real Time),1H(Hour),1D(Day),1M(Month)
    pub mes_frequency: Option<String>,
    /// 当kind = 度量时有效，用于指定度量单位
    /// Valid when kind = Measure, Used to specify the measure unit
    pub mes_unit: Option<String>,
    /// 当kind = 度量时有效，用于指定度量激活（仅在所有指定维度都存在时激活）
    /// Valid when kind = Measure, Used to specify the measure activation (only active when all specified dimensions are present)
    pub mes_act_by_dim_conf_keys: Option<Vec<String>>,
    /// 关联的事实和事实列配置。
    /// 格式：<事实配置表key>.<事实字段配置表key>
    /// Associated fact and fact column configuration.
    /// Format: <fact configuration table key>.<fact field configuration table key>
    pub rel_conf_fact_and_col_key: Option<String>,
    pub dim_exclusive_rec: Option<bool>,
    pub remark: Option<String>,
}

/// 修改事实列配置请求对象
/// Modify Fact Column Configuration Request Object
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsConfFactColModifyReq {
    /// 事实列的名称
    /// The name of the fact column
    #[oai(validator(min_length = "2"))]
    pub show_name: Option<String>,
    /// 类型 = 维度，带索引
    /// 类型 = 度量，不带索引
    /// 类型 = 扩展，仅用于记录数据，字符类型，不带索引
    /// kind = Dimension, with index
    /// kind = Measure, without index
    /// kind = Ext, for recording data only, character type, without index
    pub kind: Option<StatsFactColKind>,
    /// 当kind = Dimension时有效，用于指定关联的维度配置表
    /// Valid when kind = Dimension, used to specify the associated dimension configuration table
    pub dim_rel_conf_dim_key: Option<String>,
    /// 当kind = Dimension时有效，是否允许多值。
    /// Valid when kind = Measure, Whether to carry out weight distinct
    pub mes_data_distinct: Option<bool>,
    /// 当kind = Measure时有效，用于指定数据类型
    /// 如果为true时，对应的数据格式为数组类型，使用gin类型索引
    /// valid when kind = Dimension, whether to allow multiple values.
    /// When true, the corresponding data format is an array type, and uses the gin type index
    pub dim_multi_values: Option<bool>,
    pub dim_exclusive_rec: Option<bool>,
    /// 当kind = Measure时有效，用于指定数据类型
    /// Valid when kind = Measure, Used to specify the data type
    pub mes_data_type: Option<StatsDataTypeKind>,
    /// 当kind = Measure时有效，用于指定数据更新频率。
    /// 例如 RT(实时)，1H(小时)，1D(天)，1M(月)
    /// Valid when kind = Measure, Used to specify the data update frequency.
    /// E.g. RT(Real Time),1H(Hour),1D(Day),1M(Month)
    pub mes_frequency: Option<String>,
    /// 当kind = Measure时有效，用于指定度量单位
    /// Valid when kind = Measure, Used to specify the measure unit
    pub mes_unit: Option<String>,
    /// 当kind = Measure时有效，用于指定度量激活（仅在所有指定维度都存在时激活）
    /// Valid when kind = Measure, Used to specify the measure activation (only active when all specified dimensions are present)
    pub mes_act_by_dim_conf_keys: Option<Vec<String>>,
    /// 关联的事实和事实列配置。
    /// 格式：<事实配置表key>.<事实字段配置表key>
    /// Associated fact and fact column configuration.
    /// Format: <fact configuration table key>.<fact field configuration table key>
    pub rel_conf_fact_and_col_key: Option<String>,
    pub remark: Option<String>,
}

/// 事实列配置响应对象
/// Fact Column Configuration Response Object
#[derive(poem_openapi::Object, sea_orm::FromQueryResult, Serialize, Deserialize, Debug)]
pub struct StatsConfFactColInfoResp {
    /// 外部系统传入的主键或编码
    /// The primary key or encoding passed in from the external system
    #[oai(validator(pattern = r"^[a-z0-9_]+$"))]
    pub key: String,
    /// 事实列的名称
    /// The name of the fact column
    #[oai(validator(min_length = "2"))]
    pub show_name: String,
    /// 类型 = 维度，带索引
    /// 类型 = 度量，不带索引
    /// 类型 = 扩展，仅用于记录数据，字符类型，不带索引
    /// kind = Dimension, with index
    /// kind = Measure, without index
    /// kind = Ext, for recording data only, character type, without index
    pub kind: StatsFactColKind,
    /// 当kind = Dimension时有效，用于指定关联的维度配置表
    /// Valid when kind = Dimension, used to specify the associated dimension configuration table
    pub dim_rel_conf_dim_key: Option<String>,
    /// 当kind = Dimension时有效，是否允许多值。
    /// 当为true时，对应的数据格式为数组类型，使用gin类型索引
    /// valid when kind = Dimension, whether to allow multiple values.
    /// When true, the corresponding data format is an array type, and uses the gin type index
    pub dim_multi_values: Option<bool>,
    /// 当kind = 度量时有效，是否进行权重去重
    /// Valid when kind = Measure, Whether to carry out weight distinct
    pub mes_data_distinct: Option<bool>,
    /// 当kind = 度量时有效，用于指定数据类型
    /// Valid when kind = Measure, Used to specify the data type
    pub mes_data_type: Option<StatsDataTypeKind>,
    /// 当kind = 度量时有效，用于指定数据更新频率。
    /// 例如 RT(实时)，1H(小时)，1D(天)，1M(月)
    /// Valid when kind = Measure, Used to specify the data update frequency.
    /// E.g. RT(Real Time),1H(Hour),1D(Day),1M(Month)
    pub mes_frequency: Option<String>,
    /// 当kind = 度量时有效，用于指定度量单位
    /// Valid when kind = Measure, Used to specify the measure unit
    pub mes_unit: Option<String>,
    /// 当kind = 度量时有效，用于指定度量激活（仅在所有指定维度都存在时激活）
    /// Valid when kind = Measure, Used to specify the measure activation (only active when all specified dimensions are present)
    pub mes_act_by_dim_conf_keys: Option<Vec<String>>,
    /// 关联的事实和事实列配置。
    /// 格式：<事实配置表key>.<事实字段配置表key>
    /// Associated fact and fact column configuration.
    /// Format: <fact configuration table key>.<fact field configuration table key>
    pub rel_conf_fact_and_col_key: Option<String>,
    pub dim_exclusive_rec: Option<String>,
    pub remark: Option<String>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}
