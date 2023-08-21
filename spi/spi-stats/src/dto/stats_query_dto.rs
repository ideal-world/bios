use std::collections::HashMap;

use bios_basic::basic_enumeration::BasicQueryOpKind;
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
    /// Ignore distinct key
    /// If true or null, the distinct key will not be counted
    /// If false, the distinct key will be counted
    pub ignore_distinct: Option<bool>,
    /// List of grouped fields,
    /// the order is related to the returned hierarchy and is handled internally using ROLLUP
    pub group: Vec<StatsQueryDimensionGroupReq>,
    /// Ignore group rollup
    /// If true or null, the group rollup will not be counted
    /// If false, the group rollup will be counted
    pub ignore_group_rollup: Option<bool>,
    /// Filter conditions, two-dimensional array, OR between groups, AND within groups
    #[oai(rename = "where")]
    pub _where: Option<Vec<Vec<StatsQueryMetricsWhereReq>>>,
    pub dimension_order: Option<Vec<StatsQueryDimensionOrderReq>>,
    /// Sort conditions
    /// The code and fun must exist in Select
    pub metrics_order: Option<Vec<StatsQueryMetricsOrderReq>>,
    /// Sort conditions
    /// The code and fun must exist in Select
    pub group_order: Option<Vec<StatsQueryDimensionGroupOrderReq>>,
    pub group_agg: Option<bool>,
    /// Filter conditions after group
    /// The code and fun must exist in Select
    pub having: Option<Vec<StatsQueryMetricsHavingReq>>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub limit: Option<u32>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsQueryMetricsSelectReq {
    /// Measure column key
    pub code: String,
    /// Aggregate function
    pub fun: StatsQueryAggFunKind,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsQueryDimensionGroupReq {
    /// Dimension column key
    pub code: String,
    /// Time window function
    pub time_window: Option<StatsQueryTimeWindowKind>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsQueryMetricsWhereReq {
    /// Dimension or measure column key
    pub code: String,
    /// Operator
    pub op: BasicQueryOpKind,
    /// Value
    pub value: Value,
    /// Time window function
    pub time_window: Option<StatsQueryTimeWindowKind>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsQueryMetricsOrderReq {
    /// Measure column key
    pub code: String,
    pub fun: StatsQueryAggFunKind,
    /// Sort direction
    pub asc: bool,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsQueryDimensionGroupOrderReq {
    /// Dimension column key
    pub code: String,
    /// Time window function
    pub time_window: Option<StatsQueryTimeWindowKind>,
    /// Sort direction
    pub asc: bool,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsQueryDimensionOrderReq {
    /// Dimension column key
    pub code: String,
    /// Sort direction
    pub asc: bool,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsQueryMetricsHavingReq {
    /// Measure Column key
    pub code: String,
    /// Aggregate function
    pub fun: StatsQueryAggFunKind,
    /// Operator
    pub op: BasicQueryOpKind,
    /// Value
    pub value: Value,
}

/// Query Metrics Response
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsQueryMetricsResp {
    /// Fact key
    pub from: String,
    /// Show names
    ///
    /// key = alias name, value = show name
    ///
    /// The format of the alias: `field name__<function name>`
    pub show_names: HashMap<String, String>,
    /// Group
    ///
    /// Format with only one level (single dimension):
    /// map
    /// ```
    /// {
    ///     "":{  // The root group
    ///         "alias name1":value,
    ///         "alias name...":value,
    ///     },
    ///     "<group name1>" {
    ///         "alias name1":value,
    ///         "alias name...":value,
    ///     }
    ///     "<group name...>" {
    ///         "alias name1":value,
    ///         "alias name...":value,
    ///     }
    /// }
    /// ```
    /// array
    /// ```
    /// [   
    ///     {
    ///         "name": "<group name1>",
    ///         "value": value
    ///     },
    ///     
    /// ]
    /// ```
    ///
    /// Format with multiple levels (multiple dimensions):
    /// ```
    /// {
    ///     "":{  // The root group
    ///         "": {
    ///             "alias name1":value,
    ///             "alias name...":value,
    ///         }
    ///     },
    ///     "<group name1>" {
    ///         "": {
    ///             "alias name1":value,
    ///             "alias name...":value,
    ///         },
    ///         "<sub group name...>": {
    ///             "alias name1":value,
    ///             "alias name...":value,
    ///         }
    ///     }
    ///     "<group name...>" {
    ///         "": {
    ///             "alias name1":value,
    ///             "alias name...":value,
    ///         },
    ///         "<sub group name...>": {
    ///             "alias name1":value,
    ///             "alias name...":value,
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// # Example
    /// ```
    /// {
    ///     "from": "req",
    ///     "show_names": {
    ///         "ct__date": "创建时间",
    ///         "act_hours__sum": "实例工时",
    ///         "status__": "状态",
    ///         "plan_hours__sum": "计划工时"
    ///     },
    ///     "group": {
    ///         "": {
    ///             "": {
    ///                 "act_hours__sum": 180,
    ///                 "plan_hours__sum": 330
    ///             }
    ///         },
    ///         "2023-01-01": {
    ///             "": {
    ///                 "act_hours__sum": 120,
    ///                 "plan_hours__sum": 240
    ///             },
    ///             "open": {
    ///                 "act_hours__sum": 80,
    ///                 "plan_hours__sum": 160
    ///             },
    ///             "close": {
    ///                 "act_hours__sum": 40,
    ///                 "plan_hours__sum": 80
    ///             }
    ///         }
    ///         "2023-01-02": {
    ///             "": {
    ///                 "act_hours__sum": 60,
    ///                 "plan_hours__sum": 90
    ///             },
    ///             "open": {
    ///                 "act_hours__sum": 40,
    ///                 "plan_hours__sum": 60
    ///             },
    ///             "progress": {
    ///                 "act_hours__sum": 20,
    ///                 "plan_hours__sum": 30
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    pub group: Value,
}
