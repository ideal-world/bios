use bios_basic::spi::spi_enumeration::SpiQueryOpKind;
use serde::{Deserialize, Serialize};
use tardis::web::poem_openapi;

use crate::stats_enumeration::{StatsQueryAggFunKind, StatsQueryFunKind, StatsQueryTimeWindowKind};

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
    pub order: Option<Vec<StatsQueryMetricsOrderReq>>,
    /// Filter conditons after group
    pub having: Option<Vec<StatsQueryMetricsHavingReq>>,
    pub distinct: bool,
    pub limit: Option<u32>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsQueryMetricsSelectReq {
    /// Column key
    pub code: String,
    /// Aggregate function
    pub fun: StatsQueryAggFunKind,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsQueryMetricsGroupReq {
    /// Column key
    pub code: String,
    /// SQL function
    pub fun: Option<StatsQueryFunKind>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsQueryMetricsWhereReq {
    /// Column key
    pub code: String,
    /// SQL function
    pub fun: Option<StatsQueryFunKind>,
    /// Operator
    pub op: SpiQueryOpKind,
    /// Value
    pub value: String,
    /// Window function
    pub time_window: StatsQueryTimeWindowKind,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsQueryMetricsOrderReq {
    /// Column key
    pub code: String,
    /// SQL function
    pub fun: Option<StatsQueryFunKind>,
    /// Sort direction
    pub asc: bool,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsQueryMetricsHavingReq {
    /// Column key
    pub code: String,
    /// SQL function
    pub fun: Option<StatsQueryFunKind>,
    /// Operator
    pub op: SpiQueryOpKind,
    /// Value
    pub value: String,
}

/// Query Metrics Response
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsQueryMetricsResp {
    /// Fact key
    pub from: String,
    /// Group
    group: Vec<StatsQueryMetricsGroupResp>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsQueryMetricsGroupResp {
    /// Field alias name
    pub alias_name: String,
    /// Field show name
    pub show_name: String,
    /// Field value
    pub value: String,
    /// Sub group
    pub sub: Vec<StatsQueryMetricsGroupResp>,
}
