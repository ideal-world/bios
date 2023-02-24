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
    #[oai(validator(pattern = r"^[a-z0-9_]+$"))]
    pub key: String,
    /// The name of the dimension
    #[oai(validator(min_length = "2"))]
    pub show_name: String,
    /// Whether it is a stable dataset,
    /// if true the dimension data is recorded in the corresponding dimension table,
    /// if false the dimension data is recorded in the fact table
    pub stable_ds: bool,
    /// Dimension data type
    pub data_type: StatsDataTypeKind,
    /// Hierarchy, starting from 0, for up-rolls/down-rolls.
    /// Multiple fields can be defined for each dimension.
    /// e.g. address dimension can be province-city-district, etc.
    pub hierarchy: Option<Vec<String>>,
    pub remark: Option<String>,
}

/// Modify Dimension Configuration Request Object
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsConfDimModifyReq {
    /// The name of the dimension
    #[oai(validator(min_length = "2"))]
    pub show_name: Option<String>,
    /// Whether it is a stable dataset,
    /// if true the dimension data is recorded in the corresponding dimension table,
    /// if false the dimension data is recorded in the fact table
    pub stable_ds: Option<bool>,
    /// Dimension data type
    pub data_type: Option<StatsDataTypeKind>,
    /// Hierarchy, starting from 0, for up-rolls/down-rolls.
    /// Multiple fields can be defined for each dimension.
    /// e.g. address dimension can be province-city-district, etc.
    pub hierarchy: Option<Vec<String>>,
    pub remark: Option<String>,
}

/// Dimension Configuration Response Object
#[derive(poem_openapi::Object, sea_orm::FromQueryResult, Serialize, Deserialize, Debug)]
pub struct StatsConfDimInfoResp {
    /// The primary key or encoding passed in from the external system
    #[oai(validator(pattern = r"^[a-z0-9_]+$"))]
    pub key: String,
    /// The name of the dimension
    #[oai(validator(min_length = "2"))]
    pub show_name: String,
    /// Whether it is a stable dataset,
    /// if true the dimension data is recorded in the corresponding dimension table,
    /// if false the dimension data is recorded in the fact table
    pub stable_ds: bool,
    /// Dimension data type
    pub data_type: StatsDataTypeKind,
    /// Hierarchy, starting from 0, for up-rolls/down-rolls.
    /// Multiple fields can be defined for each dimension.
    /// e.g. address dimension can be province-city-district, etc.
    pub hierarchy: Vec<String>,
    /// Whether the dimension is enabled
    pub online: bool,
    pub remark: Option<String>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

/// Add Fact Configuration Request Object
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsConfFactAddReq {
    /// The primary key or encoding passed in from the external system
    #[oai(validator(pattern = r"^[a-z0-9_]+$"))]
    pub key: String,
    /// The name of the fact
    #[oai(validator(min_length = "2"))]
    pub show_name: String,
    /// The default maximum number of queries
    #[oai(validator(minimum(value = "1", exclusive = "false")))]
    pub query_limit: i64,
    pub remark: Option<String>,
}

/// Modify Fact Configuration Request Object
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsConfFactModifyReq {
    /// The name of the fact
    #[oai(validator(min_length = "2"))]
    pub show_name: Option<String>,
    /// The default maximum number of queries
    #[oai(validator(minimum(value = "1", exclusive = "false")))]
    pub query_limit: Option<i64>,
    pub remark: Option<String>,
}

/// Fact Configuration Response Object
#[derive(poem_openapi::Object, sea_orm::FromQueryResult, Serialize, Deserialize, Debug)]
pub struct StatsConfFactInfoResp {
    /// The primary key or encoding passed in from the external system
    #[oai(validator(pattern = r"^[a-z0-9_]+$"))]
    pub key: String,
    /// The name of the fact
    #[oai(validator(min_length = "2"))]
    pub show_name: String,
    /// The default maximum number of queries
    #[oai(validator(minimum(value = "1", exclusive = "false")))]
    pub query_limit: i64,
    /// Whether the dimension is enabled
    pub online: bool,
    pub remark: Option<String>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

/// Add Fact Column Configuration Request Object
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsConfFactColAddReq {
    /// The primary key or encoding passed in from the external system
    #[oai(validator(pattern = r"^[a-z0-9_]+$"))]
    pub key: String,
    /// The name of the fact column
    #[oai(validator(min_length = "2"))]
    pub show_name: String,
    /// kind = Dimension, with index
    /// kind = Measure, without index
    /// kind = Ext, for recording data only, character type, without index
    pub kind: StatsFactColKind,
    /// Valid when kind = Dimension, used to specify the associated dimension configuration table
    pub dim_rel_conf_dim_key: Option<String>,
    /// valid when kind = Dimension, whether to allow multiple values.
    /// When true, the corresponding data format is an array type, and uses the gin type index
    pub dim_multi_values: Option<bool>,
    /// Valid when kind = Measure, Used to specify the data type
    pub mes_data_type: Option<StatsDataTypeKind>,
    /// Valid when kind = Measure, Used to specify the data update frequency.
    /// E.g. RT(Real Time),1H(Hour),1D(Day),1M(Month)
    pub mes_frequency: Option<String>,
    /// Valid when kind = Measure, Used to specify the measure activation (only active when all specified dimensions are present)
    pub mes_act_by_dim_conf_keys: Option<Vec<String>>,
    /// Associated fact and fact column configuration.
    /// Format: <fact configuration table key>.<fact field configuration table key>
    pub rel_conf_fact_and_col_key: Option<String>,
    pub remark: Option<String>,
}

/// Modify Fact Column Configuration Request Object
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsConfFactColModifyReq {
    /// The name of the fact column
    #[oai(validator(min_length = "2"))]
    pub show_name: Option<String>,
    /// kind = Dimension, with index
    /// kind = Measure, without index
    /// kind = Ext, for recording data only, character type, without index
    pub kind: Option<StatsFactColKind>,
    /// Valid when kind = Dimension, used to specify the associated dimension configuration table
    pub dim_rel_conf_dim_key: Option<String>,
    /// valid when kind = Dimension, whether to allow multiple values.
    /// When true, the corresponding data format is an array type, and uses the gin type index
    pub dim_multi_values: Option<bool>,
    /// Valid when kind = Measure, Used to specify the data type
    pub mes_data_type: Option<StatsDataTypeKind>,
    /// Valid when kind = Measure, Used to specify the data update frequency.
    /// E.g. RT(Real Time),1H(Hour),1D(Day),1M(Month)
    pub mes_frequency: Option<String>,
    /// Valid when kind = Measure, Used to specify the measure activation (only active when all specified dimensions are present)
    pub mes_act_by_dim_conf_keys: Option<Vec<String>>,
    /// Associated fact and fact column configuration.
    /// Format: <fact configuration table key>.<fact field configuration table key>
    pub rel_conf_fact_and_col_key: Option<String>,
    pub remark: Option<String>,
}

/// Fact Column Configuration Response Object
#[derive(poem_openapi::Object, sea_orm::FromQueryResult, Serialize, Deserialize, Debug)]
pub struct StatsConfFactColInfoResp {
    /// The primary key or encoding passed in from the external system
    #[oai(validator(pattern = r"^[a-z0-9_]+$"))]
    pub key: String,
    /// The name of the fact column
    #[oai(validator(min_length = "2"))]
    pub show_name: String,
    /// kind = Dimension, with index
    /// kind = Measure, without index
    /// kind = Ext, for recording data only, character type, without index
    pub kind: StatsFactColKind,
    /// Valid when kind = Dimension, used to specify the associated dimension configuration table
    pub dim_rel_conf_dim_key: Option<String>,
    /// valid when kind = Dimension, whether to allow multiple values.
    /// When true, the corresponding data format is an array type, and uses the gin type index
    pub dim_multi_values: Option<bool>,
    /// Valid when kind = Measure, Used to specify the data type
    pub mes_data_type: Option<StatsDataTypeKind>,
    /// Valid when kind = Measure, Used to specify the data update frequency.
    /// E.g. RT(Real Time),1H(Hour),1D(Day),1M(Month)
    pub mes_frequency: Option<String>,
    /// Valid when kind = Measure, Used to specify the measure activation (only active when all specified dimensions are present)
    pub mes_act_by_dim_conf_keys: Option<Vec<String>>,
    /// Associated fact and fact column configuration.
    /// Format: <fact configuration table key>.<fact field configuration table key>
    pub rel_conf_fact_and_col_key: Option<String>,
    pub remark: Option<String>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}
