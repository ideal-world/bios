use std::collections::HashMap;

use bios_basic::basic_enumeration::BasicQueryOpKind;
use serde::{Deserialize, Serialize};
use tardis::{
    chrono::{DateTime, Utc},
    serde_json::Value,
    web::poem_openapi,
};

use crate::stats_enumeration::{StatsQueryAggFunKind, StatsQueryTimeWindowKind};

/// 查询指标请求
/// Query Metrics Request
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsQueryMetricsReq {
    /// 事实编码
    /// Fact code
    pub from: String,
    /// 字段列表
    /// List of fields
    pub select: Vec<StatsQueryMetricsSelectReq>,
    /// 是否忽略distinct key
    /// 如果为true或null，则不计算distinct key
    /// 如果为false，则计算distinct key
    /// Ignore distinct key
    /// If true or null, the distinct key will not be counted
    /// If false, the distinct key will be counted
    pub ignore_distinct: Option<bool>,
    /// 维度列表
    /// 顺序与返回的层级有关，内部使用ROLLUP处理
    /// List of grouped fields,
    /// the order is related to the returned hierarchy and is handled internally using ROLLUP
    pub group: Vec<StatsQueryDimensionGroupReq>,
    pub own_paths: Option<Vec<String>>,
    /// 是否忽略group rollup
    /// 如果为true或null，则不计算group rollup
    /// 如果为false，则计算group rollup
    /// Ignore group rollup
    /// If true or null, the group rollup will not be counted
    /// If false, the group rollup will be counted
    pub ignore_group_rollup: Option<bool>,
    /// 过滤条件, 二维数组, 组内AND, 组间OR
    /// Filter conditions, two-dimensional array, OR between groups, AND within groups
    #[oai(rename = "where")]
    pub _where: Option<Vec<Vec<StatsQueryMetricsWhereReq>>>,
    pub dimension_order: Option<Vec<StatsQueryDimensionOrderReq>>,
    /// 排序条件
    /// code和fun必须存在于Select中
    /// Sort conditions
    /// The code and fun must exist in Select
    pub metrics_order: Option<Vec<StatsQueryMetricsOrderReq>>,
    /// 分组排序条件
    /// code和fun必须存在于Select中
    /// Sort conditions
    /// The code and fun must exist in Select
    pub group_order: Option<Vec<StatsQueryDimensionGroupOrderReq>>,
    pub group_agg: Option<bool>,
    /// 分组后的过滤条件
    /// code和fun必须存在于Select中
    /// Filter conditions after group
    /// The code and fun must exist in Select
    pub having: Option<Vec<StatsQueryMetricsHavingReq>>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub limit: Option<u32>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsQueryMetricsSelectReq {
    /// 度量字段编码
    /// Measure column key
    pub code: String,
    /// 聚合函数
    /// Aggregate function
    pub fun: StatsQueryAggFunKind,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsQueryDimensionGroupReq {
    /// 维度字段编码
    /// Dimension column key
    pub code: String,
    /// 时间窗口函数
    /// Time window function
    pub time_window: Option<StatsQueryTimeWindowKind>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsQueryMetricsWhereReq {
    /// 度量字段编码
    /// Dimension or measure column key
    pub code: String,
    /// 操作符
    /// Operator
    pub op: BasicQueryOpKind,
    /// 值
    /// Value
    pub value: Value,
    /// 时间窗口函数
    /// Time window function
    pub time_window: Option<StatsQueryTimeWindowKind>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsQueryMetricsOrderReq {
    /// 度量字段编码
    /// Measure column key
    pub code: String,
    pub fun: StatsQueryAggFunKind,
    /// 排序方向
    /// Sort direction
    pub asc: bool,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsQueryDimensionGroupOrderReq {
    /// 维度字段编码
    /// Dimension column key
    pub code: String,
    /// 时间窗口函数
    /// Time window function
    pub time_window: Option<StatsQueryTimeWindowKind>,
    /// 排序方向
    /// Sort direction
    pub asc: bool,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsQueryDimensionOrderReq {
    /// 维度字段编码
    /// Dimension column key
    pub code: String,
    /// 排序方向
    /// Sort direction
    pub asc: bool,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsQueryMetricsHavingReq {
    /// 度量字段编码
    /// Measure Column key
    pub code: String,
    /// 聚合函数
    /// Aggregate function
    pub fun: StatsQueryAggFunKind,
    /// 操作符
    /// Operator
    pub op: BasicQueryOpKind,
    /// 值
    /// Value
    pub value: Value,
}

/// 查询指标响应
/// Query Metrics Response
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsQueryMetricsResp {
    /// 事实编码
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

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsQueryStatementReq {
    /// 字段列表
    /// List of fields
    pub select: Vec<StatsQueryStatementSelectReq>,
    /// 维度列表
    /// 顺序与返回的层级有关，内部使用ROLLUP处理
    /// List of grouped fields,
    /// the order is related to the returned hierarchy and is handled internally using ROLLUP
    pub group: Vec<StatsQueryDimensionGroupReq>,
    pub own_paths: Option<Vec<String>>,
    /// 是否忽略group rollup
    /// 如果为true或null，则不计算group rollup
    /// 如果为false，则计算group rollup
    /// Ignore group rollup
    /// If true or null, the group rollup will not be counted
    /// If false, the group rollup will be counted
    pub ignore_group_rollup: Option<bool>,
    /// 过滤条件, 二维数组, 组内AND, 组间OR
    /// Filter conditions, two-dimensional array, OR between groups, AND within groups
    #[oai(rename = "where")]
    pub _where: Option<Vec<Vec<StatsQueryStatementWhereReq>>>,
    pub dimension_order: Option<Vec<StatsQueryDimensionOrderReq>>,
    /// 排序条件
    /// code和fun必须存在于Select中
    /// Sort conditions
    /// The code and fun must exist in Select
    pub metrics_order: Option<Vec<StatsQueryMetricsOrderReq>>,
    /// 分组排序条件
    /// code和fun必须存在于Select中
    /// Sort conditions
    /// The code and fun must exist in Select
    pub group_order: Option<Vec<StatsQueryDimensionGroupOrderReq>>,
    pub group_agg: Option<bool>,
    /// 分组后的过滤条件
    /// code和fun必须存在于Select中
    /// Filter conditions after group
    /// The code and fun must exist in Select
    pub having: Option<Vec<StatsQueryStatementHavingReq>>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub limit: Option<u32>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsQueryStatementSelectReq {
    /// 事实编码
    /// Fact code
    pub from: String,
    /// 度量字段编码
    /// Measure column key
    pub code: String,
    /// 聚合函数
    /// Aggregate function
    pub fun: StatsQueryAggFunKind,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsQueryStatementWhereReq {
    /// 事实编码
    /// Fact code
    pub from: String,
    /// 度量字段编码
    /// Dimension or measure column key
    pub code: String,
    /// 操作符
    /// Operator
    pub op: BasicQueryOpKind,
    /// 值
    /// Value
    pub value: Value,
    /// 时间窗口函数
    /// Time window function
    pub time_window: Option<StatsQueryTimeWindowKind>,
}
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsQueryStatementHavingReq {
    /// 事实编码
    /// Fact code
    pub from: String,
    /// 度量字段编码
    /// Measure Column key
    pub code: String,
    /// 聚合函数
    /// Aggregate function
    pub fun: StatsQueryAggFunKind,
    /// 操作符
    /// Operator
    pub op: BasicQueryOpKind,
    /// 值
    /// Value
    pub value: Value,
}

/// 查询指标响应
/// Query Metrics Response
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsQueryStatementResp {
    pub group: Value,
}
