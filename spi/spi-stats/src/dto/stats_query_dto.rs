use std::collections::HashMap;

use bios_basic::spi::spi_enumeration::SpiQueryOpKind;
use serde::{Deserialize, Serialize};
use tardis::{
    chrono::{DateTime, Utc},
    serde_json::Value,
    web::poem_openapi,
};

use crate::stats_enumeration::{StatsQueryAggFunKind, StatsQueryTimeWindowKind};

/// Query Metrics Request
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsQueryMetricsReq {
    /// Fact code
    pub from: String,
    /// List of fields
    pub select: Vec<StatsQueryMetricsSelectReq>,
    /// List of grouped fields,
    /// the order is related to the returned hierarchy and is handled internally using ROLLUP
    pub group: Vec<StatsQueryMetricsGroupReq>,
    /// Filter conditions, two-dimensional array, OR between groups, AND within groups
    #[oai(rename = "where")]
    pub _where: Option<Vec<Vec<StatsQueryMetricsWhereReq>>>,
    /// Sort conditions
    /// The code and fun must exist in Select
    pub order: Option<Vec<StatsQueryMetricsOrderReq>>,
    /// Filter conditions after group
    /// The code and fun must exist in Select
    pub having: Option<Vec<StatsQueryMetricsHavingReq>>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub limit: Option<u32>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsQueryMetricsSelectReq {
    /// Fact Column key
    pub code: String,
    /// Aggregate function
    pub fun: StatsQueryAggFunKind,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsQueryMetricsGroupReq {
    /// Dimension Column key
    pub code: String,
    /// Time window function
    pub time_window: Option<StatsQueryTimeWindowKind>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsQueryMetricsWhereReq {
    /// Column key
    pub code: String,
    /// Operator
    pub op: SpiQueryOpKind,
    /// Value
    pub value: Value,
    /// Time window function
    pub time_window: Option<StatsQueryTimeWindowKind>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsQueryMetricsOrderReq {
    /// Fact Column key
    pub code: String,
    pub fun: StatsQueryAggFunKind,
    /// Sort direction
    pub asc: bool,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsQueryMetricsHavingReq {
    /// Fact Column key
    pub code: String,
    /// Aggregate function
    pub fun: StatsQueryAggFunKind,
    /// Operator
    pub op: SpiQueryOpKind,
    /// Value
    pub value: Value,
}

/// Query Metrics Response
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsQueryMetricsResp {
    /// Fact key
    pub from: String,
    /// Show names (alias_name -> show_name)
    pub show_names: HashMap<String, String>,
    /// Group
    pub group: Value,
}
